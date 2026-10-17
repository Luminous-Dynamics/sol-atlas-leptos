// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario-based ranking sensitivity for candidate sites and design alternatives.
//!
//! This module checks whether rankings are stable across caller-supplied plausible
//! preference regimes. It does not assign probabilities to stakeholder preferences
//! and must not be presented as a probabilistic forecast.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDisposition {
    Eligible,
    NeedsEvidence,
    Excluded,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensitivityCandidate {
    pub candidate_id: String,
    pub gate: GateDisposition,
    /// Normalized, benefit-oriented values in [0, 1]. Missing values are None.
    #[serde(default)]
    pub scores: BTreeMap<String, Option<f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeightScenario {
    pub scenario_id: String,
    /// Declared objective IDs -> non-negative importance weights.
    pub weights: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensitivityPolicy {
    pub objectives: BTreeSet<String>,
    /// Scores within this tolerance are grouped as a tie. Scores are normalized [0, 1].
    pub score_tolerance: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioCandidateStatus {
    Ranked,
    NeedsEvidence,
    Excluded,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioCandidateResult {
    pub status: ScenarioCandidateStatus,
    pub weighted_score: Option<f64>,
    pub rank: Option<usize>,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioRanking {
    pub scenario_id: String,
    pub candidates: BTreeMap<String, ScenarioCandidateResult>,
    /// All rank-1 candidates, so exact/near ties remain visible.
    pub winners: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateSensitivitySummary {
    pub total_scenarios: usize,
    pub ranked_scenarios: usize,
    pub needs_evidence_scenarios: usize,
    pub excluded_scenarios: usize,
    pub top_rank_scenarios: usize,
    /// Fraction of all declared scenarios where this candidate ranked first.
    /// This is scenario coverage, not a probability or prediction.
    pub top_rank_scenario_share: f64,
    pub minimum_rank: Option<usize>,
    pub maximum_rank: Option<usize>,
    pub mean_rank: Option<f64>,
    pub mean_weighted_score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PairwiseSensitivity {
    pub candidate_a: String,
    pub candidate_b: String,
    pub comparable_scenarios: usize,
    pub a_preferred_scenarios: usize,
    pub b_preferred_scenarios: usize,
    pub tied_scenarios: usize,
    /// True when scenario changes reverse the pair's ordering at least once.
    pub ordering_reversal_observed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensitivityReport {
    pub scenario_rankings: Vec<ScenarioRanking>,
    pub candidates: BTreeMap<String, CandidateSensitivitySummary>,
    pub pairwise: Vec<PairwiseSensitivity>,
    /// Set only if the same unique candidate is the sole winner in every scenario.
    pub robust_unique_winner: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SensitivityError {
    EmptyObjectives,
    EmptyObjectiveId,
    EmptyCandidateId,
    DuplicateCandidateId(String),
    EmptyScenarioId,
    DuplicateScenarioId(String),
    MissingScenarioWeight {
        scenario_id: String,
        objective_id: String,
    },
    UnknownScenarioObjective {
        scenario_id: String,
        objective_id: String,
    },
    UnknownCandidateObjective {
        candidate_id: String,
        objective_id: String,
    },
    InvalidWeight {
        scenario_id: String,
        objective_id: String,
        value: f64,
    },
    NoPositiveWeight(String),
    InvalidScoreTolerance(f64),
    InvalidScore {
        candidate_id: String,
        objective_id: String,
        value: f64,
    },
}

impl fmt::Display for SensitivityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObjectives => write!(f, "sensitivity policy must define objectives"),
            Self::EmptyObjectiveId => write!(f, "objective IDs must not be empty"),
            Self::EmptyCandidateId => write!(f, "candidate IDs must not be empty"),
            Self::DuplicateCandidateId(id) => write!(f, "duplicate candidate ID: {id}"),
            Self::EmptyScenarioId => write!(f, "scenario IDs must not be empty"),
            Self::DuplicateScenarioId(id) => write!(f, "duplicate scenario ID: {id}"),
            Self::MissingScenarioWeight { scenario_id, objective_id } => write!(
                f,
                "scenario {scenario_id} lacks a weight for objective {objective_id}"
            ),
            Self::UnknownScenarioObjective { scenario_id, objective_id } => write!(
                f,
                "scenario {scenario_id} defines undeclared objective {objective_id}"
            ),
            Self::UnknownCandidateObjective { candidate_id, objective_id } => write!(
                f,
                "candidate {candidate_id} defines undeclared objective {objective_id}"
            ),
            Self::InvalidWeight { scenario_id, objective_id, value } => write!(
                f,
                "weight for {objective_id} in scenario {scenario_id} must be finite and non-negative, got {value}"
            ),
            Self::NoPositiveWeight(id) => {
                write!(f, "scenario {id} must have at least one positive weight")
            }
            Self::InvalidScoreTolerance(value) => write!(
                f,
                "score tolerance must be finite and non-negative, got {value}"
            ),
            Self::InvalidScore { candidate_id, objective_id, value } => write!(
                f,
                "score for {objective_id} on candidate {candidate_id} must be finite and in [0, 1], got {value}"
            ),
        }
    }
}

impl std::error::Error for SensitivityError {}

fn validate(
    candidates: &[SensitivityCandidate],
    scenarios: &[WeightScenario],
    policy: &SensitivityPolicy,
) -> Result<(), SensitivityError> {
    if policy.objectives.is_empty() {
        return Err(SensitivityError::EmptyObjectives);
    }
    if policy.objectives.iter().any(|id| id.trim().is_empty()) {
        return Err(SensitivityError::EmptyObjectiveId);
    }
    if !policy.score_tolerance.is_finite() || policy.score_tolerance < 0.0 {
        return Err(SensitivityError::InvalidScoreTolerance(policy.score_tolerance));
    }
    if scenarios.is_empty() {
        return Err(SensitivityError::EmptyScenarioId);
    }

    let mut scenario_ids = BTreeSet::new();
    for scenario in scenarios {
        if scenario.scenario_id.trim().is_empty() {
            return Err(SensitivityError::EmptyScenarioId);
        }
        if !scenario_ids.insert(scenario.scenario_id.as_str()) {
            return Err(SensitivityError::DuplicateScenarioId(
                scenario.scenario_id.clone(),
            ));
        }
        for objective_id in &policy.objectives {
            if !scenario.weights.contains_key(objective_id) {
                return Err(SensitivityError::MissingScenarioWeight {
                    scenario_id: scenario.scenario_id.clone(),
                    objective_id: objective_id.clone(),
                });
            }
        }
        for (objective_id, weight) in &scenario.weights {
            if !policy.objectives.contains(objective_id) {
                return Err(SensitivityError::UnknownScenarioObjective {
                    scenario_id: scenario.scenario_id.clone(),
                    objective_id: objective_id.clone(),
                });
            }
            if !weight.is_finite() || *weight < 0.0 {
                return Err(SensitivityError::InvalidWeight {
                    scenario_id: scenario.scenario_id.clone(),
                    objective_id: objective_id.clone(),
                    value: *weight,
                });
            }
        }
        if scenario.weights.values().all(|weight| *weight == 0.0) {
            return Err(SensitivityError::NoPositiveWeight(
                scenario.scenario_id.clone(),
            ));
        }
    }

    let mut candidate_ids = BTreeSet::new();
    for candidate in candidates {
        if candidate.candidate_id.trim().is_empty() {
            return Err(SensitivityError::EmptyCandidateId);
        }
        if !candidate_ids.insert(candidate.candidate_id.as_str()) {
            return Err(SensitivityError::DuplicateCandidateId(
                candidate.candidate_id.clone(),
            ));
        }
        for (objective_id, value) in &candidate.scores {
            if !policy.objectives.contains(objective_id) {
                return Err(SensitivityError::UnknownCandidateObjective {
                    candidate_id: candidate.candidate_id.clone(),
                    objective_id: objective_id.clone(),
                });
            }
            if let Some(value) = value {
                if !value.is_finite() || !(0.0..=1.0).contains(value) {
                    return Err(SensitivityError::InvalidScore {
                        candidate_id: candidate.candidate_id.clone(),
                        objective_id: objective_id.clone(),
                        value: *value,
                    });
                }
            }
        }
    }
    Ok(())
}

fn score_candidate(
    candidate: &SensitivityCandidate,
    scenario: &WeightScenario,
) -> Result<f64, Vec<String>> {
    match candidate.gate {
        GateDisposition::Excluded => {
            return Err(vec!["upstream hard-constraint policy excluded this candidate".into()]);
        }
        GateDisposition::NeedsEvidence => {
            return Err(vec!["upstream gate disposition requires additional evidence".into()]);
        }
        GateDisposition::Eligible => {}
    }

    let maximum_weight = scenario.weights.values().copied().fold(0.0_f64, f64::max);
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    let mut missing = Vec::new();
    for (objective_id, weight) in &scenario.weights {
        if *weight == 0.0 {
            continue;
        }
        let normalized_weight = weight / maximum_weight;
        match candidate.scores.get(objective_id).copied().flatten() {
            Some(value) => {
                weighted_sum += value * normalized_weight;
                total_weight += normalized_weight;
            }
            None => missing.push(format!(
                "active objective {objective_id} lacks a usable score in scenario {}",
                scenario.scenario_id
            )),
        }
    }
    if !missing.is_empty() {
        return Err(missing);
    }
    if total_weight <= 0.0 {
        // Scenario validation makes this unreachable for complete candidate scores.
        return Err(vec!["scenario has no positive scoring weight".into()]);
    }
    Ok(weighted_sum / total_weight)
}

fn rank_scores(
    scores: &BTreeMap<String, f64>,
    tolerance: f64,
) -> BTreeMap<String, usize> {
    let mut ranked: Vec<(&String, f64)> = scores.iter().map(|(id, score)| (id, *score)).collect();
    ranked.sort_by(|(id_a, score_a), (id_b, score_b)| {
        score_b
            .total_cmp(score_a)
            .then_with(|| id_a.cmp(id_b))
    });

    let mut ranks = BTreeMap::new();
    let mut start = 0;
    while start < ranked.len() {
        let anchor = ranked[start].1;
        let mut end = start + 1;
        while end < ranked.len() && (anchor - ranked[end].1) <= tolerance {
            end += 1;
        }
        let rank = start + 1;
        for (candidate_id, _) in &ranked[start..end] {
            ranks.insert((*candidate_id).clone(), rank);
        }
        start = end;
    }
    ranks
}

/// Evaluate rank stability across a fixed, explicit set of plausible preference scenarios.
///
/// Scenario shares are descriptive coverage over scenarios supplied by the caller, not
/// probabilities that a stakeholder will choose a preference set. Input weights must be
/// declared for every objective; only positively weighted objective values are required
/// for a candidate to be ranked within a given scenario.
pub fn analyze_sensitivity(
    candidates: &[SensitivityCandidate],
    scenarios: &[WeightScenario],
    policy: &SensitivityPolicy,
) -> Result<SensitivityReport, SensitivityError> {
    validate(candidates, scenarios, policy)?;

    let mut candidates_sorted: Vec<&SensitivityCandidate> = candidates.iter().collect();
    candidates_sorted.sort_by(|a, b| a.candidate_id.cmp(&b.candidate_id));
    let mut scenarios_sorted: Vec<&WeightScenario> = scenarios.iter().collect();
    scenarios_sorted.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));

    let mut scenario_rankings = Vec::with_capacity(scenarios_sorted.len());
    for scenario in scenarios_sorted {
        let mut candidate_results = BTreeMap::new();
        let mut score_map = BTreeMap::new();
        for candidate in &candidates_sorted {
            match score_candidate(candidate, scenario) {
                Ok(score) => {
                    score_map.insert(candidate.candidate_id.clone(), score);
                    candidate_results.insert(
                        candidate.candidate_id.clone(),
                        ScenarioCandidateResult {
                            status: ScenarioCandidateStatus::Ranked,
                            weighted_score: Some(score),
                            rank: None,
                            reasons: Vec::new(),
                        },
                    );
                }
                Err(reasons) => {
                    let status = match candidate.gate {
                        GateDisposition::Excluded => ScenarioCandidateStatus::Excluded,
                        GateDisposition::NeedsEvidence | GateDisposition::Eligible => {
                            ScenarioCandidateStatus::NeedsEvidence
                        }
                    };
                    candidate_results.insert(
                        candidate.candidate_id.clone(),
                        ScenarioCandidateResult {
                            status,
                            weighted_score: None,
                            rank: None,
                            reasons,
                        },
                    );
                }
            }
        }

        let ranks = rank_scores(&score_map, policy.score_tolerance);
        for (candidate_id, rank) in ranks {
            if let Some(result) = candidate_results.get_mut(&candidate_id) {
                result.rank = Some(rank);
            }
        }
        let mut winners: Vec<String> = candidate_results
            .iter()
            .filter(|(_, result)| result.rank == Some(1))
            .map(|(candidate_id, _)| candidate_id.clone())
            .collect();
        winners.sort();
        scenario_rankings.push(ScenarioRanking {
            scenario_id: scenario.scenario_id.clone(),
            candidates: candidate_results,
            winners,
        });
    }

    let mut summaries = BTreeMap::new();
    for candidate in &candidates_sorted {
        let mut ranked_scenarios = 0;
        let mut needs_evidence = 0;
        let mut excluded = 0;
        let mut top = 0;
        let mut ranks: Vec<usize> = Vec::new();
        let mut scores = Vec::new();

        for scenario in &scenario_rankings {
            let result = &scenario.candidates[&candidate.candidate_id];
            match result.status {
                ScenarioCandidateStatus::Ranked => {
                    ranked_scenarios += 1;
                    if result.rank == Some(1) {
                        top += 1;
                    }
                    if let Some(rank) = result.rank {
                        ranks.push(rank);
                    }
                    if let Some(score) = result.weighted_score {
                        scores.push(score);
                    }
                }
                ScenarioCandidateStatus::NeedsEvidence => needs_evidence += 1,
                ScenarioCandidateStatus::Excluded => excluded += 1,
            }
        }

        summaries.insert(
            candidate.candidate_id.clone(),
            CandidateSensitivitySummary {
                total_scenarios: scenario_rankings.len(),
                ranked_scenarios,
                needs_evidence_scenarios: needs_evidence,
                excluded_scenarios: excluded,
                top_rank_scenarios: top,
                top_rank_scenario_share: if scenario_rankings.is_empty() {
                    0.0
                } else {
                    top as f64 / scenario_rankings.len() as f64
                },
                minimum_rank: ranks.iter().copied().min(),
                maximum_rank: ranks.iter().copied().max(),
                mean_rank: (!ranks.is_empty()).then(|| {
                    ranks.iter().map(|rank| *rank as f64).sum::<f64>() / ranks.len() as f64
                }),
                mean_weighted_score: (!scores.is_empty())
                    .then(|| scores.iter().sum::<f64>() / scores.len() as f64),
            },
        );
    }

    let candidate_ids: Vec<String> = candidates_sorted
        .iter()
        .map(|candidate| candidate.candidate_id.clone())
        .collect();
    let mut pairwise = Vec::new();
    for i in 0..candidate_ids.len() {
        for j in (i + 1)..candidate_ids.len() {
            let candidate_a = &candidate_ids[i];
            let candidate_b = &candidate_ids[j];
            let (mut comparable, mut a_preferred, mut b_preferred, mut tied) = (0, 0, 0, 0);
            for scenario in &scenario_rankings {
                let a = &scenario.candidates[candidate_a];
                let b = &scenario.candidates[candidate_b];
                if let (Some(rank_a), Some(rank_b)) = (a.rank, b.rank) {
                    comparable += 1;
                    if rank_a < rank_b {
                        a_preferred += 1;
                    } else if rank_b < rank_a {
                        b_preferred += 1;
                    } else {
                        tied += 1;
                    }
                }
            }
            pairwise.push(PairwiseSensitivity {
                candidate_a: candidate_a.clone(),
                candidate_b: candidate_b.clone(),
                comparable_scenarios: comparable,
                a_preferred_scenarios: a_preferred,
                b_preferred_scenarios: b_preferred,
                tied_scenarios: tied,
                ordering_reversal_observed: a_preferred > 0 && b_preferred > 0,
            });
        }
    }

    let robust_unique_winner = scenario_rankings.first().and_then(|first| {
        if first.winners.len() != 1 {
            return None;
        }
        let candidate_id = first.winners[0].clone();
        scenario_rankings
            .iter()
            .all(|scenario| scenario.winners == vec![candidate_id.clone()])
            .then_some(candidate_id)
    });

    Ok(SensitivityReport {
        scenario_rankings,
        candidates: summaries,
        pairwise,
        robust_unique_winner,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn policy() -> SensitivityPolicy {
        SensitivityPolicy {
            objectives: ["access", "carbon", "community"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            score_tolerance: 0.0,
        }
    }

    fn candidate(
        id: &str,
        gate: GateDisposition,
        access: Option<f64>,
        carbon: Option<f64>,
        community: Option<f64>,
    ) -> SensitivityCandidate {
        SensitivityCandidate {
            candidate_id: id.into(),
            gate,
            scores: [
                ("access".into(), access),
                ("carbon".into(), carbon),
                ("community".into(), community),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn scenarios() -> Vec<WeightScenario> {
        vec![
            WeightScenario {
                scenario_id: "access-first".into(),
                weights: [
                    ("access".into(), 0.8),
                    ("carbon".into(), 0.1),
                    ("community".into(), 0.1),
                ]
                .into_iter()
                .collect(),
            },
            WeightScenario {
                scenario_id: "carbon-first".into(),
                weights: [
                    ("access".into(), 0.1),
                    ("carbon".into(), 0.8),
                    ("community".into(), 0.1),
                ]
                .into_iter()
                .collect(),
            },
            WeightScenario {
                scenario_id: "community-first".into(),
                weights: [
                    ("access".into(), 0.1),
                    ("carbon".into(), 0.1),
                    ("community".into(), 0.8),
                ]
                .into_iter()
                .collect(),
            },
        ]
    }

    #[test]
    fn ranking_reversals_are_reported_across_preference_scenarios() {
        let candidates = [
            candidate("A", GateDisposition::Eligible, Some(0.95), Some(0.3), Some(0.6)),
            candidate("B", GateDisposition::Eligible, Some(0.6), Some(0.95), Some(0.5)),
            candidate("C", GateDisposition::Eligible, Some(0.7), Some(0.75), Some(0.95)),
        ];
        let report = analyze_sensitivity(&candidates, &scenarios(), &policy()).unwrap();
        let winners: BTreeMap<String, Vec<String>> = report
            .scenario_rankings
            .iter()
            .map(|scenario| (scenario.scenario_id.clone(), scenario.winners.clone()))
            .collect();
        assert_eq!(winners["access-first"], vec!["A"]);
        assert_eq!(winners["carbon-first"], vec!["B"]);
        assert_eq!(winners["community-first"], vec!["C"]);
        assert!(report.pairwise.iter().any(|pair| pair.ordering_reversal_observed));
        assert_eq!(report.robust_unique_winner, None);
    }

    #[test]
    fn same_unique_winner_across_all_scenarios_is_reported() {
        let candidates = [
            candidate("stable", GateDisposition::Eligible, Some(0.99), Some(0.99), Some(0.99)),
            candidate("other", GateDisposition::Eligible, Some(0.4), Some(0.4), Some(0.4)),
        ];
        let report = analyze_sensitivity(&candidates, &scenarios(), &policy()).unwrap();
        assert_eq!(report.robust_unique_winner, Some("stable".into()));
        assert_eq!(report.candidates["stable"].top_rank_scenarios, 3);
        assert_eq!(report.candidates["stable"].top_rank_scenario_share, 1.0);
    }

    #[test]
    fn near_ties_use_a_declared_score_tolerance() {
        let mut p = policy();
        p.score_tolerance = 0.01;
        let candidates = [
            candidate("A", GateDisposition::Eligible, Some(0.801), Some(0.8), Some(0.8)),
            candidate("B", GateDisposition::Eligible, Some(0.8), Some(0.8), Some(0.8)),
        ];
        let one = vec![WeightScenario {
            scenario_id: "equal-access".into(),
            weights: [
                ("access".into(), 1.0),
                ("carbon".into(), 0.0),
                ("community".into(), 0.0),
            ]
            .into_iter()
            .collect(),
        }];
        let result = analyze_sensitivity(&candidates, &one, &p).unwrap();
        assert_eq!(result.scenario_rankings[0].winners, vec!["A", "B"]);
        assert_eq!(result.candidates["A"].minimum_rank, Some(1));
        assert_eq!(result.candidates["B"].minimum_rank, Some(1));
    }

    #[test]
    fn excluded_candidate_never_wins_due_to_a_high_score() {
        let candidates = [
            candidate("excluded", GateDisposition::Excluded, Some(1.0), Some(1.0), Some(1.0)),
            candidate("allowed", GateDisposition::Eligible, Some(0.4), Some(0.4), Some(0.4)),
        ];
        let result = analyze_sensitivity(&candidates, &scenarios(), &policy()).unwrap();
        assert_eq!(result.candidates["excluded"].excluded_scenarios, 3);
        assert!(result.scenario_rankings.iter().all(|s| s.winners == vec!["allowed"]));
    }

    #[test]
    fn missing_active_objective_is_held_but_zero_weight_objective_can_be_irrelevant_to_that_scenario() {
        let candidates = [
            candidate("partial", GateDisposition::Eligible, Some(0.95), None, Some(0.6)),
            candidate("complete", GateDisposition::Eligible, Some(0.8), Some(0.8), Some(0.8)),
        ];
        let options = [
            WeightScenario {
                scenario_id: "access-only".into(),
                weights: [
                    ("access".into(), 1.0),
                    ("carbon".into(), 0.0),
                    ("community".into(), 0.0),
                ]
                .into_iter()
                .collect(),
            },
            WeightScenario {
                scenario_id: "carbon-active".into(),
                weights: [
                    ("access".into(), 0.1),
                    ("carbon".into(), 0.8),
                    ("community".into(), 0.1),
                ]
                .into_iter()
                .collect(),
            },
        ];
        let result = analyze_sensitivity(&candidates, &options, &policy()).unwrap();
        assert_eq!(result.scenario_rankings[0].candidates["partial"].status, ScenarioCandidateStatus::Ranked);
        assert_eq!(result.scenario_rankings[1].candidates["partial"].status, ScenarioCandidateStatus::NeedsEvidence);
        assert_eq!(result.candidates["partial"].ranked_scenarios, 1);
        assert_eq!(result.candidates["partial"].needs_evidence_scenarios, 1);
    }

    #[test]
    fn invalid_weight_and_invalid_score_are_rejected() {
        let candidates = [candidate("A", GateDisposition::Eligible, Some(0.5), Some(0.5), Some(0.5))];
        let mut invalid_scenarios = scenarios();
        invalid_scenarios[0].weights.insert("access".into(), -0.1);
        assert!(matches!(
            analyze_sensitivity(&candidates, &invalid_scenarios, &policy()),
            Err(SensitivityError::InvalidWeight { .. })
        ));

        let invalid_candidate = [candidate("bad", GateDisposition::Eligible, Some(1.1), Some(0.5), Some(0.5))];
        assert!(matches!(
            analyze_sensitivity(&invalid_candidate, &scenarios(), &policy()),
            Err(SensitivityError::InvalidScore { .. })
        ));
    }

    #[test]
    fn input_order_does_not_change_report() {
        let a = candidate("A", GateDisposition::Eligible, Some(0.95), Some(0.3), Some(0.6));
        let b = candidate("B", GateDisposition::Eligible, Some(0.6), Some(0.95), Some(0.5));
        let c = candidate("C", GateDisposition::Eligible, Some(0.7), Some(0.75), Some(0.95));
        let options = scenarios();
        let forward = analyze_sensitivity(&[a.clone(), b.clone(), c.clone()], &options, &policy()).unwrap();
        let reversed = analyze_sensitivity(&[c, a, b], &[options[2].clone(), options[0].clone(), options[1].clone()], &policy()).unwrap();
        assert_eq!(forward, reversed);
    }

    #[derive(Deserialize)]
    struct BenchmarkCase {
        case_id: String,
        policy: SensitivityPolicy,
        candidates: Vec<SensitivityCandidate>,
        scenarios: Vec<WeightScenario>,
        expected_winners: BTreeMap<String, Vec<String>>,
        expected_robust_unique_winner: Option<String>,
    }

    #[test]
    fn synthetic_weight_sensitivity_benchmark_matches_expected_winners() {
        let fixture = include_str!("../../docs/research/fixtures/site-ranking-sensitivity-v0.1.jsonl");
        let mut records = 0;
        for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
            let case: BenchmarkCase = serde_json::from_str(line).expect("valid sensitivity JSONL");
            let report = analyze_sensitivity(&case.candidates, &case.scenarios, &case.policy)
                .unwrap_or_else(|error| panic!("{}: {error}", case.case_id));
            let got: BTreeMap<String, Vec<String>> = report
                .scenario_rankings
                .iter()
                .map(|scenario| (scenario.scenario_id.clone(), scenario.winners.clone()))
                .collect();
            assert_eq!(got, case.expected_winners, "{}", case.case_id);
            assert_eq!(report.robust_unique_winner, case.expected_robust_unique_winner, "{}", case.case_id);
            records += 1;
        }
        assert!(records >= 6, "benchmark must include at least six scenarios");
    }
}
