// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Operational scorecards for rights-cleared archaeological hard-negative
//! challenges. This module consumes independently adjudicated counts; it does
//! not perform inference, polygon matching, reference adjudication, or spatial
//! measurement. Rates are valid only for the declared complete assessed area.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::heritage_benchmark_metrics::{CountMetrics, EvaluationProvenance, EvaluationUnitKind};

pub const HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION: &str =
    "luminous.heritage-hard-negative.v0.1";
pub const MAX_HARD_NEGATIVE_EVALUATION_UNITS: usize = 100_000;
pub const MAX_EXPERT_REVIEW_HOURS: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardNegativeClass {
    ErodedRockLedge,
    BedrockScarp,
    GeologicalLineament,
    GullyOrErosionChannel,
    TerraceEdge,
    RoadCutOrTrack,
    ModernRetainingWall,
    LivestockKraalOrModernEnclosure,
    FenceOrModernBoundary,
    OtherConfirmedLookalike,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardNegativeClassCount {
    pub class: HardNegativeClass,
    pub reference_object_count: usize,
    /// Model candidates adjudicated as false positives on this lookalike class.
    /// May exceed reference_object_count when multiple candidates hit one object.
    pub false_positive_prediction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardNegativeEvaluationUnit {
    pub provenance: EvaluationProvenance,
    /// Area that was actually surveyed and adequately adjudicated. Unsurveyed,
    /// inaccessible, unobservable or rights-withheld area is not in this total.
    pub adequately_assessed_area_km2: f64,
    pub assessment_receipt_ref: String,
    pub assessment_receipt_sha256: String,
    /// Confirms every prediction above the frozen threshold has a disposition:
    /// true positive, confirmed false positive, or explicitly unresolved.
    pub candidate_adjudication_complete: bool,
    pub candidate_adjudication_receipt_ref: String,
    pub candidate_adjudication_receipt_sha256: String,
    /// Confirms positive references and negative background are complete within
    /// the declared adequately assessed area.
    pub reference_coverage_complete: bool,
    pub reference_completeness_receipt_ref: String,
    pub reference_completeness_receipt_sha256: String,
    pub review_time_receipt_ref: String,
    pub review_time_receipt_sha256: String,
    pub expert_review_hours: f64,
    pub confirmed_constructed_wall_count: usize,
    pub true_positive_count: usize,
    pub false_negative_count: usize,
    pub hard_negative_classes: Vec<HardNegativeClassCount>,
    pub background_false_positive_prediction_count: usize,
    /// Predictions whose overlap or interpretation remains unresolved. They
    /// are not silently counted as either true or false positives.
    pub unresolved_prediction_count: usize,
    pub candidate_prediction_count: usize,
    pub ambiguous_reference_count: usize,
    pub not_observable_reference_count: usize,
    pub not_assessed_reference_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageHardNegativeInput {
    pub schema_version: String,
    pub challenge_id: String,
    pub model_id: String,
    pub confidence_threshold: f64,
    pub threshold_selection_receipt_ref: String,
    pub threshold_selection_receipt_sha256: String,
    pub challenge_manifest_sha256: String,
    pub reference_adjudication_protocol_sha256: String,
    pub units: Vec<HardNegativeEvaluationUnit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardNegativeClassReport {
    pub class: HardNegativeClass,
    pub reference_object_count: usize,
    pub false_positive_prediction_count: usize,
    /// This is a candidate-count / reference-object-count ratio, not a
    /// Bernoulli false-positive probability. It can exceed one.
    pub false_positive_candidates_per_reference_object: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardNegativeUnitReport {
    pub evaluation_unit_id: String,
    pub evaluation_unit_kind: EvaluationUnitKind,
    pub adequately_assessed_area_km2: f64,
    pub expert_review_hours: f64,
    pub confirmed_constructed_wall_count: usize,
    pub true_positive_count: usize,
    pub false_positive_count: usize,
    pub false_negative_count: usize,
    /// Precision is conditional on resolved predictions. Unresolved predictions
    /// are shown separately and not silently put in the false-positive bucket.
    pub resolved_prediction_metrics: CountMetrics,
    pub candidate_prediction_count: usize,
    pub resolved_prediction_count: usize,
    pub candidate_resolution_fraction: Option<f64>,
    pub unresolved_prediction_count: usize,
    pub false_positives_per_km2: f64,
    pub verified_findings_per_expert_hour: f64,
    pub false_leads_per_expert_hour: f64,
    pub ambiguous_reference_count: usize,
    pub not_observable_reference_count: usize,
    pub not_assessed_reference_count: usize,
    pub hard_negative_classes: Vec<HardNegativeClassReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageHardNegativeReport {
    pub schema_version: String,
    pub claim_ceiling: String,
    pub challenge_id: String,
    pub model_id: String,
    pub confidence_threshold: f64,
    pub threshold_selection_receipt_ref: String,
    pub threshold_selection_receipt_sha256: String,
    pub challenge_manifest_sha256: String,
    pub reference_adjudication_protocol_sha256: String,
    pub evaluation_unit_kind: EvaluationUnitKind,
    pub independent_evaluation_unit_count: usize,
    pub micro_counts: CountMetrics,
    pub macro_precision: Option<f64>,
    pub macro_precision_unit_count: usize,
    pub macro_recall: Option<f64>,
    pub macro_recall_unit_count: usize,
    pub macro_f1: Option<f64>,
    pub macro_f1_unit_count: usize,
    pub worst_unit_recall_id: Option<String>,
    pub worst_unit_recall: Option<f64>,
    pub adequately_assessed_area_km2: f64,
    pub total_expert_review_hours: f64,
    pub false_positives_per_km2: f64,
    pub verified_findings_per_expert_hour: f64,
    pub false_leads_per_expert_hour: f64,
    pub candidate_prediction_count: usize,
    pub resolved_prediction_count: usize,
    pub candidate_resolution_fraction: Option<f64>,
    pub unresolved_prediction_count: usize,
    pub ambiguous_reference_count: usize,
    pub not_observable_reference_count: usize,
    pub not_assessed_reference_count: usize,
    pub hard_negative_classes: Vec<HardNegativeClassReport>,
    pub units: Vec<HardNegativeUnitReport>,
    pub uncertainty_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageHardNegativeError {
    WrongSchemaVersion,
    EmptyChallengeOrModelId,
    InvalidThreshold,
    InvalidChallengeDigest,
    InvalidThresholdReceipt,
    InvalidUnitLimit,
    NoEvaluationUnits,
    InvalidProvenance(String),
    InvalidUnitIdentity(String),
    DuplicateEvaluationUnit(String),
    MixedEvaluationUnitKind,
    IncompleteCandidateAdjudication(String),
    IncompleteReferenceCoverage(String),
    InvalidAssessmentReceipt(String),
    InvalidArea(String),
    InvalidReviewTime(String),
    InvalidWallCounts(String),
    InvalidPredictionDispositionCounts(String),
    DuplicateHardNegativeClass(String),
    InvalidHardNegativeCounts(String),
    InconsistentModelOrProtocol,
    InvalidAggregateArithmetic,
}

impl fmt::Display for HeritageHardNegativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage hard-negative evaluation error: {self:?}")
    }
}

impl std::error::Error for HeritageHardNegativeError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_digest_and_ref(
    reference: &str,
    digest: &str,
    unit_id: &str,
) -> Result<(), HeritageHardNegativeError> {
    if !non_empty(reference) || !valid_sha256(digest) {
        return Err(HeritageHardNegativeError::InvalidAssessmentReceipt(
            unit_id.to_owned(),
        ));
    }
    Ok(())
}

fn count_metrics(
    tp: usize,
    fp: usize,
    fn_count: usize,
) -> Result<CountMetrics, HeritageHardNegativeError> {
    let precision_denominator = tp
        .checked_add(fp)
        .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
    let recall_denominator = tp
        .checked_add(fn_count)
        .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
    let precision = (precision_denominator > 0)
        .then(|| tp as f64 / precision_denominator as f64);
    let recall = (recall_denominator > 0).then(|| tp as f64 / recall_denominator as f64);
    let f1 = if recall_denominator == 0 {
        None
    } else if precision_denominator == 0 {
        Some(0.0)
    } else {
        match (precision, recall) {
            (Some(p), Some(r)) if p + r > 0.0 => Some(2.0 * p * r / (p + r)),
            (Some(_), Some(_)) => Some(0.0),
            _ => None,
        }
    };
    Ok(CountMetrics {
        true_positive: tp,
        false_positive: fp,
        false_negative: fn_count,
        precision,
        recall,
        f1,
    })
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut count = 0usize;
    let mut result = 0.0;
    for value in values {
        count += 1;
        result += (value - result) / count as f64;
    }
    (count > 0).then_some(result)
}

fn validate_unit(
    unit: &HardNegativeEvaluationUnit,
) -> Result<(usize, Vec<HardNegativeClassReport>), HeritageHardNegativeError> {
    let unit_id = &unit.provenance.evaluation_unit_id;
    unit.provenance
        .validate()
        .map_err(|error| HeritageHardNegativeError::InvalidProvenance(error.to_string()))?;

    if !non_empty(&unit.site_id)
        || !non_empty(&unit.region_id)
        || !non_empty(&unit.acquisition_id)
    {
        return Err(HeritageHardNegativeError::InvalidUnitIdentity(
            unit_id.clone(),
        ));
    }
    if !unit.candidate_adjudication_complete {
        return Err(
            HeritageHardNegativeError::IncompleteCandidateAdjudication(unit_id.clone()),
        );
    }
    if !unit.reference_coverage_complete {
        return Err(HeritageHardNegativeError::IncompleteReferenceCoverage(
            unit_id.clone(),
        ));
    }
    validate_digest_and_ref(
        &unit.assessment_receipt_ref,
        &unit.assessment_receipt_sha256,
        unit_id,
    )?;
    validate_digest_and_ref(
        &unit.candidate_adjudication_receipt_ref,
        &unit.candidate_adjudication_receipt_sha256,
        unit_id,
    )?;
    validate_digest_and_ref(
        &unit.reference_completeness_receipt_ref,
        &unit.reference_completeness_receipt_sha256,
        unit_id,
    )?;
    validate_digest_and_ref(
        &unit.review_time_receipt_ref,
        &unit.review_time_receipt_sha256,
        unit_id,
    )?;

    if !unit.adequately_assessed_area_km2.is_finite()
        || unit.adequately_assessed_area_km2 <= 0.0
    {
        return Err(HeritageHardNegativeError::InvalidArea(unit_id.clone()));
    }
    if !unit.expert_review_hours.is_finite()
        || unit.expert_review_hours <= 0.0
        || unit.expert_review_hours > MAX_EXPERT_REVIEW_HOURS
    {
        return Err(HeritageHardNegativeError::InvalidReviewTime(
            unit_id.clone(),
        ));
    }
    if unit
        .true_positive_count
        .checked_add(unit.false_negative_count)
        != Some(unit.confirmed_constructed_wall_count)
    {
        return Err(HeritageHardNegativeError::InvalidWallCounts(
            unit_id.clone(),
        ));
    }

    let mut seen_classes = BTreeSet::new();
    let mut class_reports = Vec::with_capacity(unit.hard_negative_classes.len());
    let mut class_false_positives = 0usize;
    for item in &unit.hard_negative_classes {
        if !seen_classes.insert(item.class) {
            return Err(HeritageHardNegativeError::DuplicateHardNegativeClass(
                unit_id.clone(),
            ));
        }
        if item.reference_object_count == 0 && item.false_positive_prediction_count > 0 {
            return Err(HeritageHardNegativeError::InvalidHardNegativeCounts(
                unit_id.clone(),
            ));
        }
        class_false_positives = class_false_positives
            .checked_add(item.false_positive_prediction_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        class_reports.push(HardNegativeClassReport {
            class: item.class,
            reference_object_count: item.reference_object_count,
            false_positive_prediction_count: item.false_positive_prediction_count,
            false_positive_candidates_per_reference_object: (item.reference_object_count > 0)
                .then(|| {
                    item.false_positive_prediction_count as f64
                        / item.reference_object_count as f64
                }),
        });
    }
    class_reports.sort_by_key(|item| item.class);

    let false_positive_count = class_false_positives
        .checked_add(unit.background_false_positive_prediction_count)
        .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
    let resolved_prediction_count = unit
        .true_positive_count
        .checked_add(false_positive_count)
        .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
    let disposition_count = resolved_prediction_count
        .checked_add(unit.unresolved_prediction_count)
        .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
    if disposition_count != unit.candidate_prediction_count {
        return Err(
            HeritageHardNegativeError::InvalidPredictionDispositionCounts(
                unit_id.clone(),
            ),
        );
    }

    Ok((false_positive_count, class_reports))
}

fn aggregate_classes(
    units: &[HardNegativeEvaluationUnit],
) -> Result<Vec<HardNegativeClassReport>, HeritageHardNegativeError> {
    let mut totals = BTreeMap::<HardNegativeClass, (usize, usize)>::new();
    for unit in units {
        for item in &unit.hard_negative_classes {
            let entry = totals.entry(item.class).or_default();
            entry.0 = entry
                .0
                .checked_add(item.reference_object_count)
                .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
            entry.1 = entry
                .1
                .checked_add(item.false_positive_prediction_count)
                .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        }
    }
    Ok(totals
        .into_iter()
        .map(|(class, (reference_count, false_positive_count))| HardNegativeClassReport {
            class,
            reference_object_count: reference_count,
            false_positive_prediction_count: false_positive_count,
            false_positive_candidates_per_reference_object: (reference_count > 0)
                .then(|| false_positive_count as f64 / reference_count as f64),
        })
        .collect())
}

/// Scores a hard-negative challenge using only complete, independently
/// adjudicated evaluation units. This function reports operational counts and
/// rates; it does not calculate confidence intervals or prove transfer to a
/// wider archaeological population.
pub fn evaluate_heritage_hard_negative(
    input: &HeritageHardNegativeInput,
) -> Result<HeritageHardNegativeReport, HeritageHardNegativeError> {
    if input.schema_version != HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION {
        return Err(HeritageHardNegativeError::WrongSchemaVersion);
    }
    if !non_empty(&input.challenge_id) || !non_empty(&input.model_id) {
        return Err(HeritageHardNegativeError::EmptyChallengeOrModelId);
    }
    if !input.confidence_threshold.is_finite()
        || !(0.0..=1.0).contains(&input.confidence_threshold)
    {
        return Err(HeritageHardNegativeError::InvalidThreshold);
    }
    if !non_empty(&input.threshold_selection_receipt_ref)
        || !valid_sha256(&input.threshold_selection_receipt_sha256)
    {
        return Err(HeritageHardNegativeError::InvalidThresholdReceipt);
    }
    if !valid_sha256(&input.challenge_manifest_sha256)
        || !valid_sha256(&input.reference_adjudication_protocol_sha256)
    {
        return Err(HeritageHardNegativeError::InvalidChallengeDigest);
    }
    if input.units.is_empty() {
        return Err(HeritageHardNegativeError::NoEvaluationUnits);
    }
    if input.units.len() > MAX_HARD_NEGATIVE_EVALUATION_UNITS {
        return Err(HeritageHardNegativeError::InvalidUnitLimit);
    }

    let mut ordered_units = input.units.clone();
    ordered_units.sort_by(|left, right| {
        left.provenance
            .evaluation_unit_id
            .cmp(&right.provenance.evaluation_unit_id)
    });

    let mut seen_ids = BTreeSet::new();
    let first = ordered_units.first().expect("non-empty input");
    let expected_kind = first.provenance.evaluation_unit_kind;
    let expected_model_artifact = &first.provenance.model_artifact_sha256;
    let expected_model_build = &first.provenance.model_build_sha256;
    let expected_split = &first.provenance.split_manifest_sha256;
    let expected_evaluator = &first.provenance.evaluator_build_sha256;
    let expected_metric_protocol = &first.provenance.metric_protocol_sha256;

    let mut reports = Vec::with_capacity(ordered_units.len());
    let mut total_tp = 0usize;
    let mut total_fp = 0usize;
    let mut total_fn = 0usize;
    let mut total_area = 0.0;
    let mut total_hours = 0.0;
    let mut total_candidates = 0usize;
    let mut total_resolved = 0usize;
    let mut total_unresolved = 0usize;
    let mut total_ambiguous = 0usize;
    let mut total_not_observable = 0usize;
    let mut total_not_assessed = 0usize;

    for unit in &ordered_units {
        let unit_id = &unit.provenance.evaluation_unit_id;
        if !seen_ids.insert(unit_id.clone()) {
            return Err(HeritageHardNegativeError::DuplicateEvaluationUnit(
                unit_id.clone(),
            ));
        }
        if unit.provenance.evaluation_unit_kind != expected_kind {
            return Err(HeritageHardNegativeError::MixedEvaluationUnitKind);
        }
        if unit.provenance.model_artifact_sha256 != *expected_model_artifact
            || unit.provenance.model_build_sha256 != *expected_model_build
            || unit.provenance.split_manifest_sha256 != *expected_split
            || unit.provenance.evaluator_build_sha256 != *expected_evaluator
            || unit.provenance.metric_protocol_sha256 != *expected_metric_protocol
        {
            return Err(HeritageHardNegativeError::InconsistentModelOrProtocol);
        }
        let (false_positive_count, class_reports) = validate_unit(unit)?;
        let resolved_count = unit
            .true_positive_count
            .checked_add(false_positive_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        let count_metrics = count_metrics(
            unit.true_positive_count,
            false_positive_count,
            unit.false_negative_count,
        )?;

        let candidate_resolution_fraction = (unit.candidate_prediction_count > 0)
            .then(|| resolved_count as f64 / unit.candidate_prediction_count as f64);
        reports.push(HardNegativeUnitReport {
            evaluation_unit_id: unit_id.clone(),
            evaluation_unit_kind: unit.provenance.evaluation_unit_kind,
            adequately_assessed_area_km2: unit.adequately_assessed_area_km2,
            expert_review_hours: unit.expert_review_hours,
            confirmed_constructed_wall_count: unit.confirmed_constructed_wall_count,
            true_positive_count: unit.true_positive_count,
            false_positive_count,
            false_negative_count: unit.false_negative_count,
            resolved_prediction_metrics: count_metrics,
            candidate_prediction_count: unit.candidate_prediction_count,
            resolved_prediction_count: resolved_count,
            candidate_resolution_fraction,
            unresolved_prediction_count: unit.unresolved_prediction_count,
            false_positives_per_km2: false_positive_count as f64
                / unit.adequately_assessed_area_km2,
            verified_findings_per_expert_hour: unit.true_positive_count as f64
                / unit.expert_review_hours,
            false_leads_per_expert_hour: false_positive_count as f64
                / unit.expert_review_hours,
            ambiguous_reference_count: unit.ambiguous_reference_count,
            not_observable_reference_count: unit.not_observable_reference_count,
            not_assessed_reference_count: unit.not_assessed_reference_count,
            hard_negative_classes: class_reports,
        });

        total_tp = total_tp
            .checked_add(unit.true_positive_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_fp = total_fp
            .checked_add(false_positive_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_fn = total_fn
            .checked_add(unit.false_negative_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_candidates = total_candidates
            .checked_add(unit.candidate_prediction_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_resolved = total_resolved
            .checked_add(resolved_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_unresolved = total_unresolved
            .checked_add(unit.unresolved_prediction_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_ambiguous = total_ambiguous
            .checked_add(unit.ambiguous_reference_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_not_observable = total_not_observable
            .checked_add(unit.not_observable_reference_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_not_assessed = total_not_assessed
            .checked_add(unit.not_assessed_reference_count)
            .ok_or(HeritageHardNegativeError::InvalidAggregateArithmetic)?;
        total_area += unit.adequately_assessed_area_km2;
        total_hours += unit.expert_review_hours;
    }

    if !total_area.is_finite() || total_area <= 0.0 || !total_hours.is_finite() || total_hours <= 0.0 {
        return Err(HeritageHardNegativeError::InvalidAggregateArithmetic);
    }

    let micro = count_metrics(total_tp, total_fp, total_fn)?;
    let macro_precision = mean(reports.iter().filter_map(|unit| unit.resolved_prediction_metrics.precision));
    let macro_recall = mean(reports.iter().filter_map(|unit| unit.resolved_prediction_metrics.recall));
    let macro_f1 = mean(reports.iter().filter_map(|unit| unit.resolved_prediction_metrics.f1));
    let macro_precision_unit_count = reports.iter().filter(|unit| unit.resolved_prediction_metrics.precision.is_some()).count();
    let macro_recall_unit_count = reports.iter().filter(|unit| unit.resolved_prediction_metrics.recall.is_some()).count();
    let macro_f1_unit_count = reports.iter().filter(|unit| unit.resolved_prediction_metrics.f1.is_some()).count();

    let worst = reports
        .iter()
        .filter_map(|unit| unit.resolved_prediction_metrics.recall.map(|value| (value, unit.evaluation_unit_id.as_str())))
        .min_by(|left, right| left.0.total_cmp(&right.0).then_with(|| left.1.cmp(right.1)));

    let candidate_resolution_fraction = (total_candidates > 0)
        .then(|| total_resolved as f64 / total_candidates as f64);

    Ok(HeritageHardNegativeReport {
        schema_version: HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION.to_owned(),
        claim_ceiling: "operational_hard_negative_scorecard_only_no_population_generalisation_or_archaeological_identification"
            .to_owned(),
        challenge_id: input.challenge_id.clone(),
        model_id: input.model_id.clone(),
        confidence_threshold: input.confidence_threshold,
        threshold_selection_receipt_ref: input.threshold_selection_receipt_ref.clone(),
        threshold_selection_receipt_sha256: input.threshold_selection_receipt_sha256.clone(),
        challenge_manifest_sha256: input.challenge_manifest_sha256.clone(),
        reference_adjudication_protocol_sha256: input.reference_adjudication_protocol_sha256.clone(),
        evaluation_unit_kind: expected_kind,
        independent_evaluation_unit_count: reports.len(),
        micro_counts: micro,
        macro_precision,
        macro_precision_unit_count,
        macro_recall,
        macro_recall_unit_count,
        macro_f1,
        macro_f1_unit_count,
        worst_unit_recall_id: worst.map(|item| item.1.to_owned()),
        worst_unit_recall: worst.map(|item| item.0),
        adequately_assessed_area_km2: total_area,
        total_expert_review_hours: total_hours,
        false_positives_per_km2: total_fp as f64 / total_area,
        verified_findings_per_expert_hour: total_tp as f64 / total_hours,
        false_leads_per_expert_hour: total_fp as f64 / total_hours,
        candidate_prediction_count: total_candidates,
        resolved_prediction_count: total_resolved,
        candidate_resolution_fraction,
        unresolved_prediction_count: total_unresolved,
        ambiguous_reference_count: total_ambiguous,
        not_observable_reference_count: total_not_observable,
        not_assessed_reference_count: total_not_assessed,
        hard_negative_classes: aggregate_classes(&ordered_units)?,
        units: reports,
        uncertainty_note: "no confidence intervals are computed; macro metrics are descriptive across the declared independent units. Use a preregistered site/region/acquisition-level bootstrap and spatial effective-information analysis before population-level claims."
            .to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn provenance(id: &str) -> EvaluationProvenance {
        EvaluationProvenance {
            source_doi_or_identifier: "urn:synthetic:hdb-b9".into(),
            dataset_sha256: digest('a'),
            model_artifact_sha256: digest('b'),
            model_build_sha256: digest('c'),
            prediction_sha256: digest('d'),
            reference_label_sha256: digest('e'),
            split_manifest_sha256: digest('f'),
            evaluator_build_sha256: digest('1'),
            metric_protocol_sha256: digest('2'),
            rights_receipt_ref: "urn:synthetic:rights".into(),
            evaluation_unit_kind: EvaluationUnitKind::Site,
            evaluation_unit_id: id.into(),
        }
    }

    fn unit(
        id: &str,
        tp: usize,
        fp_rock: usize,
        fp_background: usize,
        fns: usize,
        area: f64,
        hours: f64,
    ) -> HardNegativeEvaluationUnit {
        HardNegativeEvaluationUnit {
            provenance: provenance(id),
            adequately_assessed_area_km2: area,
            assessment_receipt_ref: format!("urn:synthetic:assessment:{id}"),
            assessment_receipt_sha256: digest('3'),
            candidate_adjudication_complete: true,
            candidate_adjudication_receipt_ref: format!("urn:synthetic:candidate-review:{id}"),
            candidate_adjudication_receipt_sha256: digest('4'),
            reference_coverage_complete: true,
            reference_completeness_receipt_ref: format!("urn:synthetic:reference-complete:{id}"),
            reference_completeness_receipt_sha256: digest('5'),
            review_time_receipt_ref: format!("urn:synthetic:review-time:{id}"),
            review_time_receipt_sha256: digest('6'),
            expert_review_hours: hours,
            confirmed_constructed_wall_count: tp + fns,
            true_positive_count: tp,
            false_negative_count: fns,
            hard_negative_classes: vec![
                HardNegativeClassCount {
                    class: HardNegativeClass::ErodedRockLedge,
                    reference_object_count: 2,
                    false_positive_prediction_count: fp_rock,
                },
                HardNegativeClassCount {
                    class: HardNegativeClass::LivestockKraalOrModernEnclosure,
                    reference_object_count: 1,
                    false_positive_prediction_count: 0,
                },
            ],
            background_false_positive_prediction_count: fp_background,
            unresolved_prediction_count: 1,
            candidate_prediction_count: tp + fp_rock + fp_background + 1,
            ambiguous_reference_count: 1,
            not_observable_reference_count: 1,
            not_assessed_reference_count: 2,
        }
    }

    fn input() -> HeritageHardNegativeInput {
        HeritageHardNegativeInput {
            schema_version: HERITAGE_HARD_NEGATIVE_SCHEMA_VERSION.into(),
            challenge_id: "hdb-b9-synthetic".into(),
            model_id: "model-synthetic".into(),
            confidence_threshold: 0.5,
            threshold_selection_receipt_ref: "urn:synthetic:threshold".into(),
            threshold_selection_receipt_sha256: digest('7'),
            challenge_manifest_sha256: digest('8'),
            reference_adjudication_protocol_sha256: digest('9'),
            units: vec![
                unit("site-01", 3, 1, 1, 1, 10.0, 2.0),
                unit("site-02", 2, 0, 0, 0, 5.0, 1.0),
            ],
        }
    }

    #[test]
    fn reports_micro_macro_worst_unit_and_operational_metrics() {
        let report = evaluate_heritage_hard_negative(&input()).unwrap();
        assert_eq!(report.independent_evaluation_unit_count, 2);
        assert_eq!(report.micro_counts.true_positive, 5);
        assert_eq!(report.micro_counts.false_positive, 2);
        assert_eq!(report.micro_counts.false_negative, 1);
        assert!((report.micro_counts.precision.unwrap() - 5.0 / 7.0).abs() < 1e-12);
        assert!((report.micro_counts.recall.unwrap() - 5.0 / 6.0).abs() < 1e-12);
        assert!((report.macro_precision.unwrap() - 0.8).abs() < 1e-12);
        assert!((report.macro_recall.unwrap() - 0.875).abs() < 1e-12);
        assert_eq!(report.worst_unit_recall_id.as_deref(), Some("site-01"));
        assert!((report.false_positives_per_km2 - (2.0 / 15.0)).abs() < 1e-12);
        assert!((report.verified_findings_per_expert_hour - (5.0 / 3.0)).abs() < 1e-12);
        assert!((report.false_leads_per_expert_hour - (2.0 / 3.0)).abs() < 1e-12);
        assert_eq!(report.unresolved_prediction_count, 2);
        assert_eq!(report.ambiguous_reference_count, 2);
        assert_eq!(report.hard_negative_classes.len(), 2);
    }

    #[test]
    fn input_order_does_not_change_report() {
        let first = input();
        let mut second = first.clone();
        second.units.reverse();
        assert_eq!(
            evaluate_heritage_hard_negative(&first).unwrap(),
            evaluate_heritage_hard_negative(&second).unwrap()
        );
    }

    #[test]
    fn incomplete_reference_or_candidate_adjudication_fails_closed() {
        let mut value = input();
        value.units[0].candidate_adjudication_complete = false;
        assert_eq!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::IncompleteCandidateAdjudication(
                "site-01".into()
            ))
        );

        let mut value = input();
        value.units[0].reference_coverage_complete = false;
        assert_eq!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::IncompleteReferenceCoverage(
                "site-01".into()
            ))
        );
    }

    #[test]
    fn predictions_must_have_exactly_one_resolved_or_unresolved_disposition() {
        let mut value = input();
        value.units[0].candidate_prediction_count += 1;
        assert!(matches!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::InvalidPredictionDispositionCounts(_))
        ));
    }

    #[test]
    fn positive_reference_count_must_equal_true_positives_plus_false_negatives() {
        let mut value = input();
        value.units[0].confirmed_constructed_wall_count += 1;
        assert!(matches!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::InvalidWallCounts(_))
        ));
    }

    #[test]
    fn duplicate_class_or_prediction_without_reference_fails_closed() {
        let mut value = input();
        value.units[0]
            .hard_negative_classes
            .push(value.units[0].hard_negative_classes[0].clone());
        assert!(matches!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::DuplicateHardNegativeClass(_))
        ));

        let mut value = input();
        value.units[0].hard_negative_classes[0].reference_object_count = 0;
        assert!(matches!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::InvalidHardNegativeCounts(_))
        ));
    }

    #[test]
    fn independent_unit_kind_and_model_protocol_must_be_consistent() {
        let mut value = input();
        value.units[1].provenance.evaluation_unit_kind = EvaluationUnitKind::Region;
        assert_eq!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::MixedEvaluationUnitKind)
        );

        let mut value = input();
        value.units[1].provenance.model_build_sha256 = digest('0');
        assert_eq!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::InconsistentModelOrProtocol)
        );
    }

    #[test]
    fn malformed_threshold_receipt_and_area_fail_closed() {
        let mut value = input();
        value.confidence_threshold = f64::NAN;
        assert_eq!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::InvalidThreshold)
        );

        let mut value = input();
        value.units[0].adequately_assessed_area_km2 = 0.0;
        assert!(matches!(
            evaluate_heritage_hard_negative(&value),
            Err(HeritageHardNegativeError::InvalidArea(_))
        ));
    }
}
