// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned, renderer-independent contracts for Earth-interior observations
//! and state estimates.
//!
//! This module deliberately contains no network clients or inversion solver.
//! It makes the boundary between observations and inferences explicit so
//! adapters, estimators, provenance services, and renderers cannot silently
//! collapse them into one unqualified "live" Earth state.

use serde::{Deserialize, Serialize};

/// Initial wire-contract version. Breaking changes require a new version.
pub const EARTH_INTERIOR_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityState {
    /// No local quality assessment has been performed.
    Unassessed,
    /// The source provider marked the record usable; this is not independent validation.
    SourceAccepted,
    /// A documented local review has been performed.
    LocallyReviewed,
    /// A quality concern remains unresolved.
    Suspect,
    /// Rejected for the intended analysis; preserve the record for audit.
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeismicPhase {
    Pkikp,
    Pkip,
    Pkp,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateMethod {
    GeomagneticFieldInversion,
    CoreSurfaceFlowInversion,
    SeismicWaveformAnalysis,
    JointAssimilation,
    NumericalSimulation,
    PublishedReferenceModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityStatus {
    /// No dedicated identifiability analysis has been recorded.
    NotAssessed,
    /// Some dimensions are constrained; others remain ambiguous.
    Partial,
    /// Identified only to the resolution and assumptions declared by the model.
    IdentifiedWithinResolution,
    /// More than one materially different state remains consistent with the evidence.
    NonIdentifiable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyInterpretation {
    ConfidenceInterval,
    CredibleInterval,
    PredictionInterval,
    ErrorBound,
    EnsembleRange,
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadialSignConvention {
    PositiveOutward,
    PositiveInward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationReferenceFrame {
    RelativeToMantle,
    RelativeToCrust,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeWindow {
    /// POSIX/Unix nanoseconds from the Unix epoch, using the source adapter's
    /// documented leap-second convention. This is not a substitute for the
    /// original source timestamp and time-scale metadata.
    pub start_unix_ns: i128,
    pub end_unix_ns: i128,
}

impl TimeWindow {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.end_unix_ns < self.start_unix_ns {
            return Err(ValidationError::InvalidTimeWindow);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeodeticPosition {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_m: Option<f64>,
}

impl GeodeticPosition {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.latitude_deg.is_finite() {
            return Err(ValidationError::NonFiniteValue("latitude_deg"));
        }
        if !(-90.0..=90.0).contains(&self.latitude_deg) {
            return Err(ValidationError::InvalidLatitude);
        }
        if !self.longitude_deg.is_finite() {
            return Err(ValidationError::NonFiniteValue("longitude_deg"));
        }
        // Do not silently wrap longitudes: normalization belongs in an explicit,
        // versioned adapter transform so source coordinates remain auditable.
        if !(-180.0..=180.0).contains(&self.longitude_deg) {
            return Err(ValidationError::InvalidLongitude);
        }
        if self.altitude_m.is_some_and(|value| !value.is_finite()) {
            return Err(ValidationError::InvalidAltitude);
        }
        Ok(())
    }
}

/// Spatial support is explicit: surface observations, core-boundary estimates,
/// and global scalar parameters are not interchangeable locations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EstimateSupport {
    GeodeticPoint { position: GeodeticPosition },
    CoreMantleBoundaryPoint {
        latitude_deg: f64,
        longitude_deg: f64,
        radius_m: f64,
    },
    GlobalScalar,
}

impl EstimateSupport {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::GeodeticPoint { position } => position.validate(),
            Self::CoreMantleBoundaryPoint {
                latitude_deg,
                longitude_deg,
                radius_m,
            } => {
                if !latitude_deg.is_finite() {
                    return Err(ValidationError::NonFiniteValue("latitude_deg"));
                }
                if !(-90.0..=90.0).contains(latitude_deg) {
                    return Err(ValidationError::InvalidLatitude);
                }
                if !longitude_deg.is_finite() {
                    return Err(ValidationError::NonFiniteValue("longitude_deg"));
                }
                if !(-180.0..=180.0).contains(longitude_deg) {
                    return Err(ValidationError::InvalidLongitude);
                }
                require_finite(*radius_m, "radius_m")?;
                if *radius_m <= 0.0 {
                    return Err(ValidationError::InvalidSpatialSupport);
                }
                Ok(())
            }
            Self::GlobalScalar => Ok(()),
        }
    }
}

/// Points to the source product/record. A locator is not proof of authenticity;
/// a digest checks captured bytes for equality but does not authenticate their publisher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProvenance {
    pub source_id: String,
    pub product_id: String,
    pub product_version: Option<String>,
    pub record_id: String,
    pub locator: Option<String>,
    pub licence: Option<String>,
    /// SHA-256 of the exact captured artifact bytes, when those bytes are available.
    pub captured_sha256: Option<String>,
}

impl SourceProvenance {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_non_empty(&self.source_id, "source_id")?;
        require_non_empty(&self.product_id, "product_id")?;
        require_non_empty(&self.record_id, "record_id")?;
        if let Some(version) = &self.product_version {
            require_non_empty(version, "product_version")?;
        }
        if let Some(locator) = &self.locator {
            require_non_empty(locator, "locator")?;
        }
        if let Some(licence) = &self.licence {
            require_non_empty(licence, "licence")?;
        }
        if let Some(digest) = &self.captured_sha256 {
            validate_sha256(digest)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObservationValue {
    /// Vector already transformed into local North-East-Down coordinates.
    /// Spacecraft/body-frame values must not be labelled NED before a documented transform.
    MagneticVectorNedNt {
        north_nt: f64,
        east_nt: f64,
        down_nt: f64,
        standard_uncertainty_nt: Option<[f64; 3]>,
    },
    /// Differential travel-time measurement for a declared phase and event pair.
    /// It is not, by itself, a direct measurement of inner-core rotation.
    SeismicDifferentialTravelTimeMs {
        phase: SeismicPhase,
        differential_time_ms: f64,
        standard_uncertainty_ms: Option<f64>,
        event_pair_id: String,
    },
}

impl ObservationValue {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::MagneticVectorNedNt {
                north_nt,
                east_nt,
                down_nt,
                standard_uncertainty_nt,
            } => {
                require_finite(*north_nt, "north_nt")?;
                require_finite(*east_nt, "east_nt")?;
                require_finite(*down_nt, "down_nt")?;
                if let Some(uncertainties) = standard_uncertainty_nt {
                    for value in uncertainties {
                        if !value.is_finite() {
                            return Err(ValidationError::NonFiniteValue(
                                "standard_uncertainty_nt",
                            ));
                        }
                        if *value < 0.0 {
                            return Err(ValidationError::NegativeUncertainty(
                                "standard_uncertainty_nt",
                            ));
                        }
                    }
                }
            }
            Self::SeismicDifferentialTravelTimeMs {
                differential_time_ms,
                standard_uncertainty_ms,
                event_pair_id,
                ..
            } => {
                require_finite(*differential_time_ms, "differential_time_ms")?;
                require_non_empty(event_pair_id, "event_pair_id")?;
                if let Some(value) = standard_uncertainty_ms {
                    require_finite(*value, "standard_uncertainty_ms")?;
                    if *value < 0.0 {
                        return Err(ValidationError::NegativeUncertainty(
                            "standard_uncertainty_ms",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationRecord {
    pub schema_version: u16,
    pub id: String,
    pub time: TimeWindow,
    /// Station, satellite, instrument, or network-local observation identifier.
    pub platform_or_station_id: String,
    /// Optional means unknown/unavailable; it must not be replaced with a guessed location.
    pub position: Option<GeodeticPosition>,
    pub value: ObservationValue,
    pub provenance: SourceProvenance,
    pub quality: QualityState,
}

impl ObservationRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != EARTH_INTERIOR_SCHEMA_VERSION {
            return Err(ValidationError::UnsupportedSchemaVersion);
        }
        require_non_empty(&self.id, "id")?;
        require_non_empty(&self.platform_or_station_id, "platform_or_station_id")?;
        self.time.validate()?;
        if let Some(position) = &self.position {
            position.validate()?;
        }
        self.value.validate()?;
        self.provenance.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UncertaintyInterval {
    pub lower: f64,
    pub upper: f64,
    pub interpretation: UncertaintyInterpretation,
    /// Nominal coverage/level when defined (for example 0.95). Leave absent
    /// when a probability interpretation is not supported by the source.
    pub nominal_coverage: Option<f64>,
}

impl UncertaintyInterval {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_finite(self.lower, "interval.lower")?;
        require_finite(self.upper, "interval.upper")?;
        if self.lower > self.upper {
            return Err(ValidationError::InvalidInterval);
        }
        if let Some(coverage) = self.nominal_coverage {
            if !coverage.is_finite() {
                return Err(ValidationError::NonFiniteValue("nominal_coverage"));
            }
            if !(0.0 < coverage && coverage <= 1.0) {
                return Err(ValidationError::InvalidCoverage);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EstimateValue {
    RadialMagneticFieldNt {
        value_nt: f64,
        positive_direction: RadialSignConvention,
        interval: Option<UncertaintyInterval>,
    },
    /// Local tangential flow at the core-mantle boundary, in metres/year.
    CoreSurfaceFlowMetersPerYear {
        north_m_per_year: f64,
        east_m_per_year: f64,
        north_interval: Option<UncertaintyInterval>,
        east_interval: Option<UncertaintyInterval>,
    },
    /// A model-derived relative rotation estimate; it is not a directly observed rate.
    InnerCoreDifferentialRotationDegreesPerYear {
        value_degrees_per_year: f64,
        reference_frame: RotationReferenceFrame,
        interval: Option<UncertaintyInterval>,
    },
}

impl EstimateValue {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::RadialMagneticFieldNt { value_nt, interval } => {
                require_finite(*value_nt, "value_nt")?;
                validate_optional_interval(interval)?;
            }
            Self::CoreSurfaceFlowMetersPerYear {
                north_m_per_year,
                east_m_per_year,
                north_interval,
                east_interval,
            } => {
                require_finite(*north_m_per_year, "north_m_per_year")?;
                require_finite(*east_m_per_year, "east_m_per_year")?;
                validate_optional_interval(north_interval)?;
                validate_optional_interval(east_interval)?;
            }
            Self::InnerCoreDifferentialRotationDegreesPerYear {
                value_degrees_per_year,
                interval,
            } => {
                require_finite(*value_degrees_per_year, "value_degrees_per_year")?;
                validate_optional_interval(interval)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEstimate {
    pub schema_version: u16,
    pub id: String,
    pub valid_time: TimeWindow,
    pub spatial_support: EstimateSupport,
    pub method: EstimateMethod,
    pub model_id: String,
    pub model_version: String,
    pub model_config_sha256: Option<String>,
    /// IDs refer to ObservationRecord::id or to immutable records in an upstream archive.
    pub input_observation_ids: Vec<String>,
    /// External/reference artifacts used in the derivation. Unknown licence or digest stays absent.
    pub evidence: Vec<SourceProvenance>,
    pub value: EstimateValue,
    pub identifiability: IdentifiabilityStatus,
    /// Concrete unresolved alternatives, sensitivity or null-space limitations.
    pub unresolved_ambiguities: Vec<String>,
    /// Explicit model assumptions; do not infer an empty list to mean "assumption-free".
    pub assumptions: Vec<String>,
}

impl StateEstimate {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != EARTH_INTERIOR_SCHEMA_VERSION {
            return Err(ValidationError::UnsupportedSchemaVersion);
        }
        require_non_empty(&self.id, "id")?;
        require_non_empty(&self.model_id, "model_id")?;
        require_non_empty(&self.model_version, "model_version")?;
        self.valid_time.validate()?;
        self.spatial_support.validate()?;
        match (&self.value, &self.spatial_support) {
            (
                EstimateValue::RadialMagneticFieldNt { .. },
                EstimateSupport::GeodeticPoint { .. }
                    | EstimateSupport::CoreMantleBoundaryPoint { .. },
            )
            | (
                EstimateValue::CoreSurfaceFlowMetersPerYear { .. },
                EstimateSupport::CoreMantleBoundaryPoint { .. },
            )
            | (
                EstimateValue::InnerCoreDifferentialRotationDegreesPerYear { .. },
                EstimateSupport::GlobalScalar,
            ) => {}
            _ => return Err(ValidationError::InvalidSpatialSupport),
        }
        if let Some(digest) = &self.model_config_sha256 {
            validate_sha256(digest)?;
        }
        for id in &self.input_observation_ids {
            require_non_empty(id, "input_observation_ids[]")?;
        }
        for (index, id) in self.input_observation_ids.iter().enumerate() {
            if self.input_observation_ids[..index].contains(id) {
                return Err(ValidationError::DuplicateObservationId);
            }
        }
        for evidence in &self.evidence {
            evidence.validate()?;
        }
        if self.method != EstimateMethod::NumericalSimulation
            && self.input_observation_ids.is_empty()
            && self.evidence.is_empty()
        {
            return Err(ValidationError::MissingEvidence);
        }
        if matches!(
            self.identifiability,
            IdentifiabilityStatus::Partial | IdentifiabilityStatus::NonIdentifiable
        ) && !self
            .unresolved_ambiguities
            .iter()
            .any(|item| !item.trim().is_empty())
        {
            return Err(ValidationError::MissingIdentifiabilityCaveat);
        }
        for assumption in &self.assumptions {
            require_non_empty(assumption, "assumptions[]")?;
        }
        self.value.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    UnsupportedSchemaVersion,
    EmptyField(&'static str),
    InvalidTimeWindow,
    InvalidLatitude,
    InvalidLongitude,
    InvalidAltitude,
    NonFiniteValue(&'static str),
    NegativeUncertainty(&'static str),
    InvalidInterval,
    InvalidCoverage,
    InvalidSha256,
    MissingEvidence,
    DuplicateObservationId,
    MissingIdentifiabilityCaveat,
    InvalidSpatialSupport,
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::EmptyField(field));
    }
    Ok(())
}

fn require_finite(value: f64, field: &'static str) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError::NonFiniteValue(field));
    }
    Ok(())
}

fn validate_optional_interval(
    interval: &Option<UncertaintyInterval>,
) -> Result<(), ValidationError> {
    if let Some(interval) = interval {
        interval.validate()?;
    }
    Ok(())
}

fn validate_sha256(digest: &str) -> Result<(), ValidationError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidSha256);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> SourceProvenance {
        SourceProvenance {
            source_id: "example-observatory".into(),
            product_id: "magnetic-vector".into(),
            product_version: Some("1".into()),
            record_id: "record-0001".into(),
            locator: Some("https://example.invalid/record-0001".into()),
            licence: None,
            captured_sha256: Some("a".repeat(64)),
        }
    }

    fn magnetic_observation() -> ObservationRecord {
        ObservationRecord {
            schema_version: EARTH_INTERIOR_SCHEMA_VERSION,
            id: "obs-0001".into(),
            time: TimeWindow {
                start_unix_ns: 1_700_000_000_000_000_000,
                end_unix_ns: 1_700_000_000_000_000_000,
            },
            platform_or_station_id: "station-1".into(),
            position: Some(GeodeticPosition {
                latitude_deg: -26.2,
                longitude_deg: 28.0,
                altitude_m: Some(1_700.0),
            }),
            value: ObservationValue::MagneticVectorNedNt {
                north_nt: 22_000.0,
                east_nt: 1_500.0,
                down_nt: -35_000.0,
                standard_uncertainty_nt: Some([1.0, 1.0, 1.5]),
            },
            provenance: source(),
            quality: QualityState::Unassessed,
        }
    }

    #[test]
    fn valid_observation_round_trips_without_erasing_unknown_quality() {
        let observation = magnetic_observation();
        observation.validate().unwrap();
        let encoded = serde_json::to_string(&observation).unwrap();
        let decoded: ObservationRecord = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, observation);
        assert_eq!(decoded.quality, QualityState::Unassessed);
    }

    #[test]
    fn geodetic_position_rejects_out_of_range_coordinates() {
        let position = GeodeticPosition {
            latitude_deg: 91.0,
            longitude_deg: 0.0,
            altitude_m: None,
        };
        assert_eq!(position.validate(), Err(ValidationError::InvalidLatitude));
    }

    #[test]
    fn coordinates_are_not_silently_wrapped() {
        let position = GeodeticPosition {
            latitude_deg: 0.0,
            longitude_deg: 181.0,
            altitude_m: None,
        };
        assert_eq!(position.validate(), Err(ValidationError::InvalidLongitude));
    }

    #[test]
    fn invalid_time_window_fails_closed() {
        let time = TimeWindow {
            start_unix_ns: 10,
            end_unix_ns: 9,
        };
        assert_eq!(time.validate(), Err(ValidationError::InvalidTimeWindow));
    }

    #[test]
    fn invalid_or_negative_uncertainty_is_rejected() {
        let mut observation = magnetic_observation();
        observation.value = ObservationValue::MagneticVectorNedNt {
            north_nt: 1.0,
            east_nt: 2.0,
            down_nt: 3.0,
            standard_uncertainty_nt: Some([1.0, -0.1, 1.0]),
        };
        assert_eq!(
            observation.validate(),
            Err(ValidationError::NegativeUncertainty(
                "standard_uncertainty_nt"
            ))
        );
    }

    #[test]
    fn captured_digest_must_be_a_sha256_hex_string() {
        let mut observation = magnetic_observation();
        observation.provenance.captured_sha256 = Some("not-a-digest".into());
        assert_eq!(
            observation.validate(),
            Err(ValidationError::InvalidSha256)
        );
    }

    #[test]
    fn inversion_estimate_requires_evidence_and_identifiability_caveats() {
        let estimate = StateEstimate {
            schema_version: EARTH_INTERIOR_SCHEMA_VERSION,
            id: "estimate-1".into(),
            valid_time: TimeWindow {
                start_unix_ns: 1,
                end_unix_ns: 1,
            },
            spatial_support: EstimateSupport::CoreMantleBoundaryPoint {
                latitude_deg: 0.0,
                longitude_deg: 180.0,
                radius_m: 3_480_000.0,
            },
            method: EstimateMethod::CoreSurfaceFlowInversion,
            model_id: "flow-inversion".into(),
            model_version: "0.1.0".into(),
            model_config_sha256: None,
            input_observation_ids: vec![],
            evidence: vec![],
            value: EstimateValue::CoreSurfaceFlowMetersPerYear {
                north_m_per_year: 0.0,
                east_m_per_year: 10.0,
                north_interval: None,
                east_interval: None,
            },
            identifiability: IdentifiabilityStatus::NonIdentifiable,
            unresolved_ambiguities: vec![],
            assumptions: vec!["frozen-flux approximation".into()],
        };
        assert_eq!(estimate.validate(), Err(ValidationError::MissingEvidence));
    }

    #[test]
    fn non_identifiable_estimate_must_name_an_ambiguity() {
        let mut estimate = StateEstimate {
            schema_version: EARTH_INTERIOR_SCHEMA_VERSION,
            id: "estimate-2".into(),
            valid_time: TimeWindow {
                start_unix_ns: 1,
                end_unix_ns: 1,
            },
            spatial_support: EstimateSupport::CoreMantleBoundaryPoint {
                latitude_deg: 0.0,
                longitude_deg: 180.0,
                radius_m: 3_480_000.0,
            },
            method: EstimateMethod::CoreSurfaceFlowInversion,
            model_id: "flow-inversion".into(),
            model_version: "0.1.0".into(),
            model_config_sha256: None,
            input_observation_ids: vec!["obs-0001".into()],
            evidence: vec![],
            value: EstimateValue::CoreSurfaceFlowMetersPerYear {
                north_m_per_year: 0.0,
                east_m_per_year: 10.0,
                north_interval: None,
                east_interval: None,
            },
            identifiability: IdentifiabilityStatus::NonIdentifiable,
            unresolved_ambiguities: vec![],
            assumptions: vec!["frozen-flux approximation".into()],
        };
        assert_eq!(
            estimate.validate(),
            Err(ValidationError::MissingIdentifiabilityCaveat)
        );
        estimate.unresolved_ambiguities = vec![
            "Distinct toroidal flow components can fit the same secular variation.".into(),
        ];
        assert_eq!(estimate.validate(), Ok(()));

        // A core-boundary flow must never masquerade as a global scalar or
        // a surface measurement in a renderer or downstream analysis.
        estimate.spatial_support = EstimateSupport::GlobalScalar;
        assert_eq!(
            estimate.validate(),
            Err(ValidationError::InvalidSpatialSupport)
        );
    }

    #[test]
    fn uncertainty_interval_requires_ordered_bounds_and_valid_coverage() {
        let interval = UncertaintyInterval {
            lower: 2.0,
            upper: 1.0,
            interpretation: UncertaintyInterpretation::CredibleInterval,
            nominal_coverage: Some(0.95),
        };
        assert_eq!(interval.validate(), Err(ValidationError::InvalidInterval));
        let interval = UncertaintyInterval {
            lower: 1.0,
            upper: 2.0,
            interpretation: UncertaintyInterpretation::CredibleInterval,
            nominal_coverage: Some(1.5),
        };
        assert_eq!(interval.validate(), Err(ValidationError::InvalidCoverage));
    }
}
