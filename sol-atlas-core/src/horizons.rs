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
    EphemerisProvenance, ReferencePlane, ReferenceSystem, StateVector, TimeScale,
    VectorCorrection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HorizonsParseError {
    InvalidJson(String),
    ProviderError(String),
    MissingResult,
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
            Self::UnsupportedTimeScale => write!(f, "only TDB vector epochs are supported by this parser"),
            Self::MissingStartMarker => write!(f, "Horizons response is missing $$SOE"),
            Self::MissingEndMarker => write!(f, "Horizons response is missing $$EOE"),
            Self::InvalidMarkerOrder => write!(f, "Horizons ephemeris markers are duplicated or reversed"),
            Self::MissingMetadata(field) => write!(f, "Horizons response is missing metadata: {field}"),
            Self::UnexpectedMetadata { field, expected, actual } => {
                write!(f, "unexpected Horizons metadata for {field}: expected {expected:?}, got {actual:?}")
            }
            Self::MissingJdtbdHeader => write!(f, "Horizons table does not identify its epoch column as JDTDB"),
            Self::NoVectorRows => write!(f, "Horizons response contains no vector rows"),
            Self::InvalidCsv { row, reason } => write!(f, "invalid CSV vector row {row}: {reason}"),
            Self::InvalidVector { row, reason } => write!(f, "invalid state vector row {row}: {reason}"),
            Self::InvalidStateVector(reason) => write!(f, "invalid parsed state vector: {reason}"),
        }
    }
}

impl std::error::Error for HorizonsParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HorizonsVectorRequest {
    /// Stable Sol Atlas catalogue ID, not a provider-specific numeric ID.
    pub target_id: String,
    /// Stable Sol Atlas ID for the requested coordinate origin (e.g. ssb).
    pub center_id: String,
    /// Expected body names as they appear in the Horizons response header.
    /// Keep these separate from stable IDs and provider-specific numeric codes.
    pub expected_target_name: String,
    pub expected_center_name: String,
    pub time_scale: TimeScale,
    pub reference_system: ReferenceSystem,
    pub reference_plane: ReferencePlane,
    pub vector_correction: VectorCorrection,
}

#[derive(Debug, Deserialize)]
struct HorizonsEnvelope {
    result: Option<String>,
    error: Option<String>,
}

/// Parse a Horizons JSON response containing vector table 2 (x, y, z, vx, vy,
/// vz), CSV_FORMAT=YES, and OUT_UNITS=KM-S.
///
/// Request parameters are explicit caller input and should be constructed from
/// the same canonical query whose SHA-256 is recorded in provenance. This
/// function checks the visible response header against that request, validates
/// every returned sample, and attaches supplied provenance. It does not compute
/// hashes or prove network transport authenticity itself.
pub fn parse_horizons_vectors_json(
    payload: &str,
    request: &HorizonsVectorRequest,
    provenance: &EphemerisProvenance,
) -> Result<Vec<StateVector>, HorizonsParseError> {
    if request.time_scale != TimeScale::Tdb {
        return Err(HorizonsParseError::UnsupportedTimeScale);
    }

    let envelope: HorizonsEnvelope = serde_json::from_str(payload)
        .map_err(|error| HorizonsParseError::InvalidJson(error.to_string()))?;

    if let Some(error) = envelope.error.filter(|error| !error.trim().is_empty()) {
        return Err(HorizonsParseError::ProviderError(error));
    }
    let result = envelope.result.ok_or(HorizonsParseError::MissingResult)?;
    validate_header(&result, request)?;

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
        if fields.len() < 7 {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "expected Julian date plus six state components",
            });
        }

        // The calendar-date field is text; the epoch and six trailing state
        // components must be the only seven fields that parse as plain numbers.
        // Exact-count checking fails closed if extra range or uncertainty columns
        // appear because the request was not actually vector table 2.
        let numeric: Vec<f64> = fields
            .iter()
            .filter_map(|field| field.trim().parse::<f64>().ok())
            .collect();
        if numeric.len() != 7 {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "expected exactly seven numeric fields (JDTDB, x/y/z, vx/vy/vz)",
            });
        }
        if numeric.iter().any(|value| !value.is_finite()) {
            return Err(HorizonsParseError::InvalidVector {
                row: row_index + 1,
                reason: "epoch and state components must be finite",
            });
        }

        let vector = StateVector {
            target_id: request.target_id.clone(),
            center_id: request.center_id.clone(),
            epoch_jd: numeric[0],
            time_scale: request.time_scale,
            reference_system: request.reference_system,
            reference_plane: request.reference_plane,
            vector_correction: request.vector_correction,
            position_km: [numeric[1], numeric[2], numeric[3]],
            velocity_km_s: [numeric[4], numeric[5], numeric[6]],
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
    if !target.to_ascii_lowercase().contains(&request.expected_target_name.to_ascii_lowercase()) {
        return Err(HorizonsParseError::UnexpectedMetadata {
            field: "Target body name",
            expected: request.expected_target_name.clone(),
            actual: target.to_owned(),
        });
    }

    let center = metadata_value(header, "Center body name")
        .ok_or(HorizonsParseError::MissingMetadata("Center body name"))?;
    if !center.to_ascii_lowercase().contains(&request.expected_center_name.to_ascii_lowercase()) {
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
    if !actual.to_ascii_uppercase().contains(expected) {
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
    let mut chars = record.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
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
        HorizonsVectorRequest {
            target_id: "mars".into(),
            center_id: "ssb".into(),
            expected_target_name: "Mars".into(),
            expected_center_name: "Solar System Barycenter".into(),
            time_scale: TimeScale::Tdb,
            reference_system: ReferenceSystem::Icrf,
            reference_plane: ReferencePlane::Frame,
            vector_correction: VectorCorrection::Geometric,
        }
    }

    fn provenance() -> EphemerisProvenance {
        EphemerisProvenance {
            provider: "JPL Horizons (synthetic parser fixture)".into(),
            canonical_query_sha256: "a".repeat(64),
            raw_response_sha256: "b".repeat(64),
            retrieved_at_utc: "2026-10-10T16:00:00Z".into(),
        }
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
    fn rejects_provider_errors_and_missing_markers() {
        let error = r#"{"error":"invalid target","result":""}"#;
        assert!(matches!(
            parse_horizons_vectors_json(error, &request(), &provenance()),
            Err(HorizonsParseError::ProviderError(_))
        ));

        let missing = r#"{"result":"Target body name: Mars"}"#;
        assert!(matches!(
            parse_horizons_vectors_json(missing, &request(), &provenance()),
            Err(HorizonsParseError::MissingMetadata(_))
                | Err(HorizonsParseError::MissingStartMarker)
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
