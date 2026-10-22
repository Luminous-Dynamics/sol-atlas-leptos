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
const MAX_COVERAGE_STRATA: usize = 20;
const MAX_COVERAGE_DP_STATES: usize = 100_000;
const MAX_COVERAGE_DP_TRANSITIONS: usize = 2_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub schema_version: String,
    pub suite_id: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub budget_cost_units: u32,
    /// Minimum fraction of consent/access-eligible spatial-temporal strata the
    /// selected campaign must cover. Zero disables the constraint.
    pub minimum_spatial_temporal_coverage_fraction: f64,
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
    pub model_family_id: String,
    pub model_version: String,
    pub baseline_scenario_sha256: String,
    pub alternative_scenario_sha256: String,
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
    pub consent_evidence_sha256: Option<String>,
    pub access_authorization_sha256: Option<String>,
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
    pub consent_status: ConsentStatus,
    pub access_status: AccessStatus,
    pub consent_evidence_sha256: Option<String>,
    pub access_authorization_sha256: Option<String>,
    pub correlation_group: String,
    pub cost_units: u32,
    pub symmetric_kl_nats: f64,
    pub weighted_discrimination_nats: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoverageFrontierPoint {
    pub minimum_coverage_fraction: f64,
    pub required_spatial_temporal_stratum_count: usize,
    pub status: String,
    pub selected_candidate_ids: Option<Vec<String>>,
    pub selected_cost_units: Option<u32>,
    pub achieved_coverage_fraction: Option<f64>,
    pub weighted_discrimination_nats: Option<f64>,
    pub score_loss_from_unconstrained_nats: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanReport {
    pub schema_version: String,
    pub suite_id: String,
    pub conformance_status: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub input_sha256: String,
    pub baseline_scenario_sha256: String,
    pub alternative_scenario_sha256: String,
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
    pub selected_spatial_temporal_stratum_coverage_fraction: f64,
    pub minimum_spatial_temporal_coverage_fraction: f64,
    pub required_spatial_temporal_stratum_count: usize,
    pub coverage_constraint_status: String,
    pub coverage_constraint_limit_reason: Option<String>,
    pub coverage_frontier_status: String,
    pub coverage_frontier_limit_reason: Option<String>,
    pub coverage_frontier: Vec<CoverageFrontierPoint>,
    pub coverage_constrained_selected_candidate_ids: Option<Vec<String>>,
    pub coverage_constrained_cost_units: Option<u32>,
    pub coverage_constrained_stratum_coverage_fraction: Option<f64>,
    pub coverage_constrained_weighted_discrimination_nats: Option<f64>,
    pub coverage_constraint_score_loss_nats: Option<f64>,
    pub selected_candidate_ids: Vec<String>,
    pub total_weighted_discrimination_nats: f64,
    pub greedy_ratio_baseline_selected_candidate_ids: Vec<String>,
    pub greedy_ratio_baseline_cost_units: u32,
    pub greedy_ratio_baseline_weighted_discrimination_nats: f64,
    pub exact_gain_over_greedy_ratio_baseline_nats: f64,
    pub uniform_baseline_replicates: usize,
    pub uniform_baseline_mean_weighted_discrimination_nats: f64,
    pub uniform_baseline_best_weighted_discrimination_nats: f64,
    pub uniform_baseline_mean_stratum_coverage_fraction: f64,
    pub exact_gain_over_uniform_mean_nats: f64,
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
    if !fixture
        .minimum_spatial_temporal_coverage_fraction
        .is_finite()
        || !(0.0..=1.0).contains(&fixture.minimum_spatial_temporal_coverage_fraction)
    {
        errors.push(
            "minimum_spatial_temporal_coverage_fraction must be finite and in [0, 1]".to_owned(),
        );
    }
    let mut ids = BTreeSet::new();
    let mut model_by_metric: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    if let Some(first) = fixture.candidates.first() {
        for candidate in &fixture.candidates {
            if candidate.baseline_scenario_sha256 != first.baseline_scenario_sha256
                || candidate.alternative_scenario_sha256 != first.alternative_scenario_sha256
            {
                errors.push(format!(
                    "{}: all candidates must compare the same pinned baseline/alternative scenario pair",
                    candidate.candidate_id
                ));
            }
            let key = candidate.metric_id.clone();
            let family = (
                candidate.unit.clone(),
                candidate.model_family_id.clone(),
                candidate.model_version.clone(),
            );
            if let Some(existing) = model_by_metric.get(&key) {
                if existing != &family {
                    errors.push(format!(
                        "{}: unit/model-family/model-version contract differs within metric {}",
                        candidate.candidate_id, key
                    ));
                }
            } else {
                model_by_metric.insert(key, family);
            }
        }
    }
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
            ("model_family_id", candidate.model_family_id.as_str()),
            ("model_version", candidate.model_version.as_str()),
        ] {
            if !nonempty(value) {
                errors.push(format!(
                    "{}: {name} must be non-empty",
                    candidate.candidate_id
                ));
            }
        }
        if !valid_digest(&candidate.site_ref_sha256) {
            errors.push(format!(
                "{}: site_ref_sha256 must be 64 lowercase hexadecimal characters",
                candidate.candidate_id
            ));
        }
        if !valid_digest(&candidate.baseline_scenario_sha256) {
            errors.push(format!(
                "{}: baseline_scenario_sha256 must be 64 lowercase hexadecimal characters",
                candidate.candidate_id
            ));
        }
        if !valid_digest(&candidate.alternative_scenario_sha256) {
            errors.push(format!(
                "{}: alternative_scenario_sha256 must be 64 lowercase hexadecimal characters",
                candidate.candidate_id
            ));
        }
        if candidate.baseline_scenario_sha256 == candidate.alternative_scenario_sha256 {
            errors.push(format!(
                "{}: baseline and alternative scenario digests must differ",
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
        match (
            candidate.consent_status,
            candidate.consent_evidence_sha256.as_deref(),
        ) {
            (ConsentStatus::Granted, Some(digest)) if valid_digest(digest) => {}
            (ConsentStatus::Denied | ConsentStatus::Unknown, None) => {}
            (ConsentStatus::Granted, _) => errors.push(format!(
                "{}: granted consent requires a valid consent_evidence_sha256",
                candidate.candidate_id
            )),
            (ConsentStatus::Denied | ConsentStatus::Unknown, Some(_)) => errors.push(format!(
                "{}: denied/unknown consent must not claim consent evidence",
                candidate.candidate_id
            )),
        }
        match (
            candidate.access_status,
            candidate.access_authorization_sha256.as_deref(),
        ) {
            (AccessStatus::Authorized, Some(digest)) if valid_digest(digest) => {}
            (AccessStatus::Denied | AccessStatus::Unknown, None) => {}
            (AccessStatus::Authorized, _) => errors.push(format!(
                "{}: authorized access requires a valid access_authorization_sha256",
                candidate.candidate_id
            )),
            (AccessStatus::Denied | AccessStatus::Unknown, Some(_)) => errors.push(format!(
                "{}: denied/unknown access must not claim authorization evidence",
                candidate.candidate_id
            )),
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

fn eligible_strata_count(candidates: &[Candidate]) -> usize {
    candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .map(|candidate| {
            (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            )
        })
        .collect::<BTreeSet<_>>()
        .len()
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

fn select_exact_coverage_frontier_plans(
    candidates: &[Candidate],
    scores: &BTreeMap<String, CandidateScore>,
    budget: u32,
) -> Result<Vec<Option<(u32, PlanState)>>, &'static str> {
    let eligible_strata: BTreeSet<(&str, &str)> = candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .map(|candidate| {
            (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            )
        })
        .collect();
    let stratum_count = eligible_strata.len();
    let mut frontier: Vec<Option<(u32, PlanState)>> = vec![None; stratum_count + 1];
    if stratum_count == 0 {
        return Ok(frontier);
    }
    if stratum_count > MAX_COVERAGE_STRATA {
        return Err("coverage_stratum_limit");
    }

    let stratum_indexes: BTreeMap<(&str, &str), usize> = eligible_strata
        .iter()
        .enumerate()
        .map(|(index, stratum)| (*stratum, index))
        .collect();

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

    // One exact state map generates the entire coverage-versus-score frontier.
    // The fixed state/transition caps are explicit resource bounds; exceeding
    // either cap returns a reason instead of a partial or falsely qualified plan.
    let mut states: BTreeMap<(u32, u64), PlanState> = BTreeMap::new();
    states.insert(
        (0, 0),
        PlanState {
            weighted_discrimination_nats: 0.0,
            selected_ids: Vec::new(),
        },
    );
    let mut transitions = 0usize;
    for candidates_in_group in by_group.values() {
        let previous = states.clone();
        let mut next = previous.clone();
        for ((spent, covered_mask), state) in &previous {
            for candidate in candidates_in_group {
                transitions += 1;
                if transitions > MAX_COVERAGE_DP_TRANSITIONS {
                    return Err("coverage_transition_limit");
                }
                let new_cost = *spent + candidate.cost_units;
                if new_cost > budget {
                    continue;
                }
                let stratum = (
                    candidate.spatial_stratum.as_str(),
                    candidate.temporal_stratum.as_str(),
                );
                let bit = 1u64 << stratum_indexes[&stratum];
                let new_mask = *covered_mask | bit;
                let mut selected_ids = state.selected_ids.clone();
                selected_ids.push(candidate.candidate_id.clone());
                selected_ids.sort();
                let proposed = PlanState {
                    weighted_discrimination_nats: state.weighted_discrimination_nats
                        + scores[&candidate.candidate_id].weighted_discrimination_nats,
                    selected_ids,
                };
                let slot = next
                    .entry((new_cost, new_mask))
                    .or_insert_with(|| proposed.clone());
                if better_for_same_cost(&proposed, slot) {
                    *slot = proposed;
                }
                if next.len() > MAX_COVERAGE_DP_STATES {
                    return Err("coverage_state_limit");
                }
            }
        }
        states = next;
        if states.len() > MAX_COVERAGE_DP_STATES {
            return Err("coverage_state_limit");
        }
    }

    // First retain the best plan with exactly k covered strata, then compute
    // suffix optima: frontier[k] is the best plan with coverage >= k strata.
    let mut best_exact: Vec<Option<(u32, PlanState)>> = vec![None; stratum_count + 1];
    for ((cost, mask), state) in &states {
        let covered = mask.count_ones() as usize;
        if covered == 0 {
            continue;
        }
        let slot = &mut best_exact[covered];
        if slot.as_ref().is_none_or(|(best_cost, best_state)| {
            better_final(*cost, state, *best_cost, best_state)
        }) {
            *slot = Some((*cost, state.clone()));
        }
    }
    for minimum_strata in (1..=stratum_count).rev() {
        let exact = best_exact[minimum_strata].clone();
        let higher = if minimum_strata < stratum_count {
            frontier[minimum_strata + 1].clone()
        } else {
            None
        };
        frontier[minimum_strata] = match (exact, higher) {
            (Some(left), Some(right)) => {
                if better_final(left.0, &left.1, right.0, &right.1) {
                    Some(left)
                } else {
                    Some(right)
                }
            }
            (Some(plan), None) | (None, Some(plan)) => Some(plan),
            (None, None) => None,
        };
    }
    Ok(frontier)
}

fn select_exact_coverage_constrained(
    candidates: &[Candidate],
    scores: &BTreeMap<String, CandidateScore>,
    budget: u32,
    minimum_coverage_fraction: f64,
) -> Result<Option<PlanState>, &'static str> {
    if minimum_coverage_fraction == 0.0 {
        return Ok(None);
    }
    let stratum_count = eligible_strata_count(candidates);
    if stratum_count == 0 {
        return Ok(None);
    }
    let required_strata =
        ((minimum_coverage_fraction * stratum_count as f64) - 1.0e-12).ceil() as usize;
    let frontier = select_exact_coverage_frontier_plans(candidates, scores, budget)?;
    Ok(frontier
        .get(required_strata)
        .cloned()
        .flatten()
        .map(|(_, plan)| plan))
}

fn select_in_order(
    ordered: Vec<&Candidate>,
    scores: &BTreeMap<String, CandidateScore>,
    budget: u32,
) -> PlanState {
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
    select_in_order(ordered, scores, budget)
}

fn select_seeded_uniform(
    candidates: &[Candidate],
    scores: &BTreeMap<String, CandidateScore>,
    budget: u32,
    seed: u64,
) -> PlanState {
    let mut ordered: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .collect();
    ordered.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    for index in (1..ordered.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let other = (state as usize) % (index + 1);
        ordered.swap(index, other);
    }
    select_in_order(ordered, scores, budget)
}

fn build_coverage_frontier_points(
    candidates: &[Candidate],
    plans: &[Option<(u32, PlanState)>],
    unconstrained_score: f64,
) -> Vec<CoverageFrontierPoint> {
    let total_strata = eligible_strata_count(candidates);
    if total_strata == 0 {
        return Vec::new();
    }
    (1..=total_strata)
        .map(|minimum_strata| {
            let minimum_fraction = minimum_strata as f64 / total_strata as f64;
            match plans.get(minimum_strata).cloned().flatten() {
                Some((cost, plan)) => CoverageFrontierPoint {
                    minimum_coverage_fraction: rounded(minimum_fraction),
                    required_spatial_temporal_stratum_count: minimum_strata,
                    status: "satisfied".to_owned(),
                    selected_candidate_ids: Some(plan.selected_ids.clone()),
                    selected_cost_units: Some(cost),
                    achieved_coverage_fraction: Some(rounded(stratum_coverage_fraction(
                        candidates,
                        &plan.selected_ids,
                    ))),
                    weighted_discrimination_nats: Some(rounded(plan.weighted_discrimination_nats)),
                    score_loss_from_unconstrained_nats: Some(rounded(
                        (unconstrained_score - plan.weighted_discrimination_nats).max(0.0),
                    )),
                },
                None => CoverageFrontierPoint {
                    minimum_coverage_fraction: rounded(minimum_fraction),
                    required_spatial_temporal_stratum_count: minimum_strata,
                    status: "infeasible".to_owned(),
                    selected_candidate_ids: None,
                    selected_cost_units: None,
                    achieved_coverage_fraction: None,
                    weighted_discrimination_nats: None,
                    score_loss_from_unconstrained_nats: None,
                },
            }
        })
        .collect()
}

fn stratum_coverage_fraction(candidates: &[Candidate], selected_ids: &[String]) -> f64 {
    let eligible_strata: BTreeSet<(&str, &str)> = candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .map(|candidate| {
            (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            )
        })
        .collect();
    let selected: BTreeSet<&str> = selected_ids.iter().map(String::as_str).collect();
    let selected_strata: BTreeSet<(&str, &str)> = candidates
        .iter()
        .filter(|candidate| selected.contains(candidate.candidate_id.as_str()))
        .map(|candidate| {
            (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            )
        })
        .collect();
    if eligible_strata.is_empty() {
        0.0
    } else {
        selected_strata.len() as f64 / eligible_strata.len() as f64
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
    let eligible_strata_total = eligible_strata_count(&fixture.candidates);
    let required_spatial_temporal_strata = ((fixture.minimum_spatial_temporal_coverage_fraction
        * eligible_strata_total as f64)
        - 1.0e-12)
        .ceil() as usize;
    let coverage_plan_search = select_exact_coverage_frontier_plans(
        &fixture.candidates,
        &scores,
        fixture.budget_cost_units,
    );
    let (coverage_frontier_status, coverage_frontier_limit_reason, coverage_frontier) =
        match &coverage_plan_search {
            Ok(plans) => (
                "complete".to_owned(),
                None,
                build_coverage_frontier_points(
                    &fixture.candidates,
                    plans,
                    selected.weighted_discrimination_nats,
                ),
            ),
            Err(reason) => (
                "state_limit_exceeded".to_owned(),
                Some((*reason).to_owned()),
                Vec::new(),
            ),
        };
    let (
        coverage_constraint_status,
        coverage_constraint_limit_reason,
        coverage_constrained,
        coverage_constrained_cost,
    ) = if fixture.minimum_spatial_temporal_coverage_fraction == 0.0 {
        ("not_required".to_owned(), None, None, None)
    } else {
        match &coverage_plan_search {
            Err(reason) => (
                "state_limit_exceeded".to_owned(),
                Some((*reason).to_owned()),
                None,
                None,
            ),
            Ok(plans) => match plans
                .get(required_spatial_temporal_strata)
                .cloned()
                .flatten()
            {
                Some((cost, plan)) => ("satisfied".to_owned(), None, Some(plan), Some(cost)),
                None => ("infeasible".to_owned(), None, None, None),
            },
        }
    };
    let coverage_constrained_strata: BTreeSet<(&str, &str)> = coverage_constrained
        .as_ref()
        .map(|plan| {
            let selected_ids: BTreeSet<&str> =
                plan.selected_ids.iter().map(String::as_str).collect();
            fixture
                .candidates
                .iter()
                .filter(|candidate| selected_ids.contains(candidate.candidate_id.as_str()))
                .map(|candidate| {
                    (
                        candidate.spatial_stratum.as_str(),
                        candidate.temporal_stratum.as_str(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let coverage_constrained_coverage_fraction = if eligible_strata.is_empty() {
        0.0
    } else {
        coverage_constrained_strata.len() as f64 / eligible_strata.len() as f64
    };
    let greedy_baseline =
        select_greedy_ratio(&fixture.candidates, &scores, fixture.budget_cost_units);
    const UNIFORM_REPLICATES: u64 = 64;
    let uniform_plans: Vec<PlanState> = (0..UNIFORM_REPLICATES)
        .map(|seed| {
            select_seeded_uniform(
                &fixture.candidates,
                &scores,
                fixture.budget_cost_units,
                seed,
            )
        })
        .collect();
    let uniform_mean_score = uniform_plans
        .iter()
        .map(|plan| plan.weighted_discrimination_nats)
        .sum::<f64>()
        / UNIFORM_REPLICATES as f64;
    let uniform_best_score = uniform_plans
        .iter()
        .map(|plan| plan.weighted_discrimination_nats)
        .fold(0.0_f64, f64::max);
    let uniform_mean_coverage = uniform_plans
        .iter()
        .map(|plan| stratum_coverage_fraction(&fixture.candidates, &plan.selected_ids))
        .sum::<f64>()
        / UNIFORM_REPLICATES as f64;
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
        .map(|candidate| {
            (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            )
        })
        .collect();
    let selected_strata: BTreeSet<(&str, &str)> = fixture
        .candidates
        .iter()
        .filter(|candidate| selected_set.contains(candidate.candidate_id.as_str()))
        .map(|candidate| {
            (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            )
        })
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
            consent_status: candidate.consent_status,
            access_status: candidate.access_status,
            consent_evidence_sha256: candidate.consent_evidence_sha256.clone(),
            access_authorization_sha256: candidate.access_authorization_sha256.clone(),
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
        baseline_scenario_sha256: fixture.candidates[0].baseline_scenario_sha256.clone(),
        alternative_scenario_sha256: fixture.candidates[0].alternative_scenario_sha256.clone(),
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
        selected_spatial_temporal_stratum_coverage_fraction: rounded(
            eligible_stratum_coverage_fraction,
        ),
        minimum_spatial_temporal_coverage_fraction: rounded(
            fixture.minimum_spatial_temporal_coverage_fraction,
        ),
        required_spatial_temporal_stratum_count: required_spatial_temporal_strata,
        coverage_constraint_status,
        coverage_constraint_limit_reason,
        coverage_frontier_status,
        coverage_frontier_limit_reason,
        coverage_frontier,
        coverage_constrained_selected_candidate_ids: coverage_constrained
            .as_ref()
            .map(|plan| plan.selected_ids.clone()),
        coverage_constrained_cost_units: coverage_constrained_cost,
        coverage_constrained_stratum_coverage_fraction: coverage_constrained
            .as_ref()
            .map(|_| rounded(coverage_constrained_coverage_fraction)),
        coverage_constrained_weighted_discrimination_nats: coverage_constrained
            .as_ref()
            .map(|plan| rounded(plan.weighted_discrimination_nats)),
        coverage_constraint_score_loss_nats: coverage_constrained.as_ref().map(|plan| {
            rounded(
                (selected.weighted_discrimination_nats - plan.weighted_discrimination_nats)
                    .max(0.0),
            )
        }),
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
        uniform_baseline_replicates: UNIFORM_REPLICATES as usize,
        uniform_baseline_mean_weighted_discrimination_nats: rounded(uniform_mean_score),
        uniform_baseline_best_weighted_discrimination_nats: rounded(uniform_best_score),
        uniform_baseline_mean_stratum_coverage_fraction: rounded(uniform_mean_coverage),
        exact_gain_over_uniform_mean_nats: rounded(
            selected.weighted_discrimination_nats - uniform_mean_score,
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
        assert_eq!(report.baseline_scenario_sha256, "1".repeat(64));
        assert_eq!(report.alternative_scenario_sha256, "2".repeat(64));
        assert_eq!(report.selected_cost_units, 4);
        assert_eq!(report.unspent_budget_units, 2);
        assert_eq!(report.candidate_count, 6);
        assert_eq!(report.eligible_candidate_count, 4);
        assert_eq!(report.ineligible_candidate_count, 2);
        assert_eq!(report.selected_candidate_count, 1);
        assert_eq!(report.eligible_not_selected_count, 3);
        assert_eq!(report.eligible_spatial_temporal_stratum_count, 3);
        assert_eq!(report.selected_spatial_temporal_stratum_count, 1);
        close(
            report.selected_spatial_temporal_stratum_coverage_fraction,
            1.0 / 3.0,
        );
        assert_eq!(report.minimum_spatial_temporal_coverage_fraction, 2.0 / 3.0);
        assert_eq!(report.required_spatial_temporal_stratum_count, 2);
        assert_eq!(report.coverage_constraint_status, "satisfied");
        assert_eq!(report.coverage_constraint_limit_reason, None);
        assert_eq!(report.coverage_frontier_status, "complete");
        assert_eq!(report.coverage_frontier_limit_reason, None);
        assert_eq!(report.coverage_frontier.len(), 3);
        assert_eq!(
            report.coverage_frontier[0].required_spatial_temporal_stratum_count,
            1
        );
        assert_eq!(report.coverage_frontier[0].status, "satisfied");
        assert_eq!(
            report.coverage_frontier[0].selected_candidate_ids,
            Some(vec!["candidate-a-high-density".to_owned()])
        );
        close(
            report.coverage_frontier[0]
                .weighted_discrimination_nats
                .unwrap(),
            11.0,
        );
        close(
            report.coverage_frontier[0]
                .achieved_coverage_fraction
                .unwrap(),
            1.0 / 3.0,
        );
        close(
            report.coverage_frontier[0]
                .score_loss_from_unconstrained_nats
                .unwrap(),
            0.0,
        );
        assert_eq!(
            report.coverage_frontier[1].required_spatial_temporal_stratum_count,
            2
        );
        assert_eq!(report.coverage_frontier[1].status, "satisfied");
        assert_eq!(
            report.coverage_frontier[1].selected_candidate_ids,
            Some(vec![
                "candidate-b-north-replica".to_owned(),
                "candidate-c-south-sensor".to_owned()
            ])
        );
        close(
            report.coverage_frontier[1]
                .weighted_discrimination_nats
                .unwrap(),
            10.0,
        );
        close(
            report.coverage_frontier[1]
                .score_loss_from_unconstrained_nats
                .unwrap(),
            1.0,
        );
        assert_eq!(
            report.coverage_frontier[2].required_spatial_temporal_stratum_count,
            3
        );
        assert_eq!(report.coverage_frontier[2].status, "infeasible");
        assert!(report.coverage_frontier[2].selected_candidate_ids.is_none());
        assert_eq!(
            report.selected_candidate_ids,
            vec!["candidate-a-high-density".to_owned()]
        );
        close(report.total_weighted_discrimination_nats, 11.0);
        assert_eq!(
            report.coverage_constrained_selected_candidate_ids,
            Some(vec![
                "candidate-b-north-replica".to_owned(),
                "candidate-c-south-sensor".to_owned()
            ])
        );
        assert_eq!(report.coverage_constrained_cost_units, Some(6));
        close(
            report
                .coverage_constrained_stratum_coverage_fraction
                .unwrap(),
            2.0 / 3.0,
        );
        close(
            report
                .coverage_constrained_weighted_discrimination_nats
                .unwrap(),
            10.0,
        );
        close(report.coverage_constraint_score_loss_nats.unwrap(), 1.0);
        assert_eq!(
            report.greedy_ratio_baseline_selected_candidate_ids,
            vec!["candidate-a-high-density".to_owned()]
        );
        assert_eq!(report.greedy_ratio_baseline_cost_units, 4);
        close(
            report.greedy_ratio_baseline_weighted_discrimination_nats,
            11.0,
        );
        close(report.exact_gain_over_greedy_ratio_baseline_nats, 0.0);
        assert_eq!(report.uniform_baseline_replicates, 64);
        close(
            report.uniform_baseline_mean_weighted_discrimination_nats,
            10.40625,
        );
        close(
            report.uniform_baseline_best_weighted_discrimination_nats,
            11.0,
        );
        close(
            report.uniform_baseline_mean_stratum_coverage_fraction,
            0.53125,
        );
        close(report.exact_gain_over_uniform_mean_nats, 0.59375);
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

    fn brute_force_coverage_oracle(
        candidates: &[Candidate],
        scores: &BTreeMap<String, CandidateScore>,
        budget: u32,
        minimum_coverage_fraction: f64,
    ) -> Option<(u32, PlanState)> {
        let eligible_strata: BTreeSet<(&str, &str)> = candidates
            .iter()
            .filter(|candidate| eligible(candidate))
            .map(|candidate| (
                candidate.spatial_stratum.as_str(),
                candidate.temporal_stratum.as_str(),
            ))
            .collect();
        if eligible_strata.is_empty() {
            return None;
        }
        let required_strata =
            ((minimum_coverage_fraction * eligible_strata.len() as f64) - 1.0e-12).ceil() as usize;
        let mut best: Option<(u32, PlanState)> = None;
        for mask in 0usize..(1usize << candidates.len()) {
            let mut selected_ids = Vec::new();
            let mut selected_groups = BTreeSet::new();
            let mut selected_strata = BTreeSet::new();
            let mut cost = 0u32;
            let mut score = 0.0;
            let mut feasible = true;
            for (index, candidate) in candidates.iter().enumerate() {
                if mask & (1usize << index) == 0 {
                    continue;
                }
                if !eligible(candidate)
                    || !selected_groups.insert(candidate.correlation_group.as_str())
                {
                    feasible = false;
                    break;
                }
                cost += candidate.cost_units;
                if cost > budget {
                    feasible = false;
                    break;
                }
                selected_strata.insert((
                    candidate.spatial_stratum.as_str(),
                    candidate.temporal_stratum.as_str(),
                ));
                score += scores[&candidate.candidate_id].weighted_discrimination_nats;
                selected_ids.push(candidate.candidate_id.clone());
            }
            if !feasible || selected_strata.len() < required_strata {
                continue;
            }
            selected_ids.sort();
            let proposed = PlanState {
                weighted_discrimination_nats: score,
                selected_ids,
            };
            let replace = best.as_ref().is_none_or(|(best_cost, best_plan)| {
                better_final(cost, &proposed, *best_cost, best_plan)
            });
            if replace {
                best = Some((cost, proposed));
            }
        }
        best
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
                candidate.correlation_group = format!("group-{}", (seed + index * 2) % 3);
                candidate.spatial_stratum = format!("space-{}", (seed + index * 3) % 4);
                candidate.temporal_stratum = format!("time-{}", (seed * 2 + index) % 3);
                candidate.consent_status = match (seed + index) % 5 {
                    0 => ConsentStatus::Denied,
                    1 => ConsentStatus::Unknown,
                    _ => ConsentStatus::Granted,
                };
                candidate.consent_evidence_sha256 = match candidate.consent_status {
                    ConsentStatus::Granted => Some("3".repeat(64)),
                    ConsentStatus::Denied | ConsentStatus::Unknown => None,
                };
                candidate.access_status = match (seed * 2 + index) % 5 {
                    0 => AccessStatus::Denied,
                    1 => AccessStatus::Unknown,
                    _ => AccessStatus::Authorized,
                };
                candidate.access_authorization_sha256 = match candidate.access_status {
                    AccessStatus::Authorized => Some("4".repeat(64)),
                    AccessStatus::Denied | AccessStatus::Unknown => None,
                };
            }
            validate_fixture(&fixture).expect("generated synthetic instance stays valid");
            let scores: BTreeMap<String, CandidateScore> = fixture
                .candidates
                .iter()
                .map(|candidate| (candidate.candidate_id.clone(), candidate_score(candidate)))
                .collect();
            let actual =
                select_exact_budgeted(&fixture.candidates, &scores, fixture.budget_cost_units);
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

            for minimum_coverage in [1.0 / 3.0, 2.0 / 3.0, 1.0] {
                let constrained = select_exact_coverage_constrained(
                    &fixture.candidates,
                    &scores,
                    fixture.budget_cost_units,
                    minimum_coverage,
                )
                .expect("small generated coverage DP remains within state limits");
                let coverage_oracle = brute_force_coverage_oracle(
                    &fixture.candidates,
                    &scores,
                    fixture.budget_cost_units,
                    minimum_coverage,
                );
                match (constrained, coverage_oracle) {
                    (Some(actual), Some((actual_cost, expected))) => {
                        let expected_cost: u32 = fixture
                            .candidates
                            .iter()
                            .filter(|candidate| {
                                actual.selected_ids.contains(&candidate.candidate_id)
                            })
                            .map(|candidate| candidate.cost_units)
                            .sum();
                        assert_eq!(expected_cost, actual_cost, "coverage seed={seed}");
                        assert_eq!(
                            actual.selected_ids, expected.selected_ids,
                            "coverage seed={seed}"
                        );
                        close(
                            actual.weighted_discrimination_nats,
                            expected.weighted_discrimination_nats,
                        );
                    }
                    (None, None) => {}
                    (actual, expected) => panic!(
                        "coverage feasibility mismatch seed={seed}, minimum={minimum_coverage}: actual={}, expected={}",
                        actual.is_some(),
                        expected.is_some()
                    ),
                }
            }
        }
    }

    #[test]
    fn coverage_constraint_reports_infeasible_when_budget_cannot_cover_all_strata() {
        let mut fixture = fixture();
        fixture.minimum_spatial_temporal_coverage_fraction = 1.0;
        let report = evaluate_fixture(&fixture, b"synthetic").expect("valid infeasible request");
        assert_eq!(report.coverage_constraint_status, "infeasible");
        assert_eq!(report.coverage_constraint_limit_reason, None);
        assert_eq!(report.coverage_frontier_status, "complete");
        assert_eq!(report.coverage_frontier[2].status, "infeasible");
        assert_eq!(report.required_spatial_temporal_stratum_count, 3);
        assert!(report.coverage_constrained_selected_candidate_ids.is_none());
        assert!(report.coverage_constraint_score_loss_nats.is_none());
        // The unconstrained recommendation remains visible but is never
        // represented as satisfying the requested coverage constraint.
        assert_eq!(
            report.selected_candidate_ids,
            vec!["candidate-a-high-density".to_owned()]
        );
        close(
            report.selected_spatial_temporal_stratum_coverage_fraction,
            1.0 / 3.0,
        );
    }

    #[test]
    fn coverage_state_limit_is_reported_without_claiming_constraint_success() {
        let mut fixture = fixture();
        fixture.budget_cost_units = 21;
        fixture.minimum_spatial_temporal_coverage_fraction = 1.0;
        let template = fixture.candidates[0].clone();
        fixture.candidates = (0..21)
            .map(|index| {
                let mut candidate = template.clone();
                candidate.candidate_id = format!("coverage-candidate-{index:02}");
                candidate.spatial_stratum = format!("stratum-{index:02}");
                candidate.correlation_group = format!("coverage-group-{index:02}");
                candidate.cost_units = 1;
                candidate
            })
            .collect();
        let report = evaluate_fixture(&fixture, b"synthetic").expect("valid bounded fixture");
        assert_eq!(report.coverage_constraint_status, "state_limit_exceeded");
        assert_eq!(
            report.coverage_constraint_limit_reason.as_deref(),
            Some("coverage_stratum_limit")
        );
        assert_eq!(report.coverage_frontier_status, "state_limit_exceeded");
        assert_eq!(
            report.coverage_frontier_limit_reason.as_deref(),
            Some("coverage_stratum_limit")
        );
        assert!(report.coverage_frontier.is_empty());
        assert!(report.coverage_constrained_selected_candidate_ids.is_none());
        assert_eq!(report.selected_candidate_count, 21);
        assert!(!report.coverage_constraint_status.eq("satisfied"));
    }

    #[test]
    fn coverage_search_state_cap_is_reported_explicitly() {
        let mut fixture = fixture();
        fixture.budget_cost_units = 20;
        fixture.minimum_spatial_temporal_coverage_fraction = 1.0;
        let template = fixture.candidates[0].clone();
        fixture.candidates = (0..20)
            .map(|index| {
                let mut candidate = template.clone();
                candidate.candidate_id = format!("state-candidate-{index:02}");
                candidate.spatial_stratum = format!("state-stratum-{index:02}");
                candidate.correlation_group = format!("state-group-{index:02}");
                candidate.cost_units = 1;
                candidate
            })
            .collect();
        let report = evaluate_fixture(&fixture, b"synthetic").expect("valid 20-stratum fixture");
        assert_eq!(report.coverage_constraint_status, "state_limit_exceeded");
        assert_eq!(
            report.coverage_constraint_limit_reason.as_deref(),
            Some("coverage_state_limit")
        );
        assert_eq!(report.coverage_frontier_status, "state_limit_exceeded");
        assert_eq!(
            report.coverage_frontier_limit_reason.as_deref(),
            Some("coverage_state_limit")
        );
        assert!(report.coverage_frontier.is_empty());
        assert!(report.coverage_constrained_selected_candidate_ids.is_none());
    }

    #[test]
    fn zero_coverage_constraint_is_explicitly_disabled() {
        let mut fixture = fixture();
        fixture.minimum_spatial_temporal_coverage_fraction = 0.0;
        let report = evaluate_fixture(&fixture, b"synthetic").expect("zero floor");
        assert_eq!(report.coverage_constraint_status, "not_required");
        assert!(report.coverage_constrained_selected_candidate_ids.is_none());
    }

    #[test]
    fn coverage_constraint_rejects_out_of_range_fraction() {
        let mut fixture = fixture();
        fixture.minimum_spatial_temporal_coverage_fraction = 1.01;
        assert!(validate_fixture(&fixture).is_err());
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
        let report =
            evaluate_fixture(&denied_access, b"synthetic").expect("valid access control case");
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
    fn paired_scenario_digests_must_be_distinct_and_global() {
        let mut identical_pair = fixture();
        identical_pair.candidates[0].alternative_scenario_sha256 = identical_pair.candidates[0]
            .baseline_scenario_sha256
            .clone();
        assert!(validate_fixture(&identical_pair).is_err());

        let mut inconsistent_pair = fixture();
        inconsistent_pair.candidates[1].alternative_scenario_sha256 = "3".repeat(64);
        assert!(validate_fixture(&inconsistent_pair).is_err());
    }

    #[test]
    fn repeated_metric_unit_requires_same_model_family_and_version() {
        let mut mismatched = fixture();
        mismatched.candidates[1].model_family_id = "different-model-family".to_owned();
        assert!(validate_fixture(&mismatched).is_err());

        let mut mismatched_version = fixture();
        mismatched_version.candidates[1].model_version = "9.9.9".to_owned();
        assert!(validate_fixture(&mismatched_version).is_err());
    }

    #[test]
    fn consent_and_access_states_require_matching_evidence() {
        let mut granted_without_digest = fixture();
        granted_without_digest.candidates[0].consent_evidence_sha256 = None;
        assert!(validate_fixture(&granted_without_digest).is_err());

        let mut denied_with_digest = fixture();
        denied_with_digest.candidates[4].consent_evidence_sha256 = Some("5".repeat(64));
        assert!(validate_fixture(&denied_with_digest).is_err());

        let mut authorized_without_digest = fixture();
        authorized_without_digest.candidates[0].access_authorization_sha256 = None;
        assert!(validate_fixture(&authorized_without_digest).is_err());

        let mut unknown_access_with_digest = fixture();
        unknown_access_with_digest.candidates[5].access_authorization_sha256 = Some("6".repeat(64));
        assert!(validate_fixture(&unknown_access_with_digest).is_err());
    }

    #[test]
    fn malformed_digests_and_invalid_variances_fail_closed() {
        let mut bad_site_digest = fixture();
        bad_site_digest.candidates[0].site_ref_sha256 = "not-a-digest".to_owned();
        assert!(validate_fixture(&bad_site_digest).is_err());

        let mut bad_scenario_digest = fixture();
        bad_scenario_digest.candidates[0].baseline_scenario_sha256 = "g".repeat(64);
        assert!(validate_fixture(&bad_scenario_digest).is_err());

        let mut zero_variance = fixture();
        zero_variance.candidates[0].baseline_sd = 0.0;
        assert!(validate_fixture(&zero_variance).is_err());
    }

    #[test]
    fn metric_id_cannot_mix_units_or_model_versions() {
        let mut mixed_units = fixture();
        mixed_units.candidates[1].unit = "kelvin".to_owned();
        assert!(validate_fixture(&mixed_units).is_err());

        let mut mixed_versions = fixture();
        mixed_versions.candidates[1].model_version = "9.9.9".to_owned();
        assert!(validate_fixture(&mixed_versions).is_err());
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
