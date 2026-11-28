use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, process};

const EPSILON: f64 = 1e-10;

#[derive(Clone, Debug)]
struct Survey {
    effort_units: usize,
    cost: f64,
    positive_likelihood: BTreeMap<String, f64>,
}

#[derive(Clone, Debug)]
struct Scenario {
    id: String,
    weight: f64,
    priors: BTreeMap<String, f64>,
    action_utilities: BTreeMap<String, BTreeMap<String, f64>>,
    surveys: BTreeMap<String, Survey>,
}

#[derive(Clone, Debug)]
struct Problem {
    hypotheses: Vec<String>,
    survey_ids: Vec<String>,
    max_effort_units: usize,
    tv_radii: Vec<f64>,
    cvar_alphas: Vec<f64>,
    scenarios: Vec<Scenario>,
}

#[derive(Clone, Debug)]
struct ScenarioScore {
    baseline_utility: f64,
    post_observation_utility: f64,
    gross_evsi: f64,
    acquisition_cost: f64,
    net_evsi: f64,
}

#[derive(Clone, Debug)]
struct PortfolioRecord {
    selected_survey_ids: Vec<String>,
    effort_units: usize,
    scenario_scores: Vec<ScenarioScore>,
    weighted_mean_net_evsi: f64,
    worst_case_net_evsi: f64,
    maximum_regret: f64,
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

fn string_set(value: &Value, path: &str, minimum: usize, maximum: usize) -> Vec<String> {
    let mut ids = Vec::new();
    let mut unique = BTreeSet::new();
    for (index, item) in as_array(value, path).iter().enumerate() {
        let id = as_string(item, &format!("{path}[{index}]"));
        if id.trim().is_empty() || !unique.insert(id.clone()) {
            fail(format!("{path} contains a blank or duplicate identifier"));
        }
        ids.push(id);
    }
    if !(minimum..=maximum).contains(&ids.len()) {
        fail(format!("{path} must contain between {minimum} and {maximum} items"));
    }
    ids.sort();
    ids
}

fn parse_problem(case: &Value) -> Problem {
    if as_string(required(case, "fixtureStatus", "case"), "case.fixtureStatus") != "synthetic" {
        fail("reference evaluator only accepts synthetic fixtures");
    }
    let decision = required(case, "decisionProblem", "case");
    let hypotheses = string_set(
        required(decision, "hypotheses", "decisionProblem"),
        "hypotheses",
        2,
        8,
    );
    let action_ids = string_set(
        required(decision, "actions", "decisionProblem"),
        "actions",
        2,
        8,
    );
    let survey_ids = string_set(
        required(decision, "candidateSurveyIds", "decisionProblem"),
        "candidateSurveyIds",
        1,
        8,
    );
    let max_effort_units = required(decision, "maxEffortUnits", "decisionProblem")
        .as_u64()
        .unwrap_or_else(|| fail("maxEffortUnits must be a non-negative integer")) as usize;

    let mut tv_radii = Vec::new();
    let mut tv_radius_values = BTreeSet::new();
    for (index, value) in as_array(
        required(decision, "totalVariationRadii", "decisionProblem"),
        "decisionProblem.totalVariationRadii",
    )
    .iter()
    .enumerate()
    {
        let radius = as_number(value, &format!("totalVariationRadii[{index}]"));
        if !(0.0..=1.0).contains(&radius) {
            fail("total-variation ambiguity radii must be within [0, 1]");
        }
        if !tv_radius_values.insert(radius.to_bits()) {
            fail("total-variation ambiguity radii must be unique");
        }
        tv_radii.push(radius);
    }
    if tv_radii.is_empty() {
        fail("at least one total-variation ambiguity radius is required");
    }
    tv_radii.sort_by(f64::total_cmp);

    let mut cvar_alphas = Vec::new();
    let mut cvar_alpha_values = BTreeSet::new();
    for (index, value) in as_array(
        required(decision, "cvarAlphas", "decisionProblem"),
        "decisionProblem.cvarAlphas",
    )
    .iter()
    .enumerate()
    {
        let alpha = as_number(value, &format!("cvarAlphas[{index}]"));
        if alpha <= 0.0 || alpha > 1.0 {
            fail("lower-tail CVaR alphas must be within (0, 1]");
        }
        if !cvar_alpha_values.insert(alpha.to_bits()) {
            fail("lower-tail CVaR alphas must be unique");
        }
        cvar_alphas.push(alpha);
    }
    if cvar_alphas.is_empty() {
        fail("at least one lower-tail CVaR alpha is required");
    }
    cvar_alphas.sort_by(f64::total_cmp);

    let scenario_values = as_array(
        required(decision, "scenarios", "decisionProblem"),
        "decisionProblem.scenarios",
    );
    if scenario_values.len() < 2 || scenario_values.len() > 32 {
        fail("scenario set must contain between 2 and 32 cases");
    }
    let mut scenarios = Vec::new();
    let mut scenario_ids = BTreeSet::new();
    let mut weight_sum = 0.0;
    let mut effort_by_survey = BTreeMap::<String, usize>::new();

    for (index, item) in scenario_values.iter().enumerate() {
        let path = format!("scenarios[{index}]");
        let id = as_string(required(item, "scenarioId", &path), &format!("{path}.scenarioId"));
        if !scenario_ids.insert(id.clone()) {
            fail(format!("duplicate scenario ID: {id}"));
        }
        let weight = as_number(required(item, "weight", &path), &format!("{path}.weight"));
        if !(0.0..=1.0).contains(&weight) {
            fail(format!("scenario {id} weight must be within [0, 1]"));
        }
        weight_sum += weight;

        let prior_values = required(item, "hypothesisPriors", &path)
            .as_object()
            .unwrap_or_else(|| fail(format!("{path}.hypothesisPriors must be an object")));
        if prior_values.len() != hypotheses.len() {
            fail(format!("scenario {id} must define one prior for every hypothesis and no extras"));
        }
        let mut priors = BTreeMap::new();
        let mut prior_sum = 0.0;
        for hypothesis in &hypotheses {
            let prior = as_number(
                required(
                    required(item, "hypothesisPriors", &path),
                    hypothesis,
                    &format!("{path}.hypothesisPriors"),
                ),
                &format!("{path}.hypothesisPriors.{hypothesis}"),
            );
            if !(0.0..=1.0).contains(&prior) {
                fail(format!("scenario {id} prior for {hypothesis} must be within [0, 1]"));
            }
            prior_sum += prior;
            priors.insert(hypothesis.clone(), prior);
        }
        if (prior_sum - 1.0).abs() > EPSILON {
            fail(format!("scenario {id} hypothesis priors must sum to 1.0"));
        }

        let utility_object = required(item, "actionUtilities", &path)
            .as_object()
            .unwrap_or_else(|| fail(format!("{path}.actionUtilities must be an object")));
        if utility_object.len() != action_ids.len() {
            fail(format!("scenario {id} must define utilities for every action and no extras"));
        }
        let mut action_utilities = BTreeMap::new();
        for action_id in &action_ids {
            let utility_values = required(
                required(item, "actionUtilities", &path),
                action_id,
                &format!("{path}.actionUtilities"),
            );
            let mut utilities = BTreeMap::new();
            for hypothesis in &hypotheses {
                utilities.insert(
                    hypothesis.clone(),
                    as_number(
                        required(utility_values, hypothesis, &format!("{path}.actionUtilities.{action_id}")),
                        &format!("{path}.actionUtilities.{action_id}.{hypothesis}"),
                    ),
                );
            }
            if utility_values.as_object().map(|object| object.len()) != Some(hypotheses.len()) {
                fail(format!("action {action_id} in scenario {id} has missing or extra hypothesis utilities"));
            }
            action_utilities.insert(action_id.clone(), utilities);
        }

        let survey_values = as_array(
            required(item, "surveys", &path),
            &format!("{path}.surveys"),
        );
        if survey_values.len() != survey_ids.len() {
            fail(format!("scenario {id} must define every candidate survey exactly once"));
        }
        let mut surveys = BTreeMap::new();
        for (survey_index, survey_value) in survey_values.iter().enumerate() {
            let survey_path = format!("{path}.surveys[{survey_index}]");
            let survey_id = as_string(
                required(survey_value, "surveyId", &survey_path),
                &format!("{survey_path}.surveyId"),
            );
            if !survey_ids.contains(&survey_id) {
                fail(format!("scenario {id} contains unknown survey {survey_id}"));
            }
            let effort_units = required(survey_value, "effortUnits", &survey_path)
                .as_u64()
                .unwrap_or_else(|| fail(format!("{survey_path}.effortUnits must be a non-negative integer")))
                as usize;
            let cost = as_number(
                required(survey_value, "acquisitionCostUtility", &survey_path),
                &format!("{survey_path}.acquisitionCostUtility"),
            );
            if cost < 0.0 {
                fail(format!("{survey_path}.acquisitionCostUtility cannot be negative"));
            }
            if let Some(previous) = effort_by_survey.insert(survey_id.clone(), effort_units) {
                if previous != effort_units {
                    fail(format!("survey {survey_id} effort units must be invariant across scenarios"));
                }
            }
            let likelihood_values = required(
                survey_value,
                "positiveLikelihoodByHypothesis",
                &survey_path,
            );
            let mut positive_likelihood = BTreeMap::new();
            for hypothesis in &hypotheses {
                let probability = as_number(
                    required(likelihood_values, hypothesis, &format!("{survey_path}.positiveLikelihoodByHypothesis")),
                    &format!("{survey_path}.positiveLikelihoodByHypothesis.{hypothesis}"),
                );
                if !(0.0..=1.0).contains(&probability) {
                    fail(format!("{survey_path} likelihood for {hypothesis} must be within [0, 1]"));
                }
                positive_likelihood.insert(hypothesis.clone(), probability);
            }
            if likelihood_values.as_object().map(|object| object.len()) != Some(hypotheses.len()) {
                fail(format!("{survey_path} has missing or extra hypothesis likelihoods"));
            }
            if surveys.insert(survey_id.clone(), Survey {
                effort_units,
                cost,
                positive_likelihood,
            }).is_some() {
                fail(format!("scenario {id} duplicates survey {survey_id}"));
            }
        }

        scenarios.push(Scenario {
            id,
            weight,
            priors,
            action_utilities,
            surveys,
        });
    }

    if (weight_sum - 1.0).abs() > EPSILON {
        fail(format!("scenario weights must sum to 1.0, observed {weight_sum}"));
    }
    if effort_by_survey.len() != survey_ids.len() {
        fail("one or more survey candidates are missing from the scenario inventory");
    }
    scenarios.sort_by(|a, b| a.id.cmp(&b.id));

    Problem {
        hypotheses,
        survey_ids,
        max_effort_units,
        tv_radii,
        cvar_alphas,
        scenarios,
    }
}

fn best_action_value(scenario: &Scenario, hypotheses: &[String], probabilities: &[f64]) -> (String, f64) {
    let mut best: Option<(String, f64)> = None;
    for action_id in scenario.action_utilities.keys() {
        let value = hypotheses
            .iter()
            .zip(probabilities.iter())
            .map(|(hypothesis, probability)| {
                probability * scenario.action_utilities[action_id][hypothesis]
            })
            .sum::<f64>();
        if best.as_ref().is_none_or(|(_, current)| value > *current) {
            best = Some((action_id.clone(), value));
        }
    }
    best.unwrap_or_else(|| fail("scenario has no decision actions"))
}

fn score_scenario_portfolio(
    scenario: &Scenario,
    problem: &Problem,
    selected_survey_ids: &[String],
) -> ScenarioScore {
    let prior: Vec<f64> = problem
        .hypotheses
        .iter()
        .map(|hypothesis| scenario.priors[hypothesis])
        .collect();
    let (_, baseline_utility) = best_action_value(scenario, &problem.hypotheses, &prior);
    let outcome_count = 1usize << selected_survey_ids.len();
    let mut expected_post_utility = 0.0;

    for outcome_mask in 0..outcome_count {
        let joint_mass: Vec<f64> = problem
            .hypotheses
            .iter()
            .map(|hypothesis| {
                let mut probability = scenario.priors[hypothesis];
                for (bit, survey_id) in selected_survey_ids.iter().enumerate() {
                    let survey = &scenario.surveys[survey_id];
                    let positive = outcome_mask & (1usize << bit) != 0;
                    let likelihood = survey.positive_likelihood[hypothesis];
                    probability *= if positive { likelihood } else { 1.0 - likelihood };
                }
                probability
            })
            .collect();

        let outcome_probability = joint_mass.iter().sum::<f64>();
        if outcome_probability <= EPSILON {
            continue;
        }
        let posterior: Vec<f64> = joint_mass
            .iter()
            .map(|mass| mass / outcome_probability)
            .collect();
        let (_, conditional_utility) =
            best_action_value(scenario, &problem.hypotheses, &posterior);
        expected_post_utility += outcome_probability * conditional_utility;
    }

    let gross_evsi = expected_post_utility - baseline_utility;
    let acquisition_cost = selected_survey_ids
        .iter()
        .map(|survey_id| scenario.surveys[survey_id].cost)
        .sum::<f64>();
    ScenarioScore {
        baseline_utility,
        post_observation_utility: expected_post_utility,
        gross_evsi,
        acquisition_cost,
        net_evsi: gross_evsi - acquisition_cost,
    }
}

fn portfolio_tie_is_better(candidate: &PortfolioRecord, incumbent: &PortfolioRecord, metric: &str) -> bool {
    let candidate_value = match metric {
        "mean" => candidate.weighted_mean_net_evsi,
        "worst" => candidate.worst_case_net_evsi,
        "regret" => -candidate.maximum_regret,
        _ => fail("unknown portfolio comparison metric"),
    };
    let incumbent_value = match metric {
        "mean" => incumbent.weighted_mean_net_evsi,
        "worst" => incumbent.worst_case_net_evsi,
        "regret" => -incumbent.maximum_regret,
        _ => fail("unknown portfolio comparison metric"),
    };
    if candidate_value > incumbent_value + EPSILON {
        return true;
    }
    if (candidate_value - incumbent_value).abs() <= EPSILON {
        return candidate.selected_survey_ids.len() < incumbent.selected_survey_ids.len()
            || (candidate.selected_survey_ids.len() == incumbent.selected_survey_ids.len()
                && candidate.selected_survey_ids < incumbent.selected_survey_ids);
    }
    false
}

fn choose_portfolio<'a>(records: &'a [PortfolioRecord], metric: &str) -> &'a PortfolioRecord {
    let mut best = records
        .first()
        .unwrap_or_else(|| fail("no feasible portfolios were generated"));
    for candidate in records.iter().skip(1) {
        if portfolio_tie_is_better(candidate, best, metric) {
            best = candidate;
        }
    }
    best
}

fn portfolio_json(record: &PortfolioRecord, problem: &Problem) -> Value {
    let scenario_rows: Vec<Value> = problem
        .scenarios
        .iter()
        .zip(record.scenario_scores.iter())
        .map(|(scenario, score)| json!({
            "scenarioId": scenario.id,
            "weight": scenario.weight,
            "baselineUtility": score.baseline_utility,
            "postObservationUtility": score.post_observation_utility,
            "grossEvsi": score.gross_evsi,
            "acquisitionCost": score.acquisition_cost,
            "netEvsi": score.net_evsi
        }))
        .collect();
    json!({
        "selectedSurveyIds": record.selected_survey_ids,
        "effortUnits": record.effort_units,
        "weightedMeanNetEvsi": record.weighted_mean_net_evsi,
        "worstCaseNetEvsi": record.worst_case_net_evsi,
        "maximumRegret": record.maximum_regret,
        "scenarioScores": scenario_rows
    })
}

fn worst_case_tv_weights(
    problem: &Problem,
    record: &PortfolioRecord,
    radius: f64,
) -> (f64, Vec<f64>) {
    let mut weights: Vec<f64> = problem.scenarios.iter().map(|scenario| scenario.weight).collect();
    let values: Vec<f64> = record
        .scenario_scores
        .iter()
        .map(|score| score.net_evsi)
        .collect();
    let recipient = (0..values.len())
        .min_by(|a, b| values[*a].total_cmp(&values[*b]).then_with(|| a.cmp(b)))
        .unwrap_or_else(|| fail("no scenarios available for TV ambiguity evaluation"));
    let mut donors: Vec<usize> = (0..values.len())
        .filter(|index| *index != recipient && values[*index] > values[recipient] + EPSILON)
        .collect();
    donors.sort_by(|a, b| values[*b].total_cmp(&values[*a]).then_with(|| a.cmp(b)));

    let mut remaining = radius;
    for donor in donors {
        if remaining <= EPSILON {
            break;
        }
        // TV distance is the amount of probability mass moved, i.e. 1/2 * L1.
        let transferred = weights[donor].min(remaining);
        weights[donor] -= transferred;
        weights[recipient] += transferred;
        remaining -= transferred;
    }

    let worst_case_value = weights
        .iter()
        .zip(values.iter())
        .map(|(weight, value)| weight * value)
        .sum::<f64>();
    (worst_case_value, weights)
}

fn choose_tv_portfolio<'a>(
    problem: &Problem,
    records: &'a [PortfolioRecord],
    radius: f64,
) -> (&'a PortfolioRecord, f64, Vec<f64>) {
    let mut best_record = records
        .first()
        .unwrap_or_else(|| fail("no portfolio records for total-variation optimization"));
    let (mut best_value, mut best_weights) =
        worst_case_tv_weights(problem, best_record, radius);
    for candidate in records.iter().skip(1) {
        let (candidate_value, candidate_weights) =
            worst_case_tv_weights(problem, candidate, radius);
        if candidate_value > best_value + EPSILON
            || ((candidate_value - best_value).abs() <= EPSILON
                && (candidate.selected_survey_ids.len() < best_record.selected_survey_ids.len()
                    || (candidate.selected_survey_ids.len() == best_record.selected_survey_ids.len()
                        && candidate.selected_survey_ids < best_record.selected_survey_ids)))
        {
            best_record = candidate;
            best_value = candidate_value;
            best_weights = candidate_weights;
        }
    }
    (best_record, best_value, best_weights)
}

fn lower_tail_cvar_weights(
    problem: &Problem,
    record: &PortfolioRecord,
    alpha: f64,
) -> (f64, Vec<f64>) {
    let mut order: Vec<usize> = (0..problem.scenarios.len()).collect();
    order.sort_by(|a, b| {
        record.scenario_scores[*a]
            .net_evsi
            .total_cmp(&record.scenario_scores[*b].net_evsi)
            .then_with(|| problem.scenarios[*a].id.cmp(&problem.scenarios[*b].id))
    });

    let mut remaining = alpha;
    let mut tail_weights = vec![0.0; problem.scenarios.len()];
    let mut weighted_tail_value = 0.0;
    for index in order {
        if remaining <= EPSILON {
            break;
        }
        let mass = problem.scenarios[index].weight.min(remaining);
        tail_weights[index] = mass / alpha;
        weighted_tail_value += mass * record.scenario_scores[index].net_evsi;
        remaining -= mass;
    }
    if remaining > EPSILON {
        fail("scenario probability mass is insufficient for requested CVaR tail");
    }
    (weighted_tail_value / alpha, tail_weights)
}

fn choose_cvar_portfolio<'a>(
    problem: &Problem,
    records: &'a [PortfolioRecord],
    alpha: f64,
) -> (&'a PortfolioRecord, f64, Vec<f64>) {
    let mut best_record = records
        .first()
        .unwrap_or_else(|| fail("no portfolio records for CVaR optimization"));
    let (mut best_value, mut best_weights) =
        lower_tail_cvar_weights(problem, best_record, alpha);
    for candidate in records.iter().skip(1) {
        let (candidate_value, candidate_weights) =
            lower_tail_cvar_weights(problem, candidate, alpha);
        if candidate_value > best_value + EPSILON
            || ((candidate_value - best_value).abs() <= EPSILON
                && (candidate.selected_survey_ids.len() < best_record.selected_survey_ids.len()
                    || (candidate.selected_survey_ids.len() == best_record.selected_survey_ids.len()
                        && candidate.selected_survey_ids < best_record.selected_survey_ids)))
        {
            best_record = candidate;
            best_value = candidate_value;
            best_weights = candidate_weights;
        }
    }
    (best_record, best_value, best_weights)
}

fn calculate(case: &Value) -> Value {
    let problem = parse_problem(case);
    let survey_count = problem.survey_ids.len();
    let mut records = Vec::new();

    // Include the option to make no new observation.
    for mask in 0..(1usize << survey_count) {
        let selected: Vec<String> = (0..survey_count)
            .filter(|index| mask & (1usize << index) != 0)
            .map(|index| problem.survey_ids[index].clone())
            .collect();
        let effort_units = selected
            .iter()
            .map(|survey_id| {
                problem.scenarios[0].surveys[survey_id].effort_units
            })
            .sum::<usize>();
        if effort_units > problem.max_effort_units {
            continue;
        }
        let scenario_scores: Vec<ScenarioScore> = problem
            .scenarios
            .iter()
            .map(|scenario| score_scenario_portfolio(scenario, &problem, &selected))
            .collect();
        let weighted_mean_net_evsi = problem
            .scenarios
            .iter()
            .zip(scenario_scores.iter())
            .map(|(scenario, score)| scenario.weight * score.net_evsi)
            .sum::<f64>();
        let worst_case_net_evsi = scenario_scores
            .iter()
            .map(|score| score.net_evsi)
            .fold(f64::INFINITY, f64::min);
        records.push(PortfolioRecord {
            selected_survey_ids: selected,
            effort_units,
            scenario_scores,
            weighted_mean_net_evsi,
            worst_case_net_evsi,
            maximum_regret: 0.0,
        });
    }

    let mut scenario_best_net = vec![f64::NEG_INFINITY; problem.scenarios.len()];
    for record in &records {
        for (index, score) in record.scenario_scores.iter().enumerate() {
            scenario_best_net[index] = scenario_best_net[index].max(score.net_evsi);
        }
    }
    for record in &mut records {
        record.maximum_regret = record
            .scenario_scores
            .iter()
            .enumerate()
            .map(|(index, score)| scenario_best_net[index] - score.net_evsi)
            .fold(0.0, f64::max);
    }

    records.sort_by(|a, b| a.selected_survey_ids.cmp(&b.selected_survey_ids));
    let nominal = choose_portfolio(&records, "mean");
    let maximin = choose_portfolio(&records, "worst");
    let minimax_regret = choose_portfolio(&records, "regret");
    let scenario_best_choices: Vec<Value> = problem
        .scenarios
        .iter()
        .enumerate()
        .map(|(index, scenario)| {
            let best = records
                .iter()
                .max_by(|a, b| {
                    a.scenario_scores[index]
                        .net_evsi
                        .total_cmp(&b.scenario_scores[index].net_evsi)
                        .then_with(|| b.selected_survey_ids.len().cmp(&a.selected_survey_ids.len()))
                        .then_with(|| b.selected_survey_ids.cmp(&a.selected_survey_ids))
                })
                .unwrap();
            json!({
                "scenarioId": scenario.id,
                "bestSurveyIds": best.selected_survey_ids,
                "bestNetEvsi": best.scenario_scores[index].net_evsi
            })
        })
        .collect();

    let tv_dro_policies: Vec<Value> = problem
        .tv_radii
        .iter()
        .map(|radius| {
            let (policy, worst_case_value, adversarial_weights) =
                choose_tv_portfolio(&problem, &records, *radius);
            let weights_by_scenario: BTreeMap<String, f64> = problem
                .scenarios
                .iter()
                .zip(adversarial_weights.iter())
                .map(|(scenario, weight)| (scenario.id.clone(), *weight))
                .collect();
            json!({
                "totalVariationRadius": radius,
                "selectedPolicy": portfolio_json(policy, &problem),
                "worstCaseExpectedNetEvsi": worst_case_value,
                "adversarialScenarioWeights": weights_by_scenario
            })
        })
        .collect();

    let cvar_policies: Vec<Value> = problem
        .cvar_alphas
        .iter()
        .map(|alpha| {
            let (policy, cvar_value, tail_weights) =
                choose_cvar_portfolio(&problem, &records, *alpha);
            let tail_weights_by_scenario: BTreeMap<String, f64> = problem
                .scenarios
                .iter()
                .zip(tail_weights.iter())
                .map(|(scenario, weight)| (scenario.id.clone(), *weight))
                .collect();
            json!({
                "alpha": alpha,
                "selectedPolicy": portfolio_json(policy, &problem),
                "lowerTailCvarNetEvsi": cvar_value,
                "normalizedTailScenarioWeights": tail_weights_by_scenario
            })
        })
        .collect();

    json!({
        "benchmarkId": "sol-atlas-robust-spatial-decision-evsi",
        "version": "0.1.0",
        "fixtureStatus": "synthetic",
        "scenarioCount": problem.scenarios.len(),
        "candidateSurveyIds": problem.survey_ids,
        "nominalExpectedNetEvsiPolicy": portfolio_json(nominal, &problem),
        "maximinNetEvsiPolicy": portfolio_json(maximin, &problem),
        "minimaxRegretPolicy": portfolio_json(minimax_regret, &problem),
        "totalVariationDROPolicies": tv_dro_policies,
        "lowerTailCvarPolicies": cvar_policies,
        "scenarioBestChoices": scenario_best_choices,
        "portfolioComparisons": records.iter().map(|record| portfolio_json(record, &problem)).collect::<Vec<Value>>()
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
        .unwrap_or("fixtures/robust-decision-case.json");
    if args.len() > 2 {
        eprintln!("usage: sol-atlas-robust-decision-evsi [robust-decision-case.json]");
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
        serde_json::from_str(include_str!("../../fixtures/robust-decision-case.json")).unwrap()
    }

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() <= 1e-8, "expected {right}, got {left}");
    }

    #[test]
    fn nominal_maximin_and_minimax_regret_can_choose_different_policies() {
        let result = calculate(&fixture());
        let nominal = &result["nominalExpectedNetEvsiPolicy"];
        let robust = &result["maximinNetEvsiPolicy"];
        let regret = &result["minimaxRegretPolicy"];

        assert_eq!(
            nominal["selectedSurveyIds"],
            json!(["multispectral_material_scan"])
        );
        close(
            nominal["weightedMeanNetEvsi"].as_f64().unwrap(),
            19.6,
        );
        assert_eq!(robust["selectedSurveyIds"], json!(["archive_crosscheck"]));
        close(robust["worstCaseNetEvsi"].as_f64().unwrap(), 3.0);
        assert_eq!(regret["selectedSurveyIds"], json!(["archive_crosscheck"]));
        close(regret["maximumRegret"].as_f64().unwrap(), 15.0);
        let sensor_shift_choice = result["scenarioBestChoices"]
            .as_array()
            .unwrap()
            .iter()
            .find(|choice| choice["scenarioId"] == "sensor_shift")
            .unwrap();
        assert_eq!(
            sensor_shift_choice["bestSurveyIds"],
            json!(["archive_crosscheck"])
        );
    }

    #[test]
    fn zero_radius_tv_dro_matches_nominal_and_unit_radius_matches_maximin() {
        let result = calculate(&fixture());
        let tv_policies = result["totalVariationDROPolicies"].as_array().unwrap();
        let zero_radius = tv_policies
            .iter()
            .find(|policy| policy["totalVariationRadius"] == json!(0.0))
            .unwrap();
        assert_eq!(
            zero_radius["selectedPolicy"]["selectedSurveyIds"],
            result["nominalExpectedNetEvsiPolicy"]["selectedSurveyIds"]
        );
        close(
            zero_radius["worstCaseExpectedNetEvsi"].as_f64().unwrap(),
            result["nominalExpectedNetEvsiPolicy"]["weightedMeanNetEvsi"]
                .as_f64()
                .unwrap(),
        );

        let half_radius = tv_policies
            .iter()
            .find(|policy| policy["totalVariationRadius"] == json!(0.5))
            .unwrap();
        assert_eq!(
            half_radius["selectedPolicy"]["selectedSurveyIds"],
            json!(["multispectral_material_scan"])
        );
        close(
            half_radius["worstCaseExpectedNetEvsi"].as_f64().unwrap(),
            7.6,
        );

        let shift_radius = tv_policies
            .iter()
            .find(|policy| policy["totalVariationRadius"] == json!(0.7))
            .unwrap();
        assert_eq!(
            shift_radius["selectedPolicy"]["selectedSurveyIds"],
            json!(["archive_crosscheck"])
        );
        close(
            shift_radius["worstCaseExpectedNetEvsi"].as_f64().unwrap(),
            3.5,
        );

        let unit_radius = tv_policies
            .iter()
            .find(|policy| policy["totalVariationRadius"] == json!(1.0))
            .unwrap();
        assert_eq!(
            unit_radius["selectedPolicy"]["selectedSurveyIds"],
            result["maximinNetEvsiPolicy"]["selectedSurveyIds"]
        );
        close(
            unit_radius["worstCaseExpectedNetEvsi"].as_f64().unwrap(),
            result["maximinNetEvsiPolicy"]["worstCaseNetEvsi"]
                .as_f64()
                .unwrap(),
        );
        let robust_values: Vec<f64> = tv_policies
            .iter()
            .map(|policy| policy["worstCaseExpectedNetEvsi"].as_f64().unwrap())
            .collect();
        assert!(robust_values.windows(2).all(|window| window[1] <= window[0] + EPSILON));
    }

    #[test]
    fn lower_tail_cvar_exposes_risk_preference_crossover() {
        let result = calculate(&fixture());
        let policies = result["lowerTailCvarPolicies"].as_array().unwrap();
        let ten_percent = policies
            .iter()
            .find(|policy| policy["alpha"] == json!(0.1))
            .unwrap();
        assert_eq!(
            ten_percent["selectedPolicy"]["selectedSurveyIds"],
            json!(["archive_crosscheck"])
        );
        close(
            ten_percent["lowerTailCvarNetEvsi"].as_f64().unwrap(),
            3.0,
        );

        let twenty_percent = policies
            .iter()
            .find(|policy| policy["alpha"] == json!(0.2))
            .unwrap();
        assert_eq!(
            twenty_percent["selectedPolicy"]["selectedSurveyIds"],
            json!(["multispectral_material_scan"])
        );
        close(
            twenty_percent["lowerTailCvarNetEvsi"].as_f64().unwrap(),
            8.5,
        );

        let full_distribution = policies
            .iter()
            .find(|policy| policy["alpha"] == json!(1.0))
            .unwrap();
        assert_eq!(
            full_distribution["selectedPolicy"]["selectedSurveyIds"],
            result["nominalExpectedNetEvsiPolicy"]["selectedSurveyIds"]
        );
        close(
            full_distribution["lowerTailCvarNetEvsi"].as_f64().unwrap(),
            result["nominalExpectedNetEvsiPolicy"]["weightedMeanNetEvsi"]
                .as_f64()
                .unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "within (0, 1]")]
    fn rejects_cvar_alpha_outside_open_closed_unit_interval() {
        let mut case = fixture();
        case["decisionProblem"]["cvarAlphas"][0] = json!(0.0);
        let _ = calculate(&case);
    }

    #[test]
    #[should_panic(expected = "within [0, 1]")]
    fn rejects_total_variation_radius_outside_unit_interval() {
        let mut case = fixture();
        case["decisionProblem"]["totalVariationRadii"][0] = json!(1.1);
        let _ = calculate(&case);
    }

    #[test]
    fn scenario_order_and_survey_order_do_not_change_policy() {
        let mut case = fixture();
        case["decisionProblem"]["scenarios"].as_array_mut().unwrap().reverse();
        for scenario in case["decisionProblem"]["scenarios"].as_array_mut().unwrap() {
            scenario["surveys"].as_array_mut().unwrap().reverse();
        }
        assert_eq!(calculate(&fixture()), calculate(&case));
    }

    #[test]
    #[should_panic(expected = "scenario weights must sum to 1.0")]
    fn rejects_unnormalized_scenario_weights() {
        let mut case = fixture();
        case["decisionProblem"]["scenarios"][0]["weight"] = json!(0.9);
        let _ = calculate(&case);
    }

    #[test]
    #[should_panic(expected = "must be within [0, 1]")]
    fn rejects_invalid_scenario_likelihood() {
        let mut case = fixture();
        case["decisionProblem"]["scenarios"][0]["surveys"][0]
            ["positiveLikelihoodByHypothesis"]["original_fabric"] = json!(1.1);
        let _ = calculate(&case);
    }

    #[test]
    #[should_panic(expected = "must define every candidate survey exactly once")]
    fn rejects_missing_survey_from_scenario() {
        let mut case = fixture();
        case["decisionProblem"]["scenarios"][0]["surveys"]
            .as_array_mut()
            .unwrap()
            .pop();
        let _ = calculate(&case);
    }
}
