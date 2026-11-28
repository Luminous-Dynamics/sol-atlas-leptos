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
    joint_outcomes: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Clone, Debug)]
struct Score {
    selected_indices: Vec<usize>,
    expected_post_survey_utility: f64,
    effort_units: usize,
    utility_cost: f64,
    gross_evsi: f64,
    net_evsi: f64,
}

#[derive(Clone, Debug)]
struct PolicyNode {
    value: f64,
    stop_action: Option<String>,
    survey_index: Option<usize>,
    negative: Option<Box<PolicyNode>>,
    positive: Option<Box<PolicyNode>>,
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
    if as_string(required(case, "fixtureStatus", "case"), "case.fixtureStatus") != "synthetic" {
        fail("reference evaluator only accepts synthetic fixtures");
    }
    let decision = required(case, "decisionProblem", "case");
    let hypothesis_values = as_array(
        required(decision, "hypotheses", "decisionProblem"),
        "decisionProblem.hypotheses",
    );
    if hypothesis_values.len() < 2 {
        fail("at least two hypotheses are required");
    }

    let mut hypotheses = Vec::new();
    let mut prior_sum = 0.0;
    for (index, item) in hypothesis_values.iter().enumerate() {
        let path = format!("hypotheses[{index}]");
        let id = as_string(required(item, "id", &path), &format!("{path}.id"));
        let prior = as_number(required(item, "prior", &path), &format!("{path}.prior"));
        if !(0.0..=1.0).contains(&prior) {
            fail(format!("{path}.prior must be within [0, 1]"));
        }
        if hypotheses.iter().any(|h: &Hypothesis| h.id == id) {
            fail(format!("duplicate hypothesis ID: {id}"));
        }
        prior_sum += prior;
        hypotheses.push(Hypothesis { id, prior });
    }
    if (prior_sum - 1.0).abs() > EPSILON {
        fail(format!("hypothesis priors must sum to 1.0, observed {prior_sum}"));
    }

    let action_values = as_array(
        required(decision, "actions", "decisionProblem"),
        "decisionProblem.actions",
    );
    if action_values.is_empty() {
        fail("at least one action is required");
    }
    let mut actions = Vec::new();
    for (index, item) in action_values.iter().enumerate() {
        let path = format!("actions[{index}]");
        let id = as_string(required(item, "id", &path), &format!("{path}.id"));
        let utility_values = required(item, "utilities", &path);
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
    let survey_values = as_array(
        required(decision, "surveys", "decisionProblem"),
        "decisionProblem.surveys",
    );
    if survey_values.is_empty() || survey_values.len() > 16 {
        fail("the reference evaluator requires 1 to 16 survey candidates");
    }

    let mut surveys = Vec::new();
    for (index, item) in survey_values.iter().enumerate() {
        let path = format!("surveys[{index}]");
        let id = as_string(required(item, "id", &path), &format!("{path}.id"));
        let effort_units = required(item, "effortUnits", &path)
            .as_u64()
            .unwrap_or_else(|| fail(format!("{path}.effortUnits must be a non-negative integer")))
            as usize;
        let utility_cost = as_number(
            required(item, "utilityCost", &path),
            &format!("{path}.utilityCost"),
        );
        if utility_cost < 0.0 {
            fail(format!("{path}.utilityCost cannot be negative"));
        }

        let probabilities = required(item, "positiveLikelihoodByHypothesis", &path);
        let mut positive_likelihood = BTreeMap::new();
        for hypothesis in &hypotheses {
            let probability = as_number(
                required(probabilities, &hypothesis.id, &format!("{path}.positiveLikelihoodByHypothesis")),
                &format!("{path}.positiveLikelihoodByHypothesis.{}", hypothesis.id),
            );
            if !(0.0..=1.0).contains(&probability) {
                fail(format!("{path} likelihood for {} must be within [0, 1]", hypothesis.id));
            }
            positive_likelihood.insert(hypothesis.id.clone(), probability);
        }
        if surveys.iter().any(|s: &Survey| s.id == id) {
            fail(format!("duplicate survey ID: {id}"));
        }
        surveys.push(Survey {
            id,
            effort_units,
            utility_cost,
            positive_likelihood,
        });
    }

    // The joint outcome string is indexed by this stable, sorted survey order.
    surveys.sort_by(|a, b| a.id.cmp(&b.id));
    let joint_values = required(
        decision,
        "jointOutcomeLikelihoodsByHypothesis",
        "decisionProblem",
    );
    let expected_pattern_count = 1usize << surveys.len();
    let mut joint_outcomes = BTreeMap::new();

    for hypothesis in &hypotheses {
        let outcomes = as_array(
            required(joint_values, &hypothesis.id, "jointOutcomeLikelihoodsByHypothesis"),
            &format!("jointOutcomeLikelihoodsByHypothesis.{}", hypothesis.id),
        );
        if outcomes.len() != expected_pattern_count {
            fail(format!(
                "joint model for {} must contain all {expected_pattern_count} outcome patterns",
                hypothesis.id
            ));
        }
        let mut distribution = BTreeMap::new();
        let mut sum = 0.0;
        for (index, outcome) in outcomes.iter().enumerate() {
            let path = format!("jointOutcomeLikelihoodsByHypothesis.{}[{index}]", hypothesis.id);
            let pattern = as_string(required(outcome, "outcomes", &path), &format!("{path}.outcomes"));
            if pattern.len() != surveys.len()
                || !pattern.bytes().all(|byte| byte == b'0' || byte == b'1')
            {
                fail(format!("{path}.outcomes must be a binary string of length {}", surveys.len()));
            }
            let probability = as_number(
                required(outcome, "probability", &path),
                &format!("{path}.probability"),
            );
            if !(0.0..=1.0).contains(&probability) {
                fail(format!("{path}.probability must be within [0, 1]"));
            }
            if distribution.insert(pattern.clone(), probability).is_some() {
                fail(format!("duplicate joint outcome pattern {pattern} for {}", hypothesis.id));
            }
            sum += probability;
        }
        let expected_patterns: Vec<String> = (0..expected_pattern_count)
            .map(|mask| {
                (0..surveys.len())
                    .map(|bit| if mask & (1 << bit) == 0 { '0' } else { '1' })
                    .collect()
            })
            .collect();
        for pattern in expected_patterns {
            if !distribution.contains_key(&pattern) {
                fail(format!("joint model for {} is missing pattern {pattern}", hypothesis.id));
            }
        }
        if (sum - 1.0).abs() > EPSILON {
            fail(format!(
                "joint outcome probabilities for {} must sum to 1.0, observed {sum}",
                hypothesis.id
            ));
        }
        joint_outcomes.insert(hypothesis.id.clone(), distribution);
    }

    Problem {
        hypotheses,
        actions,
        surveys,
        max_effort_units,
        joint_outcomes,
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
    best.unwrap_or_else(|| fail("no action available"))
}

fn conditional_outcome_probability(
    problem: &Problem,
    hypothesis: &Hypothesis,
    selected_indices: &[usize],
    selected_outcome_mask: usize,
    assume_conditional_independence: bool,
) -> f64 {
    if assume_conditional_independence {
        return selected_indices
            .iter()
            .enumerate()
            .map(|(bit, survey_index)| {
                let survey = &problem.surveys[*survey_index];
                let positive = selected_outcome_mask & (1usize << bit) != 0;
                let probability = survey.positive_likelihood[&hypothesis.id];
                if positive { probability } else { 1.0 - probability }
            })
            .product();
    }

    let distribution = &problem.joint_outcomes[&hypothesis.id];
    distribution
        .iter()
        .filter(|(pattern, _)| {
            selected_indices.iter().enumerate().all(|(bit, survey_index)| {
                let observed = selected_outcome_mask & (1usize << bit) != 0;
                pattern.as_bytes()[*survey_index] == if observed { b'1' } else { b'0' }
            })
        })
        .map(|(_, probability)| *probability)
        .sum()
}

fn evaluate_portfolio(
    problem: &Problem,
    selected_indices: &[usize],
    baseline: f64,
    assume_conditional_independence: bool,
) -> Score {
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
        let joint_probabilities: Vec<f64> = problem
            .hypotheses
            .iter()
            .map(|hypothesis| {
                hypothesis.prior
                    * conditional_outcome_probability(
                        problem,
                        hypothesis,
                        selected_indices,
                        outcome_mask,
                        assume_conditional_independence,
                    )
            })
            .collect();
        let outcome_probability = joint_probabilities.iter().sum::<f64>();
        if outcome_probability <= EPSILON {
            continue;
        }
        let posterior: Vec<f64> = joint_probabilities
            .iter()
            .map(|probability| probability / outcome_probability)
            .collect();
        let (_, best_utility) = best_action_value(problem, &posterior);
        expected_post_survey_utility += outcome_probability * best_utility;
    }

    let gross_evsi = expected_post_survey_utility - baseline;
    Score {
        selected_indices: selected_indices.to_vec(),
        expected_post_survey_utility,
        effort_units,
        utility_cost,
        gross_evsi,
        net_evsi: gross_evsi - utility_cost,
    }
}

fn event_likelihood(
    problem: &Problem,
    hypothesis: &Hypothesis,
    observed: &[Option<bool>],
    candidate: Option<(usize, bool)>,
) -> f64 {
    problem.joint_outcomes[&hypothesis.id]
        .iter()
        .filter(|(pattern, _)| {
            observed.iter().enumerate().all(|(index, required)| {
                required.is_none_or(|value| {
                    pattern.as_bytes()[index] == if value { b'1' } else { b'0' }
                })
            }) && candidate.is_none_or(|(index, value)| {
                pattern.as_bytes()[index] == if value { b'1' } else { b'0' }
            })
        })
        .map(|(_, probability)| *probability)
        .sum()
}

fn solve_sequential_policy(
    problem: &Problem,
    observed: &mut [Option<bool>],
    joint_mass: &[f64],
    remaining_effort: usize,
) -> PolicyNode {
    let history_probability = joint_mass.iter().sum::<f64>();
    if history_probability <= EPSILON {
        fail("sequential policy reached a zero-probability history");
    }
    let posterior: Vec<f64> = joint_mass
        .iter()
        .map(|mass| mass / history_probability)
        .collect();
    let (stop_action, stop_utility) = best_action_value(problem, &posterior);
    let mut best = PolicyNode {
        value: stop_utility,
        stop_action: Some(stop_action),
        survey_index: None,
        negative: None,
        positive: None,
    };

    for survey_index in 0..problem.surveys.len() {
        let survey = &problem.surveys[survey_index];
        if observed[survey_index].is_some() || survey.effort_units > remaining_effort {
            continue;
        }

        let mut branch_values = [0.0_f64; 2];
        let mut children: [Option<Box<PolicyNode>>; 2] = [None, None];
        for (outcome_index, positive) in [false, true].into_iter().enumerate() {
            let child_mass: Vec<f64> = problem
                .hypotheses
                .iter()
                .enumerate()
                .map(|(hypothesis_index, hypothesis)| {
                    let history_given_h = event_likelihood(problem, hypothesis, observed, None);
                    if history_given_h <= EPSILON {
                        0.0
                    } else {
                        let child_given_h =
                            event_likelihood(problem, hypothesis, observed, Some((survey_index, positive)));
                        joint_mass[hypothesis_index] * (child_given_h / history_given_h)
                    }
                })
                .collect();
            let outcome_probability = child_mass.iter().sum::<f64>() / history_probability;
            if outcome_probability <= EPSILON {
                continue;
            }

            observed[survey_index] = Some(positive);
            let child = solve_sequential_policy(
                problem,
                observed,
                &child_mass,
                remaining_effort - survey.effort_units,
            );
            observed[survey_index] = None;
            branch_values[outcome_index] = outcome_probability * child.value;
            children[outcome_index] = Some(Box::new(child));
        }

        let candidate_value = branch_values.iter().sum::<f64>() - survey.utility_cost;
        if candidate_value > best.value + EPSILON {
            best = PolicyNode {
                value: candidate_value,
                stop_action: None,
                survey_index: Some(survey_index),
                negative: children[0].take(),
                positive: children[1].take(),
            };
        }
    }
    best
}

fn policy_to_json(node: &PolicyNode, problem: &Problem) -> Value {
    match node.survey_index {
        Some(index) => json!({
            "nodeType": "survey",
            "surveyId": problem.surveys[index].id,
            "expectedNetUtilityFromNode": node.value,
            "outcomes": {
                "negative": node.negative.as_ref().map(|child| policy_to_json(child, problem)),
                "positive": node.positive.as_ref().map(|child| policy_to_json(child, problem))
            }
        }),
        None => json!({
            "nodeType": "decision",
            "action": node.stop_action,
            "expectedUtility": node.value
        }),
    }
}

fn calculate(case: &Value) -> Value {
    let problem = parse_problem(case);
    let prior: Vec<f64> = problem.hypotheses.iter().map(|h| h.prior).collect();
    let (baseline_action, baseline_utility) = best_action_value(&problem, &prior);
    let mut best = evaluate_portfolio(&problem, &[], baseline_utility, false);

    for mask in 1..(1usize << problem.surveys.len()) {
        let selected: Vec<usize> = (0..problem.surveys.len())
            .filter(|index| mask & (1usize << index) != 0)
            .collect();
        let candidate = evaluate_portfolio(&problem, &selected, baseline_utility, false);
        if candidate.effort_units > problem.max_effort_units {
            continue;
        }
        if candidate.net_evsi > best.net_evsi + EPSILON
            || ((candidate.net_evsi - best.net_evsi).abs() <= EPSILON
                && candidate.selected_indices.len() < best.selected_indices.len())
        {
            best = candidate;
        }
    }

    let pair_indices: Vec<usize> = if problem.surveys.len() >= 2 {
        vec![0, 1]
    } else {
        vec![0]
    };
    let exact_joint_pair = evaluate_portfolio(&problem, &pair_indices, baseline_utility, false);
    let independent_pair = evaluate_portfolio(&problem, &pair_indices, baseline_utility, true);
    let selected_ids: Vec<String> = best
        .selected_indices
        .iter()
        .map(|index| problem.surveys[*index].id.clone())
        .collect();

    if problem.surveys.len() > 8 {
        fail("sequential exact policy evaluator is limited to 8 survey candidates");
    }
    let mut observed = vec![None; problem.surveys.len()];
    let root_mass: Vec<f64> = problem.hypotheses.iter().map(|h| h.prior).collect();
    let sequential_policy = solve_sequential_policy(
        &problem,
        &mut observed,
        &root_mass,
        problem.max_effort_units,
    );

    json!({
        "benchmarkId": "sol-atlas-correlated-spatial-observations",
        "version": "0.1.0",
        "fixtureStatus": "synthetic",
        "baseline": {
            "bestAction": baseline_action,
            "expectedUtility": baseline_utility
        },
        "selectedSurveyIds": selected_ids,
        "sequentialPolicy": {
            "expectedNetUtility": sequential_policy.value,
            "tree": policy_to_json(&sequential_policy, &problem)
        },
        "selectedPortfolio": {
            "effortUnits": best.effort_units,
            "utilityCost": best.utility_cost,
            "expectedPostSurveyUtility": best.expected_post_survey_utility,
            "grossEvsi": best.gross_evsi,
            "netEvsi": best.net_evsi
        },
        "twoSurveyCorrelationAudit": {
            "surveyIds": pair_indices.iter().map(|i| problem.surveys[*i].id.clone()).collect::<Vec<_>>(),
            "jointModel": {
                "expectedPostSurveyUtility": exact_joint_pair.expected_post_survey_utility,
                "grossEvsi": exact_joint_pair.gross_evsi,
                "netEvsi": exact_joint_pair.net_evsi
            },
            "conditionalIndependenceAssumption": {
                "expectedPostSurveyUtility": independent_pair.expected_post_survey_utility,
                "grossEvsi": independent_pair.gross_evsi,
                "netEvsi": independent_pair.net_evsi
            },
            "grossEvsiOverstatement": independent_pair.gross_evsi - exact_joint_pair.gross_evsi
        }
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
        .unwrap_or("fixtures/correlated-case.json");
    if args.len() > 2 {
        eprintln!("usage: sol-atlas-correlated-observations [correlated-case.json]");
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
        serde_json::from_str(include_str!("../../fixtures/correlated-case.json")).unwrap()
    }

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() <= 1e-8, "expected {right}, got {left}");
    }

    #[test]
    fn correlated_observations_are_not_double_counted() {
        let result = calculate(&fixture());
        close(result["baseline"]["expectedUtility"].as_f64().unwrap(), 60.0);
        close(
            result["twoSurveyCorrelationAudit"]["jointModel"]["grossEvsi"]
                .as_f64()
                .unwrap(),
            20.0,
        );
        close(
            result["twoSurveyCorrelationAudit"]["conditionalIndependenceAssumption"]["grossEvsi"]
                .as_f64()
                .unwrap(),
            23.2,
        );
        close(
            result["twoSurveyCorrelationAudit"]["grossEvsiOverstatement"]
                .as_f64()
                .unwrap(),
            3.2,
        );
        assert_eq!(result["selectedSurveyIds"], json!(["aerial_texture_scan"]));
        close(
            result["selectedPortfolio"]["netEvsi"].as_f64().unwrap(),
            19.9,
        );
    }

    #[test]
    fn adaptive_policy_stops_after_a_correlated_repeat_has_no_marginal_value() {
        let problem = parse_problem(&fixture());
        let mut observed = vec![None; problem.surveys.len()];
        let root_mass: Vec<f64> = problem.hypotheses.iter().map(|h| h.prior).collect();
        let policy = solve_sequential_policy(
            &problem,
            &mut observed,
            &root_mass,
            problem.max_effort_units,
        );
        assert_eq!(
            policy.survey_index.map(|index| problem.surveys[index].id.as_str()),
            Some("aerial_texture_scan")
        );
        close(policy.value, 79.9);
        let negative = policy.negative.as_ref().unwrap();
        let positive = policy.positive.as_ref().unwrap();
        assert_eq!(negative.survey_index, None);
        assert_eq!(positive.survey_index, None);
        assert!(negative.stop_action.is_some());
        assert!(positive.stop_action.is_some());
    }

    #[test]
    fn candidate_order_does_not_change_semantics() {
        let mut case = fixture();
        case["decisionProblem"]["surveys"]
            .as_array_mut()
            .unwrap()
            .reverse();
        for hypothesis in ["original_fabric", "later_repair"] {
            case["decisionProblem"]["jointOutcomeLikelihoodsByHypothesis"][hypothesis]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .for_each(|item| {
                    let pattern = item["outcomes"].as_str().unwrap();
                    let reversed: String = pattern.chars().rev().collect();
                    item["outcomes"] = json!(reversed);
                });
        }
        assert_eq!(calculate(&fixture()), calculate(&case));
    }

    #[test]
    #[should_panic(expected = "must sum to 1.0")]
    fn rejects_non_normalized_joint_distribution() {
        let mut case = fixture();
        case["decisionProblem"]["jointOutcomeLikelihoodsByHypothesis"]["original_fabric"][0]
            ["probability"] = json!(0.5);
        let _ = calculate(&case);
    }

    #[test]
    #[should_panic(expected = "must contain all 4 outcome patterns")]
    fn rejects_incomplete_joint_distribution() {
        let mut case = fixture();
        case["decisionProblem"]["jointOutcomeLikelihoodsByHypothesis"]["original_fabric"]
            .as_array_mut()
            .unwrap()
            .pop();
        let _ = calculate(&case);
    }

    #[test]
    #[should_panic(expected = "binary string of length 2")]
    fn rejects_malformed_joint_outcome_pattern() {
        let mut case = fixture();
        case["decisionProblem"]["jointOutcomeLikelihoodsByHypothesis"]["original_fabric"][0]
            ["outcomes"] = json!("0x");
        let _ = calculate(&case);
    }
}
