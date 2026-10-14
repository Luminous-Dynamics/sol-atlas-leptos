// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence binding for captured World Bank Data360 HTTP responses.
//!
//! No network I/O is performed here. The caller must pass request/response
//! metadata exposed by its HTTP client and the exact response body bytes. This
//! detects mismatched artifacts but is not a signature proving that the caller
//! truthfully reported the HTTP exchange.

use crate::world_bank_data360::{
    DATA360_MAX_PAGE_SIZE, Data360PageContext, Data360PageError, ParsedData360Page,
    parse_data360_page,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub const DATA360_EVIDENCE_SCHEMA_VERSION: u16 = 1;
const DATA360_DATA_URL: &str = "https://data360api.worldbank.org/data360/data";
const DOCUMENTED_QUERY_KEYS: [&str; 16] = [
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
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data360AcquisitionEvidence {
    pub schema_version: u16,
    pub request_method: String,
    pub request_url: String,
    pub response_status: u16,
    /// Header names are normalized to lowercase; values are trimmed.
    pub response_headers: BTreeMap<String, String>,
    pub retrieved_at: String,
    pub body_len_bytes: usize,
    /// Lowercase 64-character SHA-256 of the exact captured response bytes.
    pub body_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data360EvidenceError {
    InvalidRequest(String),
    InvalidResponse(String),
    DuplicateHeader(String),
    BodyLengthMismatch { recorded: usize, actual: usize },
    BodyDigestMismatch,
    PageIdentityMismatch,
    RetrievalTimeMismatch,
    InvalidUtf8Body,
    PageParse(String),
}

impl fmt::Display for Data360EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => {
                write!(f, "invalid Data360 acquisition request: {message}")
            }
            Self::InvalidResponse(message) => {
                write!(f, "invalid Data360 acquisition response: {message}")
            }
            Self::DuplicateHeader(name) => {
                write!(f, "duplicate normalized response header {name:?}")
            }
            Self::BodyLengthMismatch { recorded, actual } => write!(
                f,
                "Data360 response byte length mismatch: recorded={recorded}, actual={actual}"
            ),
            Self::BodyDigestMismatch => {
                write!(f, "Data360 response SHA-256 does not match captured bytes")
            }
            Self::PageIdentityMismatch => write!(
                f,
                "page_snapshot_id is not the SHA-256 identity of the captured response body"
            ),
            Self::RetrievalTimeMismatch => write!(
                f,
                "response evidence retrieval time does not match page context"
            ),
            Self::InvalidUtf8Body => write!(f, "Data360 JSON response body is not valid UTF-8"),
            Self::PageParse(message) => write!(
                f,
                "Data360 page parser rejected the captured response: {message}"
            ),
        }
    }
}
impl Error for Data360EvidenceError {}

impl Data360AcquisitionEvidence {
    /// Build a receipt from request/response metadata and exact raw response
    /// bytes. Non-200 responses can be recorded but cannot qualify for parsing.
    pub fn capture(
        request_method: impl Into<String>,
        request_url: impl Into<String>,
        response_status: u16,
        response_headers: BTreeMap<String, String>,
        retrieved_at: impl Into<String>,
        response_body: &[u8],
    ) -> Result<Self, Data360EvidenceError> {
        let mut normalized_headers = BTreeMap::new();
        for (name, value) in response_headers {
            let normalized_name = name.trim().to_ascii_lowercase();
            if normalized_name.is_empty() || value.bytes().any(|byte| byte == 0x0d || byte == 0x0a)
            {
                return Err(Data360EvidenceError::InvalidResponse(
                    "response headers must have non-empty names and single-line values".to_string(),
                ));
            }
            if normalized_headers
                .insert(normalized_name.clone(), value.trim().to_string())
                .is_some()
            {
                return Err(Data360EvidenceError::DuplicateHeader(normalized_name));
            }
        }

        Ok(Self {
            schema_version: DATA360_EVIDENCE_SCHEMA_VERSION,
            request_method: request_method.into(),
            request_url: request_url.into(),
            response_status,
            response_headers: normalized_headers,
            retrieved_at: retrieved_at.into(),
            body_len_bytes: response_body.len(),
            body_sha256: sha256_hex(response_body),
        })
    }

    /// Content-address the exact body for use as page_snapshot_id.
    pub fn page_snapshot_id(&self) -> String {
        format!("sha256:{}", self.body_sha256)
    }

    /// Verify that evidence binds the raw bytes, request query and parser context.
    pub fn validate(
        &self,
        response_body: &[u8],
        context: &Data360PageContext,
    ) -> Result<(), Data360EvidenceError> {
        if self.schema_version != DATA360_EVIDENCE_SCHEMA_VERSION {
            return Err(Data360EvidenceError::InvalidResponse(format!(
                "unsupported evidence schema version {}; expected {}",
                self.schema_version, DATA360_EVIDENCE_SCHEMA_VERSION
            )));
        }
        if self.request_method != "GET" {
            return Err(Data360EvidenceError::InvalidRequest(
                "observation acquisition must use GET".to_string(),
            ));
        }
        if self.retrieved_at.trim().is_empty() {
            return Err(Data360EvidenceError::InvalidResponse(
                "retrieved_at must be non-empty".to_string(),
            ));
        }
        if self.retrieved_at != context.retrieved_at {
            return Err(Data360EvidenceError::RetrievalTimeMismatch);
        }
        if self.response_status != 200 {
            return Err(Data360EvidenceError::InvalidResponse(format!(
                "expected HTTP 200, received {}",
                self.response_status
            )));
        }

        let content_type = self.response_headers.get("content-type").ok_or_else(|| {
            Data360EvidenceError::InvalidResponse(
                "missing content-type response header".to_string(),
            )
        })?;
        let media_type = content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if media_type != "application/json" {
            return Err(Data360EvidenceError::InvalidResponse(format!(
                "expected application/json, received {content_type:?}"
            )));
        }
        if let Some(length) = self.response_headers.get("content-length") {
            let parsed_length = length.parse::<usize>().map_err(|_| {
                Data360EvidenceError::InvalidResponse(
                    "content-length must be a non-negative integer".to_string(),
                )
            })?;
            if parsed_length != response_body.len() {
                return Err(Data360EvidenceError::BodyLengthMismatch {
                    recorded: parsed_length,
                    actual: response_body.len(),
                });
            }
        }
        if self.body_len_bytes != response_body.len() {
            return Err(Data360EvidenceError::BodyLengthMismatch {
                recorded: self.body_len_bytes,
                actual: response_body.len(),
            });
        }
        if self.body_sha256 != sha256_hex(response_body) {
            return Err(Data360EvidenceError::BodyDigestMismatch);
        }
        if context.page_snapshot_id != self.page_snapshot_id() {
            return Err(Data360EvidenceError::PageIdentityMismatch);
        }

        validate_documented_request_url(&self.request_url, context)
    }
}

/// Evidence-aware parsing is the preferred entry point for a captured page.
pub fn parse_data360_page_with_evidence(
    response_body: &[u8],
    context: &Data360PageContext,
    evidence: &Data360AcquisitionEvidence,
) -> Result<ParsedData360Page, Data360EvidenceError> {
    evidence.validate(response_body, context)?;
    let body =
        std::str::from_utf8(response_body).map_err(|_| Data360EvidenceError::InvalidUtf8Body)?;
    parse_data360_page(body, context)
        .map_err(|error: Data360PageError| Data360EvidenceError::PageParse(error.to_string()))
}

fn validate_documented_request_url(
    request_url: &str,
    context: &Data360PageContext,
) -> Result<(), Data360EvidenceError> {
    if request_url.contains('#') {
        return Err(Data360EvidenceError::InvalidRequest(
            "URL fragments are not part of an HTTP request target".to_string(),
        ));
    }
    let suffix = request_url.strip_prefix(DATA360_DATA_URL).ok_or_else(|| {
        Data360EvidenceError::InvalidRequest(
            "request URL must use the documented HTTPS Data360 data endpoint".to_string(),
        )
    })?;
    if !suffix.is_empty() && !suffix.starts_with('?') {
        return Err(Data360EvidenceError::InvalidRequest(
            "request URL path must exactly match /data360/data".to_string(),
        ));
    }
    let query = suffix.strip_prefix('?').unwrap_or("");
    if query.is_empty() {
        return Err(Data360EvidenceError::InvalidRequest(
            "request URL must record its acquisition query parameters".to_string(),
        ));
    }

    let mut actual = BTreeMap::<String, String>::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            return Err(Data360EvidenceError::InvalidRequest(
                "query string contains an empty parameter".to_string(),
            ));
        }
        let (raw_name, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = percent_decode(raw_name)?;
        let value = percent_decode(raw_value)?;
        if !DOCUMENTED_QUERY_KEYS.contains(&name.as_str()) && name != "skip" {
            return Err(Data360EvidenceError::InvalidRequest(format!(
                "query parameter {name:?} is not declared by the current /data360/data operation"
            )));
        }
        if actual.insert(name.clone(), value).is_some() {
            return Err(Data360EvidenceError::InvalidRequest(format!(
                "query parameter {name:?} is repeated"
            )));
        }
    }

    let q = &context.query;
    require_query_value(&actual, "DATABASE_ID", &q.database_id)?;
    require_query_value(&actual, "INDICATOR", &q.indicator_id)?;
    require_query_value(&actual, "timePeriodFrom", &q.period_start.to_string())?;
    require_query_value(&actual, "timePeriodTo", &q.period_end.to_string())?;

    match actual.get("skip") {
        Some(value) if value == &q.requested_skip.to_string() => {}
        None if q.requested_skip == 0 => {}
        _ => {
            return Err(Data360EvidenceError::InvalidRequest(
                "skip query parameter does not match requested_skip".to_string(),
            ));
        }
    }
    for (name, expected) in &q.filters {
        require_query_value(&actual, name, expected)?;
    }
    for name in actual.keys() {
        if matches!(
            name.as_str(),
            "DATABASE_ID" | "INDICATOR" | "timePeriodFrom" | "timePeriodTo" | "skip"
        ) {
            continue;
        }
        if !q.filters.contains_key(name) {
            return Err(Data360EvidenceError::InvalidRequest(format!(
                "request contains unrecorded query filter {name:?}"
            )));
        }
    }

    // The GET /data360/data operation does not reference the reusable top
    // parameter component. Use its documented maximum of 1,000 with skip.
    if q.requested_top != DATA360_MAX_PAGE_SIZE {
        return Err(Data360EvidenceError::InvalidRequest(format!(
            "evidence-bound acquisition must use the documented {DATA360_MAX_PAGE_SIZE}-row page ceiling"
        )));
    }
    Ok(())
}

fn require_query_value(
    actual: &BTreeMap<String, String>,
    name: &str,
    expected: &str,
) -> Result<(), Data360EvidenceError> {
    if actual.get(name).map(String::as_str) == Some(expected) {
        Ok(())
    } else {
        Err(Data360EvidenceError::InvalidRequest(format!(
            "query parameter {name:?} is missing or does not match parser context"
        )))
    }
}

fn percent_decode(input: &str) -> Result<String, Data360EvidenceError> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(Data360EvidenceError::InvalidRequest(
                        "query parameter has a truncated percent escape".to_string(),
                    ));
                }
                let high = hex_nibble(bytes[index + 1]).ok_or_else(|| {
                    Data360EvidenceError::InvalidRequest(
                        "query parameter has an invalid percent escape".to_string(),
                    )
                })?;
                let low = hex_nibble(bytes[index + 2]).ok_or_else(|| {
                    Data360EvidenceError::InvalidRequest(
                        "query parameter has an invalid percent escape".to_string(),
                    )
                })?;
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|_| {
        Data360EvidenceError::InvalidRequest(
            "query parameter is not valid UTF-8 after percent decoding".to_string(),
        )
    })
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_bank_data360::Data360PageQuery;

    const RETRIEVED_AT: &str = "2026-10-14T02:00:00Z";
    const BASE_URL: &str = "https://data360api.worldbank.org/data360/data?DATABASE_ID=WB_INFECDB&INDICATOR=SYNTHETIC_INFECDB_DGE&timePeriodFrom=1990&timePeriodTo=1990";

    fn body() -> Vec<u8> {
        br#"{"count":1,"value":[{"OBS_VALUE":"12.340","TIME_FORMAT":"P1Y","UNIT_MULT":0,"COMMENT_OBS":null,"OBS_STATUS":"A","OBS_CONF":"PU","AGG_METHOD":"_Z","DECIMALS":"3","COMMENT_TS":"Synthetic test-only series; not a World Bank observation","DATA_SOURCE":"SYNTHETIC_FIXTURE","LATEST_DATA":false,"DATABASE_ID":"WB_INFECDB","INDICATOR":"SYNTHETIC_INFECDB_DGE","REF_AREA":"AAA","SEX":"_T","AGE":"_T","URBANISATION":"_T","COMP_BREAKDOWN_1":"_Z","COMP_BREAKDOWN_2":"_Z","COMP_BREAKDOWN_3":"_Z","TIME_PERIOD":"1990","FREQ":"A","UNIT_MEASURE":"INDEX","UNIT_TYPE":null}]}"#.to_vec()
    }

    fn context(page_id: &str) -> Data360PageContext {
        Data360PageContext {
            collection_snapshot_id: "synthetic-acquisition-test-v1".to_string(),
            page_snapshot_id: page_id.to_string(),
            retrieved_at: RETRIEVED_AT.to_string(),
            query: Data360PageQuery {
                database_id: "WB_INFECDB".to_string(),
                indicator_id: "SYNTHETIC_INFECDB_DGE".to_string(),
                period_start: 1990,
                period_end: 1990,
                filters: BTreeMap::new(),
                requested_skip: 0,
                requested_top: DATA360_MAX_PAGE_SIZE,
            },
        }
    }

    fn receipt(bytes: &[u8], url: &str, status: u16) -> Data360AcquisitionEvidence {
        let mut headers = BTreeMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        headers.insert("Content-Length".to_string(), bytes.len().to_string());
        Data360AcquisitionEvidence::capture("GET", url, status, headers, RETRIEVED_AT, bytes)
            .unwrap()
    }

    #[test]
    fn sha256_matches_published_known_vector() {
        assert_eq!(
            sha256_hex(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn evidence_binds_raw_body_request_context_and_parse() {
        let bytes = body();
        let evidence = receipt(&bytes, BASE_URL, 200);
        let context = context(&evidence.page_snapshot_id());
        let parsed = parse_data360_page_with_evidence(&bytes, &context, &evidence).unwrap();
        assert_eq!(
            parsed.records[0].value_source_text.as_deref(),
            Some("12.340")
        );
    }

    #[test]
    fn modified_response_body_cannot_reuse_receipt() {
        let bytes = body();
        let evidence = receipt(&bytes, BASE_URL, 200);
        let context = context(&evidence.page_snapshot_id());
        let mut altered_length = bytes.clone();
        altered_length.push(b' ');
        assert!(matches!(
            evidence.validate(&altered_length, &context),
            Err(Data360EvidenceError::BodyLengthMismatch { .. })
        ));

        let mut altered_same_length = bytes.clone();
        altered_same_length[0] = b'[';
        assert_eq!(
            evidence.validate(&altered_same_length, &context),
            Err(Data360EvidenceError::BodyDigestMismatch)
        );
    }

    #[test]
    fn only_http_200_qualifies_for_parsing() {
        let bytes = body();
        let evidence = receipt(&bytes, BASE_URL, 503);
        let context = context(&evidence.page_snapshot_id());
        assert!(matches!(
            evidence.validate(&bytes, &context),
            Err(Data360EvidenceError::InvalidResponse(_))
        ));
    }

    #[test]
    fn content_length_and_media_type_are_enforced() {
        let bytes = body();
        let mut evidence = receipt(&bytes, BASE_URL, 200);
        let context = context(&evidence.page_snapshot_id());
        evidence
            .response_headers
            .insert("content-length".to_string(), "999".to_string());
        assert!(matches!(
            evidence.validate(&bytes, &context),
            Err(Data360EvidenceError::BodyLengthMismatch { .. })
        ));
        evidence.response_headers.remove("content-length");
        evidence
            .response_headers
            .insert("content-type".to_string(), "text/html".to_string());
        assert!(matches!(
            evidence.validate(&bytes, &context),
            Err(Data360EvidenceError::InvalidResponse(_))
        ));
    }

    #[test]
    fn repeated_or_unrecorded_query_parameters_fail_closed() {
        let bytes = body();
        let repeated = receipt(&bytes, &format!("{BASE_URL}&skip=0&skip=0"), 200);
        let ctx = context(&repeated.page_snapshot_id());
        assert!(matches!(
            repeated.validate(&bytes, &ctx),
            Err(Data360EvidenceError::InvalidRequest(message)) if message.contains("repeated")
        ));

        let extra = receipt(&bytes, &format!("{BASE_URL}&REF_AREA=AAA"), 200);
        let ctx = context(&extra.page_snapshot_id());
        assert!(matches!(
            extra.validate(&bytes, &ctx),
            Err(Data360EvidenceError::InvalidRequest(message)) if message.contains("unrecorded query filter")
        ));
    }

    #[test]
    fn page_id_and_retrieval_time_must_match_receipt() {
        let bytes = body();
        let evidence = receipt(&bytes, BASE_URL, 200);
        assert_eq!(
            evidence.validate(&bytes, &context("not-the-body-digest")),
            Err(Data360EvidenceError::PageIdentityMismatch)
        );
        let mut ctx = context(&evidence.page_snapshot_id());
        ctx.retrieved_at = "2026-10-14T03:00:00Z".to_string();
        assert_eq!(
            evidence.validate(&bytes, &ctx),
            Err(Data360EvidenceError::RetrievalTimeMismatch)
        );
    }

    #[test]
    fn receipt_serialization_is_versioned_and_rejects_unknown_fields() {
        let bytes = body();
        let original = receipt(&bytes, BASE_URL, 200);
        let encoded = serde_json::to_string(&original).unwrap();
        let decoded: Data360AcquisitionEvidence = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, original);

        let mut unknown: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        unknown["future_field"] = serde_json::json!("must not be dropped");
        assert!(serde_json::from_value::<Data360AcquisitionEvidence>(unknown).is_err());

        let mut unsupported = original;
        unsupported.schema_version += 1;
        let ctx = context(&unsupported.page_snapshot_id());
        assert!(matches!(
            unsupported.validate(&bytes, &ctx),
            Err(Data360EvidenceError::InvalidResponse(message)) if message.contains("unsupported evidence schema version")
        ));
    }

    #[test]
    fn recorded_filter_matches_request_and_normalized_row() {
        let original = String::from_utf8(body()).unwrap();
        let bytes = original
            .replace("\\\"REF_AREA\\\":\\\"AAA\\\"", "\\\"REF_AREA\\\":\\\"ZAF\\\"")
            .into_bytes();
        let url = format!("{BASE_URL}&REF_AREA=ZAF");
        let evidence = receipt(&bytes, &url, 200);
        let mut ctx = context(&evidence.page_snapshot_id());
        ctx.query.filters.insert("REF_AREA".to_string(), "ZAF".to_string());
        let parsed = parse_data360_page_with_evidence(&bytes, &ctx, &evidence).unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].reference_area, "ZAF");
    }

    #[test]
    fn query_offset_must_match_page_context() {
        let bytes = body();
        let url = format!("{BASE_URL}&skip=2");
        let evidence = receipt(&bytes, &url, 200);
        let mut ctx = context(&evidence.page_snapshot_id());
        ctx.query.requested_skip = 1;
        assert!(matches!(
            evidence.validate(&bytes, &ctx),
            Err(Data360EvidenceError::InvalidRequest(message))
                if message.contains("skip query parameter")
        ));
    }

    #[test]
    fn undocumented_top_parameter_is_rejected() {
        let bytes = body();
        let evidence = receipt(&bytes, &format!("{BASE_URL}&top=1000"), 200);
        let ctx = context(&evidence.page_snapshot_id());
        assert!(matches!(
            evidence.validate(&bytes, &ctx),
            Err(Data360EvidenceError::InvalidRequest(message)) if message.contains("not declared")
        ));
    }
}
