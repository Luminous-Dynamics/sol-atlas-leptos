// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Paired human-only versus AI-assisted operational comparisons.
//!
//! This evaluator compares two completed HDB-B9 reports on the same frozen
//! challenge, reference contract, independent-unit set and assessed geography.
//! It is descriptive: no confidence intervals or population superiority claims
//! are computed. HDB-20 paired cluster inference remains a separate requirement.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::heritage_benchmark_metrics::EvaluationUnitKind;
use super::heritage_hard_negative::{
    HardNegativeClass, HardNegativeReport, HardNegativeUnitReport,
    HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION,
};

pub const HERITAGE_PAIRED_COMPARISON_SCHEMA_VERSION: &str =
    "luminous.heritage-paired-comparison.v0.1";
pub const MAX_PAIRED_COMPARISON_UNITS: usize = 100_000;
pub const PAIRED_COMPARISON_CLAIM_CEILING: &str =
    "paired_operational_comparison_only_no_population_superiority_or_archaeological_truth_claim";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenReportBinding {
    pub report_id: String,
    pub report_sha256: String,
    pub report_receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedComparisonPolicy {
    pub policy_id: String,
    pub policy_sha256: String,
    pub preregistration_receipt_ref: String,
    pub preregistration_receipt_sha256: String,
    /// Maximum tolerated absolute recall loss, expressed as a fraction.
    pub maximum_recall_loss_absolute: f64,
    /// Maximum tolerated increase in confirmed false positives per km².
    pub maximum_confirmed_false_positive_density_increase_per_km2: f64,
    /// Maximum tolerated increase in the unresolved-inclusive FP upper bound per km².
    pub maximum_upper_bound_false_positive_density_increase_per_km2: f64,
    /// Required candidate / human-only ratio in verified findings per expert-hour.
    pub minimum_verified_findings_per_expert_hour_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritagePairedComparisonInput {
    pub schema_version: String,
    pub comparison_id: String,
    pub challenge_id: String,
    pub challenge_manifest_sha256: String,
    pub reference_adjudication_protocol_sha256: String,
    pub evaluation_unit_kind: EvaluationUnitKind,
    pub baseline_binding: FrozenReportBinding,
    pub baseline_report: HardNegativeReport,
    pub candidate_binding: FrozenReportBinding,
    pub candidate_report: HardNegativeReport,
    pub policy: PairedComparisonPolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedUnitComparison {
    pub evaluation_unit_id: String,
    pub confirmed_constructed_wall_count: usize,
    pub adequately_assessed_area_km2: f64,
    pub baseline_recall: Option<f64>,
    pub candidate_recall: Option<f64>,
    pub recall_delta: Option<f64>,
    pub recall_non_inferior: Option<bool>,
    pub baseline_confirmed_false_positives_per_km2: f64,
    pub candidate_confirmed_false_positives_per_km2: f64,
    pub confirmed_false_positive_density_delta_per_km2: f64,
    pub confirmed_false_positive_density_non_inferior: bool,
    pub baseline_upper_bound_false_positives_per_km2: f64,
    pub candidate_upper_bound_false_positives_per_km2: f64,
    pub upper_bound_false_positive_density_delta_per_km2: f64,
    pub upper_bound_false_positive_density_non_inferior: bool,
    pub baseline_verified_findings_per_expert_hour: f64,
    pub candidate_verified_findings_per_expert_hour: f64,
    pub verified_findings_per_expert_hour_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritagePairedComparisonReport {
    pub schema_version: String,
    pub claim_ceiling: String,
    pub comparison_id: String,
    pub challenge_id: String,
    pub challenge_manifest_sha256: String,
    pub reference_adjudication_protocol_sha256: String,
    pub evaluation_unit_kind: EvaluationUnitKind,
    pub baseline_report_id: String,
    pub baseline_report_sha256: String,
    pub candidate_report_id: String,
    pub candidate_report_sha256: String,
    pub policy_id: String,
    pub policy_sha256: String,
    pub independent_evaluation_unit_count: usize,
    pub recall_eligible_unit_count: usize,
    pub baseline_micro_recall: Option<f64>,
    pub candidate_micro_recall: Option<f64>,
    pub micro_recall_delta: Option<f64>,
    pub baseline_macro_recall: Option<f64>,
    pub candidate_macro_recall: Option<f64>,
    pub macro_recall_delta: Option<f64>,
    pub recall_non_inferiority_pass: bool,
    pub baseline_confirmed_false_positives_per_km2: f64,
    pub candidate_confirmed_false_positives_per_km2: f64,
    pub confirmed_false_positive_density_delta_per_km2: f64,
    pub confirmed_false_positive_density_non_inferiority_pass: bool,
    pub baseline_upper_bound_false_positives_per_km2: f64,
    pub candidate_upper_bound_false_positives_per_km2: f64,
    pub upper_bound_false_positive_density_delta_per_km2: f64,
    pub upper_bound_false_positive_density_non_inferiority_pass: bool,
    pub baseline_verified_findings_per_expert_hour: f64,
    pub candidate_verified_findings_per_expert_hour: f64,
    /// None when human-only baseline productivity is zero.
    pub verified_findings_per_expert_hour_ratio: Option<f64>,
    /// None when the baseline productivity ratio is undefined.
    pub productivity_target_pass: Option<bool>,
    pub overall_operational_pass: bool,
    pub population_superiority_claim_authorized: bool,
    pub uncertainty_note: String,
    pub units: Vec<PairedUnitComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritagePairedComparisonError {
    WrongSchemaVersion,
    EmptyComparisonOrChallengeId,
    InvalidChallengeDigest,
    InvalidReportBinding(String),
    InvalidPolicy(String),
    InvalidPolicyReceipt,
    InvalidPolicyThreshold,
    ReportSchemaMismatch(String),
    ReportChallengeMismatch,
    ReportManifestMismatch,
    ReportReferenceProtocolMismatch,
    ReportUnitKindMismatch,
    DuplicateEvaluationUnit(String),
    MismatchedEvaluationUnitSet,
    MismatchedReferenceCoverage(String),
    InvalidReportMetrics(String),
    NoEvaluationUnits,
    TooManyEvaluationUnits,
    NoRecallEligibleUnits,
    InvalidAggregateArithmetic,
}

impl fmt::Display for HeritagePairedComparisonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage paired comparison error: {self:?}")
    }
}

impl std::error::Error for HeritagePairedComparisonError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn close(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1e-10 * left.abs().max(right.abs()).max(1.0)
}

fn validate_binding(
    binding: &FrozenReportBinding,
    label: &str,
) -> Result<(), HeritagePairedComparisonError> {
    if !non_empty(&binding.report_id)
        || !valid_sha256(&binding.report_sha256)
        || !non_empty(&binding.report_receipt_ref)
    {
        return Err(HeritagePairedComparisonError::InvalidReportBinding(
            label.to_owned(),
        ));
    }
    Ok(())
}

fn validate_policy(policy: &PairedComparisonPolicy) -> Result<(), HeritagePairedComparisonError> {
    if !non_empty(&policy.policy_id) {
        return Err(HeritagePairedComparisonError::InvalidPolicy(
            "empty policy id".into(),
        ));
    }
    if !valid_sha256(&policy.policy_sha256)
        || !non_empty(&policy.preregistration_receipt_ref)
        || !valid_sha256(&policy.preregistration_receipt_sha256)
    {
        return Err(HeritagePairedComparisonError::InvalidPolicyReceipt);
    }
    if !policy.maximum_recall_loss_absolute.is_finite()
        || !(0.0..=1.0).contains(&policy.maximum_recall_loss_absolute)
        || !policy
            .maximum_confirmed_false_positive_density_increase_per_km2
            .is_finite()
        || policy.maximum_confirmed_false_positive_density_increase_per_km2 < 0.0
        || !policy
            .maximum_upper_bound_false_positive_density_increase_per_km2
            .is_finite()
        || policy.maximum_upper_bound_false_positive_density_increase_per_km2 < 0.0
        || !policy
            .minimum_verified_findings_per_expert_hour_ratio
            .is_finite()
        || policy.minimum_verified_findings_per_expert_hour_ratio <= 0.0
    {
        return Err(HeritagePairedComparisonError::InvalidPolicyThreshold);
    }
    Ok(())
}

fn expected_optional_ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn expected_f1(precision: Option<f64>, recall: Option<f64>) -> Option<f64> {
    match (precision, recall) {
        (Some(p), Some(r)) if p + r > 0.0 => Some(2.0 * p * r / (p + r)),
        (Some(_), Some(_)) => Some(0.0),
        _ => None,
    }
}

fn validate_unit_metrics(
    unit: &HardNegativeUnitReport,
) -> Result<(), HeritagePairedComparisonError> {
    let label = unit.evaluation_unit_id.clone();
    if !non_empty(&label)
        || !unit.adequately_assessed_area_km2.is_finite()
        || unit.adequately_assessed_area_km2 <= 0.0
        || !unit.expert_review_hours.is_finite()
        || unit.expert_review_hours <= 0.0
    {
        return Err(HeritagePairedComparisonError::InvalidReportMetrics(
            label,
        ));
    }

    let metrics = &unit.resolved_prediction_metrics;
    if metrics.true_positive != unit.true_positive_count
        || metrics.false_positive != unit.false_positive_count
        || metrics.false_negative != unit.false_negative_count
        || unit
            .true_positive_count
            .checked_add(unit.false_negative_count)
            != Some(unit.confirmed_constructed_wall_count)
        || unit
            .true_positive_count
            .checked_add(unit.false_positive_count)
            != Some(unit.resolved_prediction_count)
        || unit
            .resolved_prediction_count
            .checked_add(unit.unresolved_prediction_count)
            != Some(unit.candidate_prediction_count)
    {
        return Err(HeritagePairedComparisonError::InvalidReportMetrics(
            label,
        ));
    }

    let precision = expected_optional_ratio(
        unit.true_positive_count,
        unit.true_positive_count
            .checked_add(unit.false_positive_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?,
    );
    let recall = expected_optional_ratio(
        unit.true_positive_count,
        unit.confirmed_constructed_wall_count,
    );
    let f1 = expected_f1(precision, recall);
    if !option_close(metrics.precision, precision)
        || !option_close(metrics.recall, recall)
        || !option_close(metrics.f1, f1)
    {
        return Err(HeritagePairedComparisonError::InvalidReportMetrics(label));
    }

    let confirmed_fp_density =
        unit.false_positive_count as f64 / unit.adequately_assessed_area_km2;
    let possible_fp_count = unit
        .false_positive_count
        .checked_add(unit.unresolved_prediction_count)
        .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
    let upper_fp_density = possible_fp_count as f64 / unit.adequately_assessed_area_km2;
    let verified_per_hour = unit.true_positive_count as f64 / unit.expert_review_hours;
    let confirmed_leads_per_hour = unit.false_positive_count as f64 / unit.expert_review_hours;
    let upper_leads_per_hour = possible_fp_count as f64 / unit.expert_review_hours;

    if !close(unit.confirmed_false_positives_per_km2, confirmed_fp_density)
        || !close(unit.upper_bound_false_positives_per_km2, upper_fp_density)
        || !close(unit.verified_findings_per_expert_hour, verified_per_hour)
        || !close(unit.confirmed_false_leads_per_expert_hour, confirmed_leads_per_hour)
        || !close(unit.upper_bound_false_leads_per_expert_hour, upper_leads_per_hour)
    {
        return Err(HeritagePairedComparisonError::InvalidReportMetrics(label));
    }
    Ok(())
}

fn option_close(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(a), Some(b)) => close(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn validate_report(
    report: &HardNegativeReport,
    expected_challenge_id: &str,
    expected_manifest_sha256: &str,
    expected_reference_protocol_sha256: &str,
    expected_unit_kind: EvaluationUnitKind,
    label: &str,
) -> Result<BTreeMap<String, &HardNegativeUnitReport>, HeritagePairedComparisonError> {
    if report.schema_version != HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION
        || report.claim_ceiling
            != "operational_hard_negative_scorecard_only_no_population_generalisation_or_archaeological_identification"
    {
        return Err(HeritagePairedComparisonError::ReportSchemaMismatch(
            label.to_owned(),
        ));
    }
    if report.challenge_id != expected_challenge_id {
        return Err(HeritagePairedComparisonError::ReportChallengeMismatch);
    }
    if report.challenge_manifest_sha256 != expected_manifest_sha256 {
        return Err(HeritagePairedComparisonError::ReportManifestMismatch);
    }
    if report.reference_adjudication_protocol_sha256 != expected_reference_protocol_sha256 {
        return Err(HeritagePairedComparisonError::ReportReferenceProtocolMismatch);
    }
    if report.evaluation_unit_kind != expected_unit_kind {
        return Err(HeritagePairedComparisonError::ReportUnitKindMismatch);
    }
    if report.units.is_empty() {
        return Err(HeritagePairedComparisonError::NoEvaluationUnits);
    }
    if report.units.len() > MAX_PAIRED_COMPARISON_UNITS {
        return Err(HeritagePairedComparisonError::TooManyEvaluationUnits);
    }
    if report.independent_evaluation_unit_count != report.units.len() {
        return Err(HeritagePairedComparisonError::InvalidReportMetrics(
            label.to_owned(),
        ));
    }

    let mut units = BTreeMap::new();
    let mut total_tp = 0usize;
    let mut total_fp = 0usize;
    let mut total_fn = 0usize;
    let mut total_area = 0.0;
    let mut total_hours = 0.0;
    let mut total_resolved = 0usize;
    let mut total_candidates = 0usize;
    let mut total_unresolved = 0usize;

    for unit in &report.units {
        validate_unit_metrics(unit)?;
        if unit.evaluation_unit_kind != expected_unit_kind {
            return Err(HeritagePairedComparisonError::ReportUnitKindMismatch);
        }
        if units
            .insert(unit.evaluation_unit_id.clone(), unit)
            .is_some()
        {
            return Err(HeritagePairedComparisonError::DuplicateEvaluationUnit(
                unit.evaluation_unit_id.clone(),
            ));
        }
        total_tp = total_tp
            .checked_add(unit.true_positive_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
        total_fp = total_fp
            .checked_add(unit.false_positive_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
        total_fn = total_fn
            .checked_add(unit.false_negative_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
        total_resolved = total_resolved
            .checked_add(unit.resolved_prediction_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
        total_candidates = total_candidates
            .checked_add(unit.candidate_prediction_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
        total_unresolved = total_unresolved
            .checked_add(unit.unresolved_prediction_count)
            .ok_or(HeritagePairedComparisonError::InvalidAggregateArithmetic)?;
        total_area += unit.adequately_assessed_area_km2;
        total_hours += unit.expert_review_hours;
    }

    let micro = &report.micro_counts;
    let micro_precision = expected_optional_ratio(total_tp, total_tp + total_fp);
    let micro_recall = expected_optional_ratio(total_tp, total_tp + total_fn);
    let micro_f1 = expected_f1(micro_precision, micro_recall);
    let precision_mean = mean(report.units.iter().filter_map(|u| u.resolved_prediction_metrics.precision));
    let recall_mean = mean(report.units.iter().filter_map(|u| u.resolved_prediction_metrics.recall));
    let f1_mean = mean(report.units.iter().filter_map(|u| u.resolved_prediction_metrics.f1));
    let expected_worst = report
        .units
        .iter()
        .filter_map(|u| u.resolved_prediction_metrics.recall.map(|r| (r, u.evaluation_unit_id.as_str())))
        .min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1)));

    if micro.true_positive != total_tp
        || micro.false_positive != total_fp
        || micro.false_negative != total_fn
        || !option_close(micro.precision, micro_precision)
        || !option_close(micro.recall, micro_recall)
        || !option_close(micro.f1, micro_f1)
        || !option_close(report.macro_precision, precision_mean)
        || !option_close(report.macro_recall, recall_mean)
        || !option_close(report.macro_f1, f1_mean)
        || report.macro_precision_unit_count
            != report.units.iter().filter(|u| u.resolved_prediction_metrics.precision.is_some()).count()
        || report.macro_recall_unit_count
            != report.units.iter().filter(|u| u.resolved_prediction_metrics.recall.is_some()).count()
        || report.macro_f1_unit_count
            != report.units.iter().filter(|u| u.resolved_prediction_metrics.f1.is_some()).count()
        || !option_str_eq(report.worst_unit_recall_id.as_deref(), expected_worst.map(|v| v.1))
        || !option_close(report.worst_unit_recall, expected_worst.map(|v| v.0))
        || !close(report.adequately_assessed_area_km2, total_area)
        || !close(report.total_expert_review_hours, total_hours)
        || total_area <= 0.0
        || total_hours <= 0.0
        || !close(
            report.confirmed_false_positives_per_km2,
            total_fp as f64 / total_area,
        )
        || !close(
            report.upper_bound_false_positives_per_km2,
            (total_fp + total_unresolved) as f64 / total_area,
        )
        || !close(
            report.verified_findings_per_expert_hour,
            total_tp as f64 / total_hours,
        )
        || !close(
            report.confirmed_false_leads_per_expert_hour,
            total_fp as f64 / total_hours,
        )
        || !close(
            report.upper_bound_false_leads_per_expert_hour,
            (total_fp + total_unresolved) as f64 / total_hours,
        )
        || report.resolved_prediction_count != total_resolved
        || report.candidate_prediction_count != total_candidates
        || report.unresolved_prediction_count != total_unresolved
        || !option_close(
            report.candidate_resolution_fraction,
            (total_candidates > 0).then(|| total_resolved as f64 / total_candidates as f64),
        )
    {
        return Err(HeritagePairedComparisonError::InvalidReportMetrics(
            label.to_owned(),
        ));
    }

    Ok(units)
}

fn option_str_eq(left: Option<&str>, right: Option<&str>) -> bool {
    left == right
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut count = 0usize;
    let mut average = 0.0;
    for value in values {
        count += 1;
        average += (value - average) / count as f64;
    }
    (count > 0).then_some(average)
}

fn class_reference_counts(unit: &HardNegativeUnitReport) -> BTreeMap<HardNegativeClass, usize> {
    unit.hard_negative_classes
        .iter()
        .map(|class| (class.class, class.reference_object_count))
        .collect()
}

fn validate_matched_references(
    baseline: &HardNegativeUnitReport,
    candidate: &HardNegativeUnitReport,
) -> Result<(), HeritagePairedComparisonError> {
    if baseline.evaluation_unit_kind != candidate.evaluation_unit_kind
        || baseline.confirmed_constructed_wall_count != candidate.confirmed_constructed_wall_count
        || baseline.ambiguous_reference_count != candidate.ambiguous_reference_count
        || baseline.not_observable_reference_count != candidate.not_observable_reference_count
        || baseline.not_assessed_reference_count != candidate.not_assessed_reference_count
        || baseline.adequately_assessed_area_km2.to_bits()
            != candidate.adequately_assessed_area_km2.to_bits()
        || class_reference_counts(baseline) != class_reference_counts(candidate)
    {
        return Err(HeritagePairedComparisonError::MismatchedReferenceCoverage(
            baseline.evaluation_unit_id.clone(),
        ));
    }
    Ok(())
}

/// Compares two frozen HDB-B9 scorecards on exactly the same independent
/// evaluation units and adjudicated reference coverage. This function emits
/// preregistered descriptive pass/fail gates, but no sampling uncertainty.
pub fn evaluate_heritage_paired_comparison(
    input: &HeritagePairedComparisonInput,
) -> Result<HeritagePairedComparisonReport, HeritagePairedComparisonError> {
    if input.schema_version != HERITAGE_PAIRED_COMPARISON_SCHEMA_VERSION {
        return Err(HeritagePairedComparisonError::WrongSchemaVersion);
    }
    if !non_empty(&input.comparison_id) || !non_empty(&input.challenge_id) {
        return Err(HeritagePairedComparisonError::EmptyComparisonOrChallengeId);
    }
    if !valid_sha256(&input.challenge_manifest_sha256)
        || !valid_sha256(&input.reference_adjudication_protocol_sha256)
    {
        return Err(HeritagePairedComparisonError::InvalidChallengeDigest);
    }
    validate_binding(&input.baseline_binding, "baseline")?;
    validate_binding(&input.candidate_binding, "candidate")?;
    validate_policy(&input.policy)?;

    let baseline = validate_report(
        &input.baseline_report,
        &input.challenge_id,
        &input.challenge_manifest_sha256,
        &input.reference_adjudication_protocol_sha256,
        input.evaluation_unit_kind,
        "baseline",
    )?;
    let candidate = validate_report(
        &input.candidate_report,
        &input.challenge_id,
        &input.challenge_manifest_sha256,
        &input.reference_adjudication_protocol_sha256,
        input.evaluation_unit_kind,
        "candidate",
    )?;

    if baseline.len() != candidate.len()
        || baseline.keys().ne(candidate.keys())
    {
        return Err(HeritagePairedComparisonError::MismatchedEvaluationUnitSet);
    }

    let mut unit_comparisons = Vec::with_capacity(baseline.len());
    let mut recall_eligible_unit_count = 0usize;
    let mut all_unit_recall_non_inferior = true;
    let mut all_unit_confirmed_fp_non_inferior = true;
    let mut all_unit_upper_fp_non_inferior = true;

    for (unit_id, baseline_unit) in &baseline {
        let candidate_unit = candidate
            .get(unit_id)
            .expect("identical paired evaluation-unit sets");
        validate_matched_references(baseline_unit, candidate_unit)?;

        let baseline_recall = baseline_unit.resolved_prediction_metrics.recall;
        let candidate_recall = candidate_unit.resolved_prediction_metrics.recall;
        let recall_delta = match (baseline_recall, candidate_recall) {
            (Some(base), Some(next)) => Some(next - base),
            (None, None) => None,
            _ => {
                return Err(HeritagePairedComparisonError::InvalidReportMetrics(
                    unit_id.clone(),
                ));
            }
        };
        let recall_non_inferior = recall_delta.map(|delta| {
            delta >= -input.policy.maximum_recall_loss_absolute
        });
        if let Some(passed) = recall_non_inferior {
            recall_eligible_unit_count += 1;
            all_unit_recall_non_inferior &= passed;
        }

        let confirmed_fp_delta = candidate_unit.confirmed_false_positives_per_km2
            - baseline_unit.confirmed_false_positives_per_km2;
        let upper_fp_delta = candidate_unit.upper_bound_false_positives_per_km2
            - baseline_unit.upper_bound_false_positives_per_km2;
        let confirmed_fp_non_inferior = confirmed_fp_delta
            <= input
                .policy
                .maximum_confirmed_false_positive_density_increase_per_km2;
        let upper_fp_non_inferior = upper_fp_delta
            <= input
                .policy
                .maximum_upper_bound_false_positive_density_increase_per_km2;
        all_unit_confirmed_fp_non_inferior &= confirmed_fp_non_inferior;
        all_unit_upper_fp_non_inferior &= upper_fp_non_inferior;

        let baseline_productivity = baseline_unit.verified_findings_per_expert_hour;
        let candidate_productivity = candidate_unit.verified_findings_per_expert_hour;
        let productivity_ratio = (baseline_productivity > 0.0)
            .then(|| candidate_productivity / baseline_productivity);

        unit_comparisons.push(PairedUnitComparison {
            evaluation_unit_id: unit_id.clone(),
            confirmed_constructed_wall_count: baseline_unit.confirmed_constructed_wall_count,
            adequately_assessed_area_km2: baseline_unit.adequately_assessed_area_km2,
            baseline_recall,
            candidate_recall,
            recall_delta,
            recall_non_inferior,
            baseline_confirmed_false_positives_per_km2: baseline_unit
                .confirmed_false_positives_per_km2,
            candidate_confirmed_false_positives_per_km2: candidate_unit
                .confirmed_false_positives_per_km2,
            confirmed_false_positive_density_delta_per_km2: confirmed_fp_delta,
            confirmed_false_positive_density_non_inferior: confirmed_fp_non_inferior,
            baseline_upper_bound_false_positives_per_km2: baseline_unit
                .upper_bound_false_positives_per_km2,
            candidate_upper_bound_false_positives_per_km2: candidate_unit
                .upper_bound_false_positives_per_km2,
            upper_bound_false_positive_density_delta_per_km2: upper_fp_delta,
            upper_bound_false_positive_density_non_inferior: upper_fp_non_inferior,
            baseline_verified_findings_per_expert_hour: baseline_productivity,
            candidate_verified_findings_per_expert_hour: candidate_productivity,
            verified_findings_per_expert_hour_ratio: productivity_ratio,
        });
    }

    if recall_eligible_unit_count == 0 {
        return Err(HeritagePairedComparisonError::NoRecallEligibleUnits);
    }

    let baseline_micro_recall = input.baseline_report.micro_counts.recall;
    let candidate_micro_recall = input.candidate_report.micro_counts.recall;
    let micro_recall_delta = match (baseline_micro_recall, candidate_micro_recall) {
        (Some(base), Some(next)) => Some(next - base),
        _ => return Err(HeritagePairedComparisonError::NoRecallEligibleUnits),
    };
    let baseline_macro_recall = mean(
        input
            .baseline_report
            .units
            .iter()
            .filter_map(|unit| unit.resolved_prediction_metrics.recall),
    );
    let candidate_macro_recall = mean(
        input
            .candidate_report
            .units
            .iter()
            .filter_map(|unit| unit.resolved_prediction_metrics.recall),
    );
    let macro_recall_delta = match (baseline_macro_recall, candidate_macro_recall) {
        (Some(base), Some(next)) => Some(next - base),
        _ => None,
    };
    let recall_non_inferiority_pass = all_unit_recall_non_inferior
        && micro_recall_delta >= -input.policy.maximum_recall_loss_absolute
        && macro_recall_delta
            .is_some_and(|delta| delta >= -input.policy.maximum_recall_loss_absolute);

    let baseline_confirmed_fp = input
        .baseline_report
        .confirmed_false_positives_per_km2;
    let candidate_confirmed_fp = input
        .candidate_report
        .confirmed_false_positives_per_km2;
    let confirmed_fp_delta = candidate_confirmed_fp - baseline_confirmed_fp;
    let confirmed_fp_pass = all_unit_confirmed_fp_non_inferior
        && confirmed_fp_delta
            <= input
                .policy
                .maximum_confirmed_false_positive_density_increase_per_km2;

    let baseline_upper_fp = input
        .baseline_report
        .upper_bound_false_positives_per_km2;
    let candidate_upper_fp = input
        .candidate_report
        .upper_bound_false_positives_per_km2;
    let upper_fp_delta = candidate_upper_fp - baseline_upper_fp;
    let upper_fp_pass = all_unit_upper_fp_non_inferior
        && upper_fp_delta
            <= input
                .policy
                .maximum_upper_bound_false_positive_density_increase_per_km2;

    let baseline_productivity = input
        .baseline_report
        .verified_findings_per_expert_hour;
    let candidate_productivity = input
        .candidate_report
        .verified_findings_per_expert_hour;
    let productivity_ratio = (baseline_productivity > 0.0)
        .then(|| candidate_productivity / baseline_productivity);
    let productivity_target_pass = productivity_ratio.map(|ratio| {
        ratio >= input.policy.minimum_verified_findings_per_expert_hour_ratio
    });

    let overall_operational_pass = recall_non_inferiority_pass
        && confirmed_fp_pass
        && upper_fp_pass
        && productivity_target_pass == Some(true);

    Ok(HeritagePairedComparisonReport {
        schema_version: HERITAGE_PAIRED_COMPARISON_SCHEMA_VERSION.to_owned(),
        claim_ceiling: PAIRED_COMPARISON_CLAIM_CEILING.to_owned(),
        comparison_id: input.comparison_id.clone(),
        challenge_id: input.challenge_id.clone(),
        challenge_manifest_sha256: input.challenge_manifest_sha256.clone(),
        reference_adjudication_protocol_sha256: input
            .reference_adjudication_protocol_sha256
            .clone(),
        evaluation_unit_kind: input.evaluation_unit_kind,
        baseline_report_id: input.baseline_binding.report_id.clone(),
        baseline_report_sha256: input.baseline_binding.report_sha256.clone(),
        candidate_report_id: input.candidate_binding.report_id.clone(),
        candidate_report_sha256: input.candidate_binding.report_sha256.clone(),
        policy_id: input.policy.policy_id.clone(),
        policy_sha256: input.policy.policy_sha256.clone(),
        independent_evaluation_unit_count: unit_comparisons.len(),
        recall_eligible_unit_count,
        baseline_micro_recall,
        candidate_micro_recall,
        micro_recall_delta,
        baseline_macro_recall,
        candidate_macro_recall,
        macro_recall_delta,
        recall_non_inferiority_pass,
        baseline_confirmed_false_positives_per_km2: baseline_confirmed_fp,
        candidate_confirmed_false_positives_per_km2: candidate_confirmed_fp,
        confirmed_false_positive_density_delta_per_km2: confirmed_fp_delta,
        confirmed_false_positive_density_non_inferiority_pass: confirmed_fp_pass,
        baseline_upper_bound_false_positives_per_km2: baseline_upper_fp,
        candidate_upper_bound_false_positives_per_km2: candidate_upper_fp,
        upper_bound_false_positive_density_delta_per_km2: upper_fp_delta,
        upper_bound_false_positive_density_non_inferiority_pass: upper_fp_pass,
        baseline_verified_findings_per_expert_hour: baseline_productivity,
        candidate_verified_findings_per_expert_hour: candidate_productivity,
        verified_findings_per_expert_hour_ratio: productivity_ratio,
        productivity_target_pass,
        overall_operational_pass,
        population_superiority_claim_authorized: false,
        uncertainty_note: "paired descriptive results only; no confidence intervals or population superiority claims are computed. Use a preregistered HDB-20 paired site/region/acquisition bootstrap and spatial effective-information analysis before generalisation.",
        units: unit_comparisons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heritage_hard_negative::{
        HardNegativeClassReport, HardNegativeUnitReport,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn binding(id: &str, byte: char) -> FrozenReportBinding {
        FrozenReportBinding {
            report_id: id.into(),
            report_sha256: digest(byte),
            report_receipt_ref: format!("urn:synthetic:report:{id}"),
        }
    }

    fn unit(
        id: &str,
        tp: usize,
        fp: usize,
        fns: usize,
        unresolved: usize,
        area: f64,
        hours: f64,
    ) -> HardNegativeUnitReport {
        let confirmed_wall_count = tp + fns;
        let resolved = tp + fp;
        let candidates = resolved + unresolved;
        let precision = expected_optional_ratio(tp, resolved);
        let recall = expected_optional_ratio(tp, confirmed_wall_count);
        let f1 = expected_f1(precision, recall);
        let upper_fp = fp + unresolved;
        HardNegativeUnitReport {
            evaluation_unit_id: id.into(),
            evaluation_unit_kind: EvaluationUnitKind::Site,
            adequately_assessed_area_km2: area,
            expert_review_hours: hours,
            confirmed_constructed_wall_count: confirmed_wall_count,
            true_positive_count: tp,
            false_positive_count: fp,
            false_negative_count: fns,
            resolved_prediction_metrics: CountMetrics {
                true_positive: tp,
                false_positive: fp,
                false_negative: fns,
                precision,
                recall,
                f1,
            },
            candidate_prediction_count: candidates,
            resolved_prediction_count: resolved,
            candidate_resolution_fraction: (candidates > 0)
                .then(|| resolved as f64 / candidates as f64),
            unresolved_prediction_count: unresolved,
            confirmed_false_positives_per_km2: fp as f64 / area,
            upper_bound_false_positives_per_km2: upper_fp as f64 / area,
            verified_findings_per_expert_hour: tp as f64 / hours,
            confirmed_false_leads_per_expert_hour: fp as f64 / hours,
            upper_bound_false_leads_per_expert_hour: upper_fp as f64 / hours,
            ambiguous_reference_count: 1,
            not_observable_reference_count: 1,
            not_assessed_reference_count: 1,
            hard_negative_classes: vec![HardNegativeClassReport {
                class: HardNegativeClass::ErodedRockLedge,
                reference_object_count: 2,
                false_positive_prediction_count: fp,
                false_positive_candidates_per_reference_object: Some(fp as f64 / 2.0),
            }],
        }
    }

    fn report(
        report_id: &str,
        tp1: usize,
        fp1: usize,
        fn1: usize,
        unresolved1: usize,
        hours1: f64,
        tp2: usize,
        fp2: usize,
        fn2: usize,
        unresolved2: usize,
        hours2: f64,
    ) -> HardNegativeReport {
        let units = vec![
            unit("site-01", tp1, fp1, fn1, unresolved1, 10.0, hours1),
            unit("site-02", tp2, fp2, fn2, unresolved2, 5.0, hours2),
        ];
        let tp = tp1 + tp2;
        let fp = fp1 + fp2;
        let fn_count = fn1 + fn2;
        let unresolved = unresolved1 + unresolved2;
        let area = 15.0;
        let hours = hours1 + hours2;
        let candidate_count = tp + fp + unresolved;
        let resolved_count = tp + fp;
        let micro_precision = expected_optional_ratio(tp, resolved_count);
        let micro_recall = expected_optional_ratio(tp, tp + fn_count);
        let micro_f1 = expected_f1(micro_precision, micro_recall);
        let mean_precision = mean(units.iter().filter_map(|u| u.resolved_prediction_metrics.precision));
        let mean_recall = mean(units.iter().filter_map(|u| u.resolved_prediction_metrics.recall));
        let mean_f1 = mean(units.iter().filter_map(|u| u.resolved_prediction_metrics.f1));
        let worst = units
            .iter()
            .filter_map(|u| u.resolved_prediction_metrics.recall.map(|r| (r, u.evaluation_unit_id.as_str())))
            .min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1)));
        HardNegativeReport {
            schema_version: HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION.into(),
            claim_ceiling: "operational_hard_negative_scorecard_only_no_population_generalisation_or_archaeological_identification".into(),
            challenge_id: "hdb-b9-paired-synthetic".into(),
            model_id: report_id.into(),
            confidence_threshold: 0.5,
            threshold_selection_receipt_ref: format!("urn:synthetic:threshold:{report_id}"),
            threshold_selection_receipt_sha256: digest(if report_id == "human" { 'a' } else { 'b' }),
            challenge_manifest_sha256: digest('c'),
            reference_adjudication_protocol_sha256: digest('d'),
            evaluation_unit_kind: EvaluationUnitKind::Site,
            independent_evaluation_unit_count: units.len(),
            micro_counts: CountMetrics {
                true_positive: tp,
                false_positive: fp,
                false_negative: fn_count,
                precision: micro_precision,
                recall: micro_recall,
                f1: micro_f1,
            },
            macro_precision: mean_precision,
            macro_precision_unit_count: units.iter().filter(|u| u.resolved_prediction_metrics.precision.is_some()).count(),
            macro_recall: mean_recall,
            macro_recall_unit_count: units.iter().filter(|u| u.resolved_prediction_metrics.recall.is_some()).count(),
            macro_f1: mean_f1,
            macro_f1_unit_count: units.iter().filter(|u| u.resolved_prediction_metrics.f1.is_some()).count(),
            worst_unit_recall_id: worst.map(|v| v.1.into()),
            worst_unit_recall: worst.map(|v| v.0),
            adequately_assessed_area_km2: area,
            total_expert_review_hours: hours,
            confirmed_false_positives_per_km2: fp as f64 / area,
            upper_bound_false_positives_per_km2: (fp + unresolved) as f64 / area,
            verified_findings_per_expert_hour: tp as f64 / hours,
            confirmed_false_leads_per_expert_hour: fp as f64 / hours,
            upper_bound_false_leads_per_expert_hour: (fp + unresolved) as f64 / hours,
            candidate_prediction_count: candidate_count,
            resolved_prediction_count: resolved_count,
            candidate_resolution_fraction: Some(resolved_count as f64 / candidate_count as f64),
            unresolved_prediction_count: unresolved,
            ambiguous_reference_count: 2,
            not_observable_reference_count: 2,
            not_assessed_reference_count: 2,
            hard_negative_classes: vec![HardNegativeClassReport {
                class: HardNegativeClass::ErodedRockLedge,
                reference_object_count: 4,
                false_positive_prediction_count: fp,
                false_positive_candidates_per_reference_object: Some(fp as f64 / 4.0),
            }],
            units,
            uncertainty_note: "synthetic paired test".into(),
        }
    }

    fn input() -> HeritagePairedComparisonInput {
        HeritagePairedComparisonInput {
            schema_version: HERITAGE_PAIRED_COMPARISON_SCHEMA_VERSION.into(),
            comparison_id: "comparison-001".into(),
            challenge_id: "hdb-b9-paired-synthetic".into(),
            challenge_manifest_sha256: digest('c'),
            reference_adjudication_protocol_sha256: digest('d'),
            evaluation_unit_kind: EvaluationUnitKind::Site,
            baseline_binding: binding("human-report", 'e'),
            baseline_report: report("human", 3, 2, 1, 1, 3.0, 2, 1, 0, 0, 3.0),
            candidate_binding: binding("ai-report", 'f'),
            candidate_report: report("ai", 3, 1, 1, 1, 1.0, 2, 0, 0, 0, 1.0),
            policy: PairedComparisonPolicy {
                policy_id: "paired-policy-001".into(),
                policy_sha256: digest('9'),
                preregistration_receipt_ref: "urn:synthetic:paired-policy".into(),
                preregistration_receipt_sha256: digest('8'),
                maximum_recall_loss_absolute: 0.02,
                maximum_confirmed_false_positive_density_increase_per_km2: 0.01,
                maximum_upper_bound_false_positive_density_increase_per_km2: 0.02,
                minimum_verified_findings_per_expert_hour_ratio: 2.0,
            },
        }
    }

    #[test]
    fn matched_candidate_passes_recall_false_positive_and_productivity_gates() {
        let report = evaluate_heritage_paired_comparison(&input()).unwrap();
        assert!(report.recall_non_inferiority_pass);
        assert!(report.confirmed_false_positive_density_non_inferiority_pass);
        assert!(report.upper_bound_false_positive_density_non_inferiority_pass);
        assert_eq!(report.verified_findings_per_expert_hour_ratio, Some(3.0));
        assert_eq!(report.productivity_target_pass, Some(true));
        assert!(report.overall_operational_pass);
        assert!(!report.population_superiority_claim_authorized);
        assert_eq!(report.independent_evaluation_unit_count, 2);
    }

    #[test]
    fn input_unit_order_does_not_change_paired_comparison() {
        let first = input();
        let mut second = first.clone();
        second.baseline_report.units.reverse();
        second.candidate_report.units.reverse();
        assert_eq!(
            evaluate_heritage_paired_comparison(&first).unwrap(),
            evaluate_heritage_paired_comparison(&second).unwrap()
        );
    }

    #[test]
    fn challenge_manifest_or_reference_protocol_mismatch_fails_closed() {
        let mut value = input();
        value.candidate_report.challenge_manifest_sha256 = digest('0');
        assert_eq!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::ReportManifestMismatch)
        );

        let mut value = input();
        value.candidate_report.reference_adjudication_protocol_sha256 = digest('0');
        assert_eq!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::ReportReferenceProtocolMismatch)
        );
    }

    #[test]
    fn mismatched_unit_set_or_reference_coverage_fails_closed() {
        let mut value = input();
        value.candidate_report.units.pop();
        value.candidate_report.independent_evaluation_unit_count = 1;
        assert_eq!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::MismatchedEvaluationUnitSet)
        );

        let mut value = input();
        value.candidate_report.units[0].confirmed_constructed_wall_count += 1;
        assert!(matches!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::InvalidReportMetrics(_))
                | Err(HeritagePairedComparisonError::MismatchedReferenceCoverage(_))
        ));
    }

    #[test]
    fn zero_human_baseline_productivity_yields_undefined_ratio() {
        let mut value = input();
        value.baseline_report.units[0].true_positive_count = 0;
        value.baseline_report.units[0].false_negative_count = 4;
        value.baseline_report.units[0].confirmed_constructed_wall_count = 4;
        value.baseline_report.units[0].resolved_prediction_metrics.true_positive = 0;
        value.baseline_report.units[0].resolved_prediction_metrics.false_negative = 4;
        value.baseline_report.units[0].resolved_prediction_metrics.precision = Some(0.0);
        value.baseline_report.units[0].resolved_prediction_metrics.recall = Some(0.0);
        value.baseline_report.units[0].resolved_prediction_metrics.f1 = Some(0.0);
        value.baseline_report.units[0].verified_findings_per_expert_hour = 0.0;
        value.baseline_report.units[0].candidate_prediction_count -= 3;
        value.baseline_report.units[0].resolved_prediction_count -= 3;
        value.baseline_report.units[0].candidate_resolution_fraction =
            Some(value.baseline_report.units[0].resolved_prediction_count as f64
                / value.baseline_report.units[0].candidate_prediction_count as f64);
        assert!(matches!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::InvalidReportMetrics(_))
        ));
    }

    #[test]
    fn invalid_policy_thresholds_and_report_receipts_fail_closed() {
        let mut value = input();
        value.policy.maximum_recall_loss_absolute = f64::NAN;
        assert_eq!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::InvalidPolicyThreshold)
        );

        let mut value = input();
        value.candidate_binding.report_sha256 = "bad".into();
        assert_eq!(
            evaluate_heritage_paired_comparison(&value),
            Err(HeritagePairedComparisonError::InvalidReportBinding("candidate".into()))
        );
    }
}
