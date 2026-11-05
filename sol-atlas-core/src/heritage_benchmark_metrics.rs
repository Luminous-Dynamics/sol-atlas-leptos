// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic stone-instance and wall-segment benchmark metrics.
//!
//! This module evaluates declared masks. It does not perform image inference,
//! transform coordinate systems, authenticate source files or rights, establish
//! archaeological identity, or prove that reference annotations are correct.
//! A metric report is evidence about the supplied inputs under the pinned
//! metric policy, not evidence of archaeological truth.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_BENCHMARK_METRICS_SCHEMA_VERSION: &str =
    "luminous.heritage-benchmark-metrics.v0.1";
pub const MAX_BENCHMARK_OBJECTS: usize = 1_000_000;
pub const MAX_TOTAL_MASK_PIXELS: usize = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterDimensions {
    pub width: u32,
    pub height: u32,
}

impl RasterDimensions {
    fn pixel_count(self) -> Result<u64, HeritageMetricError> {
        if self.width == 0 || self.height == 0 {
            return Err(HeritageMetricError::InvalidRasterDimensions);
        }
        u64::from(self.width)
            .checked_mul(u64::from(self.height))
            .ok_or(HeritageMetricError::InvalidRasterDimensions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelMask {
    /// Canonical, strictly increasing, zero-based row-major pixel indices.
    pub pixel_indices: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationUnitKind {
    Site,
    Region,
    SourcePaperTestPartition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationProvenance {
    pub source_doi_or_identifier: String,
    pub dataset_sha256: String,
    pub model_artifact_sha256: String,
    pub model_build_sha256: String,
    pub prediction_sha256: String,
    pub reference_label_sha256: String,
    pub split_manifest_sha256: String,
    pub evaluator_build_sha256: String,
    pub metric_protocol_sha256: String,
    pub rights_receipt_ref: String,
    pub evaluation_unit_kind: EvaluationUnitKind,
    pub evaluation_unit_id: String,
}

impl EvaluationProvenance {
    pub fn validate(&self) -> Result<(), HeritageMetricError> {
        if self.source_doi_or_identifier.trim().is_empty()
            || self.rights_receipt_ref.trim().is_empty()
            || self.evaluation_unit_id.trim().is_empty()
        {
            return Err(HeritageMetricError::InvalidProvenance(
                "source identifier, rights receipt and evaluation unit must be non-empty".into(),
            ));
        }
        for (name, value) in [
            ("dataset_sha256", &self.dataset_sha256),
            ("model_artifact_sha256", &self.model_artifact_sha256),
            ("model_build_sha256", &self.model_build_sha256),
            ("prediction_sha256", &self.prediction_sha256),
            ("reference_label_sha256", &self.reference_label_sha256),
            ("split_manifest_sha256", &self.split_manifest_sha256),
            ("evaluator_build_sha256", &self.evaluator_build_sha256),
            ("metric_protocol_sha256", &self.metric_protocol_sha256),
        ] {
            if !is_sha256(value) {
                return Err(HeritageMetricError::InvalidProvenance(format!(
                    "{name} is not a 64-character hexadecimal digest"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeritageMetricError {
    InvalidRasterDimensions,
    TooManyObjects,
    TooManyMaskPixels,
    InvalidMask(String),
    DuplicateIdentifier(String),
    InvalidConfidence(String),
    InvalidThreshold(String),
    InvalidSizeRule,
    InvalidProvenance(String),
    NoCoverageThresholds,
    CoverageThresholdsNotStrictlyDescending,
}

impl fmt::Display for HeritageMetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for HeritageMetricError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundTruthStone {
    pub instance_id: String,
    pub mask: PixelMask,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedStone {
    pub instance_id: String,
    pub confidence: f64,
    pub mask: PixelMask,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoneEvaluationInput {
    pub raster: RasterDimensions,
    pub iou_threshold: f64,
    pub confidence_threshold: f64,
    /// The evaluator receives objects already filtered by this declared rule.
    /// It does not infer centimetres from raster pixel masks.
    pub size_rule_id: String,
    pub provenance: EvaluationProvenance,
    pub ground_truth: Vec<GroundTruthStone>,
    pub predictions: Vec<PredictedStone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoneMatch {
    pub prediction_id: String,
    pub ground_truth_id: String,
    pub intersection_over_union: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CountMetrics {
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoneEvaluationReport {
    pub schema_version: String,
    pub claim_ceiling: String,
    pub provenance: EvaluationProvenance,
    pub raster: RasterDimensions,
    pub size_rule_id: String,
    pub iou_threshold: f64,
    pub confidence_threshold: f64,
    pub ground_truth_count: usize,
    pub prediction_count: usize,
    pub predictions_at_confidence_threshold: usize,
    pub counts: CountMetrics,
    /// All-points interpolated AP over the confidence-ranked predictions.
    pub average_precision: Option<f64>,
    /// One-to-one matches at the declared confidence operating point.
    pub matches: Vec<StoneMatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceSegmentStatus {
    Eligible,
    Ambiguous,
    NotAssessed,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundTruthWallSegment {
    pub segment_id: String,
    pub status: ReferenceSegmentStatus,
    pub mask: PixelMask,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedWallSegment {
    pub segment_id: String,
    pub confidence: f64,
    pub mask: PixelMask,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallEvaluationInput {
    pub raster: RasterDimensions,
    pub confidence_threshold: f64,
    /// Coverage thresholds must be supplied in strictly descending order.
    pub coverage_thresholds: Vec<f64>,
    pub provenance: EvaluationProvenance,
    pub ground_truth_segments: Vec<GroundTruthWallSegment>,
    pub predicted_segments: Vec<PredictedWallSegment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentCoverage {
    pub segment_id: String,
    pub status: ReferenceSegmentStatus,
    /// None means the reference segment was ambiguous, excluded or not assessed.
    pub covered_fraction: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedSegmentCoverage {
    pub segment_id: String,
    pub covered_fraction: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WallCoverageThresholdResult {
    pub threshold: f64,
    pub covered_ground_truth_count: usize,
    pub eligible_ground_truth_count: usize,
    pub covered_ground_truth_rate: Option<f64>,
    pub covered_prediction_count: usize,
    pub prediction_count: usize,
    pub covered_prediction_rate: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallEvaluationReport {
    pub schema_version: String,
    pub claim_ceiling: String,
    pub provenance: EvaluationProvenance,
    pub raster: RasterDimensions,
    pub confidence_threshold: f64,
    pub predictions_at_confidence_threshold: usize,
    pub eligible_ground_truth_segment_count: usize,
    pub ambiguous_ground_truth_segment_count: usize,
    pub not_assessed_ground_truth_segment_count: usize,
    pub excluded_ground_truth_segment_count: usize,
    pub counts: CountMetrics,
    pub ground_truth_segment_coverage: Vec<SegmentCoverage>,
    pub prediction_segment_coverage: Vec<PredictedSegmentCoverage>,
    pub coverage_threshold_results: Vec<WallCoverageThresholdResult>,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_threshold(value: f64, label: &str) -> Result<(), HeritageMetricError> {
    if !value.is_finite() || value <= 0.0 || value > 1.0 {
        return Err(HeritageMetricError::InvalidThreshold(label.into()));
    }
    Ok(())
}

fn validate_confidence(value: f64, identifier: &str) -> Result<(), HeritageMetricError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(HeritageMetricError::InvalidConfidence(identifier.into()));
    }
    Ok(())
}

fn validate_mask(
    mask: &PixelMask,
    dimensions: RasterDimensions,
    owner: &str,
) -> Result<(), HeritageMetricError> {
    let pixel_count = dimensions.pixel_count()?;
    if mask.pixel_indices.is_empty() {
        return Err(HeritageMetricError::InvalidMask(format!(
            "{owner}: empty mask"
        )));
    }
    if mask
        .pixel_indices
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err(HeritageMetricError::InvalidMask(format!(
            "{owner}: pixel indices must be strictly increasing and unique"
        )));
    }
    if mask.pixel_indices.iter().any(|index| *index >= pixel_count) {
        return Err(HeritageMetricError::InvalidMask(format!(
            "{owner}: pixel index is outside raster bounds"
        )));
    }
    Ok(())
}

fn validate_mask_budget(total: &mut usize, mask_len: usize) -> Result<(), HeritageMetricError> {
    *total = total
        .checked_add(mask_len)
        .ok_or(HeritageMetricError::TooManyMaskPixels)?;
    if *total > MAX_TOTAL_MASK_PIXELS {
        return Err(HeritageMetricError::TooManyMaskPixels);
    }
    Ok(())
}

fn insert_unique(
    identifiers: &mut BTreeSet<String>,
    identifier: &str,
) -> Result<(), HeritageMetricError> {
    if identifier.trim().is_empty() || !identifiers.insert(identifier.to_owned()) {
        return Err(HeritageMetricError::DuplicateIdentifier(identifier.into()));
    }
    Ok(())
}

fn intersection_count(left: &[u64], right: &[u64]) -> usize {
    let mut i = 0;
    let mut j = 0;
    let mut count = 0;
    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
        }
    }
    count
}

fn union_two(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut output = Vec::with_capacity(left.len().saturating_add(right.len()));
    let mut i = 0;
    let mut j = 0;
    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => {
                output.push(left[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                output.push(right[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                output.push(left[i]);
                i += 1;
                j += 1;
            }
        }
    }
    output.extend_from_slice(&left[i..]);
    output.extend_from_slice(&right[j..]);
    output
}

fn union_masks<'a, I>(masks: I) -> Vec<u64>
where
    I: IntoIterator<Item = &'a PixelMask>,
{
    masks.into_iter().fold(Vec::new(), |accumulator, mask| {
        union_two(&accumulator, &mask.pixel_indices)
    })
}

fn intersection_over_union(left: &[u64], right: &[u64]) -> f64 {
    let intersection = intersection_count(left, right);
    let union = left.len() + right.len() - intersection;
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn count_metrics(
    true_positive: usize,
    false_positive: usize,
    false_negative: usize,
) -> CountMetrics {
    let precision_denominator = true_positive + false_positive;
    let recall_denominator = true_positive + false_negative;
    let precision = if precision_denominator > 0 {
        Some(true_positive as f64 / precision_denominator as f64)
    } else {
        None
    };
    let recall = if recall_denominator > 0 {
        Some(true_positive as f64 / recall_denominator as f64)
    } else {
        None
    };
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
    CountMetrics {
        true_positive,
        false_positive,
        false_negative,
        precision,
        recall,
        f1,
    }
}

fn rank_stone_predictions(predictions: &[PredictedStone]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..predictions.len()).collect();
    indices.sort_by(|left, right| {
        predictions[*right]
            .confidence
            .total_cmp(&predictions[*left].confidence)
            .then_with(|| {
                predictions[*left]
                    .instance_id
                    .cmp(&predictions[*right].instance_id)
            })
    });
    indices
}

fn match_stone_predictions(
    predictions: &[PredictedStone],
    ground_truth: &[GroundTruthStone],
    ranked_indices: &[usize],
    iou_threshold: f64,
) -> Vec<Option<(usize, f64)>> {
    let mut matched_ground_truth = vec![false; ground_truth.len()];
    let mut matches = vec![None; predictions.len()];
    for prediction_index in ranked_indices {
        let prediction = &predictions[*prediction_index];
        let mut best: Option<(usize, f64)> = None;
        for (ground_truth_index, reference) in ground_truth.iter().enumerate() {
            if matched_ground_truth[ground_truth_index] {
                continue;
            }
            let score = intersection_over_union(
                &prediction.mask.pixel_indices,
                &reference.mask.pixel_indices,
            );
            if score < iou_threshold {
                continue;
            }
            let replace = match best {
                None => true,
                Some((best_index, best_score)) => {
                    score > best_score
                        || (score == best_score
                            && reference.instance_id < ground_truth[best_index].instance_id)
                }
            };
            if replace {
                best = Some((ground_truth_index, score));
            }
        }
        if let Some((ground_truth_index, score)) = best {
            matched_ground_truth[ground_truth_index] = true;
            matches[*prediction_index] = Some((ground_truth_index, score));
        }
    }
    matches
}

fn average_precision(
    ground_truth_count: usize,
    ranked_indices: &[usize],
    matches: &[Option<(usize, f64)>],
) -> Option<f64> {
    if ground_truth_count == 0 {
        return None;
    }
    if ranked_indices.is_empty() {
        return Some(0.0);
    }
    let mut cumulative_true_positives = 0usize;
    let mut precision_at_rank = Vec::with_capacity(ranked_indices.len());
    let mut true_positive_at_rank = Vec::with_capacity(ranked_indices.len());
    for (rank, prediction_index) in ranked_indices.iter().enumerate() {
        let is_true_positive = matches[*prediction_index].is_some();
        if is_true_positive {
            cumulative_true_positives += 1;
        }
        precision_at_rank.push(cumulative_true_positives as f64 / (rank + 1) as f64);
        true_positive_at_rank.push(is_true_positive);
    }
    let mut precision_envelope = precision_at_rank.clone();
    for index in (0..precision_envelope.len().saturating_sub(1)).rev() {
        precision_envelope[index] = precision_envelope[index].max(precision_envelope[index + 1]);
    }
    let mut previous_recall = 0.0;
    let mut area = 0.0;
    let mut true_positives = 0usize;
    for index in 0..true_positive_at_rank.len() {
        if true_positive_at_rank[index] {
            true_positives += 1;
            let recall = true_positives as f64 / ground_truth_count as f64;
            area += (recall - previous_recall) * precision_envelope[index];
            previous_recall = recall;
        }
    }
    Some(area.clamp(0.0, 1.0))
}

fn validate_stone_input(input: &StoneEvaluationInput) -> Result<(), HeritageMetricError> {
    input.raster.pixel_count()?;
    validate_threshold(input.iou_threshold, "iou_threshold")?;
    if !input.confidence_threshold.is_finite() || !(0.0..=1.0).contains(&input.confidence_threshold)
    {
        return Err(HeritageMetricError::InvalidThreshold(
            "confidence_threshold".into(),
        ));
    }
    if input.size_rule_id.trim().is_empty() {
        return Err(HeritageMetricError::InvalidSizeRule);
    }
    input.provenance.validate()?;
    if input.ground_truth.len() + input.predictions.len() > MAX_BENCHMARK_OBJECTS {
        return Err(HeritageMetricError::TooManyObjects);
    }
    let mut total_mask_pixels = 0usize;
    let mut ground_truth_ids = BTreeSet::new();
    for reference in &input.ground_truth {
        insert_unique(&mut ground_truth_ids, &reference.instance_id)?;
        validate_mask(&reference.mask, input.raster, &reference.instance_id)?;
        validate_mask_budget(&mut total_mask_pixels, reference.mask.pixel_indices.len())?;
    }
    let mut prediction_ids = BTreeSet::new();
    for prediction in &input.predictions {
        insert_unique(&mut prediction_ids, &prediction.instance_id)?;
        validate_confidence(prediction.confidence, &prediction.instance_id)?;
        validate_mask(&prediction.mask, input.raster, &prediction.instance_id)?;
        validate_mask_budget(&mut total_mask_pixels, prediction.mask.pixel_indices.len())?;
    }
    Ok(())
}

pub fn evaluate_stone_instances(
    input: &StoneEvaluationInput,
) -> Result<StoneEvaluationReport, HeritageMetricError> {
    validate_stone_input(input)?;
    let ranked_all = rank_stone_predictions(&input.predictions);
    let all_matches = match_stone_predictions(
        &input.predictions,
        &input.ground_truth,
        &ranked_all,
        input.iou_threshold,
    );
    let average_precision = average_precision(input.ground_truth.len(), &ranked_all, &all_matches);
    let selected_indices: Vec<usize> = ranked_all
        .iter()
        .copied()
        .filter(|index| input.predictions[*index].confidence >= input.confidence_threshold)
        .collect();
    let selected_matches = match_stone_predictions(
        &input.predictions,
        &input.ground_truth,
        &selected_indices,
        input.iou_threshold,
    );
    let true_positive = selected_indices
        .iter()
        .filter(|index| selected_matches[**index].is_some())
        .count();
    let false_positive = selected_indices.len() - true_positive;
    let false_negative = input.ground_truth.len() - true_positive;
    let matches = selected_indices
        .iter()
        .filter_map(|index| {
            selected_matches[*index].map(|(ground_truth_index, score)| StoneMatch {
                prediction_id: input.predictions[*index].instance_id.clone(),
                ground_truth_id: input.ground_truth[ground_truth_index].instance_id.clone(),
                intersection_over_union: score,
            })
        })
        .collect();
    Ok(StoneEvaluationReport {
        schema_version: HERITAGE_BENCHMARK_METRICS_SCHEMA_VERSION.into(),
        claim_ceiling: "metric_computation_only".into(),
        provenance: input.provenance.clone(),
        raster: input.raster,
        size_rule_id: input.size_rule_id.clone(),
        iou_threshold: input.iou_threshold,
        confidence_threshold: input.confidence_threshold,
        ground_truth_count: input.ground_truth.len(),
        prediction_count: input.predictions.len(),
        predictions_at_confidence_threshold: selected_indices.len(),
        counts: count_metrics(true_positive, false_positive, false_negative),
        average_precision,
        matches,
    })
}

fn validate_wall_input(input: &WallEvaluationInput) -> Result<(), HeritageMetricError> {
    input.raster.pixel_count()?;
    if !input.confidence_threshold.is_finite()
        || !(0.0..=1.0).contains(&input.confidence_threshold)
    {
        return Err(HeritageMetricError::InvalidThreshold(
            "confidence_threshold".into(),
        ));
    }
    if input.coverage_thresholds.is_empty() {
        return Err(HeritageMetricError::NoCoverageThresholds);
    }
    let mut previous = f64::INFINITY;
    for threshold in &input.coverage_thresholds {
        validate_threshold(*threshold, "coverage_threshold")?;
        if *threshold >= previous {
            return Err(HeritageMetricError::CoverageThresholdsNotStrictlyDescending);
        }
        previous = *threshold;
    }
    input.provenance.validate()?;
    if input.ground_truth_segments.len() + input.predicted_segments.len() > MAX_BENCHMARK_OBJECTS
    {
        return Err(HeritageMetricError::TooManyObjects);
    }
    let mut total_mask_pixels = 0usize;
    let mut ground_truth_ids = BTreeSet::new();
    for segment in &input.ground_truth_segments {
        insert_unique(&mut ground_truth_ids, &segment.segment_id)?;
        validate_mask(&segment.mask, input.raster, &segment.segment_id)?;
        validate_mask_budget(&mut total_mask_pixels, segment.mask.pixel_indices.len())?;
    }
    let mut prediction_ids = BTreeSet::new();
    for segment in &input.predicted_segments {
        insert_unique(&mut prediction_ids, &segment.segment_id)?;
        validate_confidence(segment.confidence, &segment.segment_id)?;
        validate_mask(&segment.mask, input.raster, &segment.segment_id)?;
        validate_mask_budget(&mut total_mask_pixels, segment.mask.pixel_indices.len())?;
    }
    Ok(())
}

pub fn evaluate_wall_segments(
    input: &WallEvaluationInput,
) -> Result<WallEvaluationReport, HeritageMetricError> {
    validate_wall_input(input)?;
    let selected_predictions: Vec<&PredictedWallSegment> = input
        .predicted_segments
        .iter()
        .filter(|segment| segment.confidence >= input.confidence_threshold)
        .collect();
    let eligible_ground_truth: Vec<&GroundTruthWallSegment> = input
        .ground_truth_segments
        .iter()
        .filter(|segment| segment.status == ReferenceSegmentStatus::Eligible)
        .collect();
    let ground_truth_union = union_masks(eligible_ground_truth.iter().map(|segment| &segment.mask));
    let prediction_union = union_masks(selected_predictions.iter().map(|segment| &segment.mask));
    let true_positive = intersection_count(&ground_truth_union, &prediction_union);
    let false_positive = prediction_union.len() - true_positive;
    let false_negative = ground_truth_union.len() - true_positive;
    let counts = count_metrics(true_positive, false_positive, false_negative);

    let ground_truth_segment_coverage: Vec<SegmentCoverage> = input
        .ground_truth_segments
        .iter()
        .map(|segment| {
            let covered_fraction = if segment.status == ReferenceSegmentStatus::Eligible {
                Some(
                    intersection_count(&segment.mask.pixel_indices, &prediction_union) as f64
                        / segment.mask.pixel_indices.len() as f64,
                )
            } else {
                None
            };
            SegmentCoverage {
                segment_id: segment.segment_id.clone(),
                status: segment.status,
                covered_fraction,
            }
        })
        .collect();

    let prediction_segment_coverage: Vec<PredictedSegmentCoverage> = selected_predictions
        .iter()
        .map(|segment| {
            let covered_fraction = if ground_truth_union.is_empty() {
                None
            } else {
                Some(
                    intersection_count(&segment.mask.pixel_indices, &ground_truth_union) as f64
                        / segment.mask.pixel_indices.len() as f64,
                )
            };
            PredictedSegmentCoverage {
                segment_id: segment.segment_id.clone(),
                covered_fraction,
            }
        })
        .collect();

    let eligible_ground_truth_count = eligible_ground_truth.len();
    let coverage_threshold_results = input
        .coverage_thresholds
        .iter()
        .map(|threshold| {
            let covered_ground_truth_count = ground_truth_segment_coverage
                .iter()
                .filter(|segment| {
                    segment
                        .covered_fraction
                        .is_some_and(|coverage| coverage >= *threshold)
                })
                .count();
            let covered_prediction_count = prediction_segment_coverage
                .iter()
                .filter(|segment| {
                    segment
                        .covered_fraction
                        .is_some_and(|coverage| coverage >= *threshold)
                })
                .count();
            WallCoverageThresholdResult {
                threshold: *threshold,
                covered_ground_truth_count,
                eligible_ground_truth_count,
                covered_ground_truth_rate: (eligible_ground_truth_count > 0).then_some(
                    covered_ground_truth_count as f64 / eligible_ground_truth_count as f64,
                ),
                covered_prediction_count,
                prediction_count: selected_predictions.len(),
                covered_prediction_rate: (!selected_predictions.is_empty())
                    .then_some(covered_prediction_count as f64 / selected_predictions.len() as f64),
            }
        })
        .collect();

    Ok(WallEvaluationReport {
        schema_version: HERITAGE_BENCHMARK_METRICS_SCHEMA_VERSION.into(),
        claim_ceiling: "metric_computation_only".into(),
        provenance: input.provenance.clone(),
        raster: input.raster,
        confidence_threshold: input.confidence_threshold,
        predictions_at_confidence_threshold: selected_predictions.len(),
        eligible_ground_truth_segment_count: eligible_ground_truth_count,
        ambiguous_ground_truth_segment_count: input
            .ground_truth_segments
            .iter()
            .filter(|segment| segment.status == ReferenceSegmentStatus::Ambiguous)
            .count(),
        not_assessed_ground_truth_segment_count: input
            .ground_truth_segments
            .iter()
            .filter(|segment| segment.status == ReferenceSegmentStatus::NotAssessed)
            .count(),
        excluded_ground_truth_segment_count: input
            .ground_truth_segments
            .iter()
            .filter(|segment| segment.status == ReferenceSegmentStatus::Excluded)
            .count(),
        counts,
        ground_truth_segment_coverage,
        prediction_segment_coverage,
        coverage_threshold_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> EvaluationProvenance {
        EvaluationProvenance {
            source_doi_or_identifier: "10.1016/j.jas.2025.106430".into(),
            dataset_sha256: "a".repeat(64),
            model_artifact_sha256: "b".repeat(64),
            model_build_sha256: "c".repeat(64),
            prediction_sha256: "d".repeat(64),
            reference_label_sha256: "e".repeat(64),
            split_manifest_sha256: "f".repeat(64),
            evaluator_build_sha256: "1".repeat(64),
            metric_protocol_sha256: "2".repeat(64),
            rights_receipt_ref: "receipt:synthetic-test".into(),
            evaluation_unit_kind: EvaluationUnitKind::Site,
            evaluation_unit_id: "synthetic-site-1".into(),
        }
    }

    fn mask(pixels: &[u64]) -> PixelMask {
        PixelMask {
            pixel_indices: pixels.to_vec(),
        }
    }

    fn stone_input() -> StoneEvaluationInput {
        StoneEvaluationInput {
            raster: RasterDimensions {
                width: 4,
                height: 3,
            },
            iou_threshold: 0.5,
            confidence_threshold: 0.5,
            size_rule_id: "synthetic-min-area-v1".into(),
            provenance: provenance(),
            ground_truth: vec![
                GroundTruthStone {
                    instance_id: "gt-a".into(),
                    mask: mask(&[0, 1]),
                },
                GroundTruthStone {
                    instance_id: "gt-b".into(),
                    mask: mask(&[4, 5]),
                },
            ],
            predictions: vec![
                PredictedStone {
                    instance_id: "pred-a".into(),
                    confidence: 0.9,
                    mask: mask(&[0, 1]),
                },
                PredictedStone {
                    instance_id: "pred-b".into(),
                    confidence: 0.8,
                    mask: mask(&[5, 6]),
                },
                PredictedStone {
                    instance_id: "pred-c".into(),
                    confidence: 0.3,
                    mask: mask(&[4, 5]),
                },
            ],
        }
    }

    #[test]
    fn stone_metrics_use_one_to_one_matching_and_declared_confidence_cutoff() {
        let report = evaluate_stone_instances(&stone_input()).unwrap();
        assert_eq!(report.counts.true_positive, 1);
        assert_eq!(report.counts.false_positive, 1);
        assert_eq!(report.counts.false_negative, 1);
        assert!((report.counts.precision.unwrap() - 0.5).abs() < 1e-12);
        assert!((report.counts.recall.unwrap() - 0.5).abs() < 1e-12);
        assert!((report.counts.f1.unwrap() - 0.5).abs() < 1e-12);
        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].ground_truth_id, "gt-a");
    }

    #[test]
    fn average_precision_uses_ranked_predictions_and_interpolated_precision() {
        let report = evaluate_stone_instances(&stone_input()).unwrap();
        let ap = report.average_precision.unwrap();
        assert!(ap > 0.8 && ap < 0.84, "unexpected AP: {ap}");
    }

    #[test]
    fn duplicate_prediction_cannot_match_one_reference_twice() {
        let mut input = stone_input();
        input.predictions[1].mask = mask(&[0, 1]);
        let report = evaluate_stone_instances(&input).unwrap();
        assert_eq!(report.counts.true_positive, 1);
        assert_eq!(report.counts.false_positive, 1);
        assert_eq!(report.counts.false_negative, 1);
    }

    #[test]
    fn deterministic_ties_use_prediction_and_reference_ids() {
        let mut input = stone_input();
        input.predictions[1].confidence = 0.9;
        input.predictions[1].mask = mask(&[0, 1]);
        input.predictions[0].instance_id = "pred-z".into();
        input.predictions[1].instance_id = "pred-a".into();
        let report = evaluate_stone_instances(&input).unwrap();
        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].prediction_id, "pred-a");
    }

    #[test]
    fn invalid_unsorted_or_duplicate_mask_indices_fail_closed() {
        let mut input = stone_input();
        input.ground_truth[0].mask = mask(&[1, 0]);
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(HeritageMetricError::InvalidMask(_))
        ));
        input.ground_truth[0].mask = mask(&[0, 0]);
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(HeritageMetricError::InvalidMask(_))
        ));
    }

    #[test]
    fn out_of_bounds_mask_indices_fail_closed() {
        let mut input = stone_input();
        input.predictions[0].mask = mask(&[0, 12]);
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(HeritageMetricError::InvalidMask(_))
        ));
    }

    #[test]
    fn non_finite_and_out_of_range_confidence_fail_closed() {
        let mut input = stone_input();
        input.predictions[0].confidence = f64::NAN;
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(HeritageMetricError::InvalidConfidence(_))
        ));
        input.predictions[0].confidence = 1.01;
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(HeritageMetricError::InvalidConfidence(_))
        ));
    }

    #[test]
    fn malformed_provenance_digest_fails_closed() {
        let mut input = stone_input();
        input.provenance.dataset_sha256 = "not-a-digest".into();
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(HeritageMetricError::InvalidProvenance(_))
        ));
    }

    #[test]
    fn zero_denominators_are_not_reported_as_perfect_metrics() {
        let mut input = stone_input();
        input.ground_truth.clear();
        input.predictions.clear();
        let report = evaluate_stone_instances(&input).unwrap();
        assert_eq!(report.counts.precision, None);
        assert_eq!(report.counts.recall, None);
        assert_eq!(report.counts.f1, None);
        assert_eq!(report.average_precision, None);
    }


    #[test]
    fn predictions_without_any_reference_have_undefined_recall_and_f1() {
        let mut input = stone_input();
        input.ground_truth.clear();
        let report = evaluate_stone_instances(&input).unwrap();
        assert_eq!(report.counts.precision, Some(0.0));
        assert_eq!(report.counts.recall, None);
        assert_eq!(report.counts.f1, None);
        assert_eq!(report.average_precision, None);
    }

    #[test]
    fn no_predictions_with_reference_objects_has_zero_recall_and_ap() {
        let mut input = stone_input();
        input.predictions.clear();
        let report = evaluate_stone_instances(&input).unwrap();
        assert_eq!(report.counts.precision, None);
        assert_eq!(report.counts.recall, Some(0.0));
        assert_eq!(report.counts.f1, Some(0.0));
        assert_eq!(report.average_precision, Some(0.0));
    }

    fn wall_input() -> WallEvaluationInput {
        WallEvaluationInput {
            raster: RasterDimensions {
                width: 4,
                height: 4,
            },
            confidence_threshold: 0.5,
            coverage_thresholds: vec![0.7, 0.6, 0.5],
            provenance: provenance(),
            ground_truth_segments: vec![
                GroundTruthWallSegment {
                    segment_id: "wall-a".into(),
                    status: ReferenceSegmentStatus::Eligible,
                    mask: mask(&[0, 1, 4, 5]),
                },
                GroundTruthWallSegment {
                    segment_id: "wall-ambiguous".into(),
                    status: ReferenceSegmentStatus::Ambiguous,
                    mask: mask(&[8, 9]),
                },
            ],
            predicted_segments: vec![
                PredictedWallSegment {
                    segment_id: "fragment-a".into(),
                    confidence: 0.9,
                    mask: mask(&[0, 1]),
                },
                PredictedWallSegment {
                    segment_id: "fragment-b".into(),
                    confidence: 0.8,
                    mask: mask(&[4, 5]),
                },
            ],
        }
    }

    #[test]
    fn wall_union_coverage_allows_fragments_to_cover_one_reference_segment() {
        let report = evaluate_wall_segments(&wall_input()).unwrap();
        assert_eq!(report.counts.true_positive, 4);
        assert_eq!(report.counts.false_positive, 0);
        assert_eq!(report.counts.false_negative, 0);
        assert_eq!(report.eligible_ground_truth_segment_count, 1);
        assert_eq!(report.ambiguous_ground_truth_segment_count, 1);
        assert_eq!(
            report.ground_truth_segment_coverage[0].covered_fraction,
            Some(1.0)
        );
        assert_eq!(
            report.ground_truth_segment_coverage[1].covered_fraction,
            None
        );
        assert!(
            report
                .coverage_threshold_results
                .iter()
                .all(|row| row.covered_ground_truth_rate == Some(1.0))
        );
    }

    #[test]
    fn ambiguous_and_not_assessed_segments_are_not_silently_negatives() {
        let mut input = wall_input();
        input.ground_truth_segments[0].status = ReferenceSegmentStatus::NotAssessed;
        let report = evaluate_wall_segments(&input).unwrap();
        assert_eq!(report.eligible_ground_truth_segment_count, 0);
        assert_eq!(report.not_assessed_ground_truth_segment_count, 1);
        assert_eq!(report.counts.recall, None);
        assert_eq!(
            report.ground_truth_segment_coverage[0].covered_fraction,
            None
        );
    }

    #[test]
    fn wall_coverage_thresholds_must_be_strictly_descending() {
        let mut input = wall_input();
        input.coverage_thresholds = vec![0.5, 0.7];
        assert_eq!(
            evaluate_wall_segments(&input).unwrap_err(),
            HeritageMetricError::CoverageThresholdsNotStrictlyDescending
        );
    }

    #[test]
    fn wall_confidence_cutoff_excludes_low_confidence_fragments() {
        let mut input = wall_input();
        input.predicted_segments[1].confidence = 0.1;
        let report = evaluate_wall_segments(&input).unwrap();
        assert_eq!(report.predictions_at_confidence_threshold, 1);
        assert_eq!(report.counts.true_positive, 2);
        assert_eq!(report.counts.false_negative, 2);
    }

    #[test]
    fn empty_wall_reference_denominator_is_not_a_perfect_coverage_rate() {
        let mut input = wall_input();
        input.ground_truth_segments.clear();
        let report = evaluate_wall_segments(&input).unwrap();
        assert_eq!(
            report.coverage_threshold_results[0].covered_ground_truth_rate,
            None
        );
        assert_eq!(
            report.coverage_threshold_results[0].covered_prediction_rate,
            None
        );
    }

    #[test]
    fn duplicate_wall_ids_and_empty_masks_fail_closed() {
        let mut input = wall_input();
        input.ground_truth_segments[1].segment_id = "wall-a".into();
        assert!(matches!(
            evaluate_wall_segments(&input),
            Err(HeritageMetricError::DuplicateIdentifier(_))
        ));
        input = wall_input();
        input.ground_truth_segments[0].mask = mask(&[]);
        assert!(matches!(
            evaluate_wall_segments(&input),
            Err(HeritageMetricError::InvalidMask(_))
        ));
    }


    #[test]
    fn wall_fixture_contract_round_trips_through_serde() {
        let fixture =
            include_str!("../../schemas/examples/heritage-wall-metrics-synthetic-v0.1.json");
        let input: WallEvaluationInput = serde_json::from_str(fixture).unwrap();
        let report = evaluate_wall_segments(&input).unwrap();
        assert_eq!(report.counts.true_positive, 4);
        assert_eq!(report.ambiguous_ground_truth_segment_count, 1);
    }

    #[test]
    fn fixture_contract_round_trips_through_serde() {
        let fixture =
            include_str!("../../schemas/examples/heritage-stone-metrics-synthetic-v0.1.json");
        let input: StoneEvaluationInput = serde_json::from_str(fixture).unwrap();
        let report = evaluate_stone_instances(&input).unwrap();
        assert_eq!(report.counts.true_positive, 1);
        assert_eq!(report.ground_truth_count, 1);
    }
}
