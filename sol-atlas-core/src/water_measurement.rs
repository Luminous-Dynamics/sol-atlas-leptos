// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Bayesian information-value calculations for explicitly declared measurement models.
//!
//! This module is deliberately solver-agnostic. It operates on a caller-supplied
//! prior, outcome likelihoods and decision-loss matrix. It does not generate
//! hydraulic observations, calibrate EPANET/SWMM, authenticate inputs, or prove
//! that a probability model represents a real utility network.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const PROBABILITY_TOLERANCE: f64 = 1.0e-9;
const NUMERIC_TOLERANCE: f64 = 1.0e-10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementModelInput {
    pub schema_version: String,
    pub classification: String,
    pub model_id: String,
    pub hypothesis_ids: Vec<String>,
    pub prior: Vec<f64>,
    pub decision_ids: Vec<String>,
    /// Rows are decisions; columns are hypotheses. Loss and cost units must
    /// be the same declared unit before net value can be ranked.
    pub decision_loss_matrix: Vec<Vec<f64>>,
    pub loss_unit: String,
    pub cost_unit: String,
    pub cost_comparability: String,
    pub actions: Vec<MeasurementActionInput>,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementActionInput {
    pub action_id: String,
    pub outcome_ids: Vec<String>,
    /// Rows are hypotheses; columns are outcomes P(outcome | hypothesis).
    pub likelihoods: Vec<Vec<f64>>,
    pub acquisition_cost: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MeasurementActionResult {
    pub action_id: String,
    pub expected_information_gain_bits: f64,
    pub baseline_minimum_expected_loss: f64,
    pub expected_loss_after_observation: f64,
    pub expected_value_of_sample_information: f64,
    pub acquisition_cost: f64,
    pub net_value_in_loss_units: Option<f64>,
    pub outcome_probabilities: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MeasurementModelEvaluation {
    pub schema_version: String,
    pub evaluation_state: String,
    pub model_id: String,
    pub classification: String,
    pub cost_comparability: String,
    pub selection_state: String,
    pub selected_measurement_action: Option<String>,
    pub actions: Vec<MeasurementActionResult>,
    pub claim_ceiling: String,
    pub nonclaims: Vec<String>,
}

fn valid_identifier(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte))
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn validate_unique_ids(ids: &[String], label: &str) -> Result<(), String> {
    if ids.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    let mut seen = HashSet::new();
    for id in ids {
        if !valid_identifier(id) {
            return Err(format!("{label} contains invalid identifier {id:?}"));
        }
        if !seen.insert(id) {
            return Err(format!("{label} contains duplicate identifier {id:?}"));
        }
    }
    Ok(())
}

fn validate_distribution(probabilities: &[f64], label: &str) -> Result<(), String> {
    if probabilities.is_empty() {
        return Err(format!("{label} distribution must not be empty"));
    }
    let mut total = 0.0;
    for probability in probabilities {
        if !probability.is_finite() || !(0.0..=1.0).contains(probability) {
            return Err(format!(
                "{label} probability {probability:?} is not finite and in [0, 1]"
            ));
        }
        total += probability;
    }
    if (total - 1.0).abs() > PROBABILITY_TOLERANCE {
        return Err(format!(
            "{label} probabilities sum to {total}, not 1 within tolerance {PROBABILITY_TOLERANCE}"
        ));
    }
    Ok(())
}

/// Shannon entropy in bits. Zero-probability terms contribute zero.
pub fn entropy_bits(probabilities: &[f64]) -> Result<f64, String> {
    validate_distribution(probabilities, "entropy input")?;
    Ok(probabilities
        .iter()
        .filter(|probability| **probability > 0.0)
        .map(|probability| -(*probability) * probability.log2())
        .sum())
}

fn validate_input(input: &MeasurementModelInput) -> Result<(), String> {
    if input.schema_version != "water-measurement-model/v0.1" {
        return Err("schema_version must be water-measurement-model/v0.1".to_owned());
    }
    if input.classification != "synthetic_illustrative" {
        return Err(
            "v0.1 calculator only accepts classification=synthetic_illustrative".to_owned(),
        );
    }
    if !valid_identifier(&input.model_id) {
        return Err("model_id is missing or invalid".to_owned());
    }
    validate_unique_ids(&input.hypothesis_ids, "hypothesis_ids")?;
    validate_unique_ids(&input.decision_ids, "decision_ids")?;
    if input.hypothesis_ids.len() < 2 {
        return Err("at least two hypotheses are required".to_owned());
    }
    if input.decision_ids.len() < 2 {
        return Err("at least two candidate decisions are required".to_owned());
    }
    validate_distribution(&input.prior, "prior")?;
    if input.prior.len() != input.hypothesis_ids.len() {
        return Err("prior length must match hypothesis_ids".to_owned());
    }
    if input.decision_loss_matrix.len() != input.decision_ids.len() {
        return Err("decision_loss_matrix row count must match decision_ids".to_owned());
    }
    for (row_index, row) in input.decision_loss_matrix.iter().enumerate() {
        if row.len() != input.hypothesis_ids.len() {
            return Err(format!(
                "decision_loss_matrix row {row_index} length must match hypothesis_ids"
            ));
        }
        for loss in row {
            if !loss.is_finite() || *loss < 0.0 {
                return Err(format!(
                    "decision_loss_matrix contains invalid loss {loss:?}; losses must be finite and non-negative"
                ));
            }
        }
    }
    if input.loss_unit.trim().is_empty() || input.cost_unit.trim().is_empty() {
        return Err("loss_unit and cost_unit must be non-empty".to_owned());
    }
    if !matches!(
        input.cost_comparability.as_str(),
        "same_declared_unit" | "not_comparable"
    ) {
        return Err("cost_comparability must be same_declared_unit or not_comparable".to_owned());
    }
    if input.cost_comparability == "same_declared_unit" && input.loss_unit != input.cost_unit {
        return Err(
            "same_declared_unit requires loss_unit and cost_unit to be identical".to_owned(),
        );
    }
    if input.actions.is_empty() {
        return Err("at least one measurement action is required".to_owned());
    }
    let mut action_ids = HashSet::new();
    for action in &input.actions {
        if !valid_identifier(&action.action_id) {
            return Err(format!("invalid action_id {:?}", action.action_id));
        }
        if !action_ids.insert(&action.action_id) {
            return Err(format!("duplicate action_id {:?}", action.action_id));
        }
        validate_unique_ids(&action.outcome_ids, "action outcome_ids")?;
        if action.likelihoods.len() != input.hypothesis_ids.len() {
            return Err(format!(
                "action {} likelihood row count must match hypotheses",
                action.action_id
            ));
        }
        for (hypothesis_index, row) in action.likelihoods.iter().enumerate() {
            if row.len() != action.outcome_ids.len() {
                return Err(format!(
                    "action {} likelihood row {hypothesis_index} length must match outcome_ids",
                    action.action_id
                ));
            }
            validate_distribution(
                row,
                &format!("action {} likelihood row {hypothesis_index}", action.action_id),
            )?;
        }
        if !action.acquisition_cost.is_finite() || action.acquisition_cost < 0.0 {
            return Err(format!(
                "action {} acquisition_cost must be finite and non-negative",
                action.action_id
            ));
        }
    }
    if input.assumptions.is_empty() || input.assumptions.iter().any(|s| s.trim().is_empty()) {
        return Err("at least one non-empty assumption is required".to_owned());
    }
    Ok(())
}

fn expected_loss(prior: &[f64], loss_matrix: &[Vec<f64>]) -> f64 {
    loss_matrix
        .iter()
        .map(|decision_losses| {
            prior
                .iter()
                .zip(decision_losses)
                .map(|(probability, loss)| probability * loss)
                .sum::<f64>()
        })
        .fold(f64::INFINITY, f64::min)
}

fn expected_information_gain(
    prior: &[f64],
    likelihoods: &[Vec<f64>],
    outcome_count: usize,
) -> Result<(f64, Vec<f64>), String> {
    let prior_entropy = entropy_bits(prior)?;
    let mut outcome_probabilities = vec![0.0; outcome_count];
    for (hypothesis_index, prior_probability) in prior.iter().enumerate() {
        for (outcome_index, likelihood) in likelihoods[hypothesis_index].iter().enumerate() {
            outcome_probabilities[outcome_index] += prior_probability * likelihood;
        }
    }
    let mut expected_posterior_entropy = 0.0;
    for (outcome_index, outcome_probability) in outcome_probabilities.iter().enumerate() {
        if *outcome_probability <= 0.0 {
            continue;
        }
        let unnormalised: Vec<f64> = prior
            .iter()
            .enumerate()
            .map(|(hypothesis_index, prior_probability)| {
                prior_probability * likelihoods[hypothesis_index][outcome_index]
            })
            .collect();
        let posterior: Vec<f64> = unnormalised
            .iter()
            .map(|joint_probability| joint_probability / outcome_probability)
            .collect();
        let posterior_entropy = entropy_bits(&posterior)?;
        expected_posterior_entropy += outcome_probability * posterior_entropy;
    }
    let mut gain = prior_entropy - expected_posterior_entropy;
    if gain < 0.0 && gain.abs() <= NUMERIC_TOLERANCE {
        gain = 0.0;
    }
    if !gain.is_finite() || gain < 0.0 || gain > prior_entropy + NUMERIC_TOLERANCE {
        return Err(format!("computed information gain is outside valid bounds: {gain}"));
    }
    Ok((gain, outcome_probabilities))
}

fn expected_loss_after_observation(
    prior: &[f64],
    likelihoods: &[Vec<f64>],
    loss_matrix: &[Vec<f64>],
    outcome_probabilities: &[f64],
) -> Result<f64, String> {
    let mut total = 0.0;
    for (outcome_index, outcome_probability) in outcome_probabilities.iter().enumerate() {
        if *outcome_probability <= 0.0 {
            continue;
        }
        let posterior: Vec<f64> = prior
            .iter()
            .enumerate()
            .map(|(hypothesis_index, prior_probability)| {
                prior_probability * likelihoods[hypothesis_index][outcome_index]
                    / outcome_probability
            })
            .collect();
        let minimum_posterior_loss = expected_loss(&posterior, loss_matrix);
        total += outcome_probability * minimum_posterior_loss;
    }
    if !total.is_finite() {
        return Err("expected loss after observation is not finite".to_owned());
    }
    Ok(total)
}

/// Evaluate information gain and decision-loss reduction for declared toy models.
/// Any returned number is conditional on the exact supplied prior, likelihoods,
/// loss matrix and costs. It is not a measurement result or hydraulic prediction.
pub fn evaluate_measurement_model(
    input: &MeasurementModelInput,
) -> Result<MeasurementModelEvaluation, String> {
    validate_input(input)?;
    let baseline_minimum_expected_loss = expected_loss(&input.prior, &input.decision_loss_matrix);
    if !baseline_minimum_expected_loss.is_finite() {
        return Err("baseline expected loss is not finite".to_owned());
    }

    let mut results = Vec::with_capacity(input.actions.len());
    for action in &input.actions {
        let (information_gain, outcome_probabilities) = expected_information_gain(
            &input.prior,
            &action.likelihoods,
            action.outcome_ids.len(),
        )?;
        let expected_loss_after_observation = expected_loss_after_observation(
            &input.prior,
            &action.likelihoods,
            &input.decision_loss_matrix,
            &outcome_probabilities,
        )?;
        let mut value_of_sample_information =
            baseline_minimum_expected_loss - expected_loss_after_observation;
        if value_of_sample_information < 0.0
            && value_of_sample_information.abs() <= NUMERIC_TOLERANCE
        {
            value_of_sample_information = 0.0;
        }
        if !value_of_sample_information.is_finite() || value_of_sample_information < 0.0 {
            return Err(format!(
                "action {} has impossible negative value of sample information: {}",
                action.action_id, value_of_sample_information
            ));
        }
        let net_value_in_loss_units = if input.cost_comparability == "same_declared_unit" {
            Some(value_of_sample_information - action.acquisition_cost)
        } else {
            None
        };
        results.push(MeasurementActionResult {
            action_id: action.action_id.clone(),
            expected_information_gain_bits: information_gain,
            baseline_minimum_expected_loss,
            expected_loss_after_observation,
            expected_value_of_sample_information: value_of_sample_information,
            acquisition_cost: action.acquisition_cost,
            net_value_in_loss_units,
            outcome_probabilities,
        });
    }

    let (selection_state, selected_measurement_action) =
        if input.cost_comparability != "same_declared_unit" {
            ("not_rankable_cost_units_incomparable", None)
        } else {
            let best = results.iter().max_by(|left, right| {
                left.net_value_in_loss_units
                    .unwrap_or(f64::NEG_INFINITY)
                    .total_cmp(&right.net_value_in_loss_units.unwrap_or(f64::NEG_INFINITY))
                    .then_with(|| right.action_id.cmp(&left.action_id))
            });
            match best {
                Some(best) if best.net_value_in_loss_units.unwrap_or(f64::NEG_INFINITY) > 0.0 => {
                    ("positive_net_value_candidate", Some(best.action_id.clone()))
                }
                Some(_) => ("no_positive_net_value_candidate", None),
                None => ("no_actions", None),
            }
        };

    Ok(MeasurementModelEvaluation {
        schema_version: "water-measurement-evaluation/v0.1".to_owned(),
        evaluation_state: "computed_from_declared_synthetic_probabilities".to_owned(),
        model_id: input.model_id.clone(),
        classification: input.classification.clone(),
        cost_comparability: input.cost_comparability.clone(),
        selection_state: selection_state.to_owned(),
        selected_measurement_action,
        actions: results,
        claim_ceiling: "illustrative_calculation_only".to_owned(),
        nonclaims: vec![
            "Not a hydraulic simulation, calibration result, or measurement from a real water network.".to_owned(),
            "Information gain is not automatically decision value; the loss matrix encodes assumptions.".to_owned(),
            "Net value is omitted when acquisition costs and decision losses are not comparable in the same declared unit.".to_owned(),
            "No fieldwork, infrastructure intervention, or operational control is authorised.".to_owned(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn toy_model() -> MeasurementModelInput {
        serde_json::from_value(json!({
            "schema_version":"water-measurement-model/v0.1",
            "classification":"synthetic_illustrative",
            "model_id":"toy-leak-hypotheses",
            "hypothesis_ids":["no_leak","leak_a"],
            "prior":[0.5,0.5],
            "decision_ids":["monitor","inspect_a"],
            "decision_loss_matrix":[[0.0,10.0],[4.0,0.0]],
            "loss_unit":"toy_loss_units",
            "cost_unit":"toy_loss_units",
            "cost_comparability":"same_declared_unit",
            "actions":[
                {
                    "action_id":"perfect_sensor",
                    "outcome_ids":["normal","alarm"],
                    "likelihoods":[[1.0,0.0],[0.0,1.0]],
                    "acquisition_cost":1.0
                },
                {
                    "action_id":"uninformative_sensor",
                    "outcome_ids":["normal","alarm"],
                    "likelihoods":[[0.5,0.5],[0.5,0.5]],
                    "acquisition_cost":0.0
                }
            ],
            "assumptions":[
                "Hypothesis priors and likelihoods are hand-authored solely to check arithmetic.",
                "Loss and cost share a toy unit only within this illustrative example."
            ]
        })).expect("toy input deserializes")
    }

    #[test]
    fn perfect_sensor_reduces_entropy_and_expected_loss() {
        let evaluation = evaluate_measurement_model(&toy_model()).expect("valid toy model");
        let perfect = evaluation.actions.iter().find(|r| r.action_id == "perfect_sensor").unwrap();
        assert!((perfect.expected_information_gain_bits - 1.0).abs() < NUMERIC_TOLERANCE);
        assert!((perfect.expected_value_of_sample_information - 2.0).abs() < NUMERIC_TOLERANCE);
        assert!((perfect.net_value_in_loss_units.unwrap() - 1.0).abs() < NUMERIC_TOLERANCE);
        assert_eq!(evaluation.selected_measurement_action.as_deref(), Some("perfect_sensor"));
        assert_eq!(evaluation.claim_ceiling, "illustrative_calculation_only");
    }

    #[test]
    fn uninformative_sensor_has_zero_information_and_zero_decision_value() {
        let evaluation = evaluate_measurement_model(&toy_model()).expect("valid toy model");
        let uninformative = evaluation
            .actions
            .iter()
            .find(|r| r.action_id == "uninformative_sensor")
            .unwrap();
        assert!(uninformative.expected_information_gain_bits.abs() < NUMERIC_TOLERANCE);
        assert!(uninformative.expected_value_of_sample_information.abs() < NUMERIC_TOLERANCE);
        assert!(uninformative.net_value_in_loss_units.unwrap() <= 0.0);
    }

    #[test]
    fn incompatible_cost_units_disable_net_ranking() {
        let mut model = toy_model();
        model.cost_comparability = "not_comparable".to_owned();
        model.cost_unit = "hours".to_owned();
        let evaluation =
            evaluate_measurement_model(&model).expect("information metrics remain computable");
        assert_eq!(evaluation.selection_state, "not_rankable_cost_units_incomparable");
        assert_eq!(evaluation.selected_measurement_action, None);
        assert!(evaluation.actions.iter().all(|a| a.net_value_in_loss_units.is_none()));
    }

    #[test]
    fn non_normalized_prior_is_rejected() {
        let mut model = toy_model();
        model.prior = vec![0.8, 0.8];
        assert!(evaluate_measurement_model(&model).unwrap_err().contains("sum to"));
    }

    #[test]
    fn malformed_likelihood_dimensions_are_rejected() {
        let mut model = toy_model();
        model.actions[0].likelihoods[0] = vec![1.0];
        assert!(evaluate_measurement_model(&model).unwrap_err().contains("length must match"));
    }

    #[test]
    fn zero_probability_outcomes_do_not_create_nan_posteriors() {
        let mut model = toy_model();
        model.prior = vec![1.0, 0.0];
        model.actions[0].likelihoods = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let evaluation =
            evaluate_measurement_model(&model).expect("zero-probability branch is valid");
        let perfect = evaluation.actions.iter().find(|r| r.action_id == "perfect_sensor").unwrap();
        assert!(perfect.expected_information_gain_bits.abs() < NUMERIC_TOLERANCE);
        assert!(perfect.expected_value_of_sample_information.abs() < NUMERIC_TOLERANCE);
    }

    #[test]
    fn identifiers_match_the_lowercase_contract_even_without_schema_validation() {
        let mut model = toy_model();
        model.actions[0].action_id = "Leak_A".to_owned();
        assert!(
            evaluate_measurement_model(&model)
                .unwrap_err()
                .contains("invalid action_id")
        );
    }

    #[test]
    fn mismatched_same_unit_declaration_is_rejected() {
        let mut model = toy_model();
        model.cost_unit = "currency_units".to_owned();
        assert!(evaluate_measurement_model(&model).unwrap_err().contains("must be identical"));
    }
}
