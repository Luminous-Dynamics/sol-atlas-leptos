// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Design-aware audit of both predicted-positive and predicted-negative
//! landscape strata. Known first-order inclusion probabilities support
//! Horvitz-Thompson population totals. Unknown or unobservable samples are never
//! treated as negatives; if any selected unit is unresolved, population-level
//! confusion totals and ratio metrics are withheld in v0.1.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const NEGATIVE_AUDIT_SCHEMA_VERSION: &str = "luminous.heritage-negative-audit.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStratumKind {
    HighScoreCandidates,
    MediumScoreCandidates,
    LowScoreCandidates,
    PredictedNegativeLandscape,
    ModelAbstained,
    DataUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSamplingDesign {
    /// Every unit in this stratum has first-order inclusion probability n_h/N_h.
    StratifiedSimpleRandomWithoutReplacement,
    /// The design uses unequal probabilities; each sampled unit must provide
    /// its actual first-order inclusion probability.
    KnownUnequalFirstOrderProbability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelPrediction {
    Positive,
    Negative,
    Abstained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEvidenceTier {
    FieldAssessed,
    IndependentlyExpertReviewed,
    CatalogueReference,
    ModelProposed,
    Unknown,
}

impl AuditEvidenceTier {
    fn is_independent_ground_evidence(self) -> bool {
        matches!(
            self,
            Self::FieldAssessed | Self::IndependentlyExpertReviewed
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AuditGroundTruth {
    ConfirmedPositive {
        adjudication_ref: String,
        evidence_tier: AuditEvidenceTier,
        reviewer_ref: String,
        authority_ref: String,
        rights_receipt_ref: String,
    },
    ConfirmedNegative {
        adjudication_ref: String,
        evidence_tier: AuditEvidenceTier,
        reviewer_ref: String,
        authority_ref: String,
        rights_receipt_ref: String,
        detectability_assessment_ref: String,
    },
    Unknown {
        reason_ref: String,
    },
    Unobservable {
        reason_ref: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditPopulationStratum {
    pub stratum_id: String,
    pub kind: AuditStratumKind,
    pub population_unit_count: u64,
    /// Number of units drawn from this stratum, including unknown/unobservable
    /// selected units that did not receive an adjudicated outcome.
    pub selected_unit_count: u64,
    pub sampling_design: AuditSamplingDesign,
    pub design_reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegativeAuditSample {
    pub sample_id: String,
    pub unit_id: String,
    pub landscape_id: String,
    pub stratum_id: String,
    pub inclusion_probability: f64,
    pub model_prediction: ModelPrediction,
    pub ground_truth: AuditGroundTruth,
    pub reviewer_hours: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NegativeAuditConfig {
    pub probability_tolerance: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfusionTotals {
    pub true_positive: f64,
    pub false_positive: f64,
    pub true_negative: f64,
    pub false_negative: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StratumAuditSummary {
    pub stratum_id: String,
    pub kind: AuditStratumKind,
    pub population_unit_count: u64,
    pub selected_unit_count: u64,
    pub adjudicated_positive_count: usize,
    pub adjudicated_negative_count: usize,
    pub unknown_count: usize,
    pub unobservable_count: usize,
    pub unresolved_fraction: f64,
    pub weighted_false_negative_total: Option<f64>,
    pub weighted_true_positive_total: Option<f64>,
    pub weighted_false_positive_total: Option<f64>,
    pub weighted_true_negative_total: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignWeightedMetrics {
    pub estimated_population_confusion: ConfusionTotals,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1: Option<f64>,
    pub false_negative_rate: Option<f64>,
    pub false_negative_total: f64,
    pub sampled_false_negative_discoveries_per_100_reviewer_hours: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegativeAuditReport {
    pub schema_version: String,
    pub population_frame_id: String,
    pub sampling_design_id: String,
    pub total_population_units: u64,
    pub selected_unit_count: usize,
    pub adjudicated_positive_count: usize,
    pub adjudicated_negative_count: usize,
    pub unknown_count: usize,
    pub unobservable_count: usize,
    pub unresolved_count: usize,
    pub model_abstained_count: usize,
    pub adjudication_coverage: f64,
    pub sampled_landscape_count: usize,
    pub kish_effective_sample_size: f64,
    /// Known to be a probability-sample point estimate only when all selected
    /// units are adjudicated. No confidence interval is emitted in v0.1.
    pub design_weighted_metrics: Option<DesignWeightedMetrics>,
    /// Raw sample confusion counts among adjudicated units; not population totals.
    pub adjudicated_sample_confusion: ConfusionTotals,
    pub false_negative_sample_count: usize,
    pub reviewer_hours: f64,
    pub stratum_summaries: Vec<StratumAuditSummary>,
    pub confidence_interval_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NegativeAuditError {
    EmptyPopulationFrameId,
    EmptySamplingDesignId,
    EmptyStrata,
    EmptySamples,
    InvalidConfig,
    EmptyStratumId,
    DuplicateStratumId(String),
    EmptyStratumDesignReference {
        stratum_id: String,
    },
    ZeroPopulationStratum {
        stratum_id: String,
    },
    InvalidSelectedUnitCount {
        stratum_id: String,
    },
    SelectedCountMismatch {
        stratum_id: String,
        declared: u64,
        actual: usize,
    },
    EmptySampleId,
    DuplicateSampleId(String),
    EmptyUnitId {
        sample_id: String,
    },
    DuplicateUnitId(String),
    EmptyLandscapeId {
        sample_id: String,
    },
    UnknownStratum {
        sample_id: String,
        stratum_id: String,
    },
    InvalidInclusionProbability {
        sample_id: String,
    },
    IncorrectSrsInclusionProbability {
        sample_id: String,
        expected: f64,
        actual: f64,
    },
    InvalidReviewerHours {
        sample_id: String,
    },
    InvalidExpansionWeight {
        sample_id: String,
    },
    NonFiniteWeightedTotals,
    NonFiniteReviewerHours,
    PopulationCountOverflow,
    MissingAdjudicationReference {
        sample_id: String,
    },
    InvalidEvidenceTier {
        sample_id: String,
    },
    MissingReviewerReference {
        sample_id: String,
    },
    MissingAuthorityReference {
        sample_id: String,
    },
    MissingRightsReceipt {
        sample_id: String,
    },
    MissingDetectabilityAssessment {
        sample_id: String,
    },
    MissingUnresolvedReason {
        sample_id: String,
    },
    PredictionStratumMismatch {
        sample_id: String,
    },
}

impl fmt::Display for NegativeAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage negative-space audit error: {self:?}")
    }
}

impl std::error::Error for NegativeAuditError {}

fn validate_ground_truth(sample: &NegativeAuditSample) -> Result<(), NegativeAuditError> {
    match &sample.ground_truth {
        AuditGroundTruth::ConfirmedPositive {
            adjudication_ref,
            evidence_tier,
            reviewer_ref,
            authority_ref,
            rights_receipt_ref,
        } => {
            if adjudication_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingAdjudicationReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if !evidence_tier.is_independent_ground_evidence() {
                return Err(NegativeAuditError::InvalidEvidenceTier {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if reviewer_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingReviewerReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if authority_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingAuthorityReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if rights_receipt_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingRightsReceipt {
                    sample_id: sample.sample_id.clone(),
                });
            }
        }
        AuditGroundTruth::ConfirmedNegative {
            adjudication_ref,
            evidence_tier,
            reviewer_ref,
            authority_ref,
            rights_receipt_ref,
            detectability_assessment_ref,
        } => {
            if adjudication_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingAdjudicationReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if !evidence_tier.is_independent_ground_evidence() {
                return Err(NegativeAuditError::InvalidEvidenceTier {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if reviewer_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingReviewerReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if authority_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingAuthorityReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if rights_receipt_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingRightsReceipt {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if detectability_assessment_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingDetectabilityAssessment {
                    sample_id: sample.sample_id.clone(),
                });
            }
        }
        AuditGroundTruth::Unknown { reason_ref }
        | AuditGroundTruth::Unobservable { reason_ref } => {
            if reason_ref.trim().is_empty() {
                return Err(NegativeAuditError::MissingUnresolvedReason {
                    sample_id: sample.sample_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_prediction_stratum(
    sample: &NegativeAuditSample,
    kind: AuditStratumKind,
) -> Result<(), NegativeAuditError> {
    let consistent = match kind {
        AuditStratumKind::HighScoreCandidates | AuditStratumKind::MediumScoreCandidates => {
            sample.model_prediction == ModelPrediction::Positive
        }
        AuditStratumKind::LowScoreCandidates | AuditStratumKind::PredictedNegativeLandscape => {
            sample.model_prediction == ModelPrediction::Negative
        }
        AuditStratumKind::ModelAbstained | AuditStratumKind::DataUnavailable => {
            sample.model_prediction == ModelPrediction::Abstained
        }
    };
    if !consistent {
        return Err(NegativeAuditError::PredictionStratumMismatch {
            sample_id: sample.sample_id.clone(),
        });
    }
    Ok(())
}

/// Evaluate a stratified probability audit with known unit-level inclusion
/// probabilities. The finite-population confusion totals use Horvitz-Thompson
/// expansion. If even one selected unit is unknown or unobservable, v0.1
/// withholds population-level precision/recall rather than treating it as a
/// negative or assuming response is ignorable.
pub fn evaluate_negative_space_audit(
    population_frame_id: &str,
    sampling_design_id: &str,
    strata: &[AuditPopulationStratum],
    samples: &[NegativeAuditSample],
    config: NegativeAuditConfig,
) -> Result<NegativeAuditReport, NegativeAuditError> {
    if population_frame_id.trim().is_empty() {
        return Err(NegativeAuditError::EmptyPopulationFrameId);
    }
    if sampling_design_id.trim().is_empty() {
        return Err(NegativeAuditError::EmptySamplingDesignId);
    }
    if strata.is_empty() {
        return Err(NegativeAuditError::EmptyStrata);
    }
    if samples.is_empty() {
        return Err(NegativeAuditError::EmptySamples);
    }
    if !config.probability_tolerance.is_finite()
        || config.probability_tolerance <= 0.0
        || config.probability_tolerance > 1e-6
    {
        return Err(NegativeAuditError::InvalidConfig);
    }

    let mut strata_by_id = BTreeMap::new();
    for stratum in strata {
        if stratum.stratum_id.trim().is_empty() {
            return Err(NegativeAuditError::EmptyStratumId);
        }
        if strata_by_id
            .insert(stratum.stratum_id.as_str(), stratum)
            .is_some()
        {
            return Err(NegativeAuditError::DuplicateStratumId(
                stratum.stratum_id.clone(),
            ));
        }
        if stratum.design_reference.trim().is_empty() {
            return Err(NegativeAuditError::EmptyStratumDesignReference {
                stratum_id: stratum.stratum_id.clone(),
            });
        }
        if stratum.population_unit_count == 0 {
            return Err(NegativeAuditError::ZeroPopulationStratum {
                stratum_id: stratum.stratum_id.clone(),
            });
        }
        if stratum.selected_unit_count == 0
            || stratum.selected_unit_count > stratum.population_unit_count
        {
            return Err(NegativeAuditError::InvalidSelectedUnitCount {
                stratum_id: stratum.stratum_id.clone(),
            });
        }
    }

    let mut sample_ids = BTreeSet::new();
    let mut unit_ids = BTreeSet::new();
    let mut counts_by_stratum: BTreeMap<String, usize> = BTreeMap::new();
    let mut summaries: BTreeMap<String, StratumAuditSummary> = strata
        .iter()
        .map(|stratum| {
            (
                stratum.stratum_id.clone(),
                StratumAuditSummary {
                    stratum_id: stratum.stratum_id.clone(),
                    kind: stratum.kind,
                    population_unit_count: stratum.population_unit_count,
                    selected_unit_count: stratum.selected_unit_count,
                    adjudicated_positive_count: 0,
                    adjudicated_negative_count: 0,
                    unknown_count: 0,
                    unobservable_count: 0,
                    unresolved_fraction: 0.0,
                    weighted_false_negative_total: None,
                    weighted_true_positive_total: None,
                    weighted_false_positive_total: None,
                    weighted_true_negative_total: None,
                },
            )
        })
        .collect();

    let mut raw_confusion = ConfusionTotals::default();
    let mut ht_confusion = ConfusionTotals::default();
    let mut all_resolved = true;
    let mut selected_unit_count = 0usize;
    let mut positive_count = 0usize;
    let mut negative_count = 0usize;
    let mut unknown_count = 0usize;
    let mut unobservable_count = 0usize;
    let mut sampled_landscapes = BTreeSet::new();
    let mut reviewer_hours = 0.0;
    let mut weight_sum = 0.0;
    let mut squared_weight_sum = 0.0;
    let mut false_negative_sample_count = 0usize;

    for sample in samples {
        if sample.sample_id.trim().is_empty() {
            return Err(NegativeAuditError::EmptySampleId);
        }
        if !sample_ids.insert(sample.sample_id.as_str()) {
            return Err(NegativeAuditError::DuplicateSampleId(
                sample.sample_id.clone(),
            ));
        }
        if sample.unit_id.trim().is_empty() {
            return Err(NegativeAuditError::EmptyUnitId {
                sample_id: sample.sample_id.clone(),
            });
        }
        if !unit_ids.insert(sample.unit_id.as_str()) {
            return Err(NegativeAuditError::DuplicateUnitId(sample.unit_id.clone()));
        }
        if sample.landscape_id.trim().is_empty() {
            return Err(NegativeAuditError::EmptyLandscapeId {
                sample_id: sample.sample_id.clone(),
            });
        }
        let Some(stratum) = strata_by_id.get(sample.stratum_id.as_str()) else {
            return Err(NegativeAuditError::UnknownStratum {
                sample_id: sample.sample_id.clone(),
                stratum_id: sample.stratum_id.clone(),
            });
        };
        if !sample.inclusion_probability.is_finite()
            || sample.inclusion_probability <= 0.0
            || sample.inclusion_probability > 1.0
        {
            return Err(NegativeAuditError::InvalidInclusionProbability {
                sample_id: sample.sample_id.clone(),
            });
        }
        if stratum.sampling_design == AuditSamplingDesign::StratifiedSimpleRandomWithoutReplacement
        {
            let expected =
                stratum.selected_unit_count as f64 / stratum.population_unit_count as f64;
            if (sample.inclusion_probability - expected).abs() > config.probability_tolerance {
                return Err(NegativeAuditError::IncorrectSrsInclusionProbability {
                    sample_id: sample.sample_id.clone(),
                    expected,
                    actual: sample.inclusion_probability,
                });
            }
        }
        if !sample.reviewer_hours.is_finite() || sample.reviewer_hours < 0.0 {
            return Err(NegativeAuditError::InvalidReviewerHours {
                sample_id: sample.sample_id.clone(),
            });
        }
        validate_ground_truth(sample)?;
        validate_prediction_stratum(sample, stratum.kind)?;

        selected_unit_count += 1;
        sampled_landscapes.insert(sample.landscape_id.as_str());
        reviewer_hours += sample.reviewer_hours;
        if !reviewer_hours.is_finite() {
            return Err(NegativeAuditError::NonFiniteReviewerHours);
        }
        let weight = 1.0 / sample.inclusion_probability;
        if !weight.is_finite() || weight <= 0.0 {
            return Err(NegativeAuditError::InvalidExpansionWeight {
                sample_id: sample.sample_id.clone(),
            });
        }
        weight_sum += weight;
        squared_weight_sum += weight * weight;
        if !weight_sum.is_finite() || !squared_weight_sum.is_finite() {
            return Err(NegativeAuditError::NonFiniteWeightedTotals);
        }
        *counts_by_stratum
            .entry(sample.stratum_id.clone())
            .or_default() += 1;

        let truth_positive = matches!(
            &sample.ground_truth,
            AuditGroundTruth::ConfirmedPositive { .. }
        );
        let truth_negative = matches!(
            &sample.ground_truth,
            AuditGroundTruth::ConfirmedNegative { .. }
        );

        if matches!(&sample.ground_truth, AuditGroundTruth::Unknown { .. }) {
            unknown_count += 1;
            summaries.get_mut(&sample.stratum_id).unwrap().unknown_count += 1;
            all_resolved = false;
            continue;
        }
        if matches!(&sample.ground_truth, AuditGroundTruth::Unobservable { .. }) {
            unobservable_count += 1;
            summaries
                .get_mut(&sample.stratum_id)
                .unwrap()
                .unobservable_count += 1;
            all_resolved = false;
            continue;
        }

        if truth_positive {
            positive_count += 1;
            summaries
                .get_mut(&sample.stratum_id)
                .unwrap()
                .adjudicated_positive_count += 1;
        }
        if truth_negative {
            negative_count += 1;
            summaries
                .get_mut(&sample.stratum_id)
                .unwrap()
                .adjudicated_negative_count += 1;
        }

        match (sample.model_prediction, truth_positive, truth_negative) {
            (ModelPrediction::Positive, true, false) => {
                raw_confusion.true_positive += 1.0;
                ht_confusion.true_positive += weight;
            }
            (ModelPrediction::Positive, false, true) => {
                raw_confusion.false_positive += 1.0;
                ht_confusion.false_positive += weight;
            }
            (ModelPrediction::Negative, false, true) => {
                raw_confusion.true_negative += 1.0;
                ht_confusion.true_negative += weight;
            }
            (ModelPrediction::Negative, true, false) => {
                raw_confusion.false_negative += 1.0;
                ht_confusion.false_negative += weight;
                false_negative_sample_count += 1;
            }
            (ModelPrediction::Abstained, _, _) => {
                // Abstentions are not forced into positive/negative confusion.
                all_resolved = false;
            }
            _ => {
                return Err(NegativeAuditError::PredictionStratumMismatch {
                    sample_id: sample.sample_id.clone(),
                });
            }
        }
    }

    for stratum in strata {
        let actual = counts_by_stratum
            .get(&stratum.stratum_id)
            .copied()
            .unwrap_or(0);
        if actual as u64 != stratum.selected_unit_count {
            return Err(NegativeAuditError::SelectedCountMismatch {
                stratum_id: stratum.stratum_id.clone(),
                declared: stratum.selected_unit_count,
                actual,
            });
        }
        let summary = summaries.get_mut(&stratum.stratum_id).unwrap();
        summary.unresolved_fraction =
            (summary.unknown_count + summary.unobservable_count) as f64 / actual as f64;
    }

    let unresolved_count = unknown_count + unobservable_count;
    let model_abstained_count = samples
        .iter()
        .filter(|sample| sample.model_prediction == ModelPrediction::Abstained)
        .count();
    let adjudication_coverage =
        (selected_unit_count - unresolved_count) as f64 / selected_unit_count as f64;
    let effective_sample_size = if squared_weight_sum > 0.0 {
        weight_sum * weight_sum / squared_weight_sum
    } else {
        0.0
    };
    let estimated_metrics = if all_resolved {
        for sample in samples {
            let summary = summaries.get_mut(&sample.stratum_id).unwrap();
            let weight = 1.0 / sample.inclusion_probability;
            match (sample.model_prediction, &sample.ground_truth) {
                (ModelPrediction::Positive, AuditGroundTruth::ConfirmedPositive { .. }) => {
                    summary.weighted_true_positive_total =
                        Some(summary.weighted_true_positive_total.unwrap_or(0.0) + weight);
                }
                (ModelPrediction::Positive, AuditGroundTruth::ConfirmedNegative { .. }) => {
                    summary.weighted_false_positive_total =
                        Some(summary.weighted_false_positive_total.unwrap_or(0.0) + weight);
                }
                (ModelPrediction::Negative, AuditGroundTruth::ConfirmedNegative { .. }) => {
                    summary.weighted_true_negative_total =
                        Some(summary.weighted_true_negative_total.unwrap_or(0.0) + weight);
                }
                (ModelPrediction::Negative, AuditGroundTruth::ConfirmedPositive { .. }) => {
                    summary.weighted_false_negative_total =
                        Some(summary.weighted_false_negative_total.unwrap_or(0.0) + weight);
                }
                (ModelPrediction::Abstained, _) => {}
                _ => {}
            }
        }
        let precision_denominator = ht_confusion.true_positive + ht_confusion.false_positive;
        let recall_denominator = ht_confusion.true_positive + ht_confusion.false_negative;
        let f1_denominator = 2.0 * ht_confusion.true_positive
            + ht_confusion.false_positive
            + ht_confusion.false_negative;
        let total_review_hours = reviewer_hours;
        Some(DesignWeightedMetrics {
            estimated_population_confusion: ht_confusion,
            precision: (precision_denominator > 0.0)
                .then_some(ht_confusion.true_positive / precision_denominator),
            recall: (recall_denominator > 0.0)
                .then_some(ht_confusion.true_positive / recall_denominator),
            f1: (f1_denominator > 0.0).then_some(2.0 * ht_confusion.true_positive / f1_denominator),
            false_negative_rate: (recall_denominator > 0.0)
                .then_some(ht_confusion.false_negative / recall_denominator),
            false_negative_total: ht_confusion.false_negative,
            sampled_false_negative_discoveries_per_100_reviewer_hours: (total_review_hours > 0.0)
                .then_some(false_negative_sample_count as f64 * 100.0 / total_review_hours),
        })
    } else {
        None
    };

    let stratum_summaries: Vec<StratumAuditSummary> = summaries.into_values().collect();
    let total_population_units = strata
        .iter()
        .try_fold(0_u64, |total, stratum| {
            total.checked_add(stratum.population_unit_count)
        })
        .ok_or(NegativeAuditError::PopulationCountOverflow)?;

    Ok(NegativeAuditReport {
        schema_version: NEGATIVE_AUDIT_SCHEMA_VERSION.to_owned(),
        population_frame_id: population_frame_id.to_owned(),
        sampling_design_id: sampling_design_id.to_owned(),
        total_population_units,
        selected_unit_count,
        adjudicated_positive_count: positive_count,
        adjudicated_negative_count: negative_count,
        unknown_count,
        unobservable_count,
        unresolved_count,
        model_abstained_count,
        adjudication_coverage,
        sampled_landscape_count: sampled_landscapes.len(),
        kish_effective_sample_size: effective_sample_size,
        design_weighted_metrics: estimated_metrics,
        adjudicated_sample_confusion: raw_confusion,
        false_negative_sample_count,
        reviewer_hours,
        stratum_summaries,
        confidence_interval_status:
            "not_computed_v0_1_joint_inclusion_or_cluster_variance_design_required".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stratum(
        id: &str,
        kind: AuditStratumKind,
        population: u64,
        selected: u64,
    ) -> AuditPopulationStratum {
        AuditPopulationStratum {
            stratum_id: id.into(),
            kind,
            population_unit_count: population,
            selected_unit_count: selected,
            sampling_design: AuditSamplingDesign::StratifiedSimpleRandomWithoutReplacement,
            design_reference: format!("urn:synthetic:design:{id}"),
        }
    }

    fn sample(
        id: &str,
        unit: &str,
        landscape: &str,
        stratum: &str,
        model_prediction: ModelPrediction,
        ground_truth: AuditGroundTruth,
        pi: f64,
    ) -> NegativeAuditSample {
        NegativeAuditSample {
            sample_id: id.into(),
            unit_id: unit.into(),
            landscape_id: landscape.into(),
            stratum_id: stratum.into(),
            inclusion_probability: pi,
            model_prediction,
            ground_truth,
            reviewer_hours: 1.0,
        }
    }

    fn positive() -> AuditGroundTruth {
        AuditGroundTruth::ConfirmedPositive {
            adjudication_ref: "urn:synthetic:positive-adjudication".into(),
            evidence_tier: AuditEvidenceTier::FieldAssessed,
            reviewer_ref: "urn:synthetic:reviewer".into(),
            authority_ref: "urn:synthetic:authority".into(),
            rights_receipt_ref: "urn:synthetic:rights".into(),
        }
    }

    fn negative() -> AuditGroundTruth {
        AuditGroundTruth::ConfirmedNegative {
            adjudication_ref: "urn:synthetic:negative-adjudication".into(),
            evidence_tier: AuditEvidenceTier::FieldAssessed,
            reviewer_ref: "urn:synthetic:reviewer".into(),
            authority_ref: "urn:synthetic:authority".into(),
            rights_receipt_ref: "urn:synthetic:rights".into(),
            detectability_assessment_ref: "urn:synthetic:detectability".into(),
        }
    }

    #[test]
    fn stratified_ht_estimator_expands_confusion_totals_by_inclusion_probability() {
        let strata = vec![
            stratum(
                "predicted-positive",
                AuditStratumKind::HighScoreCandidates,
                100,
                2,
            ),
            stratum(
                "predicted-negative",
                AuditStratumKind::PredictedNegativeLandscape,
                200,
                2,
            ),
        ];
        let samples = vec![
            sample(
                "s1",
                "u1",
                "l1",
                "predicted-positive",
                ModelPrediction::Positive,
                positive(),
                0.02,
            ),
            sample(
                "s2",
                "u2",
                "l1",
                "predicted-positive",
                ModelPrediction::Positive,
                negative(),
                0.02,
            ),
            sample(
                "s3",
                "u3",
                "l2",
                "predicted-negative",
                ModelPrediction::Negative,
                positive(),
                0.01,
            ),
            sample(
                "s4",
                "u4",
                "l2",
                "predicted-negative",
                ModelPrediction::Negative,
                negative(),
                0.01,
            ),
        ];
        let report = evaluate_negative_space_audit(
            "synthetic-frame",
            "synthetic-design",
            &strata,
            &samples,
            NegativeAuditConfig {
                probability_tolerance: 1e-9,
            },
        )
        .unwrap();

        let weighted = report.design_weighted_metrics.unwrap();
        assert_eq!(weighted.estimated_population_confusion.true_positive, 50.0);
        assert_eq!(weighted.estimated_population_confusion.false_positive, 50.0);
        assert_eq!(
            weighted.estimated_population_confusion.false_negative,
            100.0
        );
        assert_eq!(weighted.estimated_population_confusion.true_negative, 100.0);
        assert!((weighted.precision.unwrap() - 0.5).abs() < 1e-12);
        assert!((weighted.recall.unwrap() - (1.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn predicted_negative_stratum_detects_false_negatives_instead_of_assuming_zero() {
        let strata = vec![stratum(
            "negative-space",
            AuditStratumKind::PredictedNegativeLandscape,
            1000,
            4,
        )];
        let samples = vec![
            sample(
                "s1",
                "u1",
                "l1",
                "negative-space",
                ModelPrediction::Negative,
                positive(),
                0.004,
            ),
            sample(
                "s2",
                "u2",
                "l1",
                "negative-space",
                ModelPrediction::Negative,
                negative(),
                0.004,
            ),
            sample(
                "s3",
                "u3",
                "l2",
                "negative-space",
                ModelPrediction::Negative,
                positive(),
                0.004,
            ),
            sample(
                "s4",
                "u4",
                "l2",
                "negative-space",
                ModelPrediction::Negative,
                negative(),
                0.004,
            ),
        ];
        let report = evaluate_negative_space_audit(
            "frame",
            "design",
            &strata,
            &samples,
            NegativeAuditConfig {
                probability_tolerance: 1e-9,
            },
        )
        .unwrap();
        let weighted = report.design_weighted_metrics.unwrap();
        assert_eq!(report.false_negative_sample_count, 2);
        assert_eq!(weighted.false_negative_total, 500.0);
        assert_eq!(weighted.recall, Some(0.0));
        assert_eq!(report.adjudication_coverage, 1.0);
    }

    #[test]
    fn unknown_and_unobservable_are_not_converted_to_negative_or_population_estimates() {
        let strata = vec![stratum(
            "negative-space",
            AuditStratumKind::PredictedNegativeLandscape,
            10,
            3,
        )];
        let samples = vec![
            sample(
                "s1",
                "u1",
                "l1",
                "negative-space",
                ModelPrediction::Negative,
                positive(),
                0.3,
            ),
            sample(
                "s2",
                "u2",
                "l1",
                "negative-space",
                ModelPrediction::Negative,
                AuditGroundTruth::Unknown {
                    reason_ref: "urn:synthetic:not-adjudicated".into(),
                },
                0.3,
            ),
            sample(
                "s3",
                "u3",
                "l2",
                "negative-space",
                ModelPrediction::Negative,
                AuditGroundTruth::Unobservable {
                    reason_ref: "urn:synthetic:cloud-or-access-limit".into(),
                },
                0.3,
            ),
        ];
        let report = evaluate_negative_space_audit(
            "frame",
            "design",
            &strata,
            &samples,
            NegativeAuditConfig {
                probability_tolerance: 1e-9,
            },
        )
        .unwrap();
        assert_eq!(report.unknown_count, 1);
        assert_eq!(report.unobservable_count, 1);
        assert_eq!(report.unresolved_count, 2);
        assert!((report.adjudication_coverage - (1.0 / 3.0)).abs() < 1e-12);
        assert!(report.design_weighted_metrics.is_none());
        assert_eq!(report.adjudicated_sample_confusion.false_negative, 1.0);
    }

    #[test]
    fn model_abstentions_are_reported_and_block_population_metrics() {
        let strata = vec![stratum(
            "abstained",
            AuditStratumKind::ModelAbstained,
            10,
            1,
        )];
        let samples = vec![sample(
            "s1",
            "u1",
            "l1",
            "abstained",
            ModelPrediction::Abstained,
            positive(),
            0.1,
        )];
        let report = evaluate_negative_space_audit(
            "frame",
            "design",
            &strata,
            &samples,
            NegativeAuditConfig {
                probability_tolerance: 1e-9,
            },
        )
        .unwrap();
        assert_eq!(report.adjudicated_positive_count, 1);
        assert_eq!(report.model_abstained_count, 1);
        assert_eq!(report.unresolved_count, 0);
        assert_eq!(report.adjudication_coverage, 1.0);
        assert!(report.design_weighted_metrics.is_none());
    }

    #[test]
    fn non_independent_evidence_tier_cannot_be_used_as_confirmed_ground_truth() {
        let strata = vec![stratum("high", AuditStratumKind::HighScoreCandidates, 1, 1)];
        let invalid = AuditGroundTruth::ConfirmedPositive {
            adjudication_ref: "urn:synthetic:catalogue".into(),
            evidence_tier: AuditEvidenceTier::CatalogueReference,
            reviewer_ref: "urn:synthetic:reviewer".into(),
            authority_ref: "urn:synthetic:authority".into(),
            rights_receipt_ref: "urn:synthetic:rights".into(),
        };
        let samples = vec![sample(
            "s1",
            "u1",
            "l1",
            "high",
            ModelPrediction::Positive,
            invalid,
            1.0,
        )];
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &samples,
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::InvalidEvidenceTier { .. })
        ));
    }

    #[test]
    fn confirmed_negative_requires_detectability_assessment() {
        let strata = vec![stratum(
            "negative",
            AuditStratumKind::PredictedNegativeLandscape,
            1,
            1,
        )];
        let invalid = AuditGroundTruth::ConfirmedNegative {
            adjudication_ref: "urn:synthetic:negative".into(),
            evidence_tier: AuditEvidenceTier::FieldAssessed,
            reviewer_ref: "urn:synthetic:reviewer".into(),
            authority_ref: "urn:synthetic:authority".into(),
            rights_receipt_ref: "urn:synthetic:rights".into(),
            detectability_assessment_ref: " ".into(),
        };
        let samples = vec![sample(
            "s1",
            "u1",
            "l1",
            "negative",
            ModelPrediction::Negative,
            invalid,
            1.0,
        )];
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &samples,
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::MissingDetectabilityAssessment { .. })
        ));
    }

    #[test]
    fn srswOR_inclusion_probability_must_match_declared_stratum_design() {
        let strata = vec![stratum(
            "negative",
            AuditStratumKind::PredictedNegativeLandscape,
            100,
            10,
        )];
        let samples = vec![
            sample(
                "s1",
                "u1",
                "l1",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s2",
                "u2",
                "l1",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s3",
                "u3",
                "l1",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s4",
                "u4",
                "l2",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s5",
                "u5",
                "l2",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s6",
                "u6",
                "l2",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s7",
                "u7",
                "l3",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s8",
                "u8",
                "l3",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s9",
                "u9",
                "l3",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
            sample(
                "s10",
                "u10",
                "l4",
                "negative",
                ModelPrediction::Negative,
                negative(),
                0.2,
            ),
        ];
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &samples,
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::IncorrectSrsInclusionProbability { .. })
        ));
    }

    #[test]
    fn unequal_probability_design_uses_per_unit_inclusion_probabilities() {
        let mut design_stratum = stratum("high", AuditStratumKind::HighScoreCandidates, 100, 2);
        design_stratum.sampling_design = AuditSamplingDesign::KnownUnequalFirstOrderProbability;
        let samples = vec![
            sample(
                "s1",
                "u1",
                "l1",
                "high",
                ModelPrediction::Positive,
                positive(),
                0.01,
            ),
            sample(
                "s2",
                "u2",
                "l2",
                "high",
                ModelPrediction::Positive,
                positive(),
                0.04,
            ),
        ];
        let report = evaluate_negative_space_audit(
            "frame",
            "design",
            &[design_stratum],
            &samples,
            NegativeAuditConfig {
                probability_tolerance: 1e-9,
            },
        )
        .unwrap();
        let weighted = report.design_weighted_metrics.unwrap();
        assert_eq!(weighted.estimated_population_confusion.true_positive, 125.0);
        assert_eq!(weighted.recall, Some(1.0));
    }

    #[test]
    fn sample_counts_must_match_design_and_duplicate_units_fail_closed() {
        let strata = vec![stratum(
            "negative",
            AuditStratumKind::PredictedNegativeLandscape,
            10,
            2,
        )];
        let one = sample(
            "s1",
            "u1",
            "l1",
            "negative",
            ModelPrediction::Negative,
            negative(),
            0.2,
        );
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &[one.clone()],
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::SelectedCountMismatch { .. })
        ));
        let duplicate = sample(
            "s2",
            "u1",
            "l2",
            "negative",
            ModelPrediction::Negative,
            negative(),
            0.2,
        );
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &[one, duplicate],
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::DuplicateUnitId(_))
        ));
    }

    #[test]
    fn non_finite_expansion_weights_fail_closed() {
        let strata = vec![AuditPopulationStratum {
            stratum_id: "unequal".into(),
            kind: AuditStratumKind::PredictedNegativeLandscape,
            population_unit_count: 1,
            selected_unit_count: 1,
            sampling_design: AuditSamplingDesign::KnownUnequalFirstOrderProbability,
            design_reference: "urn:synthetic:design:unequal".into(),
        }];
        let samples = vec![sample(
            "s1",
            "u1",
            "l1",
            "unequal",
            ModelPrediction::Negative,
            negative(),
            1e-320,
        )];
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &samples,
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::InvalidExpansionWeight { .. })
        ));
    }

    #[test]
    fn prediction_and_stratum_semantics_must_match() {
        let strata = vec![stratum(
            "negative",
            AuditStratumKind::PredictedNegativeLandscape,
            1,
            1,
        )];
        let samples = vec![sample(
            "s1",
            "u1",
            "l1",
            "negative",
            ModelPrediction::Positive,
            positive(),
            1.0,
        )];
        assert!(matches!(
            evaluate_negative_space_audit(
                "frame",
                "design",
                &strata,
                &samples,
                NegativeAuditConfig {
                    probability_tolerance: 1e-9
                }
            ),
            Err(NegativeAuditError::PredictionStratumMismatch { .. })
        ));
    }

    #[test]
    fn f1_and_precision_are_none_when_their_denominators_are_zero() {
        let strata = vec![stratum("high", AuditStratumKind::HighScoreCandidates, 1, 1)];
        let samples = vec![sample(
            "s1",
            "u1",
            "l1",
            "high",
            ModelPrediction::Positive,
            negative(),
            1.0,
        )];
        let report = evaluate_negative_space_audit(
            "frame",
            "design",
            &strata,
            &samples,
            NegativeAuditConfig {
                probability_tolerance: 1e-9,
            },
        )
        .unwrap();
        let metrics = report.design_weighted_metrics.unwrap();
        assert_eq!(metrics.precision, Some(0.0));
        assert_eq!(metrics.recall, None);
        assert_eq!(metrics.f1, Some(0.0));
    }
}
