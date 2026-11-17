// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_ROBUST_DECISION_SYNTHETIC_V0.1.json"
);

#[derive(Clone, Debug)]
struct Objective {
    unit: String,
    direction: String,
}

#[derive(Clone, Copy, Debug)]
struct Interval {
    lower: f64,
    upper: f64,
}

#[derive(Clone, Debug)]
struct Candidate {
    id: String,
    feasibility: String,
    objective_completeness: String,
    intervals: BTreeMap<String, BTreeMap<String, Interval>>,
}

fn object<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn only_keys(value: &Value, allowed: &[&str], context: &str) -> Result<(), String> {
    let map = object(value, context)?;
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{context} contains unknown field {key}"));
        }
    }
    Ok(())
}

fn required_array<'a>(
    value: &'a Value,
    key: &str,
    context: &str,
) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))
}

fn required_str(value: &Value, key: &str, context: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{context}.{key} must be a non-empty string"))
}

fn required_number(value: &Value, key: &str, context: &str) -> Result<f64, String> {
    let number = value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{context}.{key} must be a number"))?;
    if !number.is_finite() {
        return Err(format!("{context}.{key} must be finite"));
    }
    Ok(number)
}

fn require_false(value: &Value, key: &str) -> Result<(), String> {
    if value.get(key).and_then(Value::as_bool) != Some(false) {
        return Err(format!("claim_ceiling.{key} must remain false"));
    }
    Ok(())
}

fn unique_insert(set: &mut BTreeSet<String>, id: String, context: &str) -> Result<(), String> {
    if !set.insert(id.clone()) {
        return Err(format!("duplicate identity {id} in {context}"));
    }
    Ok(())
}

fn interval_dominates(
    left: &Candidate,
    right: &Candidate,
    scenario_id: &str,
    objectives: &BTreeMap<String, Objective>,
) -> bool {
    let Some(left_metrics) = left.intervals.get(scenario_id) else {
        return false;
    };
    let Some(right_metrics) = right.intervals.get(scenario_id) else {
        return false;
    };

    let mut strict = false;
    for (objective_id, objective) in objectives {
        let (Some(a), Some(b)) = (
            left_metrics.get(objective_id),
            right_metrics.get(objective_id),
        ) else {
            return false;
        };

        match objective.direction.as_str() {
            "MINIMIZE" => {
                if a.upper > b.lower {
                    return false;
                }
                strict |= a.upper < b.lower;
            }
            "MAXIMIZE" => {
                if a.lower < b.upper {
                    return false;
                }
                strict |= a.lower > b.upper;
            }
            _ => return false,
        }
    }
    strict
}

fn evaluate(root: &Value) -> Result<Value, String> {
    only_keys(
        root,
        &[
            "schema_version",
            "benchmark_id",
            "brief_id",
            "design_revision",
            "maturity",
            "execution_status",
            "scenarios",
            "hard_constraints",
            "objectives",
            "candidates",
            "claim_ceiling",
        ],
        "root",
    )?;

    if required_str(root, "schema_version", "root")? != "0.1.0"
        || required_str(root, "benchmark_id", "root")? != "HOME-FREEDOM-ROBUST-DECISION-001"
        || required_str(root, "maturity", "root")? != "SYNTHETIC"
        || required_str(root, "execution_status", "root")? != "NOT_SIMULATED"
    {
        return Err("benchmark identity or synthetic claim ceiling changed".to_owned());
    }
    required_str(root, "brief_id", "root")?;
    required_str(root, "design_revision", "root")?;

    let claims = root
        .get("claim_ceiling")
        .ok_or_else(|| "root.claim_ceiling is required".to_owned())?;
    only_keys(
        claims,
        &[
            "physical_simulation_performed",
            "measured_data_admitted",
            "professional_review_passed",
            "buildability_claimed",
            "sota_superiority_claimed",
            "scalar_score_emitted",
        ],
        "claim_ceiling",
    )?;
    for key in [
        "physical_simulation_performed",
        "measured_data_admitted",
        "professional_review_passed",
        "buildability_claimed",
        "sota_superiority_claimed",
        "scalar_score_emitted",
    ] {
        require_false(claims, key)?;
    }

    let scenario_rows = required_array(root, "scenarios", "root")?;
    if scenario_rows.len() < 2 {
        return Err("at least two mandatory scenarios are required".to_owned());
    }
    let mut scenario_ids = BTreeSet::new();
    let mut trace_ids = BTreeSet::new();
    for row in scenario_rows {
        only_keys(row, &["scenario_id", "exogenous_trace_id"], "scenario")?;
        let id = required_str(row, "scenario_id", "scenario")?;
        let trace = required_str(row, "exogenous_trace_id", "scenario")?;
        unique_insert(&mut scenario_ids, id, "scenarios")?;
        unique_insert(&mut trace_ids, trace, "scenario trace identities")?;
    }

    let constraint_rows = required_array(root, "hard_constraints", "root")?;
    if constraint_rows.is_empty() {
        return Err("at least one hard constraint is required".to_owned());
    }
    let mut constraint_ids = BTreeSet::new();
    for row in constraint_rows {
        only_keys(row, &["constraint_id"], "hard_constraint")?;
        unique_insert(
            &mut constraint_ids,
            required_str(row, "constraint_id", "hard_constraint")?,
            "hard constraints",
        )?;
    }

    let objective_rows = required_array(root, "objectives", "root")?;
    if objective_rows.len() < 2 {
        return Err("at least two objectives are required".to_owned());
    }
    let mut objectives = BTreeMap::new();
    for row in objective_rows {
        only_keys(row, &["objective_id", "unit", "direction"], "objective")?;
        let id = required_str(row, "objective_id", "objective")?;
        let unit = required_str(row, "unit", "objective")?;
        let direction = required_str(row, "direction", "objective")?;
        if direction != "MINIMIZE" && direction != "MAXIMIZE" {
            return Err(format!(
                "objective {id} has unsupported direction {direction}"
            ));
        }
        if objectives
            .insert(id.clone(), Objective { unit, direction })
            .is_some()
        {
            return Err(format!("duplicate objective {id}"));
        }
    }

    let candidate_rows = required_array(root, "candidates", "root")?;
    if candidate_rows.len() < 2 {
        return Err("at least two candidates are required".to_owned());
    }
    let mut candidates = BTreeMap::<String, Candidate>::new();

    for candidate_row in candidate_rows {
        only_keys(
                candidate_row,
                &["candidate_id", "scenario_results"],
                "candidate",
            )?;
        let candidate_id = required_str(candidate_row, "candidate_id", "candidate")?;
        let result_rows = required_array(candidate_row, "scenario_results", "candidate")?;
        if result_rows.len() != scenario_ids.len() {
            return Err(format!(
                "candidate {candidate_id} must have exactly one result per mandatory scenario"
            ));
        }

        let mut seen_scenarios = BTreeSet::new();
        let mut any_fail = false;
        let mut any_unknown_gate = false;
        let mut any_unknown_metric = false;
        let mut intervals_by_scenario = BTreeMap::new();

        for scenario_row in result_rows {
            only_keys(
                scenario_row,
                &["scenario_id", "gate_results", "metric_intervals"],
                "scenario_result",
            )?;
            let scenario_id = required_str(scenario_row, "scenario_id", "scenario_result")?;
            if !scenario_ids.contains(&scenario_id) {
                return Err(format!(
                    "candidate {candidate_id} refers to undeclared scenario {scenario_id}"
                ));
            }
            unique_insert(
                    &mut seen_scenarios,
                    scenario_id.clone(),
                    "candidate scenarios",
                )?;

            let gate_rows = required_array(scenario_row, "gate_results", "scenario_result")?;
            if gate_rows.len() != constraint_ids.len() {
                return Err(format!(
                    "candidate {candidate_id}/{scenario_id} must have exactly one result per hard constraint"
                ));
            }
            let mut seen_gates = BTreeSet::new();
            for gate in gate_rows {
                only_keys(
                    gate,
                    &["constraint_id", "state", "evidence_ref"],
                    "gate_result",
                )?;
                let constraint_id = required_str(gate, "constraint_id", "gate_result")?;
                if !constraint_ids.contains(&constraint_id) {
                    return Err(format!(
                        "candidate {candidate_id}/{scenario_id} has undeclared constraint {constraint_id}"
                    ));
                }
                unique_insert(&mut seen_gates, constraint_id.clone(), "gate results")?;
                let state = required_str(gate, "state", "gate_result")?;
                let evidence = gate
                    .get("evidence_ref")
                    .ok_or_else(|| "gate_result.evidence_ref is required".to_owned())?;
                match state.as_str() {
                    "PASS" | "FAIL" => {
                        let reference = evidence.as_str().unwrap_or_default();
                        if !reference.starts_with("synthetic:") {
                            return Err(format!(
                                "synthetic gate {candidate_id}/{scenario_id}/{constraint_id} requires a synthetic evidence reference"
                            ));
                        }
                        any_fail |= state == "FAIL";
                    }
                    "UNKNOWN" => {
                        if !evidence.is_null() {
                            return Err(format!(
                                "UNKNOWN gate {candidate_id}/{scenario_id}/{constraint_id} cannot carry evidence"
                            ));
                        }
                        any_unknown_gate = true;
                    }
                    _ => {
                        return Err(format!(
                            "candidate {candidate_id}/{scenario_id} has unsupported gate state {state}"
                        ));
                    }
                }
            }

            let metric_rows = required_array(scenario_row, "metric_intervals", "scenario_result")?;
            if metric_rows.len() != objectives.len() {
                return Err(format!(
                    "candidate {candidate_id}/{scenario_id} must have exactly one interval per objective"
                ));
            }
            let mut seen_metrics = BTreeSet::new();
            let mut scenario_metrics = BTreeMap::new();
            for metric in metric_rows {
                only_keys(
                    metric,
                    &[
                        "objective_id",
                        "state",
                        "lower",
                        "upper",
                        "unit",
                        "interval_basis",
                    ],
                    "metric_interval",
                )?;
                let objective_id = required_str(metric, "objective_id", "metric_interval")?;
                let objective = objectives.get(&objective_id).ok_or_else(|| {
                    format!(
                        "candidate {candidate_id}/{scenario_id} has undeclared objective {objective_id}"
                    )
                })?;
                unique_insert(&mut seen_metrics, objective_id.clone(), "metric intervals")?;
                let unit = required_str(metric, "unit", "metric_interval")?;
                if unit != objective.unit {
                    return Err(format!(
                        "candidate {candidate_id}/{scenario_id}/{objective_id} unit mismatch: expected {}, got {unit}",
                        objective.unit
                    ));
                }
                let state = required_str(metric, "state", "metric_interval")?;
                let basis = required_str(metric, "interval_basis", "metric_interval")?;
                if state == "UNKNOWN" {
                    if !metric.get("lower").is_some_and(Value::is_null)
                        || !metric.get("upper").is_some_and(Value::is_null)
                        || basis != "UNKNOWN"
                    {
                        return Err(format!(
                            "UNKNOWN interval {candidate_id}/{scenario_id}/{objective_id} must have null bounds and UNKNOWN basis"
                        ));
                    }
                    any_unknown_metric = true;
                } else if state == "KNOWN_INTERVAL" {
                    if basis != "SYNTHETIC_BOUND" {
                        return Err(format!(
                            "synthetic interval {candidate_id}/{scenario_id}/{objective_id} must use SYNTHETIC_BOUND"
                        ));
                    }
                    let lower = required_number(metric, "lower", "metric_interval")?;
                    let upper = required_number(metric, "upper", "metric_interval")?;
                    if lower > upper {
                        return Err(format!(
                            "inverted interval {candidate_id}/{scenario_id}/{objective_id}: {lower} > {upper}"
                        ));
                    }
                    scenario_metrics.insert(objective_id, Interval { lower, upper });
                } else {
                    return Err(format!(
                        "candidate {candidate_id}/{scenario_id}/{objective_id} has unsupported interval state {state}"
                    ));
                }
            }
            intervals_by_scenario.insert(scenario_id, scenario_metrics);
        }

        if seen_scenarios != scenario_ids {
            return Err(format!(
                "candidate {candidate_id} scenario set does not exactly match the benchmark"
            ));
        }

        let feasibility = if any_fail {
            "BLOCKED"
        } else if any_unknown_gate {
            "CONDITIONAL"
        } else {
            "ROBUST_FEASIBLE"
        };
        let objective_completeness = if any_unknown_metric {
            "OBJECTIVES_INCOMPLETE"
        } else {
            "COMPLETE"
        };

        let candidate = Candidate {
            id: candidate_id.clone(),
            feasibility: feasibility.to_owned(),
            objective_completeness: objective_completeness.to_owned(),
            intervals: intervals_by_scenario,
        };
        if candidates.insert(candidate_id.clone(), candidate).is_some() {
            return Err(format!("duplicate candidate ID {candidate_id}"));
        }
    }

    let eligible: Vec<&Candidate> = candidates
        .values()
        .filter(|candidate| {
            candidate.feasibility == "ROBUST_FEASIBLE"
                && candidate.objective_completeness == "COMPLETE"
        })
        .collect();

    let mut candidate_assessments = Vec::new();
    for candidate in candidates.values() {
        candidate_assessments.push(json!({
            "candidate_id": candidate.id,
            "feasibility_disposition": candidate.feasibility,
            "objective_completeness": candidate.objective_completeness,
            "dominance_eligible": candidate.feasibility == "ROBUST_FEASIBLE"
                && candidate.objective_completeness == "COMPLETE"
        }));
    }

    let mut scenario_frontiers = Vec::new();
    let mut dominance_pairs = Vec::new();
    let mut scenario_id_list: Vec<String> = scenario_ids.iter().cloned().collect();
    scenario_id_list.sort();

    for scenario_id in &scenario_id_list {
        let mut frontier = Vec::new();
        for candidate in &eligible {
            let dominated = eligible.iter().any(|other| {
                other.id != candidate.id
                    && interval_dominates(other, candidate, scenario_id, &objectives)
            });
            if !dominated {
                frontier.push(candidate.id.clone());
            }
            for other in &eligible {
                if candidate.id != other.id
                    && interval_dominates(candidate, other, scenario_id, &objectives)
                {
                    dominance_pairs.push(json!({
                        "scope":"SCENARIO",
                        "scenario_id":scenario_id,
                        "dominates":candidate.id,
                        "dominated":other.id
                    }));
                }
            }
        }
        frontier.sort();
        scenario_frontiers.push(json!({
            "scenario_id":scenario_id,
            "candidate_ids":frontier
        }));
    }

    let mut robust_frontier = Vec::new();
    for candidate in &eligible {
        let robustly_dominated = eligible.iter().any(|other| {
            other.id != candidate.id
                && scenario_id_list.iter().all(|scenario_id| {
                    interval_dominates(other, candidate, scenario_id, &objectives)
                })
        });
        if !robustly_dominated {
            robust_frontier.push(candidate.id.clone());
        }
        for other in &eligible {
            if candidate.id != other.id
                && scenario_id_list.iter().all(|scenario_id| {
                    interval_dominates(candidate, other, scenario_id, &objectives)
                })
            {
                dominance_pairs.push(json!({
                    "scope":"ALL_SCENARIOS",
                    "scenario_id":Value::Null,
                    "dominates":candidate.id,
                    "dominated":other.id
                }));
            }
        }
    }
    robust_frontier.sort();

    let mut regret_bounds = Vec::new();
    for scenario_id in &scenario_id_list {
        for (objective_id, objective) in &objectives {
            let values: Vec<(String, Interval)> = eligible
                .iter()
                .filter_map(|candidate| {
                    candidate
                        .intervals
                        .get(scenario_id)
                        .and_then(|metrics| metrics.get(objective_id))
                        .map(|interval| (candidate.id.clone(), *interval))
                })
                .collect();
            if values.is_empty() {
                continue;
            }
            for (candidate_id, interval) in &values {
                let bound = match objective.direction.as_str() {
                    "MINIMIZE" => {
                        let best_lower = values
                            .iter()
                            .map(|(_, value)| value.lower)
                            .fold(f64::INFINITY, f64::min);
                        (interval.upper - best_lower).max(0.0)
                    }
                    "MAXIMIZE" => {
                        let best_upper = values
                            .iter()
                            .map(|(_, value)| value.upper)
                            .fold(f64::NEG_INFINITY, f64::max);
                        (best_upper - interval.lower).max(0.0)
                    }
                    _ => unreachable!("validated objective direction"),
                };
                regret_bounds.push(json!({
                    "candidate_id":candidate_id,
                    "scenario_id":scenario_id,
                    "objective_id":objective_id,
                    "unit":objective.unit,
                    "conservative_outer_bound":bound
                }));
            }
        }
    }

    dominance_pairs.sort_by(|left, right| {
        left["scope"]
            .as_str()
            .cmp(&right["scope"].as_str())
            .then(
                left["scenario_id"]
                    .to_string()
                    .cmp(&right["scenario_id"].to_string()),
            )
            .then(left["dominates"].as_str().cmp(&right["dominates"].as_str()))
            .then(left["dominated"].as_str().cmp(&right["dominated"].as_str()))
    });

    Ok(json!({
        "benchmark_id":"HOME-FREEDOM-ROBUST-DECISION-001",
        "maturity":"SYNTHETIC_ANALYSIS_ONLY",
        "candidate_assessments":candidate_assessments,
        "scenario_frontiers":scenario_frontiers,
        "robust_frontier_candidate_ids":robust_frontier,
        "dominance_pairs":dominance_pairs,
        "conservative_regret_bounds":regret_bounds,
        "scalar_score_emitted":false,
        "physical_or_buildability_claim":false
    }))
}

fn run_self_tests() -> Result<(), String> {
    let fixture: Value =
        serde_json::from_str(FIXTURE).map_err(|error| format!("fixture JSON: {error}"))?;
    let report = evaluate(&fixture)?;

    let assessments = report["candidate_assessments"]
        .as_array()
        .ok_or_else(|| "candidate_assessments missing".to_owned())?;
    let disposition = |id: &str| {
        assessments
            .iter()
            .find(|row| row["candidate_id"].as_str() == Some(id))
            .and_then(|row| row["feasibility_disposition"].as_str())
    };
    if disposition("COURTYARD_A") != Some("ROBUST_FEASIBLE")
        || disposition("REFERENCE_B") != Some("ROBUST_FEASIBLE")
        || disposition("LOW_CAPEX_C") != Some("ROBUST_FEASIBLE")
        || disposition("ADAPTIVE_D") != Some("CONDITIONAL")
        || disposition("ACCESS_FAIL_E") != Some("BLOCKED")
        || disposition("METRIC_UNKNOWN_F") != Some("ROBUST_FEASIBLE")
    {
        return Err("frozen candidate dispositions do not match their known answers".to_owned());
    }

    let completeness = assessments
        .iter()
        .find(|row| row["candidate_id"].as_str() == Some("METRIC_UNKNOWN_F"))
        .and_then(|row| row["objective_completeness"].as_str());
    if completeness != Some("OBJECTIVES_INCOMPLETE") {
        return Err("unknown objective interval was not kept incomplete".to_owned());
    }

    let robust_frontier = report["robust_frontier_candidate_ids"]
        .as_array()
        .ok_or_else(|| "robust frontier missing".to_owned())?;
    let robust_ids: BTreeSet<String> = robust_frontier
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    if robust_ids != BTreeSet::from(["COURTYARD_A".to_owned(), "LOW_CAPEX_C".to_owned()]) {
        return Err(format!("unexpected robust frontier: {robust_ids:?}"));
    }

    let robust_pairs = report["dominance_pairs"]
        .as_array()
        .ok_or_else(|| "dominance pairs missing".to_owned())?;
    if !robust_pairs.iter().any(|row| {
        row["scope"].as_str() == Some("ALL_SCENARIOS")
            && row["dominates"].as_str() == Some("COURTYARD_A")
            && row["dominated"].as_str() == Some("REFERENCE_B")
    }) {
        return Err("the known all-scenario interval dominance was not found".to_owned());
    }

    // Permuting candidates and nested rows must not alter the deterministic report.
    let mut permuted = fixture.clone();
    permuted["candidates"].as_array_mut().unwrap().reverse();
    for candidate in permuted["candidates"].as_array_mut().unwrap() {
        candidate["scenario_results"]
            .as_array_mut()
            .unwrap()
            .reverse();
        for scenario in candidate["scenario_results"].as_array_mut().unwrap() {
            scenario["gate_results"].as_array_mut().unwrap().reverse();
            scenario["metric_intervals"]
                .as_array_mut()
                .unwrap()
                .reverse();
        }
    }
    if evaluate(&permuted)? != report {
        return Err("report changed under semantically irrelevant input permutation".to_owned());
    }

    // Removing a required hard-gate row must fail closed.
    let mut missing_gate = fixture.clone();
    missing_gate["candidates"][0]["scenario_results"][0]["gate_results"]
        .as_array_mut()
        .unwrap()
        .pop();
    if evaluate(&missing_gate).is_ok() {
        return Err("missing hard-gate evidence was accepted".to_owned());
    }

    // Reversing uncertainty bounds must be rejected.
    let mut inverted = fixture.clone();
    inverted["candidates"][0]["scenario_results"][0]["metric_intervals"][0]["lower"] = json!(200.0);
    if evaluate(&inverted).is_ok() {
        return Err("inverted uncertainty interval was accepted".to_owned());
    }

    // Widening an interval must not create a dominance claim.
    let mut widened = fixture.clone();
    widened["candidates"][0]["scenario_results"][1]["metric_intervals"][1]["upper"] = json!(300.0);
    let widened_report = evaluate(&widened)?;
    let widened_pairs = widened_report["dominance_pairs"].as_array().unwrap();
    if widened_pairs.iter().any(|row| {
        row["scope"].as_str() == Some("ALL_SCENARIOS")
            && row["dominates"].as_str() == Some("COURTYARD_A")
            && row["dominated"].as_str() == Some("REFERENCE_B")
    }) {
        return Err("widened interval retained an unjustified robust-dominance claim".to_owned());
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--self-test") {
        match run_self_tests() {
            Ok(()) => {
                println!(
                    "HOME_FREEDOM_ROBUST_DECISION_PASS: synthetic oracle, fail-closed gates, interval dominance, frontiers and permutation controls"
                );
            }
            Err(error) => {
                eprintln!("HOME_FREEDOM_ROBUST_DECISION_FAIL: {error}");
                process::exit(1);
            }
        }
        return;
    }

    let source = if let Some(path) = args.first() {
        match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("Unable to read {path}: {error}");
                process::exit(2);
            }
        }
    } else {
        FIXTURE.to_owned()
    };

    let input: Value = match serde_json::from_str(&source) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Invalid JSON input: {error}");
            process::exit(2);
        }
    };
    match evaluate(&input) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
        Err(error) => {
            eprintln!("HOME_FREEDOM_ROBUST_DECISION_REJECTED: {error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_synthetic_known_answers_pass() {
        run_self_tests().expect("robust decision fixture and negative controls must pass");
    }

    #[test]
    fn unknown_gate_is_never_robust_feasible() {
        let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
        let report = evaluate(&fixture).unwrap();
        let adaptive = report["candidate_assessments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["candidate_id"].as_str() == Some("ADAPTIVE_D"))
            .unwrap();
        assert_eq!(adaptive["feasibility_disposition"], "CONDITIONAL");
        assert_eq!(adaptive["dominance_eligible"], false);
    }

    #[test]
    fn unknown_objective_does_not_become_zero() {
        let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
        let report = evaluate(&fixture).unwrap();
        let candidate = report["candidate_assessments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["candidate_id"].as_str() == Some("METRIC_UNKNOWN_F"))
            .unwrap();
        assert_eq!(candidate["objective_completeness"], "OBJECTIVES_INCOMPLETE");
        assert_eq!(candidate["dominance_eligible"], false);
    }
}
