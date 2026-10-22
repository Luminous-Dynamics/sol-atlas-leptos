// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned evidence contracts for geographic access measurements.
//!
//! This module deliberately does not route over roads, tracks, footpaths, or
//! friction surfaces. It prevents geodesic distance, modeled travel, and
//! observed trips from being silently treated as interchangeable evidence.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Current schema version for a geographic access evidence envelope.
pub const GEO_ACCESS_SCHEMA_VERSION: u16 = 1;

/// What a numeric access result actually measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessMetric {
    /// Great-circle distance computed from two coordinate pairs.
    GeodesicDistance,
    /// Distance computed by a versioned route graph or friction surface.
    ModeledRouteDistance,
    /// Travel time estimated by a versioned route model.
    ModeledTravelTime,
    /// Distance measured from a recorded trip.
    ObservedTripDistance,
    /// Duration measured from a recorded trip.
    ObservedTripTime,
}

/// A disposition, never encoded as a numeric sentinel such as zero or infinity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessState {
    Available,
    NoRoute,
    OutsideCoverage,
    Unknown,
    Stale,
    Conflicting,
}

/// Mode must describe the metric's meaning; unknown mode is explicit, not guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TravelMode {
    StraightLine,
    Walking,
    Bicycle,
    Motorcycle,
    MotorVehicle,
    AnimalDrawn,
    Multimodal,
    Unknown,
}

/// Unit for the result value and, when present, its absolute uncertainty bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricUnit {
    Meters,
    Kilometers,
    Seconds,
    Minutes,
}

/// Declared extent of route-data validation/coverage; not a claim that a digest
/// authenticates the underlying dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageState {
    NotApplicable,
    Validated,
    PartiallyValidated,
    Unvalidated,
    OutsideCoverage,
    Unknown,
}

/// Uncertainty is explicit even when the source has not quantified it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyKind {
    NotAssessed,
    QualitativeLow,
    QualitativeModerate,
    QualitativeHigh,
    QuantifiedAbsoluteBound,
}

/// An optional absolute bound is expressed in the same unit as the result.
/// Its confidence interpretation must be documented by the versioned method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UncertaintyDeclaration {
    pub kind: UncertaintyKind,
    pub absolute_bound: Option<f64>,
}

/// Versioned, privacy-conscious access evidence.
///
/// Origins and destinations are opaque references, not coordinates. Precise
/// farm coordinates should remain in the controlled spatial source. All
/// timestamps are Unix seconds. Evidence references identify source records;
/// a reference or digest shape does not independently authenticate them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoAccessEnvelope {
    pub schema_version: u16,
    pub result_ref: String,
    pub origin_ref: String,
    pub destination_ref: String,
    pub scope_ref: String,
    pub period_start_unix_seconds: i64,
    pub period_end_unix_seconds: i64,
    pub as_of_unix_seconds: i64,
    pub metric: AccessMetric,
    pub state: AccessState,
    pub travel_mode: TravelMode,
    pub unit: MetricUnit,
    pub value: Option<f64>,
    pub method_id: String,
    pub method_version: String,
    /// Required for modeled routes/times; absent for geodesic calculations.
    pub network_version: Option<String>,
    /// Site- or task-specific season label; use an explicit value such as
    /// "unknown" rather than omitting seasonal applicability.
    pub season_label: String,
    pub coverage: CoverageState,
    pub uncertainty: UncertaintyDeclaration,
    /// Opaque source references, sorted lexicographically and unique.
    pub source_evidence_refs: Vec<String>,
    /// Opaque barrier references, sorted lexicographically and unique.
    pub barrier_refs: Vec<String>,
}

/// A structural validation defect. This validates the envelope, not the truth
/// or authenticity of referenced map, trip, or model artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssue {
    UnsupportedSchemaVersion,
    InvalidCoordinate,
    EmptyRequiredField(&'static str),
    InvalidPeriod,
    InvalidTimestamp,
    InvalidMetricUnit,
    InvalidTravelMode,
    InvalidValueForState,
    NegativeOrNonFiniteValue,
    MissingNetworkVersion,
    UnexpectedNetworkVersionForGeodesic,
    MissingSourceEvidence,
    InvalidReferences(&'static str),
    InvalidCoverageState,
    MissingUncertaintyBound,
    UnexpectedUncertaintyBound,
    NegativeOrNonFiniteUncertainty,
    EmptyValidationSampleSet,
    InvalidValidationSample,
    DuplicateSampleReference,
    InconsistentValidationGroup,
    NonFiniteSummary,
}

/// Validation design is explicit so an operational sample or synthetic oracle
/// cannot be silently presented as spatial/seasonal generalization evidence.
/// Random-row holdout is intentionally not a supported generalization label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationDesign {
    SyntheticOracle,
    DescriptiveOperational,
    SpatialHoldout,
    TemporalHoldout,
    SpatialAndTemporalHoldout,
}

/// Paired travel-time observations for validating a specific model, mode, season,
/// scope, and declared evaluation partition. Samples should be measured
/// independently of predictions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TravelTimeValidationSample {
    pub sample_ref: String,
    pub scope_ref: String,
    pub method_id: String,
    pub method_version: String,
    pub travel_mode: TravelMode,
    pub season_label: String,
    pub validation_design: ValidationDesign,
    pub partition_ref: String,
    pub predicted_minutes: f64,
    pub observed_minutes: f64,
}

/// Error summary for one model-version/mode/season/scope group.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TravelTimeErrorSummary {
    pub scope_ref: String,
    pub method_id: String,
    pub method_version: String,
    pub travel_mode: TravelMode,
    pub season_label: String,
    pub validation_design: ValidationDesign,
    pub partition_ref: String,
    pub sample_count: usize,
    pub median_absolute_error_minutes: f64,
    /// Nearest-rank 90th percentile: sorted[ceil(0.9 * n) - 1].
    pub p90_absolute_error_minutes: f64,
    /// Mean signed error, predicted minus observed; positive means overprediction.
    pub mean_signed_error_minutes: f64,
}

impl GeoAccessEnvelope {
    /// Validate structural consistency and fail closed on ambiguous states.
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        if self.schema_version != GEO_ACCESS_SCHEMA_VERSION {
            issues.push(ValidationIssue::UnsupportedSchemaVersion);
        }

        for (field, value) in [
            ("result_ref", self.result_ref.as_str()),
            ("origin_ref", self.origin_ref.as_str()),
            ("destination_ref", self.destination_ref.as_str()),
            ("scope_ref", self.scope_ref.as_str()),
            ("method_id", self.method_id.as_str()),
            ("method_version", self.method_version.as_str()),
            ("season_label", self.season_label.as_str()),
        ] {
            if value.trim().is_empty() {
                issues.push(ValidationIssue::EmptyRequiredField(field));
            }
        }

        if self.period_start_unix_seconds < 0
            || self.period_end_unix_seconds <= self.period_start_unix_seconds
        {
            issues.push(ValidationIssue::InvalidPeriod);
        }
        if self.as_of_unix_seconds < 0 {
            issues.push(ValidationIssue::InvalidTimestamp);
        }

        validate_references(
            &self.source_evidence_refs,
            "source_evidence_refs",
            &mut issues,
        );
        validate_references(&self.barrier_refs, "barrier_refs", &mut issues);

        let is_distance = matches!(
            self.metric,
            AccessMetric::GeodesicDistance
                | AccessMetric::ModeledRouteDistance
                | AccessMetric::ObservedTripDistance
        );
        let unit_is_compatible = if is_distance {
            matches!(self.unit, MetricUnit::Meters | MetricUnit::Kilometers)
        } else {
            matches!(self.unit, MetricUnit::Seconds | MetricUnit::Minutes)
        };
        if !unit_is_compatible {
            issues.push(ValidationIssue::InvalidMetricUnit);
        }

        match self.metric {
            AccessMetric::GeodesicDistance => {
                if self.travel_mode != TravelMode::StraightLine {
                    issues.push(ValidationIssue::InvalidTravelMode);
                }
                if self.network_version.is_some() {
                    issues.push(ValidationIssue::UnexpectedNetworkVersionForGeodesic);
                }
                if self.coverage != CoverageState::NotApplicable {
                    issues.push(ValidationIssue::InvalidCoverageState);
                }
            }
            AccessMetric::ModeledRouteDistance | AccessMetric::ModeledTravelTime => {
                if self.travel_mode == TravelMode::StraightLine
                    || (self.state == AccessState::Available
                        && self.travel_mode == TravelMode::Unknown)
                {
                    issues.push(ValidationIssue::InvalidTravelMode);
                }
                if self
                    .network_version
                    .as_deref()
                    .is_none_or(|version| version.trim().is_empty())
                {
                    issues.push(ValidationIssue::MissingNetworkVersion);
                }
                if self.coverage == CoverageState::NotApplicable {
                    issues.push(ValidationIssue::InvalidCoverageState);
                }
                if self.state == AccessState::Available
                    && matches!(
                        self.coverage,
                        CoverageState::OutsideCoverage | CoverageState::Unknown
                    )
                {
                    issues.push(ValidationIssue::InvalidCoverageState);
                }
                // A no-route claim requires route data with enough declared
                // coverage to distinguish an absent path from a missing map.
                if self.state == AccessState::NoRoute
                    && !matches!(
                        self.coverage,
                        CoverageState::Validated | CoverageState::PartiallyValidated
                    )
                {
                    issues.push(ValidationIssue::InvalidCoverageState);
                }
            }
            AccessMetric::ObservedTripDistance | AccessMetric::ObservedTripTime => {
                if self.travel_mode == TravelMode::StraightLine
                    || (self.state == AccessState::Available
                        && self.travel_mode == TravelMode::Unknown)
                {
                    issues.push(ValidationIssue::InvalidTravelMode);
                }
                if self.coverage != CoverageState::NotApplicable {
                    issues.push(ValidationIssue::InvalidCoverageState);
                }
            }
        }

        if (self.state == AccessState::OutsideCoverage)
            != (self.coverage == CoverageState::OutsideCoverage)
        {
            issues.push(ValidationIssue::InvalidCoverageState);
        }

        if self.state != AccessState::Unknown && self.source_evidence_refs.is_empty() {
            // No-route, out-of-coverage, stale and conflicting dispositions
            // are still claims about evidence, not permission to omit it.
            issues.push(ValidationIssue::MissingSourceEvidence);
        }

        match (self.state, self.value) {
            (AccessState::Available, Some(value)) => {
                if !value.is_finite() || value < 0.0 {
                    issues.push(ValidationIssue::NegativeOrNonFiniteValue);
                }
            }
            (AccessState::Available, None) => {
                issues.push(ValidationIssue::InvalidValueForState);
            }
            (_, Some(_)) => issues.push(ValidationIssue::InvalidValueForState),
            (_, None) => {}
        }

        match (self.uncertainty.kind, self.uncertainty.absolute_bound) {
            (UncertaintyKind::QuantifiedAbsoluteBound, Some(bound)) => {
                if !bound.is_finite() || bound < 0.0 {
                    issues.push(ValidationIssue::NegativeOrNonFiniteUncertainty);
                }
            }
            (UncertaintyKind::QuantifiedAbsoluteBound, None) => {
                issues.push(ValidationIssue::MissingUncertaintyBound);
            }
            (_, Some(_)) => issues.push(ValidationIssue::UnexpectedUncertaintyBound),
            (_, None) => {}
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

fn validate_references(
    references: &[String],
    field: &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    if references
        .iter()
        .any(|reference| reference.trim().is_empty())
    {
        issues.push(ValidationIssue::InvalidReferences(field));
    }
    if references.windows(2).any(|pair| pair[0] >= pair[1]) {
        issues.push(ValidationIssue::InvalidReferences(field));
    }
}

/// Compute the canonical Sol Atlas great-circle distance after validating
/// coordinates. This delegates to the existing haversine implementation; it
/// does not turn the result into a road/track distance or travel-time estimate.
pub fn checked_geodesic_distance_km(
    origin_latitude: f64,
    origin_longitude: f64,
    destination_latitude: f64,
    destination_longitude: f64,
) -> Result<f64, ValidationIssue> {
    for (latitude, longitude) in [
        (origin_latitude, origin_longitude),
        (destination_latitude, destination_longitude),
    ] {
        if !latitude.is_finite()
            || !longitude.is_finite()
            || !(-90.0..=90.0).contains(&latitude)
            || !(-180.0..=180.0).contains(&longitude)
        {
            return Err(ValidationIssue::InvalidCoordinate);
        }
    }

    let distance = crate::geo::haversine_km(
        origin_latitude,
        origin_longitude,
        destination_latitude,
        destination_longitude,
    );
    if distance.is_finite() && distance >= 0.0 {
        Ok(distance)
    } else {
        Err(ValidationIssue::NegativeOrNonFiniteValue)
    }
}

/// Summarize route-time errors only when every pair belongs to the same
/// declared model version, scope, mode, and season. Invalid samples are errors,
/// never silently discarded. This function makes no claim about field accuracy
/// until callers supply independently observed trips.
pub fn summarize_travel_time_errors(
    samples: &[TravelTimeValidationSample],
) -> Result<TravelTimeErrorSummary, Vec<ValidationIssue>> {
    let Some(first) = samples.first() else {
        return Err(vec![ValidationIssue::EmptyValidationSampleSet]);
    };

    let mut issues = Vec::new();
    let mut seen_refs = BTreeSet::new();

    for sample in samples {
        for field in [
            ("sample_ref", sample.sample_ref.as_str()),
            ("scope_ref", sample.scope_ref.as_str()),
            ("method_id", sample.method_id.as_str()),
            ("method_version", sample.method_version.as_str()),
            ("season_label", sample.season_label.as_str()),
            ("partition_ref", sample.partition_ref.as_str()),
        ] {
            if field.1.trim().is_empty() {
                issues.push(ValidationIssue::EmptyRequiredField(field.0));
            }
        }

        if sample.travel_mode == TravelMode::StraightLine
            || sample.travel_mode == TravelMode::Unknown
            || !sample.predicted_minutes.is_finite()
            || sample.predicted_minutes < 0.0
            || !sample.observed_minutes.is_finite()
            || sample.observed_minutes < 0.0
        {
            issues.push(ValidationIssue::InvalidValidationSample);
        }

        if !seen_refs.insert(sample.sample_ref.as_str()) {
            issues.push(ValidationIssue::DuplicateSampleReference);
        }

        if sample.scope_ref != first.scope_ref
            || sample.method_id != first.method_id
            || sample.method_version != first.method_version
            || sample.travel_mode != first.travel_mode
            || sample.season_label != first.season_label
            || sample.validation_design != first.validation_design
            || sample.partition_ref != first.partition_ref
        {
            issues.push(ValidationIssue::InconsistentValidationGroup);
        }
    }

    if !issues.is_empty() {
        return Err(issues);
    }

    let mut absolute_errors: Vec<f64> = samples
        .iter()
        .map(|sample| (sample.predicted_minutes - sample.observed_minutes).abs())
        .collect();
    absolute_errors.sort_by(f64::total_cmp);

    let middle = absolute_errors.len() / 2;
    let median_absolute_error_minutes = if absolute_errors.len().is_multiple_of(2) {
        (absolute_errors[middle - 1] + absolute_errors[middle]) / 2.0
    } else {
        absolute_errors[middle]
    };
    let p90_rank = ((absolute_errors.len() as f64 * 0.90).ceil() as usize).max(1);
    let p90_absolute_error_minutes = absolute_errors[p90_rank - 1];
    let mean_signed_error_minutes = samples
        .iter()
        .map(|sample| sample.predicted_minutes - sample.observed_minutes)
        .sum::<f64>()
        / samples.len() as f64;

    if !median_absolute_error_minutes.is_finite()
        || !p90_absolute_error_minutes.is_finite()
        || !mean_signed_error_minutes.is_finite()
    {
        return Err(vec![ValidationIssue::NonFiniteSummary]);
    }

    Ok(TravelTimeErrorSummary {
        scope_ref: first.scope_ref.clone(),
        method_id: first.method_id.clone(),
        method_version: first.method_version.clone(),
        travel_mode: first.travel_mode,
        season_label: first.season_label.clone(),
        validation_design: first.validation_design,
        partition_ref: first.partition_ref.clone(),
        sample_count: samples.len(),
        median_absolute_error_minutes,
        p90_absolute_error_minutes,
        mean_signed_error_minutes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geodesic_envelope() -> GeoAccessEnvelope {
        GeoAccessEnvelope {
            schema_version: GEO_ACCESS_SCHEMA_VERSION,
            result_ref: "result:synthetic-001".into(),
            origin_ref: "site:synthetic-origin".into(),
            destination_ref: "site:synthetic-destination".into(),
            scope_ref: "scope:fixture-v1".into(),
            period_start_unix_seconds: 1_800_000_000,
            period_end_unix_seconds: 1_802_592_000,
            as_of_unix_seconds: 1_799_000_000,
            metric: AccessMetric::GeodesicDistance,
            state: AccessState::Available,
            travel_mode: TravelMode::StraightLine,
            unit: MetricUnit::Kilometers,
            value: Some(343.0),
            method_id: "sol_atlas_haversine".into(),
            method_version: "1".into(),
            network_version: None,
            season_label: "year_round".into(),
            coverage: CoverageState::NotApplicable,
            uncertainty: UncertaintyDeclaration {
                kind: UncertaintyKind::NotAssessed,
                absolute_bound: None,
            },
            source_evidence_refs: vec!["fixture:coordinate-pair-v1".into()],
            barrier_refs: vec![],
        }
    }

    fn modeled_time_envelope() -> GeoAccessEnvelope {
        let mut envelope = geodesic_envelope();
        envelope.metric = AccessMetric::ModeledTravelTime;
        envelope.travel_mode = TravelMode::Walking;
        envelope.unit = MetricUnit::Minutes;
        envelope.value = Some(45.0);
        envelope.method_id = "synthetic_route_model".into();
        envelope.method_version = "test-v1".into();
        envelope.network_version = Some("synthetic-footpath-graph-v1".into());
        envelope.coverage = CoverageState::PartiallyValidated;
        envelope.source_evidence_refs = vec![
            "fixture:network-v1".into(),
            "fixture:route-result-v1".into(),
        ];
        envelope
    }

    fn envelope_for_metric(metric: AccessMetric) -> GeoAccessEnvelope {
        match metric {
            AccessMetric::GeodesicDistance => geodesic_envelope(),
            AccessMetric::ModeledRouteDistance => {
                let mut envelope = geodesic_envelope();
                envelope.metric = metric;
                envelope.travel_mode = TravelMode::Walking;
                envelope.unit = MetricUnit::Kilometers;
                envelope.value = Some(12.0);
                envelope.method_id = "synthetic_route_model".into();
                envelope.method_version = "test-v1".into();
                envelope.network_version = Some("synthetic-footpath-graph-v1".into());
                envelope.coverage = CoverageState::Validated;
                envelope.source_evidence_refs = vec![
                    "fixture:network-v1".into(),
                    "fixture:route-result-v1".into(),
                ];
                envelope
            }
            AccessMetric::ModeledTravelTime => modeled_time_envelope(),
            AccessMetric::ObservedTripDistance => {
                let mut envelope = geodesic_envelope();
                envelope.metric = metric;
                envelope.travel_mode = TravelMode::Walking;
                envelope.method_id = "observed-trip-log".into();
                envelope.method_version = "1".into();
                envelope
            }
            AccessMetric::ObservedTripTime => {
                let mut envelope = geodesic_envelope();
                envelope.metric = metric;
                envelope.travel_mode = TravelMode::Walking;
                envelope.unit = MetricUnit::Minutes;
                envelope.value = Some(45.0);
                envelope.method_id = "observed-trip-log".into();
                envelope.method_version = "1".into();
                envelope
            }
        }
    }

    #[test]
    fn geodesic_helper_matches_canonical_haversine_exactly() {
        let cases = [
            (51.5, -0.12, 48.85, 2.35),
            (-26.2041, 28.0473, -25.7479, 28.2293),
            (0.0, 0.0, 0.0, 0.0),
        ];
        for (lat1, lon1, lat2, lon2) in cases {
            assert_eq!(
                checked_geodesic_distance_km(lat1, lon1, lat2, lon2).unwrap(),
                crate::geo::haversine_km(lat1, lon1, lat2, lon2)
            );
        }
    }

    #[test]
    fn geodesic_helper_rejects_invalid_coordinates() {
        assert_eq!(
            checked_geodesic_distance_km(91.0, 0.0, 0.0, 0.0),
            Err(ValidationIssue::InvalidCoordinate)
        );
        assert_eq!(
            checked_geodesic_distance_km(0.0, f64::NAN, 0.0, 0.0),
            Err(ValidationIssue::InvalidCoordinate)
        );
    }

    #[test]
    fn valid_geodesic_envelope_passes_and_cannot_claim_network_route() {
        assert!(geodesic_envelope().validate().is_ok());

        let mut mislabeled = geodesic_envelope();
        mislabeled.metric = AccessMetric::ModeledRouteDistance;
        assert!(mislabeled.validate().is_err());
    }

    #[test]
    fn modeled_time_requires_versioned_network_and_declared_coverage() {
        assert!(modeled_time_envelope().validate().is_ok());

        let mut no_network_version = modeled_time_envelope();
        no_network_version.network_version = None;
        assert!(no_network_version.validate().is_err());

        let mut unknown_coverage = modeled_time_envelope();
        unknown_coverage.coverage = CoverageState::Unknown;
        assert!(unknown_coverage.validate().is_err());
    }

    #[test]
    fn units_and_modes_cannot_be_silently_converted() {
        let mut impossible = geodesic_envelope();
        impossible.unit = MetricUnit::Minutes;
        impossible.travel_mode = TravelMode::Walking;
        assert!(impossible.validate().is_err());

        let mut route_called_straight_line = modeled_time_envelope();
        route_called_straight_line.travel_mode = TravelMode::StraightLine;
        assert!(route_called_straight_line.validate().is_err());
    }

    #[test]
    fn metric_unit_matrix_rejects_every_incompatible_unit() {
        let metrics = [
            AccessMetric::GeodesicDistance,
            AccessMetric::ModeledRouteDistance,
            AccessMetric::ModeledTravelTime,
            AccessMetric::ObservedTripDistance,
            AccessMetric::ObservedTripTime,
        ];
        let units = [
            MetricUnit::Meters,
            MetricUnit::Kilometers,
            MetricUnit::Seconds,
            MetricUnit::Minutes,
        ];

        for metric in metrics {
            let base = envelope_for_metric(metric);
            let is_distance = matches!(
                metric,
                AccessMetric::GeodesicDistance
                    | AccessMetric::ModeledRouteDistance
                    | AccessMetric::ObservedTripDistance
            );
            for unit in units {
                let mut candidate = base.clone();
                candidate.unit = unit;
                let expected = if is_distance {
                    matches!(unit, MetricUnit::Meters | MetricUnit::Kilometers)
                } else {
                    matches!(unit, MetricUnit::Seconds | MetricUnit::Minutes)
                };
                assert_eq!(
                    candidate.validate().is_ok(),
                    expected,
                    "metric {metric:?} accepted unexpected unit {unit:?}"
                );
            }
        }
    }

    #[test]
    fn metric_mode_matrix_never_accepts_route_or_trip_as_straight_line() {
        let metrics = [
            AccessMetric::GeodesicDistance,
            AccessMetric::ModeledRouteDistance,
            AccessMetric::ModeledTravelTime,
            AccessMetric::ObservedTripDistance,
            AccessMetric::ObservedTripTime,
        ];
        let modes = [
            TravelMode::StraightLine,
            TravelMode::Walking,
            TravelMode::Bicycle,
            TravelMode::Motorcycle,
            TravelMode::MotorVehicle,
            TravelMode::AnimalDrawn,
            TravelMode::Multimodal,
            TravelMode::Unknown,
        ];

        for metric in metrics {
            let base = envelope_for_metric(metric);
            for mode in modes {
                let mut candidate = base.clone();
                candidate.travel_mode = mode;
                let expected = match metric {
                    AccessMetric::GeodesicDistance => mode == TravelMode::StraightLine,
                    _ => !matches!(mode, TravelMode::StraightLine | TravelMode::Unknown),
                };
                assert_eq!(
                    candidate.validate().is_ok(),
                    expected,
                    "metric {metric:?} accepted unexpected mode {mode:?}"
                );
            }
        }
    }

    #[test]
    fn modeled_state_coverage_matrix_keeps_outcomes_consistent() {
        let states = [
            AccessState::Available,
            AccessState::NoRoute,
            AccessState::OutsideCoverage,
            AccessState::Unknown,
            AccessState::Stale,
            AccessState::Conflicting,
        ];
        let coverage_states = [
            CoverageState::NotApplicable,
            CoverageState::Validated,
            CoverageState::PartiallyValidated,
            CoverageState::Unvalidated,
            CoverageState::OutsideCoverage,
            CoverageState::Unknown,
        ];

        for state in states {
            for coverage in coverage_states {
                let mut candidate = modeled_time_envelope();
                candidate.state = state;
                candidate.value = (state == AccessState::Available).then_some(45.0);
                candidate.coverage = coverage;
                let coverage_matches_state = (state == AccessState::OutsideCoverage)
                    == (coverage == CoverageState::OutsideCoverage);
                let available_coverage_is_usable = state != AccessState::Available
                    || matches!(
                        coverage,
                        CoverageState::Validated
                            | CoverageState::PartiallyValidated
                            | CoverageState::Unvalidated
                    );
                let no_route_is_supported = state != AccessState::NoRoute
                    || matches!(
                        coverage,
                        CoverageState::Validated | CoverageState::PartiallyValidated
                    );
                let expected = coverage != CoverageState::NotApplicable
                    && coverage_matches_state
                    && available_coverage_is_usable
                    && no_route_is_supported;
                assert_eq!(
                    candidate.validate().is_ok(),
                    expected,
                    "state {state:?} accepted unexpected coverage {coverage:?}"
                );
            }
        }
    }

    #[test]
    fn unavailable_states_are_distinct_and_never_use_zero_as_a_sentinel() {
        let mut no_route = modeled_time_envelope();
        no_route.state = AccessState::NoRoute;
        no_route.value = None;
        no_route.coverage = CoverageState::Validated;
        assert!(no_route.validate().is_ok());

        let mut unknown = modeled_time_envelope();
        unknown.state = AccessState::Unknown;
        unknown.value = None;
        unknown.coverage = CoverageState::Unknown;
        assert!(unknown.validate().is_ok());

        let mut outside = modeled_time_envelope();
        outside.state = AccessState::OutsideCoverage;
        outside.value = None;
        outside.coverage = CoverageState::OutsideCoverage;
        assert!(outside.validate().is_ok());

        let mut false_zero = modeled_time_envelope();
        false_zero.state = AccessState::NoRoute;
        false_zero.value = Some(0.0);
        assert!(false_zero.validate().is_err());
    }

    #[test]
    fn unsupported_no_route_claims_fail_closed_on_coverage_and_evidence() {
        let mut no_route = modeled_time_envelope();
        no_route.state = AccessState::NoRoute;
        no_route.value = None;
        no_route.coverage = CoverageState::Validated;
        assert!(no_route.validate().is_ok());

        no_route.coverage = CoverageState::Unknown;
        assert!(no_route.validate().is_err());

        no_route.coverage = CoverageState::Validated;
        no_route.source_evidence_refs.clear();
        assert!(no_route.validate().is_err());

        let mut unknown = modeled_time_envelope();
        unknown.state = AccessState::Unknown;
        unknown.value = None;
        unknown.coverage = CoverageState::Unknown;
        unknown.source_evidence_refs.clear();
        assert!(unknown.validate().is_ok());

        let mut outside = modeled_time_envelope();
        outside.state = AccessState::OutsideCoverage;
        outside.value = None;
        outside.coverage = CoverageState::OutsideCoverage;
        outside.source_evidence_refs.clear();
        assert!(outside.validate().is_err());
    }

    #[test]
    fn evidence_references_must_be_nonempty_unique_and_canonical() {
        let mut duplicate = modeled_time_envelope();
        duplicate.source_evidence_refs =
            vec!["fixture:network-v1".into(), "fixture:network-v1".into()];
        assert!(duplicate.validate().is_err());

        let mut unsorted = modeled_time_envelope();
        unsorted.source_evidence_refs = vec!["fixture:z".into(), "fixture:a".into()];
        assert!(unsorted.validate().is_err());
    }

    #[test]
    fn quantified_uncertainty_requires_a_finite_nonnegative_bound() {
        let mut envelope = modeled_time_envelope();
        envelope.uncertainty = UncertaintyDeclaration {
            kind: UncertaintyKind::QuantifiedAbsoluteBound,
            absolute_bound: Some(5.0),
        };
        assert!(envelope.validate().is_ok());

        envelope.uncertainty.absolute_bound = Some(-1.0);
        assert!(envelope.validate().is_err());

        envelope.uncertainty.absolute_bound = None;
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn identical_envelopes_serialize_deterministically() {
        let envelope = modeled_time_envelope();
        let encoded = serde_json::to_vec(&envelope).unwrap();
        let decoded: GeoAccessEnvelope = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, envelope);
        assert_eq!(encoded, serde_json::to_vec(&decoded).unwrap());
    }

    fn sample(reference: &str, predicted: f64, observed: f64) -> TravelTimeValidationSample {
        TravelTimeValidationSample {
            sample_ref: reference.into(),
            scope_ref: "scope:fixture-v1".into(),
            method_id: "route-model".into(),
            method_version: "model-v1".into(),
            travel_mode: TravelMode::Walking,
            season_label: "wet-season".into(),
            validation_design: ValidationDesign::SyntheticOracle,
            partition_ref: "partition:synthetic-oracle-v1".into(),
            predicted_minutes: predicted,
            observed_minutes: observed,
        }
    }

    #[test]
    fn travel_time_error_summary_reports_median_p90_and_signed_bias() {
        let samples = [
            sample("trip:1", 10.0, 10.0),
            sample("trip:2", 20.0, 18.0),
            sample("trip:3", 30.0, 27.0),
            sample("trip:4", 40.0, 50.0),
            sample("trip:5", 50.0, 30.0),
        ];
        let sample_json = serde_json::to_vec(&samples[0]).unwrap();
        let decoded_sample: TravelTimeValidationSample =
            serde_json::from_slice(&sample_json).unwrap();
        assert_eq!(decoded_sample, samples[0]);

        let summary = summarize_travel_time_errors(&samples).unwrap();
        assert_eq!(summary.sample_count, 5);
        assert_eq!(summary.median_absolute_error_minutes, 3.0);
        assert_eq!(summary.p90_absolute_error_minutes, 20.0);
        assert_eq!(summary.mean_signed_error_minutes, 3.0);
        assert_eq!(summary.season_label, "wet-season");
        assert_eq!(summary.validation_design, ValidationDesign::SyntheticOracle);
        assert_eq!(summary.partition_ref, "partition:synthetic-oracle-v1");
        let serialized = serde_json::to_value(&summary).unwrap();
        assert_eq!(serialized["validation_design"], "synthetic_oracle");
        assert_eq!(serialized["p90_absolute_error_minutes"], 20.0);
    }

    #[test]
    fn travel_time_summary_refuses_cross_season_pooling_and_duplicate_samples() {
        let samples = [sample("trip:1", 10.0, 10.0), sample("trip:2", 20.0, 18.0)];
        assert!(summarize_travel_time_errors(&samples).is_ok());

        let mut mixed_season = samples.clone();
        mixed_season[1].season_label = "dry-season".into();
        assert!(summarize_travel_time_errors(&mixed_season).is_err());

        let mut mixed_design = samples.clone();
        mixed_design[1].validation_design = ValidationDesign::SpatialHoldout;
        assert!(summarize_travel_time_errors(&mixed_design).is_err());

        let duplicate = [sample("trip:1", 10.0, 10.0), sample("trip:1", 20.0, 18.0)];
        assert!(summarize_travel_time_errors(&duplicate).is_err());
    }

    #[test]
    fn travel_time_summary_rejects_invalid_pairs_instead_of_dropping_them() {
        let samples = [
            sample("trip:1", 10.0, 10.0),
            sample("trip:2", f64::NAN, 18.0),
        ];
        assert!(summarize_travel_time_errors(&samples).is_err());
        assert!(summarize_travel_time_errors(&[]).is_err());
    }
}
