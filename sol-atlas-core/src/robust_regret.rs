// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Probability-free, adversarial decision-regret analysis for investigation planning.
//!
//! Each scenario is treated as plausible; no probability distribution is fabricated.
//! Losses are intervals. Regret uses an upper bound on the selected decision's loss
//! minus the lowest plausible lower-bound loss for that scenario. An investigation
//! partitions scenarios by its possible observations, enabling a different decision
//! for each observed outcome. The result is a conservative minimax-regret bound,
//! not a calibrated probability, causal claim, or monetary valuation.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionLoss {
    pub decision_id: String,
    pub scenario_id: String,
    pub loss_lower: f64,
    pub loss_upper: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestigationAction {
    pub action_id: String,
    /// Exact unit identifier; must equal the problem's loss_unit.
    pub cost_unit: String,
    pub cost: f64,
    /// Every outcome still possible under this action for each scenario. Multiple outcomes
    /// represent diagnostic ambiguity; no probabilities are assigned among them.
    pub outcome_by_scenario: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RobustRegretProblem {
    /// Explicit, shared unit for all loss intervals, e.g. "risk-points-v1".
    pub loss_unit: String,
    pub scenario_ids: BTreeSet<String>,
    pub decision_ids: BTreeSet<String>,
    /// Exactly one loss interval is required for every decision × scenario pair.
    pub losses: Vec<DecisionLoss>,
    pub actions: Vec<InvestigationAction>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineRegretAssessment {
    pub selected_decision: String,
    pub worst_case_regret_bound: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationDisposition {
    Worthwhile,
    BreakEven,
    NotWorthwhile,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestigationAssessment {
    pub action_id: String,
    pub cost: f64,
    /// Worst-case regret bound after allowing a separate decision per observed outcome.
    pub post_investigation_regret_bound: f64,
    /// Baseline bound minus the post-investigation bound; not an expected monetary value.
    pub regret_bound_reduction: f64,
    /// Regret-bound reduction minus declared investigation cost.
    pub net_regret_bound_reduction: f64,
    pub disposition: InvestigationDisposition,
    pub decision_by_outcome: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RobustRegretReport {
    pub baseline: BaselineRegretAssessment,
    /// Sorted by net regret-bound reduction descending, then action ID ascending.
    pub investigations: Vec<InvestigationAssessment>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RobustRegretError {
    EmptyScenarios,
    EmptyDecisions,
    EmptyIdentifier { field: &'static str },
    UnknownScenario(String),
    UnknownDecision(String),
    DuplicateLossCell { decision_id: String, scenario_id: String },
    MissingLossCell { decision_id: String, scenario_id: String },
    InvalidLossInterval { decision_id: String, scenario_id: String },
    DuplicateAction(String),
    MissingActionOutcome { action_id: String, scenario_id: String },
    EmptyActionOutcome { action_id: String, scenario_id: String },
    InvalidActionCost(String),
    IncompatibleUnits {
        action_id: String,
        loss_unit: String,
        cost_unit: String,
    },
    NumericOverflow(String),
}

impl fmt::Display for RobustRegretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RobustRegretError {}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

impl RobustRegretProblem {
    fn validate(&self) -> Result<(), RobustRegretError> {
        if !nonempty(&self.loss_unit) {
            return Err(RobustRegretError::EmptyIdentifier { field: "loss_unit" });
        }
        if self.scenario_ids.is_empty() {
            return Err(RobustRegretError::EmptyScenarios);
        }
        if self.decision_ids.is_empty() {
            return Err(RobustRegretError::EmptyDecisions);
        }
        for (field, values) in [
            ("scenario_id", &self.scenario_ids),
            ("decision_id", &self.decision_ids),
        ] {
            if values.iter().any(|value| !nonempty(value)) {
                return Err(RobustRegretError::EmptyIdentifier { field });
            }
        }

        let mut cells = BTreeSet::new();
        for loss in &self.losses {
            if !self.scenario_ids.contains(&loss.scenario_id) {
                return Err(RobustRegretError::UnknownScenario(loss.scenario_id.clone()));
            }
            if !self.decision_ids.contains(&loss.decision_id) {
                return Err(RobustRegretError::UnknownDecision(loss.decision_id.clone()));
            }
            if !loss.loss_lower.is_finite()
                || !loss.loss_upper.is_finite()
                || loss.loss_lower < 0.0
                || loss.loss_lower > loss.loss_upper
            {
                return Err(RobustRegretError::InvalidLossInterval {
                    decision_id: loss.decision_id.clone(),
                    scenario_id: loss.scenario_id.clone(),
                });
            }
            let key = (loss.decision_id.clone(), loss.scenario_id.clone());
            if !cells.insert(key.clone()) {
                return Err(RobustRegretError::DuplicateLossCell {
                    decision_id: key.0,
                    scenario_id: key.1,
                });
            }
        }

        for decision_id in &self.decision_ids {
            for scenario_id in &self.scenario_ids {
                if !cells.contains(&(decision_id.clone(), scenario_id.clone())) {
                    return Err(RobustRegretError::MissingLossCell {
                        decision_id: decision_id.clone(),
                        scenario_id: scenario_id.clone(),
                    });
                }
            }
        }

        let mut action_ids = BTreeSet::new();
        for action in &self.actions {
            if !nonempty(&action.action_id) {
                return Err(RobustRegretError::EmptyIdentifier { field: "action_id" });
            }
            if !action_ids.insert(action.action_id.clone()) {
                return Err(RobustRegretError::DuplicateAction(action.action_id.clone()));
            }
            if !nonempty(&action.cost_unit) {
                return Err(RobustRegretError::EmptyIdentifier { field: "cost_unit" });
            }
            if action.cost_unit != self.loss_unit {
                return Err(RobustRegretError::IncompatibleUnits {
                    action_id: action.action_id.clone(),
                    loss_unit: self.loss_unit.clone(),
                    cost_unit: action.cost_unit.clone(),
                });
            }
            if !action.cost.is_finite() || action.cost < 0.0 {
                return Err(RobustRegretError::InvalidActionCost(action.action_id.clone()));
            }
            for scenario_id in action.outcome_by_scenario.keys() {
                if !self.scenario_ids.contains(scenario_id) {
                    return Err(RobustRegretError::UnknownScenario(scenario_id.clone()));
                }
            }
            for scenario_id in &self.scenario_ids {
                let outcomes = action.outcome_by_scenario.get(scenario_id).ok_or_else(|| {
                    RobustRegretError::MissingActionOutcome {
                        action_id: action.action_id.clone(),
                        scenario_id: scenario_id.clone(),
                    }
                })?;
                if outcomes.is_empty() || outcomes.iter().any(|outcome| !nonempty(outcome)) {
                    return Err(RobustRegretError::EmptyActionOutcome {
                        action_id: action.action_id.clone(),
                        scenario_id: scenario_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Computes conservative minimax-regret bounds without assigning scenario probabilities.
///
/// Loss interval endpoints are non-negative and must share comparable units. For each
/// decision/scenario cell, the regret upper bound is the selected decision's upper loss
/// minus the lowest lower loss in that scenario. This deliberately combines the selected
/// decision's upper loss with the best competitor's lower loss. It is conservative;
/// interval dependence is not modeled. An action can only change decisions by splitting
/// scenarios into outcomes.
pub fn analyze_robust_regret(
    problem: &RobustRegretProblem,
) -> Result<RobustRegretReport, RobustRegretError> {
    problem.validate()?;

    let mut minimum_lower_by_scenario = BTreeMap::new();
    for scenario_id in &problem.scenario_ids {
        let min_lower = problem
            .losses
            .iter()
            .filter(|loss| &loss.scenario_id == scenario_id)
            .map(|loss| loss.loss_lower)
            .min_by(f64::total_cmp)
            .expect("validated matrix contains each scenario");
        minimum_lower_by_scenario.insert(scenario_id.clone(), min_lower);
    }

    let loss_by_cell: BTreeMap<(String, String), &DecisionLoss> = problem
        .losses
        .iter()
        .map(|loss| ((loss.decision_id.clone(), loss.scenario_id.clone()), loss))
        .collect();

    let mut regret_by_cell = BTreeMap::new();
    for decision_id in &problem.decision_ids {
        for scenario_id in &problem.scenario_ids {
            let loss = loss_by_cell[&(decision_id.clone(), scenario_id.clone())];
            let scenario_floor = minimum_lower_by_scenario[scenario_id];
            let regret = (loss.loss_upper - scenario_floor).max(0.0);
            if !regret.is_finite() {
                return Err(RobustRegretError::NumericOverflow(format!(
                    "{decision_id}/{scenario_id}"
                )));
            }
            regret_by_cell.insert((decision_id.clone(), scenario_id.clone()), regret);
        }
    }

    let all_scenarios: Vec<String> = problem.scenario_ids.iter().cloned().collect();
    let (baseline_decision, baseline_bound) =
        best_decision_for_scenarios(&problem.decision_ids, &all_scenarios, &regret_by_cell);
    let baseline = BaselineRegretAssessment {
        selected_decision: baseline_decision,
        worst_case_regret_bound: baseline_bound,
    };

    let mut investigations = Vec::with_capacity(problem.actions.len());
    for action in &problem.actions {
        let mut scenarios_by_outcome: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for scenario_id in &problem.scenario_ids {
            for outcome in &action.outcome_by_scenario[scenario_id] {
                scenarios_by_outcome
                    .entry(outcome.clone())
                    .or_default()
                    .push(scenario_id.clone());
            }
        }

        let mut decision_by_outcome = BTreeMap::new();
        let mut post_bound: f64 = 0.0;
        for (outcome, scenarios) in &scenarios_by_outcome {
            let (decision_id, outcome_bound) =
                best_decision_for_scenarios(&problem.decision_ids, scenarios, &regret_by_cell);
            decision_by_outcome.insert(outcome.clone(), decision_id);
            post_bound = post_bound.max(outcome_bound);
        }

        let regret_bound_reduction = (baseline_bound - post_bound).max(0.0);
        let net_regret_bound_reduction = regret_bound_reduction - action.cost;
        if !net_regret_bound_reduction.is_finite() {
            return Err(RobustRegretError::NumericOverflow(action.action_id.clone()));
        }
        let disposition = if net_regret_bound_reduction > 0.0 {
            InvestigationDisposition::Worthwhile
        } else if net_regret_bound_reduction == 0.0 {
            InvestigationDisposition::BreakEven
        } else {
            InvestigationDisposition::NotWorthwhile
        };
        investigations.push(InvestigationAssessment {
            action_id: action.action_id.clone(),
            cost: action.cost,
            post_investigation_regret_bound: post_bound,
            regret_bound_reduction,
            net_regret_bound_reduction,
            disposition,
            decision_by_outcome,
        });
    }

    investigations.sort_by(|left, right| {
        right
            .net_regret_bound_reduction
            .total_cmp(&left.net_regret_bound_reduction)
            .then_with(|| left.action_id.cmp(&right.action_id))
    });

    Ok(RobustRegretReport {
        baseline,
        investigations,
    })
}

fn best_decision_for_scenarios(
    decision_ids: &BTreeSet<String>,
    scenario_ids: &[String],
    regret_by_cell: &BTreeMap<(String, String), f64>,
) -> (String, f64) {
    let mut best: Option<(String, f64)> = None;
    for decision_id in decision_ids {
        let bound = scenario_ids
            .iter()
            .map(|scenario_id| regret_by_cell[&(decision_id.clone(), scenario_id.clone())])
            .max_by(f64::total_cmp)
            .expect("every observed outcome maps to at least one scenario");
        let replace = best
            .as_ref()
            .map(|(_, best_bound)| bound.total_cmp(best_bound).is_lt())
            .unwrap_or(true);
        if replace {
            best = Some((decision_id.clone(), bound));
        }
        // BTreeSet iteration gives deterministic lexical tie-breaking.
    }
    best.expect("validated problem declares at least one decision")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct BenchmarkCase {
        case_id: String,
        problem: RobustRegretProblem,
        expected_baseline_regret_bound: f64,
        expected_baseline_decision: String,
        expected_actions: Vec<ExpectedAction>,
    }

    #[derive(Deserialize)]
    struct ExpectedAction {
        action_id: String,
        post_investigation_regret_bound: f64,
        regret_bound_reduction: f64,
        net_regret_bound_reduction: f64,
        disposition: InvestigationDisposition,
        decision_by_outcome: BTreeMap<String, String>,
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1e-9,
            "actual {actual} differs from expected {expected}"
        );
    }

    #[test]
    fn synthetic_reference_benchmark_matches_expected_decisions() {
        let mut evaluated = 0;
        for line in include_str!("../../docs/research/fixtures/robust-regret-benchmark-v0.1.jsonl")
            .lines()
            .filter(|line| !line.trim().is_empty())
        {
            let case: BenchmarkCase = serde_json::from_str(line).expect("valid JSONL fixture");
            let report = analyze_robust_regret(&case.problem)
                .unwrap_or_else(|error| panic!("{}: {error}", case.case_id));
            close(
                report.baseline.worst_case_regret_bound,
                case.expected_baseline_regret_bound,
            );
            assert_eq!(
                report.baseline.selected_decision, case.expected_baseline_decision,
                "baseline decision in {}",
                case.case_id
            );
            assert_eq!(
                report.investigations.len(),
                case.expected_actions.len(),
                "action count in {}",
                case.case_id
            );
            for expected in case.expected_actions {
                let actual = report
                    .investigations
                    .iter()
                    .find(|item| item.action_id == expected.action_id)
                    .unwrap_or_else(|| {
                        panic!("{} missing action {}", case.case_id, expected.action_id)
                    });
                close(
                    actual.post_investigation_regret_bound,
                    expected.post_investigation_regret_bound,
                );
                close(actual.regret_bound_reduction, expected.regret_bound_reduction);
                close(
                    actual.net_regret_bound_reduction,
                    expected.net_regret_bound_reduction,
                );
                assert_eq!(actual.disposition, expected.disposition, "{}", case.case_id);
                assert_eq!(
                    actual.decision_by_outcome, expected.decision_by_outcome,
                    "{}",
                    case.case_id
                );
            }
            evaluated += 1;
        }
        assert_eq!(evaluated, 5, "the pinned benchmark has five synthetic cases");
    }

    fn basic_problem() -> RobustRegretProblem {
        RobustRegretProblem {
            loss_unit: "regret-units".into(),
            scenario_ids: ["dry".to_string(), "wet".to_string()].into_iter().collect(),
            decision_ids: ["site-a".to_string(), "site-b".to_string()].into_iter().collect(),
            losses: vec![
                DecisionLoss {
                    decision_id: "site-a".into(),
                    scenario_id: "dry".into(),
                    loss_lower: 0.0,
                    loss_upper: 0.0,
                },
                DecisionLoss {
                    decision_id: "site-b".into(),
                    scenario_id: "dry".into(),
                    loss_lower: 100.0,
                    loss_upper: 100.0,
                },
                DecisionLoss {
                    decision_id: "site-a".into(),
                    scenario_id: "wet".into(),
                    loss_lower: 80.0,
                    loss_upper: 80.0,
                },
                DecisionLoss {
                    decision_id: "site-b".into(),
                    scenario_id: "wet".into(),
                    loss_lower: 0.0,
                    loss_upper: 0.0,
                },
            ],
            actions: vec![],
        }
    }

    #[test]
    fn incomplete_loss_matrix_is_rejected() {
        let mut problem = basic_problem();
        problem.losses.pop();
        assert!(matches!(
            analyze_robust_regret(&problem),
            Err(RobustRegretError::MissingLossCell { .. })
        ));
    }

    #[test]
    fn inverted_interval_is_rejected() {
        let mut problem = basic_problem();
        problem.losses[0].loss_lower = 2.0;
        assert!(matches!(
            analyze_robust_regret(&problem),
            Err(RobustRegretError::InvalidLossInterval { .. })
        ));
    }

    #[test]
    fn missing_observation_for_a_scenario_is_rejected() {
        let mut problem = basic_problem();
        problem.actions.push(InvestigationAction {
            action_id: "inspect-water".into(),
            cost_unit: "regret-units".into(),
            cost: 1.0,
            outcome_by_scenario: BTreeMap::from([("dry".into(), BTreeSet::from(["low".into()]))]),
        });
        assert!(matches!(
            analyze_robust_regret(&problem),
            Err(RobustRegretError::MissingActionOutcome { .. })
        ));
    }


    #[test]
    fn incompatible_cost_units_are_rejected() {
        let mut problem = basic_problem();
        problem.actions.push(InvestigationAction {
            action_id: "cost-in-days".into(),
            cost_unit: "days".into(),
            cost: 1.0,
            outcome_by_scenario: BTreeMap::from([
                ("dry".into(), BTreeSet::from(["same".into()])),
                ("wet".into(), BTreeSet::from(["same".into()])),
            ]),
        });
        assert!(matches!(
            analyze_robust_regret(&problem),
            Err(RobustRegretError::IncompatibleUnits { .. })
        ));
    }


    #[test]
    fn empty_possible_outcome_set_is_rejected() {
        let mut problem = basic_problem();
        problem.actions.push(InvestigationAction {
            action_id: "empty-outcome-set".into(),
            cost_unit: "regret-units".into(),
            cost: 0.0,
            outcome_by_scenario: BTreeMap::from([
                ("dry".into(), BTreeSet::new()),
                ("wet".into(), BTreeSet::from(["wet".into()])),
            ]),
        });
        assert!(matches!(
            analyze_robust_regret(&problem),
            Err(RobustRegretError::EmptyActionOutcome { .. })
        ));
    }

    #[test]
    fn investigation_cannot_increase_the_minimax_regret_bound_before_cost() {
        let mut problem = basic_problem();
        problem.actions.push(InvestigationAction {
            action_id: "partial-test".into(),
            cost_unit: "regret-units".into(),
            cost: 0.0,
            outcome_by_scenario: BTreeMap::from([
                ("dry".into(), BTreeSet::from(["not-dry".into()])),
                ("wet".into(), BTreeSet::from(["not-dry".into()])),
            ]),
        });
        problem.actions.push(InvestigationAction {
            action_id: "perfect-test".into(),
            cost_unit: "regret-units".into(),
            cost: 0.0,
            outcome_by_scenario: BTreeMap::from([
                ("dry".into(), BTreeSet::from(["dry".into()])),
                ("wet".into(), BTreeSet::from(["wet".into()])),
            ]),
        });
        let report = analyze_robust_regret(&problem).unwrap();
        for action in &report.investigations {
            assert!(
                action.post_investigation_regret_bound
                    <= report.baseline.worst_case_regret_bound + 1e-9
            );
        }
    }

    #[test]
    fn exhaustive_two_by_two_loss_grids_preserve_information_monotonicity() {
        let scenario_ids: BTreeSet<String> =
            ["scenario-a".to_string(), "scenario-b".to_string()]
                .into_iter()
                .collect();
        let decision_ids: BTreeSet<String> =
            ["decision-a".to_string(), "decision-b".to_string()]
                .into_iter()
                .collect();
        let values = [0.0, 1.0, 2.0];
        let mut evaluated = 0;

        // Enumerate all 3^4 exact-loss matrices for two decisions and two scenarios.
        for encoded in 0..81 {
            let mut remainder = encoded;
            let mut losses = Vec::with_capacity(4);
            for decision_id in ["decision-a", "decision-b"] {
                for scenario_id in ["scenario-a", "scenario-b"] {
                    let value = values[remainder % values.len()];
                    remainder /= values.len();
                    losses.push(DecisionLoss {
                        decision_id: decision_id.into(),
                        scenario_id: scenario_id.into(),
                        loss_lower: value,
                        loss_upper: value,
                    });
                }
            }

            let problem = RobustRegretProblem {
                loss_unit: "regret-units".into(),
                scenario_ids: scenario_ids.clone(),
                decision_ids: decision_ids.clone(),
                losses,
                actions: vec![
                    InvestigationAction {
                        action_id: "uninformative".into(),
                        cost_unit: "regret-units".into(),
                        cost: 0.0,
                        outcome_by_scenario: BTreeMap::from([
                            ("scenario-a".into(), "same".into()),
                            ("scenario-b".into(), "same".into()),
                        ]),
                    },
                    InvestigationAction {
                        action_id: "fully-discriminating".into(),
                        cost_unit: "regret-units".into(),
                        cost: 0.0,
                        outcome_by_scenario: BTreeMap::from([
                            ("scenario-a".into(), "outcome-a".into()),
                            ("scenario-b".into(), "outcome-b".into()),
                        ]),
                    },
                ],
            };

            let report = analyze_robust_regret(&problem).unwrap();
            let uninformative = report
                .investigations
                .iter()
                .find(|item| item.action_id == "uninformative")
                .unwrap();
            let discriminating = report
                .investigations
                .iter()
                .find(|item| item.action_id == "fully-discriminating")
                .unwrap();

            assert_eq!(
                uninformative.post_investigation_regret_bound,
                report.baseline.worst_case_regret_bound,
                "an uninformative action must not change the bound for matrix {encoded}"
            );
            assert!(
                discriminating.post_investigation_regret_bound
                    <= report.baseline.worst_case_regret_bound + 1e-9,
                "information increased regret bound for matrix {encoded}"
            );
            assert_eq!(
                discriminating.post_investigation_regret_bound, 0.0,
                "perfect state discrimination should remove exact-loss regret for matrix {encoded}"
            );
            evaluated += 1;
        }

        assert_eq!(evaluated, 81);
    }

    #[test]
    fn exhaustive_ambiguous_observation_relations_never_increase_regret_bound() {
        let scenario_ids: BTreeSet<String> =
            ["scenario-a".to_string(), "scenario-b".to_string()]
                .into_iter()
                .collect();
        let decision_ids: BTreeSet<String> =
            ["decision-a".to_string(), "decision-b".to_string()]
                .into_iter()
                .collect();
        let values = [0.0, 1.0, 2.0];
        let outcome_set = |mask: u8| {
            let mut outcomes = BTreeSet::new();
            if mask & 1 != 0 {
                outcomes.insert("outcome-a".to_string());
            }
            if mask & 2 != 0 {
                outcomes.insert("outcome-b".to_string());
            }
            outcomes
        };
        let mut evaluated = 0;

        // 3^4 loss matrices × 3^2 non-empty observation sets = 729 models.
        // An observation set may overlap another scenario's set, representing ambiguity.
        for encoded in 0..81 {
            let mut remainder = encoded;
            let mut losses = Vec::with_capacity(4);
            for decision_id in ["decision-a", "decision-b"] {
                for scenario_id in ["scenario-a", "scenario-b"] {
                    let value = values[remainder % values.len()];
                    remainder /= values.len();
                    losses.push(DecisionLoss {
                        decision_id: decision_id.into(),
                        scenario_id: scenario_id.into(),
                        loss_lower: value,
                        loss_upper: value,
                    });
                }
            }

            let baseline_problem = RobustRegretProblem {
                loss_unit: "regret-units".into(),
                scenario_ids: scenario_ids.clone(),
                decision_ids: decision_ids.clone(),
                losses: losses.clone(),
                actions: vec![],
            };
            let baseline = analyze_robust_regret(&baseline_problem)
                .unwrap()
                .baseline
                .worst_case_regret_bound;

            for mask_a in 1..=3 {
                for mask_b in 1..=3 {
                    let problem = RobustRegretProblem {
                        loss_unit: "regret-units".into(),
                        scenario_ids: scenario_ids.clone(),
                        decision_ids: decision_ids.clone(),
                        losses: losses.clone(),
                        actions: vec![InvestigationAction {
                            action_id: "set-valued-observation".into(),
                            cost_unit: "regret-units".into(),
                            cost: 0.0,
                            outcome_by_scenario: BTreeMap::from([
                                ("scenario-a".into(), outcome_set(mask_a)),
                                ("scenario-b".into(), outcome_set(mask_b)),
                            ]),
                        }],
                    };
                    let report = analyze_robust_regret(&problem).unwrap();
                    let actual = &report.investigations[0];
                    assert!(
                        actual.post_investigation_regret_bound <= baseline + 1e-9,
                        "observation relation increased regret for loss grid {encoded}, masks {mask_a}/{mask_b}"
                    );
                    if mask_a == mask_b {
                        assert_eq!(
                            actual.post_investigation_regret_bound, baseline,
                            "identical possible-outcome sets must not add information"
                        );
                    }
                    evaluated += 1;
                }
            }
        }

        assert_eq!(evaluated, 729);
    }

}
