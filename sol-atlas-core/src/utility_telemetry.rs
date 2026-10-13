// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Read-only, provenance-bearing observations for civic utility digital twins.
//!
//! This module models observations, not control commands. It separates what
//! produced a measurement from how the measurement arrived so satellite imagery
//! is not confused with satellite communications backhaul.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Utility domain represented by an observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityDomain {
    Electricity,
    Water,
    Wastewater,
    Stormwater,
    MultiUtility,
}

/// The thing to which the observation refers. A satellite cell is not an asset,
/// and a reservoir/catchment is not an individual sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Asset,
    PressureZone,
    Feeder,
    Reservoir,
    Catchment,
    GridCell,
    MunicipalArea,
}

/// Canonical metric names supported by the initial observation contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    WaterFlowRate,
    WaterPressure,
    ReservoirLevel,
    WaterPh,
    WaterTurbidity,
    FreeChlorineResidual,
    ElectricalPower,
    Voltage,
    Current,
    GridFrequency,
    TransformerTemperature,
    PumpOperatingState,
    FeederOutage,
    SurfaceWaterExtent,
    InundationExtent,
    VegetationEncroachment,
    ThermalAnomaly,
    Precipitation,
    SoilMoisture,
    AssetCondition,
    TelemetryLinkQuality,
}

/// Units are explicit to prevent silent comparison of unlike quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementUnit {
    LitresPerSecond,
    CubicMetresPerSecond,
    Kilopascal,
    Bar,
    Metre,
    Percent,
    Kilowatt,
    Megawatt,
    Volt,
    Kilovolt,
    Ampere,
    Hertz,
    DegreeCelsius,
    SquareMetre,
    SquareKilometre,
    Millimetre,
    Fraction,
    Ph,
    NephelometricTurbidityUnit,
    MilligramPerLitre,
}

/// The payload may be numeric, boolean, or categorical. Boolean and category
/// values avoid encoding states such as "pump fault" as arbitrary numbers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObservationValue {
    Numeric {
        value: f64,
        unit: MeasurementUnit,
    },
    Boolean {
        value: bool,
    },
    Category {
        code: String,
    },
}

impl MetricKind {
    /// Rejects metrics attached to unrelated utility domains.
    ///
    /// MultiUtility is explicit: it must not be inferred simply because a
    /// source happens to be shared by multiple operators.
    pub fn accepts_domain(&self, domain: UtilityDomain) -> bool {
        use MetricKind as M;
        use UtilityDomain as D;

        match self {
            M::WaterFlowRate
            | M::WaterPressure
            | M::ReservoirLevel
            | M::WaterPh
            | M::WaterTurbidity
            | M::FreeChlorineResidual
            | M::PumpOperatingState => {
                matches!(domain, D::Water | D::Wastewater | D::Stormwater | D::MultiUtility)
            }
            M::ElectricalPower
            | M::Voltage
            | M::Current
            | M::GridFrequency
            | M::TransformerTemperature
            | M::FeederOutage => matches!(domain, D::Electricity | D::MultiUtility),
            M::SurfaceWaterExtent
            | M::InundationExtent
            | M::Precipitation
            | M::SoilMoisture => {
                matches!(domain, D::Water | D::Stormwater | D::MultiUtility)
            }
            M::VegetationEncroachment => matches!(
                domain,
                D::Electricity | D::Water | D::Wastewater | D::Stormwater | D::MultiUtility
            ),
            M::ThermalAnomaly | M::AssetCondition | M::TelemetryLinkQuality => true,
        }
    }

    /// Returns whether a value has the unit/shape expected for this metric.
    pub fn accepts(&self, value: &ObservationValue) -> bool {
        use MeasurementUnit as U;
        use ObservationValue as V;

        match self {
            Self::WaterFlowRate => matches!(
                value,
                V::Numeric {
                    unit: U::LitresPerSecond | U::CubicMetresPerSecond,
                    ..
                }
            ),
            Self::WaterPressure => matches!(value, V::Numeric { unit: U::Kilopascal | U::Bar, .. }),
            Self::ReservoirLevel => matches!(value, V::Numeric { unit: U::Metre | U::Percent, .. }),
            Self::WaterPh => matches!(value, V::Numeric { unit: U::Ph, .. }),
            Self::WaterTurbidity => matches!(
                value,
                V::Numeric {
                    unit: U::NephelometricTurbidityUnit,
                    ..
                }
            ),
            Self::FreeChlorineResidual => matches!(
                value,
                V::Numeric {
                    unit: U::MilligramPerLitre,
                    ..
                }
            ),
            Self::ElectricalPower => matches!(
                value,
                V::Numeric {
                    unit: U::Kilowatt | U::Megawatt,
                    ..
                }
            ),
            Self::Voltage => matches!(value, V::Numeric { unit: U::Volt | U::Kilovolt, .. }),
            Self::Current => matches!(value, V::Numeric { unit: U::Ampere, .. }),
            Self::GridFrequency => matches!(value, V::Numeric { unit: U::Hertz, .. }),
            Self::TransformerTemperature | Self::ThermalAnomaly => {
                matches!(value, V::Numeric { unit: U::DegreeCelsius, .. })
            }
            Self::PumpOperatingState | Self::AssetCondition => {
                matches!(value, V::Category { code } if !code.trim().is_empty())
            }
            Self::FeederOutage => matches!(value, V::Boolean { .. }),
            Self::SurfaceWaterExtent | Self::InundationExtent => {
                matches!(value, V::Numeric { unit: U::SquareMetre | U::SquareKilometre, .. })
            }
            Self::VegetationEncroachment => matches!(value, V::Numeric { unit: U::Metre, .. }),
            Self::Precipitation => matches!(value, V::Numeric { unit: U::Millimetre, .. }),
            Self::SoilMoisture => matches!(
                value,
                V::Numeric {
                    unit: U::Fraction | U::Percent,
                    ..
                }
            ),
            Self::TelemetryLinkQuality => matches!(value, V::Numeric { unit: U::Percent, .. }),
        }
    }
}

/// Observation method, independent of the transport used to deliver it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationSource {
    ScadaReadOnlyExport,
    GroundSensor,
    SmartMeter,
    SatelliteOptical,
    SatelliteRadar,
    SatelliteThermal,
    SatelliteMeteorology,
    ManualInspection,
    DerivedEstimate,
    SyntheticSimulation,
}

impl ObservationSource {
    fn is_satellite(self) -> bool {
        matches!(
            self,
            Self::SatelliteOptical
                | Self::SatelliteRadar
                | Self::SatelliteThermal
                | Self::SatelliteMeteorology
        )
    }

    /// Keep satellite-derived indicators separate from quantities that must
    /// come from an in-situ instrument or an explicitly identified model.
    fn supports_metric(self, metric: MetricKind) -> bool {
        use MetricKind as M;
        match self {
            Self::SatelliteOptical => matches!(
                metric,
                M::SurfaceWaterExtent | M::VegetationEncroachment
            ),
            Self::SatelliteRadar => matches!(
                metric,
                M::SurfaceWaterExtent | M::InundationExtent | M::VegetationEncroachment
            ),
            Self::SatelliteThermal => matches!(metric, M::ThermalAnomaly),
            Self::SatelliteMeteorology => matches!(metric, M::Precipitation | M::SoilMoisture),
            _ => true,
        }
    }
}

/// How data reached the ingestion boundary. Satellite backhaul does not imply
/// that the measurement itself came from a satellite sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataTransport {
    LocalNetwork,
    Cellular,
    SatelliteBackhaul,
    StoreAndForward,
    BatchImport,
}

/// Data-quality flags are retained as evidence rather than erased by one
/// opaque confidence score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityFlag {
    CalibrationExpired,
    ClockSkewSuspected,
    OutlierFlagged,
    CloudContaminated,
    LowSpatialResolution,
    AssetAssociationUnverified,
    ModelDerived,
    DuplicateSuspected,
    MissingBaseline,
    OperatorReviewed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationQuality {
    /// Optional producer-supplied confidence, normalized to [0, 1].
    pub confidence: Option<f64>,
    /// Optional non-negative uncertainty expressed in the measurement's unit.
    pub uncertainty: Option<f64>,
    pub flags: Vec<QualityFlag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationProvenance {
    /// Stable sensor ID, dataset/product ID, or scenario ID.
    pub source_id: String,
    /// Device firmware, dataset collection, processing chain, or model version.
    pub source_version: Option<String>,
    /// SHA-256 digest of a captured evidence artifact, when available.
    /// Syntax is checked here; the referenced artifact is not fetched or hashed.
    pub evidence_sha256: Option<String>,
    /// IDs of upstream observations used to derive this result.
    pub upstream_observation_ids: Vec<String>,
    /// Structured algorithm parameters for a derived result (e.g. index,
    /// threshold, grid/resampling policy). Never infer these from a version string.
    pub processing_parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
}

/// A read-only observation suitable for replay, visualization, and analysis.
///
/// observed_at_unix_ms is the source observation/acquisition time;
/// received_at_unix_ms is when the ingestion boundary received it. Keep both:
/// satellite products and store-and-forward sensors may be delayed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtilityObservation {
    pub schema_version: u16,
    pub observation_id: String,
    pub subject_id: String,
    pub subject_kind: SubjectKind,
    pub domain: UtilityDomain,
    pub metric: MetricKind,
    pub value: ObservationValue,
    pub observed_at_unix_ms: u64,
    pub received_at_unix_ms: u64,
    pub source: ObservationSource,
    pub transport: DataTransport,
    pub provenance: ObservationProvenance,
    pub quality: ObservationQuality,
    pub location: Option<Coordinates>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    /// Available and inside the caller's source/metric-specific age policy.
    Fresh,
    /// Available, but older than the caller's source/metric-specific budget.
    Stale,
    /// Source timestamp is later than the evaluation time.
    FutureObserved,
    /// Receipt timestamp is later than the evaluation time.
    NotYetReceived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationValidationError {
    UnsupportedSchemaVersion,
    EmptyObservationId,
    EmptySubjectId,
    EmptySourceId,
    InvalidTimestampOrder,
    NonFiniteMeasurement,
    IncompatibleMeasurementShapeOrUnit,
    MetricDomainMismatch,
    EmptyCategoryCode,
    InvalidCoordinate,
    InvalidConfidence,
    InvalidUncertainty,
    InvalidEvidenceDigest,
    MissingSatelliteProductVersion,
    SatelliteMetricMismatch,
    MissingDerivationLineage,
    EmptyUpstreamObservationId,
    DuplicateUpstreamObservationId,
    MissingModelDerivedFlag,
}

impl UtilityObservation {
    /// Structural and semantic checks only. This does not authenticate a sensor,
    /// validate a satellite product, prove a reading is true, or authorize action.
    pub fn validate(&self) -> Result<(), Vec<ObservationValidationError>> {
        use ObservationSource as S;
        use ObservationValidationError as E;
        use ObservationValue as V;

        let mut errors = Vec::new();

        if self.schema_version != 1 {
            errors.push(E::UnsupportedSchemaVersion);
        }
        if self.observation_id.trim().is_empty() {
            errors.push(E::EmptyObservationId);
        }
        if self.subject_id.trim().is_empty() {
            errors.push(E::EmptySubjectId);
        }
        if self.provenance.source_id.trim().is_empty() {
            errors.push(E::EmptySourceId);
        }
        if self.received_at_unix_ms < self.observed_at_unix_ms {
            errors.push(E::InvalidTimestampOrder);
        }

        match &self.value {
            V::Numeric { value, .. } if !value.is_finite() => errors.push(E::NonFiniteMeasurement),
            V::Category { code } if code.trim().is_empty() => errors.push(E::EmptyCategoryCode),
            _ => {}
        }
        if !self.metric.accepts(&self.value) {
            errors.push(E::IncompatibleMeasurementShapeOrUnit);
        }
        if !self.metric.accepts_domain(self.domain) {
            errors.push(E::MetricDomainMismatch);
        }

        if let Some(location) = self.location {
            if !location.latitude_deg.is_finite()
                || !location.longitude_deg.is_finite()
                || !(-90.0..=90.0).contains(&location.latitude_deg)
                || !(-180.0..=180.0).contains(&location.longitude_deg)
            {
                errors.push(E::InvalidCoordinate);
            }
        }

        if let Some(confidence) = self.quality.confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                errors.push(E::InvalidConfidence);
            }
        }
        if let Some(uncertainty) = self.quality.uncertainty {
            if !uncertainty.is_finite() || uncertainty < 0.0 {
                errors.push(E::InvalidUncertainty);
            }
        }

        if let Some(digest) = self.provenance.evidence_sha256.as_deref() {
            if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                errors.push(E::InvalidEvidenceDigest);
            }
        }

        if self.source.is_satellite() {
            if self
                .provenance
                .source_version
                .as_deref()
                .is_none_or(|version| version.trim().is_empty())
            {
                errors.push(E::MissingSatelliteProductVersion);
            }
            if !self.source.supports_metric(self.metric) {
                errors.push(E::SatelliteMetricMismatch);
            }
        }

        if self.source == S::DerivedEstimate {
            if self.provenance.upstream_observation_ids.is_empty() {
                errors.push(E::MissingDerivationLineage);
            }

            let mut upstream_ids = BTreeSet::new();
            for upstream_id in &self.provenance.upstream_observation_ids {
                let normalized_id = upstream_id.trim();
                if normalized_id.is_empty() {
                    errors.push(E::EmptyUpstreamObservationId);
                } else if !upstream_ids.insert(normalized_id) {
                    errors.push(E::DuplicateUpstreamObservationId);
                }
            }
        }
        if matches!(self.source, S::DerivedEstimate | S::SyntheticSimulation)
            && !self.quality.flags.contains(&QualityFlag::ModelDerived)
        {
            errors.push(E::MissingModelDerivedFlag);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Classify age without imposing a universal threshold. Pressure telemetry,
    /// satellite imagery, and annual asset surveys need different age budgets.
    pub fn freshness_at(&self, now_unix_ms: u64, max_age_ms: u64) -> Freshness {
        if self.received_at_unix_ms > now_unix_ms {
            Freshness::NotYetReceived
        } else if self.observed_at_unix_ms > now_unix_ms {
            Freshness::FutureObserved
        } else if now_unix_ms - self.observed_at_unix_ms > max_age_ms {
            Freshness::Stale
        } else {
            Freshness::Fresh
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_pressure_observation() -> UtilityObservation {
        UtilityObservation {
            schema_version: 1,
            observation_id: "obs-0001".into(),
            subject_id: "pressure-zone-test".into(),
            subject_kind: SubjectKind::PressureZone,
            domain: UtilityDomain::Water,
            metric: MetricKind::WaterPressure,
            value: ObservationValue::Numeric { value: 420.0, unit: MeasurementUnit::Kilopascal },
            observed_at_unix_ms: 1_000,
            received_at_unix_ms: 1_100,
            source: ObservationSource::ScadaReadOnlyExport,
            transport: DataTransport::LocalNetwork,
            provenance: ObservationProvenance {
                source_id: "scada-test-export".into(),
                source_version: Some("fixture-v1".into()),
                evidence_sha256: None,
                upstream_observation_ids: Vec::new(),
                processing_parameters: None,
            },
            quality: ObservationQuality {
                confidence: Some(0.98),
                uncertainty: Some(2.0),
                flags: Vec::new(),
            },
            location: Some(Coordinates { latitude_deg: -26.2, longitude_deg: 28.0 }),
        }
    }

    fn has_error(observation: &UtilityObservation, expected: ObservationValidationError) -> bool {
        observation.validate().err().is_some_and(|errors| errors.contains(&expected))
    }

    #[test]
    fn accepts_well_formed_scada_observation() {
        assert!(valid_pressure_observation().validate().is_ok());
    }

    #[test]
    fn rejects_non_finite_measurements() {
        let mut observation = valid_pressure_observation();
        observation.value = ObservationValue::Numeric {
            value: f64::NAN,
            unit: MeasurementUnit::Kilopascal,
        };
        assert!(has_error(&observation, ObservationValidationError::NonFiniteMeasurement));
    }

    #[test]
    fn rejects_metric_unit_mismatch() {
        let mut observation = valid_pressure_observation();
        observation.value = ObservationValue::Numeric { value: 420.0, unit: MeasurementUnit::Volt };
        assert!(has_error(
            &observation,
            ObservationValidationError::IncompatibleMeasurementShapeOrUnit
        ));
    }

    #[test]
    fn rejects_electrical_voltage_labelled_as_water_domain() {
        let mut observation = valid_pressure_observation();
        observation.metric = MetricKind::Voltage;
        observation.domain = UtilityDomain::Water;
        observation.value = ObservationValue::Numeric {
            value: 230.0,
            unit: MeasurementUnit::Volt,
        };
        assert!(has_error(&observation, ObservationValidationError::MetricDomainMismatch));
    }

    #[test]
    fn explicit_multi_utility_domain_allows_cross_utility_voltage() {
        let mut observation = valid_pressure_observation();
        observation.metric = MetricKind::Voltage;
        observation.domain = UtilityDomain::MultiUtility;
        observation.value = ObservationValue::Numeric {
            value: 230.0,
            unit: MeasurementUnit::Volt,
        };
        assert!(observation.validate().is_ok());
    }

    #[test]
    fn rejects_receive_time_before_observation_time() {
        let mut observation = valid_pressure_observation();
        observation.received_at_unix_ms = 999;
        assert!(has_error(&observation, ObservationValidationError::InvalidTimestampOrder));
    }

    #[test]
    fn rejects_out_of_range_coordinates() {
        let mut observation = valid_pressure_observation();
        observation.location = Some(Coordinates { latitude_deg: -91.0, longitude_deg: 28.0 });
        assert!(has_error(&observation, ObservationValidationError::InvalidCoordinate));
    }

    #[test]
    fn rejects_confidence_outside_unit_interval() {
        let mut observation = valid_pressure_observation();
        observation.quality.confidence = Some(1.01);
        assert!(has_error(&observation, ObservationValidationError::InvalidConfidence));
    }

    #[test]
    fn satellite_observation_cannot_claim_direct_pipe_pressure() {
        let mut observation = valid_pressure_observation();
        observation.source = ObservationSource::SatelliteRadar;
        observation.transport = DataTransport::BatchImport;
        observation.provenance.source_version = Some("sentinel-1-product-id".into());
        assert!(has_error(&observation, ObservationValidationError::SatelliteMetricMismatch));
    }

    #[test]
    fn satellite_radar_water_extent_requires_product_provenance() {
        let mut observation = valid_pressure_observation();
        observation.subject_kind = SubjectKind::Reservoir;
        observation.metric = MetricKind::SurfaceWaterExtent;
        observation.value = ObservationValue::Numeric {
            value: 1_250_000.0,
            unit: MeasurementUnit::SquareMetre,
        };
        observation.source = ObservationSource::SatelliteRadar;
        observation.transport = DataTransport::BatchImport;
        observation.provenance.source_id = "sentinel-1-acquisition".into();
        observation.provenance.source_version = Some("processing-baseline-v1".into());
        observation.quality.flags = vec![QualityFlag::LowSpatialResolution];
        assert!(observation.validate().is_ok());
    }

    #[test]
    fn satellite_observations_require_a_versioned_product() {
        let mut observation = valid_pressure_observation();
        observation.metric = MetricKind::SurfaceWaterExtent;
        observation.value = ObservationValue::Numeric {
            value: 4.2,
            unit: MeasurementUnit::SquareKilometre,
        };
        observation.source = ObservationSource::SatelliteOptical;
        observation.provenance.source_version = None;
        assert!(has_error(
            &observation,
            ObservationValidationError::MissingSatelliteProductVersion
        ));
    }

    #[test]
    fn derived_estimate_requires_lineage_and_model_flag() {
        let mut observation = valid_pressure_observation();
        observation.source = ObservationSource::DerivedEstimate;
        assert!(has_error(&observation, ObservationValidationError::MissingDerivationLineage));
        assert!(has_error(&observation, ObservationValidationError::MissingModelDerivedFlag));
    }

    #[test]
    fn derived_estimate_rejects_blank_upstream_ids() {
        let mut observation = valid_pressure_observation();
        observation.source = ObservationSource::DerivedEstimate;
        observation.provenance.upstream_observation_ids = vec!["upstream-1".into(), "   ".into()];
        observation.quality.flags.push(QualityFlag::ModelDerived);

        assert!(has_error(
            &observation,
            ObservationValidationError::EmptyUpstreamObservationId
        ));
        assert!(!has_error(
            &observation,
            ObservationValidationError::MissingDerivationLineage
        ));
    }

    #[test]
    fn derived_estimate_rejects_duplicate_upstream_ids() {
        let mut observation = valid_pressure_observation();
        observation.source = ObservationSource::DerivedEstimate;
        observation.provenance.upstream_observation_ids =
            vec![" upstream-1 ".into(), "upstream-1".into()];
        observation.quality.flags.push(QualityFlag::ModelDerived);

        assert!(has_error(
            &observation,
            ObservationValidationError::DuplicateUpstreamObservationId
        ));
    }

    #[test]
    fn freshness_is_policy_specific_and_does_not_rewrite_observation_time() {
        let observation = valid_pressure_observation();
        assert_eq!(observation.freshness_at(1_200, 500), Freshness::Fresh);
        assert_eq!(observation.freshness_at(2_000, 500), Freshness::Stale);
        assert_eq!(observation.freshness_at(1_050, 500), Freshness::NotYetReceived);
        assert_eq!(observation.observed_at_unix_ms, 1_000);
    }

    #[test]
    fn rejects_malformed_sha256_digest() {
        let mut observation = valid_pressure_observation();
        observation.provenance.evidence_sha256 = Some("not-a-hash".into());
        assert!(has_error(&observation, ObservationValidationError::InvalidEvidenceDigest));
    }
}
