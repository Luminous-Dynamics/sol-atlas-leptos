// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Finite-horizon robust viability over observable information sets.
//!
//! Unlike the perfect-information state kernels, this solver chooses one
//! action for an entire belief (the set of hidden states consistent with the
//! observations available now). It groups successors by observable-history
//! identifiers, never by hidden future labels.
//!
//! The belief graph is explicit and finite. A missing transition or successor
//! belief is UNRESOLVED, not an optimistic pass. This module is a reference
//! semantics, not a probabilistic POMDP solver or a physical model.

use std::collections::{BTreeMap, BTreeSet};

use crate::viability_kernel::GateResult;

/// One hidden physical/model state. No future scenario identity is exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenWorldState {
    pub state_id: String,
    pub stage: u32,
    pub floors: BTreeMap<String, GateResult>,
}

/// One information state: all hidden states still consistent with the
/// observations available to the controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefState {
    pub belief_id: String,
    pub stage: u32,
    pub observable_history_id: String,
    pub hidden_state_ids: BTreeSet<String>,
}

/// One action selected once for the entire belief. Its mandatory gates must
/// pass for every hidden state in that belief.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefAction {
    pub action_id: String,
    pub from_belief_id: String,
    pub gates_by_hidden_state: BTreeMap<String, BTreeMap<String, GateResult>>,
}

/// One possible transition for one hidden state, common action and
/// disturbance. The next observable-history ID is what the controller sees;
/// the hidden successor ID is used only to maintain the belief set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefTransition {
    pub from_hidden_state_id: String,
    pub action_id: String,
    pub disturbance_id: String,
    pub successor_hidden_state_id: String,
    pub next_observable_history_id: String,
}

/// Finite partial-observation robust viability problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefViabilityProblem {
    pub horizon: u32,
    pub required_floors: Vec<String>,
    pub required_action_gates: Vec<String>,
    pub disturbances_by_stage: BTreeMap<u32, Vec<String>>,
    pub hidden_states: Vec<HiddenWorldState>,
    pub beliefs: Vec<BeliefState>,
    pub actions: Vec<BeliefAction>,
    pub transitions: Vec<BeliefTransition>,
    pub initial_beliefs: Vec<String>,
}

/// Viability status for one observable information state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeliefStatus {
    Viable,
    Infeasible,
    Unresolved,
}

/// Result for one belief node, including a shared action witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefResult {
    pub belief_id: String,
    pub status: BeliefStatus,
    pub witness_action: Option<String>,
    pub reasons: Vec<String>,
}

/// Exact finite-horizon result over the declared belief graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefViabilityReport {
    pub results: BTreeMap<String, BeliefResult>,
    pub viable_initial_beliefs: BTreeSet<String>,
    pub infeasible_initial_beliefs: BTreeSet<String>,
    pub unresolved_initial_beliefs: BTreeSet<String>,
    /// One non-anticipative action per observable belief node.
    pub witness_policy: BTreeMap<String, String>,
}

/// Resource limits for the finite belief graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeliefViabilityLimits {
    pub max_hidden_states: usize,
    pub max_beliefs: usize,
    pub max_actions: usize,
    pub max_transition_rows: usize,
    pub max_belief_expansion_branches: usize,
}

impl Default for BeliefViabilityLimits {
    fn default() -> Self {
        Self {
            max_hidden_states: 50_000,
            max_beliefs: 100_000,
            max_actions: 250_000,
            max_transition_rows: 2_000_000,
            max_belief_expansion_branches: 4_000_000,
        }
    }
}

/// A bounded result has no partial belief classifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeliefViabilityExecution {
    Exact(BeliefViabilityReport),
    Bounded {
        resource: String,
        observed: usize,
        limit: usize,
    },
}

/// Structural graph errors. Missing transition rows or matching successor
/// beliefs are data gaps and produce UNRESOLVED in the result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeliefViabilityError {
    EmptyRequiredFloorSet,
    EmptyHiddenStateSet,
    EmptyBeliefSet,
    EmptyInitialBeliefSet,
    EmptyDisturbanceSet(u32),
    MissingStageDisturbanceSet(u32),
    DuplicateFloor(String),
    DuplicateActionGate(String),
    DuplicateDisturbance(String),
    EmptyHiddenStateId,
    DuplicateHiddenState(String),
    HiddenStateBeyondHorizon {
        state_id: String,
        stage: u32,
    },
    MissingHiddenStateFloor {
        state_id: String,
        floor_id: String,
    },
    EmptyBeliefId,
    DuplicateBelief(String),
    EmptyObservableHistory(String),
    EmptyBeliefMembers(String),
    DuplicateBeliefMembership(String),
    UnknownBeliefMember {
        belief_id: String,
        state_id: String,
    },
    BeliefMemberWrongStage {
        belief_id: String,
        state_id: String,
    },
    DuplicateObservableHistory {
        stage: u32,
        history_id: String,
    },
    EmptyActionId,
    DuplicateAction(String),
    UnknownActionBelief {
        action_id: String,
        belief_id: String,
    },
    ActionFromTerminalBelief(String),
    UnknownActionGateState {
        action_id: String,
        state_id: String,
    },
    DuplicateTransition {
        state_id: String,
        action_id: String,
        disturbance_id: String,
    },
    UnknownTransitionState(String),
    UnknownTransitionSuccessor(String),
    UnknownTransitionDisturbance(String),
    UnknownTransitionAction(String),
    NonAdvancingTransition {
        from_stage: u32,
        to_stage: u32,
    },
    DuplicateInitialBelief(String),
    UnknownInitialBelief(String),
    ResourceLimitOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EvidenceDisposition {
    Pass,
    Fail,
    Unknown(Vec<String>),
}

fn evaluate_floor_map(
    floors: &BTreeMap<String, GateResult>,
    required: &[String],
) -> EvidenceDisposition {
    let mut unknown = Vec::new();
    let mut failed = false;

    for floor_id in required {
        match floors.get(floor_id) {
            Some(GateResult::Pass) => {}
            Some(GateResult::Fail) => failed = true,
            Some(GateResult::Unknown) | None => unknown.push(floor_id.clone()),
        }
    }

    if failed {
        EvidenceDisposition::Fail
    } else if unknown.is_empty() {
        EvidenceDisposition::Pass
    } else {
        EvidenceDisposition::Unknown(unknown)
    }
}

fn validate_problem(
    problem: &BeliefViabilityProblem,
) -> Result<
    (
        BTreeMap<String, HiddenWorldState>,
        BTreeMap<String, BeliefState>,
        BTreeMap<String, Vec<&BeliefAction>>,
        BTreeMap<(String, String, String), &BeliefTransition>,
        BTreeMap<(u32, String), String>,
    ),
    BeliefViabilityError,
> {
    if problem.required_floors.is_empty() {
        return Err(BeliefViabilityError::EmptyRequiredFloorSet);
    }
    if problem.hidden_states.is_empty() {
        return Err(BeliefViabilityError::EmptyHiddenStateSet);
    }
    if problem.beliefs.is_empty() {
        return Err(BeliefViabilityError::EmptyBeliefSet);
    }
    if problem.initial_beliefs.is_empty() {
        return Err(BeliefViabilityError::EmptyInitialBeliefSet);
    }

    let mut floor_ids = BTreeSet::new();
    for id in &problem.required_floors {
        if !floor_ids.insert(id.as_str()) {
            return Err(BeliefViabilityError::DuplicateFloor(id.clone()));
        }
    }
    let mut action_gate_ids = BTreeSet::new();
    for id in &problem.required_action_gates {
        if !action_gate_ids.insert(id.as_str()) {
            return Err(BeliefViabilityError::DuplicateActionGate(id.clone()));
        }
    }

    for stage in 0..problem.horizon {
        let disturbances = problem
            .disturbances_by_stage
            .get(&stage)
            .ok_or(BeliefViabilityError::MissingStageDisturbanceSet(stage))?;
        if disturbances.is_empty() {
            return Err(BeliefViabilityError::EmptyDisturbanceSet(stage));
        }
        let mut seen = BTreeSet::new();
        for disturbance in disturbances {
            if !seen.insert(disturbance.as_str()) {
                return Err(BeliefViabilityError::DuplicateDisturbance(
                    disturbance.clone(),
                ));
            }
        }
    }

    let mut hidden_states = BTreeMap::new();
    for state in &problem.hidden_states {
        if state.state_id.is_empty() {
            return Err(BeliefViabilityError::EmptyHiddenStateId);
        }
        if state.stage > problem.horizon {
            return Err(BeliefViabilityError::HiddenStateBeyondHorizon {
                state_id: state.state_id.clone(),
                stage: state.stage,
            });
        }
        if hidden_states
            .insert(state.state_id.clone(), state.clone())
            .is_some()
        {
            return Err(BeliefViabilityError::DuplicateHiddenState(
                state.state_id.clone(),
            ));
        }
    }
    for state in &problem.hidden_states {
        for floor_id in &problem.required_floors {
            if !state.floors.contains_key(floor_id) {
                return Err(BeliefViabilityError::MissingHiddenStateFloor {
                    state_id: state.state_id.clone(),
                    floor_id: floor_id.clone(),
                });
            }
        }
    }

    let mut beliefs = BTreeMap::new();
    let mut belief_by_history = BTreeMap::new();
    for belief in &problem.beliefs {
        if belief.belief_id.is_empty() {
            return Err(BeliefViabilityError::EmptyBeliefId);
        }
        if belief.observable_history_id.is_empty() {
            return Err(BeliefViabilityError::EmptyObservableHistory(
                belief.belief_id.clone(),
            ));
        }
        if belief.hidden_state_ids.is_empty() {
            return Err(BeliefViabilityError::EmptyBeliefMembers(
                belief.belief_id.clone(),
            ));
        }
        let key = (belief.stage, belief.observable_history_id.clone());
        if belief_by_history
            .insert(key.clone(), belief.belief_id.clone())
            .is_some()
        {
            return Err(BeliefViabilityError::DuplicateObservableHistory {
                stage: key.0,
                history_id: key.1,
            });
        }
        if beliefs
            .insert(belief.belief_id.clone(), belief.clone())
            .is_some()
        {
            return Err(BeliefViabilityError::DuplicateBelief(
                belief.belief_id.clone(),
            ));
        }
        for state_id in &belief.hidden_state_ids {
            let state = hidden_states.get(state_id).ok_or_else(|| {
                BeliefViabilityError::UnknownBeliefMember {
                    belief_id: belief.belief_id.clone(),
                    state_id: state_id.clone(),
                }
            })?;
            if state.stage != belief.stage {
                return Err(BeliefViabilityError::BeliefMemberWrongStage {
                    belief_id: belief.belief_id.clone(),
                    state_id: state_id.clone(),
                });
            }
        }
    }

    let mut actions_by_belief: BTreeMap<String, Vec<&BeliefAction>> = BTreeMap::new();
    let mut actions_by_id: BTreeMap<String, &BeliefAction> = BTreeMap::new();
    let mut action_ids = BTreeSet::new();
    for action in &problem.actions {
        if action.action_id.is_empty() {
            return Err(BeliefViabilityError::EmptyActionId);
        }
        if !action_ids.insert(action.action_id.as_str()) {
            return Err(BeliefViabilityError::DuplicateAction(
                action.action_id.clone(),
            ));
        }
        let belief = beliefs.get(&action.from_belief_id).ok_or_else(|| {
            BeliefViabilityError::UnknownActionBelief {
                action_id: action.action_id.clone(),
                belief_id: action.from_belief_id.clone(),
            }
        })?;
        if belief.stage >= problem.horizon {
            return Err(BeliefViabilityError::ActionFromTerminalBelief(
                action.action_id.clone(),
            ));
        }
        for state_id in action.gates_by_hidden_state.keys() {
            if !belief.hidden_state_ids.contains(state_id) {
                return Err(BeliefViabilityError::UnknownActionGateState {
                    action_id: action.action_id.clone(),
                    state_id: state_id.clone(),
                });
            }
        }
        actions_by_id.insert(action.action_id.clone(), action);
        actions_by_belief
            .entry(action.from_belief_id.clone())
            .or_default()
            .push(action);
    }
    for actions in actions_by_belief.values_mut() {
        actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    }

    let mut transitions = BTreeMap::new();
    for transition in &problem.transitions {
        let from_state = hidden_states
            .get(&transition.from_hidden_state_id)
            .ok_or_else(|| {
                BeliefViabilityError::UnknownTransitionState(
                    transition.from_hidden_state_id.clone(),
                )
            })?;
        let successor = hidden_states
            .get(&transition.successor_hidden_state_id)
            .ok_or_else(|| {
                BeliefViabilityError::UnknownTransitionSuccessor(
                    transition.successor_hidden_state_id.clone(),
                )
            })?;
        if successor.stage != from_state.stage + 1 {
            return Err(BeliefViabilityError::NonAdvancingTransition {
                from_stage: from_state.stage,
                to_stage: successor.stage,
            });
        }
        if !problem
            .disturbances_by_stage
            .get(&from_state.stage)
            .is_some_and(|items| items.contains(&transition.disturbance_id))
        {
            return Err(BeliefViabilityError::UnknownTransitionDisturbance(
                transition.disturbance_id.clone(),
            ));
        }
        let action = actions_by_id
            .get(&transition.action_id)
            .ok_or_else(|| BeliefViabilityError::UnknownTransitionAction(
                transition.action_id.clone(),
            ))?;
        let source_belief = beliefs
            .get(&action.from_belief_id)
            .expect("action belief validated above");
        if !source_belief.hidden_state_ids.contains(&transition.from_hidden_state_id) {
            return Err(BeliefViabilityError::UnknownTransitionState(
                transition.from_hidden_state_id.clone(),
            ));
        }

        let key = (
            transition.from_hidden_state_id.clone(),
            transition.action_id.clone(),
            transition.disturbance_id.clone(),
        );
        if transitions.insert(key.clone(), transition).is_some() {
            return Err(BeliefViabilityError::DuplicateTransition {
                state_id: key.0,
                action_id: key.1,
                disturbance_id: key.2,
            });
        }
    }

    let mut initial_ids = BTreeSet::new();
    for belief_id in &problem.initial_beliefs {
        if !initial_ids.insert(belief_id.as_str()) {
            return Err(BeliefViabilityError::DuplicateInitialBelief(
                belief_id.clone(),
            ));
        }
        if !beliefs.contains_key(belief_id) {
            return Err(BeliefViabilityError::UnknownInitialBelief(
                belief_id.clone(),
            ));
        }
    }

    Ok((
        hidden_states,
        beliefs,
        actions_by_belief,
        transitions,
        belief_by_history,
    ))
}

/// Compute exact finite-horizon robust viability over observable beliefs.
///
/// One action must work for every hidden state in a belief and every
/// disturbance. Successor states are grouped only by next observable-history
/// ID; each resulting group must match a declared successor belief exactly.
pub fn compute_belief_viability_kernel(
    problem: &BeliefViabilityProblem,
) -> Result<BeliefViabilityReport, BeliefViabilityError> {
    let (hidden_states, beliefs, actions_by_belief, transitions, belief_by_history) =
        validate_problem(problem)?;

    let mut ordered_beliefs: Vec<&BeliefState> = problem.beliefs.iter().collect();
    ordered_beliefs.sort_by(|left, right| {
        right
            .stage
            .cmp(&left.stage)
            .then_with(|| left.belief_id.cmp(&right.belief_id))
    });

    let mut results = BTreeMap::new();
    let mut witness_policy = BTreeMap::new();

    for belief in ordered_beliefs {
        let mut floor_failed = false;
        let mut floor_unknown = BTreeSet::new();

        for state_id in &belief.hidden_state_ids {
            let state = hidden_states
                .get(state_id)
                .expect("belief membership validated above");
            match evaluate_floor_map(&state.floors, &problem.required_floors) {
                EvidenceDisposition::Pass => {}
                EvidenceDisposition::Fail => floor_failed = true,
                EvidenceDisposition::Unknown(unknowns) => {
                    floor_unknown.extend(unknowns.into_iter().map(|id| {
                        format!("unknown_floor:{state_id}:{id}")
                    }));
                }
            }
        }

        if floor_failed {
            results.insert(
                belief.belief_id.clone(),
                BeliefResult {
                    belief_id: belief.belief_id.clone(),
                    status: BeliefStatus::Infeasible,
                    witness_action: None,
                    reasons: vec!["mandatory_floor_failed_in_hidden_state".to_owned()],
                },
            );
            continue;
        }
        if !floor_unknown.is_empty() {
            results.insert(
                belief.belief_id.clone(),
                BeliefResult {
                    belief_id: belief.belief_id.clone(),
                    status: BeliefStatus::Unresolved,
                    witness_action: None,
                    reasons: floor_unknown.into_iter().collect(),
                },
            );
            continue;
        }

        if belief.stage == problem.horizon {
            results.insert(
                belief.belief_id.clone(),
                BeliefResult {
                    belief_id: belief.belief_id.clone(),
                    status: BeliefStatus::Viable,
                    witness_action: None,
                    reasons: vec!["all_hidden_terminal_states_pass".to_owned()],
                },
            );
            continue;
        }

        let disturbance_ids = problem
            .disturbances_by_stage
            .get(&belief.stage)
            .expect("stage disturbance set validated above");
        let actions = actions_by_belief
            .get(&belief.belief_id)
            .map(|items| items.as_slice())
            .unwrap_or(&[]);

        let mut unknown_reasons = BTreeSet::new();
        let mut failure_reasons = BTreeSet::new();
        let mut witness = None;

        for action in actions {
            let mut action_failed = false;
            let mut action_unknown = false;
            let mut action_unknown_reasons = BTreeSet::new();

            // The same action ID is checked against every hidden state.
            for state_id in &belief.hidden_state_ids {
                let gate_map = action.gates_by_hidden_state.get(state_id);
                match gate_map {
                    None => {
                        action_unknown = true;
                        action_unknown_reasons.insert(format!(
                            "missing_action_gate_map:{}:{state_id}",
                            action.action_id
                        ));
                    }
                    Some(gates) => {
                        match evaluate_floor_map(gates, &problem.required_action_gates) {
                            EvidenceDisposition::Pass => {}
                            EvidenceDisposition::Fail => {
                                action_failed = true;
                                failure_reasons.insert(format!(
                                    "action_gate_failed:{}:{state_id}",
                                    action.action_id
                                ));
                            }
                            EvidenceDisposition::Unknown(unknowns) => {
                                action_unknown = true;
                                for gate_id in unknowns {
                                    action_unknown_reasons.insert(format!(
                                        "unknown_action_gate:{}:{state_id}:{gate_id}",
                                        action.action_id
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            if action_failed {
                continue;
            }

            let mut successor_beliefs = BTreeMap::<String, BTreeSet<String>>::new();
            let mut action_has_known_failure = false;

            for state_id in &belief.hidden_state_ids {
                for disturbance_id in disturbance_ids {
                    let key = (
                        state_id.clone(),
                        action.action_id.clone(),
                        disturbance_id.clone(),
                    );
                    let Some(transition) = transitions.get(&key) else {
                        action_unknown = true;
                        action_unknown_reasons.insert(format!(
                            "missing_transition:{}:{state_id}:{disturbance_id}",
                            action.action_id
                        ));
                        continue;
                    };

                    let successor_state = hidden_states
                        .get(&transition.successor_hidden_state_id)
                        .expect("transition successor validated above");
                    match evaluate_floor_map(
                        &successor_state.floors,
                        &problem.required_floors,
                    ) {
                        EvidenceDisposition::Fail => {
                            action_has_known_failure = true;
                            failure_reasons.insert(format!(
                                "successor_hidden_state_floor_failed:{}",
                                transition.successor_hidden_state_id
                            ));
                        }
                        EvidenceDisposition::Unknown(unknowns) => {
                            action_unknown = true;
                            for floor_id in unknowns {
                                action_unknown_reasons.insert(format!(
                                    "unknown_successor_floor:{}:{}",
                                    transition.successor_hidden_state_id,
                                    floor_id
                                ));
                            }
                        }
                        EvidenceDisposition::Pass => {}
                    }

                    if transition.next_observable_history_id.is_empty() {
                        action_unknown = true;
                        action_unknown_reasons.insert(format!(
                            "missing_next_observation:{}:{state_id}:{disturbance_id}",
                            action.action_id
                        ));
                        continue;
                    }

                    successor_beliefs
                        .entry(transition.next_observable_history_id.clone())
                        .or_default()
                        .insert(transition.successor_hidden_state_id.clone());
                }
            }
            for (history_id, successor_states) in successor_beliefs {
                let key = (belief.stage + 1, history_id.clone());
                let Some(successor_belief_id) = belief_by_history.get(&key) else {
                    action_unknown = true;
                    action_unknown_reasons.insert(format!(
                        "missing_successor_belief:{}",
                        history_id
                    ));
                    continue;
                };
                let successor_belief = beliefs
                    .get(successor_belief_id)
                    .expect("belief history index references a known belief");

                if successor_belief.hidden_state_ids != successor_states {
                    action_unknown = true;
                    action_unknown_reasons.insert(format!(
                        "successor_belief_membership_mismatch:{}",
                        successor_belief_id
                    ));
                    continue;
                }

                let successor_result = results
                    .get(successor_belief_id)
                    .expect("next-stage belief is evaluated before this belief");
                match successor_result.status {
                    BeliefStatus::Viable => {}
                    BeliefStatus::Infeasible => {
                        action_has_known_failure = true;
                        failure_reasons.insert(format!(
                            "successor_belief_infeasible:{}",
                            successor_belief_id
                        ));
                    }
                    BeliefStatus::Unresolved => {
                        action_unknown = true;
                        action_unknown_reasons.insert(format!(
                            "successor_belief_unresolved:{}",
                            successor_belief_id
                        ));
                    }
                }
            }

            // A known bad successor disqualifies the action even if another
            // disturbance branch or observation partition is missing.
            if action_has_known_failure {
                continue;
            }
            if action_unknown {
                unknown_reasons.extend(action_unknown_reasons);
                continue;
            }

            witness = Some(action.action_id.clone());
            break;
        }

        let (status, reasons) = if witness.is_some() {
            (BeliefStatus::Viable, Vec::new())
        } else if !unknown_reasons.is_empty() {
            (
                BeliefStatus::Unresolved,
                unknown_reasons.into_iter().collect(),
            )
        } else {
            let reasons = if failure_reasons.is_empty() {
                vec!["no_single_action_preserves_all_hidden_states".to_owned()]
            } else {
                failure_reasons.into_iter().collect()
            };
            (BeliefStatus::Infeasible, reasons)
        };

        if let Some(action_id) = &witness {
            witness_policy.insert(belief.belief_id.clone(), action_id.clone());
        }
        results.insert(
            belief.belief_id.clone(),
            BeliefResult {
                belief_id: belief.belief_id.clone(),
                status,
                witness_action: witness,
                reasons,
            },
        );
    }

    let mut viable_initial_beliefs = BTreeSet::new();
    let mut infeasible_initial_beliefs = BTreeSet::new();
    let mut unresolved_initial_beliefs = BTreeSet::new();
    for belief_id in &problem.initial_beliefs {
        match results
            .get(belief_id)
            .expect("initial belief presence validated above")
            .status
        {
            BeliefStatus::Viable => {
                viable_initial_beliefs.insert(belief_id.clone());
            }
            BeliefStatus::Infeasible => {
                infeasible_initial_beliefs.insert(belief_id.clone());
            }
            BeliefStatus::Unresolved => {
                unresolved_initial_beliefs.insert(belief_id.clone());
            }
        }
    }

    Ok(BeliefViabilityReport {
        results,
        viable_initial_beliefs,
        infeasible_initial_beliefs,
        unresolved_initial_beliefs,
        witness_policy,
    })
}

/// Apply caps before exact belief-state expansion. No partial policy is
/// returned when a cap is reached.
pub fn compute_belief_viability_kernel_with_limits(
    problem: &BeliefViabilityProblem,
    limits: BeliefViabilityLimits,
) -> Result<BeliefViabilityExecution, BeliefViabilityError> {
    let bounded = |resource: &str, observed: usize, limit: usize| {
        BeliefViabilityExecution::Bounded {
            resource: resource.to_owned(),
            observed,
            limit,
        }
    };

    if problem.hidden_states.len() > limits.max_hidden_states {
        return Ok(bounded(
            "hidden_states",
            problem.hidden_states.len(),
            limits.max_hidden_states,
        ));
    }
    if problem.beliefs.len() > limits.max_beliefs {
        return Ok(bounded(
            "beliefs",
            problem.beliefs.len(),
            limits.max_beliefs,
        ));
    }
    if problem.actions.len() > limits.max_actions {
        return Ok(bounded("actions", problem.actions.len(), limits.max_actions));
    }
    if problem.transitions.len() > limits.max_transition_rows {
        return Ok(bounded(
            "transition_rows",
            problem.transitions.len(),
            limits.max_transition_rows,
        ));
    }

    let hidden_belief_branches = problem
        .beliefs
        .iter()
        .try_fold(0usize, |sum, belief| {
            let action_count = problem
                .actions
                .iter()
                .filter(|action| action.from_belief_id == belief.belief_id)
                .count();
            let disturbance_count = problem
                .disturbances_by_stage
                .get(&belief.stage)
                .map(Vec::len)
                .unwrap_or(0);
            let branch_count = belief
                .hidden_state_ids
                .len()
                .checked_mul(disturbance_count)
                .and_then(|value| value.checked_mul(action_count))
                .ok_or(BeliefViabilityError::ResourceLimitOverflow)?;
            sum.checked_add(branch_count)
                .ok_or(BeliefViabilityError::ResourceLimitOverflow)
        })?;

    if hidden_belief_branches > limits.max_belief_expansion_branches {
        return Ok(bounded(
            "belief_expansion_branches",
            hidden_belief_branches,
            limits.max_belief_expansion_branches,
        ));
    }

    Ok(BeliefViabilityExecution::Exact(
        compute_belief_viability_kernel(problem)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(values: &[(&str, GateResult)]) -> BTreeMap<String, GateResult> {
        values
            .iter()
            .map(|(id, value)| ((*id).to_owned(), *value))
            .collect()
    }

    fn hidden(id: &str, stage: u32, floors: &[(&str, GateResult)]) -> HiddenWorldState {
        HiddenWorldState {
            state_id: id.to_owned(),
            stage,
            floors: map(floors),
        }
    }

    fn belief(id: &str, stage: u32, history: &str, members: &[&str]) -> BeliefState {
        BeliefState {
            belief_id: id.to_owned(),
            stage,
            observable_history_id: history.to_owned(),
            hidden_state_ids: members.iter().map(|id| (*id).to_owned()).collect(),
        }
    }

    fn action(
        id: &str,
        belief_id: &str,
        gates: &[(&str, &[(&str, GateResult)])],
    ) -> BeliefAction {
        BeliefAction {
            action_id: id.to_owned(),
            from_belief_id: belief_id.to_owned(),
            gates_by_hidden_state: gates
                .iter()
                .map(|(state_id, results)| ((*state_id).to_owned(), map(results)))
                .collect(),
        }
    }

    fn transition(
        from: &str,
        action_id: &str,
        disturbance: &str,
        to: &str,
        observation_history: &str,
    ) -> BeliefTransition {
        BeliefTransition {
            from_hidden_state_id: from.to_owned(),
            action_id: action_id.to_owned(),
            disturbance_id: disturbance.to_owned(),
            successor_hidden_state_id: to.to_owned(),
            next_observable_history_id: observation_history.to_owned(),
        }
    }

    fn problem(
        horizon: u32,
        hidden_states: Vec<HiddenWorldState>,
        beliefs: Vec<BeliefState>,
        actions: Vec<BeliefAction>,
        transitions: Vec<BeliefTransition>,
        initial: &[&str],
        disturbances: &[&str],
    ) -> BeliefViabilityProblem {
        BeliefViabilityProblem {
            horizon,
            required_floors: vec!["service".to_owned()],
            required_action_gates: vec!["authority".to_owned()],
            disturbances_by_stage: (0..horizon)
                .map(|stage| {
                    (
                        stage,
                        disturbances.iter().map(|id| (*id).to_owned()).collect(),
                    )
                })
                .collect(),
            hidden_states,
            beliefs,
            actions,
            transitions,
            initial_beliefs: initial.iter().map(|id| (*id).to_owned()).collect(),
        }
    }

    #[test]
    fn one_common_action_must_preserve_every_hidden_state() {
        let model = problem(
            1,
            vec![
                hidden("coast", 0, &[("service", GateResult::Pass)]),
                hidden("inland", 0, &[("service", GateResult::Pass)]),
                hidden("coast_ok", 1, &[("service", GateResult::Pass)]),
                hidden("coast_bad", 1, &[("service", GateResult::Fail)]),
                hidden("inland_ok", 1, &[("service", GateResult::Pass)]),
                hidden("inland_bad", 1, &[("service", GateResult::Fail)]),
            ],
            vec![
                belief("initial", 0, "same-observation", &["coast", "inland"]),
                belief("terminal_coast_ok", 1, "coast-ok", &["coast_ok"]),
                belief("terminal_coast_bad", 1, "coast-bad", &["coast_bad"]),
                belief("terminal_inland_ok", 1, "inland-ok", &["inland_ok"]),
                belief("terminal_inland_bad", 1, "inland-bad", &["inland_bad"]),
            ],
            vec![
                action(
                    "choose_coast",
                    "initial",
                    &[
                        ("coast", &[("authority", GateResult::Pass)]),
                        ("inland", &[("authority", GateResult::Pass)]),
                    ],
                ),
                action(
                    "choose_inland",
                    "initial",
                    &[
                        ("coast", &[("authority", GateResult::Pass)]),
                        ("inland", &[("authority", GateResult::Pass)]),
                    ],
                ),
            ],
            vec![
                transition("coast", "choose_coast", "none", "coast_ok", "coast-ok"),
                transition("inland", "choose_coast", "none", "inland_bad", "inland-bad"),
                transition("coast", "choose_inland", "none", "coast_bad", "coast-bad"),
                transition("inland", "choose_inland", "none", "inland_ok", "inland-ok"),
            ],
            &["initial"],
            &["none"],
        );

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.infeasible_initial_beliefs.contains("initial"));
        assert!(!report.viable_initial_beliefs.contains("initial"));
    }

    #[test]
    fn one_common_action_that_preserves_both_hidden_states_is_viable() {
        let model = problem(
            1,
            vec![
                hidden("future_a", 0, &[("service", GateResult::Pass)]),
                hidden("future_b", 0, &[("service", GateResult::Pass)]),
                hidden("terminal_a", 1, &[("service", GateResult::Pass)]),
                hidden("terminal_b", 1, &[("service", GateResult::Pass)]),
            ],
            vec![
                belief("initial", 0, "same-observation", &["future_a", "future_b"]),
                belief("terminal", 1, "same-terminal-observation", &["terminal_a", "terminal_b"]),
            ],
            vec![action(
                "shared_safe_action",
                "initial",
                &[
                    ("future_a", &[("authority", GateResult::Pass)]),
                    ("future_b", &[("authority", GateResult::Pass)]),
                ],
            )],
            vec![
                transition("future_a", "shared_safe_action", "none", "terminal_a", "same-terminal-observation"),
                transition("future_b", "shared_safe_action", "none", "terminal_b", "same-terminal-observation"),
            ],
            &["initial"],
            &["none"],
        );

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.viable_initial_beliefs.contains("initial"));
        assert_eq!(report.witness_policy["initial"], "shared_safe_action");
    }

    #[test]
    fn action_can_branch_only_after_observation_distinguishes_hidden_states() {
        let model = problem(
            2,
            vec![
                hidden("future_a_0", 0, &[("service", GateResult::Pass)]),
                hidden("future_b_0", 0, &[("service", GateResult::Pass)]),
                hidden("future_a_1", 1, &[("service", GateResult::Pass)]),
                hidden("future_b_1", 1, &[("service", GateResult::Pass)]),
                hidden("terminal_a", 2, &[("service", GateResult::Pass)]),
                hidden("terminal_b", 2, &[("service", GateResult::Pass)]),
            ],
            vec![
                belief("initial", 0, "same-before-observation", &["future_a_0", "future_b_0"]),
                belief("observed_a", 1, "observation_a", &["future_a_1"]),
                belief("observed_b", 1, "observation_b", &["future_b_1"]),
                belief("terminal_a", 2, "terminal_history_a", &["terminal_a"]),
                belief("terminal_b", 2, "terminal_history_b", &["terminal_b"]),
            ],
            vec![
                action(
                    "wait_for_observation",
                    "initial",
                    &[
                        ("future_a_0", &[("authority", GateResult::Pass)]),
                        ("future_b_0", &[("authority", GateResult::Pass)]),
                    ],
                ),
                action(
                    "action_a",
                    "observed_a",
                    &[("future_a_1", &[("authority", GateResult::Pass)])],
                ),
                action(
                    "action_b",
                    "observed_b",
                    &[("future_b_1", &[("authority", GateResult::Pass)])],
                ),
            ],
            vec![
                transition("future_a_0", "wait_for_observation", "none", "future_a_1", "observation_a"),
                transition("future_b_0", "wait_for_observation", "none", "future_b_1", "observation_b"),
                transition("future_a_1", "action_a", "none", "terminal_a", "terminal_history_a"),
                transition("future_b_1", "action_b", "none", "terminal_b", "terminal_history_b"),
            ],
            &["initial"],
            &["none"],
        );

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.viable_initial_beliefs.contains("initial"));
        assert_eq!(report.witness_policy["initial"], "wait_for_observation");
        assert_eq!(report.witness_policy["observed_a"], "action_a");
        assert_eq!(report.witness_policy["observed_b"], "action_b");
    }

    #[test]
    fn hidden_states_with_same_observation_are_merged_into_one_belief() {
        let model = problem(
            1,
            vec![
                hidden("hidden_a", 0, &[("service", GateResult::Pass)]),
                hidden("hidden_b", 0, &[("service", GateResult::Pass)]),
                hidden("next_a", 1, &[("service", GateResult::Pass)]),
                hidden("next_b", 1, &[("service", GateResult::Pass)]),
            ],
            vec![
                belief("initial", 0, "history_0", &["hidden_a", "hidden_b"]),
                belief("merged", 1, "same_new_observation", &["next_a", "next_b"]),
            ],
            vec![action(
                "act",
                "initial",
                &[
                    ("hidden_a", &[("authority", GateResult::Pass)]),
                    ("hidden_b", &[("authority", GateResult::Pass)]),
                ],
            )],
            vec![
                transition("hidden_a", "act", "none", "next_a", "same_new_observation"),
                transition("hidden_b", "act", "none", "next_b", "same_new_observation"),
            ],
            &["initial"],
            &["none"],
        );

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.viable_initial_beliefs.contains("initial"));
    }

    #[test]
    fn unknown_authority_for_one_hidden_state_is_unresolved() {
        let model = problem(
            1,
            vec![
                hidden("a", 0, &[("service", GateResult::Pass)]),
                hidden("b", 0, &[("service", GateResult::Pass)]),
                hidden("ta", 1, &[("service", GateResult::Pass)]),
                hidden("tb", 1, &[("service", GateResult::Pass)]),
            ],
            vec![
                belief("initial", 0, "same", &["a", "b"]),
                belief("terminal", 1, "terminal", &["ta", "tb"]),
            ],
            vec![action(
                "act",
                "initial",
                &[
                    ("a", &[("authority", GateResult::Pass)]),
                    ("b", &[("authority", GateResult::Unknown)]),
                ],
            )],
            vec![
                transition("a", "act", "none", "ta", "terminal"),
                transition("b", "act", "none", "tb", "terminal"),
            ],
            &["initial"],
            &["none"],
        );

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.unresolved_initial_beliefs.contains("initial"));
        assert!(!report.viable_initial_beliefs.contains("initial"));
    }

    #[test]
    fn one_hidden_ecological_floor_failure_invalidates_whole_belief() {
        let mut model = problem(
            1,
            vec![
                hidden("a", 0, &[("service", GateResult::Pass), ("ecology", GateResult::Pass)]),
                hidden("b", 0, &[("service", GateResult::Pass), ("ecology", GateResult::Fail)]),
            ],
            vec![belief("initial", 0, "same", &["a", "b"])],
            Vec::new(),
            Vec::new(),
            &["initial"],
            &["none"],
        );
        model.required_floors.push("ecology".to_owned());

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.infeasible_initial_beliefs.contains("initial"));
    }

    #[test]
    fn missing_observation_mapping_is_unresolved_not_a_free_branch() {
        let mut model = problem(
            1,
            vec![
                hidden("a", 0, &[("service", GateResult::Pass)]),
                hidden("terminal", 1, &[("service", GateResult::Pass)]),
            ],
            vec![
                belief("initial", 0, "history_0", &["a"]),
                belief("terminal", 1, "history_1", &["terminal"]),
            ],
            vec![action(
                "act",
                "initial",
                &[("a", &[("authority", GateResult::Pass)])],
            )],
            vec![transition("a", "act", "none", "terminal", "")],
            &["initial"],
            &["none"],
        );

        let report = compute_belief_viability_kernel(&model).unwrap();
        assert!(report.unresolved_initial_beliefs.contains("initial"));
    }

    #[test]
    fn adding_hidden_state_cannot_make_an_unviable_action_viable() {
        let one = problem(
            1,
            vec![
                hidden("a", 0, &[("service", GateResult::Pass)]),
                hidden("ta", 1, &[("service", GateResult::Pass)]),
            ],
            vec![
                belief("initial", 0, "history", &["a"]),
                belief("terminal", 1, "terminal", &["ta"]),
            ],
            vec![action(
                "act",
                "initial",
                &[("a", &[("authority", GateResult::Pass)])],
            )],
            vec![transition("a", "act", "none", "ta", "terminal")],
            &["initial"],
            &["none"],
        );
        let one_report = compute_belief_viability_kernel(&one).unwrap();
        assert!(one_report.viable_initial_beliefs.contains("initial"));

        let mut expanded = one;
        expanded.hidden_states.push(hidden(
            "b",
            0,
            &[("service", GateResult::Pass)],
        ));
        expanded.hidden_states.push(hidden(
            "tb",
            1,
            &[("service", GateResult::Fail)],
        ));
        expanded
            .beliefs
            .iter_mut()
            .find(|belief| belief.belief_id == "initial")
            .unwrap()
            .hidden_state_ids
            .insert("b".to_owned());
        expanded
            .beliefs
            .iter_mut()
            .find(|belief| belief.belief_id == "terminal")
            .unwrap()
            .hidden_state_ids
            .insert("tb".to_owned());
        expanded.actions[0]
            .gates_by_hidden_state
            .insert("b".to_owned(), map(&[("authority", GateResult::Pass)]));
        expanded.transitions.push(transition(
            "b",
            "act",
            "none",
            "tb",
            "terminal",
        ));

        let expanded_report = compute_belief_viability_kernel(&expanded).unwrap();
        assert!(expanded_report.infeasible_initial_beliefs.contains("initial"));
        assert!(!expanded_report.viable_initial_beliefs.contains("initial"));
    }


    #[test]
    fn exhaustive_two_hidden_two_action_two_disturbance_oracle_matches_all_256_games() {
        for encoded in 0_u16..256 {
            let disturbances = ["d0", "d1"];
            let mut hidden_states = vec![
                hidden("world_a", 0, &[("service", GateResult::Pass)]),
                hidden("world_b", 0, &[("service", GateResult::Pass)]),
            ];
            let mut beliefs = vec![belief(
                "initial",
                0,
                "same-observation",
                &["world_a", "world_b"],
            )];
            let mut actions = Vec::new();
            let mut transitions = Vec::new();
            let mut oracle_action_exists = false;

            for action_index in 0..2 {
                let action_id = if action_index == 0 { "action_a" } else { "action_b" };
                let output_history = if action_index == 0 {
                    "after_action_a"
                } else {
                    "after_action_b"
                };
                let mut terminal_ids = Vec::new();
                let mut all_branches_pass = true;

                for (world_index, world_id) in ["world_a", "world_b"].iter().enumerate() {
                    for (disturbance_index, disturbance) in disturbances.iter().enumerate() {
                        let bit_index =
                            action_index * 4 + world_index * 2 + disturbance_index;
                        let fails = ((encoded >> bit_index) & 1) != 0;
                        let terminal_id = format!(
                            "terminal_{action_index}_{world_index}_{disturbance_index}"
                        );
                        let floor = if fails {
                            all_branches_pass = false;
                            GateResult::Fail
                        } else {
                            GateResult::Pass
                        };

                        hidden_states.push(hidden(
                            &terminal_id,
                            1,
                            &[("service", floor)],
                        ));
                        terminal_ids.push(terminal_id.clone());
                        transitions.push(transition(
                            world_id,
                            action_id,
                            disturbance,
                            &terminal_id,
                            output_history,
                        ));
                    }
                }

                if all_branches_pass {
                    oracle_action_exists = true;
                }

                let member_refs = terminal_ids
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                beliefs.push(belief(
                    output_history,
                    1,
                    output_history,
                    &member_refs,
                ));
                actions.push(action(
                    action_id,
                    "initial",
                    &[
                        ("world_a", &[("authority", GateResult::Pass)]),
                        ("world_b", &[("authority", GateResult::Pass)]),
                    ],
                ));
            }

            let model = problem(
                1,
                hidden_states,
                beliefs,
                actions,
                transitions,
                &["initial"],
                &disturbances,
            );
            let report = compute_belief_viability_kernel(&model).unwrap();
            let actual = report.viable_initial_beliefs.contains("initial");

            assert_eq!(
                actual,
                oracle_action_exists,
                "belief-game mismatch for encoded utility/outcome table {encoded}"
            );
        }
    }

    #[test]
    fn belief_limits_return_bounded_without_partial_policy() {
        let model = problem(
            1,
            vec![hidden("a", 0, &[("service", GateResult::Pass)])],
            vec![belief("initial", 0, "history", &["a"])],
            Vec::new(),
            Vec::new(),
            &["initial"],
            &["none"],
        );
        let limits = BeliefViabilityLimits {
            max_hidden_states: 0,
            max_beliefs: 10,
            max_actions: 10,
            max_transition_rows: 10,
            max_belief_expansion_branches: 10,
        };

        assert_eq!(
            compute_belief_viability_kernel_with_limits(&model, limits).unwrap(),
            BeliefViabilityExecution::Bounded {
                resource: "hidden_states".to_owned(),
                observed: 1,
                limit: 0,
            }
        );
    }
}
