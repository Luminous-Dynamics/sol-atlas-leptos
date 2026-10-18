// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed evidence observations for cross-project systems-health models.
//!
//! This is a deliberately small ingestion contract, not a truth oracle:
//! validation checks structure and internal consistency, not whether a source
//! is accurate, representative, independent, or causally relevant.
//!
//! Timestamps are Unix seconds. Values are expressed in the stated unit.
//! Missing data must carry an explicit reason; it must never silently become
//! zero. A source digest establishes byte identity only, not factual validity.

use serde::{Deserialize, Serialize};

/// Current schema version accepted by this validator.
///
/// Increment this when observation semantics or required fields change.
pub const OBSERVATION_SCHEMA_VERSION: u16 = 1;

/// One sourced measurement or reported value for a metric and entity.
///
/// Keep raw observations distinct from model estimates and policy scenarios.
/// Derived values should reference this record through a future model-run
/// manifest rather than overwriting the source value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationRecord {
    /// Version of this serialized observation contract.
    pub schema_version: u16,
    /// Stable identifier for this observation record.
    pub observation_id: String,
    /// Versioned metric identifier, such as `water.service_availability`.
    pub metric_id: String,
    /// Stable identifier for the entity being measured.
    pub entity_id: String,
    /// Optional geographic entity identifier; not a free-form location name.
    pub geography_id: Option<String>,
    /// Measured or reported value in `unit`. Missing values remain `None`.
    pub value: Option<f64>,
    /// Unit for the value, e.g. `percent`, `m3/day`, or `ZAR_2025`.
    pub unit: String,
    /// Inclusive start of the observation period, as Unix seconds.
    pub period_start_unix_s: i64,
    /// Inclusive end of the observation period, as Unix seconds.
    pub period_end_unix_s: i64,
    /// Stable source URI (for example, an official dataset or public record).
    pub source_uri: String,
    /// Optional SHA-256 digest of the exact source artifact, prefixed `sha256:`.
    ///
    /// Digest availability may be restricted by source access or licensing;
    /// absent digests must be surfaced by downstream provenance reporting.
    pub source_digest: Option<String>,
    /// Version of the measurement method or transformation used to produce
    /// this observation. Use a stable version or digest, not a vague label.
    pub method_version: String,
    /// Optional uncertainty for the observation's point value.
    pub uncertainty: Option<Uncertainty>,
    /// Required when `value` is `None`; forbidden when a value is present.
    pub missing_reason: Option<MissingValueReason>,
}

/// Why an observation has no numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingValueReason {
    /// The value was not collected for this period or entity.
    NotCollected,
    /// The value exists but is withheld or inaccessible.
    Withheld,
    /// The metric does not apply to this entity or period.
    NotApplicable,
    /// The measurement is below the method's detection limit.
    BelowDetectionLimit,
    /// The recorded value was invalidated by a correction or quality review.
    Invalidated,
    /// A documented reason exists but is not represented by another variant.
    Other,
}

/// Explicit uncertainty semantics for a reported numeric value.
///
/// This contract does not assume that every uncertainty can be reduced to a
/// probability distribution. Domain-specific uncertainty types can be added
/// through a versioned schema change rather than overloaded into one number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Uncertainty {
    /// Standard deviation in the same unit as the observation value.
    StandardDeviation { value: f64 },
    /// Bounds and nominal coverage probability for an interval in the same unit.
    Interval {
        lower: f64,
        upper: f64,
        coverage_probability: f64,
    },
}

/// Structural validation failures for an observation record.
///
/// These checks cannot validate source truth, measurement bias, independence,
/// causal relevance, or representativeness; those require separate review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationValidationIssue {
    UnsupportedSchemaVersion,
    EmptyObservationId,
    EmptyMetricId,
    EmptyEntityId,
    EmptyGeographyId,
    EmptyUnit,
    EmptySourceUri,
    EmptyMethodVersion,
    InvalidPeriod,
    NonFiniteValue,
    MissingReasonMismatch,
    InvalidSourceDigest,
    InvalidStandardDeviation,
    InvalidUncertaintyInterval,
    UncertaintyWithoutValue,
}

impl ObservationRecord {
    /// Check structural validity and return all detected issues.
    ///
    /// This method is fail-closed for malformed records, while preserving all
    /// issues in one pass so ingestion tooling can explain each rejection.
    pub fn validate(&self) -> Result<(), Vec<ObservationValidationIssue>> {
        let mut issues = Vec::new();

        if self.schema_version != OBSERVATION_SCHEMA_VERSION {
            issues.push(ObservationValidationIssue::UnsupportedSchemaVersion);
        }
        if is_blank(&self.observation_id) {
            issues.push(ObservationValidationIssue::EmptyObservationId);
        }
        if is_blank(&self.metric_id) {
            issues.push(ObservationValidationIssue::EmptyMetricId);
        }
        if is_blank(&self.entity_id) {
            issues.push(ObservationValidationIssue::EmptyEntityId);
        }
        if self.geography_id.as_deref().is_some_and(is_blank) {
            issues.push(ObservationValidationIssue::EmptyGeographyId);
        }
        if is_blank(&self.unit) {
            issues.push(ObservationValidationIssue::EmptyUnit);
        }
        if is_blank(&self.source_uri) {
            issues.push(ObservationValidationIssue::EmptySourceUri);
        }
        if is_blank(&self.method_version) {
            issues.push(ObservationValidationIssue::EmptyMethodVersion);
        }
        if self.period_start_unix_s > self.period_end_unix_s {
            issues.push(ObservationValidationIssue::InvalidPeriod);
        }
        if self.value.is_some_and(|value| !value.is_finite()) {
            issues.push(ObservationValidationIssue::NonFiniteValue);
        }

        // Exactly one must be present: a value, or an explicit missing reason.
        if self.value.is_some() == self.missing_reason.is_some() {
            issues.push(ObservationValidationIssue::MissingReasonMismatch);
        }

        if self
            .source_digest
            .as_deref()
            .is_some_and(|digest| !is_sha256_digest(digest))
        {
            issues.push(ObservationValidationIssue::InvalidSourceDigest);
        }

        match &self.uncertainty {
            Some(Uncertainty::StandardDeviation { value })
                if !value.is_finite() || *value < 0.0 =>
            {
                issues.push(ObservationValidationIssue::InvalidStandardDeviation);
            }
            Some(Uncertainty::Interval {
                lower,
                upper,
                coverage_probability,
            }) if !lower.is_finite()
                || !upper.is_finite()
                || lower > upper
                || !coverage_probability.is_finite()
                || *coverage_probability <= 0.0
                || *coverage_probability >= 1.0 =>
            {
                issues.push(ObservationValidationIssue::InvalidUncertaintyInterval);
            }
            _ => {}
        }

        if self.value.is_none() && self.uncertainty.is_some() {
            issues.push(ObservationValidationIssue::UncertaintyWithoutValue);
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_observation() -> ObservationRecord {
        ObservationRecord {
            schema_version: OBSERVATION_SCHEMA_VERSION,
            observation_id: "obs-001".into(),
            metric_id: "water.service_availability".into(),
            entity_id: "service-area-001".into(),
            geography_id: Some("za-gp-jhb-example".into()),
            value: Some(97.5),
            unit: "percent".into(),
            period_start_unix_s: 1_790_000_000,
            period_end_unix_s: 1_790_086_400,
            source_uri: "https://example.org/public-water-report.csv".into(),
            source_digest: Some(format!("sha256:{}", "a".repeat(64))),
            method_version: "availability-method@1".into(),
            uncertainty: Some(Uncertainty::Interval {
                lower: 95.0,
                upper: 99.0,
                coverage_probability: 0.95,
            }),
            missing_reason: None,
        }
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut observation = valid_observation();
        observation.schema_version = OBSERVATION_SCHEMA_VERSION + 1;

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::UnsupportedSchemaVersion)
        );
    }

    #[test]
    fn accepts_well_formed_observation() {
        assert_eq!(valid_observation().validate(), Ok(()));
    }

    #[test]
    fn rejects_unexplained_missing_value() {
        let mut observation = valid_observation();
        observation.value = None;
        observation.uncertainty = None;
        observation.missing_reason = None;

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::MissingReasonMismatch)
        );
    }

    #[test]
    fn accepts_explicit_missing_value() {
        let mut observation = valid_observation();
        observation.value = None;
        observation.uncertainty = None;
        observation.missing_reason = Some(MissingValueReason::NotCollected);

        assert_eq!(observation.validate(), Ok(()));
    }

    #[test]
    fn rejects_non_finite_values() {
        let mut observation = valid_observation();
        observation.value = Some(f64::NAN);

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::NonFiniteValue)
        );
    }

    #[test]
    fn rejects_reversed_periods() {
        let mut observation = valid_observation();
        observation.period_start_unix_s = observation.period_end_unix_s + 1;

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::InvalidPeriod)
        );
    }

    #[test]
    fn rejects_malformed_source_digest() {
        let mut observation = valid_observation();
        observation.source_digest = Some("sha256:abc".into());

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::InvalidSourceDigest)
        );
    }

    #[test]
    fn rejects_negative_standard_deviation() {
        let mut observation = valid_observation();
        observation.uncertainty = Some(Uncertainty::StandardDeviation { value: -0.1 });

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::InvalidStandardDeviation)
        );
    }

    #[test]
    fn rejects_invalid_interval_coverage() {
        let mut observation = valid_observation();
        observation.uncertainty = Some(Uncertainty::Interval {
            lower: 1.0,
            upper: 2.0,
            coverage_probability: 1.0,
        });

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::InvalidUncertaintyInterval)
        );
    }

    #[test]
    fn rejects_uncertainty_without_a_value() {
        let mut observation = valid_observation();
        observation.value = None;
        observation.missing_reason = Some(MissingValueReason::NotCollected);

        assert!(
            observation
                .validate()
                .unwrap_err()
                .contains(&ObservationValidationIssue::UncertaintyWithoutValue)
        );
    }

    #[test]
    fn round_trips_through_json() {
        let observation = valid_observation();
        let json = serde_json::to_string(&observation).expect("serialize observation");
        let decoded: ObservationRecord =
            serde_json::from_str(&json).expect("deserialize observation");
        assert_eq!(decoded, observation);
        assert_eq!(decoded.validate(), Ok(()));
    }
}
