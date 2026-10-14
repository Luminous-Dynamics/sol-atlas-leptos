// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Offline, fail-closed parser for World Bank Data360 observation pages.
//!
//! This module performs no network requests. The caller supplies query context,
//! immutable collection/page identities, and the retrieval timestamp, and must
//! persist and hash the exact response bytes separately. A parsed page is not a
//! complete dataset: coverage can be established only after validating the full
//! contiguous page sequence against a stable source-reported count.
//!
//! Source reference: World Bank Data360 OpenAPI specification, DataResponse and
//! GET /data360/data. Synthetic tests are explicitly not World Bank observations.

use serde_json::{Map, Value};
use sha2::Digest;
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fmt;

/// Official Data360 OpenAPI currently documents at most 1,000 rows per response.
pub const DATA360_MAX_PAGE_SIZE: usize = 1_000;
pub const DATA360_SCHEMA_VERSION: u16 = 1;

const ENVELOPE_KEYS: [&str; 2] = ["count", "value"];
const FILTER_KEYS: [&str; 12] = [
    "REF_AREA",
    "SEX",
    "AGE",
    "URBANISATION",
    "COMP_BREAKDOWN_1",
    "COMP_BREAKDOWN_2",
    "COMP_BREAKDOWN_3",
    "TIME_PERIOD",
    "FREQ",
    "UNIT_MEASURE",
    "UNIT_TYPE",
    "UNIT_MULT",
];

const ROW_KEYS: [&str; 24] = [
    "OBS_VALUE",
    "TIME_FORMAT",
    "UNIT_MULT",
    "COMMENT_OBS",
    "OBS_STATUS",
    "OBS_CONF",
    "AGG_METHOD",
    "DECIMALS",
    "COMMENT_TS",
    "DATA_SOURCE",
    "LATEST_DATA",
    "DATABASE_ID",
    "INDICATOR",
    "REF_AREA",
    "SEX",
    "AGE",
    "URBANISATION",
    "COMP_BREAKDOWN_1",
    "COMP_BREAKDOWN_2",
    "COMP_BREAKDOWN_3",
    "TIME_PERIOD",
    "FREQ",
    "UNIT_MEASURE",
    "UNIT_TYPE",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data360PageQuery {
    pub database_id: String,
    pub indicator_id: String,
    pub period_start: u16,
    pub period_end: u16,
    /// Exact single-value dimension filters actually used for the request,
    /// excluding DATABASE_ID, INDICATOR, skip, and the period-range bounds.
    /// Supported keys are the documented Data360 query parameter spellings.
    /// This records caller-supplied request context; it cannot prove the HTTP
    /// request sent by a separate acquisition layer used these values.
    pub filters: BTreeMap<String, String>,
    pub requested_skip: usize,
    pub requested_top: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data360PageContext {
    /// One immutable collection/run ID shared by every page in a collection.
    pub collection_snapshot_id: String,
    /// Identity of these exact response bytes (prefer a SHA-256 digest).
    pub page_snapshot_id: String,
    pub retrieved_at: String,
    pub query: Data360PageQuery,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Data360Observation {
    pub row_index: usize,
    pub record_id: String,
    pub period: u16,
    pub reference_area: String,
    /// Exact source text. Null remains None and is never silently changed to zero.
    pub value_source_text: Option<String>,
    pub unit_measure: String,
    pub unit_multiplier: i64,
    pub raw_record: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data360RowRejection {
    NotAnObject,
    MissingField(String),
    InvalidFieldType(String),
    InvalidFieldValue(String),
    UnknownField(String),
    DatabaseMismatch,
    IndicatorMismatch,
    PeriodOutOfQueryRange,
    QueryFilterMismatch(String),
    DuplicateRecordKey(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedData360Row {
    pub row_index: usize,
    pub reason: Data360RowRejection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedData360Page {
    pub schema_version: u16,
    pub collection_snapshot_id: String,
    pub page_snapshot_id: String,
    pub retrieved_at: String,
    pub query: Data360PageQuery,
    pub source_reported_count: usize,
    pub rows_returned: usize,
    pub records: Vec<Data360Observation>,
    pub rejected_rows: Vec<RejectedData360Row>,
}

impl ParsedData360Page {
    /// True means more pages are required according to this response's count.
    /// It does not prove counts remained stable between requests.
    pub fn may_have_more_pages(&self) -> bool {
        self.query.requested_skip.saturating_add(self.rows_returned) < self.source_reported_count
    }

    /// A single page never proves a complete dataset snapshot.
    pub fn completeness_proven(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data360PageError {
    InvalidContext(String),
    InvalidJson(String),
    EnvelopeSchema(String),
    CountLessThanRowsReturned { reported: usize, actual: usize },
    MoreRecordsThanRequested { requested: usize, actual: usize },
}

impl fmt::Display for Data360PageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContext(message) => write!(f, "invalid Data360 page context: {message}"),
            Self::InvalidJson(message) => write!(f, "invalid Data360 JSON: {message}"),
            Self::EnvelopeSchema(message) => write!(f, "Data360 envelope schema error: {message}"),
            Self::CountLessThanRowsReturned { reported, actual } => write!(
                f,
                "Data360 reported count is smaller than returned rows: count={reported}, rows={actual}"
            ),
            Self::MoreRecordsThanRequested { requested, actual } => write!(
                f,
                "Data360 response exceeded requested limit: requested={requested}, rows={actual}"
            ),
        }
    }
}
impl Error for Data360PageError {}

/// Parse one response page without doing I/O, guessing indicator IDs, or
/// treating source-reported totals as proof of complete multi-page coverage.
pub fn parse_data360_page(
    response_json: &str,
    context: &Data360PageContext,
) -> Result<ParsedData360Page, Data360PageError> {
    validate_context(context)?;
    let parsed: Value = serde_json::from_str(response_json)
        .map_err(|error| Data360PageError::InvalidJson(error.to_string()))?;
    let envelope = parsed.as_object().ok_or_else(|| {
        Data360PageError::EnvelopeSchema("response must be a JSON object".to_string())
    })?;
    if let Some(key) = envelope
        .keys()
        .find(|key| !ENVELOPE_KEYS.contains(&key.as_str()))
    {
        return Err(Data360PageError::EnvelopeSchema(format!(
            "unsupported top-level field {key:?}"
        )));
    }
    for key in ENVELOPE_KEYS {
        if !envelope.contains_key(key) {
            return Err(Data360PageError::EnvelopeSchema(format!(
                "required top-level field {key:?} is missing"
            )));
        }
    }

    let count_value = envelope["count"].as_u64().ok_or_else(|| {
        Data360PageError::EnvelopeSchema(
            "count must be a non-negative integer JSON token".to_string(),
        )
    })?;
    let source_reported_count = usize::try_from(count_value).map_err(|_| {
        Data360PageError::EnvelopeSchema("count exceeds this platform's usize range".to_string())
    })?;
    let rows = envelope["value"].as_array().ok_or_else(|| {
        Data360PageError::EnvelopeSchema("value must be a JSON array".to_string())
    })?;
    if source_reported_count < rows.len() {
        return Err(Data360PageError::CountLessThanRowsReturned {
            reported: source_reported_count,
            actual: rows.len(),
        });
    }
    if rows.len() > context.query.requested_top {
        return Err(Data360PageError::MoreRecordsThanRequested {
            requested: context.query.requested_top,
            actual: rows.len(),
        });
    }

    let rows_returned = rows.len();
    let mut records = Vec::new();
    let mut rejected_rows = Vec::new();
    let mut seen = HashSet::new();
    for (row_index, raw) in rows.iter().enumerate() {
        match normalize_record(raw, context, row_index) {
            Ok(record) => {
                if !seen.insert(record.record_id.clone()) {
                    rejected_rows.push(RejectedData360Row {
                        row_index,
                        reason: Data360RowRejection::DuplicateRecordKey(record.record_id),
                    });
                } else {
                    records.push(record);
                }
            }
            Err(reason) => rejected_rows.push(RejectedData360Row { row_index, reason }),
        }
    }

    Ok(ParsedData360Page {
        schema_version: DATA360_SCHEMA_VERSION,
        collection_snapshot_id: context.collection_snapshot_id.clone(),
        page_snapshot_id: context.page_snapshot_id.clone(),
        retrieved_at: context.retrieved_at.clone(),
        query: context.query.clone(),
        source_reported_count,
        rows_returned,
        records,
        rejected_rows,
    })
}

fn validate_context(context: &Data360PageContext) -> Result<(), Data360PageError> {
    let q = &context.query;
    for (name, value) in [
        ("database_id", q.database_id.as_str()),
        ("indicator_id", q.indicator_id.as_str()),
        (
            "collection_snapshot_id",
            context.collection_snapshot_id.as_str(),
        ),
        ("page_snapshot_id", context.page_snapshot_id.as_str()),
        ("retrieved_at", context.retrieved_at.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(Data360PageError::InvalidContext(format!(
                "{name} must be non-empty"
            )));
        }
    }
    if q.period_start == 0 || q.period_end < q.period_start || q.period_end > 9999 {
        return Err(Data360PageError::InvalidContext(
            "annual period range must be ordered and fit a four-digit year".to_string(),
        ));
    }
    for (key, value) in &q.filters {
        if !FILTER_KEYS.contains(&key.as_str()) {
            return Err(Data360PageError::InvalidContext(format!(
                "unsupported Data360 query filter {key:?}"
            )));
        }
        if value.trim().is_empty() {
            return Err(Data360PageError::InvalidContext(format!(
                "query filter {key:?} must not be empty; omit an unused filter"
            )));
        }
        match key.as_str() {
            "UNIT_MULT" => {
                value.parse::<i64>().map_err(|_| {
                    Data360PageError::InvalidContext(
                        "UNIT_MULT query filter must be an integer".to_string(),
                    )
                })?;
            }
            _ => {}
        }
    }
    if q.requested_top == 0 || q.requested_top > DATA360_MAX_PAGE_SIZE {
        return Err(Data360PageError::InvalidContext(format!(
            "requested_top must be in 1..={DATA360_MAX_PAGE_SIZE}"
        )));
    }
    if q.requested_skip.checked_add(q.requested_top).is_none() {
        return Err(Data360PageError::InvalidContext(
            "skip + top overflows usize".to_string(),
        ));
    }
    Ok(())
}

fn normalize_record(
    raw: &Value,
    context: &Data360PageContext,
    row_index: usize,
) -> Result<Data360Observation, Data360RowRejection> {
    let object = raw.as_object().ok_or(Data360RowRejection::NotAnObject)?;
    if let Some(key) = object.keys().find(|key| !ROW_KEYS.contains(&key.as_str())) {
        return Err(Data360RowRejection::UnknownField(key.clone()));
    }
    for key in ROW_KEYS {
        if !object.contains_key(key) {
            return Err(Data360RowRejection::MissingField(key.to_string()));
        }
    }

    let database_id = required_text(object, "DATABASE_ID", false)?;
    let indicator_id = required_text(object, "INDICATOR", false)?;
    let reference_area = required_text(object, "REF_AREA", false)?;
    let period_text = required_text(object, "TIME_PERIOD", false)?;
    let frequency = required_text(object, "FREQ", false)?;
    let data_source = required_text(object, "DATA_SOURCE", false)?;
    let sex = required_text(object, "SEX", false)?;
    let age = required_text(object, "AGE", false)?;
    let urbanisation = required_text(object, "URBANISATION", false)?;
    let breakdown_1 = required_text(object, "COMP_BREAKDOWN_1", false)?;
    let breakdown_2 = required_text(object, "COMP_BREAKDOWN_2", false)?;
    let breakdown_3 = required_text(object, "COMP_BREAKDOWN_3", false)?;
    let unit_measure = required_text(object, "UNIT_MEASURE", false)?;
    let unit_type_value = nullable_text(object, "UNIT_TYPE")?;

    let _time_format = required_text(object, "TIME_FORMAT", false)?;
    let _observation_status = required_text(object, "OBS_STATUS", false)?;
    let _observation_confidence = required_text(object, "OBS_CONF", false)?;
    let _aggregation_method = required_text(object, "AGG_METHOD", false)?;
    let _decimals = required_text(object, "DECIMALS", false)?;
    let _timeseries_comment = required_text(object, "COMMENT_TS", true)?;
    let unit_multiplier = object
        .get("UNIT_MULT")
        .and_then(Value::as_i64)
        .ok_or_else(|| Data360RowRejection::InvalidFieldType("UNIT_MULT".to_string()))?;
    let _latest_data = object
        .get("LATEST_DATA")
        .and_then(Value::as_bool)
        .ok_or_else(|| Data360RowRejection::InvalidFieldType("LATEST_DATA".to_string()))?;
    nullable_text(object, "COMMENT_OBS")?;
    let value_source_text = match object.get("OBS_VALUE") {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Null) => None,
        Some(_) => {
            return Err(Data360RowRejection::InvalidFieldType(
                "OBS_VALUE".to_string(),
            ));
        }
        None => return Err(Data360RowRejection::MissingField("OBS_VALUE".to_string())),
    };

    if database_id != context.query.database_id {
        return Err(Data360RowRejection::DatabaseMismatch);
    }
    if indicator_id != context.query.indicator_id {
        return Err(Data360RowRejection::IndicatorMismatch);
    }
    if period_text.len() != 4 || !period_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Data360RowRejection::InvalidFieldValue(
            "TIME_PERIOD".to_string(),
        ));
    }
    let period = period_text
        .parse::<u16>()
        .map_err(|_| Data360RowRejection::InvalidFieldValue("TIME_PERIOD".to_string()))?;
    if period < context.query.period_start || period > context.query.period_end {
        return Err(Data360RowRejection::PeriodOutOfQueryRange);
    }

    // A collection must retain and apply the full supported dimension-filter
    // context, not merely the dataset/indicator/year range. A caller that
    // records filters inaccurately is still responsible for request provenance.
    for (name, expected) in &context.query.filters {
        let actual = match name.as_str() {
            "REF_AREA" => Some(reference_area.clone()),
            "SEX" => Some(sex.clone()),
            "AGE" => Some(age.clone()),
            "URBANISATION" => Some(urbanisation.clone()),
            "COMP_BREAKDOWN_1" => Some(breakdown_1.clone()),
            "COMP_BREAKDOWN_2" => Some(breakdown_2.clone()),
            "COMP_BREAKDOWN_3" => Some(breakdown_3.clone()),
            "TIME_PERIOD" => Some(period_text.clone()),
            "FREQ" => Some(frequency.clone()),
            "UNIT_MEASURE" => Some(unit_measure.clone()),
            "UNIT_TYPE" => unit_type_value.clone(),
            "UNIT_MULT" => Some(unit_multiplier.to_string()),
            _ => None, // validate_context rejects unsupported keys first
        };
        if actual.as_deref() != Some(expected.as_str()) {
            return Err(Data360RowRejection::QueryFilterMismatch(name.clone()));
        }
    }

    // Length-prefix every key component to avoid delimiter-collision identities.
    let components = [
        database_id.as_str(),
        indicator_id.as_str(),
        data_source.as_str(),
        reference_area.as_str(),
        period_text.as_str(),
        frequency.as_str(),
        sex.as_str(),
        age.as_str(),
        urbanisation.as_str(),
        breakdown_1.as_str(),
        breakdown_2.as_str(),
        breakdown_3.as_str(),
        unit_measure.as_str(),
        unit_type_value.as_deref().unwrap_or(""),
        if object["UNIT_TYPE"].is_null() {
            "null"
        } else {
            "string"
        },
        if object["COMMENT_TS"].is_null() {
            "null"
        } else {
            "string"
        },
        if object["OBS_CONF"].is_null() {
            "null"
        } else {
            "string"
        },
    ];
    let mut record_id = String::from("world-bank-data360-observation/v1|");
    for component in components {
        record_id.push_str(&format!("{}:{component}|", component.len()));
    }
    let multiplier_token = unit_multiplier.to_string();
    record_id.push_str(&format!("{}:{multiplier_token}|", multiplier_token.len()));

    Ok(Data360Observation {
        row_index,
        record_id,
        period,
        reference_area,
        value_source_text,
        unit_measure,
        unit_multiplier,
        raw_record: raw.clone(),
    })
}

fn required_text(
    object: &Map<String, Value>,
    key: &'static str,
    allow_empty: bool,
) -> Result<String, Data360RowRejection> {
    match object.get(key) {
        None => Err(Data360RowRejection::MissingField(key.to_string())),
        Some(Value::String(text)) if allow_empty || !text.trim().is_empty() => Ok(text.clone()),
        Some(Value::String(_)) => Err(Data360RowRejection::InvalidFieldValue(key.to_string())),
        Some(_) => Err(Data360RowRejection::InvalidFieldType(key.to_string())),
    }
}

fn nullable_text(
    object: &Map<String, Value>,
    key: &'static str,
) -> Result<Option<String>, Data360RowRejection> {
    match object.get(key) {
        None => Err(Data360RowRejection::MissingField(key.to_string())),
        Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(Data360RowRejection::InvalidFieldType(key.to_string())),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Data360SnapshotStatus {
    CompleteAgainstStableCount,
    NoPages,
    UnsupportedSchemaVersion,
    QueryMismatch,
    SourceCountDrift,
    DuplicatePageSnapshotIdentity,
    AcquisitionEvidenceInvalid,
    OffsetGapOrOverlap,
    PageAccountingMismatch,
    CountLessThanPageRows,
    RowsRejected,
    DuplicateRecordKey,
    IncompletePageSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data360SnapshotAssessment {
    pub status: Data360SnapshotStatus,
    pub expected_count: Option<usize>,
    pub rows_returned: usize,
    pub accepted_rows: usize,
    pub rejected_rows: usize,
    pub next_skip: usize,
}

/// Qualifies internal collection completeness only. It says nothing about
/// source truth, edition identity, licensing, or empirical validity. A
/// CompleteAgainstStableCount result does not prove that values stayed unchanged
/// between requests; this response contract has no source snapshot token. It
/// means only that these captured pages are internally contiguous against one
/// stable reported count, with no rejected rows or duplicate observation keys.
pub fn assess_data360_snapshot(pages: &[ParsedData360Page]) -> Data360SnapshotAssessment {
    if pages.is_empty() {
        return Data360SnapshotAssessment {
            status: Data360SnapshotStatus::NoPages,
            expected_count: None,
            rows_returned: 0,
            accepted_rows: 0,
            rejected_rows: 0,
            next_skip: 0,
        };
    }
    let first = &pages[0];
    let expected_count = first.source_reported_count;
    let mut expected_skip = 0usize;
    let mut rows_returned = 0usize;
    let mut accepted_rows = 0usize;
    let mut rejected_rows = 0usize;
    let mut seen = HashSet::new();
    let mut seen_page_snapshot_ids = HashSet::new();
    let mut status = Data360SnapshotStatus::CompleteAgainstStableCount;

    for (index, page) in pages.iter().enumerate() {
        if page.schema_version != DATA360_SCHEMA_VERSION {
            status = Data360SnapshotStatus::UnsupportedSchemaVersion;
            break;
        }
        let Some(accounted_rows) = page.records.len().checked_add(page.rejected_rows.len()) else {
            status = Data360SnapshotStatus::PageAccountingMismatch;
            break;
        };
        if accounted_rows != page.rows_returned
            || page.query.requested_top == 0
            || page.query.requested_top > DATA360_MAX_PAGE_SIZE
            || page.rows_returned > page.query.requested_top
            || page
                .query
                .requested_skip
                .checked_add(page.rows_returned)
                .is_none()
            || page.source_reported_count < page.rows_returned
            || page.collection_snapshot_id.trim().is_empty()
            || page.query.database_id.trim().is_empty()
            || page.query.indicator_id.trim().is_empty()
            || page.query.period_start == 0
            || page.query.period_end < page.query.period_start
            || page.page_snapshot_id.trim().is_empty()
            || page.retrieved_at.trim().is_empty()
        {
            status = if page.source_reported_count < page.rows_returned {
                Data360SnapshotStatus::CountLessThanPageRows
            } else {
                Data360SnapshotStatus::PageAccountingMismatch
            };
            break;
        }
        if page.collection_snapshot_id != first.collection_snapshot_id
            || page.query.database_id != first.query.database_id
            || page.query.indicator_id != first.query.indicator_id
            || page.query.period_start != first.query.period_start
            || page.query.period_end != first.query.period_end
            || page.query.filters != first.query.filters
            || page.query.requested_top != first.query.requested_top
        {
            status = Data360SnapshotStatus::QueryMismatch;
            break;
        }
        if page.source_reported_count != expected_count {
            status = Data360SnapshotStatus::SourceCountDrift;
            break;
        }
        if !seen_page_snapshot_ids.insert(page.page_snapshot_id.as_str()) {
            status = Data360SnapshotStatus::DuplicatePageSnapshotIdentity;
            break;
        }
        if page.query.requested_skip != expected_skip
            || (index > 0 && expected_skip >= expected_count)
        {
            status = Data360SnapshotStatus::OffsetGapOrOverlap;
            break;
        }
        let Some(next_rows_returned) = rows_returned.checked_add(page.rows_returned) else {
            status = Data360SnapshotStatus::PageAccountingMismatch;
            break;
        };
        let Some(next_rejected_rows) = rejected_rows.checked_add(page.rejected_rows.len()) else {
            status = Data360SnapshotStatus::PageAccountingMismatch;
            break;
        };
        let Some(next_accepted_rows) = accepted_rows.checked_add(page.records.len()) else {
            status = Data360SnapshotStatus::PageAccountingMismatch;
            break;
        };
        rows_returned = next_rows_returned;
        rejected_rows = next_rejected_rows;
        accepted_rows = next_accepted_rows;
        if !page.rejected_rows.is_empty() {
            status = Data360SnapshotStatus::RowsRejected;
        }
        let page_context = Data360PageContext {
            collection_snapshot_id: page.collection_snapshot_id.clone(),
            page_snapshot_id: page.page_snapshot_id.clone(),
            retrieved_at: page.retrieved_at.clone(),
            query: page.query.clone(),
        };
        let mut seen_row_indices = HashSet::new();
        for record in &page.records {
            if record.row_index >= page.rows_returned || !seen_row_indices.insert(record.row_index)
            {
                status = Data360SnapshotStatus::PageAccountingMismatch;
                break;
            }
            match normalize_record(&record.raw_record, &page_context, record.row_index) {
                Ok(rebuilt) if &rebuilt == record => {}
                _ => {
                    status = Data360SnapshotStatus::PageAccountingMismatch;
                    break;
                }
            }
            if !seen.insert(record.record_id.clone()) {
                status = Data360SnapshotStatus::DuplicateRecordKey;
            }
        }
        for rejected in &page.rejected_rows {
            if rejected.row_index >= page.rows_returned
                || !seen_row_indices.insert(rejected.row_index)
            {
                status = Data360SnapshotStatus::PageAccountingMismatch;
                break;
            }
        }
        if seen_row_indices.len() != page.rows_returned {
            status = Data360SnapshotStatus::PageAccountingMismatch;
            break;
        }
        let Some(next_skip) = expected_skip.checked_add(page.rows_returned) else {
            status = Data360SnapshotStatus::OffsetGapOrOverlap;
            break;
        };
        expected_skip = next_skip;
        if expected_skip > expected_count {
            status = Data360SnapshotStatus::OffsetGapOrOverlap;
            break;
        }
        if page.may_have_more_pages() && page.rows_returned != page.query.requested_top {
            status = Data360SnapshotStatus::IncompletePageSet;
            break;
        }
    }

    if status == Data360SnapshotStatus::CompleteAgainstStableCount
        && expected_skip != expected_count
    {
        status = Data360SnapshotStatus::IncompletePageSet;
    }
    Data360SnapshotAssessment {
        status,
        expected_count: Some(expected_count),
        rows_returned,
        accepted_rows,
        rejected_rows,
        next_skip: expected_skip,
    }
}


/// Schema version for byte-bound acquisition evidence manifests.
pub const DATA360_ACQUISITION_EVIDENCE_SCHEMA_VERSION: u16 = 1;

const ACQUISITION_QUERY_KEYS: [&str; 17] = [
    "DATABASE_ID",
    "INDICATOR",
    "REF_AREA",
    "SEX",
    "AGE",
    "URBANISATION",
    "COMP_BREAKDOWN_1",
    "COMP_BREAKDOWN_2",
    "COMP_BREAKDOWN_3",
    "TIME_PERIOD",
    "FREQ",
    "UNIT_MEASURE",
    "UNIT_TYPE",
    "UNIT_MULT",
    "timePeriodFrom",
    "timePeriodTo",
    "skip",
];

const ACQUISITION_RESPONSE_HEADERS: [&str; 11] = [
    "content-type",
    "content-length",
    "date",
    "etag",
    "last-modified",
    "cache-control",
    "content-encoding",
    "vary",
    "x-request-id",
    "request-id",
    "expires",
];

/// Evidence recorded by an HTTP acquisition layer for the exact body bytes it
/// hands to the parser. This structure does not itself prove that the collector
/// told the truth; it makes mismatches detectable and the capture auditable.
///
/// Only allowlisted, lower-case response headers are retained. Request
/// credentials and cookies must not be stored in this evidence object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data360AcquisitionEvidence {
    pub schema_version: u16,
    pub request_method: String,
    /// Complete HTTPS URL including its query string.
    pub request_url: String,
    /// Decoded query parameters from that URL; duplicate parameter names are
    /// rejected because a single map cannot faithfully represent duplicates.
    pub request_parameters: BTreeMap<String, String>,
    pub response_status: u16,
    /// Selected response headers only; names must be lower-case allowlisted keys.
    pub response_headers: BTreeMap<String, String>,
    /// Length and SHA-256 are computed over the exact body bytes passed to this
    /// module, before UTF-8 decoding or JSON parsing.
    pub response_body_byte_length: u64,
    /// 64 lowercase hexadecimal characters.
    pub response_body_sha256: String,
    pub retrieved_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data360AcquisitionError {
    UnsupportedSchemaVersion,
    InvalidRequestMethod,
    InvalidRequestUrl,
    DuplicateUrlQueryParameter(String),
    RequestUrlParameterMismatch,
    UnsupportedRequestParameter(String),
    MissingOrMismatchedRequestParameter(String),
    NonSuccessResponseStatus(u16),
    InvalidResponseHeaders(String),
    MissingContentType,
    UnsupportedContentType(String),
    ResponseBodyLengthMismatch,
    ContentLengthHeaderMismatch,
    InvalidSha256Format,
    ResponseBodyDigestMismatch,
    PageSnapshotIdentityMismatch,
    RetrievalTimestampMismatch,
    ResponseBodyNotUtf8,
    PageParseFailed(String),
}

impl fmt::Display for Data360AcquisitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Data360 acquisition evidence error: {self:?}")
    }
}
impl Error for Data360AcquisitionError {}

/// Validate capture provenance before treating response bytes as a parsed page.
/// The URL and query map must agree, the endpoint must be the official HTTPS
/// observation route, query values must match the parser context, the response
/// must be HTTP 200 with a JSON-compatible content type, and the digest/length
/// must match the exact bytes supplied by the caller.
pub fn validate_data360_acquisition_evidence(
    response_body: &[u8],
    evidence: &Data360AcquisitionEvidence,
    context: &Data360PageContext,
) -> Result<(), Data360AcquisitionError> {
    if evidence.schema_version != DATA360_ACQUISITION_EVIDENCE_SCHEMA_VERSION {
        return Err(Data360AcquisitionError::UnsupportedSchemaVersion);
    }
    if evidence.request_method != "GET" {
        return Err(Data360AcquisitionError::InvalidRequestMethod);
    }
    if evidence.retrieved_at.trim().is_empty() || evidence.retrieved_at != context.retrieved_at {
        return Err(Data360AcquisitionError::RetrievalTimestampMismatch);
    }

    let url = url::Url::parse(&evidence.request_url)
        .map_err(|_| Data360AcquisitionError::InvalidRequestUrl)?;
    if url.scheme() != "https"
        || url.host_str() != Some("data360api.worldbank.org")
        || url.path() != "/data360/data"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err(Data360AcquisitionError::InvalidRequestUrl);
    }

    let mut url_parameters = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        let key = key.into_owned();
        if url_parameters.insert(key.clone(), value.into_owned()).is_some() {
            return Err(Data360AcquisitionError::DuplicateUrlQueryParameter(key));
        }
    }
    if url_parameters != evidence.request_parameters {
        return Err(Data360AcquisitionError::RequestUrlParameterMismatch);
    }

    for key in evidence.request_parameters.keys() {
        if !ACQUISITION_QUERY_KEYS.contains(&key.as_str()) {
            return Err(Data360AcquisitionError::UnsupportedRequestParameter(key.clone()));
        }
    }
    let query = &context.query;
    for (key, expected) in [
        ("DATABASE_ID", query.database_id.as_str()),
        ("INDICATOR", query.indicator_id.as_str()),
        ("skip", ""),
    ] {
        let actual = evidence.request_parameters.get(key);
        let matches = if key == "skip" {
            actual.and_then(|value| value.parse::<usize>().ok()) == Some(query.requested_skip)
                && actual.map(|value| value == &query.requested_skip.to_string()) == Some(true)
        } else {
            actual.map(String::as_str) == Some(expected)
        };
        if !matches {
            return Err(Data360AcquisitionError::MissingOrMismatchedRequestParameter(
                key.to_string(),
            ));
        }
    }

    // TIME_PERIOD is a single-year query; otherwise require the documented
    // inclusive timePeriodFrom/timePeriodTo pair. Do not infer a requested
    // range merely from the parser's filtering bounds.
    match query.filters.get("TIME_PERIOD") {
        Some(single_year) => {
            if query.period_start != query.period_end
                || single_year != &query.period_start.to_string()
                || evidence.request_parameters.get("TIME_PERIOD") != Some(single_year)
                || evidence.request_parameters.contains_key("timePeriodFrom")
                || evidence.request_parameters.contains_key("timePeriodTo")
            {
                return Err(Data360AcquisitionError::MissingOrMismatchedRequestParameter(
                    "TIME_PERIOD".to_string(),
                ));
            }
        }
        None => {
            if evidence.request_parameters.get("TIME_PERIOD").is_some()
                || evidence.request_parameters.get("timePeriodFrom")
                    != Some(&query.period_start.to_string())
                || evidence.request_parameters.get("timePeriodTo")
                    != Some(&query.period_end.to_string())
            {
                return Err(Data360AcquisitionError::MissingOrMismatchedRequestParameter(
                    "timePeriodFrom/timePeriodTo".to_string(),
                ));
            }
        }
    }

    for key in FILTER_KEYS {
        if key == "TIME_PERIOD" {
            continue;
        }
        match (query.filters.get(key), evidence.request_parameters.get(key)) {
            (Some(expected), Some(actual)) if expected == actual => {}
            (None, None) => {}
            _ => {
                return Err(Data360AcquisitionError::MissingOrMismatchedRequestParameter(
                    key.to_string(),
                ));
            }
        }
    }

    if evidence.response_status != 200 {
        return Err(Data360AcquisitionError::NonSuccessResponseStatus(
            evidence.response_status,
        ));
    }
    for (name, value) in &evidence.response_headers {
        if !ACQUISITION_RESPONSE_HEADERS.contains(&name.as_str())
            || name != &name.to_ascii_lowercase()
            || value.chars().any(|character| {
                (character.is_control() && character != '\t')
                    || character == '\r'
                    || character == '\n'
            })
        {
            return Err(Data360AcquisitionError::InvalidResponseHeaders(
                name.clone(),
            ));
        }
    }
    let content_type = evidence
        .response_headers
        .get("content-type")
        .ok_or(Data360AcquisitionError::MissingContentType)?;
    let media_type = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if !["application/json", "text/json", "text/plain"].contains(&media_type.as_str()) {
        return Err(Data360AcquisitionError::UnsupportedContentType(
            media_type,
        ));
    }
    if let Some(length) = evidence.response_headers.get("content-length") {
        if length.is_empty() || !length.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(Data360AcquisitionError::InvalidResponseHeaders(
                "content-length".to_string(),
            ));
        }
        let encoding = evidence
            .response_headers
            .get("content-encoding")
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "identity".to_string());
        // A compressed HTTP Content-Length may describe the encoded payload,
        // while response_body is the exact decoded byte slice handed to parser.
        if (encoding.is_empty() || encoding == "identity")
            && length.parse::<u64>().ok() != Some(response_body.len() as u64)
        {
            return Err(Data360AcquisitionError::ContentLengthHeaderMismatch);
        }
    }

    if evidence.response_body_byte_length != response_body.len() as u64 {
        return Err(Data360AcquisitionError::ResponseBodyLengthMismatch);
    }
    let digest = evidence.response_body_sha256.as_str();
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Data360AcquisitionError::InvalidSha256Format);
    }
    let actual_digest = format!("{:x}", sha2::Sha256::digest(response_body));
    if digest != actual_digest {
        return Err(Data360AcquisitionError::ResponseBodyDigestMismatch);
    }
    if context.page_snapshot_id != format!("sha256:{actual_digest}") {
        return Err(Data360AcquisitionError::PageSnapshotIdentityMismatch);
    }
    Ok(())
}

/// Parse a response only after its capture evidence has passed validation.
/// The returned object retains the original bytes and evidence so a collection
/// assessor can independently re-check both rather than trusting a mutable
/// normalized page object alone.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBoundData360Page {
    pub acquisition: Data360AcquisitionEvidence,
    pub context: Data360PageContext,
    pub response_body: Vec<u8>,
    pub parsed_page: ParsedData360Page,
}

pub fn parse_data360_page_with_evidence(
    response_body: &[u8],
    evidence: &Data360AcquisitionEvidence,
    context: &Data360PageContext,
) -> Result<EvidenceBoundData360Page, Data360AcquisitionError> {
    validate_data360_acquisition_evidence(response_body, evidence, context)?;
    let response_text = std::str::from_utf8(response_body)
        .map_err(|_| Data360AcquisitionError::ResponseBodyNotUtf8)?;
    let parsed_page = parse_data360_page(response_text, context)
        .map_err(|error| Data360AcquisitionError::PageParseFailed(error.to_string()))?;
    Ok(EvidenceBoundData360Page {
        acquisition: evidence.clone(),
        context: context.clone(),
        response_body: response_body.to_vec(),
        parsed_page,
    })
}

/// A collection-level gate that rechecks bytes, evidence, and normalized page
/// equality for each member before invoking the ordinary pagination assessor.
pub fn assess_data360_acquired_snapshot(
    pages: &[EvidenceBoundData360Page],
) -> Data360SnapshotAssessment {
    if pages.is_empty() {
        return assess_data360_snapshot(&[]);
    }
    let mut reparsed_pages = Vec::with_capacity(pages.len());
    for acquired in pages {
        if validate_data360_acquisition_evidence(
            &acquired.response_body,
            &acquired.acquisition,
            &acquired.context,
        )
        .is_err()
        {
            return Data360SnapshotAssessment {
                status: Data360SnapshotStatus::AcquisitionEvidenceInvalid,
                expected_count: None,
                rows_returned: 0,
                accepted_rows: 0,
                rejected_rows: 0,
                next_skip: 0,
            };
        }
        let Ok(response_text) = std::str::from_utf8(&acquired.response_body) else {
            return Data360SnapshotAssessment {
                status: Data360SnapshotStatus::AcquisitionEvidenceInvalid,
                expected_count: None,
                rows_returned: 0,
                accepted_rows: 0,
                rejected_rows: 0,
                next_skip: 0,
            };
        };
        let Ok(reparsed) = parse_data360_page(response_text, &acquired.context) else {
            return Data360SnapshotAssessment {
                status: Data360SnapshotStatus::AcquisitionEvidenceInvalid,
                expected_count: None,
                rows_returned: 0,
                accepted_rows: 0,
                rejected_rows: 0,
                next_skip: 0,
            };
        };
        if reparsed != acquired.parsed_page {
            return Data360SnapshotAssessment {
                status: Data360SnapshotStatus::AcquisitionEvidenceInvalid,
                expected_count: None,
                rows_returned: 0,
                accepted_rows: 0,
                rejected_rows: 0,
                next_skip: 0,
            };
        }
        reparsed_pages.push(reparsed);
    }
    assess_data360_snapshot(&reparsed_pages)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn context(skip: usize, top: usize, page_id: &str) -> Data360PageContext {
        Data360PageContext {
            query: Data360PageQuery {
                database_id: "WB_INFECDB".to_string(),
                indicator_id: "SYNTHETIC_INFECDB_DGE".to_string(),
                period_start: 1990,
                period_end: 1992,
                filters: BTreeMap::new(),
                requested_skip: skip,
                requested_top: top,
            },
            collection_snapshot_id: "synthetic-data360-collection-v1".to_string(),
            page_snapshot_id: page_id.to_string(),
            retrieved_at: "2026-10-13T12:00:00Z".to_string(),
        }
    }

    fn row(area: &str, period: &str, value: Option<&str>) -> Value {
        let mut row = serde_json::json!({
            "OBS_VALUE": value,
            "TIME_FORMAT": "P1Y",
            "UNIT_MULT": 0,
            "COMMENT_OBS": null,
            "OBS_STATUS": "A",
            "OBS_CONF": "PU",
            "AGG_METHOD": "_Z",
            "DECIMALS": "3",
            "COMMENT_TS": "Synthetic test-only series; not a World Bank observation",
            "DATA_SOURCE": "SYNTHETIC_FIXTURE",
            "LATEST_DATA": false,
            "DATABASE_ID": "WB_INFECDB",
            "INDICATOR": "SYNTHETIC_INFECDB_DGE",
            "REF_AREA": area,
            "SEX": "_T",
            "AGE": "_T",
            "URBANISATION": "_T",
            "COMP_BREAKDOWN_1": "_Z",
            "COMP_BREAKDOWN_2": "_Z",
            "COMP_BREAKDOWN_3": "_Z",
            "TIME_PERIOD": period,
            "FREQ": "A",
            "UNIT_MEASURE": "INDEX",
            "UNIT_TYPE": null
        });
        if value.is_none() {
            row["OBS_VALUE"] = Value::Null;
        }
        row
    }

    fn response(rows: Vec<Value>, count: usize) -> String {
        serde_json::json!({ "count": count, "value": rows }).to_string()
    }

    fn parse(
        rows: Vec<Value>,
        count: usize,
        skip: usize,
        top: usize,
        page_id: &str,
    ) -> ParsedData360Page {
        parse_data360_page(&response(rows, count), &context(skip, top, page_id)).unwrap()
    }

    fn acquisition_evidence(
        response_body: &[u8],
        context: &mut Data360PageContext,
    ) -> Data360AcquisitionEvidence {
        let digest = format!("{:x}", sha2::Sha256::digest(response_body));
        context.page_snapshot_id = format!("sha256:{digest}");
        let mut parameters = BTreeMap::new();
        parameters.insert("DATABASE_ID".to_string(), context.query.database_id.clone());
        parameters.insert("INDICATOR".to_string(), context.query.indicator_id.clone());
        parameters.insert("skip".to_string(), context.query.requested_skip.to_string());
        if let Some(year) = context.query.filters.get("TIME_PERIOD") {
            parameters.insert("TIME_PERIOD".to_string(), year.clone());
        } else {
            parameters.insert("timePeriodFrom".to_string(), context.query.period_start.to_string());
            parameters.insert("timePeriodTo".to_string(), context.query.period_end.to_string());
        }
        for (key, value) in &context.query.filters {
            parameters.insert(key.clone(), value.clone());
        }
        let mut url = url::Url::parse("https://data360api.worldbank.org/data360/data").unwrap();
        url.query_pairs_mut()
            .extend_pairs(parameters.iter().map(|(key, value)| (key.as_str(), value.as_str())));
        let response_headers = BTreeMap::from([
            ("content-type".to_string(), "application/json; charset=utf-8".to_string()),
            ("content-length".to_string(), response_body.len().to_string()),
            ("etag".to_string(), "\"synthetic-fixture\"".to_string()),
        ]);
        Data360AcquisitionEvidence {
            schema_version: DATA360_ACQUISITION_EVIDENCE_SCHEMA_VERSION,
            request_method: "GET".to_string(),
            request_url: url.to_string(),
            request_parameters: parameters,
            response_status: 200,
            response_headers,
            response_body_byte_length: response_body.len() as u64,
            response_body_sha256: digest,
            retrieved_at: context.retrieved_at.clone(),
        }
    }

    #[test]
    fn acquisition_evidence_binds_exact_bytes_query_and_response_metadata() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let evidence = acquisition_evidence(body, &mut context);
        let acquired = parse_data360_page_with_evidence(body, &evidence, &context).unwrap();
        assert_eq!(acquired.parsed_page.rows_returned, 2);
        assert_eq!(
            acquired.parsed_page.page_snapshot_id,
            format!("sha256:{}", evidence.response_body_sha256)
        );
        assert_eq!(
            assess_data360_acquired_snapshot(&[acquired]).status,
            Data360SnapshotStatus::IncompletePageSet
        );
    }

    #[test]
    fn acquired_page_rejects_body_tampering_even_when_length_is_unchanged() {
        let mut body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json")
            .as_bytes().to_vec();
        let mut context = context(0, 2, "unused");
        let evidence = acquisition_evidence(&body, &mut context);
        let index = body.windows(6).position(|window| window == b"12.340").unwrap();
        body[index + 5] = b'1';
        assert_eq!(
            validate_data360_acquisition_evidence(&body, &evidence, &context),
            Err(Data360AcquisitionError::ResponseBodyDigestMismatch)
        );
    }

    #[test]
    fn acquisition_requires_url_parameters_to_match_captured_parameter_map() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let mut evidence = acquisition_evidence(body, &mut context);
        evidence.request_parameters.insert("skip".to_string(), "1".to_string());
        assert_eq!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::RequestUrlParameterMismatch)
        );
    }

    #[test]
    fn acquisition_requires_the_official_https_observation_endpoint() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let mut evidence = acquisition_evidence(body, &mut context);
        evidence.request_url = evidence.request_url.replace("/data360/data?", "/data360/indicators?");
        assert_eq!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::InvalidRequestUrl)
        );
    }

    #[test]
    fn acquisition_rejects_non_success_http_status() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let mut evidence = acquisition_evidence(body, &mut context);
        evidence.response_status = 503;
        assert_eq!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::NonSuccessResponseStatus(503))
        );
    }

    #[test]
    fn acquisition_rejects_unapproved_or_sensitive_response_headers() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let mut evidence = acquisition_evidence(body, &mut context);
        evidence.response_headers.insert("set-cookie".to_string(), "session=secret".to_string());
        assert!(matches!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::InvalidResponseHeaders(_))
        ));
    }

    #[test]
    fn acquisition_rejects_inconsistent_identity_content_length_header() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let mut evidence = acquisition_evidence(body, &mut context);
        evidence.response_headers.insert(
            "content-length".to_string(),
            (body.len() + 1).to_string(),
        );
        assert_eq!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::ContentLengthHeaderMismatch)
        );
    }

    #[test]
    fn acquisition_requires_page_identity_to_be_the_body_digest() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let evidence = acquisition_evidence(body, &mut context);
        context.page_snapshot_id = "manually-assigned-id".to_string();
        assert_eq!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::PageSnapshotIdentityMismatch)
        );
    }

    #[test]
    fn acquisition_filters_must_match_the_captured_request() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let evidence = acquisition_evidence(body, &mut context);
        context.query.filters.insert("REF_AREA".to_string(), "ZAF".to_string());
        assert_eq!(
            validate_data360_acquisition_evidence(body, &evidence, &context),
            Err(Data360AcquisitionError::MissingOrMismatchedRequestParameter("REF_AREA".to_string()))
        );
    }

    #[test]
    fn acquired_snapshot_rejects_mutated_normalized_page_after_capture() {
        let body = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json").as_bytes();
        let mut context = context(0, 2, "unused");
        let evidence = acquisition_evidence(body, &mut context);
        let mut acquired = parse_data360_page_with_evidence(body, &evidence, &context).unwrap();
        acquired.parsed_page.records[0].period = 1992;
        assert_eq!(
            assess_data360_acquired_snapshot(&[acquired]).status,
            Data360SnapshotStatus::AcquisitionEvidenceInvalid
        );
    }

    #[test]
    fn parses_synthetic_source_shaped_page_and_preserves_exact_value_text() {
        let fixture = include_str!("../tests/fixtures/synthetic_world_bank_data360_page_v1.json");
        let page = parse_data360_page(fixture, &context(0, 2, "synthetic-page-0")).unwrap();
        assert_eq!(page.source_reported_count, 3);
        assert_eq!(page.rows_returned, 2);
        assert_eq!(page.records.len(), 2);
        assert_eq!(page.records[0].value_source_text.as_deref(), Some("12.340"));
        assert_eq!(page.records[0].reference_area, "AAA");
        assert!(page.may_have_more_pages());
        assert!(!page.completeness_proven());
        assert!(page.rejected_rows.is_empty());
    }

    #[test]
    fn explicit_null_observation_value_is_missing_not_zero() {
        let page = parse(vec![row("AAA", "1990", None)], 1, 0, 10, "null-value-page");
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].value_source_text, None);
    }

    #[test]
    fn count_smaller_than_page_rows_fails_closed() {
        let result = parse_data360_page(
            &response(vec![row("AAA", "1990", Some("1.0"))], 0),
            &context(0, 10, "bad-count"),
        );
        assert_eq!(
            result,
            Err(Data360PageError::CountLessThanRowsReturned {
                reported: 0,
                actual: 1
            })
        );
    }

    #[test]
    fn unknown_envelope_fields_are_not_silently_ignored() {
        let mut envelope: Value = serde_json::from_str(&response(vec![], 0)).unwrap();
        envelope["nextLink"] = Value::String("unqualified".to_string());
        assert!(matches!(
            parse_data360_page(&envelope.to_string(), &context(0, 10, "unknown-envelope")),
            Err(Data360PageError::EnvelopeSchema(_))
        ));
    }

    #[test]
    fn request_cannot_exceed_documented_page_limit() {
        let result = parse_data360_page(&response(vec![], 0), &context(0, 1001, "too-large"));
        assert!(matches!(result, Err(Data360PageError::InvalidContext(_))));
    }

    #[test]
    fn mismatched_database_indicator_and_period_are_rejected_rows() {
        let mut wrong_database = row("AAA", "1990", Some("1"));
        wrong_database["DATABASE_ID"] = Value::String("OTHER".to_string());
        let mut wrong_indicator = row("BBB", "1990", Some("2"));
        wrong_indicator["INDICATOR"] = Value::String("OTHER_INDICATOR".to_string());
        let wrong_period = row("CCC", "1993", Some("3"));
        let page = parse(
            vec![wrong_database, wrong_indicator, wrong_period],
            3,
            0,
            10,
            "mismatch-page",
        );
        assert!(page.records.is_empty());
        assert_eq!(page.rejected_rows.len(), 3);
        assert!(matches!(
            page.rejected_rows[0].reason,
            Data360RowRejection::DatabaseMismatch
        ));
        assert!(matches!(
            page.rejected_rows[1].reason,
            Data360RowRejection::IndicatorMismatch
        ));
        assert!(matches!(
            page.rejected_rows[2].reason,
            Data360RowRejection::PeriodOutOfQueryRange
        ));
    }

    #[test]
    fn unexpected_record_fields_are_quarantined() {
        let mut extra = row("AAA", "1990", Some("1"));
        extra["unexpected"] = Value::Bool(true);
        let page = parse(vec![extra], 1, 0, 10, "extra-row-field");
        assert!(page.records.is_empty());
        assert_eq!(
            page.rejected_rows[0].reason,
            Data360RowRejection::UnknownField("unexpected".to_string())
        );
    }

    #[test]
    fn required_record_fields_are_enforced() {
        let mut missing = row("AAA", "1990", Some("1"));
        missing.as_object_mut().unwrap().remove("OBS_STATUS");
        let page = parse(vec![missing], 1, 0, 10, "missing-field");
        assert!(page.records.is_empty());
        assert_eq!(
            page.rejected_rows[0].reason,
            Data360RowRejection::MissingField("OBS_STATUS".to_string())
        );
    }

    #[test]
    fn value_must_use_the_documented_text_token() {
        let mut numeric = row("AAA", "1990", Some("1"));
        numeric["OBS_VALUE"] = Value::from(1);
        let page = parse(vec![numeric], 1, 0, 10, "numeric-value");
        assert!(page.records.is_empty());
        assert_eq!(
            page.rejected_rows[0].reason,
            Data360RowRejection::InvalidFieldType("OBS_VALUE".to_string())
        );
    }

    #[test]
    fn duplicate_keys_within_a_page_are_rejected() {
        let one = row("AAA", "1990", Some("1"));
        let two = row("AAA", "1990", Some("2"));
        let page = parse(vec![one, two], 2, 0, 10, "duplicate-page");
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.rejected_rows.len(), 1);
        assert!(matches!(
            page.rejected_rows[0].reason,
            Data360RowRejection::DuplicateRecordKey(_)
        ));
    }

    #[test]
    fn contiguous_pages_with_stable_count_qualify_complete_collection() {
        let first = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            3,
            0,
            2,
            "page-0",
        );
        let second = parse(vec![row("CCC", "1991", Some("3"))], 3, 2, 2, "page-1");
        let assessment = assess_data360_snapshot(&[first, second]);
        assert_eq!(
            assessment.status,
            Data360SnapshotStatus::CompleteAgainstStableCount
        );
        assert_eq!(assessment.expected_count, Some(3));
        assert_eq!(assessment.rows_returned, 3);
        assert_eq!(assessment.accepted_rows, 3);
        assert_eq!(assessment.rejected_rows, 0);
        assert_eq!(assessment.next_skip, 3);
    }

    #[test]
    fn collection_assessor_rechecks_public_page_accounting() {
        let mut page = parse(vec![row("AAA", "1990", Some("1"))], 1, 0, 10, "accounting");
        page.rows_returned = 0;
        assert_eq!(
            assess_data360_snapshot(&[page]).status,
            Data360SnapshotStatus::PageAccountingMismatch
        );
    }

    #[test]
    fn unsupported_public_page_schema_cannot_qualify() {
        let mut page = parse(vec![row("AAA", "1990", Some("1"))], 1, 0, 10, "schema");
        page.schema_version = DATA360_SCHEMA_VERSION + 1;
        assert_eq!(
            assess_data360_snapshot(&[page]).status,
            Data360SnapshotStatus::UnsupportedSchemaVersion
        );
    }

    #[test]
    fn assessment_rejects_mutated_normalized_values() {
        let mut page = parse(
            vec![row("AAA", "1990", Some("1"))],
            1,
            0,
            10,
            "mutated-value",
        );
        page.records[0].period = 1992;
        assert_eq!(
            assess_data360_snapshot(&[page]).status,
            Data360SnapshotStatus::PageAccountingMismatch
        );
    }

    #[test]
    fn assessment_rejects_duplicate_or_missing_row_indices() {
        let mut page = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            2,
            0,
            10,
            "mutated-index",
        );
        let first_row_index = page.records[0].row_index;
        page.records[1].row_index = first_row_index;
        assert_eq!(
            assess_data360_snapshot(&[page]).status,
            Data360SnapshotStatus::PageAccountingMismatch
        );
    }

    #[test]
    fn missing_pages_do_not_qualify_complete_collection() {
        let first = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            3,
            0,
            2,
            "page-0",
        );
        assert_eq!(
            assess_data360_snapshot(&[first]).status,
            Data360SnapshotStatus::IncompletePageSet
        );
    }

    #[test]
    fn reusing_page_snapshot_identity_prevents_complete_collection() {
        let first = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            3,
            0,
            2,
            "same-page-snapshot",
        );
        let second = parse(
            vec![row("CCC", "1991", Some("3"))],
            3,
            2,
            2,
            "same-page-snapshot",
        );
        assert_eq!(
            assess_data360_snapshot(&[first, second]).status,
            Data360SnapshotStatus::DuplicatePageSnapshotIdentity
        );
    }

    #[test]
    fn page_count_drift_prevents_complete_collection() {
        let first = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            3,
            0,
            2,
            "page-0",
        );
        let second = parse(vec![row("CCC", "1991", Some("3"))], 4, 2, 2, "page-1");
        assert_eq!(
            assess_data360_snapshot(&[first, second]).status,
            Data360SnapshotStatus::SourceCountDrift
        );
    }

    #[test]
    fn offset_gap_prevents_complete_collection() {
        let first = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            3,
            0,
            2,
            "page-0",
        );
        let second = parse(vec![row("CCC", "1991", Some("3"))], 3, 3, 2, "page-1");
        assert_eq!(
            assess_data360_snapshot(&[first, second]).status,
            Data360SnapshotStatus::OffsetGapOrOverlap
        );
    }

    #[test]
    fn rejected_rows_prevent_complete_collection() {
        let mut invalid = row("AAA", "1990", Some("1"));
        invalid["DATABASE_ID"] = Value::String("OTHER".to_string());
        let first = parse(
            vec![invalid, row("BBB", "1990", Some("2"))],
            2,
            0,
            2,
            "page-0",
        );
        assert_eq!(
            assess_data360_snapshot(&[first]).status,
            Data360SnapshotStatus::RowsRejected
        );
    }

    #[test]
    fn declared_dimension_filter_is_checked_against_each_row() {
        let mut query = context(0, 10, "dimension-filter").clone();
        query
            .query
            .filters
            .insert("REF_AREA".to_string(), "ZAF".to_string());
        let page = parse_data360_page(
            &response(
                vec![row("ZAF", "1990", Some("1")), row("AAA", "1990", Some("2"))],
                2,
            ),
            &query,
        )
        .unwrap();
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].reference_area, "ZAF");
        assert_eq!(page.rejected_rows.len(), 1);
        assert_eq!(
            page.rejected_rows[0].reason,
            Data360RowRejection::QueryFilterMismatch("REF_AREA".to_string())
        );
    }

    #[test]
    fn unsupported_query_filter_fails_closed() {
        let mut query = context(0, 10, "unknown-filter");
        query
            .query
            .filters
            .insert("COUNTRY_NAME".to_string(), "South Africa".to_string());
        assert!(matches!(
            parse_data360_page(&response(vec![], 0), &query),
            Err(Data360PageError::InvalidContext(_))
        ));
    }

    #[test]
    fn annual_query_period_cannot_exceed_four_digits() {
        let mut query = context(0, 10, "five-digit-period");
        query.query.period_end = 10000;
        assert!(matches!(
            parse_data360_page(&response(vec![], 0), &query),
            Err(Data360PageError::InvalidContext(_))
        ));
    }

    #[test]
    fn undocumented_is_latest_data_filter_is_rejected() {
        let mut query = context(0, 10, "undocumented-latest-filter");
        query
            .query
            .filters
            .insert("isLatestData".to_string(), "true".to_string());
        assert!(matches!(
            parse_data360_page(&response(vec![], 0), &query),
            Err(Data360PageError::InvalidContext(_))
        ));
    }

    #[test]
    fn differing_dimension_filters_cannot_form_one_complete_collection() {
        let mut first_context = context(0, 1, "filter-page-0");
        first_context
            .query
            .filters
            .insert("REF_AREA".to_string(), "AAA".to_string());
        let first = parse_data360_page(
            &response(vec![row("AAA", "1990", Some("1"))], 2),
            &first_context,
        )
        .unwrap();

        let mut second_context = context(1, 1, "filter-page-1");
        second_context
            .query
            .filters
            .insert("REF_AREA".to_string(), "BBB".to_string());
        let second = parse_data360_page(
            &response(vec![row("BBB", "1991", Some("2"))], 2),
            &second_context,
        )
        .unwrap();

        assert_eq!(
            assess_data360_snapshot(&[first, second]).status,
            Data360SnapshotStatus::QueryMismatch
        );
    }

    #[test]
    fn duplicate_record_across_pages_prevents_complete_collection() {
        let first = parse(
            vec![row("AAA", "1990", Some("1")), row("BBB", "1990", Some("2"))],
            3,
            0,
            2,
            "page-0",
        );
        let second = parse(vec![row("AAA", "1990", Some("3"))], 3, 2, 2, "page-1");
        assert_eq!(
            assess_data360_snapshot(&[first, second]).status,
            Data360SnapshotStatus::DuplicateRecordKey
        );
    }
}
