// Copyright (C) 2026 Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Synthetic-only, consent-aware measurement-campaign selection.
//!
//! The score is a symmetrized KL-divergence between Gaussian predictive
//! distributions, weighted by an explicit decision weight. It is a synthetic
//! discrimination proxy, not a claim of real-world expected utility or a full
//! Bayesian value-of-information calculation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_SCHEMA: &str = "luminous.landscape_measurement_selection_fixture.v0.1";
const SUITE_ID: &str = "LANDSCAPE-MEASUREMENT-PLANNER-V0.1";
const CLAIM_CEILING: &str = "SYNTHETIC_SELECTION_ONLY";
const REPORT_SCHEMA: &str = "luminous.landscape_measurement_selection_report.v0.1";
const MAX_ABS_MEAN: f64 = 1.0e9;
const MAX_SD: f64 = 1.0e6;
const MAX_WEIGHT: f64 = 1.0e6;
const MAX_COST: u32 = 100_000;
const MAX_CANDIDATES: usize = 1_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub schema_version: String,
    pub suite_id: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub budget_cost_units: u32,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub candidate_id: String,
    pub metric_id: String,
    pub unit: String,
    pub spatial_stratum: String,
    pub temporal_stratum: String,
    pub site_ref_sha256: String,
    /// Candidates in the same group are treated as redundant for v0.1:
    /// the optimizer may choose at most one of them.
    pub correlation_group: String,
    pub baseline_model_id: String,
    pub alternative_model_id: String,
    pub baseline_mean: f64,
    pub alternative_mean: f64,
    pub baseline_sd: f64,
    pub alternative_sd: f64,
    pub observation_noise_sd: f64,
    /// Explicit, caller-supplied importance weight; no hidden cross-metric weight.
    pub decision_weight: f64,
    pub cost_units: u32,
    pub consent_status: ConsentStatus,
    pub access_status: AccessStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    Granted,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessStatus {
    Authorized,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CandidateDecision {
    pub candidate_id: String,
    pub decision: String,
    pub reason: Option<String>,
    pub correlation_group: String,
    pub cost_units: u32,
    pub symmetric_kl_nats: f64,
    pub weighted_discrimination_nats: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanReport {
    pub schema_version: String,
    pub suite_id: String,
    pub conformance_status: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub input_sha256: String,
    pub budget_cost_units: u32,
    pub selected_cost_units: u32,
    pub unspent_budget_units: u32,
    pub candidate_count: usize,
    pub eligible_candidate_count: usize,
    pub ineligible_candidate_count: usize,
    pub selected_candidate_count: usize,
    pub eligible_not_selected_count: usize,
    pub eligible_spatial_temporal_stratum_count: usize,
    pub selected_spatial_temporal_stratum_count: usize,
    pub eligible_spatial_temporal_stratum_coverage_fraction: f64,
    pub selected_candidate_ids: Vec<String>,
    pub total_weighted_discrimination_nats: f64,
    pub greedy_ratio_baseline_selected_candidate_ids: Vec<String>,
    pub greedy_ratio_baseline_cost_units: u32,
    pub greedy_ratio_baseline_weighted_discrimination_nats: f64,
    pub exact_gain_over_greedy_ratio_baseline_nats: f64,
    pub decisions: Vec<CandidateDecision>,
}

#[derive(Debug, Clone)]
struct CandidateScore {
    symmetric_kl_nats: f64,
    weighted_discrimination_nats: f64,
}

#[derive(Debug, Clone)]
struct PlanState {
    weighted_discrimination_nats: f64,
    selected_ids: Vec<String>,
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

fn bounded_finite(value: f64, limit: f64) -> bool {
    value.is_finite() && value.abs() <= limit
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn validate_fixture(fixture: &Fixture) -> Result<(), Vec<String>> {
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
    if fixture.budget_cost_units == 0 || fixture.budget_cost_units > MAX_COST {
        errors.push(format!(
            "budget_cost_units must be between 1 and {MAX_COST}"
        ));
    }
    if fixture.candidates.is_empty() {
        errors.push("candidates must not be empty".to_owned());
    }
    if fixture.candidates.len() > MAX_CANDIDATES {
        errors.push(format!("candidates must not exceed {MAX_CANDIDATES}"));
    }

    let mut ids = BTreeSet::new();
    for candidate in &fixture.candidates {
        if !nonempty(&candidate.candidate_id) {
            errors.push("candidate_id must be non-empty".to_owned());
        }
        if !ids.insert(candidate.candidate_id.clone()) {
            errors.push(format!(
                "duplicate candidate_id: {}",
                candidate.candidate_id
            ));
        }
        for (name, value) in [
            ("metric_id", candidate.metric_id.as_str()),
            ("unit", candidate.unit.as_str()),
            ("spatial_stratum", candidate.spatial_stratum.as_str()),
            ("temporal_stratum", candidate.temporal_stratum.as_str()),
            ("correlation_group", candidate.correlation_group.as_str()),
            ("baseline_model_id", candidate.baseline_model_id.as_str()),
            (
                "alternative_model_id",
                candidate.alternative_model_id.as_str(),
            ),
        ] {
            if !nonempty(value) {
                errors.push(format!(
                    "{}: {name} must be non-empty",
                    candidate.candidate_id
                ));
            }
        }
        if candidate.baseline_model_id == candidate.alternative_model_id {
            errors.push(format!(
                "{}: baseline and alternative model IDs must differ",
                candidate.candidate_id
            ));
        }
        if !valid_digest(&candidate.site_ref_sha256) {
            errors.push(format!(
                "{}: site_ref_sha256 must be 64 lowercase hexadecimal characters",
                candidate.candidate_id
            ));
        }
        for (name, value) in [
            ("baseline_mean", candidate.baseline_mean),
            ("alternative_mean", candidate.alternative_mean),
        ] {
            if !bounded_finite(value, MAX_ABS_MEAN) {
                errors.push(format!(
                    "{}: {name} must be finite and within ±{MAX_ABS_MEAN:e}",
                    candidate.candidate_id
                ));
            }
        }
        for (name, value) in [
            ("baseline_sd", candidate.baseline_sd),
            ("alternative_sd", candidate.alternative_sd),
            ("observation_noise_sd", candidate.observation_noise_sd),
        ] {
            if !bounded_finite(value, MAX_SD) || value <= 0.0 {
                errors.push(format!(
                    "{}: {name} must be positive, finite, and no greater than {MAX_SD:e}",
                    candidate.candidate_id
                ));
            }
        }
        if !bounded_finite(candidate.decision_weight, MAX_WEIGHT) || candidate.decision_weight < 0.0
        {
            errors.push(format!(
                "{}: decision_weight must be finite and between 0 and {MAX_WEIGHT:e}",
                candidate.candidate_id
            ));
        }
        if candidate.cost_units == 0 || candidate.cost_units > MAX_COST {
            errors.push(format!(
                "{}: cost_units must be between 1 and {MAX_COST}",
                candidate.candidate_id
            ));
        }
        let score = candidate_score(candidate);
        if !score.symmetric_kl_nats.is_finite() || !score.weighted_discrimination_nats.is_finite() {
            errors.push(format!(
                "{}: discrimination score is not finite",
                candidate.candidate_id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        Err(errors)
    }
}

/// Symmetrized KL divergence (in nats) between the two Gaussian predictive
/// distributions after adding independent observation noise to each variance.
/// This quantifies pairwise distributional separation; it is not mutual
/// information, posterior decision regret, or a calibrated impact estimate.
pub fn symmetric_kl_nats(candidate: &Candidate) -> f64 {
    let observation_variance = candidate.observation_noise_sd.powi(2);
    let baseline_variance = candidate.baseline_sd.powi(2) + observation_variance;
    let alternative_variance = candidate.alternative_sd.powi(2) + observation_variance;
    let mean_delta = candidate.baseline_mean - candidate.alternative_mean;
    let variance_term =
        baseline_variance / alternative_variance + alternative_variance / baseline_variance - 2.0;
    let mean_term = mean_delta.powi(2) * (1.0 / baseline_variance + 1.0 / alternative_variance);
    (0.25 * (variance_term + mean_term)).max(0.0)
}

fn candidate_score(candidate: &Candidate) -> CandidateScore {
    let symmetric_kl_nats = symmetric_kl_nats(candidate);
    CandidateScore {
        symmetric_kl_nats,
        weighted_discrimination_nats: symmetric_kl_nats * candidate.decision_weight,
    }
}

fn eligible(candidate: &Candidate) -> bool {
    candidate.consent_status == ConsentStatus::Granted
        && candidate.access_status == AccessStatus::Authorized
}

fn ids_tie_break(candidate: &PlanState, current: &PlanState) -> bool {
    if candidate.selected_ids.len() != current.selected_ids.len() {
        return candidate.selected_ids.len() < current.selected_ids.len();
    }
    candidate.selected_ids < current.selected_ids
}

fn better_for_same_cost(candidate: &PlanState, current: &PlanState) -> bool {
    match candidate
        .weighted_discrimination_nats
        .total_cmp(&current.weighted_discrimination_nats)
    {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => ids_tie_break(candidate, current),
    }
}

fn better_final(
    candidate_cost: u32,
    candidate: &PlanState,
    current_cost: u32,
    current: &PlanState,
) -> bool {
    match candidate
        .weighted_discrimination_nats
        .total_cmp(&current.weighted_discrimination_nats)
    {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => {
            if candidate_cost != current_cost {
                candidate_cost < current_cost
            } else {
                ids_tie_break(candidate, current)
            }
        }
    }
}

fn select_exact_budgeted(
    candidates: &[Candidate],
    scores: &BTreeMap<String, CandidateScore>,
    budget: u32,
) -> PlanState {
    let mut by_group: BTreeMap<&str, Vec<&Candidate>> = BTreeMap::new();
    for candidate in candidates.iter().filter(|candidate| eligible(candidate)) {
        by_group
            .entry(candidate.correlation_group.as_str())
            .or_default()
            .push(candidate);
    }
    for group in by_group.values_mut() {
        group.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    }

    // Dynamic programming across correlation groups is exact for the declared
    // v0.1 budget and one-candidate-per-group constraint.
    let mut states: Vec<Option<PlanState>> = vec![None; budget as usize + 1];
    states[0] = Some(PlanState {
        weighted_discrimination_nats: 0.0,
        selected_ids: Vec::new(),
    });

    for candidates_in_group in by_group.values() {
        let previous = states.clone();
        let mut next = previous.clone(); // select no item from this group
        for (spent, state) in previous.iter().enumerate() {
            let Some(state) = state else { continue };
            for candidate in candidates_in_group {
                let new_spent = spent + candidate.cost_units as usize;
                if new_spent > budget as usize {
                    continue;
                }
                let mut selected_ids = state.selected_ids.clone();
                selected_ids.push(candidate.candidate_id.clone());
                selected_ids.sort();
                let score = &scores[&candidate.candidate_id];
                let proposed = PlanState {
                    weighted_discrimination_nats: state.weighted_discrimination_nats
                        + score.weighted_discrimination_nats,
                    selected_ids,
                };
                let slot = &mut next[new_spent];
                if slot
                    .as_ref()
                    .is_none_or(|current| better_for_same_cost(&proposed, current))
                {
                    *slot = Some(proposed);
                }
            }
        }
        states = next;
    }

    let mut best_cost = 0u32;
    let mut best = states[0]
        .clone()
        .expect("zero-cost empty plan is always a valid state");
    for (cost, state) in states.iter().enumerate() {
        let Some(state) = state else { continue };
        if better_final(cost as u32, state, best_cost, &best) {
            best_cost = cost as u32;
            best = state.clone();
        }
    }
    best
}

fn select_greedy_ratio(
    candidates: &[Candidate],
    scores: &BTreeMap<String, CandidateScore>,
    budget: u32,
) -> PlanState {
    let mut ordered: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .collect();
    ordered.sort_by(|left, right| {
        let left_ratio =
            scores[&left.candidate_id].weighted_discrimination_nats / left.cost_units as f64;
        let right_ratio =
            scores[&right.candidate_id].weighted_discrimination_nats / right.cost_units as f64;
        right_ratio
            .total_cmp(&left_ratio)
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });

    let mut selected_ids = Vec::new();
    let mut selected_groups = BTreeSet::new();
    let mut spent = 0u32;
    let mut weighted_discrimination_nats = 0.0;
    for candidate in ordered {
        if selected_groups.contains(candidate.correlation_group.as_str()) {
            continue;
        }
        if spent + candidate.cost_units > budget {
            continue;
        }
        spent += candidate.cost_units;
        selected_groups.insert(candidate.correlation_group.as_str());
        selected_ids.push(candidate.candidate_id.clone());
        weighted_discrimination_nats +=
            scores[&candidate.candidate_id].weighted_discrimination_nats;
    }
    selected_ids.sort();
    PlanState {
        weighted_discrimination_nats,
        selected_ids,
    }
}

fn rounded(value: f64) -> f64 {
    let value = (value * 1.0e9).round() / 1.0e9;
    if value == 0.0 { 0.0 } else { value }
}

pub fn evaluate_fixture_bytes(bytes: &[u8]) -> Result<PlanReport, Vec<String>> {
    let fixture: Fixture = serde_json::from_slice(bytes)
        .map_err(|error| vec![format!("fixture JSON rejected: {error}")])?;
    validate_fixture(&fixture)?;
    evaluate_fixture(&fixture, bytes)
}

pub fn evaluate_fixture(fixture: &Fixture, input_bytes: &[u8]) -> Result<PlanReport, Vec<String>> {
    validate_fixture(fixture)?;

    let scores: BTreeMap<String, CandidateScore> = fixture
        .candidates
        .iter()
        .map(|candidate| (candidate.candidate_id.clone(), candidate_score(candidate)))
        .collect();
    let eligible_count = fixture
        .candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .count();
    let ineligible_count = fixture.candidates.len() - eligible_count;
    let selected = select_exact_budgeted(&fixture.candidates, &scores, fixture.budget_cost_units);
    let greedy_baseline =
        select_greedy_ratio(&fixture.candidates, &scores, fixture.budget_cost_units);
    let selected_set: BTreeSet<&str> = selected.selected_ids.iter().map(String::as_str).collect();
    let selected_cost_units = fixture
        .candidates
        .iter()
        .filter(|candidate| selected_set.contains(candidate.candidate_id.as_str()))
        .map(|candidate| candidate.cost_units)
        .sum::<u32>();
    let greedy_baseline_cost_units = fixture
        .candidates
        .iter()
        .filter(|candidate| {
            greedy_baseline
                .selected_ids
                .iter()
                .any(|id| id == &candidate.candidate_id)
        })
        .map(|candidate| candidate.cost_units)
        .sum::<u32>();
    let selected_groups: BTreeSet<&str> = fixture
        .candidates
        .iter()
        .filter(|candidate| selected_set.contains(candidate.candidate_id.as_str()))
        .map(|candidate| candidate.correlation_group.as_str())
        .collect();
    let eligible_strata: BTreeSet<(&str, &str)> = fixture
        .candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .map(|candidate| (candidate.spatial_stratum.as_str(), candidate.temporal_stratum.as_str()))
        .collect();
    let selected_strata: BTreeSet<(&str, &str)> = fixture
        .candidates
        .iter()
        .filter(|candidate| selected_set.contains(candidate.candidate_id.as_str()))
        .map(|candidate| (candidate.spatial_stratum.as_str(), candidate.temporal_stratum.as_str()))
        .collect();
    let eligible_stratum_coverage_fraction = if eligible_strata.is_empty() {
        0.0
    } else {
        selected_strata.len() as f64 / eligible_strata.len() as f64
    };

    let mut decisions = Vec::new();
    for candidate in &fixture.candidates {
        let score = &scores[&candidate.candidate_id];
        let is_selected = selected_set.contains(candidate.candidate_id.as_str());
        let reason = if is_selected {
            None
        } else if candidate.consent_status == ConsentStatus::Denied {
            Some("consent_denied".to_owned())
        } else if candidate.consent_status == ConsentStatus::Unknown {
            Some("consent_unknown".to_owned())
        } else if candidate.access_status == AccessStatus::Denied {
            Some("access_denied".to_owned())
        } else if candidate.access_status == AccessStatus::Unknown {
            Some("access_unknown".to_owned())
        } else if selected_groups.contains(candidate.correlation_group.as_str()) {
            Some("correlation_group_already_selected".to_owned())
        } else if candidate.cost_units > fixture.budget_cost_units {
            Some("candidate_cost_exceeds_budget".to_owned())
        } else {
            Some("not_in_optimal_plan".to_owned())
        };
        decisions.push(CandidateDecision {
            candidate_id: candidate.candidate_id.clone(),
            decision: if is_selected {
                "selected"
            } else {
                "not_selected"
            }
            .to_owned(),
            reason,
            correlation_group: candidate.correlation_group.clone(),
            cost_units: candidate.cost_units,
            symmetric_kl_nats: rounded(score.symmetric_kl_nats),
            weighted_discrimination_nats: rounded(score.weighted_discrimination_nats),
        });
    }
    decisions.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));

    Ok(PlanReport {
        schema_version: REPORT_SCHEMA.to_owned(),
        suite_id: SUITE_ID.to_owned(),
        conformance_status: "PASS_SYNTHETIC_SELECTION_ONLY".to_owned(),
        data_origin: "synthetic_fixture".to_owned(),
        claim_ceiling: CLAIM_CEILING.to_owned(),
        input_sha256: sha256_hex(input_bytes),
        budget_cost_units: fixture.budget_cost_units,
        selected_cost_units,
        unspent_budget_units: fixture.budget_cost_units - selected_cost_units,
        candidate_count: fixture.candidates.len(),
        eligible_candidate_count: eligible_count,
        ineligible_candidate_count: ineligible_count,
        selected_candidate_count: selected.selected_ids.len(),
        eligible_not_selected_count: eligible_count - selected.selected_ids.len(),
        eligible_spatial_temporal_stratum_count: eligible_strata.len(),
        selected_spatial_temporal_stratum_count: selected_strata.len(),
        eligible_spatial_temporal_stratum_coverage_fraction: rounded(
            eligible_stratum_coverage_fraction,
        ),
        selected_candidate_ids: selected.selected_ids,
        total_weighted_discrimination_nats: rounded(selected.weighted_discrimination_nats),
        greedy_ratio_baseline_selected_candidate_ids: greedy_baseline.selected_ids,
        greedy_ratio_baseline_cost_units,
        greedy_ratio_baseline_weighted_discrimination_nats: rounded(
            greedy_baseline.weighted_discrimination_nats,
        ),
        exact_gain_over_greedy_ratio_baseline_nats: rounded(
            selected.weighted_discrimination_nats - greedy_baseline.weighted_discrimination_nats,
        ),
        decisions,
    })
}

pub fn run_fixture_dir(dir: &Path) -> Result<PlanReport, Vec<String>> {
    let fixture_path = dir.join("fixture.json");
    let bytes = fs::read(&fixture_path).map_err(|error| {
        vec![format!(
            "could not read {}: {error}",
            fixture_path.display()
        )]
    })?;
    evaluate_fixture_bytes(&bytes)
}

pub fn default_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/landscape/measurement-planning-v0")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Fixture {
        let bytes = fs::read(default_fixture_dir().join("fixture.json"))
            .expect("frozen measurement fixture");
        serde_json::from_slice(&bytes).expect("fixture JSON")
    }

    #[test]
    fn frozen_fixture_selects_exact_budgeted_plan() {
        let report = run_fixture_dir(&default_fixture_dir()).expect("selection report");
        assert_eq!(report.conformance_status, "PASS_SYNTHETIC_SELECTION_ONLY");
        assert_eq!(report.data_origin, "synthetic_fixture");
        assert_eq!(report.claim_ceiling, CLAIM_CEILING);
        assert_eq!(report.budget_cost_units, 6);
        assert_eq!(report.selected_cost_units, 6);
        assert_eq!(report.unspent_budget_units, 0);
        assert_eq!(report.candidate_count, 6);
        assert_eq!(report.eligible_candidate_count, 4);
        assert_eq!(report.ineligible_candidate_count, 2);
        assert_eq!(report.selected_candidate_count, 2);
        assert_eq!(report.eligible_not_selected_count, 2);
        assert_eq!(report.eligible_spatial_temporal_stratum_count, 3);
        assert_eq!(report.selected_spatial_temporal_stratum_count, 2);
        close(
            report.eligible_spatial_temporal_stratum_coverage_fraction,
            2.0 / 3.0,
        );
        assert_eq!(
            report.selected_candidate_ids,
            vec![
                "candidate-b-north-replica".to_owned(),
                "candidate-c-south-sensor".to_owned()
            ]
        );
        close(report.total_weighted_discrimination_nats, 10.0);
        assert_eq!(
            report.greedy_ratio_baseline_selected_candidate_ids,
            vec!["candidate-a-high-density".to_owned()]
        );
        assert_eq!(report.greedy_ratio_baseline_cost_units, 4);
        close(
            report.greedy_ratio_baseline_weighted_discrimination_nats,
            7.0,
        );
        close(report.exact_gain_over_greedy_ratio_baseline_nats, 3.0);
        assert_eq!(
            report
                .decisions
                .iter()
                .find(|decision| decision.candidate_id == "candidate-e-consent-denied")
                .and_then(|decision| decision.reason.as_deref()),
            Some("consent_denied")
        );
        assert_eq!(
            report
                .decisions
                .iter()
                .find(|decision| decision.candidate_id == "candidate-f-access-unknown")
                .and_then(|decision| decision.reason.as_deref()),
            Some("access_unknown")
        );
    }

    fn brute_force_oracle(
        candidates: &[Candidate],
        scores: &BTreeMap<String, CandidateScore>,
        budget: u32,
    ) -> (u32, PlanState) {
        assert!(candidates.len() <= 20);
        let empty = PlanState {
            weighted_discrimination_nats: 0.0,
            selected_ids: Vec::new(),
        };
        let mut best_cost = 0;
        let mut best = empty;
        for mask in 0usize..(1usize << candidates.len()) {
            let mut ids = Vec::new();
            let mut groups = BTreeSet::new();
            let mut cost = 0u32;
            let mut score = 0.0;
            let mut feasible = true;
            for (index, candidate) in candidates.iter().enumerate() {
                if mask & (1usize << index) == 0 {
                    continue;
                }
                if !eligible(candidate) || !groups.insert(candidate.correlation_group.as_str()) {
                    feasible = false;
                    break;
                }
                cost += candidate.cost_units;
                if cost > budget {
                    feasible = false;
                    break;
                }
                score += scores[&candidate.candidate_id].weighted_discrimination_nats;
                ids.push(candidate.candidate_id.clone());
            }
            if !feasible {
                continue;
            }
            ids.sort();
            let score_order = score.total_cmp(&best.weighted_discrimination_nats);
            let should_replace = match score_order {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Less => false,
                std::cmp::Ordering::Equal if cost < best_cost => true,
                std::cmp::Ordering::Equal if cost > best_cost => false,
                std::cmp::Ordering::Equal => {
                    ids.len() < best.selected_ids.len()
                        || (ids.len() == best.selected_ids.len() && ids < best.selected_ids)
                }
            };
            if should_replace {
                best_cost = cost;
                best = PlanState {
                    weighted_discrimination_nats: score,
                    selected_ids: ids,
                };
            }
        }
        (best_cost, best)
    }

    #[test]
    fn exact_dynamic_program_matches_exhaustive_oracle_across_small_problem_family() {
        for seed in 0..24usize {
            let mut fixture = fixture();
            fixture.budget_cost_units = (1 + (seed % 10)) as u32;
            for (index, candidate) in fixture.candidates.iter_mut().enumerate() {
                let delta = ((seed + index * 3) % 7) as f64;
                candidate.baseline_mean = 20.0 + index as f64;
                candidate.alternative_mean = candidate.baseline_mean + delta;
                candidate.decision_weight = ((seed + index) % 4) as f64;
                candidate.cost_units = (1 + ((seed * 3 + index * 2) % 5)) as u32;
                candidate.correlation_group =
                    format!("group-{}", (seed + index * 2) % 3);
            }
            validate_fixture(&fixture).expect("generated synthetic instance stays valid");
            let scores: BTreeMap<String, CandidateScore> = fixture
                .candidates
                .iter()
                .map(|candidate| {
                    (
                        candidate.candidate_id.clone(),
                        candidate_score(candidate),
                    )
                })
                .collect();
            let actual = select_exact_budgeted(
                &fixture.candidates,
                &scores,
                fixture.budget_cost_units,
            );
            let (oracle_cost, oracle) =
                brute_force_oracle(&fixture.candidates, &scores, fixture.budget_cost_units);
            let actual_cost: u32 = fixture
                .candidates
                .iter()
                .filter(|candidate| actual.selected_ids.contains(&candidate.candidate_id))
                .map(|candidate| candidate.cost_units)
                .sum();
            assert_eq!(actual_cost, oracle_cost, "seed={seed}");
            assert_eq!(actual.selected_ids, oracle.selected_ids, "seed={seed}");
            close(
                actual.weighted_discrimination_nats,
                oracle.weighted_discrimination_nats,
            );
        }
    }

    #[test]
    fn symmetric_kl_known_answer_is_nine_nats() {
        let mut candidate = fixture().candidates[0].clone();
        candidate.baseline_mean = 0.0;
        candidate.alternative_mean = 6.0;
        candidate.baseline_sd = 1.0;
        candidate.alternative_sd = 1.0;
        candidate.observation_noise_sd = 1.0;
        close(symmetric_kl_nats(&candidate), 9.0);
    }

    #[test]
    fn denied_consent_and_unknown_access_never_get_selected() {
        let report = run_fixture_dir(&default_fixture_dir()).expect("selection report");
        assert!(
            !report
                .selected_candidate_ids
                .contains(&"candidate-e-consent-denied".to_owned())
        );
        assert!(
            !report
                .selected_candidate_ids
                .contains(&"candidate-f-access-unknown".to_owned())
        );
    }

    #[test]
    fn denied_access_and_unknown_consent_fail_closed() {
        let mut denied_access = fixture();
        denied_access.candidates[0].access_status = AccessStatus::Denied;
        let report = evaluate_fixture(&denied_access, b"synthetic").expect("valid access control case");
        assert!(
            !report
                .selected_candidate_ids
                .contains(&"candidate-a-high-density".to_owned())
        );
        assert_eq!(
            report
                .decisions
                .iter()
                .find(|decision| decision.candidate_id == "candidate-a-high-density")
                .and_then(|decision| decision.reason.as_deref()),
            Some("access_denied")
        );

        let mut unknown_consent = fixture();
        unknown_consent.candidates[0].consent_status = ConsentStatus::Unknown;
        let report =
            evaluate_fixture(&unknown_consent, b"synthetic").expect("valid consent control case");
        assert!(
            !report
                .selected_candidate_ids
                .contains(&"candidate-a-high-density".to_owned())
        );
        assert_eq!(
            report
                .decisions
                .iter()
                .find(|decision| decision.candidate_id == "candidate-a-high-density")
                .and_then(|decision| decision.reason.as_deref()),
            Some("consent_unknown")
        );
    }

    #[test]
    fn duplicate_ids_and_real_data_labels_fail_closed() {
        let mut duplicate = fixture();
        duplicate.candidates[1].candidate_id = duplicate.candidates[0].candidate_id.clone();
        assert!(validate_fixture(&duplicate).is_err());

        let mut relabelled = fixture();
        relabelled.data_origin = "field_observations".to_owned();
        assert!(validate_fixture(&relabelled).is_err());
    }

    #[test]
    fn selection_respects_budget_and_correlation_groups() {
        let report = run_fixture_dir(&default_fixture_dir()).expect("selection report");
        assert!(report.selected_cost_units <= report.budget_cost_units);
        let groups: BTreeSet<&str> = report
            .decisions
            .iter()
            .filter(|decision| decision.decision == "selected")
            .map(|decision| decision.correlation_group.as_str())
            .collect();
        assert_eq!(groups.len(), report.selected_candidate_count);
    }

    #[test]
    fn report_is_deterministic_for_identical_fixture_bytes() {
        let bytes = fs::read(default_fixture_dir().join("fixture.json")).expect("fixture bytes");
        let first = evaluate_fixture_bytes(&bytes).expect("first report");
        let second = evaluate_fixture_bytes(&bytes).expect("second report");
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
        assert_eq!(first.input_sha256, sha256_hex(&bytes));
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1.0e-8,
            "actual={actual}, expected={expected}"
        );
    }
}
