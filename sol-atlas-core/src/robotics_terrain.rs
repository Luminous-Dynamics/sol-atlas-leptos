// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned, provenance-bearing terrain and hazard evidence for robotics.
//!
//! This module is a spatial evidence contract only. It does not plan routes,
//! authorize motion, authenticate remote artifacts, or claim that a terrain
//! model is physically valid. Raw coordinates remain owned by the spatial
//! source; downstream consumers receive opaque references and digests.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const ROBOTICS_TERRAIN_SCHEMA_VERSION: u16 = 1;

/// The evidence lifecycle is not interchangeable with the physical surface type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainEvidenceState {
    Available,
    Unknown,
    Stale,
    Conflicting,
    OutsideCoverage,
}

/// Evidence class is provenance metadata, not a guarantee of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainEvidenceClass {
    SyntheticFixture,
    Simulator,
    HardwareInLoop,
    PhysicalField,
    IndependentlyReviewed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainSurface {
    Paved,
    Gravel,
    Grass,
    Sand,
    Mud,
    SnowIce,
    Rock,
    WetRock,
    ForestFloor,
    Rubble,
    SoftSoil,
    LooseSlope,
    ShallowWaterEdge,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainSensorModality {
    Lidar,
    Camera,
    RgbdCamera,
    Radar,
    Ultrasonic,
    Inertial,
    WheelOdometry,
    SatellitePositioning,
    HumanSurvey,
    Fused,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainCoverageState {
    Validated,
    PartiallyValidated,
    Unvalidated,
    OutsideCoverage,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainUncertaintyKind {
    NotAssessed,
    QualitativeLow,
    QualitativeModerate,
    QualitativeHigh,
    QuantifiedAbsoluteBound,
}

/// Uncertainty radius is in metres. A quantified bound also declares its
/// confidence level; the bound's interpretation belongs to the versioned method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainUncertainty {
    pub kind: TerrainUncertaintyKind,
    pub radius_m: Option<f64>,
    pub confidence_level: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainHazardKind {
    DropOrVoid,
    UnstableSurface,
    WaterEdge,
    LowClearance,
    DynamicObstacle,
    Debris,
    Other,
}

/// Absent is an evidence-backed negative observation; Unknown is not Absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainHazardState {
    Detected,
    Absent,
    Unknown,
    Stale,
    Conflicting,
}

/// One hazard observation tied to source evidence. References must be canonical
/// (sorted, unique, non-empty strings). Confidence describes this observation,
/// not a universal probability of safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainHazardEvidence {
    pub hazard_ref: String,
    pub kind: TerrainHazardKind,
    pub state: TerrainHazardState,
    pub confidence: Option<f64>,
    pub source_evidence_refs: Vec<String>,
}

/// Versioned terrain evidence suitable for a robotics scenario handoff.
///
/// Coordinates are deliberately not embedded. Site, coordinate-frame, map
/// digests, calibration references and source evidence references link back to
/// their authoritative, access-controlled sources. A SHA-256-shaped digest or
/// opaque reference records lineage; it does not authenticate the bytes or
/// establish that the underlying terrain model is adequate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoboticsTerrainEvidenceEnvelope {
    pub schema_version: u16,
    pub evidence_ref: String,
    pub site_ref: String,
    pub coordinate_frame_ref: String,
    pub map_snapshot_sha256: String,
    pub terrain_tile_sha256: String,
    pub terrain_model_id: String,
    pub terrain_model_version: String,
    pub evidence_class: TerrainEvidenceClass,
    pub state: TerrainEvidenceState,
    pub surface: TerrainSurface,
    pub sensor_modality: TerrainSensorModality,
    pub observed_at_unix_seconds: i64,
    pub as_of_unix_seconds: i64,
    pub valid_until_unix_seconds: Option<i64>,
    pub slope_deg: Option<f64>,
    pub cross_slope_deg: Option<f64>,
    pub step_height_m: Option<f64>,
    pub rut_depth_m: Option<f64>,
    pub coverage: TerrainCoverageState,
    pub uncertainty: TerrainUncertainty,
    pub raw_observation_sha256: Option<String>,
    pub calibration_ref: Option<String>,
    pub hardware_configuration_ref: Option<String>,
    pub reviewer_ref: Option<String>,
    pub source_evidence_refs: Vec<String>,
    /// Sorted by hazard_ref, unique, and independently validated.
    pub hazards: Vec<TerrainHazardEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainEvidenceIssue {
    UnsupportedSchemaVersion,
    EmptyRequiredField(&'static str),
    InvalidDigest(&'static str),
    InvalidTimeWindow,
    InvalidTimestamp,
    InvalidFreshnessState,
    InvalidGeometry(&'static str),
    InvalidEvidenceState,
    InvalidCoverageState,
    InvalidSensorModality,
    MissingSourceEvidence,
    InvalidReferences(&'static str),
    MissingRawObservationDigest,
    MissingCalibrationReference,
    MissingHardwareConfigurationReference,
    MissingReviewerReference,
    InvalidUncertainty,
    InvalidHazardReference,
    InvalidHazardConfidence,
    InvalidHazardEvidence,
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate_references(
    references: &[String],
    field: &'static str,
    issues: &mut Vec<TerrainEvidenceIssue>,
) {
    if references.iter().any(|value| !non_empty(value)) {
        issues.push(TerrainEvidenceIssue::InvalidReferences(field));
    }
    if references.windows(2).any(|pair| pair[0] >= pair[1]) {
        issues.push(TerrainEvidenceIssue::InvalidReferences(field));
    }
}

fn finite_non_negative(value: Option<f64>) -> bool {
    value.is_none_or(|v| v.is_finite() && v >= 0.0)
}

impl TerrainHazardEvidence {
    pub fn validate(&self) -> Result<(), Vec<TerrainEvidenceIssue>> {
        let mut issues = Vec::new();

        if !non_empty(&self.hazard_ref) {
            issues.push(TerrainEvidenceIssue::InvalidHazardReference);
        }
        validate_references(
            &self.source_evidence_refs,
            "hazard.source_evidence_refs",
            &mut issues,
        );

        match self.state {
            TerrainHazardState::Detected | TerrainHazardState::Absent => {
                match self.confidence {
                    Some(value) if value.is_finite() && (0.0..=1.0).contains(&value) => {}
                    _ => issues.push(TerrainEvidenceIssue::InvalidHazardConfidence),
                }
                if self.source_evidence_refs.is_empty() {
                    issues.push(TerrainEvidenceIssue::MissingSourceEvidence);
                }
            }
            TerrainHazardState::Unknown
            | TerrainHazardState::Stale
            | TerrainHazardState::Conflicting => {
                if self.confidence.is_some() {
                    issues.push(TerrainEvidenceIssue::InvalidHazardConfidence);
                }
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// True for present or unresolved hazards. Unknown/stale/conflicting evidence
    /// is deliberately not interpreted as proof that the hazard is absent.
    pub fn blocks_clearance_claim(&self) -> bool {
        self.state != TerrainHazardState::Absent
    }
}

impl RoboticsTerrainEvidenceEnvelope {
    /// Validate structure and internal consistency; this does not validate the
    /// terrain physics, map truth, route feasibility, or robot safety.
    pub fn validate(&self) -> Result<(), Vec<TerrainEvidenceIssue>> {
        let mut issues = Vec::new();

        if self.schema_version != ROBOTICS_TERRAIN_SCHEMA_VERSION {
            issues.push(TerrainEvidenceIssue::UnsupportedSchemaVersion);
        }
        for (field, value) in [
            ("evidence_ref", self.evidence_ref.as_str()),
            ("site_ref", self.site_ref.as_str()),
            ("coordinate_frame_ref", self.coordinate_frame_ref.as_str()),
            ("terrain_model_id", self.terrain_model_id.as_str()),
            ("terrain_model_version", self.terrain_model_version.as_str()),
        ] {
            if !non_empty(value) {
                issues.push(TerrainEvidenceIssue::EmptyRequiredField(field));
            }
        }
        for (field, digest) in [
            ("map_snapshot_sha256", self.map_snapshot_sha256.as_str()),
            ("terrain_tile_sha256", self.terrain_tile_sha256.as_str()),
        ] {
            if !valid_sha256(digest) {
                issues.push(TerrainEvidenceIssue::InvalidDigest(field));
            }
        }
        if self
            .raw_observation_sha256
            .as_deref()
            .is_some_and(|digest| !valid_sha256(digest))
        {
            issues.push(TerrainEvidenceIssue::InvalidDigest(
                "raw_observation_sha256",
            ));
        }

        if self.observed_at_unix_seconds < 0 || self.as_of_unix_seconds < 0 {
            issues.push(TerrainEvidenceIssue::InvalidTimestamp);
        }
        if self.as_of_unix_seconds < self.observed_at_unix_seconds
            || self
                .valid_until_unix_seconds
                .is_some_and(|valid_until| valid_until < self.observed_at_unix_seconds)
        {
            issues.push(TerrainEvidenceIssue::InvalidTimeWindow);
        }
        if self.state == TerrainEvidenceState::Available
            && self
                .valid_until_unix_seconds
                .is_some_and(|valid_until| self.as_of_unix_seconds > valid_until)
        {
            issues.push(TerrainEvidenceIssue::InvalidFreshnessState);
        }

        for (field, value) in [
            ("slope_deg", self.slope_deg),
            ("cross_slope_deg", self.cross_slope_deg),
        ] {
            if value.is_some_and(|v| !v.is_finite() || !(0.0..=90.0).contains(&v)) {
                issues.push(TerrainEvidenceIssue::InvalidGeometry(field));
            }
        }
        for (field, value) in [
            ("step_height_m", self.step_height_m),
            ("rut_depth_m", self.rut_depth_m),
        ] {
            if !finite_non_negative(value) {
                issues.push(TerrainEvidenceIssue::InvalidGeometry(field));
            }
        }

        validate_references(
            &self.source_evidence_refs,
            "source_evidence_refs",
            &mut issues,
        );
        if self.state != TerrainEvidenceState::Unknown && self.source_evidence_refs.is_empty() {
            issues.push(TerrainEvidenceIssue::MissingSourceEvidence);
        }

        if self.state == TerrainEvidenceState::Available {
            if self.surface == TerrainSurface::Unknown {
                issues.push(TerrainEvidenceIssue::InvalidEvidenceState);
            }
            if matches!(
                self.coverage,
                TerrainCoverageState::OutsideCoverage | TerrainCoverageState::Unknown
            ) {
                issues.push(TerrainEvidenceIssue::InvalidCoverageState);
            }
            if self.sensor_modality == TerrainSensorModality::Unknown {
                issues.push(TerrainEvidenceIssue::InvalidSensorModality);
            }
        }
        if (self.state == TerrainEvidenceState::OutsideCoverage)
            != (self.coverage == TerrainCoverageState::OutsideCoverage)
        {
            issues.push(TerrainEvidenceIssue::InvalidCoverageState);
        }

        let requires_raw_and_calibration = matches!(
            self.evidence_class,
            TerrainEvidenceClass::HardwareInLoop
                | TerrainEvidenceClass::PhysicalField
                | TerrainEvidenceClass::IndependentlyReviewed
        );
        if requires_raw_and_calibration && self.raw_observation_sha256.is_none() {
            issues.push(TerrainEvidenceIssue::MissingRawObservationDigest);
        }
        if requires_raw_and_calibration
            && self.calibration_ref.as_deref().is_none_or(|value| !non_empty(value))
        {
            issues.push(TerrainEvidenceIssue::MissingCalibrationReference);
        }
        if matches!(
            self.evidence_class,
            TerrainEvidenceClass::HardwareInLoop | TerrainEvidenceClass::PhysicalField
        ) && self
            .hardware_configuration_ref
            .as_deref()
            .is_none_or(|value| !non_empty(value))
        {
            issues.push(TerrainEvidenceIssue::MissingHardwareConfigurationReference);
        }
        if self.evidence_class == TerrainEvidenceClass::IndependentlyReviewed
            && self
                .reviewer_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            issues.push(TerrainEvidenceIssue::MissingReviewerReference);
        }

        match (
            self.uncertainty.kind,
            self.uncertainty.radius_m,
            self.uncertainty.confidence_level,
        ) {
            (TerrainUncertaintyKind::QuantifiedAbsoluteBound, Some(radius), Some(confidence))
                if radius.is_finite()
                    && radius >= 0.0
                    && confidence.is_finite()
                    && (0.0..=1.0).contains(&confidence)
                    && confidence > 0.0 => {}
            (TerrainUncertaintyKind::QuantifiedAbsoluteBound, _, _) => {
                issues.push(TerrainEvidenceIssue::InvalidUncertainty);
            }
            (_, None, None) => {}
            _ => issues.push(TerrainEvidenceIssue::InvalidUncertainty),
        }

        let mut previous_hazard_ref: Option<&str> = None;
        let mut hazard_refs = BTreeSet::new();
        for hazard in &self.hazards {
            if previous_hazard_ref.is_some_and(|previous| previous >= hazard.hazard_ref.as_str())
                || !hazard_refs.insert(hazard.hazard_ref.as_str())
            {
                issues.push(TerrainEvidenceIssue::InvalidReferences("hazards"));
            }
            previous_hazard_ref = Some(&hazard.hazard_ref);
            if let Err(hazard_issues) = hazard.validate() {
                issues.extend(hazard_issues);
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Reports whether this envelope contains a detected or unresolved hazard.
    /// It never means the robot is safe to move when the method returns false.
    pub fn has_present_or_unresolved_hazards(&self) -> bool {
        self.hazards
            .iter()
            .any(TerrainHazardEvidence::blocks_clearance_claim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hazard(
        reference: &str,
        state: TerrainHazardState,
        confidence: Option<f64>,
        sources: Vec<&str>,
    ) -> TerrainHazardEvidence {
        TerrainHazardEvidence {
            hazard_ref: reference.to_string(),
            kind: TerrainHazardKind::DropOrVoid,
            state,
            confidence,
            source_evidence_refs: sources.into_iter().map(str::to_string).collect(),
        }
    }

    fn envelope() -> RoboticsTerrainEvidenceEnvelope {
        RoboticsTerrainEvidenceEnvelope {
            schema_version: ROBOTICS_TERRAIN_SCHEMA_VERSION,
            evidence_ref: "terrain-evidence-001".to_string(),
            site_ref: "site-ref-opaque".to_string(),
            coordinate_frame_ref: "frame:site-enu-v2".to_string(),
            map_snapshot_sha256: "a".repeat(64),
            terrain_tile_sha256: "b".repeat(64),
            terrain_model_id: "terrain-model".to_string(),
            terrain_model_version: "2.1.0".to_string(),
            evidence_class: TerrainEvidenceClass::Simulator,
            state: TerrainEvidenceState::Available,
            surface: TerrainSurface::Gravel,
            sensor_modality: TerrainSensorModality::Lidar,
            observed_at_unix_seconds: 1_000,
            as_of_unix_seconds: 1_010,
            valid_until_unix_seconds: Some(2_000),
            slope_deg: Some(12.0),
            cross_slope_deg: Some(4.0),
            step_height_m: Some(0.1),
            rut_depth_m: Some(0.05),
            coverage: TerrainCoverageState::PartiallyValidated,
            uncertainty: TerrainUncertainty {
                kind: TerrainUncertaintyKind::QuantifiedAbsoluteBound,
                radius_m: Some(0.2),
                confidence_level: Some(0.95),
            },
            raw_observation_sha256: None,
            calibration_ref: None,
            hardware_configuration_ref: None,
            reviewer_ref: None,
            source_evidence_refs: vec!["map-observation-01".to_string()],
            hazards: vec![hazard(
                "hazard-01",
                TerrainHazardState::Absent,
                Some(0.98),
                vec!["hazard-observation-01"],
            )],
        }
    }

    #[test]
    fn valid_terrain_evidence_round_trips_deterministically() {
        let value = envelope();
        assert_eq!(value.validate(), Ok(()));
        let first = serde_json::to_string(&value).unwrap();
        let second: RoboticsTerrainEvidenceEnvelope = serde_json::from_str(&first).unwrap();
        assert_eq!(second, value);
        assert_eq!(serde_json::to_string(&second).unwrap(), first);
    }

    #[test]
    fn unknown_hazard_never_means_absent() {
        let mut value = envelope();
        value.hazards = vec![hazard(
            "hazard-unknown",
            TerrainHazardState::Unknown,
            None,
            vec![],
        )];
        assert_eq!(value.validate(), Ok(()));
        assert!(value.has_present_or_unresolved_hazards());
    }

    #[test]
    fn detected_hazard_requires_confidence_and_source_evidence() {
        let mut value = envelope();
        value.hazards = vec![hazard(
            "hazard-detected",
            TerrainHazardState::Detected,
            None,
            vec![],
        )];
        let issues = value.validate().unwrap_err();
        assert!(issues.contains(&TerrainEvidenceIssue::InvalidHazardConfidence));
        assert!(issues.contains(&TerrainEvidenceIssue::MissingSourceEvidence));
    }

    #[test]
    fn available_terrain_cannot_have_unknown_or_outside_coverage() {
        let mut value = envelope();
        value.coverage = TerrainCoverageState::Unknown;
        assert!(value
            .validate()
            .unwrap_err()
            .contains(&TerrainEvidenceIssue::InvalidCoverageState));
        value.coverage = TerrainCoverageState::OutsideCoverage;
        assert!(value
            .validate()
            .unwrap_err()
            .contains(&TerrainEvidenceIssue::InvalidCoverageState));
    }

    #[test]
    fn expired_available_observation_must_be_marked_stale() {
        let mut value = envelope();
        value.as_of_unix_seconds = 2_500;
        assert!(value
            .validate()
            .unwrap_err()
            .contains(&TerrainEvidenceIssue::InvalidFreshnessState));
        value.state = TerrainEvidenceState::Stale;
        value.coverage = TerrainCoverageState::PartiallyValidated;
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn physical_evidence_requires_raw_digest_calibration_and_configuration() {
        let mut value = envelope();
        value.evidence_class = TerrainEvidenceClass::PhysicalField;
        let issues = value.validate().unwrap_err();
        assert!(issues.contains(&TerrainEvidenceIssue::MissingRawObservationDigest));
        assert!(issues.contains(&TerrainEvidenceIssue::MissingCalibrationReference));
        assert!(issues.contains(&TerrainEvidenceIssue::MissingHardwareConfigurationReference));
    }

    #[test]
    fn quantified_uncertainty_requires_bound_and_confidence() {
        let mut value = envelope();
        value.uncertainty.radius_m = None;
        value.uncertainty.confidence_level = None;
        assert!(value
            .validate()
            .unwrap_err()
            .contains(&TerrainEvidenceIssue::InvalidUncertainty));
    }

    #[test]
    fn references_must_be_sorted_and_unique() {
        let mut value = envelope();
        value.source_evidence_refs = vec!["z-source".to_string(), "a-source".to_string()];
        assert!(value
            .validate()
            .unwrap_err()
            .contains(&TerrainEvidenceIssue::InvalidReferences("source_evidence_refs")));
    }

    #[test]
    fn map_digests_must_be_lowercase_sha256_shape() {
        let mut value = envelope();
        value.map_snapshot_sha256 = "XYZ".to_string();
        assert!(value
            .validate()
            .unwrap_err()
            .contains(&TerrainEvidenceIssue::InvalidDigest("map_snapshot_sha256")));
    }

    #[test]
    fn evidence_envelope_does_not_embed_raw_coordinates() {
        let value = serde_json::to_value(envelope()).unwrap();
        assert!(value.get("latitude").is_none());
        assert!(value.get("longitude").is_none());
        assert!(value.get("latitude_deg").is_none());
        assert!(value.get("longitude_deg").is_none());
    }
}
