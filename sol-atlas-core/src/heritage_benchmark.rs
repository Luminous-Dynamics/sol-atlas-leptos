// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reproducible, task-specific metrics for archaeological stone instances and walls.
//!
//! v0.1 accepts rasterized masks. It pins provenance, performs deterministic
//! confidence-ranked one-to-one instance matching, computes interpolated AP101,
//! and scores wall pixels plus segment coverage. It does not authenticate source
//! artifacts, reproduce an external evaluator without source-level comparison,
//! or prove archaeological significance.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_BENCHMARK_SCHEMA_VERSION: &str = "luminous.heritage-benchmark.v0.1";
pub const MAX_RASTER_PIXELS: u64 = 100_000_000;
pub const MAX_OBJECTS_PER_RUN: usize = 250_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationUnitKind {
    Site,
    Region,
    PaperTestPartition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkRunProvenance {
    pub schema_version: String,
    pub source_id: String,
    pub dataset_sha256: String,
    pub model_artifact_sha256: String,
    pub model_build_sha256: String,
    pub prediction_sha256: String,
    pub reference_label_sha256: String,
    pub split_manifest_sha256: String,
    pub evaluator_build_sha256: String,
    pub metric_protocol_sha256: String,
    pub rights_receipt_ref: String,
    pub evaluation_unit_id: String,
    pub evaluation_unit_kind: EvaluationUnitKind,
}

impl BenchmarkRunProvenance {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_version != HERITAGE_BENCHMARK_SCHEMA_VERSION {
            return Err(BenchmarkError::SchemaVersionMismatch);
        }
        for (name, value) in [
            ("source_id", self.source_id.as_str()),
            ("rights_receipt_ref", self.rights_receipt_ref.as_str()),
            ("evaluation_unit_id", self.evaluation_unit_id.as_str()),
        ] {
            if !non_empty(value) {
                return Err(BenchmarkError::EmptyProvenanceField(name.to_owned()));
            }
        }
        for (name, digest) in [
            ("dataset_sha256", self.dataset_sha256.as_str()),
            ("model_artifact_sha256", self.model_artifact_sha256.as_str()),
            ("model_build_sha256", self.model_build_sha256.as_str()),
            ("prediction_sha256", self.prediction_sha256.as_str()),
            ("reference_label_sha256", self.reference_label_sha256.as_str()),
            ("split_manifest_sha256", self.split_manifest_sha256.as_str()),
            ("evaluator_build_sha256", self.evaluator_build_sha256.as_str()),
            ("metric_protocol_sha256", self.metric_protocol_sha256.as_str()),
        ] {
            if !valid_sha256(digest) {
                return Err(BenchmarkError::InvalidProvenanceDigest(name.to_owned()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelMask {
    pub width: u32,
    pub height: u32,
    /// Row-major linear pixel indices, strictly increasing and unique.
    pub pixel_indices: Vec<u64>,
}

impl PixelMask {
    fn raster_pixel_count(width: u32, height: u32) -> Result<u64, BenchmarkError> {
        if width == 0 || height == 0 {
            return Err(BenchmarkError::InvalidRasterDimensions);
        }
        let count = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(BenchmarkError::InvalidRasterDimensions)?;
        if count > MAX_RASTER_PIXELS {
            return Err(BenchmarkError::RasterTooLarge);
        }
        Ok(count)
    }

    fn validate(&self, width: u32, height: u32) -> Result<(), BenchmarkError> {
        if self.width != width || self.height != height {
            return Err(BenchmarkError::MaskRasterMismatch);
        }
        let count = Self::raster_pixel_count(width, height)?;
        if self.pixel_indices.is_empty() {
            return Err(BenchmarkError::EmptyMask);
        }
        let mut previous = None;
        for index in &self.pixel_indices {
            if *index >= count {
                return Err(BenchmarkError::PixelIndexOutOfBounds);
            }
            if previous.is_some_and(|prior| prior >= *index) {
                return Err(BenchmarkError::PixelIndicesNotStrictlyIncreasing);
            }
            previous = Some(*index);
        }
        Ok(())
    }

    pub fn area_pixels(&self) -> usize {
        self.pixel_indices.len()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundTruthInstance {
    pub object_id: String,
    pub mask: PixelMask,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedInstance {
    pub object_id: String,
    pub confidence: f64,
    pub mask: PixelMask,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoneInstanceEvaluationInput {
    pub provenance: BenchmarkRunProvenance,
    pub raster_width: u32,
    pub raster_height: u32,
    pub iou_threshold: f64,
    pub operating_confidence_threshold: f64,
    /// Identifies upstream object-size eligibility and filtering semantics.
    /// The evaluator scores the supplied eligible objects; it does not infer
    /// centimetres or reinterpret this rule from pixel masks.
    pub size_rule_id: String,
    pub size_rule_description: String,
    pub ground_truth: Vec<GroundTruthInstance>,
    pub predictions: Vec<PredictedInstance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceMatch {
    pub prediction_id: String,
    pub ground_truth_id: String,
    pub confidence: f64,
    pub intersection_over_union: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoneInstanceEvaluationReport {
    pub schema_version: String,
    pub provenance: BenchmarkRunProvenance,
    pub matching_policy: String,
    pub average_precision_policy: String,
    pub raster_width: u32,
    pub raster_height: u32,
    pub iou_threshold: f64,
    pub operating_confidence_threshold: f64,
    pub size_rule_id: String,
    pub size_rule_description: String,
    pub ground_truth_count: usize,
    pub prediction_count: usize,
    pub evaluated_prediction_count: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1: Option<f64>,
    pub average_precision_101: Option<f64>,
    pub matched_pairs: Vec<InstanceMatch>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallSegment {
    pub segment_id: String,
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
    pub provenance: BenchmarkRunProvenance,
    pub raster_width: u32,
    pub raster_height: u32,
    pub prediction_confidence_threshold: f64,
    /// Fraction of a reference/predicted segment's pixels covered by the other class.
    pub segment_coverage_thresholds: Vec<f64>,
    pub ground_truth_segments: Vec<WallSegment>,
    pub predicted_segments: Vec<PredictedWallSegment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentCoverage {
    pub segment_id: String,
    pub covered_fraction: f64,
    pub qualifies_at_any_threshold: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallCoverageThresholdReport {
    pub threshold: f64,
    pub covered_ground_truth_count: usize,
    pub ground_truth_segment_count: usize,
    pub covered_ground_truth_rate: Option<f64>,
    pub covered_prediction_count: usize,
    pub prediction_segment_count: usize,
    pub covered_prediction_rate: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallEvaluationReport {
    pub schema_version: String,
    pub provenance: BenchmarkRunProvenance,
    pub raster_width: u32,
    pub raster_height: u32,
    pub prediction_confidence_threshold: f64,
    pub ground_truth_pixel_count: usize,
    pub predicted_pixel_count: usize,
    pub pixel_true_positives: usize,
    pub pixel_false_positives: usize,
    pub pixel_false_negatives: usize,
    pub pixel_precision: Option<f64>,
    pub pixel_recall: Option<f64>,
    pub pixel_f1: Option<f64>,
    pub ground_truth_segment_coverages: Vec<SegmentCoverage>,
    pub prediction_segment_coverages: Vec<SegmentCoverage>,
    pub coverage_threshold_reports: Vec<WallCoverageThresholdReport>,
    pub coverage_policy: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkError {
    SchemaVersionMismatch,
    EmptyProvenanceField(String),
    InvalidProvenanceDigest(String),
    InvalidRasterDimensions,
    RasterTooLarge,
    MaskRasterMismatch,
    EmptyMask,
    PixelIndexOutOfBounds,
    PixelIndicesNotStrictlyIncreasing,
    TooManyObjects,
    EmptyObjectId,
    DuplicateObjectId(String),
    InvalidConfidence(String),
    InvalidIouThreshold,
    InvalidOperatingThreshold,
    EmptySizeRule,
    NoCoverageThresholds,
    InvalidCoverageThreshold,
    DuplicateCoverageThreshold,
    TooManyCoverageThresholds,
}

impl fmt::Display for BenchmarkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for BenchmarkError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn safe_ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn f1_from(precision: Option<f64>, recall: Option<f64>) -> Option<f64> {
    match (precision, recall) {
        (Some(p), Some(r)) if p + r > 0.0 => Some(2.0 * p * r / (p + r)),
        (Some(_), Some(_)) => Some(0.0),
        _ => None,
    }
}

fn intersection_count(a: &[u64], b: &[u64]) -> usize {
    let (mut i, mut j, mut count) = (0usize, 0usize, 0usize);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
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

fn mask_iou(a: &PixelMask, b: &PixelMask) -> f64 {
    let intersection = intersection_count(&a.pixel_indices, &b.pixel_indices);
    let union = a.pixel_indices.len() + b.pixel_indices.len() - intersection;
    intersection as f64 / union as f64
}

fn validate_instance_inputs(
    width: u32,
    height: u32,
    ground_truth: &[GroundTruthInstance],
    predictions: &[PredictedInstance],
) -> Result<(), BenchmarkError> {
    let total = ground_truth
        .len()
        .checked_add(predictions.len())
        .ok_or(BenchmarkError::TooManyObjects)?;
    if total > MAX_OBJECTS_PER_RUN {
        return Err(BenchmarkError::TooManyObjects);
    }
    PixelMask::raster_pixel_count(width, height)?;
    let mut gt_ids = BTreeSet::new();
    for object in ground_truth {
        if !non_empty(&object.object_id) {
            return Err(BenchmarkError::EmptyObjectId);
        }
        if !gt_ids.insert(object.object_id.as_str()) {
            return Err(BenchmarkError::DuplicateObjectId(object.object_id.clone()));
        }
        object.mask.validate(width, height)?;
    }
    let mut pred_ids = BTreeSet::new();
    for object in predictions {
        if !non_empty(&object.object_id) {
            return Err(BenchmarkError::EmptyObjectId);
        }
        if !pred_ids.insert(object.object_id.as_str()) {
            return Err(BenchmarkError::DuplicateObjectId(object.object_id.clone()));
        }
        if !object.confidence.is_finite() || !(0.0..=1.0).contains(&object.confidence) {
            return Err(BenchmarkError::InvalidConfidence(object.object_id.clone()));
        }
        object.mask.validate(width, height)?;
    }
    Ok(())
}

fn ranked_predictions<'a>(
    predictions: &'a [PredictedInstance],
    min_confidence: Option<f64>,
) -> Vec<&'a PredictedInstance> {
    let mut ranked: Vec<&PredictedInstance> = predictions
        .iter()
        .filter(|pred| min_confidence.is_none_or(|cutoff| pred.confidence >= cutoff))
        .collect();
    ranked.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| a.object_id.cmp(&b.object_id))
    });
    ranked
}

fn match_ranked_predictions(
    ranked: &[&PredictedInstance],
    ground_truth: &[GroundTruthInstance],
    iou_threshold: f64,
) -> (Vec<InstanceMatch>, Vec<bool>) {
    let mut matched_gt = vec![false; ground_truth.len()];
    let mut matches = Vec::new();
    let mut true_positive_sequence = Vec::with_capacity(ranked.len());

    for prediction in ranked {
        let mut best: Option<(usize, f64)> = None;
        for (gt_index, target) in ground_truth.iter().enumerate() {
            if matched_gt[gt_index] {
                continue;
            }
            let iou = mask_iou(&prediction.mask, &target.mask);
            if iou < iou_threshold {
                continue;
            }
            match best {
                None => best = Some((gt_index, iou)),
                Some((best_index, best_iou))
                    if iou > best_iou
                        || (iou == best_iou
                            && target.object_id < ground_truth[best_index].object_id) =>
                {
                    best = Some((gt_index, iou));
                }
                _ => {}
            }
        }
        if let Some((gt_index, iou)) = best {
            matched_gt[gt_index] = true;
            matches.push(InstanceMatch {
                prediction_id: prediction.object_id.clone(),
                ground_truth_id: ground_truth[gt_index].object_id.clone(),
                confidence: prediction.confidence,
                intersection_over_union: iou,
            });
            true_positive_sequence.push(true);
        } else {
            true_positive_sequence.push(false);
        }
    }
    (matches, true_positive_sequence)
}

fn ap101(
    ranked: &[&PredictedInstance],
    ground_truth_count: usize,
    iou_threshold: f64,
    true_positive_sequence: &[bool],
) -> Option<f64> {
    if ground_truth_count == 0 {
        return None;
    }
    if ranked.is_empty() {
        return Some(0.0);
    }

    let mut cumulative_tp = 0usize;
    let mut cumulative_fp = 0usize;
    let mut points = Vec::<(f64, f64)>::with_capacity(ranked.len());
    for (index, is_tp) in true_positive_sequence.iter().enumerate() {
        if *is_tp {
            cumulative_tp += 1;
        } else {
            cumulative_fp += 1;
        }
        let recall = cumulative_tp as f64 / ground_truth_count as f64;
        let precision = cumulative_tp as f64 / (cumulative_tp + cumulative_fp) as f64;
        points.push((recall, precision));
        let _ = index;
    }

    let mut sum = 0.0;
    for step in 0..=100 {
        let recall_level = step as f64 / 100.0;
        let interpolated_precision = points
            .iter()
            .filter(|(recall, _)| *recall + f64::EPSILON >= recall_level)
            .map(|(_, precision)| *precision)
            .fold(0.0_f64, f64::max);
        sum += interpolated_precision;
    }
    let _ = (iou_threshold, ranked);
    Some(sum / 101.0)
}

pub fn evaluate_stone_instances(
    input: &StoneInstanceEvaluationInput,
) -> Result<StoneInstanceEvaluationReport, BenchmarkError> {
    input.provenance.validate()?;
    if !input.iou_threshold.is_finite()
        || input.iou_threshold <= 0.0
        || input.iou_threshold > 1.0
    {
        return Err(BenchmarkError::InvalidIouThreshold);
    }
    if !input.operating_confidence_threshold.is_finite()
        || !(0.0..=1.0).contains(&input.operating_confidence_threshold)
    {
        return Err(BenchmarkError::InvalidOperatingThreshold);
    }
    if !non_empty(&input.size_rule_id) || !non_empty(&input.size_rule_description) {
        return Err(BenchmarkError::EmptySizeRule);
    }
    validate_instance_inputs(
        input.raster_width,
        input.raster_height,
        &input.ground_truth,
        &input.predictions,
    )?;

    let operating_ranked = ranked_predictions(
        &input.predictions,
        Some(input.operating_confidence_threshold),
    );
    let (matched_pairs, operating_sequence) = match_ranked_predictions(
        &operating_ranked,
        &input.ground_truth,
        input.iou_threshold,
    );
    let true_positives = operating_sequence.iter().filter(|value| **value).count();
    let evaluated_prediction_count = operating_ranked.len();
    let false_positives = evaluated_prediction_count - true_positives;
    let false_negatives = input.ground_truth.len() - true_positives;
    let precision = safe_ratio(true_positives, evaluated_prediction_count);
    let recall = safe_ratio(true_positives, input.ground_truth.len());
    let f1 = f1_from(precision, recall);

    let all_ranked = ranked_predictions(&input.predictions, None);
    let (_, ap_sequence) =
        match_ranked_predictions(&all_ranked, &input.ground_truth, input.iou_threshold);
    let average_precision_101 = ap101(
        &all_ranked,
        input.ground_truth.len(),
        input.iou_threshold,
        &ap_sequence,
    );

    Ok(StoneInstanceEvaluationReport {
        schema_version: HERITAGE_BENCHMARK_SCHEMA_VERSION.to_owned(),
        provenance: input.provenance.clone(),
        matching_policy:
            "confidence_descending_then_prediction_id;greedy_best_iou_unmatched_ground_truth;one_to_one"
                .to_owned(),
        average_precision_policy: "101_point_interpolated_precision_envelope".to_owned(),
        raster_width: input.raster_width,
        raster_height: input.raster_height,
        iou_threshold: input.iou_threshold,
        operating_confidence_threshold: input.operating_confidence_threshold,
        size_rule_id: input.size_rule_id.clone(),
        size_rule_description: input.size_rule_description.clone(),
        ground_truth_count: input.ground_truth.len(),
        prediction_count: input.predictions.len(),
        evaluated_prediction_count,
        true_positives,
        false_positives,
        false_negatives,
        precision,
        recall,
        f1,
        average_precision_101,
        matched_pairs,
        claim_ceiling: "raster_mask_instance_metrics_under_declared_matching_and_size_rules".to_owned(),
    })
}

fn union_pixels<'a>(masks: impl IntoIterator<Item = &'a PixelMask>) -> BTreeSet<u64> {
    masks
        .into_iter()
        .flat_map(|mask| mask.pixel_indices.iter().copied())
        .collect()
}

fn validate_wall_inputs(input: &WallEvaluationInput) -> Result<(), BenchmarkError> {
    PixelMask::raster_pixel_count(input.raster_width, input.raster_height)?;
    let total = input
        .ground_truth_segments
        .len()
        .checked_add(input.predicted_segments.len())
        .ok_or(BenchmarkError::TooManyObjects)?;
    if total > MAX_OBJECTS_PER_RUN {
        return Err(BenchmarkError::TooManyObjects);
    }
    if !input.prediction_confidence_threshold.is_finite()
        || !(0.0..=1.0).contains(&input.prediction_confidence_threshold)
    {
        return Err(BenchmarkError::InvalidOperatingThreshold);
    }
    if input.segment_coverage_thresholds.is_empty() {
        return Err(BenchmarkError::NoCoverageThresholds);
    }
    if input.segment_coverage_thresholds.len() > 100 {
        return Err(BenchmarkError::TooManyCoverageThresholds);
    }
    let mut thresholds = BTreeSet::new();
    for threshold in &input.segment_coverage_thresholds {
        if !threshold.is_finite() || !(0.0..=1.0).contains(threshold) {
            return Err(BenchmarkError::InvalidCoverageThreshold);
        }
        if !thresholds.insert(threshold.to_bits()) {
            return Err(BenchmarkError::DuplicateCoverageThreshold);
        }
    }

    let mut gt_ids = BTreeSet::new();
    for segment in &input.ground_truth_segments {
        if !non_empty(&segment.segment_id) {
            return Err(BenchmarkError::EmptyObjectId);
        }
        if !gt_ids.insert(segment.segment_id.as_str()) {
            return Err(BenchmarkError::DuplicateObjectId(segment.segment_id.clone()));
        }
        segment.mask.validate(input.raster_width, input.raster_height)?;
    }
    let mut pred_ids = BTreeSet::new();
    for segment in &input.predicted_segments {
        if !non_empty(&segment.segment_id) {
            return Err(BenchmarkError::EmptyObjectId);
        }
        if !pred_ids.insert(segment.segment_id.as_str()) {
            return Err(BenchmarkError::DuplicateObjectId(segment.segment_id.clone()));
        }
        if !segment.confidence.is_finite() || !(0.0..=1.0).contains(&segment.confidence) {
            return Err(BenchmarkError::InvalidConfidence(segment.segment_id.clone()));
        }
        segment.mask.validate(input.raster_width, input.raster_height)?;
    }
    Ok(())
}

pub fn evaluate_wall_segments(
    input: &WallEvaluationInput,
) -> Result<WallEvaluationReport, BenchmarkError> {
    input.provenance.validate()?;
    validate_wall_inputs(input)?;

    let active_predictions: Vec<&PredictedWallSegment> = input
        .predicted_segments
        .iter()
        .filter(|segment| segment.confidence >= input.prediction_confidence_threshold)
        .collect();
    let gt_union = union_pixels(
        input
            .ground_truth_segments
            .iter()
            .map(|segment| &segment.mask),
    );
    let pred_union = union_pixels(active_predictions.iter().map(|segment| &segment.mask));

    let pixel_true_positives = gt_union.intersection(&pred_union).count();
    let ground_truth_pixel_count = gt_union.len();
    let predicted_pixel_count = pred_union.len();
    let pixel_false_positives = predicted_pixel_count - pixel_true_positives;
    let pixel_false_negatives = ground_truth_pixel_count - pixel_true_positives;
    let pixel_precision = safe_ratio(pixel_true_positives, predicted_pixel_count);
    let pixel_recall = safe_ratio(pixel_true_positives, ground_truth_pixel_count);
    let pixel_f1 = f1_from(pixel_precision, pixel_recall);

    let ground_truth_segment_coverages: Vec<SegmentCoverage> = input
        .ground_truth_segments
        .iter()
        .map(|segment| {
            let intersection = segment
                .mask
                .pixel_indices
                .iter()
                .filter(|pixel| pred_union.contains(pixel))
                .count();
            SegmentCoverage {
                segment_id: segment.segment_id.clone(),
                covered_fraction: intersection as f64 / segment.mask.pixel_indices.len() as f64,
                qualifies_at_any_threshold: input
                    .segment_coverage_thresholds
                    .iter()
                    .any(|threshold| {
                        intersection as f64 / segment.mask.pixel_indices.len() as f64 >= *threshold
                    }),
            }
        })
        .collect();

    let prediction_segment_coverages: Vec<SegmentCoverage> = active_predictions
        .iter()
        .map(|segment| {
            let intersection = segment
                .mask
                .pixel_indices
                .iter()
                .filter(|pixel| gt_union.contains(pixel))
                .count();
            SegmentCoverage {
                segment_id: segment.segment_id.clone(),
                covered_fraction: intersection as f64 / segment.mask.pixel_indices.len() as f64,
                qualifies_at_any_threshold: input
                    .segment_coverage_thresholds
                    .iter()
                    .any(|threshold| {
                        intersection as f64 / segment.mask.pixel_indices.len() as f64 >= *threshold
                    }),
            }
        })
        .collect();

    let coverage_threshold_reports = input
        .segment_coverage_thresholds
        .iter()
        .map(|threshold| {
            let covered_ground_truth_count = ground_truth_segment_coverages
                .iter()
                .filter(|coverage| coverage.covered_fraction >= *threshold)
                .count();
            let covered_prediction_count = prediction_segment_coverages
                .iter()
                .filter(|coverage| coverage.covered_fraction >= *threshold)
                .count();
            WallCoverageThresholdReport {
                threshold: *threshold,
                covered_ground_truth_count,
                ground_truth_segment_count: input.ground_truth_segments.len(),
                covered_ground_truth_rate: safe_ratio(
                    covered_ground_truth_count,
                    input.ground_truth_segments.len(),
                ),
                covered_prediction_count,
                prediction_segment_count: active_predictions.len(),
                covered_prediction_rate: safe_ratio(
                    covered_prediction_count,
                    active_predictions.len(),
                ),
            }
        })
        .collect();

    Ok(WallEvaluationReport {
        schema_version: HERITAGE_BENCHMARK_SCHEMA_VERSION.to_owned(),
        provenance: input.provenance.clone(),
        raster_width: input.raster_width,
        raster_height: input.raster_height,
        prediction_confidence_threshold: input.prediction_confidence_threshold,
        ground_truth_pixel_count,
        predicted_pixel_count,
        pixel_true_positives,
        pixel_false_positives,
        pixel_false_negatives,
        pixel_precision,
        pixel_recall,
        pixel_f1,
        ground_truth_segment_coverages,
        prediction_segment_coverages,
        coverage_threshold_reports,
        coverage_policy:
            "coverage_of_each_segment_by_union_of_opposite_class_masks;fragments_combine;confidence_filtered_predictions"
                .to_owned(),
        claim_ceiling: "wall_pixel_and_segment_coverage_metrics_under_declared_union_policy".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(ch: char) -> String {
        std::iter::repeat_n(ch, 64).collect()
    }

    fn provenance() -> BenchmarkRunProvenance {
        BenchmarkRunProvenance {
            schema_version: HERITAGE_BENCHMARK_SCHEMA_VERSION.to_owned(),
            source_id: "doi:10.1016/j.jas.2025.106430".to_owned(),
            dataset_sha256: digest('a'),
            model_artifact_sha256: digest('b'),
            model_build_sha256: digest('c'),
            prediction_sha256: digest('d'),
            reference_label_sha256: digest('e'),
            split_manifest_sha256: digest('f'),
            evaluator_build_sha256: digest('1'),
            metric_protocol_sha256: digest('2'),
            rights_receipt_ref: "rights://synthetic/approved".to_owned(),
            evaluation_unit_id: "site-synthetic-1".to_owned(),
            evaluation_unit_kind: EvaluationUnitKind::Site,
        }
    }

    fn mask(width: u32, height: u32, pixels: &[u64]) -> PixelMask {
        PixelMask {
            width,
            height,
            pixel_indices: pixels.to_vec(),
        }
    }

    fn stone_input() -> StoneInstanceEvaluationInput {
        StoneInstanceEvaluationInput {
            provenance: provenance(),
            raster_width: 10,
            raster_height: 10,
            iou_threshold: 0.5,
            operating_confidence_threshold: 0.5,
            size_rule_id: "min-stone-15x15".to_owned(),
            size_rule_description: "objects retained under the 15x15 source size rule".to_owned(),
            ground_truth: vec![
                GroundTruthInstance {
                    object_id: "gt-1".to_owned(),
                    mask: mask(10, 10, &[0, 1, 2, 10]),
                },
                GroundTruthInstance {
                    object_id: "gt-2".to_owned(),
                    mask: mask(10, 10, &[30, 31, 32, 40]),
                },
            ],
            predictions: vec![
                PredictedInstance {
                    object_id: "pred-1".to_owned(),
                    confidence: 0.95,
                    mask: mask(10, 10, &[0, 1, 2, 10]),
                },
                PredictedInstance {
                    object_id: "pred-2".to_owned(),
                    confidence: 0.80,
                    mask: mask(10, 10, &[30, 31, 32, 40]),
                },
                PredictedInstance {
                    object_id: "pred-duplicate".to_owned(),
                    confidence: 0.60,
                    mask: mask(10, 10, &[0, 1, 2, 10]),
                },
            ],
        }
    }

    fn wall_input() -> WallEvaluationInput {
        WallEvaluationInput {
            provenance: provenance(),
            raster_width: 10,
            raster_height: 10,
            prediction_confidence_threshold: 0.5,
            segment_coverage_thresholds: vec![0.7, 0.6, 0.5],
            ground_truth_segments: vec![
                WallSegment {
                    segment_id: "gt-wall-1".to_owned(),
                    mask: mask(10, 10, &[0, 1, 2, 3]),
                },
                WallSegment {
                    segment_id: "gt-wall-2".to_owned(),
                    mask: mask(10, 10, &[20, 21, 22, 23]),
                },
            ],
            predicted_segments: vec![
                PredictedWallSegment {
                    segment_id: "pred-wall-1a".to_owned(),
                    confidence: 0.9,
                    mask: mask(10, 10, &[0, 1]),
                },
                PredictedWallSegment {
                    segment_id: "pred-wall-1b".to_owned(),
                    confidence: 0.8,
                    mask: mask(10, 10, &[2, 3]),
                },
                PredictedWallSegment {
                    segment_id: "pred-wall-2".to_owned(),
                    confidence: 0.4,
                    mask: mask(10, 10, &[20, 21, 22, 23]),
                },
                PredictedWallSegment {
                    segment_id: "false-positive".to_owned(),
                    confidence: 0.7,
                    mask: mask(10, 10, &[40, 41]),
                },
            ],
        }
    }

    #[test]
    fn stone_instance_scoring_is_one_to_one_and_confidence_ranked() {
        let report = evaluate_stone_instances(&stone_input()).expect("stone report");
        assert_eq!(report.true_positives, 2);
        assert_eq!(report.false_positives, 1);
        assert_eq!(report.false_negatives, 0);
        assert_eq!(report.precision, Some(2.0 / 3.0));
        assert_eq!(report.recall, Some(1.0));
        assert_eq!(report.f1, Some(0.8));
        assert_eq!(report.matched_pairs.len(), 2);
        assert_eq!(report.average_precision_101, Some(1.0));
    }

    #[test]
    fn instance_iou_threshold_changes_matching_not_mask_validation() {
        let mut input = stone_input();
        input.iou_threshold = 1.0;
        input.predictions[0].mask = mask(10, 10, &[0, 1, 2]);
        let report = evaluate_stone_instances(&input).expect("valid masks");
        assert_eq!(report.true_positives, 1);
        assert_eq!(report.false_negatives, 1);
    }

    #[test]
    fn confidence_threshold_changes_operating_point_but_not_ap101() {
        let mut input = stone_input();
        let base = evaluate_stone_instances(&input).expect("base report");
        input.operating_confidence_threshold = 0.9;
        let high = evaluate_stone_instances(&input).expect("high confidence report");
        assert_eq!(base.average_precision_101, high.average_precision_101);
        assert_eq!(high.evaluated_prediction_count, 1);
        assert_eq!(high.true_positives, 1);
    }

    #[test]
    fn no_ground_truth_does_not_report_perfect_recall_or_ap() {
        let mut input = stone_input();
        input.ground_truth.clear();
        let report = evaluate_stone_instances(&input).expect("empty reference set");
        assert_eq!(report.recall, None);
        assert_eq!(report.average_precision_101, None);
        assert_eq!(report.f1, None);
    }

    #[test]
    fn empty_predictions_report_zero_recall_when_ground_truth_exists() {
        let mut input = stone_input();
        input.predictions.clear();
        let report = evaluate_stone_instances(&input).expect("no detections");
        assert_eq!(report.precision, None);
        assert_eq!(report.recall, Some(0.0));
        assert_eq!(report.f1, Some(0.0));
        assert_eq!(report.average_precision_101, Some(0.0));
    }

    #[test]
    fn duplicate_ids_and_duplicate_mask_pixels_fail_closed() {
        let mut input = stone_input();
        input.predictions[1].object_id = input.predictions[0].object_id.clone();
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(BenchmarkError::DuplicateObjectId(_))
        ));

        input = stone_input();
        input.ground_truth[0].mask.pixel_indices = vec![0, 0];
        assert_eq!(
            evaluate_stone_instances(&input),
            Err(BenchmarkError::PixelIndicesNotStrictlyIncreasing)
        );
    }

    #[test]
    fn invalid_provenance_digest_and_confidence_are_rejected() {
        let mut input = stone_input();
        input.provenance.model_artifact_sha256 = "not-a-digest".to_owned();
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(BenchmarkError::InvalidProvenanceDigest(_))
        ));
        input = stone_input();
        input.predictions[0].confidence = f64::NAN;
        assert!(matches!(
            evaluate_stone_instances(&input),
            Err(BenchmarkError::InvalidConfidence(_))
        ));
    }

    #[test]
    fn wall_pixel_metrics_use_union_masks_and_confidence_filter() {
        let report = evaluate_wall_segments(&wall_input()).expect("wall report");
        assert_eq!(report.ground_truth_pixel_count, 8);
        assert_eq!(report.predicted_pixel_count, 6);
        assert_eq!(report.pixel_true_positives, 4);
        assert_eq!(report.pixel_false_positives, 2);
        assert_eq!(report.pixel_false_negatives, 4);
        assert_eq!(report.pixel_precision, Some(2.0 / 3.0));
        assert_eq!(report.pixel_recall, Some(0.5));
        assert_eq!(report.pixel_f1, Some(4.0 / 7.0));
    }

    #[test]
    fn wall_segment_fragments_combine_for_union_coverage() {
        let report = evaluate_wall_segments(&wall_input()).expect("wall report");
        assert_eq!(report.ground_truth_segment_coverages[0].covered_fraction, 1.0);
        assert_eq!(report.ground_truth_segment_coverages[1].covered_fraction, 0.0);
        assert_eq!(report.prediction_segment_coverages[0].covered_fraction, 1.0);
        assert_eq!(report.prediction_segment_coverages[1].covered_fraction, 1.0);
    }

    #[test]
    fn wall_coverage_curve_is_computed_at_every_declared_threshold() {
        let report = evaluate_wall_segments(&wall_input()).expect("wall report");
        assert_eq!(report.coverage_threshold_reports.len(), 3);
        assert_eq!(report.coverage_threshold_reports[0].threshold, 0.7);
        assert_eq!(report.coverage_threshold_reports[0].covered_ground_truth_count, 1);
        assert_eq!(report.coverage_threshold_reports[0].covered_prediction_count, 2);
        assert_eq!(report.coverage_threshold_reports[2].covered_ground_truth_rate, Some(0.5));
    }

    #[test]
    fn invalid_coverage_threshold_and_mask_dimensions_are_rejected() {
        let mut input = wall_input();
        input.segment_coverage_thresholds = vec![0.5, 1.5];
        assert_eq!(
            evaluate_wall_segments(&input),
            Err(BenchmarkError::InvalidCoverageThreshold)
        );
        input = wall_input();
        input.predicted_segments[0].mask.width = 9;
        assert_eq!(
            evaluate_wall_segments(&input),
            Err(BenchmarkError::MaskRasterMismatch)
        );
    }

    #[test]
    fn wall_empty_denominators_are_explicit() {
        let mut input = wall_input();
        input.ground_truth_segments.clear();
        let report = evaluate_wall_segments(&input).expect("empty GT report");
        assert_eq!(report.pixel_recall, None);
        assert_eq!(report.pixel_f1, None);
        assert_eq!(
            report.coverage_threshold_reports[0].covered_ground_truth_rate,
            None
        );
    }

    #[test]
    fn benchmark_schemas_and_synthetic_fixtures_parse() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-benchmark-v0.1.schema.json"
        ))
        .expect("schema json");
        let stone: StoneInstanceEvaluationInput = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-benchmark-stone-synthetic-v0.1.json"
        ))
        .expect("stone fixture");
        let wall: WallEvaluationInput = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-benchmark-wall-synthetic-v0.1.json"
        ))
        .expect("wall fixture");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(evaluate_stone_instances(&stone).is_ok());
        assert!(evaluate_wall_segments(&wall).is_ok());
    }
}
