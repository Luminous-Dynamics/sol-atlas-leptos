// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! HDB-B10: task-specific fitness-for-purpose evaluation.
//!
//! Conventional extraction metrics are necessary but do not prove that the
//! resulting dataset supports a downstream archaeological analysis. This
//! evaluator scores one preregistered intended use at a time: feature
//! extraction, site-level descriptor estimation, or between-site comparison.
//! It never upgrades a lower-level pass into a higher-level claim.
//!
//! Population uncertainty, spatial effective information and confidence
//! intervals remain HDB-20 responsibilities. This module reports deterministic
//! point estimates and per-unit gates only.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_FITNESS_SCHEMA_VERSION: &str = "luminous.heritage-fitness-for-purpose.v0.1";
pub const MINIMUM_UNITS_FOR_BETWEEN_UNIT_COMPARISON: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntendedUse {
    FeatureExtraction,
    SiteDescriptorEstimation,
    BetweenSiteComparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependentUnitKind {
    Site,
    Region,
    Acquisition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitAssessmentState {
    Evaluated,
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitnessThresholds {
    pub minimum_macro_precision: Option<f64>,
    pub minimum_macro_recall: Option<f64>,
    pub minimum_macro_f1: Option<f64>,
    /// Matched-detection geometry metric. It is deliberately separate from
    /// feature-count F1 and cannot replace the precision/recall gates.
    pub minimum_macro_matched_iou: Option<f64>,
    pub maximum_descriptor_mae: Option<f64>,
    pub maximum_absolute_descriptor_bias: Option<f64>,
    pub minimum_rank_spearman: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageFitnessProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub intended_use: IntendedUse,
    pub intended_claim: String,
    pub independent_unit_kind: IndependentUnitKind,
    pub minimum_independent_units: usize,
    pub descriptor_id: Option<String>,
    pub descriptor_units: Option<String>,
    pub thresholds: FitnessThresholds,
    pub metric_definition_sha256: String,
    pub reference_dataset_sha256: String,
    pub prediction_dataset_sha256: String,
    pub split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub method_build_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitnessUnitObservation {
    pub unit_id: String,
    pub assessment_state: UnitAssessmentState,
    /// Required when evaluated for feature extraction. All three counts must
    /// be present together; a not-assessed unit must leave them absent.
    pub true_positives: Option<usize>,
    pub false_positives: Option<usize>,
    pub false_negatives: Option<usize>,
    /// Mean IoU over matched detections for this independent unit.
    pub mean_matched_iou: Option<f64>,
    /// Reference/predicted downstream descriptor values (e.g. a masonry
    /// regularity or distribution statistic), not model probabilities.
    pub reference_descriptor: Option<f64>,
    pub predicted_descriptor: Option<f64>,
    pub not_assessed_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageFitnessInput {
    pub profile: HeritageFitnessProfile,
    pub units: Vec<FitnessUnitObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitnessUnitMetrics {
    pub unit_id: String,
    pub true_positives: Option<usize>,
    pub false_positives: Option<usize>,
    pub false_negatives: Option<usize>,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1: Option<f64>,
    pub mean_matched_iou: Option<f64>,
    pub reference_descriptor: Option<f64>,
    pub predicted_descriptor: Option<f64>,
    pub descriptor_absolute_error: Option<f64>,
    pub descriptor_signed_error: Option<f64>,
    pub assessment_state: UnitAssessmentState,
    pub not_assessed_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitnessGate {
    MinimumMacroPrecision,
    MinimumMacroRecall,
    MinimumMacroF1,
    MinimumMacroMatchedIou,
    MaximumDescriptorMae,
    MaximumAbsoluteDescriptorBias,
    MinimumRankSpearman,
    MinimumIndependentUnitCount,
    MinimumScoredDetectionUnitCount,
    AllUnitsHaveMatchedIou,
    AllUnitsHaveDescriptorValues,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitnessGateResult {
    pub gate: FitnessGate,
    pub passed: bool,
    pub observed_value: Option<f64>,
    pub threshold_value: Option<f64>,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitnessClaimCeiling {
    NotFitForDeclaredUse,
    FeatureExtractionOnly,
    SiteDescriptorEstimation,
    BetweenSiteComparison,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageFitnessReport {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub intended_use: IntendedUse,
    pub intended_claim: String,
    pub independent_unit_kind: IndependentUnitKind,
    pub descriptor_id: Option<String>,
    pub descriptor_units: Option<String>,
    pub submitted_unit_count: usize,
    pub evaluated_unit_count: usize,
    pub not_assessed_unit_count: usize,
    pub scored_detection_unit_count: usize,
    pub macro_precision: Option<f64>,
    pub macro_recall: Option<f64>,
    pub macro_f1: Option<f64>,
    pub pooled_precision: Option<f64>,
    pub pooled_recall: Option<f64>,
    pub pooled_f1: Option<f64>,
    pub macro_matched_iou: Option<f64>,
    pub descriptor_mean_absolute_error: Option<f64>,
    pub descriptor_mean_signed_error: Option<f64>,
    pub descriptor_rank_spearman: Option<f64>,
    pub gates: Vec<FitnessGateResult>,
    pub passed: bool,
    pub maximum_claim_ceiling: FitnessClaimCeiling,
    /// Always false: HDB-20 owns cluster-level intervals and effective
    /// independent information; this report is not a population CI.
    pub population_uncertainty_claimed: bool,
    pub metric_definition_sha256: String,
    pub reference_dataset_sha256: String,
    pub prediction_dataset_sha256: String,
    pub split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub method_build_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_ref: String,
    pub unit_metrics: Vec<FitnessUnitMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageFitnessError {
    WrongSchemaVersion,
    EmptyProfileId,
    EmptyIntendedClaim,
    EmptyDescriptorId,
    EmptyDescriptorUnits,
    InvalidDigest(&'static str),
    EmptyEvidenceReference(&'static str),
    InvalidMinimumIndependentUnitCount,
    InvalidThreshold(&'static str),
    ThresholdsDoNotMatchIntendedUse,
    DuplicateUnitId(String),
    EmptyUnitId,
    MissingUnitAssessmentReason(String),
    UnexpectedUnitAssessmentReason(String),
    PartialDetectionCounts(String),
    MissingDetectionCounts(String),
    MissingMatchedIou(String),
    InvalidMatchedIou(String),
    MissingDescriptor(String),
    InvalidDescriptor(String),
    NoUnits,
    AllUnitsNotAssessed,
}

impl fmt::Display for HeritageFitnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage fitness-for-purpose error: {self:?}")
    }
}

impl std::error::Error for HeritageFitnessError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn require_digest(value: &str, label: &'static str) -> Result<(), HeritageFitnessError> {
    if !valid_sha256(value) {
        return Err(HeritageFitnessError::InvalidDigest(label));
    }
    Ok(())
}

fn require_text(value: &str, label: &'static str) -> Result<(), HeritageFitnessError> {
    if !non_empty(value) {
        return Err(HeritageFitnessError::EmptyEvidenceReference(label));
    }
    Ok(())
}

fn valid_unit_interval(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn validate_threshold(
    value: Option<f64>,
    label: &'static str,
    minimum: f64,
    maximum: f64,
) -> Result<(), HeritageFitnessError> {
    if value.is_some_and(|value| !value.is_finite() || value < minimum || value > maximum) {
        return Err(HeritageFitnessError::InvalidThreshold(label));
    }
    Ok(())
}

fn validate_profile(profile: &HeritageFitnessProfile) -> Result<(), HeritageFitnessError> {
    if profile.schema_version != HERITAGE_FITNESS_SCHEMA_VERSION {
        return Err(HeritageFitnessError::WrongSchemaVersion);
    }
    require_text(&profile.profile_id, "profile_id")?;
    if !non_empty(&profile.intended_claim) {
        return Err(HeritageFitnessError::EmptyIntendedClaim);
    }
    if profile.minimum_independent_units == 0
        || (profile.intended_use == IntendedUse::BetweenSiteComparison
            && profile.minimum_independent_units < MINIMUM_UNITS_FOR_BETWEEN_UNIT_COMPARISON)
    {
        return Err(HeritageFitnessError::InvalidMinimumIndependentUnitCount);
    }
    require_digest(&profile.profile_sha256, "profile_sha256")?;
    require_digest(&profile.metric_definition_sha256, "metric_definition_sha256")?;
    require_digest(&profile.reference_dataset_sha256, "reference_dataset_sha256")?;
    require_digest(&profile.prediction_dataset_sha256, "prediction_dataset_sha256")?;
    require_digest(&profile.split_manifest_sha256, "split_manifest_sha256")?;
    require_digest(&profile.validation_receipt_sha256, "validation_receipt_sha256")?;
    require_digest(&profile.method_build_sha256, "method_build_sha256")?;
    require_text(&profile.validation_receipt_ref, "validation_receipt_ref")?;
    require_text(&profile.rights_decision_id, "rights_decision_id")?;
    require_text(&profile.rights_receipt_ref, "rights_receipt_ref")?;

    match profile.intended_use {
        IntendedUse::FeatureExtraction => {
            if profile.descriptor_id.is_some() || profile.descriptor_units.is_some() {
                return Err(HeritageFitnessError::ThresholdsDoNotMatchIntendedUse);
            }
            for (value, label) in [
                (
                    profile.thresholds.minimum_macro_precision,
                    "minimum_macro_precision",
                ),
                (
                    profile.thresholds.minimum_macro_recall,
                    "minimum_macro_recall",
                ),
                (profile.thresholds.minimum_macro_f1, "minimum_macro_f1"),
                (
                    profile.thresholds.minimum_macro_matched_iou,
                    "minimum_macro_matched_iou",
                ),
            ] {
                if value.is_none() {
                    return Err(HeritageFitnessError::ThresholdsDoNotMatchIntendedUse);
                }
                validate_threshold(value, label, 0.0, 1.0)?;
            }
            if profile.thresholds.maximum_descriptor_mae.is_some()
                || profile.thresholds.maximum_absolute_descriptor_bias.is_some()
                || profile.thresholds.minimum_rank_spearman.is_some()
            {
                return Err(HeritageFitnessError::ThresholdsDoNotMatchIntendedUse);
            }
        }
        IntendedUse::SiteDescriptorEstimation => {
            validate_descriptor_profile(profile)?;
            if profile.thresholds.maximum_descriptor_mae.is_none()
                || profile.thresholds.maximum_absolute_descriptor_bias.is_none()
                || profile.thresholds.minimum_macro_precision.is_some()
                || profile.thresholds.minimum_macro_recall.is_some()
                || profile.thresholds.minimum_macro_f1.is_some()
                || profile.thresholds.minimum_macro_matched_iou.is_some()
                || profile.thresholds.minimum_rank_spearman.is_some()
            {
                return Err(HeritageFitnessError::ThresholdsDoNotMatchIntendedUse);
            }
            validate_threshold(
                profile.thresholds.maximum_descriptor_mae,
                "maximum_descriptor_mae",
                0.0,
                f64::MAX,
            )?;
            validate_threshold(
                profile.thresholds.maximum_absolute_descriptor_bias,
                "maximum_absolute_descriptor_bias",
                0.0,
                f64::MAX,
            )?;
        }
        IntendedUse::BetweenSiteComparison => {
            validate_descriptor_profile(profile)?;
            if profile.thresholds.maximum_descriptor_mae.is_none()
                || profile.thresholds.maximum_absolute_descriptor_bias.is_none()
                || profile.thresholds.minimum_rank_spearman.is_none()
                || profile.thresholds.minimum_macro_precision.is_some()
                || profile.thresholds.minimum_macro_recall.is_some()
                || profile.thresholds.minimum_macro_f1.is_some()
                || profile.thresholds.minimum_macro_matched_iou.is_some()
            {
                return Err(HeritageFitnessError::ThresholdsDoNotMatchIntendedUse);
            }
            validate_threshold(
                profile.thresholds.maximum_descriptor_mae,
                "maximum_descriptor_mae",
                0.0,
                f64::MAX,
            )?;
            validate_threshold(
                profile.thresholds.maximum_absolute_descriptor_bias,
                "maximum_absolute_descriptor_bias",
                0.0,
                f64::MAX,
            )?;
            validate_threshold(
                profile.thresholds.minimum_rank_spearman,
                "minimum_rank_spearman",
                -1.0,
                1.0,
            )?;
        }
    }
    Ok(())
}

fn validate_descriptor_profile(
    profile: &HeritageFitnessProfile,
) -> Result<(), HeritageFitnessError> {
    if profile
        .descriptor_id
        .as_deref()
        .is_none_or(|value| !non_empty(value))
    {
        return Err(HeritageFitnessError::EmptyDescriptorId);
    }
    if profile
        .descriptor_units
        .as_deref()
        .is_none_or(|value| !non_empty(value))
    {
        return Err(HeritageFitnessError::EmptyDescriptorUnits);
    }
    Ok(())
}

fn validate_units(
    profile: &HeritageFitnessProfile,
    units: &[FitnessUnitObservation],
) -> Result<(), HeritageFitnessError> {
    if units.is_empty() {
        return Err(HeritageFitnessError::NoUnits);
    }
    let mut ids = BTreeSet::new();
    for unit in units {
        if !non_empty(&unit.unit_id) {
            return Err(HeritageFitnessError::EmptyUnitId);
        }
        if !ids.insert(unit.unit_id.clone()) {
            return Err(HeritageFitnessError::DuplicateUnitId(unit.unit_id.clone()));
        }
        match unit.assessment_state {
            UnitAssessmentState::NotAssessed => {
                if unit
                    .not_assessed_reason
                    .as_deref()
                    .is_none_or(|reason| !non_empty(reason))
                {
                    return Err(HeritageFitnessError::MissingUnitAssessmentReason(
                        unit.unit_id.clone(),
                    ));
                }
                if unit.true_positives.is_some()
                    || unit.false_positives.is_some()
                    || unit.false_negatives.is_some()
                    || unit.mean_matched_iou.is_some()
                    || unit.reference_descriptor.is_some()
                    || unit.predicted_descriptor.is_some()
                {
                    return Err(HeritageFitnessError::UnexpectedUnitAssessmentReason(
                        unit.unit_id.clone(),
                    ));
                }
            }
            UnitAssessmentState::Evaluated => {
                if unit.not_assessed_reason.is_some() {
                    return Err(HeritageFitnessError::UnexpectedUnitAssessmentReason(
                        unit.unit_id.clone(),
                    ));
                }
                let count_presence = [
                    unit.true_positives.is_some(),
                    unit.false_positives.is_some(),
                    unit.false_negatives.is_some(),
                ];
                if count_presence.iter().any(|present| *present)
                    && !count_presence.iter().all(|present| *present)
                {
                    return Err(HeritageFitnessError::PartialDetectionCounts(
                        unit.unit_id.clone(),
                    ));
                }
                if profile.intended_use == IntendedUse::FeatureExtraction
                    && !count_presence.iter().all(|present| *present)
                {
                    return Err(HeritageFitnessError::MissingDetectionCounts(
                        unit.unit_id.clone(),
                    ));
                }
                if let Some(iou) = unit.mean_matched_iou {
                    if !valid_unit_interval(iou) {
                        return Err(HeritageFitnessError::InvalidMatchedIou(
                            unit.unit_id.clone(),
                        ));
                    }
                } else if profile.intended_use == IntendedUse::FeatureExtraction {
                    return Err(HeritageFitnessError::MissingMatchedIou(unit.unit_id.clone()));
                }
                let descriptors_present = unit.reference_descriptor.is_some()
                    && unit.predicted_descriptor.is_some();
                if unit.reference_descriptor.is_some() != unit.predicted_descriptor.is_some() {
                    return Err(HeritageFitnessError::MissingDescriptor(
                        unit.unit_id.clone(),
                    ));
                }
                if matches!(
                    profile.intended_use,
                    IntendedUse::SiteDescriptorEstimation | IntendedUse::BetweenSiteComparison
                ) && !descriptors_present
                {
                    return Err(HeritageFitnessError::MissingDescriptor(
                        unit.unit_id.clone(),
                    ));
                }
                for descriptor in [unit.reference_descriptor, unit.predicted_descriptor]
                    .into_iter()
                    .flatten()
                {
                    if !descriptor.is_finite() {
                        return Err(HeritageFitnessError::InvalidDescriptor(
                            unit.unit_id.clone(),
                        ));
                    }
                }
                if let (Some(reference), Some(predicted)) =
                    (unit.reference_descriptor, unit.predicted_descriptor)
                {
                    if !(predicted - reference).is_finite() {
                        return Err(HeritageFitnessError::InvalidDescriptor(
                            unit.unit_id.clone(),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn precision(tp: usize, fp: usize) -> Option<f64> {
    let numerator = tp as f64;
    let denominator = numerator + fp as f64;
    (denominator > 0.0).then_some(numerator / denominator)
}

fn recall(tp: usize, r#fn: usize) -> Option<f64> {
    let numerator = tp as f64;
    let denominator = numerator + r#fn as f64;
    (denominator > 0.0).then_some(numerator / denominator)
}

fn f1(tp: usize, fp: usize, r#fn: usize) -> Option<f64> {
    let numerator = 2.0 * tp as f64;
    let denominator = numerator + fp as f64 + r#fn as f64;
    (denominator > 0.0).then_some(numerator / denominator)
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut count = 0usize;
    let mut current = 0.0;
    for value in values {
        count += 1;
        current += (value - current) / count as f64;
    }
    (count > 0).then_some(current)
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| values[*left].total_cmp(&values[*right]));
    let mut ranks = vec![0.0; values.len()];
    let mut cursor = 0usize;
    while cursor < order.len() {
        let mut end = cursor + 1;
        while end < order.len()
            && values[order[cursor]].total_cmp(&values[order[end]]).is_eq()
        {
            end += 1;
        }
        let average_rank = ((cursor + 1 + end) as f64) / 2.0;
        for position in cursor..end {
            ranks[order[position]] = average_rank;
        }
        cursor = end;
    }
    ranks
}

fn spearman_rank_correlation(reference: &[f64], predicted: &[f64]) -> Option<f64> {
    if reference.len() != predicted.len() || reference.len() < 2 {
        return None;
    }
    let reference_ranks = average_ranks(reference);
    let predicted_ranks = average_ranks(predicted);
    let reference_mean = mean(reference_ranks.iter().copied())?;
    let predicted_mean = mean(predicted_ranks.iter().copied())?;
    let covariance = reference_ranks
        .iter()
        .zip(&predicted_ranks)
        .map(|(left, right)| (left - reference_mean) * (right - predicted_mean))
        .sum::<f64>();
    let reference_variance = reference_ranks
        .iter()
        .map(|value| (value - reference_mean).powi(2))
        .sum::<f64>();
    let predicted_variance = predicted_ranks
        .iter()
        .map(|value| (value - predicted_mean).powi(2))
        .sum::<f64>();
    let denominator = (reference_variance * predicted_variance).sqrt();
    (denominator > 0.0).then_some((covariance / denominator).clamp(-1.0, 1.0))
}

fn unit_metrics(unit: &FitnessUnitObservation) -> FitnessUnitMetrics {
    let tp = unit.true_positives;
    let fp = unit.false_positives;
    let r#fn = unit.false_negatives;
    let descriptor_absolute_error = unit
        .reference_descriptor
        .zip(unit.predicted_descriptor)
        .map(|(reference, predicted)| (predicted - reference).abs());
    let descriptor_signed_error = unit
        .reference_descriptor
        .zip(unit.predicted_descriptor)
        .map(|(reference, predicted)| predicted - reference);
    let p = tp.zip(fp).and_then(|(tp, fp)| precision(tp, fp));
    let r = tp.zip(r#fn).and_then(|(tp, r#fn)| recall(tp, r#fn));
    let f = tp
        .zip(fp)
        .zip(r#fn)
        .and_then(|((tp, fp), r#fn)| f1(tp, fp, r#fn));
    FitnessUnitMetrics {
        unit_id: unit.unit_id.clone(),
        true_positives: tp,
        false_positives: fp,
        false_negatives: r#fn,
        precision: p,
        recall: r,
        f1: f,
        mean_matched_iou: unit.mean_matched_iou,
        reference_descriptor: unit.reference_descriptor,
        predicted_descriptor: unit.predicted_descriptor,
        descriptor_absolute_error,
        descriptor_signed_error,
        assessment_state: unit.assessment_state,
        not_assessed_reason: unit.not_assessed_reason.clone(),
    }
}

fn gate(
    gate: FitnessGate,
    observed: Option<f64>,
    threshold: Option<f64>,
    comparison_passed: impl FnOnce(f64, f64) -> bool,
    explanation: &str,
) -> FitnessGateResult {
    let passed = match (observed, threshold) {
        (Some(observed), Some(threshold)) => comparison_passed(observed, threshold),
        _ => false,
    };
    FitnessGateResult {
        gate,
        passed,
        observed_value: observed,
        threshold_value: threshold,
        explanation: explanation.to_owned(),
    }
}

/// Evaluates the output only against the single intended use preregistered in
/// the profile. The output is not a population confidence interval; use HDB-20
/// for independent-site/region/acquisition resampling and effective information.
pub fn evaluate_heritage_fitness(
    input: &HeritageFitnessInput,
) -> Result<HeritageFitnessReport, HeritageFitnessError> {
    validate_profile(&input.profile)?;
    validate_units(&input.profile, &input.units)?;

    let mut units = input.units.clone();
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    let evaluated_units = units
        .iter()
        .filter(|unit| unit.assessment_state == UnitAssessmentState::Evaluated)
        .collect::<Vec<_>>();
    if evaluated_units.is_empty() {
        return Err(HeritageFitnessError::AllUnitsNotAssessed);
    }
    let unit_metrics = units.iter().map(unit_metrics).collect::<Vec<_>>();

    let precisions = unit_metrics.iter().filter_map(|unit| unit.precision);
    let recalls = unit_metrics.iter().filter_map(|unit| unit.recall);
    let f1s = unit_metrics.iter().filter_map(|unit| unit.f1);
    let ious = unit_metrics
        .iter()
        .filter_map(|unit| unit.mean_matched_iou);
    let macro_precision = mean(precisions);
    let macro_recall = mean(recalls);
    let macro_f1 = mean(f1s);
    let macro_matched_iou = mean(ious);

    let total_tp = unit_metrics
        .iter()
        .filter_map(|unit| unit.true_positives)
        .map(|value| value as u128)
        .sum::<u128>();
    let total_fp = unit_metrics
        .iter()
        .filter_map(|unit| unit.false_positives)
        .map(|value| value as u128)
        .sum::<u128>();
    let total_fn = unit_metrics
        .iter()
        .filter_map(|unit| unit.false_negatives)
        .map(|value| value as u128)
        .sum::<u128>();
    let pooled_tp = total_tp as f64;
    let pooled_fp = total_fp as f64;
    let pooled_fn = total_fn as f64;
    let pooled_precision = (pooled_tp + pooled_fp > 0.0)
        .then_some(pooled_tp / (pooled_tp + pooled_fp));
    let pooled_recall =
        (pooled_tp + pooled_fn > 0.0).then_some(pooled_tp / (pooled_tp + pooled_fn));
    let pooled_f1 = (2.0 * pooled_tp + pooled_fp + pooled_fn > 0.0)
        .then_some(2.0 * pooled_tp / (2.0 * pooled_tp + pooled_fp + pooled_fn));

    let descriptor_mean_absolute_error = mean(
        unit_metrics
            .iter()
            .filter_map(|unit| unit.descriptor_absolute_error),
    );
    let descriptor_mean_signed_error = mean(
        unit_metrics
            .iter()
            .filter_map(|unit| unit.descriptor_signed_error),
    );
    let reference_descriptors = evaluated_units
        .iter()
        .filter_map(|unit| unit.reference_descriptor)
        .collect::<Vec<_>>();
    let predicted_descriptors = evaluated_units
        .iter()
        .filter_map(|unit| unit.predicted_descriptor)
        .collect::<Vec<_>>();
    let descriptor_rank_spearman = if reference_descriptors.len() == evaluated_units.len()
        && predicted_descriptors.len() == evaluated_units.len()
    {
        spearman_rank_correlation(&reference_descriptors, &predicted_descriptors)
    } else {
        None
    };

    let evaluated_count = evaluated_units.len();
    let not_assessed_count = units.len() - evaluated_count;
    let scored_detection_unit_count = unit_metrics
        .iter()
        .filter(|unit| unit.f1.is_some())
        .count();
    let mut gates = Vec::new();

    gates.push(gate(
        FitnessGate::MinimumIndependentUnitCount,
        Some(evaluated_count as f64),
        Some(input.profile.minimum_independent_units as f64),
        |observed, threshold| observed >= threshold,
        "Number of evaluated independent units must meet the preregistered minimum.",
    ));

    match input.profile.intended_use {
        IntendedUse::FeatureExtraction => {
            gates.push(gate(
                FitnessGate::MinimumMacroPrecision,
                macro_precision,
                input.profile.thresholds.minimum_macro_precision,
                |observed, threshold| observed >= threshold,
                "Macro precision must meet the feature-extraction threshold.",
            ));
            gates.push(gate(
                FitnessGate::MinimumMacroRecall,
                macro_recall,
                input.profile.thresholds.minimum_macro_recall,
                |observed, threshold| observed >= threshold,
                "Macro recall must meet the feature-extraction threshold.",
            ));
            gates.push(gate(
                FitnessGate::MinimumMacroF1,
                macro_f1,
                input.profile.thresholds.minimum_macro_f1,
                |observed, threshold| observed >= threshold,
                "Macro F1 must meet the feature-extraction threshold.",
            ));
            gates.push(gate(
                FitnessGate::MinimumMacroMatchedIou,
                macro_matched_iou,
                input.profile.thresholds.minimum_macro_matched_iou,
                |observed, threshold| observed >= threshold,
                "Matched-detection geometry IoU is a separate gate; it cannot substitute for F1.",
            ));
            gates.push(gate(
                FitnessGate::MinimumScoredDetectionUnitCount,
                Some(scored_detection_unit_count as f64),
                Some(input.profile.minimum_independent_units as f64),
                |observed, threshold| observed >= threshold,
                "Sufficient independent units must have a defined detection F1 score.",
            ));
        }
        IntendedUse::SiteDescriptorEstimation | IntendedUse::BetweenSiteComparison => {
            let descriptor_unit_count = evaluated_units
            .iter()
            .filter(|unit| unit.reference_descriptor.is_some() && unit.predicted_descriptor.is_some())
            .count();
            gates.push(gate(
                FitnessGate::AllUnitsHaveDescriptorValues,
                (descriptor_unit_count == evaluated_count).then_some(1.0),
                Some(1.0),
                |observed, threshold| observed >= threshold,
                "Every evaluated unit must have both reference and predicted descriptor values.",
            ));
            gates.push(gate(
                FitnessGate::MaximumDescriptorMae,
                descriptor_mean_absolute_error,
                input.profile.thresholds.maximum_descriptor_mae,
                |observed, threshold| observed <= threshold,
                "Mean absolute descriptor error must remain within the declared inferential tolerance.",
            ));
            gates.push(gate(
                FitnessGate::MaximumAbsoluteDescriptorBias,
                descriptor_mean_signed_error.map(f64::abs),
                input.profile.thresholds.maximum_absolute_descriptor_bias,
                |observed, threshold| observed <= threshold,
                "Absolute signed bias must remain within the declared inferential tolerance.",
            ));
            if input.profile.intended_use == IntendedUse::BetweenSiteComparison {
                gates.push(gate(
                    FitnessGate::MinimumRankSpearman,
                    descriptor_rank_spearman,
                    input.profile.thresholds.minimum_rank_spearman,
                    |observed, threshold| observed >= threshold,
                    "Across-unit descriptor ordering must meet the preregistered rank-agreement threshold.",
                ));
            }
        }
    }

    let passed = gates.iter().all(|result| result.passed);
    let maximum_claim_ceiling = if !passed {
        FitnessClaimCeiling::NotFitForDeclaredUse
    } else {
        match input.profile.intended_use {
            IntendedUse::FeatureExtraction => FitnessClaimCeiling::FeatureExtractionOnly,
            IntendedUse::SiteDescriptorEstimation => FitnessClaimCeiling::SiteDescriptorEstimation,
            IntendedUse::BetweenSiteComparison => FitnessClaimCeiling::BetweenSiteComparison,
        }
    };

    Ok(HeritageFitnessReport {
        schema_version: HERITAGE_FITNESS_SCHEMA_VERSION.to_owned(),
        profile_id: input.profile.profile_id.clone(),
        profile_sha256: input.profile.profile_sha256.clone(),
        intended_use: input.profile.intended_use,
        intended_claim: input.profile.intended_claim.clone(),
        independent_unit_kind: input.profile.independent_unit_kind,
        descriptor_id: input.profile.descriptor_id.clone(),
        descriptor_units: input.profile.descriptor_units.clone(),
        submitted_unit_count: units.len(),
        evaluated_unit_count: evaluated_count,
        not_assessed_unit_count: not_assessed_count,
        scored_detection_unit_count,
        macro_precision,
        macro_recall,
        macro_f1,
        pooled_precision,
        pooled_recall,
        pooled_f1,
        macro_matched_iou,
        descriptor_mean_absolute_error,
        descriptor_mean_signed_error,
        descriptor_rank_spearman,
        gates,
        passed,
        maximum_claim_ceiling,
        population_uncertainty_claimed: false,
        metric_definition_sha256: input.profile.metric_definition_sha256.clone(),
        reference_dataset_sha256: input.profile.reference_dataset_sha256.clone(),
        prediction_dataset_sha256: input.profile.prediction_dataset_sha256.clone(),
        split_manifest_sha256: input.profile.split_manifest_sha256.clone(),
        validation_receipt_ref: input.profile.validation_receipt_ref.clone(),
        validation_receipt_sha256: input.profile.validation_receipt_sha256.clone(),
        method_build_sha256: input.profile.method_build_sha256.clone(),
        rights_decision_id: input.profile.rights_decision_id.clone(),
        rights_receipt_ref: input.profile.rights_receipt_ref.clone(),
        unit_metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn base_profile(
        intended_use: IntendedUse,
        thresholds: FitnessThresholds,
    ) -> HeritageFitnessProfile {
        let descriptor_use = matches!(
            intended_use,
            IntendedUse::SiteDescriptorEstimation | IntendedUse::BetweenSiteComparison
        );
        HeritageFitnessProfile {
            schema_version: HERITAGE_FITNESS_SCHEMA_VERSION.into(),
            profile_id: "fitness-profile-001".into(),
            profile_sha256: digest('a'),
            intended_use,
            intended_claim: "Declared benchmark claim only.".into(),
            independent_unit_kind: IndependentUnitKind::Site,
            minimum_independent_units: if intended_use == IntendedUse::BetweenSiteComparison {
                3
            } else {
                1
            },
            descriptor_id: descriptor_use.then(|| "masonry_gini".into()),
            descriptor_units: descriptor_use.then(|| "dimensionless".into()),
            thresholds,
            metric_definition_sha256: digest('b'),
            reference_dataset_sha256: digest('c'),
            prediction_dataset_sha256: digest('d'),
            split_manifest_sha256: digest('e'),
            validation_receipt_ref: "urn:fitness:validation".into(),
            validation_receipt_sha256: digest('f'),
            method_build_sha256: digest('1'),
            rights_decision_id: "rights:fitness".into(),
            rights_receipt_ref: "urn:rights:fitness".into(),
        }
    }

    fn feature_profile() -> HeritageFitnessProfile {
        base_profile(
            IntendedUse::FeatureExtraction,
            FitnessThresholds {
                minimum_macro_precision: Some(0.8),
                minimum_macro_recall: Some(0.8),
                minimum_macro_f1: Some(0.8),
                minimum_macro_matched_iou: Some(0.8),
                maximum_descriptor_mae: None,
                maximum_absolute_descriptor_bias: None,
                minimum_rank_spearman: None,
            },
        )
    }

    fn feature_unit(unit_id: &str, tp: usize, fp: usize, r#fn: usize, iou: f64) -> FitnessUnitObservation {
        FitnessUnitObservation {
            unit_id: unit_id.into(),
            assessment_state: UnitAssessmentState::Evaluated,
            true_positives: Some(tp),
            false_positives: Some(fp),
            false_negatives: Some(r#fn),
            mean_matched_iou: Some(iou),
            reference_descriptor: None,
            predicted_descriptor: None,
            not_assessed_reason: None,
        }
    }

    fn descriptor_profile(use_case: IntendedUse) -> HeritageFitnessProfile {
        let thresholds = match use_case {
            IntendedUse::SiteDescriptorEstimation => FitnessThresholds {
                minimum_macro_precision: None,
                minimum_macro_recall: None,
                minimum_macro_f1: None,
                minimum_macro_matched_iou: None,
                maximum_descriptor_mae: Some(0.05),
                maximum_absolute_descriptor_bias: Some(0.03),
                minimum_rank_spearman: None,
            },
            IntendedUse::BetweenSiteComparison => FitnessThresholds {
                minimum_macro_precision: None,
                minimum_macro_recall: None,
                minimum_macro_f1: None,
                minimum_macro_matched_iou: None,
                maximum_descriptor_mae: Some(0.05),
                maximum_absolute_descriptor_bias: Some(0.03),
                minimum_rank_spearman: Some(0.8),
            },
            IntendedUse::FeatureExtraction => unreachable!(),
        };
        base_profile(use_case, thresholds)
    }

    fn descriptor_unit(
        unit_id: &str,
        reference: f64,
        predicted: f64,
    ) -> FitnessUnitObservation {
        FitnessUnitObservation {
            unit_id: unit_id.into(),
            assessment_state: UnitAssessmentState::Evaluated,
            true_positives: None,
            false_positives: None,
            false_negatives: None,
            mean_matched_iou: None,
            reference_descriptor: Some(reference),
            predicted_descriptor: Some(predicted),
            not_assessed_reason: None,
        }
    }

    #[test]
    fn high_matched_iou_does_not_hide_failed_detection_f1() {
        let input = HeritageFitnessInput {
            profile: feature_profile(),
            units: vec![
                feature_unit("site-a", 10, 10, 10, 0.93),
                feature_unit("site-b", 10, 10, 10, 0.94),
            ],
        };
        let report = evaluate_heritage_fitness(&input).unwrap();
        assert!((report.macro_matched_iou.unwrap() - 0.935).abs() < 1e-12);
        assert!((report.macro_f1.unwrap() - 0.5).abs() < 1e-12);
        assert!(!report.passed);
        assert_eq!(
            report.maximum_claim_ceiling,
            FitnessClaimCeiling::NotFitForDeclaredUse
        );
        assert!(report
            .gates
            .iter()
            .any(|gate| gate.gate == FitnessGate::MinimumMacroF1 && !gate.passed));
    }

    #[test]
    fn feature_extraction_pass_does_not_imply_site_descriptor_fitness() {
        let feature = HeritageFitnessInput {
            profile: feature_profile(),
            units: vec![
                feature_unit("site-a", 95, 5, 5, 0.95),
                feature_unit("site-b", 94, 6, 6, 0.94),
            ],
        };
        assert!(evaluate_heritage_fitness(&feature).unwrap().passed);

        let descriptor = HeritageFitnessInput {
            profile: descriptor_profile(IntendedUse::SiteDescriptorEstimation),
            units: vec![
                descriptor_unit("site-a", 0.2, 0.4),
                descriptor_unit("site-b", 0.3, 0.5),
            ],
        };
        let report = evaluate_heritage_fitness(&descriptor).unwrap();
        assert!(!report.passed);
        assert!(report.descriptor_mean_absolute_error.unwrap() > 0.05);
    }

    #[test]
    fn between_site_rank_reversal_fails_even_when_descriptor_error_is_bounded() {
        let input = HeritageFitnessInput {
            profile: descriptor_profile(IntendedUse::BetweenSiteComparison),
            units: vec![
                descriptor_unit("site-a", 0.20, 0.22),
                descriptor_unit("site-b", 0.21, 0.21),
                descriptor_unit("site-c", 0.22, 0.20),
            ],
        };
        // This fixture actually preserves ordering. Reverse it to create the
        // negative control while retaining bounded per-site absolute error.
        let mut input = input;
        input.units[0].predicted_descriptor = Some(0.3);
        input.units[1].predicted_descriptor = Some(0.2);
        input.units[2].predicted_descriptor = Some(0.1);
        let report = evaluate_heritage_fitness(&input).unwrap();
        assert_eq!(report.descriptor_rank_spearman, Some(-1.0));
        assert!(!report.passed);
    }

    #[test]
    fn not_assessed_units_remain_absent_not_zero_valued_negatives() {
        let mut not_assessed = feature_unit("site-b", 0, 0, 0, 0.0);
        not_assessed.assessment_state = UnitAssessmentState::NotAssessed;
        not_assessed.true_positives = None;
        not_assessed.false_positives = None;
        not_assessed.false_negatives = None;
        not_assessed.mean_matched_iou = None;
        not_assessed.not_assessed_reason = Some("no rights-cleared imagery".into());

        let report = evaluate_heritage_fitness(&HeritageFitnessInput {
            profile: feature_profile(),
            units: vec![feature_unit("site-a", 9, 1, 1, 0.9), not_assessed],
        })
        .unwrap();
        assert_eq!(report.evaluated_unit_count, 1);
        assert_eq!(report.not_assessed_unit_count, 1);
        assert_eq!(report.unit_metrics[1].assessment_state, UnitAssessmentState::NotAssessed);
        assert_eq!(report.unit_metrics[1].f1, None);
    }

    #[test]
    fn input_order_does_not_change_canonical_report() {
        let first = HeritageFitnessInput {
            profile: feature_profile(),
            units: vec![
                feature_unit("site-b", 8, 1, 2, 0.88),
                feature_unit("site-a", 9, 1, 1, 0.91),
            ],
        };
        let mut second = first.clone();
        second.units.reverse();
        assert_eq!(
            evaluate_heritage_fitness(&first).unwrap(),
            evaluate_heritage_fitness(&second).unwrap()
        );
    }

    #[test]
    fn pixel_or_stone_is_not_an_independent_resampling_unit() {
        let mut profile = feature_profile();
        profile.minimum_independent_units = 1;
        profile.independent_unit_kind = IndependentUnitKind::Site;
        // The type deliberately has no Pixel, Tile or Stone variant. Those
        // belong inside a declared independent unit, not in the site sample.
        assert_eq!(
            evaluate_heritage_fitness(&HeritageFitnessInput {
                profile,
                units: vec![feature_unit("site-a", 1, 0, 0, 1.0)],
            })
            .unwrap()
            .maximum_claim_ceiling,
            FitnessClaimCeiling::FeatureExtractionOnly
        );
    }

    #[test]
    fn tied_ranks_use_average_rank_and_constant_rank_vectors_are_undefined() {
        let spearman = spearman_rank_correlation(&[1.0, 1.0, 2.0], &[2.0, 2.0, 3.0]);
        assert_eq!(spearman, Some(1.0));
        assert_eq!(spearman_rank_correlation(&[1.0, 1.0], &[2.0, 3.0]), None);
    }

    #[test]
    fn between_site_comparison_requires_at_least_three_units() {
        let mut profile = descriptor_profile(IntendedUse::BetweenSiteComparison);
        profile.minimum_independent_units = 2;
        assert_eq!(
            evaluate_heritage_fitness(&HeritageFitnessInput {
                profile,
                units: vec![
                    descriptor_unit("site-a", 1.0, 1.0),
                    descriptor_unit("site-b", 2.0, 2.0),
                ],
            }),
            Err(HeritageFitnessError::InvalidMinimumIndependentUnitCount)
        );
    }
}
