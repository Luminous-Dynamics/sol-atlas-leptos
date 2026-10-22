// Copyright (C) 2026 Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Synthetic-only multi-scenario robustness and regret evaluation.
//! This module does not assign probabilities to scenarios or claim field impact.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const FIXTURE_SCHEMA: &str = "luminous.landscape_scenario_robustness_fixture.v0.1";
const REPORT_SCHEMA: &str = "luminous.landscape_scenario_robustness_report.v0.1";
const SUITE_ID: &str = "LANDSCAPE-SCENARIO-ROBUSTNESS-V0.1";
const CLAIM_CEILING: &str = "SCENARIO_ROBUSTNESS_ONLY";
const MAX_SCENARIOS: usize = 20;
const MAX_ALTERNATIVES: usize = 50;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobustnessFixture {
    pub schema_version: String,
    pub suite_id: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub utility_method_sha256: String,
    pub minimum_worst_case_utility_points: f64,
    pub scenarios: Vec<Scenario>,
    pub alternatives: Vec<Alternative>,
    pub outcomes: Vec<Outcome>,
    pub critical_constraints: Vec<CriticalConstraint>,
    pub constraint_outcomes: Vec<ConstraintOutcome>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub scenario_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alternative {
    pub alternative_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub scenario_id: String,
    pub alternative_id: String,
    pub utility_points: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriticalConstraint {
    pub constraint_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstraintOutcome {
    pub scenario_id: String,
    pub alternative_id: String,
    pub constraint_id: String,
    pub satisfied: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConstraintViolation {
    pub scenario_id: String,
    pub constraint_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityMethodSpec {
    #[serde(rename = "$schema")]
    pub json_schema_uri: String,
    #[serde(rename = "$id")]
    pub schema_id: String,
    pub schema_version: String,
    pub metric_name: String,
    pub unit: String,
    pub scale: UtilityScale,
    pub normalization: String,
    pub scenario_weighting: String,
    pub missing_data: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityScale {
    pub minimum: f64,
    pub maximum: f64,
    pub higher_is_better: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioUtility {
    pub scenario_id: String,
    pub utility_points: f64,
    pub regret_points: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AlternativeScorecard {
    pub alternative_id: String,
    pub label: String,
    pub worst_case_utility_points: f64,
    /// Descriptive average over the listed scenario set. It is not expected utility.
    pub unweighted_mean_utility_points: f64,
    pub maximum_regret_points: f64,
    /// Descriptive average regret over the listed scenario set. No scenario probabilities are assumed.
    pub unweighted_mean_regret_points: f64,
    pub robustly_qualified: bool,
    pub blocking_constraint_violations: Vec<ConstraintViolation>,
    pub scenario_utilities: Vec<ScenarioUtility>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub best_utility_points: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioOmissionResult {
    pub omitted_scenario_id: String,
    pub remaining_scenario_count: usize,
    pub maximin_winner: String,
    pub minimax_regret_winner: String,
    pub maximin_winner_changed: bool,
    pub minimax_regret_winner_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RobustnessReport {
    pub schema_version: String,
    pub suite_id: String,
    pub conformance_status: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub input_sha256: String,
    pub utility_method_sha256: String,
    pub scenario_count: usize,
    pub alternative_count: usize,
    pub minimum_worst_case_utility_points: f64,
    pub scorecards: Vec<AlternativeScorecard>,
    pub scenario_results: Vec<ScenarioResult>,
    /// Winner under the declared maximin-utility policy.
    pub maximin_winner: String,
    /// Winner under the declared minimax-regret policy. This may differ from maximin.
    pub minimax_regret_winner: String,
    /// Best utility-maximizing alternative among those passing all hard constraints
    /// in every scenario and meeting the worst-case utility threshold.
    pub feasible_maximin_winner: Option<String>,
    /// Minimax-regret alternative among those passing all hard constraints
    /// in every scenario and meeting the worst-case utility threshold.
    pub feasible_minimax_regret_winner: Option<String>,
    /// Leave-one-scenario-out diagnostic; this is not a reweighted expected score.
    pub scenario_omission_audit: Vec<ScenarioOmissionResult>,
    pub maximin_omission_stability_fraction: f64,
    pub minimax_regret_omission_stability_fraction: f64,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_digit() || ('a'..='f').contains(&ch))
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn validate_fixture(fixture: &RobustnessFixture) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if fixture.schema_version != FIXTURE_SCHEMA {
        errors.push(format!(
            "unsupported schema_version: {}",
            fixture.schema_version
        ));
    }
    if fixture.suite_id != SUITE_ID {
        errors.push(format!("unexpected suite_id: {}", fixture.suite_id));
    }
    if fixture.data_origin != "synthetic_fixture" {
        errors.push("v0.1 accepts synthetic_fixture only".to_owned());
    }
    if fixture.claim_ceiling != CLAIM_CEILING {
        errors.push(format!("claim_ceiling must be {CLAIM_CEILING}"));
    }
    if !valid_digest(&fixture.utility_method_sha256) {
        errors.push("utility_method_sha256 must be 64 lowercase hexadecimal characters".to_owned());
    }
    if !fixture.minimum_worst_case_utility_points.is_finite()
        || !(0.0..=100.0).contains(&fixture.minimum_worst_case_utility_points)
    {
        errors.push("minimum_worst_case_utility_points must be finite and in [0, 100]".to_owned());
    }
    if fixture.scenarios.len() < 2 || fixture.scenarios.len() > MAX_SCENARIOS {
        errors.push(format!(
            "scenarios must contain 2..={MAX_SCENARIOS} entries"
        ));
    }
    if fixture.alternatives.len() < 2 || fixture.alternatives.len() > MAX_ALTERNATIVES {
        errors.push(format!(
            "alternatives must contain 2..={MAX_ALTERNATIVES} entries"
        ));
    }

    let mut scenario_ids = BTreeSet::new();
    for scenario in &fixture.scenarios {
        if !nonempty(&scenario.scenario_id) || !nonempty(&scenario.label) {
            errors.push("scenario IDs and labels must be non-empty".to_owned());
        }
        if !scenario_ids.insert(scenario.scenario_id.as_str()) {
            errors.push(format!("duplicate scenario_id: {}", scenario.scenario_id));
        }
    }
    let mut alternative_ids = BTreeSet::new();
    for alternative in &fixture.alternatives {
        if !nonempty(&alternative.alternative_id) || !nonempty(&alternative.label) {
            errors.push("alternative IDs and labels must be non-empty".to_owned());
        }
        if !alternative_ids.insert(alternative.alternative_id.as_str()) {
            errors.push(format!(
                "duplicate alternative_id: {}",
                alternative.alternative_id
            ));
        }
    }

    let mut pairs = BTreeSet::new();
    for outcome in &fixture.outcomes {
        if !scenario_ids.contains(outcome.scenario_id.as_str()) {
            errors.push(format!(
                "outcome references unknown scenario: {}",
                outcome.scenario_id
            ));
        }
        if !alternative_ids.contains(outcome.alternative_id.as_str()) {
            errors.push(format!(
                "outcome references unknown alternative: {}",
                outcome.alternative_id
            ));
        }
        if !outcome.utility_points.is_finite() || !(0.0..=100.0).contains(&outcome.utility_points) {
            errors.push(format!(
                "{} / {}: utility_points must be finite and in [0, 100]",
                outcome.scenario_id, outcome.alternative_id
            ));
        }
        if !pairs.insert((
            outcome.scenario_id.as_str(),
            outcome.alternative_id.as_str(),
        )) {
            errors.push(format!(
                "duplicate outcome for scenario {} and alternative {}",
                outcome.scenario_id, outcome.alternative_id
            ));
        }
    }
    let expected_pairs = fixture
        .scenarios
        .len()
        .saturating_mul(fixture.alternatives.len());
    if pairs.len() != expected_pairs {
        errors.push(format!(
            "outcomes must contain exactly one result for every scenario/alternative pair: expected {expected_pairs}, found {}",
            pairs.len()
        ));
    }
    if fixture.outcomes.len() != expected_pairs {
        errors.push(format!(
            "outcome count must equal scenarios × alternatives ({expected_pairs})"
        ));
    }
    let mut constraint_ids = BTreeSet::new();
    if fixture.critical_constraints.is_empty() || fixture.critical_constraints.len() > 20 {
        errors.push("critical_constraints must contain 1..=20 explicit non-compensatory constraints".to_owned());
    }
    for constraint in &fixture.critical_constraints {
        if !nonempty(&constraint.constraint_id) || !nonempty(&constraint.label) {
            errors.push("critical constraint IDs and labels must be non-empty".to_owned());
        }
        if !constraint_ids.insert(constraint.constraint_id.as_str()) {
            errors.push(format!("duplicate constraint_id: {}", constraint.constraint_id));
        }
    }

    let mut constraint_pairs = BTreeSet::new();
    for outcome in &fixture.constraint_outcomes {
        if !scenario_ids.contains(outcome.scenario_id.as_str()) {
            errors.push(format!("constraint outcome references unknown scenario: {}", outcome.scenario_id));
        }
        if !alternative_ids.contains(outcome.alternative_id.as_str()) {
            errors.push(format!("constraint outcome references unknown alternative: {}", outcome.alternative_id));
        }
        if !constraint_ids.contains(outcome.constraint_id.as_str()) {
            errors.push(format!("constraint outcome references unknown constraint: {}", outcome.constraint_id));
        }
        if !constraint_pairs.insert((
            outcome.scenario_id.as_str(),
            outcome.alternative_id.as_str(),
            outcome.constraint_id.as_str(),
        )) {
            errors.push(format!(
                "duplicate constraint outcome for scenario {}, alternative {}, constraint {}",
                outcome.scenario_id, outcome.alternative_id, outcome.constraint_id
            ));
        }
    }
    let expected_constraint_outcomes = fixture
        .scenarios
        .len()
        .saturating_mul(fixture.alternatives.len())
        .saturating_mul(fixture.critical_constraints.len());
    if constraint_pairs.len() != expected_constraint_outcomes {
        errors.push(format!(
            "constraint_outcomes must contain exactly one result for every scenario/alternative/constraint tuple: expected {expected_constraint_outcomes}, found {}",
            constraint_pairs.len()
        ));
    }
    if fixture.constraint_outcomes.len() != expected_constraint_outcomes {
        errors.push(format!(
            "constraint_outcome count must equal scenarios × alternatives × constraints ({expected_constraint_outcomes})"
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        Err(errors)
    }
}

fn choose_winners_for_scenarios(
    scenario_ids: &[&str],
    alternatives: &[&Alternative],
    utility_by_pair: &BTreeMap<(&str, &str), f64>,
) -> (String, String) {
    let mut maximin_winner = String::new();
    let mut maximin_score = f64::NEG_INFINITY;
    let mut minimax_regret_winner = String::new();
    let mut minimax_regret_score = f64::INFINITY;

    for alternative in alternatives {
        let worst_case = scenario_ids
            .iter()
            .map(|scenario_id| {
                utility_by_pair[&(*scenario_id, alternative.alternative_id.as_str())]
            })
            .fold(f64::INFINITY, f64::min);
        if worst_case > maximin_score
            || (worst_case.total_cmp(&maximin_score).is_eq()
                && (maximin_winner.is_empty() || alternative.alternative_id < maximin_winner))
        {
            maximin_score = worst_case;
            maximin_winner = alternative.alternative_id.clone();
        }

        let maximum_regret = scenario_ids
            .iter()
            .map(|scenario_id| {
                let best_in_scenario = alternatives
                    .iter()
                    .map(|candidate| {
                        utility_by_pair[&(*scenario_id, candidate.alternative_id.as_str())]
                    })
                    .fold(f64::NEG_INFINITY, f64::max);
                best_in_scenario
                    - utility_by_pair[&(*scenario_id, alternative.alternative_id.as_str())]
            })
            .fold(0.0_f64, f64::max);
        if maximum_regret < minimax_regret_score
            || (maximum_regret.total_cmp(&minimax_regret_score).is_eq()
                && (minimax_regret_winner.is_empty()
                    || alternative.alternative_id < minimax_regret_winner))
        {
            minimax_regret_score = maximum_regret;
            minimax_regret_winner = alternative.alternative_id.clone();
        }
    }

    (maximin_winner, minimax_regret_winner)
}

pub fn validate_utility_method_bytes(
    utility_method_bytes: &[u8],
) -> Result<UtilityMethodSpec, Vec<String>> {
    let method: UtilityMethodSpec = serde_json::from_slice(utility_method_bytes)
        .map_err(|error| vec![format!("utility method JSON rejected: {error}")])?;
    let mut errors = Vec::new();
    if method.json_schema_uri != "https://json-schema.org/draft/2020-12/schema" {
        errors.push("utility method $schema must use JSON Schema draft 2020-12".to_owned());
    }
    if method.schema_id
        != "https://luminousdynamics.io/schemas/landscape-utility-method-v0.schema.json"
    {
        errors.push("utility method $id does not match the v0 contract".to_owned());
    }
    if method.schema_version != "luminous.landscape_utility_method.v0" {
        errors.push("utility method schema_version does not match v0".to_owned());
    }
    if !nonempty(&method.metric_name) {
        errors.push("utility method metric_name must be non-empty".to_owned());
    }
    if method.unit != "utility_points" {
        errors.push("utility method unit must be utility_points".to_owned());
    }
    if method.scale.minimum != 0.0
        || method.scale.maximum != 100.0
        || !method.scale.higher_is_better
    {
        errors.push("utility method scale must be [0, 100] with higher_is_better=true".to_owned());
    }
    if !nonempty(&method.normalization) {
        errors.push("utility method normalization must be explicitly documented".to_owned());
    }
    if method.scenario_weighting
        != "none; listed cases are stress scenarios, not probability-weighted forecasts"
    {
        errors.push(
            "v0.1 does not infer or accept scenario probability weights; use an explicitly reviewed method version"
                .to_owned(),
        );
    }
    if method.missing_data != "fail_closed" {
        errors.push("utility method missing_data must be fail_closed".to_owned());
    }
    if method.claim_ceiling != "scenario_robustness_only_not_real_world_impact" {
        errors.push("utility method claim_ceiling exceeds or contradicts the v0.1 scope".to_owned());
    }
    if errors.is_empty() {
        Ok(method)
    } else {
        errors.sort();
        Err(errors)
    }
}

pub fn evaluate_fixture_bytes(
    fixture_bytes: &[u8],
    utility_method_bytes: &[u8],
) -> Result<RobustnessReport, Vec<String>> {
    let fixture: RobustnessFixture = serde_json::from_slice(fixture_bytes)
        .map_err(|error| vec![format!("fixture JSON rejected: {error}")])?;
    validate_fixture(&fixture)?;
    let actual_method_sha = sha256_hex(utility_method_bytes);
    if actual_method_sha != fixture.utility_method_sha256 {
        return Err(vec![format!(
            "utility method digest mismatch: fixture says {}, actual bytes hash to {}",
            fixture.utility_method_sha256, actual_method_sha
        )]);
    }
    // Hash identity and semantic validity are separate gates: exact bytes may
    // still describe a scale or weighting policy outside this contract.
    validate_utility_method_bytes(utility_method_bytes)?;

    let mut scenario_ids: Vec<&str> = fixture
        .scenarios
        .iter()
        .map(|scenario| scenario.scenario_id.as_str())
        .collect();
    scenario_ids.sort_unstable();
    let mut alternatives: Vec<&Alternative> = fixture.alternatives.iter().collect();
    alternatives.sort_by(|left, right| left.alternative_id.cmp(&right.alternative_id));

    let mut utility_by_pair: BTreeMap<(&str, &str), f64> = BTreeMap::new();
    for outcome in &fixture.outcomes {
        utility_by_pair.insert(
            (
                outcome.scenario_id.as_str(),
                outcome.alternative_id.as_str(),
            ),
            outcome.utility_points,
        );
    }
    let scenario_best: BTreeMap<&str, f64> = scenario_ids
        .iter()
        .map(|scenario_id| {
            let best = alternatives
                .iter()
                .map(|alternative| {
                    utility_by_pair[&(scenario_id, alternative.alternative_id.as_str())]
                })
                .fold(f64::NEG_INFINITY, f64::max);
            (*scenario_id, best)
        })
        .collect();

    let constraint_labels: BTreeMap<&str, &str> = fixture
        .critical_constraints
        .iter()
        .map(|constraint| (constraint.constraint_id.as_str(), constraint.label.as_str()))
        .collect();
    let mut blocking_by_pair: BTreeMap<(&str, &str), Vec<ConstraintViolation>> =
        BTreeMap::new();
    for outcome in fixture.constraint_outcomes.iter().filter(|outcome| !outcome.satisfied) {
        blocking_by_pair
            .entry((outcome.scenario_id.as_str(), outcome.alternative_id.as_str()))
            .or_default()
            .push(ConstraintViolation {
                scenario_id: outcome.scenario_id.clone(),
                constraint_id: outcome.constraint_id.clone(),
                label: constraint_labels[ outcome.constraint_id.as_str() ].to_owned(),
            });
    }

    let mut scorecards = Vec::with_capacity(alternatives.len());
    for alternative in &alternatives {
        let raw_utilities: Vec<f64> = scenario_ids
            .iter()
            .map(|scenario_id| utility_by_pair[&(scenario_id, alternative.alternative_id.as_str())])
            .collect();
        let scenario_utilities: Vec<ScenarioUtility> = scenario_ids
            .iter()
            .map(|scenario_id| {
                let utility = utility_by_pair[&(scenario_id, alternative.alternative_id.as_str())];
                ScenarioUtility {
                    scenario_id: (*scenario_id).to_owned(),
                    utility_points: rounded(utility),
                    regret_points: rounded(scenario_best[scenario_id] - utility),
                }
            })
            .collect();
        // Make decisions from raw validated values. Rounding is presentation
        // only; it must not create artificial ties that then fall to ID order.
        let worst_case_utility = raw_utilities.iter().copied().fold(f64::INFINITY, f64::min);
        let mean_utility = raw_utilities.iter().sum::<f64>() / raw_utilities.len() as f64;
        let raw_regrets: Vec<f64> = scenario_ids
            .iter()
            .map(|scenario_id| {
                scenario_best[scenario_id]
                    - utility_by_pair[&(scenario_id, alternative.alternative_id.as_str())]
            })
            .collect();
        let maximum_regret = raw_regrets.iter().copied().fold(0.0_f64, f64::max);
        let mean_regret = raw_regrets.iter().sum::<f64>() / raw_regrets.len() as f64;
        let blocking_constraint_violations: Vec<ConstraintViolation> = scenario_ids
            .iter()
            .flat_map(|scenario_id| {
                blocking_by_pair
                    .get(&(*scenario_id, alternative.alternative_id.as_str()))
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();
        scorecards.push(AlternativeScorecard {
            alternative_id: alternative.alternative_id.clone(),
            label: alternative.label.clone(),
            worst_case_utility_points: rounded(worst_case_utility),
            unweighted_mean_utility_points: rounded(mean_utility),
            maximum_regret_points: rounded(maximum_regret),
            unweighted_mean_regret_points: rounded(mean_regret),
            robustly_qualified: worst_case_utility >= fixture.minimum_worst_case_utility_points
                && blocking_constraint_violations.is_empty(),
            blocking_constraint_violations,
            scenario_utilities,
        });
    }

    let (maximin_winner, minimax_regret_winner) =
        choose_winners_for_scenarios(&scenario_ids, &alternatives, &utility_by_pair);
    let feasible_alternatives: Vec<&Alternative> = alternatives
        .iter()
        .copied()
        .filter(|alternative| {
            scorecards.iter().any(|scorecard| {
                scorecard.alternative_id == alternative.alternative_id
                    && scorecard.robustly_qualified
            })
        })
        .collect();
    let (feasible_maximin_winner, feasible_minimax_regret_winner) =
        if feasible_alternatives.is_empty() {
            (None, None)
        } else {
            let (maximin, regret) = choose_winners_for_scenarios(
                &scenario_ids,
                &feasible_alternatives,
                &utility_by_pair,
            );
            (Some(maximin), Some(regret))
        };
    // Test the composition sensitivity of each decision rule by removing one
    // declared scenario at a time. Remaining cases are not reweighted and are
    // not interpreted as probabilities.
    let mut scenario_omission_audit = Vec::with_capacity(scenario_ids.len());
    let mut maximin_stable_count = 0usize;
    let mut minimax_regret_stable_count = 0usize;
    for omitted_scenario_id in &scenario_ids {
        let remaining_scenarios: Vec<&str> = scenario_ids
            .iter()
            .copied()
            .filter(|scenario_id| scenario_id != omitted_scenario_id)
            .collect();
        let (omitted_maximin_winner, omitted_minimax_regret_winner) =
            choose_winners_for_scenarios(&remaining_scenarios, &alternatives, &utility_by_pair);
        let maximin_winner_changed = omitted_maximin_winner != maximin_winner;
        let minimax_regret_winner_changed = omitted_minimax_regret_winner != minimax_regret_winner;
        if !maximin_winner_changed {
            maximin_stable_count += 1;
        }
        if !minimax_regret_winner_changed {
            minimax_regret_stable_count += 1;
        }
        scenario_omission_audit.push(ScenarioOmissionResult {
            omitted_scenario_id: (*omitted_scenario_id).to_owned(),
            remaining_scenario_count: remaining_scenarios.len(),
            maximin_winner: omitted_maximin_winner,
            minimax_regret_winner: omitted_minimax_regret_winner,
            maximin_winner_changed,
            minimax_regret_winner_changed,
        });
    }
    let maximin_omission_stability_fraction =
        maximin_stable_count as f64 / scenario_omission_audit.len() as f64;
    let minimax_regret_omission_stability_fraction =
        minimax_regret_stable_count as f64 / scenario_omission_audit.len() as f64;
    let scenario_results = scenario_ids
        .iter()
        .map(|scenario_id| ScenarioResult {
            scenario_id: (*scenario_id).to_owned(),
            best_utility_points: rounded(scenario_best[scenario_id]),
        })
        .collect();

    Ok(RobustnessReport {
        schema_version: REPORT_SCHEMA.to_owned(),
        suite_id: SUITE_ID.to_owned(),
        conformance_status: "PASS_SYNTHETIC_SCENARIO_ROBUSTNESS_ONLY".to_owned(),
        data_origin: "synthetic_fixture".to_owned(),
        claim_ceiling: CLAIM_CEILING.to_owned(),
        input_sha256: sha256_hex(fixture_bytes),
        utility_method_sha256: actual_method_sha,
        scenario_count: fixture.scenarios.len(),
        alternative_count: fixture.alternatives.len(),
        minimum_worst_case_utility_points: rounded(fixture.minimum_worst_case_utility_points),
        scorecards,
        scenario_results,
        maximin_winner,
        minimax_regret_winner,
        feasible_maximin_winner,
        feasible_minimax_regret_winner,
        scenario_omission_audit,
        maximin_omission_stability_fraction: rounded(maximin_omission_stability_fraction),
        minimax_regret_omission_stability_fraction: rounded(
            minimax_regret_omission_stability_fraction,
        ),
    })
}

fn rounded(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub fn default_fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/landscape/robustness-v0/fixture.json")
}

pub fn default_utility_method_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/landscape-utility-method-v0.schema.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture_bytes() -> Vec<u8> {
        fs::read(default_fixture_path()).expect("frozen robustness fixture")
    }

    fn method_bytes() -> Vec<u8> {
        fs::read(default_utility_method_path()).expect("frozen utility method")
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1.0e-8,
            "actual={actual}, expected={expected}"
        );
    }

    #[test]
    fn frozen_fixture_exposes_mean_versus_robustness_tradeoff() {
        let report =
            evaluate_fixture_bytes(&fixture_bytes(), &method_bytes()).expect("fixture report");
        assert_eq!(
            report.conformance_status,
            "PASS_SYNTHETIC_SCENARIO_ROBUSTNESS_ONLY"
        );
        assert_eq!(report.scenario_count, 4);
        assert_eq!(report.alternative_count, 3);
        assert_eq!(report.maximin_winner, "strategy-c");
        assert_eq!(report.minimax_regret_winner, "strategy-a");
        assert_eq!(report.feasible_maximin_winner.as_deref(), Some("strategy-a"));
        assert_eq!(report.feasible_minimax_regret_winner.as_deref(), Some("strategy-a"));
        assert_eq!(report.scenario_omission_audit.len(), 4);
        close(report.maximin_omission_stability_fraction, 0.75);
        close(report.minimax_regret_omission_stability_fraction, 1.0);
        assert_eq!(
            report.scenario_omission_audit[0].omitted_scenario_id,
            "baseline"
        );
        assert_eq!(
            report.scenario_omission_audit[0].maximin_winner,
            "strategy-c"
        );
        assert!(!report.scenario_omission_audit[0].maximin_winner_changed);
        assert_eq!(
            report.scenario_omission_audit[1].omitted_scenario_id,
            "extreme_rain"
        );
        assert_eq!(
            report.scenario_omission_audit[1].maximin_winner,
            "strategy-a"
        );
        assert!(report.scenario_omission_audit[1].maximin_winner_changed);
        assert_eq!(
            report.scenario_omission_audit[1].minimax_regret_winner,
            "strategy-a"
        );
        assert!(!report.scenario_omission_audit[1].minimax_regret_winner_changed);
        assert!(
            report
                .scenario_omission_audit
                .iter()
                .all(|entry| entry.remaining_scenario_count == 3)
        );
        let a = report
            .scorecards
            .iter()
            .find(|item| item.alternative_id == "strategy-a")
            .unwrap();
        let b = report
            .scorecards
            .iter()
            .find(|item| item.alternative_id == "strategy-b")
            .unwrap();
        let c = report
            .scorecards
            .iter()
            .find(|item| item.alternative_id == "strategy-c")
            .unwrap();
        close(a.worst_case_utility_points, 70.0);
        close(a.unweighted_mean_utility_points, 76.25);
        close(a.maximum_regret_points, 10.0);
        close(a.unweighted_mean_regret_points, 4.0);
        assert!(a.robustly_qualified);
        assert!(a.blocking_constraint_violations.is_empty());
        close(b.worst_case_utility_points, 40.0);
        close(b.unweighted_mean_utility_points, 59.25);
        close(b.maximum_regret_points, 35.0);
        assert!(!b.robustly_qualified);
        close(c.worst_case_utility_points, 72.0);
        close(c.unweighted_mean_utility_points, 73.75);
        close(c.maximum_regret_points, 17.0);
        close(c.unweighted_mean_regret_points, 6.5);
        assert!(!c.robustly_qualified);
        assert_eq!(c.blocking_constraint_violations.len(), 1);
        assert_eq!(c.blocking_constraint_violations[0].scenario_id, "maintenance_shock");
        assert_eq!(c.blocking_constraint_violations[0].constraint_id, "accessible_route_continuity");
    }

    #[test]
    fn raw_scores_not_rounded_display_values_determine_winners() {
        let mut fixture: RobustnessFixture =
            serde_json::from_slice(&fixture_bytes()).expect("fixture parses");
        fixture.scenarios = vec![
            Scenario {
                scenario_id: "case-a".to_owned(),
                label: "close-score test A".to_owned(),
            },
            Scenario {
                scenario_id: "case-b".to_owned(),
                label: "close-score test B".to_owned(),
            },
        ];
        fixture.alternatives = vec![
            Alternative {
                alternative_id: "alpha".to_owned(),
                label: "lexically earlier but slightly worse".to_owned(),
            },
            Alternative {
                alternative_id: "beta".to_owned(),
                label: "slightly better utility".to_owned(),
            },
        ];
        fixture.minimum_worst_case_utility_points = 0.0;
        fixture.outcomes = vec![
            Outcome {
                scenario_id: "case-a".to_owned(),
                alternative_id: "alpha".to_owned(),
                utility_points: 50.0000001,
            },
            Outcome {
                scenario_id: "case-b".to_owned(),
                alternative_id: "alpha".to_owned(),
                utility_points: 60.0,
            },
            Outcome {
                scenario_id: "case-a".to_owned(),
                alternative_id: "beta".to_owned(),
                utility_points: 50.0000004,
            },
            Outcome {
                scenario_id: "case-b".to_owned(),
                alternative_id: "beta".to_owned(),
                utility_points: 60.0,
            },
        ];
        let bytes = serde_json::to_vec(&fixture).expect("serialize fixture");
        let report = evaluate_fixture_bytes(&bytes, &method_bytes()).expect("close-score report");
        assert_eq!(report.maximin_winner, "beta");
        let alpha = report
            .scorecards
            .iter()
            .find(|item| item.alternative_id == "alpha")
            .unwrap();
        let beta = report
            .scorecards
            .iter()
            .find(|item| item.alternative_id == "beta")
            .unwrap();
        close(alpha.worst_case_utility_points, 50.0);
        close(beta.worst_case_utility_points, 50.0);
        assert_eq!(
            alpha.worst_case_utility_points,
            beta.worst_case_utility_points
        );
    }

    #[test]
    fn utility_method_semantics_are_validated_even_when_digest_matches() {
        let bytes = method_bytes();
        let mut method: serde_json::Value = serde_json::from_slice(&bytes).expect("method JSON");
        method["scale"]["maximum"] = serde_json::json!(1000);
        let mutated = serde_json::to_vec(&method).expect("mutated method JSON");
        assert!(validate_utility_method_bytes(&mutated)
            .unwrap_err()
            .iter()
            .any(|error| error.contains("scale must be [0, 100]")));

        method["scale"]["maximum"] = serde_json::json!(100);
        method["scenario_weighting"] =
            serde_json::json!("equal probability for every scenario");
        let mutated = serde_json::to_vec(&method).expect("mutated weighting method");
        assert!(validate_utility_method_bytes(&mutated)
            .unwrap_err()
            .iter()
            .any(|error| error.contains("does not infer or accept scenario probability weights")));
    }

    #[test]
    fn utility_method_rejects_undeclared_fields() {
        let bytes = method_bytes();
        let mut method: serde_json::Value = serde_json::from_slice(&bytes).expect("method JSON");
        method["secret_policy_override"] = serde_json::json!(true);
        let mutated = serde_json::to_vec(&method).expect("method with unknown field");
        assert!(validate_utility_method_bytes(&mutated).is_err());
    }

    #[test]
    fn incomplete_critical_constraint_matrix_fails_closed() {
        let mut fixture: RobustnessFixture =
            serde_json::from_slice(&fixture_bytes()).expect("fixture parses");
        fixture.constraint_outcomes.pop();
        assert!(validate_fixture(&fixture)
            .unwrap_err()
            .iter()
            .any(|error| error.contains("exactly one result for every scenario/alternative/constraint tuple")));
    }

    #[test]
    fn no_hard_feasible_alternative_means_no_gated_recommendation() {
        let mut fixture: RobustnessFixture =
            serde_json::from_slice(&fixture_bytes()).expect("fixture parses");
        for outcome in &mut fixture.constraint_outcomes {
            outcome.satisfied = false;
        }
        let bytes = serde_json::to_vec(&fixture).expect("fixture JSON");
        let report = evaluate_fixture_bytes(&bytes, &method_bytes()).expect("valid but infeasible panel");
        assert_eq!(report.maximin_winner, "strategy-c");
        assert_eq!(report.feasible_maximin_winner, None);
        assert_eq!(report.feasible_minimax_regret_winner, None);
        assert!(report.scorecards.iter().all(|scorecard| !scorecard.robustly_qualified));
    }

    #[test]
    fn method_digest_is_bound_to_exact_bytes() {
        let mut bad_method = method_bytes();
        bad_method.push(b' ');
        assert!(
            evaluate_fixture_bytes(&fixture_bytes(), &bad_method)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("utility method digest mismatch"))
        );
    }

    #[test]
    fn missing_scenario_alternative_pair_fails_closed() {
        let mut fixture: RobustnessFixture =
            serde_json::from_slice(&fixture_bytes()).expect("fixture parses");
        fixture.outcomes.pop();
        assert!(
            validate_fixture(&fixture)
                .unwrap_err()
                .iter()
                .any(|error| error
                    .contains("exactly one result for every scenario/alternative pair"))
        );
    }

    #[test]
    fn duplicate_scenario_alternative_pair_is_rejected() {
        let mut fixture: RobustnessFixture =
            serde_json::from_slice(&fixture_bytes()).expect("fixture parses");
        fixture.outcomes.push(fixture.outcomes[0].clone());
        assert!(
            validate_fixture(&fixture)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("duplicate outcome"))
        );
    }

    #[test]
    fn invalid_utility_range_and_observed_relabel_fail_closed() {
        let mut fixture: RobustnessFixture =
            serde_json::from_slice(&fixture_bytes()).expect("fixture parses");
        fixture.outcomes[0].utility_points = 101.0;
        assert!(validate_fixture(&fixture).is_err());
        fixture.outcomes[0].utility_points = 90.0;
        fixture.data_origin = "field_observation".to_owned();
        assert!(validate_fixture(&fixture).is_err());
    }

    #[test]
    fn identical_bytes_produce_identical_reports() {
        let bytes = fixture_bytes();
        let first = evaluate_fixture_bytes(&bytes, &method_bytes()).expect("first report");
        let second = evaluate_fixture_bytes(&bytes, &method_bytes()).expect("second report");
        assert_eq!(
            serde_json::to_vec(&first).expect("first JSON"),
            serde_json::to_vec(&second).expect("second JSON")
        );
        assert_eq!(first.input_sha256, sha256_hex(&bytes));
    }
}
