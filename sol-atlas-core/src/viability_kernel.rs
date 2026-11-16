// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Exact finite-state robust viability for infrastructure configuration spaces.
//!
//! This bounded reference kernel is not a city-scale simulator, policy
//! generator, hydraulic model, or authority system. The controller chooses an
//! admissible action; every declared disturbance branch must remain viable.
//!
//! It uses explicit finite state/action sets, ordered maps and conjunctive
//! mandatory floors. Unknown evidence never becomes a passing floor or action.

use std::collections::{BTreeMap, BTreeSet};

/// Result of a mandatory state or action gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateResult {
    Pass,
    Fail,
    Unknown,
}

/// Robust status of one state in the finite configuration space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViabilityStatus {
    /// At least one admissible action has only viable disturbance successors.
    Viable,
    /// No admissible action preserves every mandatory floor.
    Infeasible,
    /// Missing or unknown evidence prevents a robust conclusion.
    Unresolved,
}

/// One finite configuration at a discrete time stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViabilityState {
    pub state_id: String,
    pub stage: u32,
    pub floors: BTreeMap<String, GateResult>,
}

/// One control action from a state, with named successors for modeled
/// disturbance branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViabilityAction {
    pub action_id: String,
    pub from_state_id: String,
    pub gates: BTreeMap<String, GateResult>,
    pub successor_by_disturbance: BTreeMap<String, String>,
}

/// Complete finite robust-viability problem.
///
/// disturbances_by_stage[t] is the declared disturbance set at stage t.
/// Every viable action must specify a successor for every disturbance.
/// Missing branches produce Unresolved, not optimistic viability.
/// Every transition must advance exactly one stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViabilityProblem {
    pub horizon: u32,
    pub required_floors: Vec<String>,
    pub required_action_gates: Vec<String>,
    pub disturbances_by_stage: BTreeMap<u32, Vec<String>>,
    pub states: Vec<ViabilityState>,
    pub actions: Vec<ViabilityAction>,
    pub initial_states: Vec<String>,
}

/// Result for one state, with a witness action if robustly viable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateViability {
    pub state_id: String,
    pub stage: u32,
    pub status: ViabilityStatus,
    pub witness_action: Option<String>,
    pub reasons: Vec<String>,
}

/// Exact report over the declared finite state universe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViabilityReport {
    pub state_results: BTreeMap<String, StateViability>,
    pub robust_viable_initial_states: BTreeSet<String>,
    pub infeasible_initial_states: BTreeSet<String>,
    pub unresolved_initial_states: BTreeSet<String>,
    /// One verifiable witness action for each viable non-terminal state.
    pub witness_policy: BTreeMap<String, String>,
}

/// Problem shape and graph errors. Unknown evidence inside an otherwise
/// well-formed state/action is reflected in ViabilityStatus::Unresolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViabilityError {
    EmptyRequiredFloorSet,
    EmptyStateSet,
    EmptyInitialStateSet,
    DuplicateRequiredFloor(String),
    DuplicateRequiredActionGate(String),
    MissingStageDisturbanceSet(u32),
    EmptyDisturbanceSet(u32),
    DuplicateDisturbance {
        stage: u32,
        disturbance_id: String,
    },
    DuplicateState(String),
    EmptyStateId,
    StateBeyondHorizon {
        state_id: String,
        stage: u32,
    },
    MissingStateFloor {
        state_id: String,
        floor_id: String,
    },
    DuplicateAction(String),
    EmptyActionId,
    UnknownActionSource {
        action_id: String,
        state_id: String,
    },
    ActionFromTerminalState(String),
    UnexpectedDisturbance {
        action_id: String,
        disturbance_id: String,
    },
    UnknownSuccessor {
        action_id: String,
        disturbance_id: String,
        state_id: String,
    },
    NonAdvancingSuccessor {
        action_id: String,
        from_stage: u32,
        to_stage: u32,
    },
    DuplicateInitialState(String),
    UnknownInitialState(String),
    NonAnticipativityViolation {
        decision_time: u32,
        observable_history_id: String,
        actions: Vec<String>,
    },
    TimingOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GateDisposition {
    Passed,
    Failed,
    Unknown(Vec<String>),
}

fn validate_problem(
    problem: &ViabilityProblem,
) -> Result<BTreeMap<String, ViabilityState>, ViabilityError> {
    if problem.required_floors.is_empty() {
        return Err(ViabilityError::EmptyRequiredFloorSet);
    }
    if problem.states.is_empty() {
        return Err(ViabilityError::EmptyStateSet);
    }
    if problem.initial_states.is_empty() {
        return Err(ViabilityError::EmptyInitialStateSet);
    }

    let mut required_floor_set = BTreeSet::new();
    for floor_id in &problem.required_floors {
        if !required_floor_set.insert(floor_id.as_str()) {
            return Err(ViabilityError::DuplicateRequiredFloor(floor_id.clone()));
        }
    }

    let mut required_action_gate_set = BTreeSet::new();
    for gate_id in &problem.required_action_gates {
        if !required_action_gate_set.insert(gate_id.as_str()) {
            return Err(ViabilityError::DuplicateRequiredActionGate(gate_id.clone()));
        }
    }

    for stage in 0..problem.horizon {
        let disturbances = problem
            .disturbances_by_stage
            .get(&stage)
            .ok_or(ViabilityError::MissingStageDisturbanceSet(stage))?;
        if disturbances.is_empty() {
            return Err(ViabilityError::EmptyDisturbanceSet(stage));
        }
        let mut seen = BTreeSet::new();
        for disturbance_id in disturbances {
            if !seen.insert(disturbance_id.as_str()) {
                return Err(ViabilityError::DuplicateDisturbance {
                    stage,
                    disturbance_id: disturbance_id.clone(),
                });
            }
        }
    }

    let mut states = BTreeMap::new();
    for state in &problem.states {
        if state.state_id.is_empty() {
            return Err(ViabilityError::EmptyStateId);
        }
        if state.stage > problem.horizon {
            return Err(ViabilityError::StateBeyondHorizon {
                state_id: state.state_id.clone(),
                stage: state.stage,
            });
        }
        if states.insert(state.state_id.clone(), state.clone()).is_some() {
            return Err(ViabilityError::DuplicateState(state.state_id.clone()));
        }
    }

    for state in &problem.states {
        for floor_id in &problem.required_floors {
            if !state.floors.contains_key(floor_id) {
                return Err(ViabilityError::MissingStateFloor {
                    state_id: state.state_id.clone(),
                    floor_id: floor_id.clone(),
                });
            }
        }
    }

    let mut action_ids = BTreeSet::new();
    for action in &problem.actions {
        if action.action_id.is_empty() {
            return Err(ViabilityError::EmptyActionId);
        }
        if !action_ids.insert(action.action_id.as_str()) {
            return Err(ViabilityError::DuplicateAction(action.action_id.clone()));
        }
        let source = states.get(&action.from_state_id).ok_or_else(|| {
            ViabilityError::UnknownActionSource {
                action_id: action.action_id.clone(),
                state_id: action.from_state_id.clone(),
            }
        })?;
        if source.stage >= problem.horizon {
            return Err(ViabilityError::ActionFromTerminalState(action.action_id.clone()));
        }

        let allowed_disturbances = problem
            .disturbances_by_stage
            .get(&source.stage)
            .expect("stage disturbance set validated above");

        for (disturbance_id, successor_id) in &action.successor_by_disturbance {
            if !allowed_disturbances.iter().any(|allowed| allowed == disturbance_id) {
                return Err(ViabilityError::UnexpectedDisturbance {
                    action_id: action.action_id.clone(),
                    disturbance_id: disturbance_id.clone(),
                });
            }
            let successor = states.get(successor_id).ok_or_else(|| {
                ViabilityError::UnknownSuccessor {
                    action_id: action.action_id.clone(),
                    disturbance_id: disturbance_id.clone(),
                    state_id: successor_id.clone(),
                }
            })?;
            if successor.stage != source.stage + 1 {
                return Err(ViabilityError::NonAdvancingSuccessor {
                    action_id: action.action_id.clone(),
                    from_stage: source.stage,
                    to_stage: successor.stage,
                });
            }
        }
    }

    let mut initial_ids = BTreeSet::new();
    for initial_id in &problem.initial_states {
        if !initial_ids.insert(initial_id.as_str()) {
            return Err(ViabilityError::DuplicateInitialState(initial_id.clone()));
        }
        if !states.contains_key(initial_id) {
            return Err(ViabilityError::UnknownInitialState(initial_id.clone()));
        }
    }

    Ok(states)
}

fn evaluate_state_floors(
    state: &ViabilityState,
    required_floors: &[String],
) -> GateDisposition {
    let mut unknown = Vec::new();
    let mut failed = false;

    for floor_id in required_floors {
        match state.floors.get(floor_id).expect("validated above") {
            GateResult::Pass => {}
            GateResult::Fail => failed = true,
            GateResult::Unknown => unknown.push(floor_id.clone()),
        }
    }

    if failed {
        GateDisposition::Failed
    } else if unknown.is_empty() {
        GateDisposition::Passed
    } else {
        GateDisposition::Unknown(unknown)
    }
}

fn evaluate_action_gates(
    action: &ViabilityAction,
    required_action_gates: &[String],
) -> GateDisposition {
    let mut unknown = Vec::new();
    let mut failed = false;

    for gate_id in required_action_gates {
        match action.gates.get(gate_id) {
            Some(GateResult::Pass) => {}
            Some(GateResult::Fail) => failed = true,
            Some(GateResult::Unknown) | None => unknown.push(gate_id.clone()),
        }
    }

    if failed {
        GateDisposition::Failed
    } else if unknown.is_empty() {
        GateDisposition::Passed
    } else {
        GateDisposition::Unknown(unknown)
    }
}

/// Compute the finite robust-viability kernel by backward recursion.
///
/// At state s and stage t < T, the state is Viable iff its mandatory floors
/// pass and there exists an action whose gates pass and whose successor is
/// Viable for every declared disturbance. If no such action exists but an
/// otherwise plausible action remains unknown, the state is Unresolved.
pub fn compute_viability_kernel(
    problem: &ViabilityProblem,
) -> Result<ViabilityReport, ViabilityError> {
    validate_problem(problem)?;

    let mut actions_by_state: BTreeMap<String, Vec<&ViabilityAction>> = BTreeMap::new();
    for action in &problem.actions {
        actions_by_state
            .entry(action.from_state_id.clone())
            .or_default()
            .push(action);
    }

    let mut ordered_states: Vec<&ViabilityState> = problem.states.iter().collect();
    ordered_states.sort_by(|left, right| {
        right
            .stage
            .cmp(&left.stage)
            .then_with(|| left.state_id.cmp(&right.state_id))
    });

    let mut state_results: BTreeMap<String, StateViability> = BTreeMap::new();
    let mut witness_policy = BTreeMap::new();

    for state in ordered_states {
        match evaluate_state_floors(state, &problem.required_floors) {
            GateDisposition::Failed => {
                state_results.insert(
                    state.state_id.clone(),
                    StateViability {
                        state_id: state.state_id.clone(),
                        stage: state.stage,
                        status: ViabilityStatus::Infeasible,
                        witness_action: None,
                        reasons: vec!["mandatory_state_floor_failed".to_owned()],
                    },
                );
                continue;
            }
            GateDisposition::Unknown(unknowns) => {
                state_results.insert(
                    state.state_id.clone(),
                    StateViability {
                        state_id: state.state_id.clone(),
                        stage: state.stage,
                        status: ViabilityStatus::Unresolved,
                        witness_action: None,
                        reasons: unknowns
                            .into_iter()
                            .map(|id| format!("unknown_state_floor:{id}"))
                            .collect(),
                    },
                );
                continue;
            }
            GateDisposition::Passed => {}
        }

        if state.stage == problem.horizon {
            state_results.insert(
                state.state_id.clone(),
                StateViability {
                    state_id: state.state_id.clone(),
                    stage: state.stage,
                    status: ViabilityStatus::Viable,
                    witness_action: None,
                    reasons: vec!["terminal_floors_pass".to_owned()],
                },
            );
            continue;
        }

        let disturbance_ids = problem
            .disturbances_by_stage
            .get(&state.stage)
            .expect("stage disturbance set validated above");
        let actions = actions_by_state
            .get(&state.state_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let mut unknown_reasons = BTreeSet::new();
        let mut infeasible_reasons = BTreeSet::new();
        let mut viable_witness: Option<String> = None;

        for action in actions {
            match evaluate_action_gates(action, &problem.required_action_gates) {
                GateDisposition::Failed => {
                    infeasible_reasons.insert(format!("action_gate_failed:{}", action.action_id));
                    continue;
                }
                GateDisposition::Unknown(unknowns) => {
                    for gate_id in unknowns {
                        unknown_reasons.insert(format!(
                            "unknown_action_gate:{}:{gate_id}",
                            action.action_id
                        ));
                    }
                    continue;
                }
                GateDisposition::Passed => {}
            }

            let mut action_has_known_failure = false;
            let mut action_has_unknown = false;
            let mut action_unknown_reasons = BTreeSet::new();

            for disturbance_id in disturbance_ids {
                let Some(successor_id) =
                    action.successor_by_disturbance.get(disturbance_id)
                else {
                    action_has_unknown = true;
                    action_unknown_reasons.insert(format!(
                        "missing_disturbance_successor:{}:{disturbance_id}",
                        action.action_id
                    ));
                    continue;
                };

                let successor = state_results
                    .get(successor_id)
                    .expect("successors are exactly one stage later and processed first");

                match successor.status {
                    ViabilityStatus::Viable => {}
                    ViabilityStatus::Infeasible => {
                        action_has_known_failure = true;
                        infeasible_reasons.insert(format!(
                            "successor_infeasible:{}:{disturbance_id}",
                            successor_id
                        ));
                    }
                    ViabilityStatus::Unresolved => {
                        action_has_unknown = true;
                        action_unknown_reasons.insert(format!(
                            "successor_unresolved:{}:{disturbance_id}",
                            successor_id
                        ));
                    }
                }
            }

            // A single known failing disturbance definitively disqualifies
            // this action; its other missing branches do not make it viable.
            if action_has_known_failure {
                continue;
            }
            if action_has_unknown {
                unknown_reasons.extend(action_unknown_reasons);
                continue;
            }

            viable_witness = Some(action.action_id.clone());
            break;
        }

        let (status, witness_action, reasons) = if let Some(action_id) = viable_witness {
            (ViabilityStatus::Viable, Some(action_id), Vec::new())
        } else if !unknown_reasons.is_empty() {
            (
                ViabilityStatus::Unresolved,
                None,
                unknown_reasons.into_iter().collect(),
            )
        } else {
            let reasons = if infeasible_reasons.is_empty() {
                vec!["no_admissible_transition_action".to_owned()]
            } else {
                infeasible_reasons.into_iter().collect()
            };
            (ViabilityStatus::Infeasible, None, reasons)
        };

        if let Some(action_id) = &witness_action {
            witness_policy.insert(state.state_id.clone(), action_id.clone());
        }

        state_results.insert(
            state.state_id.clone(),
            StateViability {
                state_id: state.state_id.clone(),
                stage: state.stage,
                status,
                witness_action,
                reasons,
            },
        );
    }

    let mut robust_viable_initial_states = BTreeSet::new();
    let mut infeasible_initial_states = BTreeSet::new();
    let mut unresolved_initial_states = BTreeSet::new();

    for initial_id in &problem.initial_states {
        let result = state_results
            .get(initial_id)
            .expect("initial state presence validated above");
        match result.status {
            ViabilityStatus::Viable => {
                robust_viable_initial_states.insert(initial_id.clone());
            }
            ViabilityStatus::Infeasible => {
                infeasible_initial_states.insert(initial_id.clone());
            }
            ViabilityStatus::Unresolved => {
                unresolved_initial_states.insert(initial_id.clone());
            }
        }
    }

    Ok(ViabilityReport {
        state_results,
        robust_viable_initial_states,
        infeasible_initial_states,
        unresolved_initial_states,
        witness_policy,
    })
}

/// A policy choice at one observable decision history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub decision_time: u32,
    pub observable_history_id: String,
    pub action_id: String,
}

/// Identical observable histories at the same time must receive identical
/// actions. Hidden future scenario identity is deliberately not accepted.
pub fn validate_non_anticipativity(
    decisions: &[PolicyDecision],
) -> Result<(), ViabilityError> {
    let mut actions_by_history: BTreeMap<(u32, String), BTreeSet<String>> = BTreeMap::new();

    for decision in decisions {
        actions_by_history
            .entry((
                decision.decision_time,
                decision.observable_history_id.clone(),
            ))
            .or_default()
            .insert(decision.action_id.clone());
    }

    for ((decision_time, observable_history_id), actions) in actions_by_history {
        if actions.len() > 1 {
            return Err(ViabilityError::NonAnticipativityViolation {
                decision_time,
                observable_history_id,
                actions: actions.into_iter().collect(),
            });
        }
    }

    Ok(())
}

/// Timing result for a delayed signpost and finite-duration action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionLeadTimeReport {
    pub observation_time: u32,
    pub completion_time: u32,
    pub deadline: u32,
    pub feasible: bool,
}

/// Test whether an action completes by an explicit service-floor deadline.
/// Overflow is rejected, not wrapped into an artificially early time.
pub fn evaluate_action_lead_time(
    signal_crossing_time: u32,
    observation_delay: u32,
    action_lead_time: u32,
    service_floor_deadline: u32,
) -> Result<ActionLeadTimeReport, ViabilityError> {
    let observation_time = signal_crossing_time
        .checked_add(observation_delay)
        .ok_or(ViabilityError::TimingOverflow)?;
    let completion_time = observation_time
        .checked_add(action_lead_time)
        .ok_or(ViabilityError::TimingOverflow)?;

    Ok(ActionLeadTimeReport {
        observation_time,
        completion_time,
        deadline: service_floor_deadline,
        feasible: completion_time <= service_floor_deadline,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor_map(values: &[(&str, GateResult)]) -> BTreeMap<String, GateResult> {
        values
            .iter()
            .map(|(id, result)| ((*id).to_owned(), *result))
            .collect()
    }

    fn state(id: &str, stage: u32, floor_values: &[(&str, GateResult)]) -> ViabilityState {
        ViabilityState {
            state_id: id.to_owned(),
            stage,
            floors: floor_map(floor_values),
        }
    }

    fn action(
        id: &str,
        from: &str,
        action_gates: &[(&str, GateResult)],
        successors: &[(&str, &str)],
    ) -> ViabilityAction {
        ViabilityAction {
            action_id: id.to_owned(),
            from_state_id: from.to_owned(),
            gates: floor_map(action_gates),
            successor_by_disturbance: successors
                .iter()
                .map(|(disturbance, successor)| {
                    ((*disturbance).to_owned(), (*successor).to_owned())
                })
                .collect(),
        }
    }

    fn base_problem(
        horizon: u32,
        states: Vec<ViabilityState>,
        actions: Vec<ViabilityAction>,
        initial_states: &[&str],
    ) -> ViabilityProblem {
        ViabilityProblem {
            horizon,
            required_floors: vec!["freight".to_owned()],
            required_action_gates: vec!["authority".to_owned()],
            disturbances_by_stage: (0..horizon)
                .map(|stage| (stage, vec!["none".to_owned()]))
                .collect(),
            states,
            actions,
            initial_states: initial_states.iter().map(|id| (*id).to_owned()).collect(),
        }
    }

    #[test]
    fn dvk_001_single_path_preserves_service_to_terminal_state() {
        let problem = base_problem(
            1,
            vec![
                state("start", 0, &[("freight", GateResult::Pass)]),
                state("commissioned", 1, &[("freight", GateResult::Pass)]),
            ],
            vec![action(
                "commission",
                "start",
                &[("authority", GateResult::Pass)],
                &[("none", "commissioned")],
            )],
            &["start"],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.robust_viable_initial_states.contains("start"));
        assert_eq!(report.witness_policy.get("start"), Some(&"commission".to_owned()));
    }

    #[test]
    fn dvk_002_two_period_buffer_is_viable_but_one_period_is_not() {
        let problem = base_problem(
            2,
            vec![
                state("buffer1_start", 0, &[("freight", GateResult::Pass)]),
                state("buffer1_exhausted", 1, &[("freight", GateResult::Fail)]),
                state("buffer2_start", 0, &[("freight", GateResult::Pass)]),
                state("buffer2_mid", 1, &[("freight", GateResult::Pass)]),
                state("new_service", 2, &[("freight", GateResult::Pass)]),
            ],
            vec![
                action(
                    "relocate_buffer1",
                    "buffer1_start",
                    &[("authority", GateResult::Pass)],
                    &[("none", "buffer1_exhausted")],
                ),
                action(
                    "continue_buffer2",
                    "buffer2_start",
                    &[("authority", GateResult::Pass)],
                    &[("none", "buffer2_mid")],
                ),
                action(
                    "commission_replacement",
                    "buffer2_mid",
                    &[("authority", GateResult::Pass)],
                    &[("none", "new_service")],
                ),
            ],
            &["buffer1_start", "buffer2_start"],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("buffer1_start"));
        assert!(report.robust_viable_initial_states.contains("buffer2_start"));
    }

    #[test]
    fn dvk_003_identical_observations_cannot_select_different_actions() {
        let decisions = vec![
            PolicyDecision {
                decision_time: 0,
                observable_history_id: "same-observation".to_owned(),
                action_id: "relocate_coast".to_owned(),
            },
            PolicyDecision {
                decision_time: 0,
                observable_history_id: "same-observation".to_owned(),
                action_id: "relocate_inland".to_owned(),
            },
        ];

        assert_eq!(
            validate_non_anticipativity(&decisions),
            Err(ViabilityError::NonAnticipativityViolation {
                decision_time: 0,
                observable_history_id: "same-observation".to_owned(),
                actions: vec!["relocate_coast".to_owned(), "relocate_inland".to_owned()],
            })
        );
    }

    #[test]
    fn dvk_004_shared_dependency_breaks_robust_viability() {
        let mut problem = base_problem(
            1,
            vec![
                state("operating", 0, &[("freight", GateResult::Pass)]),
                state("routes_available", 1, &[("freight", GateResult::Pass)]),
                state("no_route_service_lost", 1, &[("freight", GateResult::Fail)]),
            ],
            vec![action(
                "operate_two_nominal_routes",
                "operating",
                &[("authority", GateResult::Pass)],
                &[
                    ("power_ok", "routes_available"),
                    ("power_fail", "no_route_service_lost"),
                ],
            )],
            &["operating"],
        );
        problem.disturbances_by_stage.insert(
            0,
            vec!["power_ok".to_owned(), "power_fail".to_owned()],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("operating"));
        assert!(report.robust_viable_initial_states.is_empty());
        assert_eq!(
            report.state_results["routes_available"].status,
            ViabilityStatus::Viable
        );
    }

    #[test]
    fn dvk_005_failed_authority_gate_removes_transition_action() {
        let problem = base_problem(
            1,
            vec![
                state("current", 0, &[("freight", GateResult::Pass)]),
                state("terminal_failure", 1, &[("freight", GateResult::Fail)]),
            ],
            vec![action(
                "relocate",
                "current",
                &[("authority", GateResult::Fail)],
                &[("none", "terminal_failure")],
            )],
            &["current"],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("current"));
    }

    #[test]
    fn dvk_006_freight_cannot_compensate_for_ecological_floor_failure() {
        let mut problem = base_problem(
            1,
            vec![
                ViabilityState {
                    state_id: "before".to_owned(),
                    stage: 0,
                    floors: floor_map(&[
                        ("freight", GateResult::Pass),
                        ("environmental_flow", GateResult::Pass),
                    ]),
                },
                ViabilityState {
                    state_id: "after".to_owned(),
                    stage: 1,
                    floors: floor_map(&[
                        ("freight", GateResult::Pass),
                        ("environmental_flow", GateResult::Fail),
                    ]),
                },
            ],
            vec![action(
                "divert_water_to_navigation",
                "before",
                &[("authority", GateResult::Pass)],
                &[("none", "after")],
            )],
            &["before"],
        );
        problem.required_floors.push("environmental_flow".to_owned());

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("before"));
        assert_eq!(
            report.state_results["after"].status,
            ViabilityStatus::Infeasible
        );
    }

    #[test]
    fn dvk_007_success_on_one_branch_is_not_robust_viability() {
        let mut problem = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("normal_terminal", 1, &[("freight", GateResult::Pass)]),
                state("compound_failure", 1, &[("freight", GateResult::Fail)]),
            ],
            vec![action(
                "single_plan",
                "initial",
                &[("authority", GateResult::Pass)],
                &[
                    ("normal", "normal_terminal"),
                    ("compound", "compound_failure"),
                ],
            )],
            &["initial"],
        );
        problem.disturbances_by_stage.insert(
            0,
            vec!["normal".to_owned(), "compound".to_owned()],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("initial"));
        assert_eq!(
            report.state_results["normal_terminal"].status,
            ViabilityStatus::Viable
        );
        assert_eq!(
            report.state_results["compound_failure"].status,
            ViabilityStatus::Infeasible
        );
    }

    #[test]
    fn dvk_008_future_terminal_does_not_rescue_intermediate_service_gap() {
        let problem = base_problem(
            2,
            vec![
                state("old_service", 0, &[("freight", GateResult::Pass)]),
                state("gap_period", 1, &[("freight", GateResult::Fail)]),
                state("new_service", 2, &[("freight", GateResult::Pass)]),
            ],
            vec![
                action(
                    "retire_old",
                    "old_service",
                    &[("authority", GateResult::Pass)],
                    &[("none", "gap_period")],
                ),
                action(
                    "commission_new",
                    "gap_period",
                    &[("authority", GateResult::Pass)],
                    &[("none", "new_service")],
                ),
            ],
            &["old_service"],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("old_service"));
        assert_eq!(
            report.state_results["new_service"].status,
            ViabilityStatus::Viable
        );
    }

    #[test]
    fn dvk_010_adding_disturbance_cannot_enlarge_robust_kernel() {
        let mut base = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("service_ok", 1, &[("freight", GateResult::Pass)]),
                state("service_lost", 1, &[("freight", GateResult::Fail)]),
            ],
            vec![action(
                "operate",
                "initial",
                &[("authority", GateResult::Pass)],
                &[("normal", "service_ok")],
            )],
            &["initial"],
        );
        base.disturbances_by_stage.insert(0, vec!["normal".to_owned()]);
        let base_report = compute_viability_kernel(&base).unwrap();
        assert!(base_report.robust_viable_initial_states.contains("initial"));

        let mut expanded = base;
        expanded.disturbances_by_stage.insert(
            0,
            vec!["normal".to_owned(), "power_fail".to_owned()],
        );
        expanded.actions[0]
            .successor_by_disturbance
            .insert("power_fail".to_owned(), "service_lost".to_owned());
        let expanded_report = compute_viability_kernel(&expanded).unwrap();
        assert!(expanded_report.infeasible_initial_states.contains("initial"));
        assert!(!expanded_report.robust_viable_initial_states.contains("initial"));
    }

    #[test]
    fn dvk_011_delayed_detection_completes_after_service_deadline() {
        let report = evaluate_action_lead_time(4, 2, 3, 8).unwrap();
        assert_eq!(report.observation_time, 6);
        assert_eq!(report.completion_time, 9);
        assert_eq!(report.deadline, 8);
        assert!(!report.feasible);
    }

    #[test]
    fn missing_disturbance_branch_makes_state_unresolved() {
        let mut problem = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("normal_terminal", 1, &[("freight", GateResult::Pass)]),
            ],
            vec![action(
                "partial_action",
                "initial",
                &[("authority", GateResult::Pass)],
                &[("normal", "normal_terminal")],
            )],
            &["initial"],
        );
        problem.disturbances_by_stage.insert(
            0,
            vec!["normal".to_owned(), "unknown_branch".to_owned()],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.unresolved_initial_states.contains("initial"));
        assert!(!report.robust_viable_initial_states.contains("initial"));
    }

    #[test]
    fn known_failing_branch_dominates_missing_branch_for_same_action() {
        let mut problem = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("success", 1, &[("freight", GateResult::Pass)]),
                state("failure", 1, &[("freight", GateResult::Fail)]),
            ],
            vec![action(
                "partly_known_bad_action",
                "initial",
                &[("authority", GateResult::Pass)],
                &[("normal", "success"), ("power_fail", "failure")],
            )],
            &["initial"],
        );
        problem.disturbances_by_stage.insert(
            0,
            vec![
                "normal".to_owned(),
                "power_fail".to_owned(),
                "unobserved".to_owned(),
            ],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.infeasible_initial_states.contains("initial"));
        assert!(!report.unresolved_initial_states.contains("initial"));
    }

    #[test]
    fn unknown_action_gate_does_not_become_admissible() {
        let problem = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("terminal", 1, &[("freight", GateResult::Pass)]),
            ],
            vec![action(
                "unknown_authority",
                "initial",
                &[("authority", GateResult::Unknown)],
                &[("none", "terminal")],
            )],
            &["initial"],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.unresolved_initial_states.contains("initial"));
    }

    #[test]
    fn unknown_state_floor_produces_unresolved_not_viable() {
        let problem = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("terminal", 1, &[("freight", GateResult::Unknown)]),
            ],
            vec![action(
                "commission",
                "initial",
                &[("authority", GateResult::Pass)],
                &[("none", "terminal")],
            )],
            &["initial"],
        );

        let report = compute_viability_kernel(&problem).unwrap();
        assert!(report.unresolved_initial_states.contains("initial"));
    }

    #[test]
    fn state_and_action_order_do_not_change_viability() {
        let problem = base_problem(
            1,
            vec![
                state("initial", 0, &[("freight", GateResult::Pass)]),
                state("good", 1, &[("freight", GateResult::Pass)]),
                state("bad", 1, &[("freight", GateResult::Fail)]),
            ],
            vec![
                action(
                    "bad_action",
                    "initial",
                    &[("authority", GateResult::Pass)],
                    &[("none", "bad")],
                ),
                action(
                    "good_action",
                    "initial",
                    &[("authority", GateResult::Pass)],
                    &[("none", "good")],
                ),
            ],
            &["initial"],
        );

        let first = compute_viability_kernel(&problem).unwrap();
        let mut reversed = problem.clone();
        reversed.states.reverse();
        reversed.actions.reverse();
        let second = compute_viability_kernel(&reversed).unwrap();
        assert_eq!(first.state_results, second.state_results);
        assert_eq!(first.witness_policy, second.witness_policy);
    }

    #[test]
    fn action_finishing_exactly_at_deadline_is_feasible() {
        let report = evaluate_action_lead_time(4, 1, 3, 8).unwrap();
        assert_eq!(report.completion_time, 8);
        assert!(report.feasible);
    }

    #[test]
    fn timing_overflow_fails_closed() {
        assert_eq!(
            evaluate_action_lead_time(u32::MAX, 1, 1, u32::MAX),
            Err(ViabilityError::TimingOverflow)
        );
    }
}
