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
    TradeAreaKind, TradeFlow, TradeObservation, TRADE_OBSERVATION_SCHEMA_VERSION,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
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
    pub type_code: String,
    pub frequency_code: String,
    pub classification_code: String,
    pub classification_revision: String,
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
    /// Returned rows reached the requested maximum and may be truncated.
    AtRequestedLimitPossiblyTruncated,
    /// Some rows were rejected and the returned set is not safe to treat as complete.
    BelowLimitWithRejectedRows,
    /// The limit was reached and one or more rows were rejected.
    AtLimitWithRejectedRows,
}

impl PreviewCoverage {
    /// Preview output never proves that the full underlying trade universe was returned.
    pub fn completeness_proven(self) -> bool {
        false
    }

    pub fn may_be_truncated(self) -> bool {
        matches!(
            self,
            Self::AtRequestedLimitPossiblyTruncated | Self::AtLimitWithRejectedRows
        )
    }
}

/// Why a row could not be normalized. Rejected rows are surfaced, never silently dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnComtradeRowRejection {
    NotAnObject,
    MissingField(String),
    InvalidFieldType(String),
    QueryDimensionMismatch(String),
    UnknownAreaCode(String),
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
    pub response_count: usize,
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
    CountDataMismatch { reported: usize, actual: usize },
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
            Self::CountDataMismatch { reported, actual } => write!(
                f,
                "UN Comtrade count/data mismatch: count={reported}, rows={actual}"
            ),
            Self::MoreRecordsThanRequested { requested, actual } => write!(
                f,
                "UN Comtrade response exceeded requested limit: requested={requested}, rows={actual}"
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

    let envelope: PreviewEnvelope = serde_json::from_str(response_json)
        .map_err(|error| UnComtradePreviewError::InvalidJson(error.to_string()))?;

    if let Some(error) = envelope.error.as_ref().filter(|value| api_error_present(value)) {
        return Err(UnComtradePreviewError::ApiReportedError(error.to_string()));
    }

    if envelope.count != envelope.data.len() {
        return Err(UnComtradePreviewError::CountDataMismatch {
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

    let mut records = Vec::new();
    let mut rejected_rows = Vec::new();
    for (row_index, raw_record) in envelope.data.into_iter().enumerate() {
        match normalize_record(&raw_record, context) {
            Ok(observation) => records.push(UnComtradeRecord {
                observation,
                raw_record,
            }),
            Err(reason) => rejected_rows.push(RejectedUnComtradeRow { row_index, reason }),
        }
    }

    let response_count = envelope.count;
    let at_limit = response_count >= context.query.max_records;
    let coverage = if response_count == 0 {
        PreviewCoverage::EmptyResponse
    } else if !rejected_rows.is_empty() && at_limit {
        PreviewCoverage::AtLimitWithRejectedRows
    } else if !rejected_rows.is_empty() {
        PreviewCoverage::BelowLimitWithRejectedRows
    } else if at_limit {
        PreviewCoverage::AtRequestedLimitPossiblyTruncated
    } else {
        PreviewCoverage::BelowRequestedLimitCompletenessUnproven
    };

    Ok(ParsedUnComtradePreview {
        source_snapshot_id: context.source_snapshot_id.clone(),
        area_reference_snapshot_id: context.area_reference_snapshot_id.clone(),
        retrieved_at: context.retrieved_at.clone(),
        query: context.query.clone(),
        elapsed_time: envelope.elapsed_time.as_ref().map(value_to_text),
        response_count,
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
        ("classification_revision", query.classification_revision.as_str()),
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
    let primary_value = required_scalar(object, "primaryValue")?;

    // These are required dimensions because they can alter the meaning/grain
    // of the reported value even when the main bilateral query fields match.
    let second_partner_code = required_scalar(object, "partner2Code")?;
    let transport_mode_code = required_scalar(object, "motCode")?;
    let customs_procedure_code = required_scalar(object, "customsCode")?;

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

    let dimension_signature = format!(
        "{UN_COMTRADE_DIMENSIONS_VERSION}|partner2={second_partner_code}|mot={transport_mode_code}|customs={customs_procedure_code}"
    );
    let record_id = format!(
        "comtrade:{}/{}/{}/{}/{}/{}/{}/{}/partner2={}/mot={}/customs={}",
        query.type_code,
        query.frequency_code,
        query.classification_code,
        period,
        reporter_code,
        partner_code,
        flow_code,
        commodity_code,
        second_partner_code,
        transport_mode_code,
        customs_procedure_code
    );

    Ok(TradeObservation {
        schema_version: TRADE_OBSERVATION_SCHEMA_VERSION,
        record_id,
        source_id: UN_COMTRADE_SOURCE_ID.to_string(),
        source_snapshot_id: context.source_snapshot_id.clone(),
        source_record_id: None,
        source_url: None,
        reporter: reporter_code,
        reporter_kind,
        partner: partner_code,
        partner_kind,
        geography_code_system: UN_COMTRADE_GEOGRAPHY_CODE_SYSTEM.to_string(),
        period,
        disaggregation_signature: dimension_signature,
        commodity_code,
        classification_revision: query.classification_revision.clone(),
        flow,
        trade_value_source_text: Some(primary_value),
        currency: "USD".to_string(),
        value_unit: "USD".to_string(),
        quantity: optional_f64(object, "netWgt"),
        quantity_unit: optional_scalar(object, "qtyUnitAbbr"),
        source_published_at: optional_scalar(object, "refYear")
            .or_else(|| optional_scalar(object, "period")),
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

fn optional_scalar(object: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    object.get(key).and_then(scalar_to_text)
}

fn optional_f64(object: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
    object.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    })
}

fn scalar_to_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
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
            classification_revision: "HS-2022".to_string(),
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
            "partner2Code": 0,
            "motCode": "0",
            "customsCode": "C00",
            "primaryValue": 9007199254740993,
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
        let raw = response(vec![row()], 1);
        let parsed = parse_un_comtrade_preview(&raw, &context(10)).unwrap();
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
        assert!(parsed.records[0].raw_record.get("anExtraRecordField").is_some());
        assert!(!parsed.coverage.completeness_proven());
        assert_eq!(
            parsed.coverage,
            PreviewCoverage::BelowRequestedLimitCompletenessUnproven
        );
    }

    #[test]
    fn reaching_requested_limit_is_marked_as_possibly_truncated() {
        let parsed = parse_un_comtrade_preview(&response(vec![row(), row()], 2), &context(2))
            .unwrap();
        assert_eq!(
            parsed.coverage,
            PreviewCoverage::AtRequestedLimitPossiblyTruncated
        );
        assert!(parsed.coverage.may_be_truncated());
        assert!(!parsed.coverage.completeness_proven());
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
            PreviewCoverage::BelowLimitWithRejectedRows
        );
    }

    #[test]
    fn unknown_area_code_is_rejected() {
        let mut unknown = row();
        unknown["reporterCode"] = Value::from(9999);
        let mut ctx = context(10);
        ctx.query.reporter_code = "9999".to_string();
        ctx.area_kinds.remove("9999");
        let parsed = parse_un_comtrade_preview(&response(vec![unknown], 1), &ctx).unwrap();
        assert_eq!(
            parsed.rejected_rows[0].reason,
            UnComtradeRowRejection::UnknownAreaCode("9999".to_string())
        );
    }

    #[test]
    fn count_mismatch_and_unknown_envelope_fields_fail_closed() {
        assert_eq!(
            parse_un_comtrade_preview(&response(vec![row()], 2), &context(10)),
            Err(UnComtradePreviewError::CountDataMismatch {
                reported: 2,
                actual: 1
            })
        );

        let mut value: Value = serde_json::from_str(&response(vec![row()], 1)).unwrap();
        value["unexpectedEnvelopeField"] = Value::Bool(true);
        let result = parse_un_comtrade_preview(&value.to_string(), &context(10));
        assert!(matches!(
            result,
            Err(UnComtradePreviewError::InvalidJson(_))
        ));
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
