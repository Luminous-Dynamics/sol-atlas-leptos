// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware industrial capacity records for Sol Atlas.
//!
//! This module defines a reusable data contract only. It intentionally does not
//! seed or imply real-world facility records; datasets must be sourced, dated,
//! and validated before they are presented as observed capacity.

use crate::types::DataKind;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Broad facility roles used for regional manufacturing and maritime planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndustrialFacilityKind {
    SteelMill,
    Foundry,
    Smelter,
    HeatTreatment,
    CncWorkshop,
    WeldingFabricator,
    NdtInspection,
    AdditiveManufacturing,
    Shipyard,
    DryDock,
    ShipRepair,
    Port,
    Recycling,
    Other,
}

/// Operational state of a facility, distinct from the confidence in its record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacilityOperatingStatus {
    Operating,
    TemporarilyUnavailable,
    Planned,
    Closed,
    Unknown,
}

/// A traceable source for a facility or capacity assertion.
///
/// For `Observed` evidence, the dataset snapshot date and artifact digest are
/// required. A digest identifies the referenced bytes; it does not prove the
/// source is accurate, the operator authorized the claim, or the facility has
/// a particular capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndustrialEvidence {
    pub kind: DataKind,
    /// Stable source/dataset identifier.
    pub source_id: String,
    /// Source locator or controlled evidence-store reference.
    pub source_uri: String,
    /// Snapshot date in YYYY-MM-DD format, when known.
    pub snapshot_date: Option<String>,
    /// SHA-256 digest of the exact source artifact, when available.
    pub artifact_sha256: Option<String>,
}

/// Explicit capacity dimensions. The enum makes units part of the schema,
/// avoiding comparisons between unlike quantities (for example, hours and tonnes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndustrialCapacityMetric {
    SteelTonnesPerYear,
    NonFerrousTonnesPerYear,
    FoundryTonnesPerYear,
    MachiningHoursPerYear,
    WeldingHoursPerYear,
    NdtInspectionHoursPerYear,
    HeatTreatmentHoursPerYear,
    VesselSlots,
    DryDockLengthMetres,
    CargoTonnesPerYear,
    RecyclingTonnesPerYear,
}

/// A dated capacity estimate with its own evidence source.
///
/// `value` represents nominal/declared capacity in the metric's units, not
/// necessarily spare or immediately available capacity. Utilization is optional;
/// missing utilization must not be interpreted as zero.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndustrialCapacityObservation {
    pub metric: IndustrialCapacityMetric,
    pub value: f64,
    /// Calendar year the capacity assertion applies to.
    pub year: u16,
    /// Fraction in [0, 1], only when supported by evidence.
    pub utilization: Option<f64>,
    pub evidence: IndustrialEvidence,
}

/// A map-ready industrial facility record with explicit provenance.
///
/// Coordinates are WGS84 decimal degrees. Capacity values are never aggregated
/// across different metrics; regional planning must request a compatible metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndustrialFacility {
    pub id: String,
    pub name: String,
    pub kind: IndustrialFacilityKind,
    pub status: FacilityOperatingStatus,
    pub lat: f64,
    pub lon: f64,
    /// ISO 3166-1 alpha-2 country code, uppercase.
    pub country_code: String,
    pub evidence: IndustrialEvidence,
    #[serde(default)]
    pub capacities: Vec<IndustrialCapacityObservation>,
}

/// Validate a facility and all attached capacity observations.
///
/// Returns every detected issue to help curators fix records in one pass.
/// This is structural validation, not verification of the external evidence or
/// a guarantee that nominal capacity is actually available.
pub fn validate_industrial_facility(facility: &IndustrialFacility) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();

    for (field, value, limit) in [
        ("id", facility.id.as_str(), 256usize),
        ("name", facility.name.as_str(), 512usize),
        ("country_code", facility.country_code.as_str(), 2usize),
    ] {
        if value.trim().is_empty() {
            issues.push(format!("{field} cannot be empty"));
        } else if value.len() > limit {
            issues.push(format!("{field} exceeds {limit} bytes"));
        }
    }
    if facility.country_code.len() != 2
        || !facility
            .country_code
            .bytes()
            .all(|b| b.is_ascii_uppercase())
    {
        issues.push("country_code must be two uppercase ASCII letters".into());
    }
    if !facility.lat.is_finite() || !(-90.0..=90.0).contains(&facility.lat) {
        issues.push("lat must be finite and within [-90, 90]".into());
    }
    if !facility.lon.is_finite() || !(-180.0..=180.0).contains(&facility.lon) {
        issues.push("lon must be finite and within [-180, 180]".into());
    }

    validate_evidence("facility evidence", &facility.evidence, &mut issues);

    for (index, capacity) in facility.capacities.iter().enumerate() {
        if !capacity.value.is_finite() || capacity.value < 0.0 {
            issues.push(format!(
                "capacities[{index}].value must be finite and non-negative"
            ));
        }
        if !(1900..=2200).contains(&capacity.year) {
            issues.push(format!("capacities[{index}].year is outside [1900, 2200]"));
        }
        if let Some(utilization) = capacity.utilization {
            if !utilization.is_finite() || !(0.0..=1.0).contains(&utilization) {
                issues.push(format!(
                    "capacities[{index}].utilization must be finite and within [0, 1]"
                ));
            }
        }
        validate_evidence(
            &format!("capacity evidence[{index}]"),
            &capacity.evidence,
            &mut issues,
        );
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

/// Validate a collection and reject duplicate stable IDs.
///
/// This function does not deduplicate silently: duplicate IDs may refer to
/// conflicting sources or revisions and need curator resolution.
pub fn validate_industrial_dataset(facilities: &[IndustrialFacility]) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();
    let mut ids = HashSet::with_capacity(facilities.len());

    for (index, facility) in facilities.iter().enumerate() {
        if !ids.insert(facility.id.as_str()) {
            issues.push(format!(
                "duplicate facility id '{}' at index {index}",
                facility.id
            ));
        }
        if let Err(facility_issues) = validate_industrial_facility(facility) {
            for issue in facility_issues {
                issues.push(format!("facility[{index}]: {issue}"));
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

fn validate_evidence(label: &str, evidence: &IndustrialEvidence, issues: &mut Vec<String>) {
    if evidence.source_id.trim().is_empty() || evidence.source_id.len() > 256 {
        issues.push(format!(
            "{label}: source_id must be non-empty and at most 256 bytes"
        ));
    }
    if evidence.source_uri.trim().is_empty() || evidence.source_uri.len() > 2048 {
        issues.push(format!(
            "{label}: source_uri must be non-empty and at most 2048 bytes"
        ));
    }
    if let Some(digest) = &evidence.artifact_sha256 {
        if !valid_sha256(digest) {
            issues.push(format!(
                "{label}: artifact_sha256 must be 64 hexadecimal characters"
            ));
        }
    }

    match evidence.kind {
        DataKind::Observed => {
            match &evidence.snapshot_date {
                Some(date) if valid_iso_date(date) => {}
                _ => issues.push(format!(
                    "{label}: observed evidence requires a valid YYYY-MM-DD snapshot_date"
                )),
            }
            if !evidence
                .artifact_sha256
                .as_deref()
                .is_some_and(valid_sha256)
            {
                issues.push(format!(
                    "{label}: observed evidence requires a SHA-256 artifact digest"
                ));
            }
        }
        DataKind::Curated | DataKind::Scenario => {
            if let Some(date) = &evidence.snapshot_date {
                if !valid_iso_date(date) {
                    issues.push(format!("{label}: snapshot_date must be YYYY-MM-DD"));
                }
            }
        }
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }

    let year = value[0..4].parse::<u16>().ok();
    let month = value[5..7].parse::<u8>().ok();
    let day = value[8..10].parse::<u8>().ok();
    let (Some(year), Some(month), Some(day)) = (year, month, day) else {
        return false;
    };
    if !(1..=9999).contains(&year) {
        return false;
    }

    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days_in_month).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(kind: DataKind) -> IndustrialEvidence {
        IndustrialEvidence {
            kind,
            source_id: "registry-snapshot-001".into(),
            source_uri: "https://example.invalid/industrial/record-001".into(),
            snapshot_date: (kind == DataKind::Observed).then(|| "2026-10-18".into()),
            artifact_sha256: (kind == DataKind::Observed).then(|| "a".repeat(64)),
        }
    }

    fn fixture() -> IndustrialFacility {
        IndustrialFacility {
            id: "facility-001".into(),
            name: "Illustrative facility record".into(),
            kind: IndustrialFacilityKind::CncWorkshop,
            status: FacilityOperatingStatus::Unknown,
            lat: -26.2041,
            lon: 28.0473,
            country_code: "ZA".into(),
            evidence: evidence(DataKind::Observed),
            capacities: vec![IndustrialCapacityObservation {
                metric: IndustrialCapacityMetric::MachiningHoursPerYear,
                value: 12000.0,
                year: 2026,
                utilization: None,
                evidence: evidence(DataKind::Observed),
            }],
        }
    }

    #[test]
    fn valid_facility_record_is_accepted() {
        assert!(validate_industrial_facility(&fixture()).is_ok());
    }

    #[test]
    fn observed_evidence_requires_date_and_digest() {
        let mut record = fixture();
        record.evidence.snapshot_date = None;
        record.evidence.artifact_sha256 = None;
        let issues = validate_industrial_facility(&record).unwrap_err();
        assert!(issues.iter().any(|issue| issue.contains("snapshot_date")));
        assert!(issues.iter().any(|issue| issue.contains("SHA-256")));
    }

    #[test]
    fn coordinates_and_country_codes_are_validated() {
        let mut record = fixture();
        record.lat = f64::NAN;
        record.lon = 181.0;
        record.country_code = "usa".into();
        let issues = validate_industrial_facility(&record).unwrap_err();
        assert!(issues.iter().any(|issue| issue.contains("lat")));
        assert!(issues.iter().any(|issue| issue.contains("lon")));
        assert!(issues.iter().any(|issue| issue.contains("country_code")));
    }

    #[test]
    fn utilization_and_capacity_reject_non_finite_values() {
        let mut record = fixture();
        record.capacities[0].value = f64::INFINITY;
        record.capacities[0].utilization = Some(f64::NAN);
        let issues = validate_industrial_facility(&record).unwrap_err();
        assert!(issues.iter().any(|issue| issue.contains("value")));
        assert!(issues.iter().any(|issue| issue.contains("utilization")));
    }

    #[test]
    fn observed_snapshot_date_checks_calendar_boundaries() {
        let mut record = fixture();
        record.evidence.snapshot_date = Some("2026-02-29".into());
        let issues = validate_industrial_facility(&record).unwrap_err();
        assert!(issues.iter().any(|issue| issue.contains("snapshot_date")));

        record.evidence.snapshot_date = Some("2024-02-29".into());
        assert!(validate_industrial_facility(&record).is_ok());
    }

    #[test]
    fn duplicate_facility_ids_are_not_silently_accepted() {
        let record = fixture();
        let issues = validate_industrial_dataset(&[record.clone(), record]).unwrap_err();
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("duplicate facility id"))
        );
    }

    #[test]
    fn scenario_records_do_not_require_observation_digest() {
        let mut record = fixture();
        record.evidence = evidence(DataKind::Scenario);
        record.evidence.snapshot_date = None;
        record.evidence.artifact_sha256 = None;
        assert!(validate_industrial_facility(&record).is_ok());
    }
}
