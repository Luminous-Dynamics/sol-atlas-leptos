// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Strict offline parser for UN Comtrade preview responses.
//!
//! This module performs no HTTP requests and accepts no API keys. The caller
//! supplies the exact query context, source snapshot identity, retrieval time,
//! and a pinned area-code classification table. Response rows are retained
//! verbatim as JSON values alongside normalized observations.
//!
//! Preview data is always marked as incomplete/unproven coverage. Reaching a
//! request/preview ceiling is explicitly called possibly truncated; fewer
//! returned rows do not prove that a broader query or dataset is complete.
//! This parser is not a trade-misinvoicing estimator and does not infer crime.

use crate::economic_intelligence::{
    compare_mirror_pair, MirrorPairStatus, ObservationSourceCoverage, TradeAreaKind, TradeFlow,
    TradeObservation, TRADE_OBSERVATION_SCHEMA_VERSION,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fmt;

/// Current preview ceiling documented by the UN Comtrade public preview API.
pub const UN_COMTRADE_PREVIEW_MAX_RECORDS: usize = 500;
pub const UN_COMTRADE_SOURCE_ID: &str = "un-comtrade-preview-api";
pub const UN_COMTRADE_GEOGRAPHY_CODE_SYSTEM: &str = "UN_M49";
pub const UN_COMTRADE_DIMENSIONS_VERSION: &str = "uncomtrade-dimensions/v1";

/// Query dimensions expected for every row in one response.
///
/// Values are strings so leading zeroes and source code tokens are not lost.
/// `classification_revision` is provenance supplied by a pinned metadata
/// mapping; it is not guessed from `classification_code`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnComtradePreviewQuery {
    /// UN Comtrade record type. This adapter currently qualifies goods (C) only.
    pub type_code: String,
    /// Reporting interval: annual (A) or monthly (M).
    pub frequency_code: String,
    /// Classification family requested from the API (e.g. HS).
    pub classification_code: String,
    /// Effective row-level classification code returned by the API (e.g. H6).
    pub record_classification_code: String,
    /// Pinned human-reviewed meaning/revision associated with the row-level code.
    pub classification_revision: String,
    /// Source-reviewed valuation basis for primaryValue; never guessed from flow.
    /// Use e.g. known:fob, known:cif, or unknown, grounded in metadata.
    pub valuation_basis_signature: String,
    pub period: String,
    pub reporter_code: String,
    pub partner_code: String,
    /// UN Comtrade final-trade flow codes: X for exports, M for imports.
    pub flow_code: String,
    pub commodity_code: String,
    /// Maximum records requested from the preview API (1..=500).
    pub max_records: usize,
}

/// Non-secret provenance and reference data required to interpret one response.
#[derive(Debug, Clone)]
pub struct UnComtradePreviewContext {
    pub query: UnComtradePreviewQuery,
    /// Digest or immutable ID assigned by the ingestion layer to the raw response bytes.
    pub source_snapshot_id: String,
    /// Identity/version of the reporter/partner area reference table.
    pub area_reference_snapshot_id: String,
    /// Retrieval time in a caller-supplied, explicit format (prefer RFC 3339).
    pub retrieved_at: String,
    /// Map from UN Comtrade area code to a reviewed, versioned area classification.
    /// Unknown codes are rejected; names/descriptions are never used to infer area type.
    pub area_kinds: BTreeMap<String, TradeAreaKind>,
}

/// Whether the returned preview could be used as evidence of complete coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewCoverage {
    /// API response returned no records for the exact query.
    EmptyResponse,
    /// Fewer than the requested maximum were returned, but total-query coverage
    /// has not been independently proven.
    BelowRequestedLimitCompletenessUnproven,
    /// The source count exceeds the rows carried in the payload or the requested
    /// record ceiling was reached; completeness cannot be assumed.
    PossiblyTruncated,
    /// Some rows were rejected; completeness cannot be assumed.
    RowsRejectedCompletenessUnproven,
    /// Truncation is possible and one or more rows were rejected.
    PossiblyTruncatedWithRejectedRows,
}

impl PreviewCoverage {
    /// Preview output never proves that the full underlying trade universe was returned.
    pub fn completeness_proven(self) -> bool {
        false
    }

    pub fn may_be_truncated(self) -> bool {
        matches!(
            self,
            Self::PossiblyTruncated | Self::PossiblyTruncatedWithRejectedRows
        )
    }
}

/// Why a row could not be normalized. Rejected rows are surfaced, never silently dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnComtradeRowRejection {
    NotAnObject,
    MissingField(String),
    InvalidFieldType(String),
    InvalidFieldValue(String),
    QueryDimensionMismatch(String),
    /// The source marks a value aggregated above the requested commodity level.
    CommodityAggregationAboveRequestedLevel,
    UnknownAreaCode(String),
    DuplicateRecordKey(String),
    UnsupportedFlowCode(String),
}

/// A rejected source row plus its index into the original response's `data` array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedUnComtradeRow {
    pub row_index: usize,
    pub reason: UnComtradeRowRejection,
}

/// A normalized observation paired with the original record object.
#[derive(Debug, Clone, PartialEq)]
pub struct UnComtradeRecord {
    pub observation: TradeObservation,
    pub raw_record: Value,
}

/// Parsed preview response and explicit coverage/provenance metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUnComtradePreview {
    pub source_snapshot_id: String,
    pub area_reference_snapshot_id: String,
    pub retrieved_at: String,
    pub query: UnComtradePreviewQuery,
    pub elapsed_time: Option<String>,
    /// Count value reported by the source envelope. Its semantics may differ
    /// from payload row count across endpoint variants.
    pub source_reported_count: usize,
    /// Number of row objects actually included in this response payload.
    pub rows_returned: usize,
    pub records: Vec<UnComtradeRecord>,
    pub rejected_rows: Vec<RejectedUnComtradeRow>,
    pub coverage: PreviewCoverage,
}

/// Fail-closed response-level errors. No records are returned after one of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnComtradePreviewError {
    InvalidContext(String),
    InvalidJson(String),
    EnvelopeSchema(String),
    ApiReportedError(String),
    CountLessThanRowsReturned { reported: usize, actual: usize },
    MoreRecordsThanRequested { requested: usize, actual: usize },
}

impl fmt::Display for UnComtradePreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContext(message) => write!(f, "invalid UN Comtrade context: {message}"),
            Self::InvalidJson(message) => write!(f, "invalid UN Comtrade JSON: {message}"),
            Self::EnvelopeSchema(message) => {
                write!(f, "UN Comtrade envelope schema error: {message}")
            }
            Self::ApiReportedError(message) => {
                write!(f, "UN Comtrade API reported an error: {message}")
            }
            Self::CountLessThanRowsReturned { reported, actual } => write!(
                f,
                concat!(
                    "UN Comtrade reported count is smaller than returned rows: ",
                    "count={reported}, rows={actual}"
                )
            ),
            Self::MoreRecordsThanRequested { requested, actual } => write!(
                f,
                concat!(
                    "UN Comtrade response exceeded requested limit: ",
                    "requested={requested}, rows={actual}"
                )
            ),
        }
    }
}

impl Error for UnComtradePreviewError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewEnvelope {
    #[serde(rename = "elapsedTime", default)]
    elapsed_time: Option<Value>,
    count: usize,
    data: Vec<Value>,
    #[serde(default)]
    error: Option<Value>,
}

/// Parse a frozen UN Comtrade preview response using its exact request context.
///
/// The parser only normalizes records whose principal query dimensions match
/// the requested query. Any individual row that cannot be normalized is added
/// to `rejected_rows`; its original raw record remains available to the caller
/// because it is retained in the rejected row only as an index into the exact
/// response snapshot. Callers must persist the original response bytes associated
/// with `source_snapshot_id`.
pub fn parse_un_comtrade_preview(
    response_json: &str,
    context: &UnComtradePreviewContext,
) -> Result<ParsedUnComtradePreview, UnComtradePreviewError> {
    validate_context(context)?;

    let parsed: Value = serde_json::from_str(response_json)
        .map_err(|error| UnComtradePreviewError::InvalidJson(error.to_string()))?;
    let object = parsed.as_object().ok_or_else(|| {
        UnComtradePreviewError::EnvelopeSchema(
            "top-level response must be a JSON object".to_string(),
        )
    })?;
    const ENVELOPE_KEYS: [&str; 4] = ["elapsedTime", "count", "data", "error"];
    if let Some(unknown) = object.keys().find(|key| !ENVELOPE_KEYS.contains(&key.as_str())) {
        return Err(UnComtradePreviewError::EnvelopeSchema(format!(
            "unsupported top-level field {unknown:?}"
        )));
    }
    for required in ["count", "data"] {
        if !object.contains_key(required) {
            return Err(UnComtradePreviewError::EnvelopeSchema(format!(
                "required top-level field {required:?} is missing"
            )));
        }
    }
    let envelope: PreviewEnvelope = serde_json::from_value(parsed)
        .map_err(|error| UnComtradePreviewError::EnvelopeSchema(error.to_string()))?;

    if let Some(error) = envelope.error.as_ref().filter(|value| api_error_present(value)) {
        return Err(UnComtradePreviewError::ApiReportedError(error.to_string()));
    }

    // Some API modes may report the underlying match count while returning
    // only a capped page. Keep the source count and payload length separate;
    // a larger count is a truncation signal, not a parser failure.
    if envelope.count < envelope.data.len() {
        return Err(UnComtradePreviewError::CountLessThanRowsReturned {
            reported: envelope.count,
            actual: envelope.data.len(),
        });
    }
    if envelope.data.len() > context.query.max_records {
        return Err(UnComtradePreviewError::MoreRecordsThanRequested {
            requested: context.query.max_records,
            actual: envelope.data.len(),
        });
    }

    let source_reported_count = envelope.count;
    let rows_returned = envelope.data.len();
    let mut records = Vec::new();
    let mut rejected_rows = Vec::new();
    let mut seen_record_ids = HashSet::new();
    for (row_index, raw_record) in envelope.data.into_iter().enumerate() {
        match normalize_record(&raw_record, context) {
            Ok(observation) => {
                if !seen_record_ids.insert(observation.record_id.clone()) {
                    rejected_rows.push(RejectedUnComtradeRow {
                        row_index,
                        reason: UnComtradeRowRejection::DuplicateRecordKey(
                            observation.record_id,
                        ),
                    });
                } else {
                    records.push(UnComtradeRecord {
                        observation,
                        raw_record,
                    });
                }
            }
            Err(reason) => rejected_rows.push(RejectedUnComtradeRow { row_index, reason }),
        }
    }

    let possibly_truncated =
        rows_returned >= context.query.max_records || source_reported_count > rows_returned;
    let coverage = if rows_returned == 0 && source_reported_count == 0 {
        PreviewCoverage::EmptyResponse
    } else if !rejected_rows.is_empty() && possibly_truncated {
        PreviewCoverage::PossiblyTruncatedWithRejectedRows
    } else if !rejected_rows.is_empty() {
        PreviewCoverage::RowsRejectedCompletenessUnproven
    } else if possibly_truncated {
        PreviewCoverage::PossiblyTruncated
    } else {
        PreviewCoverage::BelowRequestedLimitCompletenessUnproven
    };

    let observation_coverage = match coverage {
        PreviewCoverage::EmptyResponse
        | PreviewCoverage::BelowRequestedLimitCompletenessUnproven => {
            ObservationSourceCoverage::PreviewCompletenessUnproven
        }
        PreviewCoverage::PossiblyTruncated => ObservationSourceCoverage::PreviewPossiblyTruncated,
        PreviewCoverage::RowsRejectedCompletenessUnproven => {
            ObservationSourceCoverage::PreviewRowsRejected
        }
        PreviewCoverage::PossiblyTruncatedWithRejectedRows => {
            ObservationSourceCoverage::PreviewPossiblyTruncatedWithRejectedRows
        }
    };
    for record in &mut records {
        record.observation.source_coverage = observation_coverage;
    }

    Ok(ParsedUnComtradePreview {
        source_snapshot_id: context.source_snapshot_id.clone(),
        area_reference_snapshot_id: context.area_reference_snapshot_id.clone(),
        retrieved_at: context.retrieved_at.clone(),
        query: context.query.clone(),
        elapsed_time: envelope.elapsed_time.as_ref().map(value_to_text),
        source_reported_count,
        rows_returned,
        records,
        rejected_rows,
        coverage,
    })
}

fn validate_context(
    context: &UnComtradePreviewContext,
) -> Result<(), UnComtradePreviewError> {
    let query = &context.query;
    for (name, value) in [
        ("type_code", query.type_code.as_str()),
        ("frequency_code", query.frequency_code.as_str()),
        ("classification_code", query.classification_code.as_str()),
        (
            "record_classification_code",
            query.record_classification_code.as_str(),
        ),
        ("classification_revision", query.classification_revision.as_str()),
        (
            "valuation_basis_signature",
            query.valuation_basis_signature.as_str(),
        ),
        ("period", query.period.as_str()),
        ("reporter_code", query.reporter_code.as_str()),
        ("partner_code", query.partner_code.as_str()),
        ("commodity_code", query.commodity_code.as_str()),
        ("source_snapshot_id", context.source_snapshot_id.as_str()),
        (
            "area_reference_snapshot_id",
            context.area_reference_snapshot_id.as_str(),
        ),
        ("retrieved_at", context.retrieved_at.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(UnComtradePreviewError::InvalidContext(format!(
                "{name} must be non-empty"
            )));
        }
    }
    if query.type_code != "C" {
        return Err(UnComtradePreviewError::InvalidContext(
            "this adapter supports goods type_code C only; services require a separately qualified schema"
                .to_string(),
        ));
    }
    if !matches!(query.frequency_code.as_str(), "A" | "M") {
        return Err(UnComtradePreviewError::InvalidContext(
            "frequency_code must be A (annual) or M (monthly)".to_string(),
        ));
    }
    let period_bytes = query.period.as_bytes();
    let expected_period_len = if query.frequency_code == "A" { 4 } else { 6 };
    let valid_month = period_bytes
        .get(4..6)
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|month| month.parse::<u8>().ok())
        .is_some_and(|month| (1..=12).contains(&month));
    if period_bytes.len() != expected_period_len
        || !period_bytes.iter().all(u8::is_ascii_digit)
        || (query.frequency_code == "M" && !valid_month)
    {
        return Err(UnComtradePreviewError::InvalidContext(
            "period must be YYYY for annual data or YYYYMM with month 01..12 for monthly data"
                .to_string(),
        ));
    }
    if query.max_records == 0 || query.max_records > UN_COMTRADE_PREVIEW_MAX_RECORDS {
        return Err(UnComtradePreviewError::InvalidContext(format!(
            "max_records must be in 1..={UN_COMTRADE_PREVIEW_MAX_RECORDS}"
        )));
    }
    if !matches!(query.flow_code.as_str(), "X" | "M") {
        return Err(UnComtradePreviewError::InvalidContext(
            "flow_code must be X (exports) or M (imports)".to_string(),
        ));
    }
    if !context
        .area_kinds
        .contains_key(query.reporter_code.as_str())
        || !context
            .area_kinds
            .contains_key(query.partner_code.as_str())
    {
        return Err(UnComtradePreviewError::InvalidContext(
            "request reporter and partner must exist in the pinned area reference table"
                .to_string(),
        ));
    }
    Ok(())
}

fn api_error_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !text.trim().is_empty(),
        _ => true,
    }
}

fn normalize_record(
    raw: &Value,
    context: &UnComtradePreviewContext,
) -> Result<TradeObservation, UnComtradeRowRejection> {
    let object = raw
        .as_object()
        .ok_or(UnComtradeRowRejection::NotAnObject)?;
    let query = &context.query;

    let type_code = required_scalar(object, "typeCode")?;
    let frequency_code = required_scalar(object, "freqCode")?;
    let period = required_scalar(object, "period")?;
    let reporter_code = required_scalar(object, "reporterCode")?;
    let partner_code = required_scalar(object, "partnerCode")?;
    let flow_code = required_scalar(object, "flowCode")?;
    let commodity_code = required_scalar(object, "cmdCode")?;
    let record_classification_code = required_scalar(object, "classificationCode")?;
    let is_aggregated_above = required_bool(object, "isAggrAbove")?;
    let primary_value = required_number_token(object, "primaryValue")?;

    // These are required dimensions because they can alter the meaning/grain
    // of the reported value even when the main bilateral query fields match.
    let second_partner_code = required_scalar(object, "partner2Code")?;
    let transport_mode_code = required_scalar(object, "motCode")?;
    let customs_procedure_code = required_scalar(object, "customsCode")?;
    let reported_flag = required_bool(object, "isReported")?.to_string();
    let legacy_estimation_flag =
        required_nonnegative_integer(object, "legacyEstimationFlag")?;

    let reporter_kind = context
        .area_kinds
        .get(&reporter_code)
        .copied()
        .ok_or_else(|| UnComtradeRowRejection::UnknownAreaCode(reporter_code.clone()))?;
    let partner_kind = context
        .area_kinds
        .get(&partner_code)
        .copied()
        .ok_or_else(|| UnComtradeRowRejection::UnknownAreaCode(partner_code.clone()))?;

    for (name, actual, expected) in [
        ("typeCode", type_code.as_str(), query.type_code.as_str()),
        ("freqCode", frequency_code.as_str(), query.frequency_code.as_str()),
        ("period", period.as_str(), query.period.as_str()),
        (
            "reporterCode",
            reporter_code.as_str(),
            query.reporter_code.as_str(),
        ),
        (
            "partnerCode",
            partner_code.as_str(),
            query.partner_code.as_str(),
        ),
        ("flowCode", flow_code.as_str(), query.flow_code.as_str()),
        (
            "cmdCode",
            commodity_code.as_str(),
            query.commodity_code.as_str(),
        ),
        (
            "classificationCode",
            record_classification_code.as_str(),
            query.record_classification_code.as_str(),
        ),
    ] {
        if actual != expected {
            return Err(UnComtradeRowRejection::QueryDimensionMismatch(
                name.to_string(),
            ));
        }
    }

    let flow = match flow_code.as_str() {
        "X" => TradeFlow::Export,
        "M" => TradeFlow::Import,
        _ => return Err(UnComtradeRowRejection::UnsupportedFlowCode(flow_code)),
    };
    if is_aggregated_above {
        return Err(UnComtradeRowRejection::CommodityAggregationAboveRequestedLevel);
    }
    let dimension_signature = format!(
        concat!(
            "{UN_COMTRADE_DIMENSIONS_VERSION}|partner2={}|",
            "mot={}|customs={}"
        ),
        second_partner_code,
        transport_mode_code,
        customs_procedure_code
    );
    let source_state_signature = format!(
        concat!(
            "uncomtrade-source-state/v1|reported={reported_flag}|",
            "legacy_estimation={legacy_estimation_flag}"
        )
    );
    // Malformed optional fields are rejected, not silently collapsed into None.
    let quantity = optional_f64(object, "qty")?;
    let quantity_unit = optional_scalar(object, "qtyUnitAbbr")?;
    let record_id = format!(
        concat!(
            "comtrade:{}/{}/{}/{}/{}/{}/{}/{}/row_classification={}/",
            "partner2={}/mot={}/customs={}/reported={}/legacy_estimation={}"
        ),
        query.type_code,
        query.frequency_code,
        query.classification_code,
        period,
        reporter_code,
        partner_code,
        flow_code,
        commodity_code,
        record_classification_code,
        second_partner_code,
        transport_mode_code,
        customs_procedure_code,
        reported_flag,
        legacy_estimation_flag
    );

    Ok(TradeObservation {
        schema_version: TRADE_OBSERVATION_SCHEMA_VERSION,
        record_id,
        source_id: UN_COMTRADE_SOURCE_ID.to_string(),
        source_snapshot_id: context.source_snapshot_id.clone(),
        area_reference_snapshot_id: context.area_reference_snapshot_id.clone(),
        source_record_id: None,
        source_url: None,
        reporter: reporter_code,
        reporter_kind,
        partner: partner_code,
        partner_kind,
        geography_code_system: UN_COMTRADE_GEOGRAPHY_CODE_SYSTEM.to_string(),
        series_signature: format!(
            "uncomtrade-series/v2|type={}|frequency={}|classification={}|row_classification={}",
            query.type_code,
            query.frequency_code,
            query.classification_code,
            record_classification_code
        ),
        period,
        disaggregation_signature: dimension_signature,
        source_state_signature,
        source_coverage: ObservationSourceCoverage::PreviewCompletenessUnproven,
        commodity_code,
        classification_revision: query.classification_revision.clone(),
        flow,
        trade_value_source_text: Some(primary_value),
        currency: "USD".to_string(),
        valuation_basis_signature: query.valuation_basis_signature.clone(),
        value_unit: "USD".to_string(),
        quantity,
        quantity_unit,
        // The API row reference period is not the publication/revision time.
        // Preserve it separately as period; leave source_published_at absent
        // until qualified metadata provides a distinct publication timestamp.
        source_published_at: None,
        retrieved_at: context.retrieved_at.clone(),
    })
}

fn required_scalar(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<String, UnComtradeRowRejection> {
    object
        .get(key)
        .ok_or_else(|| UnComtradeRowRejection::MissingField(key.to_string()))
        .and_then(|value| {
            scalar_to_text(value).ok_or_else(|| {
                UnComtradeRowRejection::InvalidFieldType(key.to_string())
            })
        })
}

fn required_bool(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<bool, UnComtradeRowRejection> {
    match object.get(key) {
        None => Err(UnComtradeRowRejection::MissingField(key.to_string())),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(UnComtradeRowRejection::InvalidFieldType(key.to_string())),
    }
}

fn required_number_token(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<String, UnComtradeRowRejection> {
    match object.get(key) {
        None => Err(UnComtradeRowRejection::MissingField(key.to_string())),
        Some(Value::Number(number)) => Ok(number.to_string()),
        Some(_) => Err(UnComtradeRowRejection::InvalidFieldType(key.to_string())),
    }
}

fn required_nonnegative_integer(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<String, UnComtradeRowRejection> {
    match object.get(key) {
        None => Err(UnComtradeRowRejection::MissingField(key.to_string())),
        Some(Value::Number(number)) => {
            let token = number.to_string();
            if token.parse::<u64>().is_ok() {
                Ok(token)
            } else {
                // Require an integer JSON token, not 1.0 or 1e0.
                Err(UnComtradeRowRejection::InvalidFieldValue(key.to_string()))
            }
        }
        Some(_) => Err(UnComtradeRowRejection::InvalidFieldType(key.to_string())),
    }
}

fn optional_scalar(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<Option<String>, UnComtradeRowRejection> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(UnComtradeRowRejection::InvalidFieldType(key.to_string())),
    }
}

fn optional_f64(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<Option<f64>, UnComtradeRowRejection> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| UnComtradeRowRejection::InvalidFieldValue(key.to_string())),
        Some(_) => Err(UnComtradeRowRejection::InvalidFieldType(key.to_string())),
    }
}

fn scalar_to_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        // Boolean values are not valid substitute identifiers or code tokens.
        _ => None,
    }
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(max_records: usize) -> UnComtradePreviewContext {
        let query = UnComtradePreviewQuery {
            type_code: "C".to_string(),
            frequency_code: "A".to_string(),
            classification_code: "HS".to_string(),
            record_classification_code: "H6".to_string(),
            classification_revision: "HS-2022".to_string(),
            valuation_basis_signature: "known:fob".to_string(),
            period: "2025".to_string(),
            reporter_code: "842".to_string(),
            partner_code: "156".to_string(),
            flow_code: "X".to_string(),
            commodity_code: "010121".to_string(),
            max_records,
        };
        let area_kinds = [
            ("842".to_string(), TradeAreaKind::ReportingArea),
            ("156".to_string(), TradeAreaKind::ReportingArea),
            ("0".to_string(), TradeAreaKind::Aggregate),
        ]
        .into_iter()
        .collect();

        UnComtradePreviewContext {
            query,
            source_snapshot_id: "synthetic-comtrade-response-v1".to_string(),
            area_reference_snapshot_id: "synthetic-area-table-v1".to_string(),
            retrieved_at: "2026-02-01T00:00:00Z".to_string(),
            area_kinds,
        }
    }

    fn row() -> Value {
        serde_json::json!({
            "typeCode": "C",
            "freqCode": "A",
            "period": "2025",
            "reporterCode": 842,
            "reporterISO": "USA",
            "partnerCode": 156,
            "partnerISO": "CHN",
            "flowCode": "X",
            "cmdCode": "010121",
            "classificationCode": "H6",
            "isAggrAbove": false,
            "partner2Code": 0,
            "motCode": "0",
            "customsCode": "C00",
            "primaryValue": 9007199254740993,
            "isReported": true,
            "legacyEstimationFlag": 0,
            "netWgt": 12.5,
            "qtyUnitAbbr": "kg",
            "refYear": 2025,
            "anExtraRecordField": "retained in raw_record"
        })
    }

    fn response(rows: Vec<Value>, reported_count: usize) -> String {
        serde_json::json!({
            "elapsedTime": "0.01 sec",
            "count": reported_count,
            "data": rows,
            "error": null
        })
        .to_string()
    }

    #[test]
    fn parses_expected_preview_row_and_retains_raw_record() {
        let raw = include_str!("../tests/fixtures/synthetic_un_comtrade_preview_v1.json");
        let parsed = parse_un_comtrade_preview(raw, &context(10)).unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert!(parsed.rejected_rows.is_empty());
        assert_eq!(
            parsed.records[0]
                .observation
                .trade_value_source_text
                .as_deref(),
            Some("9007199254740993")
        );
        assert_eq!(
            parsed.records[0].observation.reporter,
            "842"
        );
        assert_eq!(
            parsed.records[0].observation.disaggregation_signature,
            "uncomtrade-dimensions/v1|partner2=0|mot=0|customs=C00"
        );
        assert_eq!(
            parsed.records[0].observation.source_state_signature,
            "uncomtrade-source-state/v1|reported=true|legacy_estimation=0"
        );
        assert_eq!(
            parsed.records[0]
                .raw_record
                .get("reporterDesc")
                .and_then(Value::as_str),
            Some("Synthetic fixture reporter A; identifier reused for schema testing")
        );
        assert!(!parsed.coverage.completeness_proven());
        assert_eq!(
            parsed.coverage,
            PreviewCoverage::BelowRequestedLimitCompletenessUnproven
        );
        assert_eq!(
            parsed.records[0].observation.source_coverage,
            ObservationSourceCoverage::PreviewCompletenessUnproven
        );
    }

    #[test]
    fn source_count_larger_than_payload_marks_possible_truncation() {
        let parsed = parse_un_comtrade_preview(&response(vec![row()], 2), &context(10)).unwrap();
        assert_eq!(parsed.source_reported_count, 2);
        assert_eq!(parsed.rows_returned, 1);
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.coverage, PreviewCoverage::PossiblyTruncated);
        assert!(parsed.coverage.may_be_truncated());
        assert_eq!(
            parsed.records[0].observation.source_coverage,
            ObservationSourceCoverage::PreviewPossiblyTruncated
        );
    }

    #[test]
    fn reaching_requested_limit_is_marked_as_possibly_truncated() {
        let mut second = row();
        second["motCode"] = Value::String("1".to_string());
        let parsed = parse_un_comtrade_preview(&response(vec![row(), second], 2), &context(2))
            .unwrap();
        assert_eq!(parsed.records.len(), 2);
        assert!(parsed.rejected_rows.is_empty());
        assert_eq!(
            parsed.coverage,
            PreviewCoverage::PossiblyTruncated
        );
        assert!(parsed.coverage.may_be_truncated());
        assert!(!parsed.coverage.completeness_proven());
        assert_eq!(
            parsed.records[0].observation.source_coverage,
            ObservationSourceCoverage::PreviewPossiblyTruncated
        );
    }

    #[test]
    fn duplicate_source_keys_are_quarantined() {
        let parsed = parse_un_comtrade_preview(&response(vec![row(), row()], 2), &context(10))
            .unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.rejected_rows.len(), 1);
        assert!(matches!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::DuplicateRecordKey(_)
        ));
        assert_eq!(
            parsed.coverage,
            PreviewCoverage::RowsRejectedCompletenessUnproven
        );
    }

    #[test]
    fn parser_preserves_caller_reviewed_valuation_basis_without_inference() {
        let parsed = parse_un_comtrade_preview(
            &response(vec![row()], 1),
            &context(10),
        )
        .unwrap();
        assert_eq!(
            parsed.records[0].observation.valuation_basis_signature,
            "known:fob"
        );
    }

    #[test]
    fn preview_rows_cannot_be_used_for_gap_calculations_without_complete_coverage() {
        let exporter_raw = row();
        let mut importer_raw = row();
        importer_raw["reporterCode"] = Value::from(156);
        importer_raw["partnerCode"] = Value::from(842);
        importer_raw["flowCode"] = Value::String("M".to_string());
        importer_raw["isReported"] = Value::Bool(false);
        importer_raw["legacyEstimationFlag"] = Value::from(1);

        let exporter_context = context(10);
        let mut importer_context = context(10);
        importer_context.query.reporter_code = "156".to_string();
        importer_context.query.partner_code = "842".to_string();
        importer_context.query.flow_code = "M".to_string();
        importer_context.query.valuation_basis_signature = "known:cif".to_string();

        let exporter =
            parse_un_comtrade_preview(&response(vec![exporter_raw], 1), &exporter_context)
                .unwrap();
        let importer =
            parse_un_comtrade_preview(&response(vec![importer_raw], 1), &importer_context)
                .unwrap();
        let result = compare_mirror_pair(
            &[exporter.records[0].observation.clone()],
            &[importer.records[0].observation.clone()],
        );
        assert_eq!(result.status, MirrorPairStatus::SourceCoverageInsufficient);
        assert_eq!(result.absolute_difference_source_text, None);
    }

    #[test]
    fn query_mismatched_rows_are_rejected_not_silently_used() {
        let mut mismatched = row();
        mismatched["partnerCode"] = Value::from(0);
        let parsed = parse_un_comtrade_preview(&response(vec![mismatched], 1), &context(10))
            .unwrap();
        assert_eq!(parsed.records.len(), 0);
        assert_eq!(parsed.rejected_rows.len(), 1);
        assert_eq!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::QueryDimensionMismatch("partnerCode".to_string())
        );
        assert_eq!(
            parsed.coverage,
            PreviewCoverage::RowsRejectedCompletenessUnproven
        );
    }

    #[test]
    fn row_classification_mismatch_is_rejected() {
        let mut wrong_revision = row();
        wrong_revision["classificationCode"] = Value::String("H5".to_string());
        let parsed =
            parse_un_comtrade_preview(&response(vec![wrong_revision], 1), &context(10)).unwrap();
        assert_eq!(parsed.records.len(), 0);
        assert_eq!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::QueryDimensionMismatch("classificationCode".to_string())
        );
    }

    #[test]
    fn value_aggregated_above_requested_commodity_is_rejected() {
        let mut aggregated = row();
        aggregated["isAggrAbove"] = Value::Bool(true);
        let parsed = parse_un_comtrade_preview(&response(vec![aggregated], 1), &context(10))
            .unwrap();
        assert_eq!(parsed.records.len(), 0);
        assert_eq!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::CommodityAggregationAboveRequestedLevel
        );
    }

    #[test]
    fn unknown_area_code_is_rejected() {
        let mut unknown = row();
        // This row does not match the query and also uses a code absent from
        // the pinned reference table. Unknown identity is reported explicitly.
        unknown["reporterCode"] = Value::from(9999);
        let ctx = context(10);
        let parsed = parse_un_comtrade_preview(&response(vec![unknown], 1), &ctx).unwrap();
        assert_eq!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::UnknownAreaCode("9999".to_string())
        );
    }

    #[test]
    fn count_mismatch_and_unknown_envelope_fields_fail_closed() {
        assert_eq!(
            parse_un_comtrade_preview(&response(vec![row()], 0), &context(10)),
            Err(UnComtradePreviewError::CountLessThanRowsReturned {
                reported: 0,
                actual: 1
            })
        );

        let mut value: Value = serde_json::from_str(&response(vec![row()], 1)).unwrap();
        value["unexpectedEnvelopeField"] = Value::Bool(true);
        let result = parse_un_comtrade_preview(&value.to_string(), &context(10));
        assert!(matches!(
            result,
            Err(UnComtradePreviewError::EnvelopeSchema(_))
        ));
    }

    #[test]
    fn rejects_string_flags_and_non_numeric_primary_values() {
        let ctx = context(10);
        for (field, bad_value) in [
            ("isAggrAbove", Value::String("false".to_string())),
            ("isReported", Value::String("true".to_string())),
            ("primaryValue", Value::String("9007199254740993".to_string())),
        ] {
            let mut candidate = row();
            candidate[field] = bad_value;
            let parsed =
                parse_un_comtrade_preview(&response(vec![candidate], 1), &ctx).unwrap();
            assert!(parsed.records.is_empty(), "{field} must be type-checked");
            assert_eq!(
                parsed.rejected_rows,
                vec![RejectedUnComtradeRow {
                    row_index: 0,
                    reason: UnComtradeRowRejection::InvalidFieldType(field.to_string()),
                }],
                "{field} should fail with an explicit field-type rejection"
            );
        }
    }

    #[test]
    fn rejects_malformed_optional_quantity_fields() {
        let ctx = context(10);
        for (field, bad_value, reason) in [
            (
                "qty",
                Value::String("12.5".to_string()),
                UnComtradeRowRejection::InvalidFieldType("qty".to_string()),
            ),
            (
                "qtyUnitAbbr",
                Value::from(8),
                UnComtradeRowRejection::InvalidFieldType("qtyUnitAbbr".to_string()),
            ),
        ] {
            let mut candidate = row();
            candidate[field] = bad_value;
            let parsed = parse_un_comtrade_preview(&response(vec![candidate], 1), &ctx).unwrap();
            assert!(parsed.records.is_empty(), "{field} should be rejected");
            assert_eq!(parsed.rejected_rows[0].reason, reason);
        }
    }

    #[test]
    fn nullable_optional_quantity_fields_remain_nullable() {
        let ctx = context(10);
        let mut candidate = row();
        candidate["qty"] = Value::Null;
        candidate["qtyUnitAbbr"] = Value::Null;
        let parsed = parse_un_comtrade_preview(&response(vec![candidate], 1), &ctx).unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].observation.quantity, None);
        assert_eq!(parsed.records[0].observation.quantity_unit, None);
    }

    #[test]
    fn rejects_boolean_dimensions_instead_of_stringifying_them() {
        let ctx = context(10);
        let mut candidate = row();
        candidate["partner2Code"] = Value::Bool(false);
        let parsed = parse_un_comtrade_preview(&response(vec![candidate], 1), &ctx).unwrap();
        assert!(parsed.records.is_empty());
        assert_eq!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::InvalidFieldType("partner2Code".to_string())
        );
    }

    #[test]
    fn rejects_invalid_legacy_estimation_flag_type_and_value() {
        let ctx = context(10);
        for bad_value in [
            Value::String("0".to_string()),
            Value::Bool(false),
            Value::from(1.5),
            Value::from(-1),
        ] {
            let mut candidate = row();
            candidate["legacyEstimationFlag"] = bad_value;
            let parsed =
                parse_un_comtrade_preview(&response(vec![candidate], 1), &ctx).unwrap();
            assert!(parsed.records.is_empty());
            assert_eq!(parsed.rejected_rows.len(), 1);
            assert!(matches!(
                parsed.rejected_rows[0].reason,
                UnComtradeRowRejection::InvalidFieldType(_)
                    | UnComtradeRowRejection::InvalidFieldValue(_)
            ));
        }
    }

    #[test]
    fn rejects_unqualified_type_frequency_and_period_values() {
        let mut ctx = context(10);
        ctx.query.type_code = "S".to_string();
        assert!(matches!(
            parse_un_comtrade_preview(&response(vec![row()], 1), &ctx),
            Err(UnComtradePreviewError::InvalidContext(_))
        ));

        ctx.query.type_code = "C".to_string();
        ctx.query.frequency_code = "Q".to_string();
        assert!(matches!(
            parse_un_comtrade_preview(&response(vec![row()], 1), &ctx),
            Err(UnComtradePreviewError::InvalidContext(_))
        ));

        ctx.query.frequency_code = "M".to_string();
        ctx.query.period = "202513".to_string();
        assert!(matches!(
            parse_un_comtrade_preview(&response(vec![row()], 1), &ctx),
            Err(UnComtradePreviewError::InvalidContext(_))
        ));

        ctx.query.period = "202501".to_string();
        let mut monthly_row = row();
        monthly_row["freqCode"] = Value::String("M".to_string());
        monthly_row["period"] = Value::String("202501".to_string());
        assert!(
            parse_un_comtrade_preview(&response(vec![monthly_row], 1), &ctx).is_ok(),
            "valid monthly period and matching row should parse"
        );
    }

    #[test]
    fn invalid_context_and_api_errors_do_not_return_rows() {
        let mut ctx = context(501);
        assert!(matches!(
            parse_un_comtrade_preview(&response(vec![row()], 1), &ctx),
            Err(UnComtradePreviewError::InvalidContext(_))
        ));

        ctx = context(10);
        let response_with_error = serde_json::json!({
            "elapsedTime": "0.01 sec",
            "count": 0,
            "data": [],
            "error": "synthetic upstream failure"
        });
        assert!(matches!(
            parse_un_comtrade_preview(&response_with_error.to_string(), &ctx),
            Err(UnComtradePreviewError::ApiReportedError(_))
        ));
    }
}
