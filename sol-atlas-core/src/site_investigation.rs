// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Expected-value-of-information analysis for site investigations.
//!
//! This evaluates a discrete decision model supplied by the caller. Priors,
//! losses, and test likelihoods must be evidence-backed by the project team.
//! It does not invent probabilities or certify engineering decisions.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const PROBABILITY_TOLERANCE: f64 = 1e-9;
const NUMERIC_TOLERANCE: f64 = 1e-7;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionModel {
    /// Prior probability of each mutually exclusive state of the world.
    pub state_probabilities: BTreeMap<String, f64>,
    /// decision ID -> state ID -> loss in a single common utility unit.
    pub losses: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Investigation {
    pub investigation_id: String,
    /// Investigation cost in the same utility unit used by the loss table.
    pub cost: f64,
    /// Outcome ID -> state ID -> P(outcome | state).
    pub outcome_likelihoods: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestigationValue {
    pub investigation_id: String,
    pub expected_loss_before_information: f64,
    pub expected_loss_after_sample_information: f64,
    pub expected_loss_with_perfect_information: f64,
    /// Expected Value of Sample Information (EVSI).
    pub evsi: f64,
    /// Expected Value of Perfect Information (EVPI), an upper bound on EVSI.
    pub evpi: f64,
    pub investigation_cost: f64,
    pub net_value: f64,
    pub recommended_for_economic_review: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueOfInformationError {
    EmptyStates,
    EmptyDecisions,
    EmptyInvestigationId,
    EmptyStateId,
    EmptyDecisionId,
    EmptyOutcomeId,
    InvalidProbability {
        label: String,
        value: f64,
    },
    InvalidLoss {
        decision_id: String,
        state_id: String,
        value: f64,
    },
    MissingDecisionLoss {
        decision_id: String,
        state_id: String,
    },
    UnknownDecisionState {
        decision_id: String,
        state_id: String,
    },
    MissingOutcomeLikelihood {
        investigation_id: String,
        outcome_id: String,
        state_id: String,
    },
    UnknownOutcomeState {
        investigation_id: String,
        outcome_id: String,
        state_id: String,
    },
    LikelihoodsDoNotSumToOne {
        investigation_id: String,
        state_id: String,
        sum: f64,
    },
    InvalidCost {
        investigation_id: String,
        cost: f64,
    },
    DuplicateInvestigationId(String),
}

impl fmt::Display for ValueOfInformationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStates => write!(f, "decision model must define at least one state"),
            Self::EmptyDecisions => write!(f, "decision model must define at least one decision"),
            Self::EmptyInvestigationId => write!(f, "investigation ID must not be empty"),
            Self::EmptyStateId => write!(f, "state IDs must not be empty"),
            Self::EmptyDecisionId => write!(f, "decision IDs must not be empty"),
            Self::EmptyOutcomeId => write!(f, "outcome IDs must not be empty"),
            Self::InvalidProbability { label, value } => {
                write!(f, "probability {label} must be finite and in [0, 1], got {value}")
            }
            Self::InvalidLoss {
                decision_id,
                state_id,
                value,
            } => write!(
                f,
                "loss for decision {decision_id} in state {state_id} must be finite and non-negative, got {value}"
            ),
            Self::MissingDecisionLoss {
                decision_id,
                state_id,
            } => write!(
                f,
                "decision {decision_id} has no loss for state {state_id}"
            ),
            Self::UnknownDecisionState {
                decision_id,
                state_id,
            } => write!(
                f,
                "decision {decision_id} refers to undeclared state {state_id}"
            ),
            Self::MissingOutcomeLikelihood {
                investigation_id,
                outcome_id,
                state_id,
            } => write!(
                f,
                "investigation {investigation_id} outcome {outcome_id} has no likelihood for state {state_id}"
            ),
            Self::UnknownOutcomeState {
                investigation_id,
                outcome_id,
                state_id,
            } => write!(
                f,
                "investigation {investigation_id} outcome {outcome_id} refers to undeclared state {state_id}"
            ),
            Self::LikelihoodsDoNotSumToOne {
                investigation_id,
                state_id,
                sum,
            } => write!(
                f,
                "outcome likelihoods for investigation {investigation_id}, state {state_id} sum to {sum}, not 1"
            ),
            Self::InvalidCost {
                investigation_id,
                cost,
            } => write!(
                f,
                "investigation {investigation_id} cost must be finite and non-negative, got {cost}"
            ),
            Self::DuplicateInvestigationId(id) => {
                write!(f, "duplicate investigation ID: {id}")
            }
        }
    }
}

impl std::error::Error for ValueOfInformationError {}

fn validate_model(model: &DecisionModel) -> Result<(), ValueOfInformationError> {
    if model.state_probabilities.is_empty() {
        return Err(ValueOfInformationError::EmptyStates);
    }
    if model.losses.is_empty() {
        return Err(ValueOfInformationError::EmptyDecisions);
    }

    let mut prior_sum = 0.0;
    for (state_id, probability) in &model.state_probabilities {
        if state_id.trim().is_empty() {
            return Err(ValueOfInformationError::EmptyStateId);
        }
        if !probability.is_finite() || !(0.0..=1.0).contains(probability) {
            return Err(ValueOfInformationError::InvalidProbability {
                label: format!("prior for state {state_id}"),
                value: *probability,
            });
        }
        prior_sum += probability;
    }
    if (prior_sum - 1.0).abs() > PROBABILITY_TOLERANCE {
        return Err(ValueOfInformationError::InvalidProbability {
            label: format!("sum of state priors (sum = {prior_sum})"),
            value: prior_sum,
        });
    }

    for (decision_id, losses) in &model.losses {
        if decision_id.trim().is_empty() {
            return Err(ValueOfInformationError::EmptyDecisionId);
        }
        for (state_id, loss) in losses {
            if !model.state_probabilities.contains_key(state_id) {
                return Err(ValueOfInformationError::UnknownDecisionState {
                    decision_id: decision_id.clone(),
                    state_id: state_id.clone(),
                });
            }
            if !loss.is_finite() || *loss < 0.0 {
                return Err(ValueOfInformationError::InvalidLoss {
                    decision_id: decision_id.clone(),
                    state_id: state_id.clone(),
                    value: *loss,
                });
            }
        }
        for state_id in model.state_probabilities.keys() {
            if !losses.contains_key(state_id) {
                return Err(ValueOfInformationError::MissingDecisionLoss {
                    decision_id: decision_id.clone(),
                    state_id: state_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_investigation(
    model: &DecisionModel,
    investigation: &Investigation,
) -> Result<(), ValueOfInformationError> {
    if investigation.investigation_id.trim().is_empty() {
        return Err(ValueOfInformationError::EmptyInvestigationId);
    }
    if !investigation.cost.is_finite() || investigation.cost < 0.0 {
        return Err(ValueOfInformationError::InvalidCost {
            investigation_id: investigation.investigation_id.clone(),
            cost: investigation.cost,
        });
    }
    if investigation.outcome_likelihoods.is_empty() {
        return Err(ValueOfInformationError::EmptyOutcomeId);
    }

    for (outcome_id, likelihoods) in &investigation.outcome_likelihoods {
        if outcome_id.trim().is_empty() {
            return Err(ValueOfInformationError::EmptyOutcomeId);
        }
        for (state_id, probability) in likelihoods {
            if !model.state_probabilities.contains_key(state_id) {
                return Err(ValueOfInformationError::UnknownOutcomeState {
                    investigation_id: investigation.investigation_id.clone(),
                    outcome_id: outcome_id.clone(),
                    state_id: state_id.clone(),
                });
            }
            if !probability.is_finite() || !(0.0..=1.0).contains(probability) {
                return Err(ValueOfInformationError::InvalidProbability {
                    label: format!(
                        "P(outcome {} | state {}) for investigation {}",
                        outcome_id, state_id, investigation.investigation_id
                    ),
                    value: *probability,
                });
            }
        }
        for state_id in model.state_probabilities.keys() {
            if !likelihoods.contains_key(state_id) {
                return Err(ValueOfInformationError::MissingOutcomeLikelihood {
                    investigation_id: investigation.investigation_id.clone(),
                    outcome_id: outcome_id.clone(),
                    state_id: state_id.clone(),
                });
            }
        }
    }

    for state_id in model.state_probabilities.keys() {
        let sum: f64 = investigation
            .outcome_likelihoods
            .values()
            .map(|likelihoods| likelihoods[state_id])
            .sum();
        if (sum - 1.0).abs() > PROBABILITY_TOLERANCE {
            return Err(ValueOfInformationError::LikelihoodsDoNotSumToOne {
                investigation_id: investigation.investigation_id.clone(),
                state_id: state_id.clone(),
                sum,
            });
        }
    }
    Ok(())
}

fn choose_loss(losses: impl Iterator<Item = f64>) -> f64 {
    losses.fold(f64::INFINITY, f64::min)
}

/// Evaluate an investigation using expected losses and Bayes' rule.
///
/// EVSI = minimum expected loss before investigation minus expected minimum
/// loss after observing the test outcome. The priors, losses, likelihoods, and
/// cost must use a coherent model and a single utility unit (typically money).
pub fn assess_investigation(
    model: &DecisionModel,
    investigation: &Investigation,
) -> Result<InvestigationValue, ValueOfInformationError> {
    validate_model(model)?;
    validate_investigation(model, investigation)?;

    let before = choose_loss(model.losses.values().map(|losses| {
        model
            .state_probabilities
            .iter()
            .map(|(state, probability)| probability * losses[state])
            .sum()
    }));

    let perfect_information = model
        .state_probabilities
        .iter()
        .map(|(state, probability)| {
            probability
                * choose_loss(model.losses.values().map(|losses| losses[state]))
        })
        .sum::<f64>();

    let mut after_sample_information = 0.0;
    for likelihoods in investigation.outcome_likelihoods.values() {
        let outcome_probability: f64 = model
            .state_probabilities
            .iter()
            .map(|(state, prior)| prior * likelihoods[state])
            .sum();

        if outcome_probability <= f64::EPSILON {
            continue;
        }

        let best_conditional_expected_loss = choose_loss(model.losses.values().map(|losses| {
            model
                .state_probabilities
                .iter()
                .map(|(state, prior)| {
                    prior * likelihoods[state] / outcome_probability * losses[state]
                })
                .sum()
        }));
        after_sample_information += outcome_probability * best_conditional_expected_loss;
    }

    let evsi = (before - after_sample_information).max(0.0);
    let evpi = (before - perfect_information).max(0.0);

    // Protect against accidental implementation errors or malformed floating-point inputs.
    if evsi > evpi + NUMERIC_TOLERANCE {
        return Err(ValueOfInformationError::InvalidProbability {
            label: format!(
                "computed EVSI exceeds EVPI for investigation {}",
                investigation.investigation_id
            ),
            value: evsi / evpi.max(f64::EPSILON),
        });
    }

    let net_value = evsi - investigation.cost;
    Ok(InvestigationValue {
        investigation_id: investigation.investigation_id.clone(),
        expected_loss_before_information: before,
        expected_loss_after_sample_information: after_sample_information,
        expected_loss_with_perfect_information: perfect_information,
        evsi,
        evpi,
        investigation_cost: investigation.cost,
        net_value,
        recommended_for_economic_review: net_value > 0.0,
    })
}

/// Rank investigations by net expected value, then EVSI, then stable identifier.
/// This is an economic triage only; safety-critical or legally mandatory surveys
/// must be performed regardless of an estimated negative net value.
pub fn rank_investigations(
    model: &DecisionModel,
    investigations: &[Investigation],
) -> Result<Vec<InvestigationValue>, ValueOfInformationError> {
    validate_model(model)?;
    let mut seen = BTreeSet::new();
    let mut values = Vec::with_capacity(investigations.len());
    for investigation in investigations {
        if !seen.insert(investigation.investigation_id.as_str()) {
            return Err(ValueOfInformationError::DuplicateInvestigationId(
                investigation.investigation_id.clone(),
            ));
        }
        values.push(assess_investigation(model, investigation)?);
    }
    values.sort_by(|a, b| {
        b.net_value
            .total_cmp(&a.net_value)
            .then_with(|| b.evsi.total_cmp(&a.evsi))
            .then_with(|| a.investigation_id.cmp(&b.investigation_id))
    });
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision_model() -> DecisionModel {
        DecisionModel {
            state_probabilities: [
                ("good_ground".into(), 0.7),
                ("poor_ground".into(), 0.3),
            ]
            .into_iter()
            .collect(),
            losses: [
                (
                    "standard_foundation".into(),
                    [
                        ("good_ground".into(), 500_000.0),
                        ("poor_ground".into(), 4_000_000.0),
                    ]
                    .into_iter()
                    .collect(),
                ),
                (
                    "deep_foundation".into(),
                    [
                        ("good_ground".into(), 1_200_000.0),
                        ("poor_ground".into(), 1_500_000.0),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn investigation(
        investigation_id: &str,
        cost: f64,
        high_if_good: f64,
        high_if_poor: f64,
    ) -> Investigation {
        Investigation {
            investigation_id: investigation_id.into(),
            cost,
            outcome_likelihoods: [
                (
                    "high_strength".into(),
                    [
                        ("good_ground".into(), high_if_good),
                        ("poor_ground".into(), high_if_poor),
                    ]
                    .into_iter()
                    .collect(),
                ),
                (
                    "low_strength".into(),
                    [
                        ("good_ground".into(), 1.0 - high_if_good),
                        ("poor_ground".into(), 1.0 - high_if_poor),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn perfect_information_reaches_the_evpi_upper_bound() {
        let model = decision_model();
        let test = Investigation {
            investigation_id: "perfect_borehole_test".into(),
            cost: 25_000.0,
            outcome_likelihoods: [
                (
                    "good_signal".into(),
                    [
                        ("good_ground".into(), 1.0),
                        ("poor_ground".into(), 0.0),
                    ]
                    .into_iter()
                    .collect(),
                ),
                (
                    "poor_signal".into(),
                    [
                        ("good_ground".into(), 0.0),
                        ("poor_ground".into(), 1.0),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]
            .into_iter()
            .collect(),
        };
        let value = assess_investigation(&model, &test).unwrap();
        assert!((value.expected_loss_before_information - 1_290_000.0).abs() < 1e-6);
        assert!((value.expected_loss_with_perfect_information - 800_000.0).abs() < 1e-6);
        assert!((value.evsi - value.evpi).abs() < 1e-6);
        assert!((value.net_value - 465_000.0).abs() < 1e-6);
        assert!(value.recommended_for_economic_review);
    }

    #[test]
    fn uninformative_investigation_has_zero_evsi() {
        let model = decision_model();
        let test = investigation("uninformative", 1_000.0, 0.5, 0.5);
        let value = assess_investigation(&model, &test).unwrap();
        assert!(value.evsi.abs() < 1e-7);
        assert!((value.net_value + 1_000.0).abs() < 1e-7);
        assert!(!value.recommended_for_economic_review);
    }

    #[test]
    fn ranking_prefers_investigations_with_higher_net_value() {
        let model = decision_model();
        let options = [
            investigation("weak_signal", 10_000.0, 0.6, 0.4),
            investigation("strong_signal", 25_000.0, 0.95, 0.3),
            investigation("uninformative", 1_000.0, 0.5, 0.5),
        ];
        let ranked = rank_investigations(&model, &options).unwrap();
        assert_eq!(ranked.first().unwrap().investigation_id, "strong_signal");
        assert!(ranked[0].net_value > ranked[1].net_value);
        assert!(!ranked.last().unwrap().recommended_for_economic_review);
    }

    #[test]
    fn invalid_outcome_likelihood_sum_is_rejected() {
        let model = decision_model();
        let mut test = investigation("bad-probabilities", 10.0, 0.8, 0.6);
        test.outcome_likelihoods
            .get_mut("low_strength")
            .unwrap()
            .insert("poor_ground".into(), 0.6);
        assert!(matches!(
            assess_investigation(&model, &test),
            Err(ValueOfInformationError::LikelihoodsDoNotSumToOne { .. })
        ));
    }

    #[test]
    fn duplicate_investigation_ids_are_rejected() {
        let model = decision_model();
        let a = investigation("same", 10.0, 0.8, 0.4);
        let b = investigation("same", 20.0, 0.9, 0.2);
        assert!(matches!(
            rank_investigations(&model, &[a, b]),
            Err(ValueOfInformationError::DuplicateInvestigationId(_))
        ));
    }
}
