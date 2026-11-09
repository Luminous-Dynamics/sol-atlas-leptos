// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Renderer-independent evidence metadata for Sol Atlas layers.
//!
//! Structural validation here does not establish scientific truth. In
//! particular, a content digest is not source authentication, a signature is
//! not proof of correctness, and an anomaly is not a confirmed detection.
//! The module keeps observations, model outputs, scenario assumptions and
//! speculative hypotheses explicitly distinct.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Primary epistemic class assigned to a layer record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicClass {
    ObservedMeasurement,
    ArchaeologicalEvidence,
    PrimaryHistoricalSource,
    OralOrCommunityHistory,
    CompiledSecondarySource,
    ModelInference,
    ScenarioAssumption,
    SpeculativeHypothesis,
    ConfirmedDetection,
}

/// Value availability is data. `Present` does not imply empirical
/// measurement: the epistemic class distinguishes observed values from model
/// outputs, assumptions and hypotheses. Missing states must never become 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Missingness {
    Present,
    NotObserved,
    Unknown,
    NotApplicable,
    WithheldByCustodian,
    UnavailableFromSource,
    InvalidOrRejected,
}

/// Verification describes a process performed on this record; it is not a
/// general-purpose truth score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    NotVerified,
    StructurallyValidated,
    IndependentlyVerified,
    Disputed,
    Revoked,
}

/// Source snapshot and its reuse constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub source_id: String,
    pub locator: String,
    pub source_version: Option<String>,
    /// Content digest in the form sha256:<64 lowercase hexadecimal chars>.
    pub snapshot_digest: Option<String>,
    pub licence_id: Option<String>,
    pub attribution: Option<String>,
    /// UTC Unix epoch seconds, not local wall-clock time.
    pub retrieved_at_unix_seconds: Option<i64>,
}

impl SourceRef {
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.source_id.trim().is_empty() {
            return Err(EvidenceError::EmptyField("source_id"));
        }
        if self.locator.trim().is_empty() {
            return Err(EvidenceError::EmptyField("source locator"));
        }
        if let Some(digest) = &self.snapshot_digest {
            validate_sha256(digest)?;
        }
        Ok(())
    }
}

/// Explicit temporal scope. Both values are UTC Unix epoch seconds. Open
/// bounds are allowed; closed intervals must be ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalExtent {
    pub valid_from_unix_seconds: Option<i64>,
    pub valid_until_unix_seconds: Option<i64>,
    pub observed_at_unix_seconds: Option<i64>,
    pub published_at_unix_seconds: Option<i64>,
}

impl TemporalExtent {
    pub fn validate(self) -> Result<(), EvidenceError> {
        if let (Some(start), Some(end)) =
            (self.valid_from_unix_seconds, self.valid_until_unix_seconds)
        {
            if end < start {
                return Err(EvidenceError::InvalidTemporalInterval { start, end });
            }
        }
        Ok(())
    }
}

/// Geographic scope. Geometry is referenced rather than interpreted by this
/// metadata module; the renderer remains a consumer, not the source of truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpatialScope {
    Global,
    Region {
        region_id: String,
        boundary_source_id: Option<String>,
        crs: String,
    },
    Point {
        latitude: f64,
        longitude: f64,
        crs: String,
    },
    GeometryRef {
        geometry_ref: String,
        crs: String,
        resolution_metres: Option<f64>,
    },
}

impl SpatialScope {
    pub fn validate(&self) -> Result<(), EvidenceError> {
        match self {
            Self::Global => Ok(()),
            Self::Region { region_id, crs, .. } => {
                if region_id.trim().is_empty() {
                    return Err(EvidenceError::EmptyField("region_id"));
                }
                validate_crs(crs)
            }
            Self::Point {
                latitude,
                longitude,
                crs,
            } => {
                if !latitude.is_finite() || !(-90.0..=90.0).contains(latitude) {
                    return Err(EvidenceError::InvalidCoordinate("latitude"));
                }
                if !longitude.is_finite() || !(-180.0..=180.0).contains(longitude) {
                    return Err(EvidenceError::InvalidCoordinate("longitude"));
                }
                validate_crs(crs)
            }
            Self::GeometryRef {
                geometry_ref,
                crs,
                resolution_metres,
            } => {
                if geometry_ref.trim().is_empty() {
                    return Err(EvidenceError::EmptyField("geometry_ref"));
                }
                validate_crs(crs)?;
                if let Some(resolution) = resolution_metres {
                    if !resolution.is_finite() || *resolution <= 0.0 {
                        return Err(EvidenceError::InvalidPositiveValue("resolution_metres"));
                    }
                }
                Ok(())
            }
        }
    }
}

/// One measured quantity. Stocks and flows are distinguished by quantity/unit
/// conventions in the producer's schema; this type never silently converts
/// between them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub quantity: String,
    pub value: f64,
    pub unit: String,
    /// Optional one-sigma standard deviation in the same unit as value.
    pub standard_uncertainty: Option<f64>,
    pub method_ref: Option<String>,
    pub aggregation_window: Option<String>,
}

impl Measurement {
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.quantity.trim().is_empty() {
            return Err(EvidenceError::EmptyField("quantity"));
        }
        if self.unit.trim().is_empty() {
            return Err(EvidenceError::EmptyField("unit"));
        }
        if !self.value.is_finite() {
            return Err(EvidenceError::NonFiniteValue("measurement.value"));
        }
        if let Some(uncertainty) = self.standard_uncertainty {
            if !uncertainty.is_finite() || uncertainty < 0.0 {
                return Err(EvidenceError::InvalidNonNegativeValue(
                    "standard_uncertainty",
                ));
            }
        }
        Ok(())
    }
}

/// Complete evidence envelope attached to a single Sol-Atlas layer record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceMetadata {
    pub record_id: String,
    pub epistemic_class: EpistemicClass,
    pub missingness: Missingness,
    pub temporal: TemporalExtent,
    pub spatial: SpatialScope,
    pub sources: Vec<SourceRef>,
    pub measurement: Option<Measurement>,
    pub model_ref: Option<String>,
    pub scenario_ref: Option<String>,
    pub hypothesis_ref: Option<String>,
    pub derivation_refs: Vec<String>,
    pub verifier_refs: Vec<String>,
    pub rights_and_access: Vec<String>,
    pub limitations: Vec<String>,
    pub verification_status: VerificationStatus,
}

impl EvidenceMetadata {
    /// Validate structural requirements only. This does not fetch sources,
    /// check scientific validity, or authorize redistribution.
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.record_id.trim().is_empty() {
            return Err(EvidenceError::EmptyField("record_id"));
        }
        self.temporal.validate()?;
        self.spatial.validate()?;

        for source in &self.sources {
            source.validate()?;
        }

        match self.missingness {
            Missingness::Present => {
                let measurement = self
                    .measurement
                    .as_ref()
                    .ok_or(EvidenceError::MeasuredValueMissing)?;
                measurement.validate()?;
            }
            _ if self.measurement.is_some() => {
                return Err(EvidenceError::ValuePresentForMissingRecord);
            }
            _ => {}
        }

        match self.epistemic_class {
            EpistemicClass::ObservedMeasurement => {
                self.require_source()?;
                if self.missingness != Missingness::Present {
                    return Err(EvidenceError::ObservedMeasurementNotMeasured);
                }
            }
            EpistemicClass::ArchaeologicalEvidence
            | EpistemicClass::PrimaryHistoricalSource
            | EpistemicClass::OralOrCommunityHistory
            | EpistemicClass::CompiledSecondarySource => self.require_source()?,
            EpistemicClass::ModelInference => {
                if self.model_ref.as_deref().unwrap_or("").trim().is_empty() {
                    return Err(EvidenceError::MissingReference("model_ref"));
                }
                if self.sources.is_empty() && self.derivation_refs.is_empty() {
                    return Err(EvidenceError::ModelInferenceWithoutLineage);
                }
            }
            EpistemicClass::ScenarioAssumption => {
                if self.scenario_ref.as_deref().unwrap_or("").trim().is_empty() {
                    return Err(EvidenceError::MissingReference("scenario_ref"));
                }
            }
            EpistemicClass::SpeculativeHypothesis => {
                if self
                    .hypothesis_ref
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    return Err(EvidenceError::MissingReference("hypothesis_ref"));
                }
            }
            EpistemicClass::ConfirmedDetection => {
                self.require_source()?;
                if self.verifier_refs.is_empty() {
                    return Err(EvidenceError::ConfirmedDetectionWithoutVerifier);
                }
                if self.verification_status != VerificationStatus::IndependentlyVerified {
                    return Err(EvidenceError::ConfirmedDetectionNotIndependentlyVerified);
                }
            }
        }

        if self.verification_status == VerificationStatus::IndependentlyVerified
            && self.verifier_refs.is_empty()
        {
            return Err(EvidenceError::IndependentVerificationWithoutReceipt);
        }
        Ok(())
    }

    fn require_source(&self) -> Result<(), EvidenceError> {
        if self.sources.is_empty() {
            Err(EvidenceError::MissingSource)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    EmptyField(&'static str),
    InvalidDigest,
    InvalidTemporalInterval { start: i64, end: i64 },
    InvalidCoordinate(&'static str),
    InvalidPositiveValue(&'static str),
    InvalidNonNegativeValue(&'static str),
    NonFiniteValue(&'static str),
    InvalidCrs,
    MissingSource,
    MissingReference(&'static str),
    MeasuredValueMissing,
    ValuePresentForMissingRecord,
    ObservedMeasurementNotMeasured,
    ModelInferenceWithoutLineage,
    ConfirmedDetectionWithoutVerifier,
    ConfirmedDetectionNotIndependentlyVerified,
    IndependentVerificationWithoutReceipt,
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDigest => {
                write!(f, "snapshot digest must be sha256:<64 lowercase hex chars>")
            }
            Self::InvalidTemporalInterval { start, end } => {
                write!(
                    f,
                    "temporal interval ends before it starts: {start} > {end}"
                )
            }
            Self::InvalidCoordinate(field) => write!(f, "invalid {field} coordinate"),
            Self::InvalidPositiveValue(field) => write!(f, "{field} must be finite and positive"),
            Self::InvalidNonNegativeValue(field) => {
                write!(f, "{field} must be finite and non-negative")
            }
            Self::NonFiniteValue(field) => write!(f, "{field} must be finite"),
            Self::InvalidCrs => write!(f, "coordinate reference system must not be empty"),
            Self::MissingSource => write!(
                f,
                "this epistemic class requires at least one source reference"
            ),
            Self::MissingReference(field) => {
                write!(f, "{field} is required for this epistemic class")
            }
            Self::MeasuredValueMissing => write!(f, "measured records require a measurement"),
            Self::ValuePresentForMissingRecord => write!(
                f,
                "non-measured records must not contain a numeric measurement"
            ),
            Self::ObservedMeasurementNotMeasured => {
                write!(f, "observed_measurement requires a present value")
            }
            Self::ModelInferenceWithoutLineage => write!(
                f,
                "model inference requires a source or derivation reference"
            ),
            Self::ConfirmedDetectionWithoutVerifier => write!(
                f,
                "confirmed detection requires an independent verifier receipt"
            ),
            Self::ConfirmedDetectionNotIndependentlyVerified => write!(
                f,
                "confirmed detection requires independently_verified status"
            ),
            Self::IndependentVerificationWithoutReceipt => write!(
                f,
                "independent verification requires at least one verifier receipt"
            ),
        }
    }
}

impl std::error::Error for EvidenceError {}

fn validate_sha256(digest: &str) -> Result<(), EvidenceError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(EvidenceError::InvalidDigest);
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(EvidenceError::InvalidDigest);
    }
    Ok(())
}

fn validate_crs(crs: &str) -> Result<(), EvidenceError> {
    if crs.trim().is_empty() {
        Err(EvidenceError::InvalidCrs)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporal() -> TemporalExtent {
        TemporalExtent {
            valid_from_unix_seconds: Some(1_700_000_000),
            valid_until_unix_seconds: Some(1_700_003_600),
            observed_at_unix_seconds: Some(1_700_004_000),
            published_at_unix_seconds: Some(1_700_010_000),
        }
    }

    fn source() -> SourceRef {
        SourceRef {
            source_id: "energy-site-registry-v1".into(),
            locator: "https://example.invalid/energy/sites".into(),
            source_version: Some("v1".into()),
            snapshot_digest: Some(format!("sha256:{}", "a".repeat(64))),
            licence_id: Some("CC-BY-4.0".into()),
            attribution: Some("Example source".into()),
            retrieved_at_unix_seconds: Some(1_700_010_000),
        }
    }

    fn measured_record() -> EvidenceMetadata {
        EvidenceMetadata {
            record_id: "site-001-capacity".into(),
            epistemic_class: EpistemicClass::ObservedMeasurement,
            missingness: Missingness::Present,
            temporal: temporal(),
            spatial: SpatialScope::Point {
                latitude: -26.2,
                longitude: 28.0,
                crs: "EPSG:4326".into(),
            },
            sources: vec![source()],
            measurement: Some(Measurement {
                quantity: "installed_capacity".into(),
                value: 120.0,
                unit: "MW".into(),
                standard_uncertainty: Some(1.5),
                method_ref: Some("registry-method-v1".into()),
                aggregation_window: None,
            }),
            model_ref: None,
            scenario_ref: None,
            hypothesis_ref: None,
            derivation_refs: vec![],
            verifier_refs: vec![],
            rights_and_access: vec!["public".into()],
            limitations: vec!["nameplate capacity, not delivered energy".into()],
            verification_status: VerificationStatus::StructurallyValidated,
        }
    }

    #[test]
    fn measured_record_requires_units_source_and_temporal_spatial_scope() {
        assert!(measured_record().validate().is_ok());
    }

    #[test]
    fn invalid_digest_is_rejected() {
        let mut record = measured_record();
        record.sources[0].snapshot_digest = Some("md5:bad".into());
        assert_eq!(record.validate(), Err(EvidenceError::InvalidDigest));
    }

    #[test]
    fn missing_values_are_not_coerced_to_zero() {
        let mut record = measured_record();
        record.missingness = Missingness::Unknown;
        record.measurement = None;
        assert!(record.validate().is_ok());

        record.measurement = Some(Measurement {
            quantity: "capacity".into(),
            value: 0.0,
            unit: "MW".into(),
            standard_uncertainty: None,
            method_ref: None,
            aggregation_window: None,
        });
        assert_eq!(
            record.validate(),
            Err(EvidenceError::ValuePresentForMissingRecord)
        );
    }

    #[test]
    fn scenario_assumption_requires_scenario_reference() {
        let mut record = measured_record();
        record.epistemic_class = EpistemicClass::ScenarioAssumption;
        record.missingness = Missingness::Unknown;
        record.measurement = None;
        record.scenario_ref = None;
        assert_eq!(
            record.validate(),
            Err(EvidenceError::MissingReference("scenario_ref"))
        );

        record.scenario_ref = Some("water-shock-scenario-v1".into());
        assert!(record.validate().is_ok());
    }

    #[test]
    fn scenario_numeric_value_is_present_but_not_misclassified_as_observed() {
        let mut record = measured_record();
        record.epistemic_class = EpistemicClass::ScenarioAssumption;
        record.scenario_ref = Some("drought-response-v1".into());
        record.missingness = Missingness::Present;
        assert!(record.validate().is_ok());
    }

    #[test]
    fn model_inference_requires_model_and_lineage() {
        let mut record = measured_record();
        record.epistemic_class = EpistemicClass::ModelInference;
        record.model_ref = Some("grid-stress-model-v2".into());
        record.sources.clear();
        assert_eq!(
            record.validate(),
            Err(EvidenceError::ModelInferenceWithoutLineage)
        );

        record.derivation_refs.push("input-observation-001".into());
        assert!(record.validate().is_ok());
    }

    #[test]
    fn confirmed_detection_requires_independent_verifier_receipt() {
        let mut record = measured_record();
        record.epistemic_class = EpistemicClass::ConfirmedDetection;
        assert_eq!(
            record.validate(),
            Err(EvidenceError::ConfirmedDetectionWithoutVerifier)
        );

        record
            .verifier_refs
            .push("independent-review-receipt-1".into());
        assert_eq!(
            record.validate(),
            Err(EvidenceError::ConfirmedDetectionNotIndependentlyVerified)
        );

        record.verification_status = VerificationStatus::IndependentlyVerified;
        assert!(record.validate().is_ok());
    }

    #[test]
    fn independent_verification_without_receipt_is_rejected() {
        let mut record = measured_record();
        record.verification_status = VerificationStatus::IndependentlyVerified;
        assert_eq!(
            record.validate(),
            Err(EvidenceError::IndependentVerificationWithoutReceipt)
        );
    }

    #[test]
    fn invalid_temporal_interval_is_rejected() {
        let mut record = measured_record();
        record.temporal.valid_from_unix_seconds = Some(20);
        record.temporal.valid_until_unix_seconds = Some(10);
        assert_eq!(
            record.validate(),
            Err(EvidenceError::InvalidTemporalInterval { start: 20, end: 10 })
        );
    }

    #[test]
    fn invalid_coordinates_are_rejected() {
        let mut record = measured_record();
        record.spatial = SpatialScope::Point {
            latitude: 100.0,
            longitude: 28.0,
            crs: "EPSG:4326".into(),
        };
        assert_eq!(
            record.validate(),
            Err(EvidenceError::InvalidCoordinate("latitude"))
        );
    }

    #[test]
    fn non_finite_measurement_and_negative_uncertainty_are_rejected() {
        let mut record = measured_record();
        record.measurement.as_mut().unwrap().value = f64::NAN;
        assert_eq!(
            record.validate(),
            Err(EvidenceError::NonFiniteValue("measurement.value"))
        );

        record = measured_record();
        record.measurement.as_mut().unwrap().standard_uncertainty = Some(-1.0);
        assert_eq!(
            record.validate(),
            Err(EvidenceError::InvalidNonNegativeValue(
                "standard_uncertainty"
            ))
        );
    }

    #[test]
    fn oral_and_community_history_requires_a_source_without_forcing_a_scalar_measurement() {
        let mut record = measured_record();
        record.epistemic_class = EpistemicClass::OralOrCommunityHistory;
        record.missingness = Missingness::NotApplicable;
        record.measurement = None;
        assert!(record.validate().is_ok());
    }

    #[test]
    fn speculative_hypothesis_requires_explicit_hypothesis_reference() {
        let mut record = measured_record();
        record.epistemic_class = EpistemicClass::SpeculativeHypothesis;
        record.missingness = Missingness::Unknown;
        record.measurement = None;
        record.hypothesis_ref = None;
        assert_eq!(
            record.validate(),
            Err(EvidenceError::MissingReference("hypothesis_ref"))
        );

        record.hypothesis_ref = Some("ocean-world-biosphere-hypothesis-01".into());
        assert!(record.validate().is_ok());
    }
}
