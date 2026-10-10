// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fail-closed parser for one-shot JPL Horizons CSV vector tables.
//!
//! The caller owns HTTP, URL canonicalization, and SHA-256 hashing. This module
//! accepts the unmodified JSON response and explicit request/provenance context;
//! it never performs network access. This first increment deliberately supports
//! TDB vectors in KM-S units with vector table 2 and CSV output only.

use serde::Deserialize;

use crate::system_catalog::{
    catalog_object, CatalogObject, EphemerisProvenance, ReferencePlane, ReferenceSystem,
    StateVector, TimeScale, VectorCorrection,
};

/// Conservative URL ceiling; larger batches should use the file-based Horizons
/// API rather than risk intermediaries rejecting an oversized GET request.
const MAX_CANONICAL_GET_URL_BYTES: usize = 7_500;

// The published GET docs are versioned 1.3 but their JSON examples show
// signature.version 1.0. The file API page is versioned 1.0 but shows examples
// with signature.version 0.2. Until byte-for-byte provider captures resolve this
// documentation drift, accept only each endpoint's explicitly documented values.
const SUPPORTED_GET_API_SIGNATURE_VERSIONS: &[&str] = &["1.0", "1.3"];
const SUPPORTED_FILE_API_SIGNATURE_VERSIONS: &[&str] = &["0.2", "1.0"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HorizonsParseError {
    InvalidJson(String),
    ProviderError(String),
    MissingResult,
    MissingSignature,
    UnexpectedApiSource(String),
    UnsupportedApiVersion(String),
    InvalidRequest(&'static str),
    ProvenanceMismatch,
    UnsupportedTimeScale,
    MissingStartMarker,
    MissingEndMarker,
    InvalidMarkerOrder,
    MissingMetadata(&'static str),
    UnexpectedMetadata {
        field: &'static str,
        expected: String,
        actual: String,
    },
    MissingJdtbdHeader,
    MissingVectorColumnHeader,
    UnexpectedVectorColumns {
        actual: Vec<String>,
    },
    NoVectorRows,
    InvalidCsv {
        row: usize,
        reason: &'static str,
    },
    InvalidVector {
        row: usize,
        reason: &'static str,
    },
    InvalidStateVector(&'static str),
}

impl std::fmt::Display for HorizonsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(f, "invalid Horizons JSON: {message}"),
            Self::ProviderError(message) => write!(f, "Horizons API error: {message}"),
            Self::MissingResult => write!(f, "Horizons response has no result field"),
            Self::MissingSignature => write!(f, "Horizons JSON is missing its signature object"),
            Self::UnexpectedApiSource(source) => {
                write!(f, "unexpected Horizons API source: {source:?}")
            }
            Self::UnsupportedApiVersion(version) => {
                write!(f, "unsupported Horizons API signature version: {version:?}")
            }
            Self::InvalidRequest(reason) => {
                write!(f, "invalid Horizons vector request: {reason}")
            }
            Self::ProvenanceMismatch => write!(
                f,
                "ephemeris provenance hashes do not match the exact query and response bytes"
            ),
            Self::UnsupportedTimeScale => {
                write!(f, "only TDB vector epochs are supported by this parser")
            }
            Self::MissingStartMarker => write!(f, "Horizons response is missing $$SOE"),
            Self::MissingEndMarker => write!(f, "Horizons response is missing $$EOE"),
            Self::InvalidMarkerOrder => {
                write!(f, "Horizons ephemeris markers are duplicated or reversed")
            }
            Self::MissingMetadata(field) => {
                write!(f, "Horizons response is missing metadata: {field}")
            }
            Self::UnexpectedMetadata { field, expected, actual } => {
                write!(
                    f,
                    "unexpected Horizons metadata for {field}: expected {expected:?}, got {actual:?}"
                )
            }
            Self::MissingJdtbdHeader => {
                write!(f, "Horizons table does not identify its epoch column as JDTDB")
            }
            Self::MissingVectorColumnHeader => {
                write!(f, "Horizons vector table is missing its labelled CSV column header")
            }
            Self::UnexpectedVectorColumns { actual } => {
                write!(f, "unexpected Horizons vector CSV columns: {actual:?}")
            }
            Self::NoVectorRows => write!(f, "Horizons response contains no vector rows"),
            Self::InvalidCsv { row, reason } => {
                write!(f, "invalid CSV vector row {row}: {reason}")
            }
            Self::InvalidVector { row, reason } => {
                write!(f, "invalid state vector row {row}: {reason}")
            }
            Self::InvalidStateVector(reason) => {
                write!(f, "invalid parsed state vector: {reason}")
            }
        }
    }
}

impl std::error::Error for HorizonsParseError {}

/// Transport plan selected from the same deterministic request model.
/// The file upload input and identity are stable logical request bytes; callers
/// are responsible for the multipart HTTP exchange itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HorizonsRequestPlan {
    Get {
        url: String,
    },
    FilePost {
        endpoint: &'static str,
        format: &'static str,
        input_file: String,
    },
}

impl HorizonsRequestPlan {
    /// Deterministic identity to hash for provenance. The POST identity binds
    /// the endpoint, response format, and exact file input, not a random
    /// multipart boundary generated by an HTTP client.
    pub fn canonical_request_identity(&self) -> String {
        match self {
            Self::Get { url } => url.clone(),
            Self::FilePost { endpoint, format, input_file } => format!(
                "POST\n{endpoint}\nformat={format}\nfield=input\ninput-bytes:\n{input_file}"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HorizonsVectorRequest {
    /// Stable Sol Atlas catalogue ID, not a provider-specific numeric ID.
    pub target_id: String,
    /// Stable Sol Atlas ID for the requested coordinate origin (e.g. ssb).
    pub center_id: String,
    /// Provider target expression, e.g. 499 for Mars or
    /// DES=1999 AN10; for a small-body designation. URL delimiters are rejected;
    /// the builder quotes and percent-encodes this value itself.
    pub provider_target: String,
    /// Provider centre expression, e.g. @0 for the solar-system barycentre.
    pub provider_center: String,
    /// Expected body names as they appear in the Horizons response header.
    /// Keep these separate from stable IDs and provider-specific numeric codes.
    pub expected_target_name: String,
    pub expected_center_name: String,
    /// Discrete Julian-date epochs, interpreted in time_scale. TDB only for now.
    pub epochs_jd: Vec<f64>,
    pub time_scale: TimeScale,
    pub reference_system: ReferenceSystem,
    pub reference_plane: ReferencePlane,
    pub vector_correction: VectorCorrection,
}

impl HorizonsVectorRequest {
    /// Build a query from catalogue metadata so callers cannot accidentally use
    /// a display name as a provider code. Aggregate layers are not point targets.
    pub fn for_catalog_object(
        object: &CatalogObject,
        center_id: impl Into<String>,
        provider_center: impl Into<String>,
        expected_center_name: impl Into<String>,
        epochs_jd: Vec<f64>,
        time_scale: TimeScale,
        reference_system: ReferenceSystem,
        reference_plane: ReferencePlane,
        vector_correction: VectorCorrection,
    ) -> Result<Self, HorizonsParseError> {
        let provider_target = object.ephemeris_target.ok_or(
            HorizonsParseError::InvalidRequest(
                "aggregate catalogue layers cannot be queried as point targets",
            ),
        )?;

        let request = Self {
            target_id: object.id.to_owned(),
            center_id: center_id.into(),
            provider_target: provider_target.to_owned(),
            provider_center: provider_center.into(),
            expected_target_name: object.name.to_owned(),
            expected_center_name: expected_center_name.into(),
            epochs_jd,
            time_scale,
            reference_system,
            reference_plane,
            vector_correction,
        };
        validate_request(&request)?;
        Ok(request)
    }

    /// Produce a stable Horizons batch input file for the file-based API.
    /// This supports large TLIST requests that exceed common GET URL limits.
    pub fn canonical_file_input(&self) -> Result<String, HorizonsParseError> {
        validate_request(self)?;
        let epochs = self
            .epochs_jd
            .iter()
            .map(|epoch| quoted(&epoch.to_string()))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(format!(
            concat!(
                concat!("!", "$", "$", "SOF\n"),
                "COMMAND={}\n",
                "OBJ_DATA='YES'\n",
                "MAKE_EPHEM='YES'\n",
                "TABLE_TYPE='VECTORS'\n",
                "CENTER={}\n",
                "OUT_UNITS='KM-S'\n",
                "REF_SYSTEM={}\n",
                "REF_PLANE={}\n",
                "TIME_TYPE='TDB'\n",
                "TLIST={}\n",
                "TLIST_TYPE='JD'\n",
                "VECT_CORR={}\n",
                "VEC_LABELS='YES'\n",
                "VECT_TABLE='2'\n",
                "CSV_FORMAT='YES'\n",
                concat!("!", "$", "$", "EOF\n")
            ),
            quoted(&self.provider_target),
            quoted(&self.provider_center),
            quoted(reference_system_label(self.reference_system)),
            quoted(reference_plane_label(self.reference_plane)),
            epochs,
            quoted(vector_correction_label(self.vector_correction)),
        ))
    }

    /// Choose GET when its canonical URL fits the conservative limit; otherwise
    /// return the official file-upload contract instead of losing the request.
    pub fn request_plan(&self) -> Result<HorizonsRequestPlan, HorizonsParseError> {
        match self.canonical_url() {
            Ok(url) => Ok(HorizonsRequestPlan::Get { url }),
            Err(HorizonsParseError::InvalidRequest(
                "canonical GET URL exceeds 7500 bytes; use the file-based Horizons API",
            )) => Ok(HorizonsRequestPlan::FilePost {
                endpoint: "https://ssd.jpl.nasa.gov/api/horizons_file.api",
                format: "json",
                input_file: self.canonical_file_input()?,
            }),
            Err(error) => Err(error),
        }
    }

    /// Stable identity to hash for request provenance, independent of any
    /// HTTP client's transient multipart boundary.
    pub fn canonical_request_identity(&self) -> Result<String, HorizonsParseError> {
        Ok(self.request_plan()?.canonical_request_identity())
    }

    /// Build a deterministic, fully explicit Horizons API URL. Hash the returned
    /// UTF-8 bytes for canonical_request_sha256 when the plan is GET.
    /// Query parameters are emitted in a fixed order with fixed encoding.
    pub fn canonical_url(&self) -> Result<String, HorizonsParseError> {
        validate_request(self)?;
        // Horizons TLIST expects each time individually quoted, with
        // commas or spaces between entries. Quoting the whole comma-separated
        // list would not express a list of discrete epochs.
        let epochs = self
            .epochs_jd
            .iter()
            .map(|epoch| quoted(&epoch.to_string()))
            .collect::<Vec<_>>()
            .join(" ");

        let params = [
            ("COMMAND", quoted(&self.provider_target)),
            ("CENTER", quoted(&self.provider_center)),
            ("CSV_FORMAT", quoted("YES")),
            ("EPHEM_TYPE", quoted("VECTORS")),
            ("MAKE_EPHEM", quoted("YES")),
            ("OBJ_DATA", quoted("YES")),
            ("OUT_UNITS", quoted("KM-S")),
            ("REF_PLANE", quoted(reference_plane_label(self.reference_plane))),
            ("REF_SYSTEM", quoted(reference_system_label(self.reference_system))),
            ("TIME_TYPE", quoted("TDB")),
            ("TLIST", epochs),
            ("TLIST_TYPE", quoted("JD")),
            ("VEC_CORR", quoted(vector_correction_label(self.vector_correction))),
            ("VEC_LABELS", quoted("YES")),
            ("VEC_TABLE", quoted("2")),
            ("format", "json".to_owned()),
        ];

        let query = params
            .iter()
            .map(|(key, value)| format!("{key}={}", encode_query_component(value)))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("https://ssd.jpl.nasa.gov/api/horizons.api?{query}");
        if url.len() > MAX_CANONICAL_GET_URL_BYTES {
            return Err(HorizonsParseError::InvalidRequest(
                "canonical GET URL exceeds 7500 bytes; use the file-based Horizons API",
            ));
        }
        Ok(url)
    }
}

fn quoted(value: &str) -> String {
    format!("'{value}'")
}

fn validate_request(request: &HorizonsVectorRequest) -> Result<(), HorizonsParseError> {
    if request.time_scale != TimeScale::Tdb {
        return Err(HorizonsParseError::UnsupportedTimeScale);
    }
    if request.target_id.trim().is_empty() || request.center_id.trim().is_empty() {
        return Err(HorizonsParseError::InvalidRequest(
            "stable target and center IDs are required",
        ));
    }
    if request.expected_target_name.trim().is_empty()
        || request.expected_center_name.trim().is_empty()
    {
        return Err(HorizonsParseError::InvalidRequest(
            "expected target and center names are required",
        ));
    }
    if !safe_horizons_token(&request.provider_target)
        || !safe_horizons_token(&request.provider_center)
    {
        return Err(HorizonsParseError::InvalidRequest(concat!(
            "provider target/center may contain only alphanumeric characters, ",
            "spaces, - _ . @ ; ( ) =",
        )));
    }
    if request.epochs_jd.is_empty() || request.epochs_jd.len() > 10_000 {
        return Err(HorizonsParseError::InvalidRequest(
            "expected between 1 and 10000 requested epochs",
        ));
    }
    if request.epochs_jd.iter().any(|epoch| !epoch.is_finite() || *epoch <= 0.0) {
        return Err(HorizonsParseError::InvalidRequest(
            "all Julian-date epochs must be finite and positive",
        ));
    }
    if request.epochs_jd.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(HorizonsParseError::InvalidRequest(
            "TLIST epochs must be strictly increasing and unique",
        ));
    }
    Ok(())
}

fn safe_horizons_token(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b' ' | b'-' | b'_' | b'.' | b'@' | b';' | b'(' | b')' | b'=')
        })
}

fn encode_query_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0F) as usize] as char);
        }
    }
    encoded
}

#[derive(Debug, Deserialize)]
struct HorizonsEnvelope {
    signature: Option<HorizonsSignature>,
    result: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HorizonsSignature {
    source: Option<String>,
    version: Option<String>,
}

/// Parse a Horizons JSON response containing vector table 2 (x, y, z, vx, vy,
/// vz), CSV_FORMAT=YES, and OUT_UNITS=KM-S.
///
/// Build a plan with request_plan(), then hash its canonical_request_identity()
/// together with the unmodified response body using EphemerisProvenance::from_bytes().
/// This parser verifies both digests, checks the visible response header, and
/// validates every returned sample. It does not independently prove network
/// transport or provider authenticity.
pub fn parse_horizons_vectors_json(
    payload: &str,
    request: &HorizonsVectorRequest,
    provenance: &EphemerisProvenance,
) -> Result<Vec<StateVector>, HorizonsParseError> {
    validate_request(request)?;
    let request_plan = request.request_plan()?;

    let envelope: HorizonsEnvelope = serde_json::from_str(payload)
        .map_err(|error| HorizonsParseError::InvalidJson(error.to_string()))?;

    if let Some(error) = envelope.error.filter(|error| !error.trim().is_empty()) {
        return Err(HorizonsParseError::ProviderError(error));
    }

    // The official API documentation says to check the payload signature version
    // because a version change provides no guarantee that output format is stable.
    let signature = envelope
        .signature
        .ok_or(HorizonsParseError::MissingSignature)?;
    let source = signature.source.unwrap_or_default();
    if source != "NASA/JPL Horizons API" {
        return Err(HorizonsParseError::UnexpectedApiSource(source));
    }
    let version = signature.version.unwrap_or_default();
    if !signature_version_supported(&request_plan, &version) {
        return Err(HorizonsParseError::UnsupportedApiVersion(version));
    }

    let result = envelope.result.ok_or(HorizonsParseError::MissingResult)?;

    let lines: Vec<&str> = result.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (line.trim() == "$$SOE").then_some(index))
        .collect();
    let ends: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (line.trim() == "$$EOE").then_some(index))
        .collect();

    if starts.is_empty() {
        return Err(HorizonsParseError::MissingStartMarker);
    }
    if ends.is_empty() {
        return Err(HorizonsParseError::MissingEndMarker);
    }
    if starts.len() != 1 || ends.len() != 1 || starts[0] >= ends[0] {
        return Err(HorizonsParseError::InvalidMarkerOrder);
    }

    validate_header(&result, request)?;
    validate_vector_column_header(&lines[..starts[0]])?;

    let mut vectors = Vec::new();
    for (row_index, raw_line) in lines[starts[0] + 1..ends[0]].iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let fields = split_csv_record(line).map_err(|reason| {
            HorizonsParseError::InvalidCsv {
                row: row_index + 1,
                reason,
            }
        })?;
        if fields.len() != 8 {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "expected exactly eight CSV fields (JDTDB, calendar date, and six state components)",
            });
        }

        // The epoch must be the first field and the six state components must
        // be the six trailing fields. A numeric count alone is not enough: it
        // could silently accept a malformed/reordered CSV row.
        let numeric_count = fields
            .iter()
            .filter(|field| field.trim().parse::<f64>().is_ok())
            .count();
        if numeric_count != 7 {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "expected exactly seven numeric fields (JDTDB, x/y/z, vx/vy/vz)",
            });
        }
        if vectors.len() >= request.epochs_jd.len() {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "response contains more rows than requested TLIST epochs",
            });
        }

        let epoch_jd = fields[0].trim().parse::<f64>().map_err(|_| {
            HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "first CSV field must be JDTDB",
            }
        })?;
        let trailing = &fields[fields.len() - 6..];
        let mut components = [0.0_f64; 6];
        for (index, field) in trailing.iter().enumerate() {
            components[index] = field.trim().parse::<f64>().map_err(|_| {
                HorizonsParseError::InvalidVector {
                    row: row_index + 1,
                    reason: "last six CSV fields must be x/y/z/vx/vy/vz",
                }
            })?;
        }
        if !epoch_jd.is_finite() || components.iter().any(|value| !value.is_finite()) {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "epoch and state components must be finite",
            });
        }
        if (epoch_jd - request.epochs_jd[vectors.len()]).abs() > 1.0e-8 {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "returned JDTDB epoch does not match the corresponding requested epoch",
            });
        }

        let vector = StateVector {
            target_id: request.target_id.clone(),
            center_id: request.center_id.clone(),
            epoch_jd,
            time_scale: request.time_scale,
            reference_system: request.reference_system,
            reference_plane: request.reference_plane,
            vector_correction: request.vector_correction,
            position_km: [components[0], components[1], components[2]],
            velocity_km_s: [components[3], components[4], components[5]],
            provenance: provenance.clone(),
        };
        vector
            .validate()
            .map_err(HorizonsParseError::InvalidStateVector)?;
        vectors.push(vector);
    }

    if vectors.is_empty() {
        return Err(HorizonsParseError::NoVectorRows);
    }
    if vectors.len() != request.epochs_jd.len() {
        return Err(HorizonsParseError::InvalidVector {
            row: vectors.len() + 1,
            reason: "response sample count does not match requested TLIST epochs",
        });
    }
    let canonical_request_identity = request_plan.canonical_request_identity();
    if !provenance.verifies_bytes(&canonical_request_identity, payload.as_bytes()) {
        return Err(HorizonsParseError::ProvenanceMismatch);
    }
    Ok(vectors)
}

fn validate_header(
    result: &str,
    request: &HorizonsVectorRequest,
) -> Result<(), HorizonsParseError> {
    if request.target_id.trim().is_empty()
        || request.center_id.trim().is_empty()
        || request.expected_target_name.trim().is_empty()
        || request.expected_center_name.trim().is_empty()
    {
        return Err(HorizonsParseError::InvalidStateVector(
            "target, center, and expected body names are required",
        ));
    }

    let header = result
        .split("$$SOE")
        .next()
        .ok_or(HorizonsParseError::MissingStartMarker)?;

    let target = metadata_value(header, "Target body name")
        .ok_or(HorizonsParseError::MissingMetadata("Target body name"))?;
    if !header_body_name_matches(target, &request.expected_target_name) {
        return Err(HorizonsParseError::UnexpectedMetadata {
            field: "Target body name",
            expected: request.expected_target_name.clone(),
            actual: target.to_owned(),
        });
    }

    let center = metadata_value(header, "Center body name")
        .ok_or(HorizonsParseError::MissingMetadata("Center body name"))?;
    if !header_body_name_matches(center, &request.expected_center_name) {
        return Err(HorizonsParseError::UnexpectedMetadata {
            field: "Center body name",
            expected: request.expected_center_name.clone(),
            actual: center.to_owned(),
        });
    }

    expect_metadata(header, "Reference frame", reference_system_label(request.reference_system))?;
    expect_metadata(header, "Reference plane", reference_plane_label(request.reference_plane))?;
    expect_metadata(
        header,
        "Aberration corrections",
        vector_correction_label(request.vector_correction),
    )?;
    expect_metadata(header, "Output units", "KM-S")?;

    if !header.lines().any(|line| line.to_ascii_uppercase().contains("JDTDB")) {
        return Err(HorizonsParseError::MissingJdtbdHeader);
    }
    Ok(())
}

fn validate_vector_column_header(header_lines: &[&str]) -> Result<(), HorizonsParseError> {
    // VEC_LABELS=YES plus CSV_FORMAT=YES should expose the epoch and the
    // six state-vector columns before $SOE. Do not guess axis order from data.
    let columns_line = header_lines.iter().rev().find(|line| {
        line.to_ascii_uppercase().contains("JDTDB") && line.contains(',')
    });
    let Some(columns_line) = columns_line else {
        return Err(HorizonsParseError::MissingVectorColumnHeader);
    };
    let columns = split_csv_record(columns_line).map_err(|_| {
        HorizonsParseError::MissingVectorColumnHeader
    })?;
    let normalized: Vec<String> = columns
        .iter()
        .map(|column| column.trim().to_ascii_uppercase())
        .collect();
    let component_labels = ["X", "Y", "Z", "VX", "VY", "VZ"];
    let valid = normalized.len() == 8
        && normalized[0] == "JDTDB"
        && normalized[1].starts_with("CALENDAR DATE")
        && component_labels
            .iter()
            .enumerate()
            .all(|(index, expected)| normalized[index + 2].as_str() == *expected);
    if !valid {
        return Err(HorizonsParseError::UnexpectedVectorColumns { actual: columns });
    }
    Ok(())
}

fn signature_version_supported(plan: &HorizonsRequestPlan, version: &str) -> bool {
    match plan {
        HorizonsRequestPlan::Get { .. } => {
            SUPPORTED_GET_API_SIGNATURE_VERSIONS.contains(&version)
        }
        HorizonsRequestPlan::FilePost { .. } => {
            SUPPORTED_FILE_API_SIGNATURE_VERSIONS.contains(&version)
        }
    }
}

fn header_body_name_matches(actual: &str, expected: &str) -> bool {
    // Horizons headers can append a numeric SPK ID, parenthesized ID, or source
    // detail. Some numbered asteroids are rendered as "1 Ceres (1)", while
    // catalogue display names omit the designation prefix. Match that prefix
    // only when it is a numeric token; do not accept arbitrary substring hits.
    let before_source = actual.split('{').next().unwrap_or(actual).trim();
    let body_name = before_source
        .split('(')
        .next()
        .unwrap_or(before_source)
        .trim();
    if body_name.eq_ignore_ascii_case(expected.trim()) {
        return true;
    }

    let mut parts = body_name.splitn(2, char::is_whitespace);
    let first = parts.next().unwrap_or_default();
    let remainder = parts.next().unwrap_or_default().trim();
    !first.is_empty()
        && first.bytes().all(|byte| byte.is_ascii_digit())
        && remainder.eq_ignore_ascii_case(expected.trim())
}

fn metadata_value<'a>(header: &'a str, label: &str) -> Option<&'a str> {
    header.lines().find_map(|line| {
        let (candidate, value) = line.split_once(':')?;
        (candidate.trim().eq_ignore_ascii_case(label)).then_some(value.trim())
    })
}

fn expect_metadata(
    header: &str,
    label: &'static str,
    expected: &'static str,
) -> Result<(), HorizonsParseError> {
    let actual = metadata_value(header, label)
        .ok_or(HorizonsParseError::MissingMetadata(label))?;
    let normalized = actual.trim();
    let matches = if label == "Reference frame" && expected == "B1950" {
        normalized.eq_ignore_ascii_case("B1950")
            || normalized.eq_ignore_ascii_case("FK4/B1950")
    } else {
        normalized.eq_ignore_ascii_case(expected)
    };
    if !matches {
        return Err(HorizonsParseError::UnexpectedMetadata {
            field: label,
            expected: expected.to_owned(),
            actual: actual.to_owned(),
        });
    }
    Ok(())
}

fn reference_system_label(system: ReferenceSystem) -> &'static str {
    match system {
        ReferenceSystem::Icrf => "ICRF",
        ReferenceSystem::B1950 => "B1950",
    }
}

fn reference_plane_label(plane: ReferencePlane) -> &'static str {
    match plane {
        ReferencePlane::Ecliptic => "ECLIPTIC",
        ReferencePlane::Frame => "FRAME",
        ReferencePlane::BodyEquator => "BODY EQUATOR",
    }
}

fn vector_correction_label(correction: VectorCorrection) -> &'static str {
    match correction {
        VectorCorrection::Geometric => "NONE",
        VectorCorrection::LightTime => "LT",
        VectorCorrection::LightTimeAndStellarAberration => "LT+S",
    }
}

/// Small CSV reader for Horizons rows, including quoted fields and escaped
/// double quotes. It does not split on commas inside quoted fields.
fn split_csv_record(record: &str) -> Result<Vec<String>, &'static str> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut closed_quote = false;
    let mut chars = record.chars().peekable();

    while let Some(ch) = chars.next() {
        if quoted {
            match ch {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => {
                    quoted = false;
                    closed_quote = true;
                }
                _ => field.push(ch),
            }
            continue;
        }

        if closed_quote {
            match ch {
                ',' => {
                    fields.push(std::mem::take(&mut field));
                    closed_quote = false;
                }
                ' ' | '\t' => {}
                _ => return Err("unexpected character after closing quote"),
            }
            continue;
        }

        match ch {
            ',' => fields.push(std::mem::take(&mut field)),
            '"' if field.is_empty() => quoted = true,
            '"' => return Err("quote inside an unquoted field"),
            _ => field.push(ch),
        }
    }

    if quoted {
        return Err("unterminated quoted field");
    }
    fields.push(field);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../tests/fixtures/horizons/mars_ssb_tdb_frame_km_s.json");

    fn request() -> HorizonsVectorRequest {
        HorizonsVectorRequest::for_catalog_object(
            catalog_object("mars").unwrap(),
            "ssb",
            "@0",
            "Solar System Barycenter",
            vec![2_461_323.5],
            TimeScale::Tdb,
            ReferenceSystem::Icrf,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
        )
        .unwrap()
    }

    fn provenance_for(payload: &str) -> EphemerisProvenance {
        EphemerisProvenance::from_bytes(
            "JPL Horizons (synthetic parser fixture)",
            &request().canonical_request_identity().unwrap(),
            payload.as_bytes(),
            "2026-10-10T16:00:00Z",
        )
    }

    fn provenance() -> EphemerisProvenance {
        provenance_for(FIXTURE)
    }

    #[test]
    fn catalogue_request_uses_provider_id_and_rejects_aggregate_layers() {
        let ceres = HorizonsVectorRequest::for_catalog_object(
            catalog_object("ceres").unwrap(),
            "ssb",
            "@0",
            "Solar System Barycenter",
            vec![2_461_323.5],
            TimeScale::Tdb,
            ReferenceSystem::Icrf,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
        )
        .unwrap();
        assert_eq!(ceres.target_id, "ceres");
        assert_eq!(ceres.expected_target_name, "Ceres");
        assert_eq!(ceres.provider_target, "1;");

        assert!(matches!(
            HorizonsVectorRequest::for_catalog_object(
                catalog_object("comets").unwrap(),
                "ssb",
                "@0",
                "Solar System Barycenter",
                vec![2_461_323.5],
                TimeScale::Tdb,
                ReferenceSystem::Icrf,
                ReferencePlane::Frame,
                VectorCorrection::Geometric,
            ),
            Err(HorizonsParseError::InvalidRequest(
                "aggregate catalogue layers cannot be queried as point targets"
            ))
        ));
    }

    #[test]
    fn normal_request_selects_get_transport_and_same_identity() {
        let req = request();
        let plan = req.request_plan().unwrap();
        assert_eq!(
            plan,
            HorizonsRequestPlan::Get {
                url: req.canonical_url().unwrap()
            }
        );
        assert_eq!(
            req.canonical_request_identity().unwrap(),
            req.canonical_url().unwrap()
        );
    }

    #[test]
    fn oversized_tlist_selects_file_post_with_explicit_vector_contract() {
        let mut req = request();
        req.epochs_jd = (0..1_000)
            .map(|index| 2_461_323.5 + f64::from(index))
            .collect();
        let plan = req.request_plan().unwrap();
        let HorizonsRequestPlan::FilePost { endpoint, format, input_file } = plan else {
            panic!("expected file-based POST for a large epoch list");
        };

        assert_eq!(endpoint, "https://ssd.jpl.nasa.gov/api/horizons_file.api");
        assert_eq!(format, "json");
        assert!(input_file.starts_with(concat!("!", "$", "$", "SOF\n")));
        assert!(input_file.contains("COMMAND='499'\n"));
        assert!(input_file.contains("TABLE_TYPE='VECTORS'\n"));
        assert!(input_file.contains("CENTER='@0'\n"));
        assert!(input_file.contains("TLIST='2461323.5' '2461324.5'"));
        assert!(input_file.contains("REF_SYSTEM='ICRF'\n"));
        assert!(input_file.contains("REF_PLANE='FRAME'\n"));
        assert!(input_file.contains("VECT_CORR='NONE'\n"));
        assert!(input_file.contains("VECT_TABLE='2'\n"));
        assert!(input_file.ends_with(concat!("!", "$", "$", "EOF\n")));
        assert!(req.canonical_request_identity().unwrap().starts_with(concat!(
            "POST\nhttps://ssd.jpl.nasa.gov/api/horizons_file.api\n",
            "format=json\nfield=input\ninput-bytes:\n"
        )));
    }

    #[test]
    fn canonical_url_pins_every_ephemeris_setting_and_encodes_values() {
        let url = request().canonical_url().unwrap();
        assert!(url.starts_with("https://ssd.jpl.nasa.gov/api/horizons.api?"));
        assert!(url.contains("COMMAND=%27499%27"));
        assert!(url.contains("CENTER=%27%400%27"));
        assert!(url.contains("TLIST=%272461323.5%27"));
        assert!(url.contains("TIME_TYPE=%27TDB%27"));
        assert!(url.contains("REF_SYSTEM=%27ICRF%27"));
        assert!(url.contains("REF_PLANE=%27FRAME%27"));
        assert!(url.contains("VEC_CORR=%27NONE%27"));
        assert!(url.contains("OUT_UNITS=%27KM-S%27"));
        assert!(url.contains("VEC_TABLE=%272%27"));
        assert!(url.ends_with("format=json"));
    }

    #[test]
    fn canonical_url_quotes_every_tlist_epoch_individually() {
        let mut request = request();
        request.epochs_jd = vec![2_461_323.5, 2_461_324.5];
        let url = request.canonical_url().unwrap();
        assert!(url.contains("TLIST=%272461323.5%27%20%272461324.5%27"));
    }

    #[test]
    fn canonical_url_fails_closed_when_get_url_is_too_large() {
        let mut request = request();
        request.epochs_jd = (0..1_000)
            .map(|index| 2_461_323.5 + f64::from(index))
            .collect();
        assert!(matches!(
            request.canonical_url(),
            Err(HorizonsParseError::InvalidRequest(
                "canonical GET URL exceeds 7500 bytes; use the file-based Horizons API"
            ))
        ));
    }

    #[test]
    fn body_name_comparison_rejects_substring_collisions() {
        assert!(header_body_name_matches("Mars (499) {source: test}", "Mars"));
        assert!(header_body_name_matches("Solar System Barycenter (0)", "Solar System Barycenter"));
        assert!(header_body_name_matches("1 Ceres (1)", "Ceres"));
        assert!(header_body_name_matches("136108 Haumea (136108)", "Haumea"));
        assert!(!header_body_name_matches("Mars Barycenter (4)", "Mars"));
        assert!(!header_body_name_matches("MarsSomething (499)", "Mars"));
    }

    #[test]
    fn canonical_url_rejects_duplicate_or_descending_epochs() {
        let mut request = request();
        request.epochs_jd = vec![2_461_324.5, 2_461_323.5];
        assert_eq!(
            request.canonical_url(),
            Err(HorizonsParseError::InvalidRequest(
                "TLIST epochs must be strictly increasing and unique"
            ))
        );

        request.epochs_jd = vec![2_461_323.5, 2_461_323.5];
        assert_eq!(
            request.canonical_url(),
            Err(HorizonsParseError::InvalidRequest(
                "TLIST epochs must be strictly increasing and unique"
            ))
        );
    }

    #[test]
    fn canonical_url_supports_encoded_small_body_designations() {
        let mut request = request();
        request.provider_target = "DES=1999 AN10;".into();
        let url = request.canonical_url().unwrap();
        assert!(url.contains("COMMAND=%27DES%3D1999%20AN10%3B%27"));
        assert!(url.contains("TLIST=%272461323.5%27"));
    }

    #[test]
    fn canonical_url_rejects_query_injection_and_invalid_epochs() {
        let mut bad = request();
        bad.provider_target = "499&OBJ_DATA=NO".into();
        assert!(matches!(
            bad.canonical_url(),
            Err(HorizonsParseError::InvalidRequest(_))
        ));

        let mut bad = request();
        bad.epochs_jd = vec![f64::NAN];
        assert!(matches!(
            bad.canonical_url(),
            Err(HorizonsParseError::InvalidRequest(_))
        ));
    }

    #[test]
    fn rejects_response_epoch_that_does_not_match_request() {
        let payload = FIXTURE.replace("2461323.500000000", "2461324.500000000");
        assert!(matches!(
            parse_horizons_vectors_json(&payload, &request(), &provenance()),
            Err(HorizonsParseError::InvalidVector {
                reason: "returned JDTDB epoch does not match the corresponding requested epoch",
                ..
            })
        ));
    }

    #[test]
    fn parses_one_tdb_km_s_state_from_json_envelope() {
        let states = parse_horizons_vectors_json(FIXTURE, &request(), &provenance()).unwrap();
        assert_eq!(states.len(), 1);
        let state = &states[0];
        assert_eq!(state.target_id, "mars");
        assert_eq!(state.center_id, "ssb");
        assert_eq!(state.time_scale, TimeScale::Tdb);
        assert_eq!(state.reference_system, ReferenceSystem::Icrf);
        assert_eq!(state.reference_plane, ReferencePlane::Frame);
        assert_eq!(state.vector_correction, VectorCorrection::Geometric);
        assert!((state.epoch_jd - 2_461_323.5).abs() < 1e-9);
        assert!((state.position_km[0] - 1.782345678901234e8).abs() < 1e-3);
        assert!((state.velocity_km_s[2] - 9.876543210987654).abs() < 1e-9);
        assert!(state.validate().is_ok());
    }

    #[test]
    fn rejects_missing_or_reordered_vector_column_labels() {
        let missing = FIXTURE.replace(
            "JDTDB, Calendar Date (TDB), X, Y, Z, VX, VY, VZ",
            "Calendar Date (TDB), JD, X, Y, Z, VX, VY, VZ",
        );
        assert_eq!(
            parse_horizons_vectors_json(&missing, &request(), &provenance()),
            Err(HorizonsParseError::MissingJdtbdHeader)
        );

        let reordered = FIXTURE.replace(
            "JDTDB, Calendar Date (TDB), X, Y, Z, VX, VY, VZ",
            "JDTDB, Calendar Date (TDB), Y, X, Z, VX, VY, VZ",
        );
        assert!(matches!(
            parse_horizons_vectors_json(&reordered, &request(), &provenance()),
            Err(HorizonsParseError::UnexpectedVectorColumns { .. })
        ));
    }

    #[test]
    fn rejects_wrong_target_or_center_in_response_header() {
        let payload = FIXTURE.replace("Target body name: Mars", "Target body name: Venus");
        assert!(matches!(
            parse_horizons_vectors_json(&payload, &request(), &provenance()),
            Err(HorizonsParseError::UnexpectedMetadata { field: "Target body name", .. })
        ));

        let payload = FIXTURE.replace(
            "Center body name: Solar System Barycenter",
            "Center body name: Earth",
        );
        assert!(matches!(
            parse_horizons_vectors_json(&payload, &request(), &provenance()),
            Err(HorizonsParseError::UnexpectedMetadata { field: "Center body name", .. })
        ));
    }

    #[test]
    fn rejects_hashes_that_do_not_match_the_exact_query_or_payload() {
        let mut wrong_response = provenance();
        wrong_response.raw_response_sha256 = "0".repeat(64);
        assert_eq!(
            parse_horizons_vectors_json(FIXTURE, &request(), &wrong_response),
            Err(HorizonsParseError::ProvenanceMismatch)
        );

        let mut wrong_query = provenance();
        wrong_query.canonical_request_sha256 = "0".repeat(64);
        assert_eq!(
            parse_horizons_vectors_json(FIXTURE, &request(), &wrong_query),
            Err(HorizonsParseError::ProvenanceMismatch)
        );
    }

    #[test]
    fn signature_version_allowlist_is_transport_specific() {
        assert!(signature_version_supported(&HorizonsRequestPlan::Get {
            url: "https://example.invalid".into(),
        }, "1.0"));
        assert!(signature_version_supported(&HorizonsRequestPlan::Get {
            url: "https://example.invalid".into(),
        }, "1.3"));
        assert!(!signature_version_supported(&HorizonsRequestPlan::Get {
            url: "https://example.invalid".into(),
        }, "0.2"));
        assert!(signature_version_supported(&HorizonsRequestPlan::FilePost {
            endpoint: "https://example.invalid",
            format: "json",
            input_file: String::new(),
        }, "0.2"));
        assert!(signature_version_supported(&HorizonsRequestPlan::FilePost {
            endpoint: "https://example.invalid",
            format: "json",
            input_file: String::new(),
        }, "1.0"));
        assert!(!signature_version_supported(&HorizonsRequestPlan::FilePost {
            endpoint: "https://example.invalid",
            format: "json",
            input_file: String::new(),
        }, "1.3"));
    }

    #[test]
    fn rejects_missing_or_changed_api_signature() {
        let no_signature = r#"{"result":"$SOE\n$EOE"}"#;
        assert_eq!(
            parse_horizons_vectors_json(no_signature, &request(), &provenance()),
            Err(HorizonsParseError::MissingSignature)
        );

        let wrong_source = FIXTURE.replace(
            "NASA/JPL Horizons API",
            "Other provider",
        );
        assert!(matches!(
            parse_horizons_vectors_json(&wrong_source, &request(), &provenance()),
            Err(HorizonsParseError::UnexpectedApiSource(_))
        ));

        let documented_legacy_signature = FIXTURE.replace(
            "\"version\": \"1.3\"",
            "\"version\": \"1.0\"",
        );
        assert!(
            parse_horizons_vectors_json(
                &documented_legacy_signature,
                &request(),
                &provenance_for(&documented_legacy_signature)
            )
            .is_ok()
        );

        let wrong_version = FIXTURE.replace(
            "\"version\": \"1.3\"",
            "\"version\": \"99.0\"",
        );
        assert_eq!(
            parse_horizons_vectors_json(&wrong_version, &request(), &provenance()),
            Err(HorizonsParseError::UnsupportedApiVersion("99.0".into()))
        );
    }

    #[test]
    fn rejects_provider_errors_and_missing_markers() {
        let error = r#"{"error":"invalid target","result":""}"#;
        assert!(matches!(
            parse_horizons_vectors_json(error, &request(), &provenance()),
            Err(HorizonsParseError::ProviderError(_))
        ));

        let missing = concat!(
            r#"{"signature":{"source":"NASA/JPL Horizons API","version":"1.3"},"#,
            r#""result":"Target body name: Mars"}"#,
        );
        assert_eq!(
            parse_horizons_vectors_json(missing, &request(), &provenance()),
            Err(HorizonsParseError::MissingStartMarker)
        );
    }

    #[test]
    fn rejects_missing_calendar_date_column_in_vector_row() {
        let payload = FIXTURE.replace(
            "2461323.500000000, A.D. 2026-Oct-10 00:00:00.0000 TDB, ",
            "2461323.500000000, ",
        );
        assert!(matches!(
            parse_horizons_vectors_json(&payload, &request(), &provenance()),
            Err(HorizonsParseError::InvalidVector {
                reason: "expected exactly eight CSV fields (JDTDB, calendar date, and six state components)",
                ..
            })
        ));
    }

    #[test]
    fn rejects_extra_numeric_columns_and_non_finite_components() {
        let extra = FIXTURE.replace(
            "9.876543210987654E+00",
            "9.876543210987654E+00, 123.0",
        );
        assert!(matches!(
            parse_horizons_vectors_json(&extra, &request(), &provenance()),
            Err(HorizonsParseError::InvalidVector { .. })
        ));

        let non_finite = FIXTURE.replace("1.782345678901234E+08", "NaN");
        assert!(matches!(
            parse_horizons_vectors_json(&non_finite, &request(), &provenance()),
            Err(HorizonsParseError::InvalidVector { .. })
        ));
    }

    #[test]
    fn rejects_light_time_mode_mismatch_even_when_metadata_prefix_matches() {
        let payload = FIXTURE.replace(
            "Aberration corrections : NONE",
            "Aberration corrections : LT+S",
        );
        assert!(matches!(
            parse_horizons_vectors_json(&payload, &request(), &provenance()),
            Err(HorizonsParseError::UnexpectedMetadata {
                field: "Aberration corrections",
                ..
            })
        ));
    }

    #[test]
    fn rejects_non_numeric_epoch_in_first_column() {
        let payload = FIXTURE.replace("2461323.500000000", "not-a-julian-date");
        assert!(matches!(
            parse_horizons_vectors_json(&payload, &request(), &provenance()),
            Err(HorizonsParseError::InvalidVector { .. })
        ));
    }

    #[test]
    fn csv_split_rejects_malformed_quote_placement() {
        assert_eq!(
            split_csv_record(r#"1,"valid"suffix,2"#),
            Err("unexpected character after closing quote")
        );
        assert_eq!(
            split_csv_record(r#"1,bad"quote,2"#),
            Err("quote inside an unquoted field")
        );
        assert_eq!(
            split_csv_record(r#"1,"unterminated,2"#),
            Err("unterminated quoted field")
        );
    }

    #[test]
    fn refuses_non_tdb_epoch_interpretation() {
        let mut request = request();
        request.time_scale = TimeScale::Utc;
        assert_eq!(
            parse_horizons_vectors_json(FIXTURE, &request, &provenance()),
            Err(HorizonsParseError::UnsupportedTimeScale)
        );
    }

    #[test]
    fn csv_split_preserves_quoted_commas_and_escaped_quotes() {
        assert_eq!(
            split_csv_record(r#"1,"date, with comma","a ""quote""",2"#).unwrap(),
            vec!["1", "date, with comma", "a \"quote\"", "2"]
        );
    }
}
