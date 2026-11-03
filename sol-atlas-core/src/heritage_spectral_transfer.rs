// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Directional spectral-geometric transfer evaluation for archaeological cropmark proxies.
//!
//! This module implements the published directional-strength calculation from
//! monthly A->A and A->H spectral-cluster distances. The source->target direction
//! is explicit and is never symmetrized. It evaluates a proxy-transfer affinity;
//! it does not detect archaeology, prove buried remains, or establish that a
//! cropmark transfer result applies to other archaeological feature classes.
//!
//! A separate validation evaluator measures whether DSS ranks observed transfer
//! outcomes across many directed site pairs. Directed pairs share sites and are
//! dyadically dependent, so no naive pair-level confidence interval is emitted.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const DIRECTIONAL_SPECTRAL_TRANSFER_SCHEMA_VERSION: &str =
    "luminous.heritage-directional-spectral-transfer.v0.1";
pub const DSS_VALIDATION_SCHEMA_VERSION: &str = "luminous.heritage-dss-validation.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpectralDistanceMetric {
    Euclidean,
    BandWiseWasserstein,
    KolmogorovSmirnov,
    Mahalanobis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferSplitProtocol {
    PixelRandomInterpolation,
    WithinSiteSpatialBlocked,
    HeldOutSite,
    HeldOutRegion,
}

impl TransferSplitProtocol {
    pub fn is_primary_cross_site_generalization(self) -> bool {
        matches!(self, Self::HeldOutSite | Self::HeldOutRegion)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpectralMonthlyDistances {
    pub month_ordinal: i64,
    pub source_archaeological_sample_count: usize,
    pub target_archaeological_sample_count: usize,
    pub target_vegetated_control_sample_count: usize,
    /// Distance between source A and target A spectral subclusters.
    pub archaeological_to_archaeological_distance: f64,
    /// Distance between source A and target H (non-cropmark vegetated control).
    pub archaeological_to_control_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionalSpectralTransferInput {
    pub pair_id: String,
    pub source_site_id: String,
    pub target_site_id: String,
    pub distance_metric: SpectralDistanceMetric,
    pub split_protocol: TransferSplitProtocol,
    pub source_split_manifest_sha256: String,
    pub target_split_manifest_sha256: String,
    pub feature_transform_sha256: String,
    pub input_channel_semantics: String,
    pub phenophase_filter_ref: String,
    /// Number of months with the smallest positive A->A / A->H ratios retained.
    pub top_m_months: usize,
    /// Minimum number of shared valid phenophase months required for evaluation.
    pub minimum_shared_months: usize,
    /// Strictly increasing shared months with typed sample counts and distances.
    pub months: Vec<SpectralMonthlyDistances>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionalSpectralMonthResult {
    pub month_ordinal: i64,
    pub archaeological_to_archaeological_distance: f64,
    pub archaeological_to_control_distance: f64,
    pub directional_ratio: f64,
    pub reciprocal_affinity: f64,
    pub selected_in_top_m: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionalSpectralTransferReport {
    pub schema_version: String,
    pub pair_id: String,
    pub source_site_id: String,
    pub target_site_id: String,
    pub distance_metric: SpectralDistanceMetric,
    pub split_protocol: TransferSplitProtocol,
    pub primary_cross_site_generalization_eligible: bool,
    pub source_split_manifest_sha256: String,
    pub target_split_manifest_sha256: String,
    pub feature_transform_sha256: String,
    pub input_channel_semantics: String,
    pub phenophase_filter_ref: String,
    pub top_m_months: usize,
    pub shared_month_count: usize,
    pub selected_month_ordinals: Vec<i64>,
    /// Mean reciprocal of the top-M smallest directional A->A / A->H ratios.
    pub directional_strength_score: f64,
    pub months: Vec<DirectionalSpectralMonthResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectionalSpectralTransferError {
    EmptyPairId,
    EmptySourceSiteId,
    EmptyTargetSiteId,
    SourceAndTargetMustDiffer,
    InvalidSourceSplitDigest,
    InvalidTargetSplitDigest,
    InvalidFeatureTransformDigest,
    EmptyInputChannelSemantics,
    EmptyPhenophaseFilterReference,
    InvalidTopM,
    InvalidMinimumSharedMonths,
    InsufficientSharedMonths { found: usize, required: usize },
    MonthOrdinalInvalid { month_ordinal: i64 },
    MonthsNotStrictlyIncreasing,
    EmptySourceArchaeologicalCluster { month_ordinal: i64 },
    EmptyTargetArchaeologicalCluster { month_ordinal: i64 },
    EmptyTargetControlCluster { month_ordinal: i64 },
    InvalidDistance { month_ordinal: i64 },
    InvalidDirectionalRatio { month_ordinal: i64 },
    ScoreArithmeticOverflow,
}

impl fmt::Display for DirectionalSpectralTransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage directional spectral transfer error: {self:?}")
    }
}

impl std::error::Error for DirectionalSpectralTransferError {}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_month_ordinal(ordinal: i64) -> bool {
    let year = ordinal.div_euclid(12);
    (1..=9999).contains(&year)
}

fn validate_input(
    input: &DirectionalSpectralTransferInput,
) -> Result<(), DirectionalSpectralTransferError> {
    if input.pair_id.trim().is_empty() {
        return Err(DirectionalSpectralTransferError::EmptyPairId);
    }
    if input.source_site_id.trim().is_empty() {
        return Err(DirectionalSpectralTransferError::EmptySourceSiteId);
    }
    if input.target_site_id.trim().is_empty() {
        return Err(DirectionalSpectralTransferError::EmptyTargetSiteId);
    }
    if input.source_site_id == input.target_site_id {
        return Err(DirectionalSpectralTransferError::SourceAndTargetMustDiffer);
    }
    if !valid_sha256(&input.source_split_manifest_sha256) {
        return Err(DirectionalSpectralTransferError::InvalidSourceSplitDigest);
    }
    if !valid_sha256(&input.target_split_manifest_sha256) {
        return Err(DirectionalSpectralTransferError::InvalidTargetSplitDigest);
    }
    if !valid_sha256(&input.feature_transform_sha256) {
        return Err(DirectionalSpectralTransferError::InvalidFeatureTransformDigest);
    }
    if input.input_channel_semantics.trim().is_empty() {
        return Err(DirectionalSpectralTransferError::EmptyInputChannelSemantics);
    }
    if input.phenophase_filter_ref.trim().is_empty() {
        return Err(DirectionalSpectralTransferError::EmptyPhenophaseFilterReference);
    }
    if input.top_m_months == 0 {
        return Err(DirectionalSpectralTransferError::InvalidTopM);
    }
    if input.minimum_shared_months == 0 || input.top_m_months > input.minimum_shared_months {
        return Err(DirectionalSpectralTransferError::InvalidMinimumSharedMonths);
    }
    if input.months.len() < input.minimum_shared_months || input.months.len() < input.top_m_months {
        return Err(DirectionalSpectralTransferError::InsufficientSharedMonths {
            found: input.months.len(),
            required: input.minimum_shared_months.max(input.top_m_months),
        });
    }

    let mut previous = None;
    for month in &input.months {
        if !valid_month_ordinal(month.month_ordinal) {
            return Err(DirectionalSpectralTransferError::MonthOrdinalInvalid {
                month_ordinal: month.month_ordinal,
            });
        }
        if previous.is_some_and(|ordinal| month.month_ordinal <= ordinal) {
            return Err(DirectionalSpectralTransferError::MonthsNotStrictlyIncreasing);
        }
        if month.source_archaeological_sample_count == 0 {
            return Err(
                DirectionalSpectralTransferError::EmptySourceArchaeologicalCluster {
                    month_ordinal: month.month_ordinal,
                },
            );
        }
        if month.target_archaeological_sample_count == 0 {
            return Err(
                DirectionalSpectralTransferError::EmptyTargetArchaeologicalCluster {
                    month_ordinal: month.month_ordinal,
                },
            );
        }
        if month.target_vegetated_control_sample_count == 0 {
            return Err(
                DirectionalSpectralTransferError::EmptyTargetControlCluster {
                    month_ordinal: month.month_ordinal,
                },
            );
        }
        if !month.archaeological_to_archaeological_distance.is_finite()
            || month.archaeological_to_archaeological_distance <= 0.0
            || !month.archaeological_to_control_distance.is_finite()
            || month.archaeological_to_control_distance <= 0.0
        {
            return Err(DirectionalSpectralTransferError::InvalidDistance {
                month_ordinal: month.month_ordinal,
            });
        }
        let ratio = month.archaeological_to_archaeological_distance
            / month.archaeological_to_control_distance;
        let reciprocal = 1.0 / ratio;
        if !ratio.is_finite() || ratio <= 0.0 || !reciprocal.is_finite() || reciprocal <= 0.0 {
            return Err(DirectionalSpectralTransferError::InvalidDirectionalRatio {
                month_ordinal: month.month_ordinal,
            });
        }
        previous = Some(month.month_ordinal);
    }
    Ok(())
}

/// Compute one ordered source-site -> target-site Directional Strength Score.
///
/// For each shared month m, r(m) = d(A_source, A_target) / d(A_source, H_target).
/// DSS is the mean of 1/r over the top-M months with the smallest r values.
/// Reversing source and target requires a new input and a separate evaluation.
pub fn evaluate_directional_spectral_transfer(
    input: &DirectionalSpectralTransferInput,
) -> Result<DirectionalSpectralTransferReport, DirectionalSpectralTransferError> {
    validate_input(input)?;

    let mut month_results = input
        .months
        .iter()
        .map(|month| {
            let ratio = month.archaeological_to_archaeological_distance
                / month.archaeological_to_control_distance;
            let affinity = 1.0 / ratio;
            DirectionalSpectralMonthResult {
                month_ordinal: month.month_ordinal,
                archaeological_to_archaeological_distance: month
                    .archaeological_to_archaeological_distance,
                archaeological_to_control_distance: month.archaeological_to_control_distance,
                directional_ratio: ratio,
                reciprocal_affinity: affinity,
                selected_in_top_m: false,
            }
        })
        .collect::<Vec<_>>();

    let mut ranked = (0..month_results.len()).collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        month_results[*left]
            .directional_ratio
            .total_cmp(&month_results[*right].directional_ratio)
            .then_with(|| {
                month_results[*left]
                    .month_ordinal
                    .cmp(&month_results[*right].month_ordinal)
            })
    });

    let mut selected_month_ordinals = Vec::with_capacity(input.top_m_months);
    let mut score_sum = 0.0;
    for index in ranked.into_iter().take(input.top_m_months) {
        month_results[index].selected_in_top_m = true;
        selected_month_ordinals.push(month_results[index].month_ordinal);
        score_sum += month_results[index].reciprocal_affinity;
    }
    let directional_strength_score = score_sum / input.top_m_months as f64;
    if !directional_strength_score.is_finite() || directional_strength_score <= 0.0 {
        return Err(DirectionalSpectralTransferError::ScoreArithmeticOverflow);
    }

    Ok(DirectionalSpectralTransferReport {
        schema_version: DIRECTIONAL_SPECTRAL_TRANSFER_SCHEMA_VERSION.to_owned(),
        pair_id: input.pair_id.clone(),
        source_site_id: input.source_site_id.clone(),
        target_site_id: input.target_site_id.clone(),
        distance_metric: input.distance_metric,
        split_protocol: input.split_protocol,
        primary_cross_site_generalization_eligible: input
            .split_protocol
            .is_primary_cross_site_generalization(),
        source_split_manifest_sha256: input.source_split_manifest_sha256.clone(),
        target_split_manifest_sha256: input.target_split_manifest_sha256.clone(),
        feature_transform_sha256: input.feature_transform_sha256.clone(),
        input_channel_semantics: input.input_channel_semantics.clone(),
        phenophase_filter_ref: input.phenophase_filter_ref.clone(),
        top_m_months: input.top_m_months,
        shared_month_count: input.months.len(),
        selected_month_ordinals,
        directional_strength_score,
        months: month_results,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferOutcomeMetric {
    F1,
    PrecisionRecallAuc,
    RecallAtFixedReviewBudget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedTransferPair {
    pub pair_id: String,
    pub source_site_id: String,
    pub target_site_id: String,
    pub directional_strength_score: f64,
    /// Observed held-out transfer performance for the declared outcome metric.
    pub observed_transfer_value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DssCorrelationInput {
    pub outcome_metric: TransferOutcomeMetric,
    pub pairs: Vec<ObservedTransferPair>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteLeaveOutCorrelation {
    pub excluded_site_id: String,
    pub remaining_pair_count: usize,
    pub remaining_site_count: usize,
    pub spearman_rho: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DssCorrelationReport {
    pub schema_version: String,
    pub outcome_metric: TransferOutcomeMetric,
    pub directed_pair_count: usize,
    pub unique_site_count: usize,
    pub spearman_rho: Option<f64>,
    /// Directed pairs reuse sites and are dyadically dependent.
    pub directed_pairs_are_independent: bool,
    /// No naive pair-level confidence interval is emitted in v0.1.
    pub confidence_interval: Option<[f64; 2]>,
    pub site_leave_out: Vec<SiteLeaveOutCorrelation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DssCorrelationError {
    TooFewDirectedPairs,
    TooFewUniqueSites,
    EmptyPairId,
    EmptySiteId,
    SourceAndTargetMustDiffer,
    DuplicatePairId(String),
    DuplicateDirectedSitePair {
        source_site_id: String,
        target_site_id: String,
    },
    InvalidDssValue {
        pair_id: String,
    },
    InvalidObservedTransferValue {
        pair_id: String,
    },
}

impl fmt::Display for DssCorrelationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage DSS correlation error: {self:?}")
    }
}

impl std::error::Error for DssCorrelationError {}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut indices = (0..values.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| values[*left].total_cmp(&values[*right]));
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0usize;
    while start < indices.len() {
        let mut end = start + 1;
        while end < indices.len()
            && values[indices[start]]
                .total_cmp(&values[indices[end]])
                .is_eq()
        {
            end += 1;
        }
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
    for (x, y) in left.iter().zip(right) {
        let dx = x - left_mean;
        let dy = y - right_mean;
        covariance += dx * dy;
        left_variance += dx * dx;
        right_variance += dy * dy;
    }
    let denominator = (left_variance * right_variance).sqrt();
    (denominator.is_finite() && denominator > 0.0)
        .then_some((covariance / denominator).clamp(-1.0, 1.0))
}

fn spearman_correlation(pairs: &[ObservedTransferPair]) -> Option<f64> {
    if pairs.len() < 3 {
        return None;
    }
    let dss = pairs
        .iter()
        .map(|pair| pair.directional_strength_score)
        .collect::<Vec<_>>();
    let observed = pairs
        .iter()
        .map(|pair| pair.observed_transfer_value)
        .collect::<Vec<_>>();
    pearson_correlation(&average_ranks(&dss), &average_ranks(&observed))
}

fn validate_correlation_pairs(
    pairs: &[ObservedTransferPair],
) -> Result<BTreeSet<String>, DssCorrelationError> {
    if pairs.len() < 6 {
        return Err(DssCorrelationError::TooFewDirectedPairs);
    }
    let mut pair_ids = BTreeSet::new();
    let mut directed_site_pairs = BTreeSet::new();
    let mut sites = BTreeSet::new();

    for pair in pairs {
        if pair.pair_id.trim().is_empty() {
            return Err(DssCorrelationError::EmptyPairId);
        }
        if !pair_ids.insert(pair.pair_id.as_str()) {
            return Err(DssCorrelationError::DuplicatePairId(pair.pair_id.clone()));
        }
        if pair.source_site_id.trim().is_empty() || pair.target_site_id.trim().is_empty() {
            return Err(DssCorrelationError::EmptySiteId);
        }
        if pair.source_site_id == pair.target_site_id {
            return Err(DssCorrelationError::SourceAndTargetMustDiffer);
        }
        if !directed_site_pairs.insert((pair.source_site_id.as_str(), pair.target_site_id.as_str()))
        {
            return Err(DssCorrelationError::DuplicateDirectedSitePair {
                source_site_id: pair.source_site_id.clone(),
                target_site_id: pair.target_site_id.clone(),
            });
        }
        if !pair.directional_strength_score.is_finite() || pair.directional_strength_score <= 0.0 {
            return Err(DssCorrelationError::InvalidDssValue {
                pair_id: pair.pair_id.clone(),
            });
        }
        if !pair.observed_transfer_value.is_finite()
            || !(0.0..=1.0).contains(&pair.observed_transfer_value)
        {
            return Err(DssCorrelationError::InvalidObservedTransferValue {
                pair_id: pair.pair_id.clone(),
            });
        }
        sites.insert(pair.source_site_id.clone());
        sites.insert(pair.target_site_id.clone());
    }

    if sites.len() < 4 {
        return Err(DssCorrelationError::TooFewUniqueSites);
    }
    Ok(sites)
}

/// Measure how well DSS ranks observed transfer performance across directed
/// site pairs. Requires at least six pairs and four unique sites. Reports
/// leave-one-site-out sensitivity; no pair-IID confidence interval is claimed.
pub fn evaluate_dss_transfer_correlation(
    input: &DssCorrelationInput,
) -> Result<DssCorrelationReport, DssCorrelationError> {
    let sites = validate_correlation_pairs(&input.pairs)?;
    let spearman_rho = spearman_correlation(&input.pairs);

    let mut site_leave_out = Vec::with_capacity(sites.len());
    for excluded_site_id in &sites {
        let remaining = input
            .pairs
            .iter()
            .filter(|pair| {
                pair.source_site_id != *excluded_site_id && pair.target_site_id != *excluded_site_id
            })
            .cloned()
            .collect::<Vec<_>>();
        let remaining_sites = remaining
            .iter()
            .flat_map(|pair| [pair.source_site_id.clone(), pair.target_site_id.clone()])
            .collect::<BTreeSet<_>>();
        site_leave_out.push(SiteLeaveOutCorrelation {
            excluded_site_id: excluded_site_id.clone(),
            remaining_pair_count: remaining.len(),
            remaining_site_count: remaining_sites.len(),
            spearman_rho: spearman_correlation(&remaining),
        });
    }

    Ok(DssCorrelationReport {
        schema_version: DSS_VALIDATION_SCHEMA_VERSION.to_owned(),
        outcome_metric: input.outcome_metric,
        directed_pair_count: input.pairs.len(),
        unique_site_count: sites.len(),
        spearman_rho,
        directed_pairs_are_independent: false,
        confidence_interval: None,
        site_leave_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(year: i32, month: u8) -> i64 {
        i64::from(year) * 12 + i64::from(month) - 1
    }

    fn dss_input(
        source: &str,
        target: &str,
        distances: &[(u8, f64, f64)],
    ) -> DirectionalSpectralTransferInput {
        DirectionalSpectralTransferInput {
            pair_id: format!("{source}->{target}"),
            source_site_id: source.into(),
            target_site_id: target.into(),
            distance_metric: SpectralDistanceMetric::Euclidean,
            split_protocol: TransferSplitProtocol::HeldOutSite,
            source_split_manifest_sha256: "a".repeat(64),
            target_split_manifest_sha256: "b".repeat(64),
            feature_transform_sha256: "c".repeat(64),
            input_channel_semantics: "Sentinel-2 L2A B02/B03/B04/B08 surface reflectance".into(),
            phenophase_filter_ref: "urn:synthetic:ndvi-phenophase-filter".into(),
            top_m_months: 2,
            minimum_shared_months: 3,
            months: distances
                .iter()
                .map(|(month_id, aa, ah)| SpectralMonthlyDistances {
                    month_ordinal: month(2020, *month_id),
                    source_archaeological_sample_count: 20,
                    target_archaeological_sample_count: 18,
                    target_vegetated_control_sample_count: 25,
                    archaeological_to_archaeological_distance: *aa,
                    archaeological_to_control_distance: *ah,
                })
                .collect(),
        }
    }

    #[test]
    fn published_top_m_reciprocal_strength_formula_is_computed_exactly() {
        let input = dss_input(
            "site-a",
            "site-b",
            &[(3, 1.0, 4.0), (4, 2.0, 4.0), (5, 3.0, 3.0)],
        );
        let report = evaluate_directional_spectral_transfer(&input).unwrap();
        assert_eq!(
            report.selected_month_ordinals,
            vec![month(2020, 3), month(2020, 4)]
        );
        assert!((report.directional_strength_score - 3.0).abs() < 1e-12);
        assert_eq!(
            report
                .months
                .iter()
                .filter(|item| item.selected_in_top_m)
                .count(),
            2
        );
        assert!(report.primary_cross_site_generalization_eligible);
    }

    #[test]
    fn source_to_target_and_target_to_source_are_distinct_results() {
        let forward = dss_input(
            "site-a",
            "site-b",
            &[(3, 1.0, 4.0), (4, 2.0, 4.0), (5, 3.0, 3.0)],
        );
        let reverse = dss_input(
            "site-b",
            "site-a",
            &[(3, 3.0, 3.0), (4, 2.0, 4.0), (5, 1.0, 4.0)],
        );
        let f = evaluate_directional_spectral_transfer(&forward).unwrap();
        let r = evaluate_directional_spectral_transfer(&reverse).unwrap();
        assert_ne!(f.directional_strength_score, r.directional_strength_score);
        assert_eq!(f.source_site_id, "site-a");
        assert_eq!(r.source_site_id, "site-b");
    }

    #[test]
    fn tied_month_ratios_use_earliest_month_as_deterministic_tiebreak() {
        let input = dss_input(
            "site-a",
            "site-b",
            &[(3, 1.0, 2.0), (4, 2.0, 4.0), (5, 1.0, 2.0)],
        );
        let report = evaluate_directional_spectral_transfer(&input).unwrap();
        assert_eq!(
            report.selected_month_ordinals,
            vec![month(2020, 3), month(2020, 4)]
        );
    }

    #[test]
    fn pixel_random_split_is_retained_but_not_primary_generalization_eligible() {
        let mut input = dss_input(
            "site-a",
            "site-b",
            &[(3, 1.0, 4.0), (4, 2.0, 4.0), (5, 3.0, 3.0)],
        );
        input.split_protocol = TransferSplitProtocol::PixelRandomInterpolation;
        let report = evaluate_directional_spectral_transfer(&input).unwrap();
        assert!(!report.primary_cross_site_generalization_eligible);
    }

    #[test]
    fn zero_non_finite_and_empty_distances_fail_closed() {
        let mut input = dss_input("site-a", "site-b", &[(3, 1.0, 4.0), (4, 2.0, 4.0), (5, 3.0, 3.0)]);
        input.months[0].archaeological_to_archaeological_distance = 0.0;
        assert!(matches!(
            evaluate_directional_spectral_transfer(&input),
            Err(DirectionalSpectralTransferError::InvalidDistance { .. })
        ));
        input.months[0].archaeological_to_archaeological_distance = f64::INFINITY;
        assert!(matches!(
            evaluate_directional_spectral_transfer(&input),
            Err(DirectionalSpectralTransferError::InvalidDistance { .. })
        ));
        input.months[0].archaeological_to_archaeological_distance = 1.0;
        input.months[0].source_archaeological_sample_count = 0;
        assert!(matches!(
            evaluate_directional_spectral_transfer(&input),
            Err(DirectionalSpectralTransferError::EmptySourceArchaeologicalCluster { .. })
        ));
    }

    #[test]
    fn insufficient_month_coverage_duplicate_months_and_bad_digests_fail_closed() {
        let mut input = dss_input("site-a", "site-b", &[(3, 1.0, 4.0), (4, 2.0, 4.0)]);
        assert!(matches!(
            evaluate_directional_spectral_transfer(&input),
            Err(DirectionalSpectralTransferError::InsufficientSharedMonths { .. })
        ));
        input = dss_input(
            "site-a",
            "site-b",
            &[(3, 1.0, 4.0), (4, 2.0, 4.0), (5, 3.0, 3.0)],
        );
        input.months[2].month_ordinal = input.months[1].month_ordinal;
        assert_eq!(
            evaluate_directional_spectral_transfer(&input),
            Err(DirectionalSpectralTransferError::MonthsNotStrictlyIncreasing)
        );
        input.months[2].month_ordinal = month(2020, 5);
        input.target_split_manifest_sha256 = "invalid".into();
        assert_eq!(
            evaluate_directional_spectral_transfer(&input),
            Err(DirectionalSpectralTransferError::InvalidTargetSplitDigest)
        );
    }

    fn correlation_input() -> DssCorrelationInput {
        let pairs = vec![
            ("a", "b", 0.9, 0.9),
            ("b", "a", 0.8, 0.8),
            ("a", "c", 0.7, 0.7),
            ("c", "a", 0.6, 0.6),
            ("b", "c", 0.5, 0.5),
            ("c", "b", 0.4, 0.4),
            ("a", "d", 0.3, 0.3),
            ("d", "a", 0.2, 0.2),
            ("b", "d", 0.15, 0.15),
            ("d", "b", 0.1, 0.1),
            ("c", "d", 0.08, 0.08),
            ("d", "c", 0.05, 0.05),
        ];
        DssCorrelationInput {
            outcome_metric: TransferOutcomeMetric::PrecisionRecallAuc,
            pairs: pairs
                .iter()
                .enumerate()
                .map(
                    |(index, (source, target, dss, observed))| ObservedTransferPair {
                        pair_id: format!("pair-{index}"),
                        source_site_id: (*source).into(),
                        target_site_id: (*target).into(),
                        directional_strength_score: *dss,
                        observed_transfer_value: *observed,
                    },
                )
                .collect(),
        }
    }

    #[test]
    fn dss_validation_uses_spearman_and_reports_site_reuse() {
        let report = evaluate_dss_transfer_correlation(&correlation_input()).unwrap();
        assert_eq!(report.directed_pair_count, 12);
        assert_eq!(report.unique_site_count, 4);
        assert_eq!(report.spearman_rho, Some(1.0));
        assert!(!report.directed_pairs_are_independent);
        assert_eq!(report.confidence_interval, None);
        assert_eq!(report.site_leave_out.len(), 4);
        assert!(
            report
                .site_leave_out
                .iter()
                .all(|item| item.spearman_rho == Some(1.0))
        );
    }

    #[test]
    fn spearman_ranks_average_ties_instead_of_ordering_by_input_position() {
        let pairs = vec![
            ("a", "b", 1.0, 0.2),
            ("b", "a", 1.0, 0.2),
            ("a", "c", 2.0, 0.8),
            ("c", "a", 2.0, 0.8),
            ("b", "c", 3.0, 0.9),
            ("c", "b", 3.0, 0.9),
        ];
        let input = DssCorrelationInput {
            outcome_metric: TransferOutcomeMetric::F1,
            pairs: pairs
                .iter()
                .enumerate()
                .map(
                    |(index, (source, target, dss, observed))| ObservedTransferPair {
                        pair_id: format!("tie-{index}"),
                        source_site_id: (*source).into(),
                        target_site_id: (*target).into(),
                        directional_strength_score: *dss,
                        observed_transfer_value: *observed,
                    },
                )
                .collect(),
        };
        let report = evaluate_dss_transfer_correlation(&input).unwrap();
        assert_eq!(report.spearman_rho, Some(1.0));
    }

    #[test]
    fn dss_correlation_requires_multiple_sites_and_unique_directed_pairs() {
        let mut input = correlation_input();
        input.pairs.truncate(6);
        assert_eq!(
            evaluate_dss_transfer_correlation(&input),
            Err(DssCorrelationError::TooFewUniqueSites)
        );

        let mut duplicate = correlation_input();
        duplicate.pairs[1].source_site_id = duplicate.pairs[0].source_site_id.clone();
        duplicate.pairs[1].target_site_id = duplicate.pairs[0].target_site_id.clone();
        assert!(matches!(
            evaluate_dss_transfer_correlation(&duplicate),
            Err(DssCorrelationError::DuplicateDirectedSitePair { .. })
        ));
    }
}
