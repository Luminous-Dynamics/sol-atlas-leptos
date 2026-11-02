// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Directional spectral-geometric transfer metrics for cropmark-like proxy tasks.
//!
//! This module implements the directional ratio and top-M reciprocal average
//! described in Gravanis & Agapiou (PNAS Nexus, 2026), over externally computed
//! A-to-A and A-to-H spectral-cluster distances. It does not compute spectra,
//! construct cropmark labels, or establish archaeological significance.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const SPECTRAL_TRANSFER_SCHEMA_VERSION: &str =
    "luminous.heritage-spectral-transfer.v0.1";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SpectralDistanceMetric {
    Euclidean,
    Wasserstein,
    KolmogorovSmirnov,
    Mahalanobis,
}

pub const SPECTRAL_DISTANCE_METRICS: [SpectralDistanceMetric; 4] = [
    SpectralDistanceMetric::Euclidean,
    SpectralDistanceMetric::Wasserstein,
    SpectralDistanceMetric::KolmogorovSmirnov,
    SpectralDistanceMetric::Mahalanobis,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyDirectionalDistance {
    pub calendar_month: u8,
    pub metric: SpectralDistanceMetric,
    /// d(A_source(month), A_target(month)).
    pub source_archaeology_to_target_archaeology: f64,
    /// d(A_source(month), H_target(month)).
    pub source_archaeology_to_target_background: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionalSpectralTransferInput {
    pub source_region_id: String,
    pub target_region_id: String,
    /// This v0.1 implementation is deliberately scoped to the cropmark
    /// vegetation-proxy A/H class definition.
    pub task_scope: String,
    pub positive_class_semantics: String,
    pub background_class_semantics: String,
    pub label_protocol_ref: String,
    /// Digest for the band/feature transformations used to create clusters.
    pub feature_definition_sha256: String,
    /// Digest for the implementation that computed the distance samples.
    pub distance_pipeline_sha256: String,
    pub monthly_distances: Vec<MonthlyDirectionalDistance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectionalSpectralTransferConfig {
    /// Number of shared months with the lowest directional ratios to retain.
    /// The cited study uses M=3.
    pub top_month_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyDirectionalRatio {
    pub calendar_month: u8,
    /// r_source_to_target = d(A_s,A_t) / d(A_s,H_t); lower is stronger.
    pub ratio: f64,
    pub reciprocal_strength: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDirectionalStrength {
    pub metric: SpectralDistanceMetric,
    pub shared_month_count: usize,
    /// Month ratios are in chronological order.
    pub monthly_ratios: Vec<MonthlyDirectionalRatio>,
    /// The selected top-M months are in ascending-ratio rank order.
    pub selected_calendar_months: Vec<u8>,
    pub selected_reciprocal_strengths: Vec<f64>,
    /// Mean reciprocal ratio across the selected top-M months.
    pub directional_strength_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionalSpectralTransferReport {
    pub schema_version: String,
    pub source_region_id: String,
    pub target_region_id: String,
    pub task_scope: String,
    pub positive_class_semantics: String,
    pub background_class_semantics: String,
    pub label_protocol_ref: String,
    pub feature_definition_sha256: String,
    pub distance_pipeline_sha256: String,
    pub top_month_count: usize,
    /// A single report preserves source -> target; it is never symmetrised.
    pub metrics: Vec<MetricDirectionalStrength>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferAssociationRecord {
    pub metric: SpectralDistanceMetric,
    pub source_region_id: String,
    pub target_region_id: String,
    pub directional_strength_score: f64,
    /// Held-out target performance from the corresponding one-way experiment.
    pub target_transfer_f1: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionJackknifeAssociation {
    pub omitted_region_id: String,
    pub remaining_directed_pair_count: usize,
    /// None means too few remaining pairs or a constant rank vector.
    pub spearman_rank_correlation: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferAssociationReport {
    pub metric: SpectralDistanceMetric,
    pub directed_pair_count: usize,
    pub unique_region_count: usize,
    pub spearman_rank_correlation: Option<f64>,
    pub pearson_correlation: Option<f64>,
    /// Leave-one-landscape-out sensitivity. This is not a confidence interval.
    pub region_jackknife: Vec<RegionJackknifeAssociation>,
    pub region_jackknife_spearman_min: Option<f64>,
    pub region_jackknife_spearman_max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectralTransferError {
    EmptySourceRegionId,
    EmptyTargetRegionId,
    IdenticalSourceAndTarget,
    InvalidTaskScope,
    EmptyPositiveClassSemantics,
    EmptyBackgroundClassSemantics,
    EmptyLabelProtocolReference,
    InvalidFeatureDefinitionDigest,
    InvalidDistancePipelineDigest,
    EmptyMonthlyDistances,
    InvalidTopMonthCount,
    InvalidCalendarMonth { calendar_month: u8 },
    DuplicateMonthMetric {
        calendar_month: u8,
        metric: SpectralDistanceMetric,
    },
    InvalidDistance {
        calendar_month: u8,
        metric: SpectralDistanceMetric,
    },
    InvalidDirectionalRatio {
        calendar_month: u8,
        metric: SpectralDistanceMetric,
    },
    NonFiniteDirectionalStrength {
        metric: SpectralDistanceMetric,
    },
    InsufficientSharedMonths {
        metric: SpectralDistanceMetric,
        available: usize,
        required: usize,
    },
    InsufficientDirectedPairs,
    MixedAssociationMetrics,
    DuplicateDirectedPair {
        source_region_id: String,
        target_region_id: String,
    },
    InvalidAssociationScore,
}

impl fmt::Display for SpectralTransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage spectral transfer error: {self:?}")
    }
}

impl std::error::Error for SpectralTransferError {}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_calendar_month(value: u8) -> bool {
    (1..=12).contains(&value)
}

fn directional_ratio(
    distance: &MonthlyDirectionalDistance,
) -> Result<f64, SpectralTransferError> {
    let numerator = distance.source_archaeology_to_target_archaeology;
    let denominator = distance.source_archaeology_to_target_background;

    // The published ratio is undefined for zero/negative distance values.
    // Fail closed rather than introduce an undocumented epsilon regulariser.
    if !numerator.is_finite()
        || !denominator.is_finite()
        || numerator <= 0.0
        || denominator <= 0.0
    {
        return Err(SpectralTransferError::InvalidDistance {
            calendar_month: distance.calendar_month,
            metric: distance.metric,
        });
    }

    let ratio = numerator / denominator;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(SpectralTransferError::InvalidDirectionalRatio {
            calendar_month: distance.calendar_month,
            metric: distance.metric,
        });
    }
    Ok(ratio)
}

/// Calculate the directional DSS for one ordered source -> target pair.
///
/// For each shared month m:
///   r_st(m) = d(A_s(m), A_t(m)) / d(A_s(m), H_t(m))
/// Select the M months with the lowest r_st and compute:
///   DSS_st = mean(1 / r_st(m)) over those selected months.
///
/// A reverse target -> source result requires a separate input. The score is
/// meaningful only for the cropmark-proxy A/H definitions and distance
/// implementation recorded by the caller; it is not a universal archaeology
/// similarity score.
pub fn calculate_directional_spectral_transfer(
    input: &DirectionalSpectralTransferInput,
    config: DirectionalSpectralTransferConfig,
) -> Result<DirectionalSpectralTransferReport, SpectralTransferError> {
    if input.source_region_id.trim().is_empty() {
        return Err(SpectralTransferError::EmptySourceRegionId);
    }
    if input.target_region_id.trim().is_empty() {
        return Err(SpectralTransferError::EmptyTargetRegionId);
    }
    if input.source_region_id == input.target_region_id {
        return Err(SpectralTransferError::IdenticalSourceAndTarget);
    }
    if input.task_scope != "cropmark_vegetation_proxy" {
        return Err(SpectralTransferError::InvalidTaskScope);
    }
    if input.positive_class_semantics.trim().is_empty() {
        return Err(SpectralTransferError::EmptyPositiveClassSemantics);
    }
    if input.background_class_semantics.trim().is_empty() {
        return Err(SpectralTransferError::EmptyBackgroundClassSemantics);
    }
    if input.label_protocol_ref.trim().is_empty() {
        return Err(SpectralTransferError::EmptyLabelProtocolReference);
    }
    if !is_sha256(&input.feature_definition_sha256) {
        return Err(SpectralTransferError::InvalidFeatureDefinitionDigest);
    }
    if !is_sha256(&input.distance_pipeline_sha256) {
        return Err(SpectralTransferError::InvalidDistancePipelineDigest);
    }
    if input.monthly_distances.is_empty() {
        return Err(SpectralTransferError::EmptyMonthlyDistances);
    }
    if config.top_month_count == 0 || config.top_month_count > 12 {
        return Err(SpectralTransferError::InvalidTopMonthCount);
    }

    let mut month_metric_keys = BTreeSet::new();
    for distance in &input.monthly_distances {
        if !valid_calendar_month(distance.calendar_month) {
            return Err(SpectralTransferError::InvalidCalendarMonth {
                calendar_month: distance.calendar_month,
            });
        }
        if !month_metric_keys.insert((distance.calendar_month, distance.metric)) {
            return Err(SpectralTransferError::DuplicateMonthMetric {
                calendar_month: distance.calendar_month,
                metric: distance.metric,
            });
        }
    }

    let mut metric_results = Vec::with_capacity(SPECTRAL_DISTANCE_METRICS.len());
    for metric in SPECTRAL_DISTANCE_METRICS {
        let mut ratios = Vec::new();
        for distance in input.monthly_distances.iter().filter(|item| item.metric == metric) {
            let ratio = directional_ratio(distance)?;
            let reciprocal_strength = 1.0 / ratio;
            if !reciprocal_strength.is_finite() || reciprocal_strength <= 0.0 {
                return Err(SpectralTransferError::NonFiniteDirectionalStrength { metric });
            }
            ratios.push(MonthlyDirectionalRatio {
                calendar_month: distance.calendar_month,
                ratio,
                reciprocal_strength,
            });
        }

        if ratios.len() < config.top_month_count {
            return Err(SpectralTransferError::InsufficientSharedMonths {
                metric,
                available: ratios.len(),
                required: config.top_month_count,
            });
        }

        ratios.sort_by_key(|entry| entry.calendar_month);
        let mut selected = ratios.clone();
        selected.sort_by(|left, right| {
            left.ratio
                .total_cmp(&right.ratio)
                .then(left.calendar_month.cmp(&right.calendar_month))
        });
        selected.truncate(config.top_month_count);

        let selected_reciprocal_strengths: Vec<f64> = selected
            .iter()
            .map(|entry| entry.reciprocal_strength)
            .collect();
        let dss = selected_reciprocal_strengths.iter().sum::<f64>()
            / selected_reciprocal_strengths.len() as f64;
        if !dss.is_finite() {
            return Err(SpectralTransferError::NonFiniteDirectionalStrength { metric });
        }

        metric_results.push(MetricDirectionalStrength {
            metric,
            shared_month_count: ratios.len(),
            monthly_ratios: ratios,
            selected_calendar_months: selected.iter().map(|entry| entry.calendar_month).collect(),
            selected_reciprocal_strengths,
            directional_strength_score: dss,
        });
    }

    Ok(DirectionalSpectralTransferReport {
        schema_version: SPECTRAL_TRANSFER_SCHEMA_VERSION.to_owned(),
        source_region_id: input.source_region_id.clone(),
        target_region_id: input.target_region_id.clone(),
        task_scope: input.task_scope.clone(),
        positive_class_semantics: input.positive_class_semantics.clone(),
        background_class_semantics: input.background_class_semantics.clone(),
        label_protocol_ref: input.label_protocol_ref.clone(),
        feature_definition_sha256: input.feature_definition_sha256.clone(),
        distance_pipeline_sha256: input.distance_pipeline_sha256.clone(),
        top_month_count: config.top_month_count,
        metrics: metric_results,
    })
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut indices: Vec<usize> = (0..values.len()).collect();
    indices.sort_by(|left, right| {
        values[*left]
            .total_cmp(&values[*right])
            .then(left.cmp(right))
    });

    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < indices.len() {
        let mut end = start + 1;
        while end < indices.len()
            && values[indices[start]].total_cmp(&values[indices[end]]).is_eq()
        {
            end += 1;
        }
        // One-based average rank for ties.
        let average_rank = ((start + 1) as f64 + end as f64) / 2.0;
        for index in &indices[start..end] {
            ranks[*index] = average_rank;
        }
        start = end;
    }
    ranks
}

fn pearson_correlation(left: &[f64], right: &[f64]) -> Option<f64> {
    if left.len() != right.len() || left.len() < 2 {
        return None;
    }
    let left_mean = left.iter().sum::<f64>() / left.len() as f64;
    let right_mean = right.iter().sum::<f64>() / right.len() as f64;
    let mut covariance = 0.0;
    let mut left_variance = 0.0;
    let mut right_variance = 0.0;
    for (left_value, right_value) in left.iter().zip(right) {
        let left_delta = left_value - left_mean;
        let right_delta = right_value - right_mean;
        covariance += left_delta * right_delta;
        left_variance += left_delta * left_delta;
        right_variance += right_delta * right_delta;
    }
    let denominator = (left_variance * right_variance).sqrt();
    if !denominator.is_finite() || denominator <= 0.0 {
        return None;
    }
    let correlation = covariance / denominator;
    correlation.is_finite().then_some(correlation)
}

/// Compare a single metric's DSS ranking with observed one-way target F1 scores.
/// Use only independently held-out directed pairs. Reversing a pair produces
/// another distinct record; do not average/symmetrise it before this analysis.
pub fn evaluate_transfer_score_association(
    records: &[TransferAssociationRecord],
) -> Result<TransferAssociationReport, SpectralTransferError> {
    if records.len() < 3 {
        return Err(SpectralTransferError::InsufficientDirectedPairs);
    }
    let metric = records[0].metric;
    let mut pairs = BTreeSet::new();
    let mut dss = Vec::with_capacity(records.len());
    let mut f1 = Vec::with_capacity(records.len());

    for record in records {
        if record.metric != metric {
            return Err(SpectralTransferError::MixedAssociationMetrics);
        }
        if record.source_region_id.trim().is_empty()
            || record.target_region_id.trim().is_empty()
            || record.source_region_id == record.target_region_id
        {
            return Err(SpectralTransferError::InvalidAssociationScore);
        }
        if !pairs.insert((
            record.source_region_id.as_str(),
            record.target_region_id.as_str(),
        )) {
            return Err(SpectralTransferError::DuplicateDirectedPair {
                source_region_id: record.source_region_id.clone(),
                target_region_id: record.target_region_id.clone(),
            });
        }
        if !record.directional_strength_score.is_finite()
            || record.directional_strength_score < 0.0
            || !record.target_transfer_f1.is_finite()
            || !(0.0..=1.0).contains(&record.target_transfer_f1)
        {
            return Err(SpectralTransferError::InvalidAssociationScore);
        }
        dss.push(record.directional_strength_score);
        f1.push(record.target_transfer_f1);
    }

    let dss_ranks = average_ranks(&dss);
    let f1_ranks = average_ranks(&f1);
    let spearman_rank_correlation = pearson_correlation(&dss_ranks, &f1_ranks);
    let pearson = pearson_correlation(&dss, &f1);

    // Remove every pair touching one landscape. This is a deterministic
    // sensitivity diagnostic that reduces the chance a single landscape
    // dominates the association; it is not a bootstrap confidence interval.
    let regions: BTreeSet<&str> = records
        .iter()
        .flat_map(|record| [
            record.source_region_id.as_str(),
            record.target_region_id.as_str(),
        ])
        .collect();
    let mut region_jackknife = Vec::with_capacity(regions.len());
    let mut jackknife_spearman_values = Vec::new();

    for omitted_region_id in &regions {
        let remaining: Vec<&TransferAssociationRecord> = records
            .iter()
            .filter(|record| {
                record.source_region_id.as_str() != *omitted_region_id
                    && record.target_region_id.as_str() != *omitted_region_id
            })
            .collect();

        let (remaining_dss, remaining_f1): (Vec<f64>, Vec<f64>) = remaining
            .iter()
            .map(|record| (
                record.directional_strength_score,
                record.target_transfer_f1,
            ))
            .unzip();

        let remaining_spearman = if remaining.len() >= 3 {
            let dss_rank = average_ranks(&remaining_dss);
            let f1_rank = average_ranks(&remaining_f1);
            pearson_correlation(&dss_rank, &f1_rank)
        } else {
            None
        };

        if let Some(value) = remaining_spearman {
            jackknife_spearman_values.push(value);
        }
        region_jackknife.push(RegionJackknifeAssociation {
            omitted_region_id: (*omitted_region_id).to_owned(),
            remaining_directed_pair_count: remaining.len(),
            spearman_rank_correlation: remaining_spearman,
        });
    }

    let region_jackknife_spearman_min = jackknife_spearman_values
        .iter()
        .copied()
        .min_by(f64::total_cmp);
    let region_jackknife_spearman_max = jackknife_spearman_values
        .iter()
        .copied()
        .max_by(f64::total_cmp);

    Ok(TransferAssociationReport {
        metric,
        directed_pair_count: records.len(),
        unique_region_count: regions.len(),
        spearman_rank_correlation,
        pearson_correlation: pearson,
        region_jackknife,
        region_jackknife_spearman_min,
        region_jackknife_spearman_max,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(source: &str, target: &str, distance_pairs: &[(u8, f64, f64)]) -> DirectionalSpectralTransferInput {
        let mut monthly_distances = Vec::new();
        for (month, aa, ah) in distance_pairs {
            for metric in SPECTRAL_DISTANCE_METRICS {
                monthly_distances.push(MonthlyDirectionalDistance {
                    calendar_month: *month,
                    metric,
                    source_archaeology_to_target_archaeology: *aa,
                    source_archaeology_to_target_background: *ah,
                });
            }
        }
        DirectionalSpectralTransferInput {
            source_region_id: source.to_owned(),
            target_region_id: target.to_owned(),
            task_scope: "cropmark_vegetation_proxy".into(),
            positive_class_semantics: "vegetation over buried archaeological remains".into(),
            background_class_semantics: "other vegetated pixels".into(),
            label_protocol_ref: "urn:synthetic:cropmark-label-protocol:v1".into(),
            feature_definition_sha256: "a".repeat(64),
            distance_pipeline_sha256: "b".repeat(64),
            monthly_distances,
        }
    }

    fn config() -> DirectionalSpectralTransferConfig {
        DirectionalSpectralTransferConfig { top_month_count: 3 }
    }

    #[test]
    fn dss_matches_top_three_reciprocal_ratio_formula() {
        let data = input("source-a", "target-b", &[
            (1, 1.0, 4.0), // r=.25, reciprocal=4
            (2, 1.0, 2.0), // r=.50, reciprocal=2
            (3, 1.5, 3.0), // r=.50, reciprocal=2
            (4, 3.0, 2.0), // r=1.5, reciprocal=.666..., not top-3
        ]);
        let report = calculate_directional_spectral_transfer(&data, config()).unwrap();
        let euclidean = report.metrics.iter()
            .find(|metric| metric.metric == SpectralDistanceMetric::Euclidean)
            .unwrap();
        assert_eq!(euclidean.selected_calendar_months, vec![1, 2, 3]);
        assert!((euclidean.directional_strength_score - (8.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn reverse_direction_is_a_distinct_non_symmetric_measurement() {
        let forward = calculate_directional_spectral_transfer(
            &input("region-a", "region-b", &[(1, 1.0, 4.0), (2, 1.0, 2.0), (3, 1.0, 2.0)]),
            config(),
        ).unwrap();
        let reverse = calculate_directional_spectral_transfer(
            &input("region-b", "region-a", &[(1, 1.0, 1.0), (2, 2.0, 1.0), (3, 2.0, 1.0)]),
            config(),
        ).unwrap();
        let forward_dss = forward.metrics[0].directional_strength_score;
        let reverse_dss = reverse.metrics[0].directional_strength_score;
        assert!(forward_dss > reverse_dss);
        assert_ne!(forward.source_region_id, reverse.source_region_id);
    }

    #[test]
    fn each_of_the_four_distance_families_is_reported() {
        let report = calculate_directional_spectral_transfer(
            &input("source-a", "target-b", &[
                (1, 1.0, 3.0), (2, 1.0, 3.0), (3, 1.0, 3.0),
            ]),
            config(),
        ).unwrap();
        assert_eq!(report.metrics.len(), 4);
        for metric in SPECTRAL_DISTANCE_METRICS {
            assert!(report.metrics.iter().any(|entry| entry.metric == metric));
        }
    }

    #[test]
    fn insufficient_shared_months_fail_closed_per_metric() {
        let result = calculate_directional_spectral_transfer(
            &input("source-a", "target-b", &[
                (1, 1.0, 3.0), (2, 1.0, 3.0),
            ]),
            config(),
        );
        assert!(matches!(
            result,
            Err(SpectralTransferError::InsufficientSharedMonths { .. })
        ));
    }

    #[test]
    fn zero_distances_and_duplicate_month_metric_pairs_fail_closed() {
        let zero = calculate_directional_spectral_transfer(
            &input("source-a", "target-b", &[
                (1, 0.0, 3.0), (2, 1.0, 3.0), (3, 1.0, 3.0),
            ]),
            config(),
        );
        assert!(matches!(zero, Err(SpectralTransferError::InvalidDistance { .. })));

        let mut duplicate = input("source-a", "target-b", &[
            (1, 1.0, 3.0), (2, 1.0, 3.0), (3, 1.0, 3.0),
        ]);
        duplicate.monthly_distances.push(duplicate.monthly_distances[0].clone());
        assert!(matches!(
            calculate_directional_spectral_transfer(&duplicate, config()),
            Err(SpectralTransferError::DuplicateMonthMetric { .. })
        ));
    }

    #[test]
    fn association_computes_spearman_and_pearson_on_directed_pairs() {
        let records = vec![
            ("a", "b", 1.0, 0.2),
            ("a", "c", 2.0, 0.4),
            ("b", "a", 3.0, 0.6),
            ("b", "c", 4.0, 0.8),
        ].into_iter().map(|(source, target, dss, f1)| TransferAssociationRecord {
            metric: SpectralDistanceMetric::Euclidean,
            source_region_id: source.into(),
            target_region_id: target.into(),
            directional_strength_score: dss,
            target_transfer_f1: f1,
        }).collect::<Vec<_>>();
        let report = evaluate_transfer_score_association(&records).unwrap();
        assert_eq!(report.directed_pair_count, 4);
        assert_eq!(report.spearman_rank_correlation, Some(1.0));
        assert_eq!(report.pearson_correlation, Some(1.0));
    }

    #[test]
    fn region_jackknife_removes_all_pairs_touching_each_landscape() {
        let records = vec![
            ("a", "b", 1.0, 0.1),
            ("b", "c", 2.0, 0.2),
            ("c", "d", 3.0, 0.3),
            ("d", "e", 4.0, 0.4),
            ("e", "a", 5.0, 0.5),
        ].into_iter().map(|(source, target, dss, f1)| TransferAssociationRecord {
            metric: SpectralDistanceMetric::Euclidean,
            source_region_id: source.into(),
            target_region_id: target.into(),
            directional_strength_score: dss,
            target_transfer_f1: f1,
        }).collect::<Vec<_>>();

        let report = evaluate_transfer_score_association(&records).unwrap();
        assert_eq!(report.unique_region_count, 5);
        assert_eq!(report.region_jackknife.len(), 5);
        assert!(report.region_jackknife.iter().all(|entry| {
            entry.remaining_directed_pair_count == 3
                && entry.spearman_rank_correlation == Some(1.0)
        }));
        assert_eq!(report.region_jackknife_spearman_min, Some(1.0));
        assert_eq!(report.region_jackknife_spearman_max, Some(1.0));
    }

    #[test]
    fn rank_association_uses_average_ranks_for_ties() {
        let records = vec![
            ("a", "b", 1.0, 0.2),
            ("a", "c", 1.0, 0.4),
            ("b", "a", 2.0, 0.4),
            ("b", "c", 3.0, 0.8),
        ].into_iter().map(|(source, target, dss, f1)| TransferAssociationRecord {
            metric: SpectralDistanceMetric::Wasserstein,
            source_region_id: source.into(),
            target_region_id: target.into(),
            directional_strength_score: dss,
            target_transfer_f1: f1,
        }).collect::<Vec<_>>();
        let report = evaluate_transfer_score_association(&records).unwrap();
        assert!(report.spearman_rank_correlation.is_some());
    }

    #[test]
    fn non_cropmark_proxy_scope_is_rejected() {
        let mut data = input("source-a", "target-b", &[
            (1, 1.0, 3.0), (2, 1.0, 3.0), (3, 1.0, 3.0),
        ]);
        data.task_scope = "stone_wall_detection".into();
        assert_eq!(
            calculate_directional_spectral_transfer(&data, config()),
            Err(SpectralTransferError::InvalidTaskScope)
        );
    }

    #[test]
    fn calendar_month_outside_one_through_twelve_is_rejected() {
        let mut data = input("source-a", "target-b", &[
            (1, 1.0, 3.0), (2, 1.0, 3.0), (3, 1.0, 3.0),
        ]);
        data.monthly_distances[0].calendar_month = 13;
        assert_eq!(
            calculate_directional_spectral_transfer(&data, config()),
            Err(SpectralTransferError::InvalidCalendarMonth { calendar_month: 13 })
        );
    }

    #[test]
    fn too_few_pairs_mixed_metrics_and_repeated_directions_fail_closed() {
        let one = TransferAssociationRecord {
            metric: SpectralDistanceMetric::Euclidean,
            source_region_id: "a".into(),
            target_region_id: "b".into(),
            directional_strength_score: 1.0,
            target_transfer_f1: 0.5,
        };
        assert_eq!(
            evaluate_transfer_score_association(&[one.clone(), one.clone()]),
            Err(SpectralTransferError::InsufficientDirectedPairs)
        );

        let mut duplicate = vec![one.clone(), one.clone(), one.clone()];
        assert!(matches!(
            evaluate_transfer_score_association(&duplicate),
            Err(SpectralTransferError::DuplicateDirectedPair { .. })
        ));

        duplicate[1].target_region_id = "c".into();
        duplicate[1].metric = SpectralDistanceMetric::Wasserstein;
        duplicate[2].target_region_id = "d".into();
        assert_eq!(
            evaluate_transfer_score_association(&duplicate),
            Err(SpectralTransferError::MixedAssociationMetrics)
        );
    }
}
