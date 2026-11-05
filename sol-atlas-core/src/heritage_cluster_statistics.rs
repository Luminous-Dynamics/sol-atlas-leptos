// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cluster-level uncertainty for heritage benchmark metrics.
//!
//! The independent resampling unit is a site, region, spatial block, sensor
//! family or acquisition epoch—not a pixel, chip or individual stone.
//!
//! This module accepts an externally supplied correlation matrix and evidence
//! reference. It validates matrix mathematics, computes an equal-variance
//! effective-unit diagnostic, and performs a deterministic percentile
//! bootstrap over independent units. It does not estimate spatial dependence,
//! authenticate evidence, prove independence or establish archaeological truth.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_CLUSTER_STATISTICS_SCHEMA_VERSION: &str =
    "luminous.heritage-cluster-statistics.v0.1";
pub const MAX_INDEPENDENT_UNITS: usize = 512;
pub const MIN_BOOTSTRAP_REPLICATES: usize = 1_000;
pub const MAX_BOOTSTRAP_REPLICATES: usize = 100_000;
const MATRIX_TOLERANCE: f64 = 1.0e-10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependentUnitKind {
    Site,
    Region,
    SpatialBlock,
    SensorFamily,
    AcquisitionEpoch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitMetricObservation {
    pub unit_id: String,
    pub metric_value: f64,
    /// Descriptive support only. The macro-unit estimand does not weight by it.
    pub support_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationEvidence {
    /// Digest of the fitted correlation model and its parameters.
    pub model_sha256: String,
    /// E.g. variogram, correlogram, residual correlation, or design evidence.
    pub method: String,
    /// Reference to the retained evidence supporting this correlation matrix.
    pub evidence_ref: String,
    /// Receipt for the evidence package; this module does not authenticate it.
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterBootstrapInput {
    pub metric_name: String,
    pub split_manifest_sha256: String,
    pub evaluation_run_sha256: String,
    pub metric_definition_sha256: String,
    pub unit_kind: IndependentUnitKind,
    /// Row/column order of correlation_matrix is exactly this observation order.
    pub observations: Vec<UnitMetricObservation>,
    /// Correlation matrix of unit-level metric estimation errors, not raw pixels.
    pub correlation_matrix: Vec<Vec<f64>>,
    pub correlation_evidence: CorrelationEvidence,
    /// The K_eff formula requires this equal marginal variance assumption.
    pub equal_marginal_variance_assumed: bool,
    pub bootstrap_seed: u64,
    pub bootstrap_replicates: usize,
    pub confidence_level: f64,
    pub minimum_effective_units: f64,
    pub maximum_permitted_inter_unit_correlation: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeritageClusterStatisticsError {
    EmptyMetricName,
    InvalidDigest(String),
    MissingCorrelationEvidence,
    EqualMarginalVarianceAssumptionRequired,
    TooManyIndependentUnits,
    NoIndependentUnits,
    DuplicateOrEmptyUnitId(String),
    NonFiniteMetricValue(String),
    ZeroSupportCount(String),
    CorrelationMatrixDimensionMismatch,
    CorrelationMatrixNotSquare,
    NonFiniteCorrelation { row: usize, column: usize },
    CorrelationOutOfRange { row: usize, column: usize },
    CorrelationDiagonalNotOne(usize),
    CorrelationMatrixNotSymmetric { row: usize, column: usize },
    CorrelationMatrixNotPositiveSemidefinite,
    InvalidBootstrapReplicateCount,
    InvalidConfidenceLevel,
    InvalidMinimumEffectiveUnits,
    InvalidMaximumCorrelation,
    InvalidCorrelationSum,
}

impl fmt::Display for HeritageClusterStatisticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for HeritageClusterStatisticsError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceIntervalStatus {
    Qualified,
    InsufficientEffectiveUnits,
    CorrelationExceedsTolerance,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PercentileInterval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterBootstrapReport {
    pub schema_version: String,
    pub claim_ceiling: String,
    pub metric_name: String,
    pub split_manifest_sha256: String,
    pub evaluation_run_sha256: String,
    pub metric_definition_sha256: String,
    pub unit_kind: IndependentUnitKind,
    pub observations: Vec<UnitMetricObservation>,
    pub correlation_evidence: CorrelationEvidence,
    pub bootstrap_seed: u64,
    pub confidence_level: f64,
    pub requested_replicates: usize,
    pub completed_replicates: usize,
    pub independent_unit_count: usize,
    pub correlation_matrix_sum: f64,
    pub effective_unit_count: f64,
    pub maximum_off_diagonal_correlation: f64,
    pub status: ConfidenceIntervalStatus,
    pub interval: Option<PercentileInterval>,
    pub bootstrap_standard_error: Option<f64>,
    pub macro_unit_mean: f64,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_digest(value: &str, label: &str) -> Result<(), HeritageClusterStatisticsError> {
    if !is_sha256(value) {
        return Err(HeritageClusterStatisticsError::InvalidDigest(
            label.to_owned(),
        ));
    }
    Ok(())
}

fn validate_input(input: &ClusterBootstrapInput) -> Result<(), HeritageClusterStatisticsError> {
    if input.metric_name.trim().is_empty() {
        return Err(HeritageClusterStatisticsError::EmptyMetricName);
    }
    validate_digest(&input.split_manifest_sha256, "split_manifest_sha256")?;
    validate_digest(&input.evaluation_run_sha256, "evaluation_run_sha256")?;
    validate_digest(&input.metric_definition_sha256, "metric_definition_sha256")?;
    validate_digest(
        &input.correlation_evidence.model_sha256,
        "correlation_model_sha256",
    )?;
    if input.correlation_evidence.method.trim().is_empty()
        || input.correlation_evidence.evidence_ref.trim().is_empty()
        || input.correlation_evidence.receipt_ref.trim().is_empty()
    {
        return Err(HeritageClusterStatisticsError::MissingCorrelationEvidence);
    }
    if !input.equal_marginal_variance_assumed {
        return Err(
            HeritageClusterStatisticsError::EqualMarginalVarianceAssumptionRequired,
        );
    }
    if input.observations.is_empty() {
        return Err(HeritageClusterStatisticsError::NoIndependentUnits);
    }
    if input.observations.len() > MAX_INDEPENDENT_UNITS {
        return Err(HeritageClusterStatisticsError::TooManyIndependentUnits);
    }
    let mut identifiers = BTreeSet::new();
    for observation in &input.observations {
        if observation.unit_id.trim().is_empty()
            || !identifiers.insert(observation.unit_id.clone())
        {
            return Err(HeritageClusterStatisticsError::DuplicateOrEmptyUnitId(
                observation.unit_id.clone(),
            ));
        }
        if !observation.metric_value.is_finite() {
            return Err(HeritageClusterStatisticsError::NonFiniteMetricValue(
                observation.unit_id.clone(),
            ));
        }
        if observation.support_count == 0 {
            return Err(HeritageClusterStatisticsError::ZeroSupportCount(
                observation.unit_id.clone(),
            ));
        }
    }
    if input.bootstrap_replicates < MIN_BOOTSTRAP_REPLICATES
        || input.bootstrap_replicates > MAX_BOOTSTRAP_REPLICATES
    {
        return Err(HeritageClusterStatisticsError::InvalidBootstrapReplicateCount);
    }
    if !input.confidence_level.is_finite()
        || input.confidence_level <= 0.0
        || input.confidence_level >= 1.0
    {
        return Err(HeritageClusterStatisticsError::InvalidConfidenceLevel);
    }
    if !input.minimum_effective_units.is_finite() || input.minimum_effective_units < 1.0 {
        return Err(HeritageClusterStatisticsError::InvalidMinimumEffectiveUnits);
    }
    if !input.maximum_permitted_inter_unit_correlation.is_finite()
        || !(0.0..=1.0).contains(&input.maximum_permitted_inter_unit_correlation)
    {
        return Err(HeritageClusterStatisticsError::InvalidMaximumCorrelation);
    }
    validate_correlation_matrix(&input.correlation_matrix, input.observations.len())?;
    Ok(())
}

fn validate_correlation_matrix(
    matrix: &[Vec<f64>],
    unit_count: usize,
) -> Result<(), HeritageClusterStatisticsError> {
    if matrix.len() != unit_count {
        return Err(HeritageClusterStatisticsError::CorrelationMatrixDimensionMismatch);
    }
    if matrix.iter().any(|row| row.len() != unit_count) {
        return Err(HeritageClusterStatisticsError::CorrelationMatrixNotSquare);
    }
    for row in 0..unit_count {
        for column in 0..unit_count {
            let value = matrix[row][column];
            if !value.is_finite() {
                return Err(HeritageClusterStatisticsError::NonFiniteCorrelation {
                    row,
                    column,
                });
            }
            if !(0.0..=1.0).contains(&value) {
                return Err(HeritageClusterStatisticsError::CorrelationOutOfRange {
                    row,
                    column,
                });
            }
            if row == column && (value - 1.0).abs() > MATRIX_TOLERANCE {
                return Err(HeritageClusterStatisticsError::CorrelationDiagonalNotOne(
                    row,
                ));
            }
            if (matrix[row][column] - matrix[column][row]).abs() > MATRIX_TOLERANCE {
                return Err(HeritageClusterStatisticsError::CorrelationMatrixNotSymmetric {
                    row,
                    column,
                });
            }
        }
    }
    if !is_positive_semidefinite(matrix) {
        return Err(
            HeritageClusterStatisticsError::CorrelationMatrixNotPositiveSemidefinite,
        );
    }
    Ok(())
}

/// LDLᵀ factorisation test that accepts positive-semidefinite matrices,
/// including valid singular correlation matrices, within a fixed tolerance.
fn is_positive_semidefinite(matrix: &[Vec<f64>]) -> bool {
    let size = matrix.len();
    let mut lower = vec![vec![0.0_f64; size]; size];
    let mut diagonal = vec![0.0_f64; size];

    for pivot_index in 0..size {
        let mut pivot = matrix[pivot_index][pivot_index];
        for previous in 0..pivot_index {
            pivot -= lower[pivot_index][previous]
                * lower[pivot_index][previous]
                * diagonal[previous];
        }
        if pivot < -MATRIX_TOLERANCE {
            return false;
        }
        if pivot <= MATRIX_TOLERANCE {
            diagonal[pivot_index] = 0.0;
            for row in pivot_index + 1..size {
                let mut residual = matrix[row][pivot_index];
                for previous in 0..pivot_index {
                    residual -= lower[row][previous]
                        * lower[pivot_index][previous]
                        * diagonal[previous];
                }
                if residual.abs() > MATRIX_TOLERANCE {
                    return false;
                }
                lower[row][pivot_index] = 0.0;
            }
        } else {
            diagonal[pivot_index] = pivot;
            for row in pivot_index + 1..size {
                let mut residual = matrix[row][pivot_index];
                for previous in 0..pivot_index {
                    residual -= lower[row][previous]
                        * lower[pivot_index][previous]
                        * diagonal[previous];
                }
                lower[row][pivot_index] = residual / pivot;
            }
        }
    }
    true
}

fn correlation_diagnostics(matrix: &[Vec<f64>]) -> (f64, f64, f64) {
    let unit_count = matrix.len();
    let matrix_sum: f64 = matrix.iter().flatten().sum();
    let effective_units =
        ((unit_count * unit_count) as f64 / matrix_sum).clamp(1.0, unit_count as f64);
    let mut maximum_off_diagonal = 0.0_f64;
    for row in 0..unit_count {
        for column in row + 1..unit_count {
            maximum_off_diagonal =
                maximum_off_diagonal.max(matrix[row][column]);
        }
    }
    (matrix_sum, effective_units, maximum_off_diagonal)
}

#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
        value ^ (value >> 31)
    }

    /// Unbiased sampling from [0, upper_bound), using rejection rather than
    /// modulo-only reduction.
    fn next_bounded(&mut self, upper_bound: usize) -> usize {
        let bound = upper_bound as u64;
        let rejection_threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.next_u64();
            if value >= rejection_threshold {
                return (value % bound) as usize;
            }
        }
    }
}

fn macro_mean(observations: &[UnitMetricObservation]) -> f64 {
    observations
        .iter()
        .map(|observation| observation.metric_value)
        .sum::<f64>()
        / observations.len() as f64
}

fn quantile(sorted_values: &[f64], probability: f64) -> f64 {
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    let position = probability * (sorted_values.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    let fraction = position - lower_index as f64;
    sorted_values[lower_index]
        + fraction * (sorted_values[upper_index] - sorted_values[lower_index])
}

pub fn evaluate_cluster_bootstrap(
    input: &ClusterBootstrapInput,
) -> Result<ClusterBootstrapReport, HeritageClusterStatisticsError> {
    validate_input(input)?;
    let unit_count = input.observations.len();
    let (matrix_sum, effective_unit_count, maximum_off_diagonal_correlation) =
        correlation_diagnostics(&input.correlation_matrix);
    if !matrix_sum.is_finite() || matrix_sum <= 0.0 {
        return Err(HeritageClusterStatisticsError::InvalidCorrelationSum);
    }
    let status = if effective_unit_count < input.minimum_effective_units {
        ConfidenceIntervalStatus::InsufficientEffectiveUnits
    } else if maximum_off_diagonal_correlation
        > input.maximum_permitted_inter_unit_correlation
    {
        ConfidenceIntervalStatus::CorrelationExceedsTolerance
    } else {
        ConfidenceIntervalStatus::Qualified
    };

    let macro_unit_mean = macro_mean(&input.observations);
    let mut interval = None;
    let mut bootstrap_standard_error = None;
    let mut completed_replicates = 0;
    if status == ConfidenceIntervalStatus::Qualified {
        let mut random = SplitMix64::new(input.bootstrap_seed);
        let mut bootstrap_means = Vec::with_capacity(input.bootstrap_replicates);
        for _ in 0..input.bootstrap_replicates {
            let mut sum = 0.0;
            for _ in 0..unit_count {
                let index = random.next_bounded(unit_count);
                sum += input.observations[index].metric_value;
            }
            bootstrap_means.push(sum / unit_count as f64);
        }
        completed_replicates = bootstrap_means.len();
        let bootstrap_mean =
            bootstrap_means.iter().sum::<f64>() / bootstrap_means.len() as f64;
        let squared_deviations = bootstrap_means
            .iter()
            .map(|value| (value - bootstrap_mean).powi(2))
            .sum::<f64>();
        let standard_error =
            (squared_deviations / (bootstrap_means.len() - 1) as f64).sqrt();
        bootstrap_means.sort_by(f64::total_cmp);
        let tail_probability = (1.0 - input.confidence_level) / 2.0;
        interval = Some(PercentileInterval {
            lower: quantile(&bootstrap_means, tail_probability),
            upper: quantile(&bootstrap_means, 1.0 - tail_probability),
        });
        bootstrap_standard_error = Some(standard_error);
    }

    Ok(ClusterBootstrapReport {
        schema_version: HERITAGE_CLUSTER_STATISTICS_SCHEMA_VERSION.into(),
        claim_ceiling: "conditional_cluster_bootstrap_only".into(),
        metric_name: input.metric_name.clone(),
        split_manifest_sha256: input.split_manifest_sha256.clone(),
        evaluation_run_sha256: input.evaluation_run_sha256.clone(),
        metric_definition_sha256: input.metric_definition_sha256.clone(),
        unit_kind: input.unit_kind,
        observations: input.observations.clone(),
        correlation_evidence: input.correlation_evidence.clone(),
        bootstrap_seed: input.bootstrap_seed,
        confidence_level: input.confidence_level,
        requested_replicates: input.bootstrap_replicates,
        completed_replicates,
        independent_unit_count: unit_count,
        correlation_matrix_sum: matrix_sum,
        effective_unit_count,
        maximum_off_diagonal_correlation,
        status,
        interval,
        bootstrap_standard_error,
        macro_unit_mean,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn input() -> ClusterBootstrapInput {
        ClusterBootstrapInput {
            metric_name: "site_macro_f1".into(),
            split_manifest_sha256: digest('a'),
            evaluation_run_sha256: digest('b'),
            metric_definition_sha256: digest('c'),
            unit_kind: IndependentUnitKind::Site,
            observations: vec![
                UnitMetricObservation {
                    unit_id: "site-a".into(),
                    metric_value: 0.6,
                    support_count: 10,
                },
                UnitMetricObservation {
                    unit_id: "site-b".into(),
                    metric_value: 0.8,
                    support_count: 1_000,
                },
                UnitMetricObservation {
                    unit_id: "site-c".into(),
                    metric_value: 1.0,
                    support_count: 100,
                },
            ],
            correlation_matrix: vec![
                vec![1.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0],
                vec![0.0, 0.0, 1.0],
            ],
            correlation_evidence: CorrelationEvidence {
                model_sha256: digest('d'),
                method: "synthetic_independent_control".into(),
                evidence_ref: "evidence:synthetic-correlation".into(),
                receipt_ref: "receipt:synthetic-correlation".into(),
            },
            equal_marginal_variance_assumed: true,
            bootstrap_seed: 42,
            bootstrap_replicates: 2_000,
            confidence_level: 0.95,
            minimum_effective_units: 2.0,
            maximum_permitted_inter_unit_correlation: 0.5,
        }
    }

    #[test]
    fn independent_units_have_full_effective_count_and_qualified_interval() {
        let report = evaluate_cluster_bootstrap(&input()).unwrap();
        assert_eq!(report.status, ConfidenceIntervalStatus::Qualified);
        assert_eq!(report.independent_unit_count, 3);
        assert!((report.effective_unit_count - 3.0).abs() < 1e-12);
        assert_eq!(report.completed_replicates, 2_000);
        assert!(report.interval.unwrap().lower <= report.interval.unwrap().upper);
        assert!(report.bootstrap_standard_error.unwrap() > 0.0);
    }

    #[test]
    fn bootstrap_is_bitwise_deterministic_for_same_ordered_inputs_and_seed() {
        let first = evaluate_cluster_bootstrap(&input()).unwrap();
        let second = evaluate_cluster_bootstrap(&input()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn observations_are_equally_weighted_not_weighted_by_pixel_or_object_support() {
        let report = evaluate_cluster_bootstrap(&input()).unwrap();
        assert!((report.macro_unit_mean - 0.8).abs() < 1e-12);
    }

    #[test]
    fn correlation_matrix_reduces_effective_information() {
        let mut value = input();
        value.correlation_matrix = vec![
            vec![1.0, 0.8, 0.8],
            vec![0.8, 1.0, 0.8],
            vec![0.8, 0.8, 1.0],
        ];
        let report = evaluate_cluster_bootstrap(&value).unwrap();
        assert!((report.effective_unit_count - (9.0 / 7.8)).abs() < 1e-12);
        assert_eq!(
            report.status,
            ConfidenceIntervalStatus::InsufficientEffectiveUnits
        );
        assert_eq!(report.interval, None);
        assert_eq!(report.completed_replicates, 0);
    }

    #[test]
    fn excessive_inter_unit_correlation_withholds_interval() {
        let mut value = input();
        value.correlation_matrix = vec![
            vec![1.0, 0.6, 0.0],
            vec![0.6, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        value.maximum_permitted_inter_unit_correlation = 0.5;
        let report = evaluate_cluster_bootstrap(&value).unwrap();
        assert_eq!(
            report.status,
            ConfidenceIntervalStatus::CorrelationExceedsTolerance
        );
        assert_eq!(report.interval, None);
    }

    #[test]
    fn non_positive_semidefinite_matrix_fails_closed() {
        let mut value = input();
        value.correlation_matrix = vec![
            vec![1.0, 0.9, 0.9],
            vec![0.9, 1.0, 0.0],
            vec![0.9, 0.0, 1.0],
        ];
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::CorrelationMatrixNotPositiveSemidefinite
        );
    }

    #[test]
    fn singular_positive_semidefinite_matrix_is_accepted_then_withheld_by_neff() {
        let mut value = input();
        value.correlation_matrix = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let report = evaluate_cluster_bootstrap(&value).unwrap();
        assert!((report.effective_unit_count - 1.0).abs() < 1e-12);
        assert_eq!(
            report.status,
            ConfidenceIntervalStatus::InsufficientEffectiveUnits
        );
        assert_eq!(report.interval, None);
    }

    #[test]
    fn asymmetric_matrix_fails_closed() {
        let mut value = input();
        value.correlation_matrix[0][1] = 0.2;
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::CorrelationMatrixNotSymmetric {
                row: 0,
                column: 1
            }
        );
    }

    #[test]
    fn negative_correlation_is_outside_v01_assumptions() {
        let mut value = input();
        value.correlation_matrix[0][1] = -0.1;
        value.correlation_matrix[1][0] = -0.1;
        assert!(matches!(
            evaluate_cluster_bootstrap(&value),
            Err(HeritageClusterStatisticsError::CorrelationOutOfRange { .. })
        ));
    }

    #[test]
    fn matrix_dimensions_diagonal_and_non_finite_values_are_checked() {
        let mut value = input();
        value.correlation_matrix.pop();
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::CorrelationMatrixDimensionMismatch
        );
        value = input();
        value.correlation_matrix[0][0] = 0.9;
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::CorrelationDiagonalNotOne(0)
        );
        value = input();
        value.correlation_matrix[0][1] = f64::NAN;
        assert!(matches!(
            evaluate_cluster_bootstrap(&value),
            Err(HeritageClusterStatisticsError::NonFiniteCorrelation { .. })
        ));
    }

    #[test]
    fn invalid_evidence_digests_and_missing_receipts_fail_closed() {
        let mut value = input();
        value.split_manifest_sha256 = "not-a-digest".into();
        assert!(matches!(
            evaluate_cluster_bootstrap(&value),
            Err(HeritageClusterStatisticsError::InvalidDigest(_))
        ));
        value = input();
        value.correlation_evidence.receipt_ref.clear();
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::MissingCorrelationEvidence
        );
    }

    #[test]
    fn equal_marginal_variance_assumption_is_mandatory() {
        let mut value = input();
        value.equal_marginal_variance_assumed = false;
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::EqualMarginalVarianceAssumptionRequired
        );
    }

    #[test]
    fn duplicate_ids_and_zero_support_fail_closed() {
        let mut value = input();
        value.observations[1].unit_id = value.observations[0].unit_id.clone();
        assert!(matches!(
            evaluate_cluster_bootstrap(&value),
            Err(HeritageClusterStatisticsError::DuplicateOrEmptyUnitId(_))
        ));
        value = input();
        value.observations[0].support_count = 0;
        assert!(matches!(
            evaluate_cluster_bootstrap(&value),
            Err(HeritageClusterStatisticsError::ZeroSupportCount(_))
        ));
    }

    #[test]
    fn replicate_and_confidence_configuration_is_preregistered() {
        let mut value = input();
        value.bootstrap_replicates = 999;
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::InvalidBootstrapReplicateCount
        );
        value = input();
        value.confidence_level = 1.0;
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::InvalidConfidenceLevel
        );
    }

    #[test]
    fn invalid_metric_values_and_too_few_units_fail_closed() {
        let mut value = input();
        value.observations[0].metric_value = f64::INFINITY;
        assert!(matches!(
            evaluate_cluster_bootstrap(&value),
            Err(HeritageClusterStatisticsError::NonFiniteMetricValue(_))
        ));
        value = input();
        value.observations.clear();
        assert_eq!(
            evaluate_cluster_bootstrap(&value).unwrap_err(),
            HeritageClusterStatisticsError::NoIndependentUnits
        );
    }

    #[test]
    fn effective_unit_count_with_two_fully_correlated_units_is_one() {
        let mut value = input();
        value.observations.truncate(2);
        value.correlation_matrix = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let report = evaluate_cluster_bootstrap(&value).unwrap();
        assert!((report.effective_unit_count - 1.0).abs() < 1e-12);
        assert_eq!(report.interval, None);
    }

    #[test]
    fn one_unit_with_explicit_minimum_one_can_be_reported_conditionally() {
        let mut value = input();
        value.observations.truncate(1);
        value.correlation_matrix = vec![vec![1.0]];
        value.minimum_effective_units = 1.0;
        let report = evaluate_cluster_bootstrap(&value).unwrap();
        assert_eq!(report.status, ConfidenceIntervalStatus::Qualified);
        assert_eq!(report.interval.unwrap().lower, 0.6);
        assert_eq!(report.interval.unwrap().upper, 0.6);
    }
}
