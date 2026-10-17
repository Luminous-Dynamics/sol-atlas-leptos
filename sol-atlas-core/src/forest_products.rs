// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance-aware forest and wood-processing context.
//!
//! This module is a typed data contract, not a forest inventory, harvest
//! authorization, environmental assessment, or proof that a source claim is
//! true. "ScreeningContext" means only that a sourced record may be displayed
//! and considered by a planning workflow; it never authorizes harvesting.

use serde::{Deserialize, Serialize};

const MAX_COVERAGE_BASIS_POINTS: u16 = 10_000;

/// The evidential status of a claim. These labels must not be silently upgraded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    Observed,
    Estimated,
    Curated,
    Scenario,
    Unknown,
}

/// Context category used by forest and industrial-wood planning screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFeatureKind {
    ForestArea,
    Plantation,
    ExclusionZone,
    Sawmill,
    DryingFacility,
    ResidualRecoveryFacility,
    ReclaimedWoodFacility,
}

/// A map's coverage or management label is not a permit or legal determination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagementStatus {
    /// The source provides no usable management-status evidence.
    Unknown,
    /// The feature is explicitly marked as restricted/excluded by its source.
    Excluded,
    /// Human or domain review is required before using this feature in planning.
    ReviewRequired,
    /// The source records no restriction for this feature; this is not permission.
    NoRestrictionRecorded,
}

/// Why a feature should or should not enter a first-pass screening workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningStatus {
    /// Record can be used as sourced context only, never as operating authority.
    ScreeningContextOnly,
    ReviewRequired,
    Excluded,
    StaleEvidence,
    InvalidRecord,
}

/// Temporal quality of the underlying source snapshot at a requested epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    FreshWithinDeclaredValidity,
    Expired,
    FutureDated,
    CaptureTimeUnknown,
    ValidityWindowUnknown,
}

/// Instrument calibration state, separate from the observation's evidence class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentCalibrationStatus {
    Current,
    Expired,
    NotApplicable,
    Unknown,
}

/// Geographic bounds in decimal degrees (WGS 84 by convention).
///
/// This v0.1 rectangle representation does not encode arbitrary polygons and
/// does not support rectangles crossing the antimeridian. It is a bounding box,
/// not a legal or ecological boundary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoBoundingBox {
    pub south_latitude_deg: f64,
    pub west_longitude_deg: f64,
    pub north_latitude_deg: f64,
    pub east_longitude_deg: f64,
}

impl GeoBoundingBox {
    pub fn validate(&self) -> Result<(), String> {
        let values = [
            self.south_latitude_deg,
            self.west_longitude_deg,
            self.north_latitude_deg,
            self.east_longitude_deg,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err("bounding-box coordinates must be finite".to_owned());
        }
        if !(-90.0..=90.0).contains(&self.south_latitude_deg)
            || !(-90.0..=90.0).contains(&self.north_latitude_deg)
        {
            return Err("latitude must be within [-90, 90] degrees".to_owned());
        }
        if !(-180.0..=180.0).contains(&self.west_longitude_deg)
            || !(-180.0..=180.0).contains(&self.east_longitude_deg)
        {
            return Err("longitude must be within [-180, 180] degrees".to_owned());
        }
        if self.south_latitude_deg > self.north_latitude_deg {
            return Err("south latitude must not exceed north latitude".to_owned());
        }
        if self.west_longitude_deg > self.east_longitude_deg {
            return Err(
                "antimeridian-crossing boxes are not represented by this v0.1 contract"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

/// Source, coverage, and temporal metadata for one feature record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialProvenance {
    /// Stable identifier allocated by the source or by an auditable ingestion step.
    pub source_id: String,
    /// URI of the original data or its source record; it is not fetched here.
    pub source_uri: String,
    pub publisher: String,
    /// Unix epoch seconds. None means the capture time is unknown.
    pub captured_at_epoch_seconds: Option<u64>,
    /// Optional declared validity limit. None means there is no usable expiry bound.
    pub valid_until_epoch_seconds: Option<u64>,
    pub coordinate_reference_system: String,
    /// Optional nominal pixel/grid resolution in metres. Not a positional accuracy claim.
    pub spatial_resolution_meters: Option<f64>,
    /// Estimated spatial coverage, in basis points (10,000 = 100%).
    pub coverage_basis_points: Option<u16>,
    pub evidence_class: EvidenceClass,
    /// Human-readable unknowns, limitations, sampling notes, or uncertainty.
    pub uncertainty_notes: Vec<String>,
}

impl SpatialProvenance {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.source_id.trim().is_empty() {
            errors.push("source_id must not be empty".to_owned());
        }
        if self.source_uri.trim().is_empty() {
            errors.push("source_uri must not be empty".to_owned());
        }
        if self.publisher.trim().is_empty() {
            errors.push("publisher must not be empty".to_owned());
        }
        if self.coordinate_reference_system.trim().is_empty() {
            errors.push("coordinate_reference_system must not be empty".to_owned());
        }
        if let Some(resolution) = self.spatial_resolution_meters {
            if !resolution.is_finite() || resolution <= 0.0 {
                errors.push("spatial_resolution_meters must be finite and positive".to_owned());
            }
        }
        if self.coverage_basis_points.is_some_and(|value| value > MAX_COVERAGE_BASIS_POINTS) {
            errors.push("coverage_basis_points must be within [0, 10000]".to_owned());
        }
        if let (Some(captured), Some(valid_until)) = (
            self.captured_at_epoch_seconds,
            self.valid_until_epoch_seconds,
        ) {
            if valid_until < captured {
                errors.push("valid_until must not precede captured_at".to_owned());
            }
        }
        if matches!(self.evidence_class, EvidenceClass::Estimated | EvidenceClass::Scenario | EvidenceClass::Unknown)
            && self.uncertainty_notes.iter().all(|note| note.trim().is_empty())
        {
            errors.push(
                "estimated, scenario, and unknown evidence require an uncertainty note".to_owned(),
            );
        }
        if self.uncertainty_notes.iter().any(|note| note.trim().is_empty()) {
            errors.push("uncertainty notes must not contain blank entries".to_owned());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    pub fn freshness_at(&self, now_epoch_seconds: u64) -> Freshness {
        let Some(captured_at) = self.captured_at_epoch_seconds else {
            return Freshness::CaptureTimeUnknown;
        };
        if captured_at > now_epoch_seconds {
            return Freshness::FutureDated;
        }
        let Some(valid_until) = self.valid_until_epoch_seconds else {
            return Freshness::ValidityWindowUnknown;
        };
        if now_epoch_seconds > valid_until {
            return Freshness::Expired;
        }
        Freshness::FreshWithinDeclaredValidity
    }
}

/// One typed process/resource measurement attached to a map feature.
///
/// A value is not useful without its unit, evidence state, source reference and
/// measurement timestamp. This type does not assert that an instrument was
/// calibrated or that the reported value is accurate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceMeasurement {
    pub attribute: String,
    pub value: f64,
    pub unit: String,
    pub evidence_class: EvidenceClass,
    pub measured_at_epoch_seconds: Option<u64>,
    /// Absolute uncertainty in the same unit as value, when quantified.
    pub absolute_uncertainty: Option<f64>,
    pub instrument_id: Option<String>,
    pub calibration_status: InstrumentCalibrationStatus,
    pub source_reference: String,
    pub uncertainty_note: Option<String>,
}

impl ResourceMeasurement {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.attribute.trim().is_empty() {
            errors.push("measurement attribute must not be empty".to_owned());
        }
        if !self.value.is_finite() {
            errors.push("measurement value must be finite".to_owned());
        }
        if self.unit.trim().is_empty() {
            errors.push("measurement unit must not be empty".to_owned());
        }
        if self.source_reference.trim().is_empty() {
            errors.push("measurement source_reference must not be empty".to_owned());
        }
        if let Some(uncertainty) = self.absolute_uncertainty {
            if !uncertainty.is_finite() || uncertainty < 0.0 {
                errors.push(
                    "absolute_uncertainty must be finite and non-negative".to_owned(),
                );
            }
        }
        if self.instrument_id.as_ref().is_some_and(|id| id.trim().is_empty()) {
            errors.push("instrument_id must be omitted rather than blank".to_owned());
        }
        if self.calibration_status == InstrumentCalibrationStatus::Current
            && self.instrument_id.is_none()
        {
            errors.push("current calibration requires an instrument_id".to_owned());
        }
        if self.instrument_id.is_some()
            && matches!(
                self.calibration_status,
                InstrumentCalibrationStatus::Expired | InstrumentCalibrationStatus::Unknown
            )
            && self.uncertainty_note.as_deref().is_none_or(|note| note.trim().is_empty())
        {
            errors.push(
                "expired or unknown instrument calibration requires an uncertainty note"
                    .to_owned(),
            );
        }
        if matches!(self.evidence_class, EvidenceClass::Estimated | EvidenceClass::Scenario | EvidenceClass::Unknown)
            && self.uncertainty_note.as_deref().is_none_or(|note| note.trim().is_empty())
            && self.absolute_uncertainty.is_none()
        {
            errors.push(
                "estimated, scenario, and unknown measurements require a numeric uncertainty or an uncertainty note"
                    .to_owned(),
            );
        }
        if self.uncertainty_note.as_deref().is_some_and(|note| note.trim().is_empty()) {
            errors.push("uncertainty_note must be omitted rather than blank".to_owned());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// A bounded spatial record for forests, plantations, mills, kilns and recovery sites.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WoodResourceFeature {
    pub feature_id: String,
    pub display_name: String,
    pub kind: ResourceFeatureKind,
    pub extent: GeoBoundingBox,
    pub provenance: SpatialProvenance,
    pub management_status: ManagementStatus,
    pub measurements: Vec<ResourceMeasurement>,
}

impl WoodResourceFeature {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.feature_id.trim().is_empty() {
            errors.push("feature_id must not be empty".to_owned());
        }
        if self.display_name.trim().is_empty() {
            errors.push("display_name must not be empty".to_owned());
        }
        if let Err(error) = self.extent.validate() {
            errors.push(error);
        }
        if let Err(provenance_errors) = self.provenance.validate() {
            errors.extend(provenance_errors.into_iter().map(|error| format!("provenance: {error}")));
        }
        for (index, measurement) in self.measurements.iter().enumerate() {
            if let Err(measurement_errors) = measurement.validate() {
                errors.extend(
                    measurement_errors
                        .into_iter()
                        .map(|error| format!("measurements[{index}]: {error}")),
                );
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Conservative first-pass display/simulation gate, not a harvest decision.
    ///
    /// A return value of ScreeningContextOnly permits use as context in a
    /// simulation only. It never grants a permit, establishes sustainability,
    /// qualifies structural lumber, or authorizes equipment.
    pub fn screening_status_at(&self, now_epoch_seconds: u64) -> ScreeningStatus {
        if self.validate().is_err() {
            return ScreeningStatus::InvalidRecord;
        }
        if self.kind == ResourceFeatureKind::ExclusionZone
            || self.management_status == ManagementStatus::Excluded
        {
            return ScreeningStatus::Excluded;
        }
        match self.provenance.freshness_at(now_epoch_seconds) {
            Freshness::Expired | Freshness::FutureDated => ScreeningStatus::StaleEvidence,
            Freshness::CaptureTimeUnknown | Freshness::ValidityWindowUnknown => {
                ScreeningStatus::ReviewRequired
            }
            Freshness::FreshWithinDeclaredValidity => {
                let sufficient_evidence = matches!(
                    self.provenance.evidence_class,
                    EvidenceClass::Observed | EvidenceClass::Curated
                );
                let coverage_declared = self
                    .provenance
                    .coverage_basis_points
                    .is_some_and(|coverage| coverage > 0);
                if sufficient_evidence
                    && coverage_declared
                    && self.management_status == ManagementStatus::NoRestrictionRecorded
                {
                    ScreeningStatus::ScreeningContextOnly
                } else {
                    ScreeningStatus::ReviewRequired
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> SpatialProvenance {
        SpatialProvenance {
            source_id: "inventory-snapshot-001".to_owned(),
            source_uri: "https://example.invalid/dataset/001".to_owned(),
            publisher: "synthetic test publisher".to_owned(),
            captured_at_epoch_seconds: Some(100),
            valid_until_epoch_seconds: Some(200),
            coordinate_reference_system: "EPSG:4326".to_owned(),
            spatial_resolution_meters: Some(30.0),
            coverage_basis_points: Some(9_500),
            evidence_class: EvidenceClass::Observed,
            uncertainty_notes: vec![],
        }
    }

    fn feature() -> WoodResourceFeature {
        WoodResourceFeature {
            feature_id: "forest-001".to_owned(),
            display_name: "Synthetic forest patch".to_owned(),
            kind: ResourceFeatureKind::ForestArea,
            extent: GeoBoundingBox {
                south_latitude_deg: -25.0,
                west_longitude_deg: 25.0,
                north_latitude_deg: -24.0,
                east_longitude_deg: 26.0,
            },
            provenance: provenance(),
            management_status: ManagementStatus::NoRestrictionRecorded,
            measurements: vec![ResourceMeasurement {
                attribute: "standing_volume".to_owned(),
                value: 100.0,
                unit: "synthetic_cubic_meters".to_owned(),
                evidence_class: EvidenceClass::Observed,
                measured_at_epoch_seconds: Some(120),
                absolute_uncertainty: Some(5.0),
                instrument_id: None,
                calibration_status: InstrumentCalibrationStatus::NotApplicable,
                source_reference: "inventory-snapshot-001:plot-4".to_owned(),
                uncertainty_note: None,
            }],
        }
    }

    #[test]
    fn current_sourced_feature_is_context_only_not_harvest_authority() {
        let value = feature();
        assert!(value.validate().is_ok());
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::ScreeningContextOnly
        );
    }

    #[test]
    fn expired_or_future_dated_sources_do_not_enter_current_screening() {
        let mut value = feature();
        assert_eq!(
            value.screening_status_at(201),
            ScreeningStatus::StaleEvidence
        );
        value.provenance.captured_at_epoch_seconds = Some(250);
        value.provenance.valid_until_epoch_seconds = Some(300);
        assert_eq!(
            value.screening_status_at(200),
            ScreeningStatus::StaleEvidence
        );
    }

    #[test]
    fn unknown_capture_or_validity_requires_review() {
        let mut value = feature();
        value.provenance.captured_at_epoch_seconds = None;
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::ReviewRequired
        );
        value.provenance.captured_at_epoch_seconds = Some(100);
        value.provenance.valid_until_epoch_seconds = None;
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::ReviewRequired
        );
    }

    #[test]
    fn exclusions_override_economic_or_measurement_fields() {
        let mut value = feature();
        value.kind = ResourceFeatureKind::ExclusionZone;
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::Excluded
        );
        value.kind = ResourceFeatureKind::ForestArea;
        value.management_status = ManagementStatus::Excluded;
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::Excluded
        );
    }

    #[test]
    fn estimated_evidence_needs_an_uncertainty_note() {
        let mut value = feature();
        value.provenance.evidence_class = EvidenceClass::Estimated;
        value.provenance.uncertainty_notes.clear();
        let errors = value.validate().expect_err("estimated evidence requires uncertainty");
        assert!(errors.iter().any(|error| error.contains("uncertainty note")));
    }

    #[test]
    fn numeric_uncertainty_allows_estimated_measurements_without_free_text() {
        let mut measurement = feature().measurements.remove(0);
        measurement.evidence_class = EvidenceClass::Estimated;
        measurement.uncertainty_note = None;
        measurement.absolute_uncertainty = Some(12.5);
        assert!(measurement.validate().is_ok());
    }

    #[test]
    fn malformed_numeric_uncertainty_fails_closed() {
        let mut measurement = feature().measurements.remove(0);
        measurement.absolute_uncertainty = Some(f64::NAN);
        let errors = measurement.validate().expect_err("uncertainty must be finite");
        assert!(errors.iter().any(|error| error.contains("absolute_uncertainty")));
        measurement.absolute_uncertainty = Some(-1.0);
        assert!(measurement.validate().is_err());
    }

    #[test]
    fn current_calibration_requires_instrument_identity() {
        let mut measurement = feature().measurements.remove(0);
        measurement.calibration_status = InstrumentCalibrationStatus::Current;
        measurement.instrument_id = None;
        assert!(measurement.validate().expect_err("instrument identity required")
            .iter().any(|error| error.contains("current calibration requires")));
    }

    #[test]
    fn expired_instrument_calibration_requires_caveat() {
        let mut measurement = feature().measurements.remove(0);
        measurement.calibration_status = InstrumentCalibrationStatus::Expired;
        measurement.instrument_id = Some("scanner-001".to_owned());
        measurement.uncertainty_note = None;
        assert!(measurement.validate().expect_err("expired calibration must be disclosed")
            .iter().any(|error| error.contains("uncertainty note")));
    }

    #[test]
    fn invalid_spatial_bounds_fail_closed() {
        let mut value = feature();
        value.extent.north_latitude_deg = f64::NAN;
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::InvalidRecord
        );
    }

    #[test]
    fn antimeridian_crossing_bounds_are_rejected_in_v0_1() {
        let bounds = GeoBoundingBox {
            south_latitude_deg: 1.0,
            west_longitude_deg: 170.0,
            north_latitude_deg: 2.0,
            east_longitude_deg: -170.0,
        };
        assert!(bounds.validate().is_err());
    }

    #[test]
    fn missing_coverage_never_becomes_screening_context() {
        let mut value = feature();
        value.provenance.coverage_basis_points = None;
        assert_eq!(
            value.screening_status_at(150),
            ScreeningStatus::ReviewRequired
        );
    }

    #[test]
    fn serde_round_trip_preserves_evidence_classes() {
        let value = feature();
        let json = serde_json::to_string(&value).expect("feature serializes");
        let decoded: WoodResourceFeature =
            serde_json::from_str(&json).expect("feature deserializes");
        assert_eq!(decoded, value);
    }
}
