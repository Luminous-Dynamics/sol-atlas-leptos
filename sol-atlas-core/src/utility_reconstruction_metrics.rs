// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Asset-association metrics for blinded utility-reconstruction benchmarks.
//!
//! Scores only independently admissible, adjudicated event truth. Clutter is
//! a per-event singleton, not a shared asset. Explicit abstentions are also
//! per-event singletons and therefore reduce completeness without inventing
//! an asset. This module does not validate physical geometry or topology.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetTruthTier {
    StructuredIndependentSurvey,
    TrenchImageInterpreted,
    OwnerRecordOnly,
    Conflicted,
    Insufficient,
}

impl AssetTruthTier {
    fn is_score_eligible(self) -> bool {
        matches!(
            self,
            Self::StructuredIndependentSurvey | Self::TrenchImageInterpreted
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetReferenceEvent {
    pub event_id: String,
    /// None means adjudicated non-utility clutter, not unknown truth.
    /// Non-admitted truth tiers must use None and are never scored.
    pub truth_asset_id: Option<String>,
    pub truth_tier: AssetTruthTier,
    pub truth_evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum PredictedEventDisposition {
    CandidateAsset { candidate_asset_id: String },
    Abstain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredictedAssetAssignment {
    pub event_id: String,
    pub disposition: PredictedEventDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetAssociationEvaluation {
    pub evaluation_id: String,
    pub scope_id: String,
    pub reference_events: Vec<AssetReferenceEvent>,
    /// Exactly one prediction disposition is required for every reference
    /// event, including excluded truth tiers. Missing is not abstention.
    pub predictions: Vec<PredictedAssetAssignment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetAssociationError {
    EmptyIdentifier(&'static str),
    DuplicateReferenceEvent(String),
    DuplicatePrediction(String),
    MissingPrediction(String),
    UnknownPredictionEvent(String),
    InvalidCandidateAssetNamespace(String),
    EligibleEventSetEmpty,
    InvalidTruthAssetForTier(String),
    MissingTruthEvidence(String),
    CountOverflow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetAssociationMetrics {
    pub evaluated_events: u64,
    pub excluded_owner_record_only_events: u64,
    pub excluded_conflicted_events: u64,
    pub excluded_insufficient_truth_events: u64,
    pub truth_utility_asset_count: u64,
    pub predicted_candidate_asset_count: u64,
    pub false_candidate_asset_count: u64,
    pub missed_truth_asset_count: u64,
    pub assigned_event_count: u64,
    pub abstained_event_count: u64,
    pub abstention_rate: f64,
    pub false_association_clutter_events: u64,
    pub abstained_utility_events: u64,
    /// B-cubed averages over every eligible event, including abstentions and
    /// clutter singleton cases. This prevents silent denominator shrinkage.
    pub b_cubed_precision: f64,
    pub b_cubed_recall: f64,
    pub b_cubed_f1: f64,
    pub pairwise_true_positive_pairs: u64,
    pub overmerge_pairs: u64,
    pub oversplit_pairs: u64,
    /// None means there were no predicted (or true) positive pairs.
    pub pairwise_precision: Option<f64>,
    pub pairwise_recall: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ClusterKey {
    Utility(String),
    Singleton(String),
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn checked_choose_two(n: usize) -> Result<u64, AssetAssociationError> {
    let n = u64::try_from(n).map_err(|_| AssetAssociationError::CountOverflow)?;
    n.checked_mul(n.saturating_sub(1))
        .and_then(|value| value.checked_div(2))
        .ok_or(AssetAssociationError::CountOverflow)
}

fn count_u64(value: usize) -> Result<u64, AssetAssociationError> {
    u64::try_from(value).map_err(|_| AssetAssociationError::CountOverflow)
}

/// Validate event identity, truth-tier admissibility and exact prediction
/// coverage. Unknown or unreviewed truth is excluded, not treated as clutter.
pub fn validate_asset_association_evaluation(
    evaluation: &AssetAssociationEvaluation,
) -> Vec<AssetAssociationError> {
    let mut errors = Vec::new();

    if !non_empty(&evaluation.evaluation_id) {
        errors.push(AssetAssociationError::EmptyIdentifier("evaluation_id"));
    }
    if !non_empty(&evaluation.scope_id) {
        errors.push(AssetAssociationError::EmptyIdentifier("scope_id"));
    }

    let mut reference_ids = BTreeSet::new();
    for event in &evaluation.reference_events {
        if !non_empty(&event.event_id) {
            errors.push(AssetAssociationError::EmptyIdentifier("reference_event_id"));
        }
        if !reference_ids.insert(event.event_id.as_str()) {
            errors.push(AssetAssociationError::DuplicateReferenceEvent(
                event.event_id.clone(),
            ));
        }

        if event.truth_tier.is_score_eligible() {
            if !non_empty(&event.truth_evidence_id) {
                errors.push(AssetAssociationError::MissingTruthEvidence(
                    event.event_id.clone(),
                ));
            }
            if event
                .truth_asset_id
                .as_deref()
                .is_some_and(|asset_id| !non_empty(asset_id))
            {
                errors.push(AssetAssociationError::InvalidTruthAssetForTier(
                    event.event_id.clone(),
                ));
            }
        } else if event.truth_asset_id.is_some() {
            errors.push(AssetAssociationError::InvalidTruthAssetForTier(
                event.event_id.clone(),
            ));
        }
    }

    let mut prediction_ids = BTreeSet::new();
    for prediction in &evaluation.predictions {
        if !prediction_ids.insert(prediction.event_id.as_str()) {
            errors.push(AssetAssociationError::DuplicatePrediction(
                prediction.event_id.clone(),
            ));
        }
        if !reference_ids.contains(prediction.event_id.as_str()) {
            errors.push(AssetAssociationError::UnknownPredictionEvent(
                prediction.event_id.clone(),
            ));
        }
        if let PredictedEventDisposition::CandidateAsset { candidate_asset_id } =
            &prediction.disposition
        {
            if !candidate_asset_id.starts_with("candidate:")
                || candidate_asset_id.len() <= "candidate:".len()
            {
                errors.push(AssetAssociationError::InvalidCandidateAssetNamespace(
                    candidate_asset_id.clone(),
                ));
            }
        }
    }

    for event_id in &reference_ids {
        if !prediction_ids.contains(event_id) {
            errors.push(AssetAssociationError::MissingPrediction(
                (*event_id).to_string(),
            ));
        }
    }

    if !evaluation
        .reference_events
        .iter()
        .any(|event| event.truth_tier.is_score_eligible())
    {
        errors.push(AssetAssociationError::EligibleEventSetEmpty);
    }

    errors
}

/// Score event-to-asset clustering without treating uncertain labels as
/// negative examples. B-cubed and pairwise metrics are reported together:
/// B-cubed is event-balanced; pairwise counts expose explicit merge/split
/// errors. A clutter truth event and an abstained prediction each form their
/// own singleton cluster.
pub fn score_asset_association(
    evaluation: &AssetAssociationEvaluation,
) -> Result<AssetAssociationMetrics, Vec<AssetAssociationError>> {
    let errors = validate_asset_association_evaluation(evaluation);
    if !errors.is_empty() {
        return Err(errors);
    }

    let predictions: BTreeMap<&str, &PredictedEventDisposition> = evaluation
        .predictions
        .iter()
        .map(|prediction| (prediction.event_id.as_str(), &prediction.disposition))
        .collect();

    let mut eligible = Vec::new();
    let mut excluded_owner = 0usize;
    let mut excluded_conflicted = 0usize;
    let mut excluded_insufficient = 0usize;

    for event in &evaluation.reference_events {
        match event.truth_tier {
            AssetTruthTier::StructuredIndependentSurvey | AssetTruthTier::TrenchImageInterpreted => {
                eligible.push(event);
            }
            AssetTruthTier::OwnerRecordOnly => excluded_owner += 1,
            AssetTruthTier::Conflicted => excluded_conflicted += 1,
            AssetTruthTier::Insufficient => excluded_insufficient += 1,
        }
    }

    let mut predicted_clusters: BTreeMap<ClusterKey, Vec<&str>> = BTreeMap::new();
    let mut truth_clusters: BTreeMap<ClusterKey, Vec<&str>> = BTreeMap::new();
    let mut contingency: BTreeMap<(ClusterKey, ClusterKey), usize> = BTreeMap::new();
    let mut truth_asset_ids = BTreeSet::new();
    let mut candidate_asset_ids = BTreeSet::new();
    let mut predicted_asset_truth_members: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut true_asset_has_prediction: BTreeMap<String, bool> = BTreeMap::new();

    let mut assigned_events = 0usize;
    let mut abstained_events = 0usize;
    let mut clutter_false_associations = 0usize;
    let mut abstained_utility_events = 0usize;

    for event in &eligible {
        let truth_key = match event.truth_asset_id.as_deref() {
            Some(asset_id) => {
                truth_asset_ids.insert(asset_id.to_string());
                true_asset_has_prediction.entry(asset_id.to_string()).or_insert(false);
                ClusterKey::Utility(asset_id.to_string())
            }
            None => ClusterKey::Singleton(format!("truth-clutter:{}", event.event_id)),
        };

        let prediction = predictions[event.event_id.as_str()];
        let predicted_key = match prediction {
            PredictedEventDisposition::CandidateAsset { candidate_asset_id } => {
                assigned_events += 1;
                candidate_asset_ids.insert(candidate_asset_id.clone());
                if let Some(truth_asset_id) = event.truth_asset_id.as_deref() {
                    true_asset_has_prediction.insert(truth_asset_id.to_string(), true);
                    predicted_asset_truth_members
                        .entry(candidate_asset_id.clone())
                        .or_default()
                        .insert(truth_asset_id.to_string());
                } else {
                    clutter_false_associations += 1;
                }
                ClusterKey::Utility(candidate_asset_id.clone())
            }
            PredictedEventDisposition::Abstain => {
                abstained_events += 1;
                if event.truth_asset_id.is_some() {
                    abstained_utility_events += 1;
                }
                ClusterKey::Singleton(format!("abstain:{}", event.event_id))
            }
        };

        predicted_clusters
            .entry(predicted_key.clone())
            .or_default()
            .push(event.event_id.as_str());
        truth_clusters
            .entry(truth_key.clone())
            .or_default()
            .push(event.event_id.as_str());
        *contingency
            .entry((predicted_key, truth_key))
            .or_insert(0) += 1;
    }

    let mut b3_precision_sum = 0.0;
    let mut b3_recall_sum = 0.0;
    for event in &eligible {
        let prediction = predictions[event.event_id.as_str()];
        let predicted_key = match prediction {
            PredictedEventDisposition::CandidateAsset { candidate_asset_id } => {
                ClusterKey::Utility(candidate_asset_id.clone())
            }
            PredictedEventDisposition::Abstain => {
                ClusterKey::Singleton(format!("abstain:{}", event.event_id))
            }
        };
        let truth_key = match event.truth_asset_id.as_deref() {
            Some(asset_id) => ClusterKey::Utility(asset_id.to_string()),
            None => ClusterKey::Singleton(format!("truth-clutter:{}", event.event_id)),
        };
        let intersection = contingency
            .get(&(predicted_key.clone(), truth_key.clone()))
            .copied()
            .unwrap_or(0) as f64;
        let predicted_size = predicted_clusters[&predicted_key].len() as f64;
        let truth_size = truth_clusters[&truth_key].len() as f64;
        b3_precision_sum += intersection / predicted_size;
        b3_recall_sum += intersection / truth_size;
    }

    let total_predicted_pairs = predicted_clusters
        .values()
        .try_fold(0u64, |sum, members| {
            sum.checked_add(checked_choose_two(members.len())?)
                .ok_or(AssetAssociationError::CountOverflow)
        })
        .map_err(|error| vec![error])?;
    let total_truth_pairs = truth_clusters
        .values()
        .try_fold(0u64, |sum, members| {
            sum.checked_add(checked_choose_two(members.len())?)
                .ok_or(AssetAssociationError::CountOverflow)
        })
        .map_err(|error| vec![error])?;
    let true_positive_pairs = contingency
        .values()
        .try_fold(0u64, |sum, count| {
            sum.checked_add(checked_choose_two(*count)?)
                .ok_or(AssetAssociationError::CountOverflow)
        })
        .map_err(|error| vec![error])?;

    let overmerge_pairs = total_predicted_pairs
        .checked_sub(true_positive_pairs)
        .ok_or_else(|| vec![AssetAssociationError::CountOverflow])?;
    let oversplit_pairs = total_truth_pairs
        .checked_sub(true_positive_pairs)
        .ok_or_else(|| vec![AssetAssociationError::CountOverflow])?;

    let candidate_assets_with_truth: BTreeSet<String> = predicted_asset_truth_members
        .values()
        .flat_map(|assets| assets.iter().cloned())
        .collect();
    let false_candidate_assets = candidate_asset_ids
        .iter()
        .filter(|candidate_id| {
            predicted_asset_truth_members
                .get(*candidate_id)
                .is_none_or(BTreeSet::is_empty)
        })
        .count();
    let missed_truth_assets = truth_asset_ids
        .iter()
        .filter(|asset_id| !candidate_assets_with_truth.contains(*asset_id))
        .count();

    let event_count = eligible.len() as f64;
    let b3_precision = b3_precision_sum / event_count;
    let b3_recall = b3_recall_sum / event_count;
    let b3_f1 = if b3_precision + b3_recall > 0.0 {
        2.0 * b3_precision * b3_recall / (b3_precision + b3_recall)
    } else {
        0.0
    };

    let assigned_count = count_u64(assigned_events).map_err(|error| vec![error])?;
    let abstained_count = count_u64(abstained_events).map_err(|error| vec![error])?;
    let precision_denominator = true_positive_pairs + overmerge_pairs;
    let recall_denominator = true_positive_pairs + oversplit_pairs;

    Ok(AssetAssociationMetrics {
        evaluated_events: count_u64(eligible.len()).map_err(|error| vec![error])?,
        excluded_owner_record_only_events: count_u64(excluded_owner)
            .map_err(|error| vec![error])?,
        excluded_conflicted_events: count_u64(excluded_conflicted)
            .map_err(|error| vec![error])?,
        excluded_insufficient_truth_events: count_u64(excluded_insufficient)
            .map_err(|error| vec![error])?,
        truth_utility_asset_count: count_u64(truth_asset_ids.len()).map_err(|error| vec![error])?,
        predicted_candidate_asset_count: count_u64(candidate_asset_ids.len())
            .map_err(|error| vec![error])?,
        false_candidate_asset_count: count_u64(false_candidate_assets)
            .map_err(|error| vec![error])?,
        missed_truth_asset_count: count_u64(missed_truth_assets).map_err(|error| vec![error])?,
        assigned_event_count: assigned_count,
        abstained_event_count: abstained_count,
        abstention_rate: abstained_count as f64 / event_count,
        false_association_clutter_events: count_u64(clutter_false_associations)
            .map_err(|error| vec![error])?,
        abstained_utility_events: count_u64(abstained_utility_events)
            .map_err(|error| vec![error])?,
        b_cubed_precision: b3_precision,
        b_cubed_recall: b3_recall,
        b_cubed_f1: b3_f1,
        pairwise_true_positive_pairs: true_positive_pairs,
        overmerge_pairs,
        oversplit_pairs,
        pairwise_precision: (precision_denominator > 0)
            .then_some(true_positive_pairs as f64 / precision_denominator as f64),
        pairwise_recall: (recall_denominator > 0)
            .then_some(true_positive_pairs as f64 / recall_denominator as f64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/asset-association-evaluation-synthetic-v0.1.json"
    );

    #[test]
    fn committed_asset_association_fixture_parses_and_scores() {
        let evaluation: AssetAssociationEvaluation =
            serde_json::from_str(FIXTURE).expect("asset association fixture parses");
        let metrics = score_asset_association(&evaluation).unwrap();
        assert_eq!(metrics.evaluated_events, 4);
        assert_eq!(metrics.excluded_owner_record_only_events, 1);
        assert_eq!(metrics.truth_utility_asset_count, 2);
        assert_eq!(metrics.predicted_candidate_asset_count, 2);
        assert_eq!(metrics.overmerge_pairs, 0);
        assert_eq!(metrics.oversplit_pairs, 0);
        assert_eq!(metrics.b_cubed_f1, 1.0);
    }

    fn evaluation() -> AssetAssociationEvaluation {
        AssetAssociationEvaluation {
            evaluation_id: "synthetic-asset-association-001".to_string(),
            scope_id: "synthetic-corridor-a".to_string(),
            reference_events: vec![
                AssetReferenceEvent {
                    event_id: "event-1".to_string(),
                    truth_asset_id: Some("truth:utility-a".to_string()),
                    truth_tier: AssetTruthTier::StructuredIndependentSurvey,
                    truth_evidence_id: "truth-evidence-1".to_string(),
                },
                AssetReferenceEvent {
                    event_id: "event-2".to_string(),
                    truth_asset_id: Some("truth:utility-a".to_string()),
                    truth_tier: AssetTruthTier::TrenchImageInterpreted,
                    truth_evidence_id: "truth-evidence-2".to_string(),
                },
                AssetReferenceEvent {
                    event_id: "event-clutter".to_string(),
                    truth_asset_id: None,
                    truth_tier: AssetTruthTier::StructuredIndependentSurvey,
                    truth_evidence_id: "truth-evidence-clutter".to_string(),
                },
            ],
            predictions: vec![
                PredictedAssetAssignment {
                    event_id: "event-1".to_string(),
                    disposition: PredictedEventDisposition::CandidateAsset {
                        candidate_asset_id: "candidate:utility-a".to_string(),
                    },
                },
                PredictedAssetAssignment {
                    event_id: "event-2".to_string(),
                    disposition: PredictedEventDisposition::CandidateAsset {
                        candidate_asset_id: "candidate:utility-a".to_string(),
                    },
                },
                PredictedAssetAssignment {
                    event_id: "event-clutter".to_string(),
                    disposition: PredictedEventDisposition::Abstain,
                },
            ],
        }
    }

    #[test]
    fn correct_asset_grouping_scores_perfectly() {
        let metrics = score_asset_association(&evaluation()).unwrap();
        assert_eq!(metrics.evaluated_events, 3);
        assert_eq!(metrics.truth_utility_asset_count, 1);
        assert_eq!(metrics.predicted_candidate_asset_count, 1);
        assert_eq!(metrics.false_candidate_asset_count, 0);
        assert_eq!(metrics.missed_truth_asset_count, 0);
        assert_eq!(metrics.overmerge_pairs, 0);
        assert_eq!(metrics.oversplit_pairs, 0);
        assert_eq!(metrics.b_cubed_f1, 1.0);
        assert_eq!(metrics.pairwise_precision, Some(1.0));
        assert_eq!(metrics.pairwise_recall, Some(1.0));
    }

    #[test]
    fn merging_clutter_into_a_real_asset_is_counted_as_overmerge() {
        let mut evaluation = evaluation();
        evaluation.predictions[2].disposition =
            PredictedEventDisposition::CandidateAsset {
                candidate_asset_id: "candidate:utility-a".to_string(),
            };
        let metrics = score_asset_association(&evaluation).unwrap();
        assert_eq!(metrics.overmerge_pairs, 2);
        assert_eq!(metrics.false_association_clutter_events, 1);
        assert_eq!(metrics.pairwise_precision, Some(1.0 / 3.0));
    }

    #[test]
    fn splitting_one_truth_asset_is_counted_as_oversplit() {
        let mut evaluation = evaluation();
        evaluation.predictions[1].disposition =
            PredictedEventDisposition::CandidateAsset {
                candidate_asset_id: "candidate:utility-b".to_string(),
            };
        let metrics = score_asset_association(&evaluation).unwrap();
        assert_eq!(metrics.overmerge_pairs, 0);
        assert_eq!(metrics.oversplit_pairs, 1);
        assert_eq!(metrics.pairwise_recall, Some(0.0));
        assert_eq!(metrics.missed_truth_asset_count, 0);
    }

    #[test]
    fn abstention_reduces_completeness_without_claiming_false_absence() {
        let mut evaluation = evaluation();
        evaluation.predictions[1].disposition = PredictedEventDisposition::Abstain;
        let metrics = score_asset_association(&evaluation).unwrap();
        assert_eq!(metrics.abstained_utility_events, 1);
        assert_eq!(metrics.oversplit_pairs, 1);
        assert_eq!(metrics.false_association_clutter_events, 0);
    }

    #[test]
    fn record_only_truth_is_excluded_not_promoted_to_clutter() {
        let mut evaluation = evaluation();
        evaluation.reference_events.push(AssetReferenceEvent {
            event_id: "event-record-only".to_string(),
            truth_asset_id: None,
            truth_tier: AssetTruthTier::OwnerRecordOnly,
            truth_evidence_id: "owner-record-only".to_string(),
        });
        evaluation.predictions.push(PredictedAssetAssignment {
            event_id: "event-record-only".to_string(),
            disposition: PredictedEventDisposition::CandidateAsset {
                candidate_asset_id: "candidate:unverified".to_string(),
            },
        });
        let metrics = score_asset_association(&evaluation).unwrap();
        assert_eq!(metrics.evaluated_events, 3);
        assert_eq!(metrics.excluded_owner_record_only_events, 1);
        assert_eq!(metrics.predicted_candidate_asset_count, 1);
    }

    #[test]
    fn missing_predictions_are_not_silently_treated_as_abstentions() {
        let mut evaluation = evaluation();
        evaluation.predictions.pop();
        assert!(validate_asset_association_evaluation(&evaluation)
            .contains(&AssetAssociationError::MissingPrediction(
                "event-clutter".to_string()
            )));
    }

    #[test]
    fn unreviewed_truth_cannot_be_promoted_to_a_negative_example() {
        let mut evaluation = evaluation();
        evaluation.reference_events[0].truth_tier = AssetTruthTier::Conflicted;
        assert!(validate_asset_association_evaluation(&evaluation)
            .contains(&AssetAssociationError::InvalidTruthAssetForTier(
                "event-1".to_string()
            )));
    }
}
