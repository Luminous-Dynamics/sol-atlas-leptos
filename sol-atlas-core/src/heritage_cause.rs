// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cause-aware evaluation for heritage change candidates.
//!
//! Change detection, cause assessment and heritage significance are distinct
//! tasks. This evaluator handles only cause assessment. It does not infer the
//! cause of a real observation or assign cultural/heritage significance.
//! Unknown labels are excluded, not treated as no-change negatives. Abstained
//! predictions are excluded from conditional classification metrics and counted
//! in coverage. Every asserted cause must cite at least one evidence reference.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const CAUSE_EVALUATION_SCHEMA_VERSION: &str =
    "luminous.heritage-cause-evaluation.v0.1";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ChangeCause {
    ArchaeologicalExcavationLooting,
    AgricultureCultivation,
    SeasonalVegetationPhenology,
    ErosionGeomorphic,
    HydrologyFlood,
    FireVegetationClearance,
    ModernConstructionInfrastructure,
    QuarryMining,
    MilitaryConflict,
    SensorProcessingArtifact,
    OtherKnownCause,
}

pub const CHANGE_CAUSES: [ChangeCause; 11] = [
    ChangeCause::ArchaeologicalExcavationLooting,
    ChangeCause::AgricultureCultivation,
    ChangeCause::SeasonalVegetationPhenology,
    ChangeCause::ErosionGeomorphic,
    ChangeCause::HydrologyFlood,
    ChangeCause::FireVegetationClearance,
    ChangeCause::ModernConstructionInfrastructure,
    ChangeCause::QuarryMining,
    ChangeCause::MilitaryConflict,
    ChangeCause::SensorProcessingArtifact,
    ChangeCause::OtherKnownCause,
];

impl ChangeCause {
    pub fn is_archaeological_claim(self) -> bool {
        matches!(self, Self::ArchaeologicalExcavationLooting)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CauseGroundTruth {
    /// Expert/custodian adjudication is complete for this candidate and the
    /// cause list is exhaustive within the declared taxonomy.
    CompleteCauses {
        causes: Vec<ChangeCause>,
        adjudication_ref: String,
    },
    /// A reviewer confirmed that no relevant change occurred in the reviewed
    /// observation window. This is a verified negative, not an unknown.
    ConfirmedNoRelevantChange {
        adjudication_ref: String,
    },
    /// The cause is not known or cannot be adjudicated. This contributes to
    /// neither positive nor negative cause metrics.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseProbability {
    pub cause: ChangeCause,
    pub probability: f64,
    /// Provenance references that support this particular predicted cause.
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausePrediction {
    pub abstained: bool,
    pub scores: Vec<CauseProbability>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseEvaluationSample {
    pub sample_id: String,
    pub ground_truth: CauseGroundTruth,
    pub prediction: CausePrediction,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CauseEvaluationConfig {
    /// One globally fixed threshold for v0.1. Class-specific thresholds may be
    /// introduced later but must be selected on validation data only.
    pub cause_threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseClassMetrics {
    pub cause: ChangeCause,
    pub support_count: usize,
    pub predicted_positive_count: usize,
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1: Option<f64>,
    pub brier_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseEvaluationReport {
    pub schema_version: String,
    pub sample_count: usize,
    pub complete_cause_label_count: usize,
    pub confirmed_no_change_count: usize,
    pub unknown_label_count: usize,
    pub eligible_labelled_count: usize,
    pub covered_count: usize,
    pub abstained_count: usize,
    pub coverage: Option<f64>,
    /// Macro F1 over supported causes among non-abstained labelled samples.
    pub supported_macro_f1: Option<f64>,
    /// Cause claims are evaluated separately from anomaly/change detection.
    pub predicted_archaeological_claim_count: usize,
    pub true_archaeological_claim_count: usize,
    pub false_archaeological_claim_count: usize,
    pub archaeological_claim_precision: Option<f64>,
    pub false_archaeological_claims_per_100_covered_samples: Option<f64>,
    pub cause_threshold: f64,
    pub classes: Vec<CauseClassMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CauseEvaluationError {
    InvalidThreshold,
    EmptySampleId,
    DuplicateSampleId(String),
    EmptyAdjudicationReference { sample_id: String },
    CompleteCauseListEmpty { sample_id: String },
    DuplicateGroundTruthCause { sample_id: String, cause: ChangeCause },
    DuplicatePredictionCause { sample_id: String, cause: ChangeCause },
    MissingPredictionCause { sample_id: String, cause: ChangeCause },
    ExtraPredictionCause { sample_id: String, cause: ChangeCause },
    InvalidProbability { sample_id: String, cause: ChangeCause },
    AssertedCauseMissingEvidence { sample_id: String, cause: ChangeCause },
}

impl fmt::Display for CauseEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage cause evaluation error: {self:?}")
    }
}

impl std::error::Error for CauseEvaluationError {}

#[derive(Debug, Clone, Copy, Default)]
struct Confusion {
    support: usize,
    predicted: usize,
    tp: usize,
    fp: usize,
    fn_count: usize,
    brier_sum: f64,
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

fn validate_sample(
    sample: &CauseEvaluationSample,
    config: CauseEvaluationConfig,
) -> Result<(), CauseEvaluationError> {
    if sample.sample_id.trim().is_empty() {
        return Err(CauseEvaluationError::EmptySampleId);
    }

    let mut truth = BTreeSet::new();
    match &sample.ground_truth {
        CauseGroundTruth::CompleteCauses {
            causes,
            adjudication_ref,
        } => {
            if adjudication_ref.trim().is_empty() {
                return Err(CauseEvaluationError::EmptyAdjudicationReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
            if causes.is_empty() {
                return Err(CauseEvaluationError::CompleteCauseListEmpty {
                    sample_id: sample.sample_id.clone(),
                });
            }
            for cause in causes {
                if !truth.insert(*cause) {
                    return Err(CauseEvaluationError::DuplicateGroundTruthCause {
                        sample_id: sample.sample_id.clone(),
                        cause: *cause,
                    });
                }
            }
        }
        CauseGroundTruth::ConfirmedNoRelevantChange { adjudication_ref } => {
            if adjudication_ref.trim().is_empty() {
                return Err(CauseEvaluationError::EmptyAdjudicationReference {
                    sample_id: sample.sample_id.clone(),
                });
            }
        }
        CauseGroundTruth::Unknown => {}
    }

    let mut predicted = BTreeSet::new();
    for score in &sample.prediction.scores {
        if !predicted.insert(score.cause) {
            return Err(CauseEvaluationError::DuplicatePredictionCause {
                sample_id: sample.sample_id.clone(),
                cause: score.cause,
            });
        }
        if !score.probability.is_finite()
            || !(0.0..=1.0).contains(&score.probability)
        {
            return Err(CauseEvaluationError::InvalidProbability {
                sample_id: sample.sample_id.clone(),
                cause: score.cause,
            });
        }
        if !sample.prediction.abstained
            && score.probability >= config.cause_threshold
            && score.evidence_refs.iter().all(|reference| reference.trim().is_empty())
        {
            return Err(CauseEvaluationError::AssertedCauseMissingEvidence {
                sample_id: sample.sample_id.clone(),
                cause: score.cause,
            });
        }
    }

    for cause in CHANGE_CAUSES {
        if !predicted.contains(&cause) {
            return Err(CauseEvaluationError::MissingPredictionCause {
                sample_id: sample.sample_id.clone(),
                cause,
            });
        }
    }
    for cause in predicted {
        if !CHANGE_CAUSES.contains(&cause) {
            return Err(CauseEvaluationError::ExtraPredictionCause {
                sample_id: sample.sample_id.clone(),
                cause,
            });
        }
    }

    Ok(())
}

/// Evaluate multi-label cause predictions under explicit complete/negative/unknown
/// ground-truth semantics. Metrics are conditional on non-abstaining labelled
/// samples. Unknown samples are never counted as negative controls.
pub fn evaluate_change_causes(
    samples: &[CauseEvaluationSample],
    config: CauseEvaluationConfig,
) -> Result<CauseEvaluationReport, CauseEvaluationError> {
    if !config.cause_threshold.is_finite()
        || !(0.0..=1.0).contains(&config.cause_threshold)
    {
        return Err(CauseEvaluationError::InvalidThreshold);
    }

    let mut sample_ids = BTreeSet::new();
    let mut confusion = CHANGE_CAUSES.map(|_| Confusion::default());
    let mut complete_cause_label_count = 0;
    let mut confirmed_no_change_count = 0;
    let mut unknown_label_count = 0;
    let mut eligible_labelled_count = 0;
    let mut covered_count = 0;
    let mut predicted_archaeological_claim_count = 0;
    let mut true_archaeological_claim_count = 0;
    let mut false_archaeological_claim_count = 0;

    for sample in samples {
        if !sample_ids.insert(sample.sample_id.as_str()) {
            return Err(CauseEvaluationError::DuplicateSampleId(
                sample.sample_id.clone(),
            ));
        }
        validate_sample(sample, config)?;

        let truth: BTreeSet<ChangeCause> = match &sample.ground_truth {
            CauseGroundTruth::CompleteCauses { causes, .. } => {
                complete_cause_label_count += 1;
                causes.iter().copied().collect()
            }
            CauseGroundTruth::ConfirmedNoRelevantChange { .. } => {
                confirmed_no_change_count += 1;
                BTreeSet::new()
            }
            CauseGroundTruth::Unknown => {
                unknown_label_count += 1;
                continue;
            }
        };
        eligible_labelled_count += 1;

        if sample.prediction.abstained {
            continue;
        }
        covered_count += 1;

        let mut predicted_archaeological = false;
        for (index, cause) in CHANGE_CAUSES.iter().enumerate() {
            let score = sample.prediction.scores
                .iter()
                .find(|score| score.cause == *cause)
                .expect("validated predictions contain every cause");
            let predicted_positive = score.probability >= config.cause_threshold;
            let true_positive = truth.contains(cause);

            if true_positive {
                confusion[index].support += 1;
            }
            if predicted_positive {
                confusion[index].predicted += 1;
            }
            match (true_positive, predicted_positive) {
                (true, true) => confusion[index].tp += 1,
                (false, true) => confusion[index].fp += 1,
                (true, false) => confusion[index].fn_count += 1,
                (false, false) => {}
            }
            let target = if true_positive { 1.0 } else { 0.0 };
            confusion[index].brier_sum +=
                (score.probability - target) * (score.probability - target);

            if *cause == ChangeCause::ArchaeologicalExcavationLooting
                && predicted_positive
            {
                predicted_archaeological = true;
            }
        }

        if predicted_archaeological {
            predicted_archaeological_claim_count += 1;
            if truth.contains(&ChangeCause::ArchaeologicalExcavationLooting) {
                true_archaeological_claim_count += 1;
            } else {
                false_archaeological_claim_count += 1;
            }
        }
    }

    let classes: Vec<CauseClassMetrics> = CHANGE_CAUSES.iter().enumerate().map(|(index, cause)| {
        let values = confusion[index];
        let precision = ratio(values.tp, values.predicted);
        let recall = ratio(values.tp, values.support);
        let f1_denominator = 2 * values.tp + values.fp + values.fn_count;
        let f1 = (f1_denominator > 0)
            .then_some((2 * values.tp) as f64 / f1_denominator as f64);
        CauseClassMetrics {
            cause: *cause,
            support_count: values.support,
            predicted_positive_count: values.predicted,
            true_positive: values.tp,
            false_positive: values.fp,
            false_negative: values.fn_count,
            precision,
            recall,
            f1,
            brier_score: (covered_count > 0)
                .then_some(values.brier_sum / covered_count as f64),
        }
    }).collect();

    let supported_f1: Vec<f64> = classes.iter()
        .filter(|metrics| metrics.support_count > 0 || metrics.predicted_positive_count > 0)
        .filter_map(|metrics| metrics.f1)
        .collect();
    let supported_macro_f1 = (!supported_f1.is_empty())
        .then_some(supported_f1.iter().sum::<f64>() / supported_f1.len() as f64);

    Ok(CauseEvaluationReport {
        schema_version: CAUSE_EVALUATION_SCHEMA_VERSION.to_owned(),
        sample_count: samples.len(),
        complete_cause_label_count,
        confirmed_no_change_count,
        unknown_label_count,
        eligible_labelled_count,
        covered_count,
        abstained_count: eligible_labelled_count.saturating_sub(covered_count),
        coverage: ratio(covered_count, eligible_labelled_count),
        supported_macro_f1,
        predicted_archaeological_claim_count,
        true_archaeological_claim_count,
        false_archaeological_claim_count,
        archaeological_claim_precision: ratio(
            true_archaeological_claim_count,
            predicted_archaeological_claim_count,
        ),
        false_archaeological_claims_per_100_covered_samples: (covered_count > 0)
            .then_some(false_archaeological_claim_count as f64 * 100.0 / covered_count as f64),
        cause_threshold: config.cause_threshold,
        classes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> CauseEvaluationConfig {
        CauseEvaluationConfig { cause_threshold: 0.7 }
    }

    fn scores(predicted: &[(ChangeCause, f64)], abstained: bool) -> CausePrediction {
        let scores = CHANGE_CAUSES.iter().map(|cause| {
            let probability = predicted.iter()
                .find(|(predicted_cause, _)| predicted_cause == cause)
                .map(|(_, probability)| *probability)
                .unwrap_or(0.05);
            CauseProbability {
                cause: *cause,
                probability,
                evidence_refs: if probability >= 0.7 {
                    vec![format!("urn:synthetic:evidence:{cause:?}")]
                } else {
                    Vec::new()
                },
            }
        }).collect();
        CausePrediction { abstained, scores }
    }

    fn sample(
        id: &str,
        ground_truth: CauseGroundTruth,
        prediction: CausePrediction,
    ) -> CauseEvaluationSample {
        CauseEvaluationSample {
            sample_id: id.to_owned(),
            ground_truth,
            prediction,
        }
    }

    #[test]
    fn correct_archaeological_cause_is_counted_as_true_claim() {
        let sample = sample(
            "sample-a",
            CauseGroundTruth::CompleteCauses {
                causes: vec![ChangeCause::ArchaeologicalExcavationLooting],
                adjudication_ref: "urn:review:a".into(),
            },
            scores(&[(ChangeCause::ArchaeologicalExcavationLooting, 0.95)], false),
        );
        let report = evaluate_change_causes(&[sample], config()).unwrap();
        assert_eq!(report.predicted_archaeological_claim_count, 1);
        assert_eq!(report.true_archaeological_claim_count, 1);
        assert_eq!(report.false_archaeological_claim_count, 0);
        assert_eq!(report.archaeological_claim_precision, Some(1.0));
    }

    #[test]
    fn agriculture_false_positive_is_an_archaeological_false_claim() {
        let sample = sample(
            "sample-b",
            CauseGroundTruth::CompleteCauses {
                causes: vec![ChangeCause::AgricultureCultivation],
                adjudication_ref: "urn:review:b".into(),
            },
            scores(&[
                (ChangeCause::AgricultureCultivation, 0.9),
                (ChangeCause::ArchaeologicalExcavationLooting, 0.85),
            ], false),
        );
        let report = evaluate_change_causes(&[sample], config()).unwrap();
        assert_eq!(report.false_archaeological_claim_count, 1);
        assert_eq!(report.false_archaeological_claims_per_100_covered_samples, Some(100.0));
        assert_eq!(report.archaeological_claim_precision, Some(0.0));
    }

    #[test]
    fn confirmed_no_change_is_negative_but_unknown_is_not() {
        let no_change = sample(
            "no-change",
            CauseGroundTruth::ConfirmedNoRelevantChange {
                adjudication_ref: "urn:review:no-change".into(),
            },
            scores(&[(ChangeCause::ArchaeologicalExcavationLooting, 0.8)], false),
        );
        let unknown = sample(
            "unknown",
            CauseGroundTruth::Unknown,
            scores(&[(ChangeCause::ArchaeologicalExcavationLooting, 0.95)], false),
        );
        let report = evaluate_change_causes(&[no_change, unknown], config()).unwrap();
        assert_eq!(report.confirmed_no_change_count, 1);
        assert_eq!(report.unknown_label_count, 1);
        assert_eq!(report.eligible_labelled_count, 1);
        assert_eq!(report.covered_count, 1);
        assert_eq!(report.false_archaeological_claim_count, 1);
    }

    #[test]
    fn abstention_reduces_coverage_without_becoming_a_negative_prediction() {
        let sample = sample(
            "abstain",
            CauseGroundTruth::CompleteCauses {
                causes: vec![ChangeCause::ArchaeologicalExcavationLooting],
                adjudication_ref: "urn:review:abstain".into(),
            },
            scores(&[(ChangeCause::ArchaeologicalExcavationLooting, 0.99)], true),
        );
        let report = evaluate_change_causes(&[sample], config()).unwrap();
        assert_eq!(report.coverage, Some(0.0));
        assert_eq!(report.abstained_count, 1);
        assert_eq!(report.predicted_archaeological_claim_count, 0);
        assert_eq!(report.supported_macro_f1, None);
    }

    #[test]
    fn multi_cause_truth_supports_multiple_correct_labels() {
        let sample = sample(
            "mixed",
            CauseGroundTruth::CompleteCauses {
                causes: vec![
                    ChangeCause::AgricultureCultivation,
                    ChangeCause::SeasonalVegetationPhenology,
                ],
                adjudication_ref: "urn:review:mixed".into(),
            },
            scores(&[
                (ChangeCause::AgricultureCultivation, 0.9),
                (ChangeCause::SeasonalVegetationPhenology, 0.8),
            ], false),
        );
        let report = evaluate_change_causes(&[sample], config()).unwrap();
        assert_eq!(report.supported_macro_f1, Some(1.0));
    }

    #[test]
    fn every_asserted_cause_requires_an_evidence_reference() {
        let mut prediction = scores(&[(ChangeCause::ArchaeologicalExcavationLooting, 0.9)], false);
        prediction.scores.iter_mut()
            .find(|score| score.cause == ChangeCause::ArchaeologicalExcavationLooting)
            .unwrap().evidence_refs.clear();
        let sample = sample(
            "no-evidence",
            CauseGroundTruth::CompleteCauses {
                causes: vec![ChangeCause::ArchaeologicalExcavationLooting],
                adjudication_ref: "urn:review:evidence".into(),
            },
            prediction,
        );
        assert!(matches!(
            evaluate_change_causes(&[sample], config()),
            Err(CauseEvaluationError::AssertedCauseMissingEvidence { .. })
        ));
    }

    #[test]
    fn missing_duplicate_and_non_finite_scores_fail_closed() {
        let ground_truth = CauseGroundTruth::ConfirmedNoRelevantChange {
            adjudication_ref: "urn:review:negative".into(),
        };
        let mut missing = scores(&[], false);
        missing.scores.pop();
        assert!(matches!(
            evaluate_change_causes(&[sample("missing", ground_truth.clone(), missing)], config()),
            Err(CauseEvaluationError::MissingPredictionCause { .. })
        ));

        let mut duplicate = scores(&[], false);
        duplicate.scores[1].cause = duplicate.scores[0].cause;
        assert!(matches!(
            evaluate_change_causes(&[sample("duplicate", ground_truth.clone(), duplicate)], config()),
            Err(CauseEvaluationError::DuplicatePredictionCause { .. })
        ));

        let mut non_finite = scores(&[], false);
        non_finite.scores[0].probability = f64::NAN;
        assert!(matches!(
            evaluate_change_causes(&[sample("nan", ground_truth, non_finite)], config()),
            Err(CauseEvaluationError::InvalidProbability { .. })
        ));
    }

    #[test]
    fn zero_predicted_positive_scores_f1_as_zero_when_truth_is_positive() {
        let sample = sample(
            "missed-looting",
            CauseGroundTruth::CompleteCauses {
                causes: vec![ChangeCause::ArchaeologicalExcavationLooting],
                adjudication_ref: "urn:review:missed".into(),
            },
            scores(&[], false),
        );
        let report = evaluate_change_causes(&[sample], config()).unwrap();
        let metrics = report.classes.iter()
            .find(|metrics| metrics.cause == ChangeCause::ArchaeologicalExcavationLooting)
            .unwrap();
        assert_eq!(metrics.precision, None);
        assert_eq!(metrics.recall, Some(0.0));
        assert_eq!(metrics.f1, Some(0.0));
        assert_eq!(report.supported_macro_f1, Some(0.0));
    }

    #[test]
    fn duplicate_sample_ids_and_invalid_thresholds_fail_closed() {
        let no_change = CauseGroundTruth::ConfirmedNoRelevantChange {
            adjudication_ref: "urn:review:unique".into(),
        };
        let one = sample("same", no_change.clone(), scores(&[], false));
        let two = sample("same", no_change, scores(&[], false));
        assert!(matches!(
            evaluate_change_causes(&[one, two], config()),
            Err(CauseEvaluationError::DuplicateSampleId(_))
        ));

        assert_eq!(
            evaluate_change_causes(&[], CauseEvaluationConfig { cause_threshold: 1.5 }),
            Err(CauseEvaluationError::InvalidThreshold)
        );
    }
}
