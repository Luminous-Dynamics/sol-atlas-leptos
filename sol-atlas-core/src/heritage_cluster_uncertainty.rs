// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cluster-level metric intervals and correlation-adjusted effective units.
//!
//! This module bootstraps declared independent evaluation units, not their
//! constituent pixels or objects. A supplied correlation matrix describes
//! correlation between unit-level metric estimation errors. It is validated
//! and used to compute an equal-variance effective-unit diagnostic.
//!
//! The validator cannot authenticate the correlation evidence or establish
//! statistical independence. Confidence intervals are withheld when the
//! declared correlation tolerance or minimum effective-unit gate is not met.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_CLUSTER_UNCERTAINTY_SCHEMA_VERSION: &str =
    "luminous.heritage-cluster-uncertainty.v0.1";
pub const MIN_BOOTSTRAP_REPLICATES: usize = 1_000;
pub const MAX_BOOTSTRAP_REPLICATES: usize = 100_000;
pub const MIN_DECLARED_EFFECTIVE_UNITS: usize = 5;
pub const MAX_INDEPENDENT_UNITS: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependentUnitKind {
    Site,
    Region,
    BufferedSpatialBlock,
    SensorFamily,
    AcquisitionEpoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    HigherIsBetter,
    LowerIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterIntervalStatus {
    Qualified,
    InsufficientEffectiveUnits,
    CorrelationExceedsTolerance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndependentUnitMetric {
    pub unit_id: String,
    /// One metric result for the complete independent unit.
    pub metric_value: f64,
    /// Number of lower-level items contributing to this unit-level metric.
    /// This is descriptive support, not a weight in the macro-unit estimand.
    pub support_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterBootstrapInput {
    pub schema_version: String,
    pub evaluation_run_sha256: String,
    pub split_manifest_sha256: String,
    pub metric_definition_sha256: String,
    pub correlation_model_sha256: String,
    pub metric_id: String,
    pub metric_unit: String,
    pub metric_direction: MetricDirection,
    pub independent_unit_kind: IndependentUnitKind,
    pub independence_basis_ref: String,
    pub correlation_evidence_ref: String,
    /// Minimum effective number of independent units required to emit a CI.
    pub minimum_effective_units: usize,
    pub bootstrap_replicates: usize,
    pub random_seed: u64,
    /// Supported nominal coverage levels: 0.90, 0.95 or 0.99.
    pub confidence_level: f64,
    /// Preregistered maximum permitted inter-unit metric-error correlation.
    pub maximum_inter_unit_correlation: f64,
    pub units: Vec<IndependentUnitMetric>,
    /// Correlation matrix for unit-level metric estimation errors.
    pub correlation_matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterBootstrapReport {
    pub schema_version: String,
    pub evaluation_run_sha256: String,
    pub split_manifest_sha256: String,
    pub metric_definition_sha256: String,
    pub correlation_model_sha256: String,
    pub metric_id: String,
    pub metric_unit: String,
    pub metric_direction: MetricDirection,
    pub independent_unit_kind: IndependentUnitKind,
    pub unit_count: usize,
    pub total_support_count: u64,
    pub minimum_unit_support_count: u64,
    pub maximum_unit_support_count: u64,
    /// Equal-weight arithmetic mean of independent-unit metric values.
    pub macro_unit_mean: f64,
    pub confidence_level: f64,
    pub bootstrap_replicates: usize,
    pub random_seed: u64,
    pub correlation_adjusted_effective_units: f64,
    pub maximum_observed_inter_unit_correlation: f64,
    pub maximum_allowed_inter_unit_correlation: f64,
    pub interval_status: ClusterIntervalStatus,
    pub percentile_lower: Option<f64>,
    pub percentile_upper: Option<f64>,
    pub bootstrap_standard_error: Option<f64>,
    pub claim_ceiling: String,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterBootstrapError {
    SchemaVersionMismatch,
    InvalidEvaluationRunDigest,
    InvalidSplitManifestDigest,
    InvalidMetricDefinitionDigest,
    InvalidCorrelationModelDigest,
    EmptyMetricIdentity,
    EmptyIndependenceBasisReceipt,
    EmptyCorrelationEvidenceReceipt,
    TooFewIndependentUnits,
    TooManyIndependentUnits,
    EmptyUnitId,
    DuplicateUnitId(String),
    InvalidMetricValue(String),
    ZeroSupportCount(String),
    SupportCountOverflow,
    InvalidMinimumEffectiveUnits,
    InvalidBootstrapReplicateCount,
    UnsupportedConfidenceLevel,
    InvalidCorrelationTolerance,
    CorrelationMatrixShapeMismatch,
    CorrelationNotFiniteOrOutOfRange,
    CorrelationMatrixNotSymmetric,
    CorrelationMatrixDiagonalNotOne,
    CorrelationMatrixNotPositiveSemidefinite,
    InvalidEffectiveUnitCount,
}

impl fmt::Display for ClusterBootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ClusterBootstrapError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn correlation_matrix_is_psd(matrix: &[Vec<f64>]) -> bool {
    let n = matrix.len();
    let mut lower = vec![vec![0.0; n]; n];
    let tolerance = 1e-10;

    for i in 0..n {
        for j in 0..=i {
            let mut residual = matrix[i][j];
            for k in 0..j {
                residual -= lower[i][k] * lower[j][k];
            }
            if i == j {
                if residual < -tolerance {
                    return false;
                }
                lower[i][j] = residual.max(0.0).sqrt();
            } else if lower[j][j] > tolerance.sqrt() {
                lower[i][j] = residual / lower[j][j];
            } else if residual.abs() > tolerance {
                return false;
            }
        }
    }
    true
}

fn validate_correlation_matrix(
    matrix: &[Vec<f64>],
    unit_count: usize,
) -> Result<(f64, f64), ClusterBootstrapError> {
    if matrix.len() != unit_count || matrix.iter().any(|row| row.len() != unit_count) {
        return Err(ClusterBootstrapError::CorrelationMatrixShapeMismatch);
    }

    let symmetry_tolerance = 1e-12;
    let mut correlation_sum = 0.0;
    let mut maximum_off_diagonal = 0.0_f64;
    for i in 0..unit_count {
        for j in 0..unit_count {
            let value = matrix[i][j];
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(ClusterBootstrapError::CorrelationNotFiniteOrOutOfRange);
            }
            if (value - matrix[j][i]).abs() > symmetry_tolerance {
                return Err(ClusterBootstrapError::CorrelationMatrixNotSymmetric);
            }
            if i == j {
                if (value - 1.0).abs() > symmetry_tolerance {
                    return Err(ClusterBootstrapError::CorrelationMatrixDiagonalNotOne);
                }
            } else {
                maximum_off_diagonal = maximum_off_diagonal.max(value);
            }
            correlation_sum += value;
        }
    }
    if !correlation_matrix_is_psd(matrix) {
        return Err(ClusterBootstrapError::CorrelationMatrixNotPositiveSemidefinite);
    }

    let n = unit_count as f64;
    let effective_units = n * n / correlation_sum;
    if !effective_units.is_finite()
        || effective_units < 1.0 - 1e-9
        || effective_units > n + 1e-9
    {
        return Err(ClusterBootstrapError::InvalidEffectiveUnitCount);
    }

    Ok((effective_units.clamp(1.0, n), maximum_off_diagonal))
}

#[derive(Debug, Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn index_below(&mut self, bound: usize) -> usize {
        let bound = bound as u64;
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return (value % bound) as usize;
            }
        }
    }
}

fn percentile(sorted: &[f64], probability: f64) -> f64 {
    let position = probability * (sorted.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        sorted[lower_index]
    } else {
        let fraction = position - lower_index as f64;
        sorted[lower_index] * (1.0 - fraction) + sorted[upper_index] * fraction
    }
}

fn run_cluster_bootstrap(
    values: &[f64],
    replicates: usize,
    seed: u64,
    confidence_level: f64,
) -> (f64, f64, f64) {
    let mut rng = SplitMix64::new(seed);
    let unit_count = values.len();
    let mut bootstrap_means = Vec::with_capacity(replicates);

    for _ in 0..replicates {
        let mut sum = 0.0;
        for _ in 0..unit_count {
            sum += values[rng.index_below(unit_count)];
        }
        bootstrap_means.push(sum / unit_count as f64);
    }

    bootstrap_means.sort_unstable_by(f64::total_cmp);
    let alpha = (1.0 - confidence_level) / 2.0;
    let lower = percentile(&bootstrap_means, alpha);
    let upper = percentile(&bootstrap_means, 1.0 - alpha);
    let mean = bootstrap_means.iter().sum::<f64>() / replicates as f64;
    let variance = bootstrap_means
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (replicates - 1) as f64;
    (lower, upper, variance.sqrt())
}

pub fn evaluate_cluster_metric_uncertainty(
    input: &ClusterBootstrapInput,
) -> Result<ClusterBootstrapReport, ClusterBootstrapError> {
    if input.schema_version != HERITAGE_CLUSTER_UNCERTAINTY_SCHEMA_VERSION {
        return Err(ClusterBootstrapError::SchemaVersionMismatch);
    }
    if !valid_sha256(&input.evaluation_run_sha256) {
        return Err(ClusterBootstrapError::InvalidEvaluationRunDigest);
    }
    if !valid_sha256(&input.split_manifest_sha256) {
        return Err(ClusterBootstrapError::InvalidSplitManifestDigest);
    }
    if !valid_sha256(&input.metric_definition_sha256) {
        return Err(ClusterBootstrapError::InvalidMetricDefinitionDigest);
    }
    if !valid_sha256(&input.correlation_model_sha256) {
        return Err(ClusterBootstrapError::InvalidCorrelationModelDigest);
    }
    if !non_empty(&input.metric_id) || !non_empty(&input.metric_unit) {
        return Err(ClusterBootstrapError::EmptyMetricIdentity);
    }
    if !non_empty(&input.independence_basis_ref) {
        return Err(ClusterBootstrapError::EmptyIndependenceBasisReceipt);
    }
    if !non_empty(&input.correlation_evidence_ref) {
        return Err(ClusterBootstrapError::EmptyCorrelationEvidenceReceipt);
    }
    if input.units.len() < 2 {
        return Err(ClusterBootstrapError::TooFewIndependentUnits);
    }
    if input.units.len() > MAX_INDEPENDENT_UNITS {
        return Err(ClusterBootstrapError::TooManyIndependentUnits);
    }
    if input.minimum_effective_units < MIN_DECLARED_EFFECTIVE_UNITS {
        return Err(ClusterBootstrapError::InvalidMinimumEffectiveUnits);
    }
    if !(MIN_BOOTSTRAP_REPLICATES..=MAX_BOOTSTRAP_REPLICATES)
        .contains(&input.bootstrap_replicates)
    {
        return Err(ClusterBootstrapError::InvalidBootstrapReplicateCount);
    }
    if ![0.90, 0.95, 0.99]
        .iter()
        .any(|level| (input.confidence_level - level).abs() < 1e-12)
    {
        return Err(ClusterBootstrapError::UnsupportedConfidenceLevel);
    }
    if !input.maximum_inter_unit_correlation.is_finite()
        || !(0.0..=1.0).contains(&input.maximum_inter_unit_correlation)
    {
        return Err(ClusterBootstrapError::InvalidCorrelationTolerance);
    }

    let mut unit_ids = BTreeSet::new();
    let mut metric_values = Vec::with_capacity(input.units.len());
    let mut total_support_count = 0_u64;
    let mut minimum_unit_support_count = u64::MAX;
    let mut maximum_unit_support_count = 0_u64;

    for unit in &input.units {
        if !non_empty(&unit.unit_id) {
            return Err(ClusterBootstrapError::EmptyUnitId);
        }
        if !unit_ids.insert(unit.unit_id.as_str()) {
            return Err(ClusterBootstrapError::DuplicateUnitId(unit.unit_id.clone()));
        }
        if !unit.metric_value.is_finite() {
            return Err(ClusterBootstrapError::InvalidMetricValue(
                unit.unit_id.clone(),
            ));
        }
        if unit.support_count == 0 {
            return Err(ClusterBootstrapError::ZeroSupportCount(
                unit.unit_id.clone(),
            ));
        }
        total_support_count = total_support_count
            .checked_add(unit.support_count)
            .ok_or(ClusterBootstrapError::SupportCountOverflow)?;
        minimum_unit_support_count = minimum_unit_support_count.min(unit.support_count);
        maximum_unit_support_count = maximum_unit_support_count.max(unit.support_count);
        metric_values.push(unit.metric_value);
    }

    let (effective_units, maximum_observed_inter_unit_correlation) =
        validate_correlation_matrix(&input.correlation_matrix, input.units.len())?;
    let macro_unit_mean = metric_values.iter().sum::<f64>() / metric_values.len() as f64;

    let interval_status = if maximum_observed_inter_unit_correlation
        > input.maximum_inter_unit_correlation
    {
        ClusterIntervalStatus::CorrelationExceedsTolerance
    } else if effective_units < input.minimum_effective_units as f64 {
        ClusterIntervalStatus::InsufficientEffectiveUnits
    } else {
        ClusterIntervalStatus::Qualified
    };

    let (percentile_lower, percentile_upper, bootstrap_standard_error) =
        if interval_status == ClusterIntervalStatus::Qualified {
            let (lower, upper, standard_error) = run_cluster_bootstrap(
                &metric_values,
                input.bootstrap_replicates,
                input.random_seed,
                input.confidence_level,
            );
            (Some(lower), Some(upper), Some(standard_error))
        } else {
            (None, None, None)
        };

    Ok(ClusterBootstrapReport {
        schema_version: HERITAGE_CLUSTER_UNCERTAINTY_SCHEMA_VERSION.to_owned(),
        evaluation_run_sha256: input.evaluation_run_sha256.clone(),
        split_manifest_sha256: input.split_manifest_sha256.clone(),
        metric_definition_sha256: input.metric_definition_sha256.clone(),
        correlation_model_sha256: input.correlation_model_sha256.clone(),
        metric_id: input.metric_id.clone(),
        metric_unit: input.metric_unit.clone(),
        metric_direction: input.metric_direction,
        independent_unit_kind: input.independent_unit_kind,
        unit_count: input.units.len(),
        total_support_count,
        minimum_unit_support_count,
        maximum_unit_support_count,
        macro_unit_mean,
        confidence_level: input.confidence_level,
        bootstrap_replicates: input.bootstrap_replicates,
        random_seed: input.random_seed,
        correlation_adjusted_effective_units: effective_units,
        maximum_observed_inter_unit_correlation,
        maximum_allowed_inter_unit_correlation: input.maximum_inter_unit_correlation,
        interval_status,
        percentile_lower,
        percentile_upper,
        bootstrap_standard_error,
        claim_ceiling: "macro_independent_unit_mean_with_conditional_percentile_bootstrap".to_owned(),
        assumptions: vec![
            "Each supplied metric value summarizes one complete declared independent evaluation unit.".to_owned(),
            "The point estimate is an equal-weight macro mean across independent units; support_count is descriptive, not a weight.".to_owned(),
            "The supplied correlation matrix refers to correlation among unit-level metric estimation errors.".to_owned(),
            "Effective units use the equal-marginal-variance design-effect formula K² divided by the sum of all correlation-matrix entries.".to_owned(),
            "The percentile bootstrap resamples independent units only and is emitted only when declared correlation and effective-unit gates pass.".to_owned(),
            "Correlation evidence, independence-basis receipts, evaluation digests and metric semantics are externally supplied and not authenticated here.".to_owned(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn unit(id: &str, value: f64) -> IndependentUnitMetric {
        IndependentUnitMetric {
            unit_id: id.to_owned(),
            metric_value: value,
            support_count: 100,
        }
    }

    fn input() -> ClusterBootstrapInput {
        ClusterBootstrapInput {
            schema_version: HERITAGE_CLUSTER_UNCERTAINTY_SCHEMA_VERSION.to_owned(),
            evaluation_run_sha256: digest('a'),
            split_manifest_sha256: digest('b'),
            metric_definition_sha256: digest('c'),
            correlation_model_sha256: digest('d'),
            metric_id: "stone-instance-f1".to_owned(),
            metric_unit: "score".to_owned(),
            metric_direction: MetricDirection::HigherIsBetter,
            independent_unit_kind: IndependentUnitKind::Site,
            independence_basis_ref: "hdb19://site-holdout/receipt/1".to_owned(),
            correlation_evidence_ref: "correlogram://metric-errors/receipt/1".to_owned(),
            minimum_effective_units: 5,
            bootstrap_replicates: MIN_BOOTSTRAP_REPLICATES,
            random_seed: 42,
            confidence_level: 0.95,
            maximum_inter_unit_correlation: 0.05,
            units: vec![
                unit("site-1", 0.81),
                unit("site-2", 0.83),
                unit("site-3", 0.75),
                unit("site-4", 0.88),
                unit("site-5", 0.79),
                unit("site-6", 0.86),
                unit("site-7", 0.84),
                unit("site-8", 0.82),
                unit("site-9", 0.80),
                unit("site-10", 0.87),
            ],
            correlation_matrix: vec![vec![1.0; 10]; 10],
        }
        .with_identity_correlation()
    }

    trait IdentityCorrelation {
        fn with_identity_correlation(self) -> Self;
    }

    impl IdentityCorrelation for ClusterBootstrapInput {
        fn with_identity_correlation(mut self) -> Self {
            for i in 0..self.correlation_matrix.len() {
                for j in 0..self.correlation_matrix.len() {
                    self.correlation_matrix[i][j] = if i == j { 1.0 } else { 0.0 };
                }
            }
            self
        }
    }

    #[test]
    fn identity_correlation_keeps_all_effective_units_and_emits_ci() {
        let report = evaluate_cluster_metric_uncertainty(&input()).expect("valid input");
        assert_eq!(report.unit_count, 10);
        assert!((report.correlation_adjusted_effective_units - 10.0).abs() < 1e-12);
        assert_eq!(report.interval_status, ClusterIntervalStatus::Qualified);
        assert!(report.percentile_lower.unwrap() <= report.macro_unit_mean);
        assert!(report.percentile_upper.unwrap() >= report.macro_unit_mean);
        assert!(report.bootstrap_standard_error.unwrap() > 0.0);
    }

    #[test]
    fn fully_correlated_units_reduce_effective_count_to_one_and_withhold_ci() {
        let mut value = input();
        value.correlation_matrix = vec![vec![1.0; 10]; 10];
        let report = evaluate_cluster_metric_uncertainty(&value).expect("valid correlation matrix");
        assert!((report.correlation_adjusted_effective_units - 1.0).abs() < 1e-12);
        assert_eq!(
            report.interval_status,
            ClusterIntervalStatus::CorrelationExceedsTolerance
        );
        assert!(report.percentile_lower.is_none());
        assert!(report.percentile_upper.is_none());
    }

    #[test]
    fn excessive_correlation_withholds_interval_even_when_effective_count_is_above_minimum() {
        let mut value = input();
        let mut matrix = vec![vec![0.0; 10]; 10];
        for i in 0..10 {
            matrix[i][i] = 1.0;
            for j in 0..10 {
                if i != j {
                    matrix[i][j] = 0.08;
                }
            }
        }
        value.correlation_matrix = matrix;
        let report = evaluate_cluster_metric_uncertainty(&value).expect("valid correlation matrix");
        assert!(report.correlation_adjusted_effective_units > 5.0);
        assert_eq!(
            report.interval_status,
            ClusterIntervalStatus::CorrelationExceedsTolerance
        );
        assert!(report.percentile_lower.is_none());
    }

    #[test]
    fn insufficient_effective_units_withholds_interval() {
        let mut value = input();
        value.minimum_effective_units = 11;
        let report = evaluate_cluster_metric_uncertainty(&value).expect("valid input");
        assert_eq!(
            report.interval_status,
            ClusterIntervalStatus::InsufficientEffectiveUnits
        );
        assert!(report.percentile_lower.is_none());
    }

    #[test]
    fn bootstrap_is_deterministic_for_same_seed_and_exact_input() {
        let value = input();
        let first = evaluate_cluster_metric_uncertainty(&value).expect("first");
        let second = evaluate_cluster_metric_uncertainty(&value).expect("second");
        assert_eq!(first.percentile_lower, second.percentile_lower);
        assert_eq!(first.percentile_upper, second.percentile_upper);
        assert_eq!(first.bootstrap_standard_error, second.bootstrap_standard_error);
    }

    #[test]
    fn correlation_matrix_must_be_positive_semidefinite() {
        let mut value = input();
        value.correlation_matrix = vec![
            vec![1.0, 0.9, 0.1],
            vec![0.9, 1.0, 0.9],
            vec![0.1, 0.9, 1.0],
        ];
        value.units.truncate(3);
        assert_eq!(
            evaluate_cluster_metric_uncertainty(&value),
            Err(ClusterBootstrapError::CorrelationMatrixNotPositiveSemidefinite)
        );
    }

    #[test]
    fn duplicate_units_and_zero_support_fail_closed() {
        let mut value = input();
        value.units[1].unit_id = value.units[0].unit_id.clone();
        assert!(matches!(
            evaluate_cluster_metric_uncertainty(&value),
            Err(ClusterBootstrapError::DuplicateUnitId(_))
        ));
        value.units[1].unit_id = "site-2".to_owned();
        value.units[1].support_count = 0;
        assert!(matches!(
            evaluate_cluster_metric_uncertainty(&value),
            Err(ClusterBootstrapError::ZeroSupportCount(_))
        ));
    }

    #[test]
    fn schema_and_fixture_are_parseable_and_fixture_is_auditable() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-cluster-uncertainty-v0.1.schema.json"
        ))
        .expect("schema JSON");
        let fixture: ClusterBootstrapInput = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-cluster-uncertainty-synthetic-v0.1.json"
        ))
        .expect("fixture JSON");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(
            evaluate_cluster_metric_uncertainty(&fixture)
                .expect("fixture evaluation")
                .interval_status,
            ClusterIntervalStatus::Qualified
        );
    }
}
