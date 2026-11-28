use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs, process};

const EPSILON: f64 = 1e-10;

#[derive(Clone, Debug)]
struct Hypothesis {
    id: String,
    prior: f64,
}

#[derive(Clone, Debug)]
struct Action {
    id: String,
    utilities: BTreeMap<String, f64>,
}

#[derive(Clone, Debug)]
struct Survey {
    id: String,
    effort_units: usize,
    utility_cost: f64,
    positive_likelihood: BTreeMap<String, f64>,
}

#[derive(Clone, Debug)]
struct Problem {
    hypotheses: Vec<Hypothesis>,
    actions: Vec<Action>,
    surveys: Vec<Survey>,
    max_effort_units: usize,
}

#[derive(Clone, Debug)]
struct PortfolioScore {
    selected_indices: Vec<usize>,
    expected_post_survey_utility: f64,
    effort_units: usize,
    utility_cost: f64,
    gross_evsi: f64,
    net_evsi: f64,
}

fn fail(message: impl AsRef<str>) -> ! {
    panic!("INVALID: {}", message.as_ref());
}

fn required<'a>(value: &'a Value, key: &str, path: &str) -> &'a Value {
    value
        .get(key)
        .unwrap_or_else(|| fail(format!("{path}.{key} is required")))
}

fn as_array<'a>(value: &'a Value, path: &str) -> &'a [Value] {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_else(|| fail(format!("{path} must be an array")))
}

fn as_string(value: &Value, path: &str) -> String {
    value
        .as_str()
        .unwrap_or_else(|| fail(format!("{path} must be a string")))
        .to_owned()
}

fn as_number(value: &Value, path: &str) -> f64 {
    let number = value
        .as_f64()
        .unwrap_or_else(|| fail(format!("{path} must be numeric")));
    if !number.is_finite() {
        fail(format!("{path} must be finite"));
    }
    number
}

fn parse_problem(case: &Value) -> Problem {
    if as_string(
        required(case, "fixtureStatus", "case"),
        "case.fixtureStatus",
    ) != "synthetic"
    {
        fail("reference evaluator only accepts the registered synthetic fixture");
    }
    let decision = required(case, "decisionProblem", "case");
    let hypotheses_values = as_array(
        required(decision, "hypotheses", "decisionProblem"),
        "hypotheses",
    );
    if hypotheses_values.len() < 2 {
        fail("at least two hypotheses are required");
    }

    let mut hypotheses = Vec::new();
    let mut prior_sum = 0.0;
    for (index, value) in hypotheses_values.iter().enumerate() {
        let path = format!("hypotheses[{index}]");
        let id = as_string(required(value, "id", &path), &format!("{path}.id"));
        let prior = as_number(required(value, "prior", &path), &format!("{path}.prior"));
        if !(0.0..=1.0).contains(&prior) {
            fail(format!("{path}.prior must be within [0, 1]"));
        }
        if hypotheses.iter().any(|item: &Hypothesis| item.id == id) {
            fail(format!("duplicate hypothesis ID: {id}"));
        }
        prior_sum += prior;
        hypotheses.push(Hypothesis { id, prior });
    }
    if (prior_sum - 1.0).abs() > EPSILON {
        fail(format!(
            "hypothesis priors must sum to 1.0, observed {prior_sum}"
        ));
    }

    let action_values = as_array(required(decision, "actions", "decisionProblem"), "actions");
    if action_values.is_empty() {
        fail("at least one decision action is required");
    }
    let mut actions = Vec::new();
    for (index, value) in action_values.iter().enumerate() {
        let path = format!("actions[{index}]");
        let id = as_string(required(value, "id", &path), &format!("{path}.id"));
        let utility_values = required(value, "utilities", &path);
        let mut utilities = BTreeMap::new();
        for hypothesis in &hypotheses {
            let utility = as_number(
                required(utility_values, &hypothesis.id, &format!("{path}.utilities")),
                &format!("{path}.utilities.{}", hypothesis.id),
            );
            utilities.insert(hypothesis.id.clone(), utility);
        }
        actions.push(Action { id, utilities });
    }

    let max_effort_units = required(decision, "maxEffortUnits", "decisionProblem")
        .as_u64()
        .unwrap_or_else(|| fail("maxEffortUnits must be a non-negative integer"))
        as usize;
    let survey_values = as_array(required(decision, "surveys", "decisionProblem"), "surveys");
    if survey_values.len() > 16 {
        fail("reference evaluator is limited to 16 survey candidates");
    }
    let mut surveys = Vec::new();
    for (index, value) in survey_values.iter().enumerate() {
        let path = format!("surveys[{index}]");
        let id = as_string(required(value, "id", &path), &format!("{path}.id"));
        let effort_units = required(value, "effortUnits", &path)
            .as_u64()
            .unwrap_or_else(|| fail(format!("{path}.effortUnits must be a non-negative integer")))
            as usize;
        let utility_cost = as_number(
            required(value, "utilityCost", &path),
            &format!("{path}.utilityCost"),
        );
        if utility_cost < 0.0 {
            fail(format!("{path}.utilityCost cannot be negative"));
        }
        let probabilities = required(value, "positiveLikelihoodByHypothesis", &path);
        let mut positive_likelihood = BTreeMap::new();
        for hypothesis in &hypotheses {
            let probability = as_number(
                required(
                    probabilities,
                    &hypothesis.id,
                    &format!("{path}.positiveLikelihoodByHypothesis"),
                ),
                &format!("{path}.positiveLikelihoodByHypothesis.{}", hypothesis.id),
            );
            if !(0.0..=1.0).contains(&probability) {
                fail(format!(
                    "{path}.positiveLikelihoodByHypothesis.{} must be within [0, 1]",
                    hypothesis.id
                ));
            }
            positive_likelihood.insert(hypothesis.id.clone(), probability);
        }
        if surveys.iter().any(|item: &Survey| item.id == id) {
            fail(format!("duplicate survey ID: {id}"));
        }
        surveys.push(Survey {
            id,
            effort_units,
            utility_cost,
            positive_likelihood,
        });
    }

    // Stable ID ordering makes the result independent of transport-array order.
    surveys.sort_by(|a, b| a.id.cmp(&b.id));
    Problem {
        hypotheses,
        actions,
        surveys,
        max_effort_units,
    }
}

fn best_action_value(problem: &Problem, probabilities: &[f64]) -> (String, f64) {
    let mut best: Option<(String, f64)> = None;
    for action in &problem.actions {
        let value = problem
            .hypotheses
            .iter()
            .zip(probabilities.iter())
            .map(|(hypothesis, probability)| probability * action.utilities[&hypothesis.id])
            .sum::<f64>();
        if best.as_ref().is_none_or(|(_, current)| value > *current) {
            best = Some((action.id.clone(), value));
        }
    }
    best.unwrap_or_else(|| fail("no decision action available"))
}

fn evaluate_portfolio(
    problem: &Problem,
    selected_indices: &[usize],
    baseline: f64,
) -> PortfolioScore {
    let effort_units = selected_indices
        .iter()
        .map(|index| problem.surveys[*index].effort_units)
        .sum::<usize>();
    let utility_cost = selected_indices
        .iter()
        .map(|index| problem.surveys[*index].utility_cost)
        .sum::<f64>();

    let outcome_count = 1usize << selected_indices.len();
    let mut expected_post_survey_utility = 0.0;
    for outcome_mask in 0..outcome_count {
        let mut joint_probabilities: Vec<f64> = problem
            .hypotheses
            .iter()
            .map(|hypothesis| hypothesis.prior)
            .collect();

        for (bit, survey_index) in selected_indices.iter().enumerate() {
            let survey = &problem.surveys[*survey_index];
            let positive_outcome = outcome_mask & (1usize << bit) != 0;
            for (hypothesis_index, hypothesis) in problem.hypotheses.iter().enumerate() {
                let positive_probability = survey.positive_likelihood[&hypothesis.id];
                let outcome_probability = if positive_outcome {
                    positive_probability
                } else {
                    1.0 - positive_probability
                };
                joint_probabilities[hypothesis_index] *= outcome_probability;
            }
        }

        let outcome_probability = joint_probabilities.iter().sum::<f64>();
        if outcome_probability <= EPSILON {
            continue;
        }
        let posterior: Vec<f64> = joint_probabilities
            .iter()
            .map(|probability| probability / outcome_probability)
            .collect();
        let (_, best_conditional_utility) = best_action_value(problem, &posterior);
        expected_post_survey_utility += outcome_probability * best_conditional_utility;
    }

    let gross_evsi = expected_post_survey_utility - baseline;
    PortfolioScore {
        selected_indices: selected_indices.to_vec(),
        expected_post_survey_utility,
        effort_units,
        utility_cost,
        gross_evsi,
        net_evsi: gross_evsi - utility_cost,
    }
}

fn entropy_bits(probabilities: &[f64]) -> f64 {
    probabilities
        .iter()
        .filter(|probability| **probability > 0.0)
        .map(|probability| -probability * probability.log2())
        .sum()
}

fn expected_information_gain(problem: &Problem, survey: &Survey) -> f64 {
    let prior: Vec<f64> = problem
        .hypotheses
        .iter()
        .map(|hypothesis| hypothesis.prior)
        .collect();
    let prior_entropy = entropy_bits(&prior);
    let mut expected_posterior_entropy = 0.0;

    for positive_outcome in [true, false] {
        let joint: Vec<f64> = problem
            .hypotheses
            .iter()
            .map(|hypothesis| {
                let probability = survey.positive_likelihood[&hypothesis.id];
                hypothesis.prior
                    * if positive_outcome {
                        probability
                    } else {
                        1.0 - probability
                    }
            })
            .collect();
        let outcome_probability = joint.iter().sum::<f64>();
        if outcome_probability <= EPSILON {
            continue;
        }
        let posterior: Vec<f64> = joint
            .iter()
            .map(|probability| probability / outcome_probability)
            .collect();
        expected_posterior_entropy += outcome_probability * entropy_bits(&posterior);
    }

    prior_entropy - expected_posterior_entropy
}

fn calculate(case: &Value) -> Value {
    let problem = parse_problem(case);
    let prior: Vec<f64> = problem
        .hypotheses
        .iter()
        .map(|hypothesis| hypothesis.prior)
        .collect();
    let (baseline_action, baseline_utility) = best_action_value(&problem, &prior);

    let mut best = evaluate_portfolio(&problem, &[], baseline_utility);
    let portfolio_count = 1usize << problem.surveys.len();
    for mask in 1..portfolio_count {
        let selected: Vec<usize> = (0..problem.surveys.len())
            .filter(|index| mask & (1usize << index) != 0)
            .collect();
        let candidate = evaluate_portfolio(&problem, &selected, baseline_utility);
        if candidate.effort_units > problem.max_effort_units {
            continue;
        }
        let better_value = candidate.net_evsi > best.net_evsi + EPSILON;
        let stable_tie_break = (candidate.net_evsi - best.net_evsi).abs() <= EPSILON
            && candidate.selected_indices.len() < best.selected_indices.len();
        if better_value || stable_tie_break {
            best = candidate;
        }
    }

    let selected_ids: Vec<String> = best
        .selected_indices
        .iter()
        .map(|index| problem.surveys[*index].id.clone())
        .collect();

    let mut information_gain: Vec<(String, f64)> = problem
        .surveys
        .iter()
        .map(|survey| {
            (
                survey.id.clone(),
                expected_information_gain(&problem, survey),
            )
        })
        .collect();
    information_gain.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let (top_eig_survey_id, top_eig_bits) = information_gain
        .first()
        .cloned()
        .unwrap_or_else(|| fail("no survey candidates available"));
    let top_eig_index = problem
        .surveys
        .iter()
        .position(|survey| survey.id == top_eig_survey_id)
        .unwrap_or_else(|| fail("highest-EIG survey is missing from candidates"));
    let top_eig_score = evaluate_portfolio(&problem, &[top_eig_index], baseline_utility);

    json!({
        "benchmarkId": "sol-atlas-active-spatial-investigation",
        "version": "0.1.0",
        "fixtureStatus": "synthetic",
        "baseline": {
            "bestAction": baseline_action,
            "expectedUtility": baseline_utility
        },
        "selectedSurveyIds": selected_ids,
        "highestInformationGainSingleSurvey": {
            "surveyId": top_eig_survey_id,
            "expectedInformationGainBits": top_eig_bits,
            "expectedPostSurveyUtility": top_eig_score.expected_post_survey_utility,
            "grossEvsi": top_eig_score.gross_evsi,
            "netEvsi": top_eig_score.net_evsi
        },
        "portfolio": {
            "effortUnits": best.effort_units,
            "utilityCost": best.utility_cost,
            "expectedPostSurveyUtility": best.expected_post_survey_utility,
            "grossEvsi": best.gross_evsi,
            "netEvsi": best.net_evsi
        },
        "informationGainRanking": information_gain
            .iter()
            .map(|(id, score)| json!({
                "surveyId": id,
                "expectedInformationGainBits": score
            }))
            .collect::<Vec<Value>>()
    })
}

fn read_json(path: &str) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|error| fail(format!("cannot read {path}: {error}")));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| fail(format!("invalid JSON in {path}: {error}")))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("fixtures/decision-case.json");
    if args.len() > 2 {
        eprintln!("usage: sol-atlas-active-investigation [decision-case.json]");
        process::exit(64);
    }
    let result = calculate(&read_json(path));
    println!(
        "{}",
        serde_json::to_string_pretty(&result).expect("result serializes")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../../fixtures/decision-case.json")).unwrap()
    }

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() <= 1e-8, "expected {right}, got {left}");
    }

    #[test]
    fn selects_observations_by_net_decision_value_not_entropy_alone() {
        let result = calculate(&fixture());
        assert_eq!(result["baseline"]["bestAction"], "preserve_in_situ");
        close(
            result["baseline"]["expectedUtility"].as_f64().unwrap(),
            72.0,
        );
        assert_eq!(
            result["selectedSurveyIds"],
            json!(["stratigraphic_microsection"])
        );
        close(
            result["portfolio"]["expectedPostSurveyUtility"]
                .as_f64()
                .unwrap(),
            79.125,
        );
        close(result["portfolio"]["grossEvsi"].as_f64().unwrap(), 7.125);
        close(result["portfolio"]["netEvsi"].as_f64().unwrap(), 4.625);
        assert_eq!(
            result["informationGainRanking"][0]["surveyId"],
            "ornamental_subtype_catalog"
        );
        assert_eq!(
            result["highestInformationGainSingleSurvey"]["surveyId"],
            "ornamental_subtype_catalog"
        );
        close(
            result["highestInformationGainSingleSurvey"]["expectedInformationGainBits"]
                .as_f64()
                .unwrap(),
            0.6529738536349244,
        );
        close(
            result["highestInformationGainSingleSurvey"]["grossEvsi"]
                .as_f64()
                .unwrap(),
            0.0,
        );
        close(
            result["highestInformationGainSingleSurvey"]["netEvsi"]
                .as_f64()
                .unwrap(),
            -0.4,
        );
        assert_eq!(result["selectedSurveyIds"][0], "stratigraphic_microsection");
    }

    #[test]
    fn survey_order_does_not_change_the_result() {
        let mut case = fixture();
        case["decisionProblem"]["surveys"]
            .as_array_mut()
            .unwrap()
            .reverse();
        assert_eq!(calculate(&fixture()), calculate(&case));
    }

    #[test]
    fn no_survey_is_selected_when_all_decisions_are_equivalent() {
        let mut case = fixture();
        let hypothesis_ids: Vec<String> = case["decisionProblem"]["hypotheses"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hypothesis| hypothesis["id"].as_str().unwrap().to_owned())
            .collect();
        for action in case["decisionProblem"]["actions"].as_array_mut().unwrap() {
            for hypothesis_id in &hypothesis_ids {
                action["utilities"][hypothesis_id] = json!(50.0);
            }
        }
        let result = calculate(&case);
        assert_eq!(result["selectedSurveyIds"], json!([]));
        close(result["portfolio"]["netEvsi"].as_f64().unwrap(), 0.0);
    }

    #[test]
    #[should_panic(expected = "hypothesis priors must sum to 1.0")]
    fn rejects_unnormalized_priors() {
        let mut case = fixture();
        case["decisionProblem"]["hypotheses"][0]["prior"] = json!(0.9);
        let _ = calculate(&case);
    }

    #[test]
    #[should_panic(expected = "must be within [0, 1]")]
    fn rejects_invalid_likelihoods() {
        let mut case = fixture();
        case["decisionProblem"]["surveys"][0]["positiveLikelihoodByHypothesis"]["later_repair"] =
            json!(1.2);
        let _ = calculate(&case);
    }
}
