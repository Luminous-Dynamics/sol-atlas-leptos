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
    pub scenario_utilities: Vec<ScenarioUtility>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub best_utility_points: f64,
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
    if errors.is_empty() {
        Ok(())
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

    let mut scorecards = Vec::with_capacity(alternatives.len());
    for alternative in &alternatives {
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
        let worst_case_utility = scenario_utilities
            .iter()
            .map(|outcome| outcome.utility_points)
            .fold(f64::INFINITY, f64::min);
        let mean_utility = scenario_utilities
            .iter()
            .map(|outcome| outcome.utility_points)
            .sum::<f64>()
            / scenario_utilities.len() as f64;
        let maximum_regret = scenario_utilities
            .iter()
            .map(|outcome| outcome.regret_points)
            .fold(0.0_f64, f64::max);
        let mean_regret = scenario_utilities
            .iter()
            .map(|outcome| outcome.regret_points)
            .sum::<f64>()
            / scenario_utilities.len() as f64;
        scorecards.push(AlternativeScorecard {
            alternative_id: alternative.alternative_id.clone(),
            label: alternative.label.clone(),
            worst_case_utility_points: rounded(worst_case_utility),
            unweighted_mean_utility_points: rounded(mean_utility),
            maximum_regret_points: rounded(maximum_regret),
            unweighted_mean_regret_points: rounded(mean_regret),
            robustly_qualified: worst_case_utility >= fixture.minimum_worst_case_utility_points,
            scenario_utilities,
        });
    }

    let maximin_winner = scorecards
        .iter()
        .min_by(|left, right| {
            right
                .worst_case_utility_points
                .total_cmp(&left.worst_case_utility_points)
                .then_with(|| left.alternative_id.cmp(&right.alternative_id))
        })
        .map(|scorecard| scorecard.alternative_id.clone())
        .unwrap_or_default();
    let minimax_regret_winner = scorecards
        .iter()
        .min_by(|left, right| {
            left.maximum_regret_points
                .total_cmp(&right.maximum_regret_points)
                .then_with(|| left.alternative_id.cmp(&right.alternative_id))
        })
        .map(|scorecard| scorecard.alternative_id.clone())
        .unwrap_or_default();
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
        close(b.worst_case_utility_points, 40.0);
        close(b.unweighted_mean_utility_points, 59.25);
        close(b.maximum_regret_points, 35.0);
        assert!(!b.robustly_qualified);
        close(c.worst_case_utility_points, 72.0);
        close(c.unweighted_mean_utility_points, 73.75);
        close(c.maximum_regret_points, 17.0);
        close(c.unweighted_mean_regret_points, 6.5);
        assert!(c.robustly_qualified);
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
