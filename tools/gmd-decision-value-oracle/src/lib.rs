//! Deterministic, dependency-free decision-value oracle for GMD response studies.
//!
//! The oracle compares current-evidence decisions with a delayed observation
//! policy over a finite, explicitly declared scenario-state set. It does not
//! forecast physical states, produce physical admissibility itself, grant
//! authority, dispatch equipment, or execute an intervention.
//!
//! All probabilities use integer parts per million. All loss values and
//! decision weights are owner-supplied benchmark inputs. Cross-service
//! scalarization is therefore explicit and must not replace per-service
//! reporting or owner-defined hard-floor constraints.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

pub const PROBABILITY_SCALE: u64 = 1_000_000;
const EXPECTED_VALUE_DENOMINATOR: u128 = 1_000_000_000_000;
const MAX_STATES: usize = 64;
const MAX_SERVICES: usize = 16;
const MAX_ACTIONS: usize = 64;
const MAX_OBSERVATION_OUTCOMES: usize = 32;

/// A finite state in the frozen benchmark scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioState {
    pub state_id: String,
    pub prior_weight_ppm: u32,
}

/// One independently reported service-loss dimension.
///
/// decision_weight is an explicit, owner-supplied conversion weight into the
/// benchmark's common decision-loss units. It must never be silently inferred.
/// The service's native loss unit is retained in every output metric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDimension {
    pub service_id: String,
    pub loss_unit: String,
    pub decision_weight: u64,
    /// Hard floors block interventions that would breach the floor in any
    /// state that remains possible. A HOLD decision is still reportable when
    /// the current state already violates a floor; its loss remains visible.
    pub hard_floor: bool,
}

/// Whether an action is an intervention or an explicit no-intervention hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Intervention,
    Hold,
}

/// Independent readiness gates supplied by qualified upstream evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionGates {
    pub human_approved: bool,
    pub resources_available: bool,
    pub evidence_fresh: bool,
    pub scope_verified: bool,
}

/// One candidate action. Physical admissibility and floor preservation are
/// supplied per scenario state and are not inferred by this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateAction {
    pub action_id: String,
    pub kind: ActionKind,
    pub owner_ref: String,
    pub procedure_ref: String,
    pub procedure_sha256: String,
    pub physical_evidence_ref: String,
    pub physical_evidence_sha256: String,
    /// Conservative maximum completion duration after the decision can start.
    pub maximum_completion_seconds: u64,
    pub safety_buffer_seconds: u64,
    pub gates: ActionGates,
    /// One Boolean for each scenario state.
    pub physically_admissible_by_state: Vec<bool>,
    /// State-major matrix: [state][service].
    pub service_floor_preserved_by_state: Vec<Vec<bool>>,
    /// State-major matrix: [state][service], in each service's declared unit.
    pub service_loss_by_state: Vec<Vec<u64>>,
}

/// A possible discrete result of an observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationOutcome {
    pub outcome_id: String,
    /// P(outcome | state), one integer likelihood per scenario state.
    pub likelihood_by_state_ppm: Vec<u32>,
}

/// One proposed measurement or data acquisition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationOption {
    pub observation_id: String,
    pub source_ref: String,
    pub source_sha256: String,
    pub rights_evidence_ref: String,
    pub likelihood_model_ref: String,
    pub likelihood_model_sha256: String,
    /// Request-specific rights gate. No right is inferred from another use.
    pub rights_eligible: bool,
    /// Calibration, clock, coordinates, sampling and quality are qualified.
    pub quality_qualified: bool,
    /// Joint likelihood model accounts for known common-mode/correlated errors.
    pub common_mode_model_qualified: bool,
    /// Event-relative elapsed seconds from decision time t0.
    pub arrival_seconds: u64,
    /// Owner-approved decision-loss units; zero is allowed for a free measure.
    pub cost_score_units: u64,
    pub outcomes: Vec<ObservationOutcome>,
}

/// Complete frozen decision problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionProblem {
    pub scenario_id: String,
    pub manifest_sha256: String,
    pub decision_deadline_seconds: u64,
    pub states: Vec<ScenarioState>,
    pub services: Vec<ServiceDimension>,
    pub actions: Vec<CandidateAction>,
    pub observation: ObservationOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationDisposition {
    Eligible,
    RightsIneligible,
    QualityUnqualified,
    CommonModeUnqualified,
    ArrivesAfterDeadline,
}

impl ObservationDisposition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "ELIGIBLE",
            Self::RightsIneligible => "RIGHTS_INELIGIBLE",
            Self::QualityUnqualified => "QUALITY_UNQUALIFIED",
            Self::CommonModeUnqualified => "COMMON_MODE_UNQUALIFIED",
            Self::ArrivesAfterDeadline => "ARRIVES_AFTER_DEADLINE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    ObserveThenUseOutcomePolicy,
    UseCurrentEvidence,
}

impl Recommendation {
    fn as_str(self) -> &'static str {
        match self {
            Self::ObserveThenUseOutcomePolicy => "OBSERVE_THEN_USE_OUTCOME_POLICY",
            Self::UseCurrentEvidence => "USE_CURRENT_EVIDENCE",
        }
    }
}

/// Expected loss is a rational value with an exact integer numerator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceLossMetric {
    pub service_id: String,
    pub loss_unit: String,
    pub numerator: u128,
    pub denominator: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeDecision {
    pub outcome_id: String,
    pub action_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionResult {
    pub scenario_id: String,
    pub manifest_sha256: String,
    pub decision_deadline_seconds: u64,
    pub observation_id: String,
    pub observation_arrival_seconds: u64,
    pub observation_disposition: ObservationDisposition,
    pub baseline_action_id: String,
    pub baseline_expected_service_losses: Vec<ServiceLossMetric>,
    pub observed_expected_service_losses: Option<Vec<ServiceLossMetric>>,
    pub outcome_decisions: Vec<OutcomeDecision>,
    pub baseline_weighted_loss_numerator: u128,
    pub observed_weighted_loss_numerator: Option<u128>,
    pub observation_cost_numerator: Option<u128>,
    /// Positive means observation improves the owner-weighted expected loss
    /// after measurement cost. Negative means waiting for it is harmful.
    pub net_value_numerator: Option<i128>,
    pub expected_value_denominator: u128,
    pub post_observation_has_intervention: bool,
    /// This evaluator never issues or authorizes an operational command.
    pub execution_authorized: bool,
    pub recommendation: Recommendation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionError {
    EmptyRequiredField,
    InvalidDigest,
    InvalidDeadline,
    EmptyStateSet,
    TooManyStates,
    TooManyServices,
    TooManyActions,
    TooManyObservationOutcomes,
    DuplicateIdentifier,
    InvalidPriorProbabilities,
    InvalidObservationLikelihoods,
    InvalidMatrixShape,
    InvalidDecisionWeight,
    MissingHoldAction,
    InvalidHoldAction,
    InvalidInterventionDuration,
    NoFeasibleAction,
    ArithmeticOverflow,
}

impl fmt::Display for DecisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyRequiredField => "required identifiers and evidence references must be non-empty and trimmed",
            Self::InvalidDigest => "SHA-256 values must be 64 lowercase hexadecimal characters",
            Self::InvalidDeadline => "decision deadline must be positive",
            Self::EmptyStateSet => "scenario must contain at least one state, service and action",
            Self::TooManyStates => "state count exceeds the deterministic oracle limit",
            Self::TooManyServices => "service count exceeds the deterministic oracle limit",
            Self::TooManyActions => "action count exceeds the deterministic oracle limit",
            Self::TooManyObservationOutcomes => "observation outcome count exceeds the deterministic oracle limit",
            Self::DuplicateIdentifier => "identifiers must be unique within their collection",
            Self::InvalidPriorProbabilities => "prior probabilities must be positive and sum exactly to one million ppm",
            Self::InvalidObservationLikelihoods => "each state's observation likelihoods must sum exactly to one million ppm",
            Self::InvalidMatrixShape => "state/service matrices do not match the declared dimensions",
            Self::InvalidDecisionWeight => "service decision weights must be positive",
            Self::MissingHoldAction => "at least one explicit HOLD action is required",
            Self::InvalidHoldAction => "HOLD must have zero duration, open gates and physical availability in all states",
            Self::InvalidInterventionDuration => "intervention maximum completion duration must be positive",
            Self::NoFeasibleAction => "no admissible action is available for the remaining state set and time window",
            Self::ArithmeticOverflow => "exact integer arithmetic overflowed",
        };
        f.write_str(message)
    }
}

impl Error for DecisionError {}

/// Evaluate ACT_NOW against one delayed observation policy.
///
/// An intervention is eligible only when all readiness gates pass, it can
/// complete conservatively before the deadline, it is physically admissible in
/// every state with non-zero posterior support, and it preserves every
/// declared hard service floor in those states. HOLD remains an explicit
/// outcome and is never described as successful recovery merely because it
/// was selected.
pub fn evaluate(problem: &DecisionProblem) -> Result<DecisionResult, DecisionError> {
    validate_problem(problem)?;

    let baseline_action_index = select_action(problem, None, 0)?;
    let baseline_service_numerators = service_loss_numerators_for_action(
        problem,
        baseline_action_index,
        None,
    )?;
    let baseline_service_numerators =
        lift_prior_numerators_to_joint_scale(baseline_service_numerators)?;
    let baseline_weighted_loss_numerator =
        weighted_loss_numerator(problem, &baseline_service_numerators)?;

    let observation_disposition = observation_disposition(problem);
    let mut observed_expected_service_losses = None;
    let mut observed_weighted_loss_numerator = None;
    let mut observation_cost_numerator = None;
    let mut net_value_numerator = None;
    let mut outcome_decisions = Vec::new();
    let mut post_observation_has_intervention = false;
    let mut recommendation = Recommendation::UseCurrentEvidence;

    if observation_disposition == ObservationDisposition::Eligible {
        let mut observed_service_numerators = vec![0_u128; problem.services.len()];

        for outcome in &problem.observation.outcomes {
            let marginal_mass = outcome
                .likelihood_by_state_ppm
                .iter()
                .zip(problem.states.iter())
                .try_fold(0_u128, |acc, (likelihood, state)| {
                    let joint = u128::from(state.prior_weight_ppm)
                        .checked_mul(u128::from(*likelihood))
                        .ok_or(DecisionError::ArithmeticOverflow)?;
                    acc.checked_add(joint).ok_or(DecisionError::ArithmeticOverflow)
                })?;

            if marginal_mass == 0 {
                continue;
            }

            let action_index = select_action(
                problem,
                Some(&outcome.likelihood_by_state_ppm),
                problem.observation.arrival_seconds,
            )?;
            let action = &problem.actions[action_index];
            if action.kind == ActionKind::Intervention {
                post_observation_has_intervention = true;
            }
            outcome_decisions.push(OutcomeDecision {
                outcome_id: outcome.outcome_id.clone(),
                action_id: action.action_id.clone(),
            });

            for (state_index, state) in problem.states.iter().enumerate() {
                let joint_weight = u128::from(state.prior_weight_ppm)
                    .checked_mul(u128::from(outcome.likelihood_by_state_ppm[state_index]))
                    .ok_or(DecisionError::ArithmeticOverflow)?;
                for (service_index, total) in observed_service_numerators.iter_mut().enumerate() {
                    let term = joint_weight
                        .checked_mul(
                            u128::from(action.service_loss_by_state[state_index][service_index]),
                        )
                        .ok_or(DecisionError::ArithmeticOverflow)?;
                    *total = total
                        .checked_add(term)
                        .ok_or(DecisionError::ArithmeticOverflow)?;
                }
            }
        }

        if outcome_decisions.is_empty() {
            return Err(DecisionError::InvalidObservationLikelihoods);
        }

        let observed_weighted = weighted_loss_numerator(problem, &observed_service_numerators)?;
        let cost_numerator = u128::from(problem.observation.cost_score_units)
            .checked_mul(EXPECTED_VALUE_DENOMINATOR)
            .ok_or(DecisionError::ArithmeticOverflow)?;
        let net_numerator = signed_difference(
            baseline_weighted_loss_numerator,
            observed_weighted
                .checked_add(cost_numerator)
                .ok_or(DecisionError::ArithmeticOverflow)?,
        )?;

        if net_numerator > 0 {
            recommendation = Recommendation::ObserveThenUseOutcomePolicy;
        }

        observed_expected_service_losses = Some(service_metrics(problem, observed_service_numerators));
        observed_weighted_loss_numerator = Some(observed_weighted);
        observation_cost_numerator = Some(cost_numerator);
        net_value_numerator = Some(net_numerator);
    }

    Ok(DecisionResult {
        scenario_id: problem.scenario_id.clone(),
        manifest_sha256: problem.manifest_sha256.clone(),
        decision_deadline_seconds: problem.decision_deadline_seconds,
        observation_id: problem.observation.observation_id.clone(),
        observation_arrival_seconds: problem.observation.arrival_seconds,
        observation_disposition,
        baseline_action_id: problem.actions[baseline_action_index].action_id.clone(),
        baseline_expected_service_losses: service_metrics(problem, baseline_service_numerators),
        observed_expected_service_losses,
        outcome_decisions,
        baseline_weighted_loss_numerator,
        observed_weighted_loss_numerator,
        observation_cost_numerator,
        net_value_numerator,
        expected_value_denominator: EXPECTED_VALUE_DENOMINATOR,
        post_observation_has_intervention,
        execution_authorized: false,
        recommendation,
    })
}

fn validate_problem(problem: &DecisionProblem) -> Result<(), DecisionError> {
    validate_required(&problem.scenario_id)?;
    if !valid_sha256(&problem.manifest_sha256) {
        return Err(DecisionError::InvalidDigest);
    }
    if problem.decision_deadline_seconds == 0 {
        return Err(DecisionError::InvalidDeadline);
    }
    if problem.states.is_empty() || problem.services.is_empty() || problem.actions.is_empty() {
        return Err(DecisionError::EmptyStateSet);
    }
    if problem.states.len() > MAX_STATES {
        return Err(DecisionError::TooManyStates);
    }
    if problem.services.len() > MAX_SERVICES {
        return Err(DecisionError::TooManyServices);
    }
    if problem.actions.len() > MAX_ACTIONS {
        return Err(DecisionError::TooManyActions);
    }
    if problem.observation.outcomes.is_empty()
        || problem.observation.outcomes.len() > MAX_OBSERVATION_OUTCOMES
    {
        return Err(DecisionError::TooManyObservationOutcomes);
    }

    let mut state_ids = BTreeSet::new();
    let mut prior_sum = 0_u64;
    for state in &problem.states {
        validate_required(&state.state_id)?;
        if !state_ids.insert(state.state_id.as_str()) {
            return Err(DecisionError::DuplicateIdentifier);
        }
        if state.prior_weight_ppm == 0 || u64::from(state.prior_weight_ppm) > PROBABILITY_SCALE {
            return Err(DecisionError::InvalidPriorProbabilities);
        }
        prior_sum = prior_sum
            .checked_add(u64::from(state.prior_weight_ppm))
            .ok_or(DecisionError::ArithmeticOverflow)?;
    }
    if prior_sum != PROBABILITY_SCALE {
        return Err(DecisionError::InvalidPriorProbabilities);
    }

    let mut service_ids = BTreeSet::new();
    for service in &problem.services {
        validate_required(&service.service_id)?;
        validate_required(&service.loss_unit)?;
        if !service_ids.insert(service.service_id.as_str()) {
            return Err(DecisionError::DuplicateIdentifier);
        }
        if service.decision_weight == 0 {
            return Err(DecisionError::InvalidDecisionWeight);
        }
    }

    let mut action_ids = BTreeSet::new();
    let mut has_hold = false;
    for action in &problem.actions {
        for value in [
            &action.action_id,
            &action.owner_ref,
            &action.procedure_ref,
            &action.physical_evidence_ref,
        ] {
            validate_required(value)?;
        }
        if !action_ids.insert(action.action_id.as_str()) {
            return Err(DecisionError::DuplicateIdentifier);
        }
        if !valid_sha256(&action.procedure_sha256)
            || !valid_sha256(&action.physical_evidence_sha256)
        {
            return Err(DecisionError::InvalidDigest);
        }
        if action.physically_admissible_by_state.len() != problem.states.len()
            || action.service_floor_preserved_by_state.len() != problem.states.len()
            || action.service_loss_by_state.len() != problem.states.len()
            || action
                .service_floor_preserved_by_state
                .iter()
                .any(|row| row.len() != problem.services.len())
            || action
                .service_loss_by_state
                .iter()
                .any(|row| row.len() != problem.services.len())
        {
            return Err(DecisionError::InvalidMatrixShape);
        }
        match action.kind {
            ActionKind::Hold => {
                has_hold = true;
                if action.maximum_completion_seconds != 0 || action.safety_buffer_seconds != 0 {
                    return Err(DecisionError::InvalidHoldAction);
                }
            }
            ActionKind::Intervention => {
                if action.maximum_completion_seconds == 0 {
                    return Err(DecisionError::InvalidInterventionDuration);
                }
            }
        }
    }
    if !has_hold {
        return Err(DecisionError::MissingHoldAction);
    }

    let observation = &problem.observation;
    for value in [
        &observation.observation_id,
        &observation.source_ref,
        &observation.rights_evidence_ref,
        &observation.likelihood_model_ref,
    ] {
        validate_required(value)?;
    }
    if !valid_sha256(&observation.source_sha256)
        || !valid_sha256(&observation.likelihood_model_sha256)
    {
        return Err(DecisionError::InvalidDigest);
    }

    let mut outcome_ids = BTreeSet::new();
    let mut state_likelihood_sums = vec![0_u64; problem.states.len()];
    for outcome in &observation.outcomes {
        validate_required(&outcome.outcome_id)?;
        if !outcome_ids.insert(outcome.outcome_id.as_str()) {
            return Err(DecisionError::DuplicateIdentifier);
        }
        if outcome.likelihood_by_state_ppm.len() != problem.states.len() {
            return Err(DecisionError::InvalidMatrixShape);
        }
        for (index, likelihood) in outcome.likelihood_by_state_ppm.iter().enumerate() {
            if u64::from(*likelihood) > PROBABILITY_SCALE {
                return Err(DecisionError::InvalidObservationLikelihoods);
            }
            state_likelihood_sums[index] = state_likelihood_sums[index]
                .checked_add(u64::from(*likelihood))
                .ok_or(DecisionError::ArithmeticOverflow)?;
        }
    }
    if state_likelihood_sums
        .iter()
        .any(|sum| *sum != PROBABILITY_SCALE)
    {
        return Err(DecisionError::InvalidObservationLikelihoods);
    }

    Ok(())
}

fn observation_disposition(problem: &DecisionProblem) -> ObservationDisposition {
    let observation = &problem.observation;
    if !observation.rights_eligible {
        ObservationDisposition::RightsIneligible
    } else if !observation.common_mode_model_qualified {
        ObservationDisposition::CommonModeUnqualified
    } else if !observation.quality_qualified {
        ObservationDisposition::QualityUnqualified
    } else if observation.arrival_seconds >= problem.decision_deadline_seconds {
        ObservationDisposition::ArrivesAfterDeadline
    } else {
        ObservationDisposition::Eligible
    }
}

fn validate_required(value: &str) -> Result<(), DecisionError> {
    if value.trim().is_empty() || value.trim() != value {
        Err(DecisionError::EmptyRequiredField)
    } else {
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn gates_pass(gates: ActionGates) -> bool {
    gates.human_approved && gates.resources_available && gates.evidence_fresh && gates.scope_verified
}

/// Choose an action using the supplied (possibly unnormalized) joint weights.
/// An action must be safe for every state in the current support, not merely
/// the most likely state. Ties resolve lexicographically by action identifier.
fn select_action(
    problem: &DecisionProblem,
    likelihoods: Option<&[u32]>,
    arrival_seconds: u64,
) -> Result<usize, DecisionError> {
    let mut best: Option<(usize, u128)> = None;

    for (action_index, action) in problem.actions.iter().enumerate() {
        if action.kind == ActionKind::Intervention && !gates_pass(action.gates) {
            continue;
        }
        let completes_by_deadline = arrival_seconds
            .checked_add(action.maximum_completion_seconds)
            .and_then(|value| value.checked_add(action.safety_buffer_seconds))
            .is_some_and(|completion| completion <= problem.decision_deadline_seconds);
        if !completes_by_deadline {
            continue;
        }

        let mut has_support = false;
        let mut admissible = true;
        let mut score = 0_u128;

        for (state_index, state) in problem.states.iter().enumerate() {
            let likelihood = likelihoods.map_or(u128::from(PROBABILITY_SCALE), |values| {
                u128::from(values[state_index])
            });
            let joint_weight = u128::from(state.prior_weight_ppm)
                .checked_mul(likelihood)
                .ok_or(DecisionError::ArithmeticOverflow)?;
            if joint_weight == 0 {
                continue;
            }
            has_support = true;
            if action.kind == ActionKind::Intervention {
                if !action.physically_admissible_by_state[state_index] {
                    admissible = false;
                    break;
                }
                for (service_index, service) in problem.services.iter().enumerate() {
                    if service.hard_floor
                        && !action.service_floor_preserved_by_state[state_index][service_index]
                    {
                        admissible = false;
                        break;
                    }
                }
            }
            if !admissible {
                break;
            }
            for (service_index, service) in problem.services.iter().enumerate() {
                let loss_units =
                    u128::from(action.service_loss_by_state[state_index][service_index]);
                let weighted_loss = loss_units
                    .checked_mul(u128::from(service.decision_weight))
                    .ok_or(DecisionError::ArithmeticOverflow)?;
                let term = joint_weight
                    .checked_mul(weighted_loss)
                    .ok_or(DecisionError::ArithmeticOverflow)?;
                score = score
                    .checked_add(term)
                    .ok_or(DecisionError::ArithmeticOverflow)?;
            }
        }

        if !admissible || !has_support {
            continue;
        }
        let replace = best.as_ref().is_none_or(|(best_index, best_score)| {
            score < *best_score
                || (score == *best_score
                    && action.action_id < problem.actions[*best_index].action_id)
        });
        if replace {
            best = Some((action_index, score));
        }
    }

    best.map(|(index, _)| index)
        .ok_or(DecisionError::NoFeasibleAction)
}

fn service_loss_numerators_for_action(
    problem: &DecisionProblem,
    action_index: usize,
    likelihoods: Option<&[u32]>,
) -> Result<Vec<u128>, DecisionError> {
    let action = &problem.actions[action_index];
    let mut totals = vec![0_u128; problem.services.len()];
    for (state_index, state) in problem.states.iter().enumerate() {
        let likelihood = likelihoods.map_or(u128::from(PROBABILITY_SCALE), |values| {
            u128::from(values[state_index])
        });
        let joint_weight = u128::from(state.prior_weight_ppm)
            .checked_mul(likelihood)
            .ok_or(DecisionError::ArithmeticOverflow)?;
        for (service_index, total) in totals.iter_mut().enumerate() {
            let term = joint_weight
                .checked_mul(u128::from(action.service_loss_by_state[state_index][service_index]))
                .ok_or(DecisionError::ArithmeticOverflow)?;
            *total = total
                .checked_add(term)
                .ok_or(DecisionError::ArithmeticOverflow)?;
        }
    }
    Ok(totals)
}

fn weighted_loss_numerator(
    problem: &DecisionProblem,
    service_numerators: &[u128],
) -> Result<u128, DecisionError> {
    let mut total = 0_u128;
    for (service, numerator) in problem.services.iter().zip(service_numerators.iter()) {
        total = total
            .checked_add(
                numerator
                    .checked_mul(u128::from(service.decision_weight))
                    .ok_or(DecisionError::ArithmeticOverflow)?,
            )
            .ok_or(DecisionError::ArithmeticOverflow)?;
    }
    Ok(total)
}

/// Lift prior-weighted numerators (denominator 1e6) to the common
/// prior-times-likelihood denominator (1e12) used by every receipt metric.
fn lift_prior_numerators_to_joint_scale(
    numerators: Vec<u128>,
) -> Result<Vec<u128>, DecisionError> {
    numerators
        .into_iter()
        .map(|numerator| {
            numerator
                .checked_mul(u128::from(PROBABILITY_SCALE))
                .ok_or(DecisionError::ArithmeticOverflow)
        })
        .collect()
}

fn service_metrics(
    problem: &DecisionProblem,
    numerators: Vec<u128>,
) -> Vec<ServiceLossMetric> {
    problem
        .services
        .iter()
        .zip(numerators)
        .map(|(service, numerator)| ServiceLossMetric {
            service_id: service.service_id.clone(),
            loss_unit: service.loss_unit.clone(),
            numerator,
            denominator: EXPECTED_VALUE_DENOMINATOR,
        })
        .collect()
}

fn signed_difference(left: u128, right: u128) -> Result<i128, DecisionError> {
    if left >= right {
        i128::try_from(left - right).map_err(|_| DecisionError::ArithmeticOverflow)
    } else {
        i128::try_from(right - left)
            .map(|magnitude| -magnitude)
            .map_err(|_| DecisionError::ArithmeticOverflow)
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", u32::from(value));
            }
            value => escaped.push(value),
        }
    }
    escaped
}

fn service_metrics_json(metrics: &[ServiceLossMetric]) -> String {
    let entries: Vec<String> = metrics
        .iter()
        .map(|metric| {
            format!(
                "{{\"service_id\":\"{}\",\"loss_unit\":\"{}\",\"numerator\":\"{}\",\"denominator\":\"{}\"}}",
                json_escape(&metric.service_id),
                json_escape(&metric.loss_unit),
                metric.numerator,
                metric.denominator
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

impl DecisionResult {
    /// Stable JSON receipt, with exact large integers encoded as strings.
    pub fn to_json(&self) -> String {
        let observed_services = self
            .observed_expected_service_losses
            .as_ref()
            .map(|metrics| service_metrics_json(metrics))
            .unwrap_or_else(|| "null".to_owned());
        let outcome_entries: Vec<String> = self
            .outcome_decisions
            .iter()
            .map(|decision| {
                format!(
                    "{{\"outcome_id\":\"{}\",\"action_id\":\"{}\"}}",
                    json_escape(&decision.outcome_id),
                    json_escape(&decision.action_id)
                )
            })
            .collect();
        let outcomes_json = format!("[{}]", outcome_entries.join(","));
        let observed_weighted = self
            .observed_weighted_loss_numerator
            .map(|value| format!("\"{}\"", value))
            .unwrap_or_else(|| "null".to_owned());
        let cost = self
            .observation_cost_numerator
            .map(|value| format!("\"{}\"", value))
            .unwrap_or_else(|| "null".to_owned());
        let net = self
            .net_value_numerator
            .map(|value| format!("\"{}\"", value))
            .unwrap_or_else(|| "null".to_owned());

        format!(
            "{{\"schema_version\":\"0.1.0\",\"scenario_id\":\"{}\",\"manifest_sha256\":\"{}\",\"decision_deadline_seconds\":{},\"observation_id\":\"{}\",\"observation_arrival_seconds\":{},\"observation_disposition\":\"{}\",\"baseline_action_id\":\"{}\",\"baseline_expected_service_losses\":{},\"observed_expected_service_losses\":{},\"outcome_decisions\":{},\"baseline_weighted_loss_numerator\":\"{}\",\"observed_weighted_loss_numerator\":{},\"observation_cost_numerator\":{},\"net_value_numerator\":{},\"expected_value_denominator\":\"{}\",\"post_observation_has_intervention\":{},\"execution_authorized\":{},\"recommendation\":\"{}\"}}",
            json_escape(&self.scenario_id),
            self.manifest_sha256,
            self.decision_deadline_seconds,
            json_escape(&self.observation_id),
            self.observation_arrival_seconds,
            self.observation_disposition.as_str(),
            json_escape(&self.baseline_action_id),
            service_metrics_json(&self.baseline_expected_service_losses),
            observed_services,
            outcomes_json,
            self.baseline_weighted_loss_numerator,
            observed_weighted,
            cost,
            net,
            self.expected_value_denominator,
            self.post_observation_has_intervention,
            self.execution_authorized,
            self.recommendation.as_str()
        )
    }
}

/// Positive-value known-answer fixture shared by the CLI and unit tests.
pub fn positive_voi_fixture() -> DecisionProblem {
    let all_gates = ActionGates {
        human_approved: true,
        resources_available: true,
        evidence_fresh: true,
        scope_verified: true,
    };

    DecisionProblem {
        scenario_id: "synthetic-positive-voi-v0.1".to_owned(),
        manifest_sha256: "a".repeat(64),
        decision_deadline_seconds: 100,
        states: vec![
            ScenarioState {
                state_id: "moderate-impact".to_owned(),
                prior_weight_ppm: 500_000,
            },
            ScenarioState {
                state_id: "severe-impact".to_owned(),
                prior_weight_ppm: 500_000,
            },
        ],
        services: vec![ServiceDimension {
            service_id: "regional-power".to_owned(),
            loss_unit: "service-deficit-minutes".to_owned(),
            decision_weight: 1,
            hard_floor: true,
        }],
        actions: vec![
            CandidateAction {
                action_id: "hold".to_owned(),
                kind: ActionKind::Hold,
                owner_ref: "owner:synthetic-operator".to_owned(),
                procedure_ref: "procedure:maintain-current-state".to_owned(),
                procedure_sha256: "b".repeat(64),
                physical_evidence_ref: "evidence:fixture-current-state".to_owned(),
                physical_evidence_sha256: "c".repeat(64),
                maximum_completion_seconds: 0,
                safety_buffer_seconds: 0,
                gates: all_gates,
                physically_admissible_by_state: vec![true, true],
                service_floor_preserved_by_state: vec![vec![false], vec![false]],
                service_loss_by_state: vec![vec![40], vec![40]],
            },
            CandidateAction {
                action_id: "mitigate-low-state".to_owned(),
                kind: ActionKind::Intervention,
                owner_ref: "owner:synthetic-operator".to_owned(),
                procedure_ref: "procedure:synthetic-mitigation".to_owned(),
                procedure_sha256: "d".repeat(64),
                physical_evidence_ref: "evidence:qualified-synthetic-physics".to_owned(),
                physical_evidence_sha256: "e".repeat(64),
                maximum_completion_seconds: 10,
                safety_buffer_seconds: 5,
                gates: all_gates,
                physically_admissible_by_state: vec![true, false],
                service_floor_preserved_by_state: vec![vec![true], vec![false]],
                service_loss_by_state: vec![vec![0], vec![100]],
            },
        ],
        observation: ObservationOption {
            observation_id: "targeted-gic-observation".to_owned(),
            source_ref: "source:synthetic-observation".to_owned(),
            source_sha256: "f".repeat(64),
            rights_evidence_ref: "rights:explicit-commercial-use".to_owned(),
            likelihood_model_ref: "model:qualified-joint-likelihood".to_owned(),
            likelihood_model_sha256: "1".repeat(64),
            rights_eligible: true,
            quality_qualified: true,
            common_mode_model_qualified: true,
            arrival_seconds: 5,
            cost_score_units: 2,
            outcomes: vec![
                ObservationOutcome {
                    outcome_id: "moderate-signal".to_owned(),
                    likelihood_by_state_ppm: vec![1_000_000, 0],
                },
                ObservationOutcome {
                    outcome_id: "severe-signal".to_owned(),
                    likelihood_by_state_ppm: vec![0, 1_000_000],
                },
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_value_observation_selects_state_conditioned_action() {
        let result = evaluate(&positive_voi_fixture()).expect("valid fixture");
        assert_eq!(result.baseline_action_id, "hold");
        assert_eq!(
            result.observation_disposition,
            ObservationDisposition::Eligible
        );
        assert_eq!(
            result.recommendation,
            Recommendation::ObserveThenUseOutcomePolicy
        );
        assert_eq!(result.baseline_weighted_loss_numerator, 40_000_000_000_000);
        assert_eq!(
            result.baseline_expected_service_losses[0].numerator,
            40_000_000_000_000
        );
        assert_eq!(
            result.baseline_expected_service_losses[0].denominator,
            1_000_000_000_000
        );
        let observed_service_losses = result
            .observed_expected_service_losses
            .as_ref()
            .expect("qualified observation has service metrics");
        assert_eq!(observed_service_losses[0].numerator, 20_000_000_000_000);
        assert_eq!(
            result.observed_weighted_loss_numerator,
            Some(20_000_000_000_000)
        );
        assert_eq!(
            result.observation_cost_numerator,
            Some(2_000_000_000_000)
        );
        assert_eq!(result.net_value_numerator, Some(18_000_000_000_000));
        assert_eq!(result.outcome_decisions.len(), 2);
        assert_eq!(result.outcome_decisions[0].action_id, "mitigate-low-state");
        assert_eq!(result.outcome_decisions[1].action_id, "hold");
        assert!(!result.execution_authorized);
    }

    #[test]
    fn uninformative_observation_has_zero_value_when_free() {
        let mut problem = positive_voi_fixture();
        problem.observation.cost_score_units = 0;
        problem.observation.outcomes = vec![
            ObservationOutcome {
                outcome_id: "signal-a".to_owned(),
                likelihood_by_state_ppm: vec![500_000, 500_000],
            },
            ObservationOutcome {
                outcome_id: "signal-b".to_owned(),
                likelihood_by_state_ppm: vec![500_000, 500_000],
            },
        ];
        let result = evaluate(&problem).expect("valid fixture");
        assert_eq!(result.net_value_numerator, Some(0));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }

    #[test]
    fn waiting_for_a_late_observation_can_be_strictly_worse() {
        let mut problem = positive_voi_fixture();
        let all_gates = ActionGates {
            human_approved: true,
            resources_available: true,
            evidence_fresh: true,
            scope_verified: true,
        };
        problem.actions[1] = CandidateAction {
            action_id: "act-now".to_owned(),
            kind: ActionKind::Intervention,
            owner_ref: "owner:synthetic-operator".to_owned(),
            procedure_ref: "procedure:act-now".to_owned(),
            procedure_sha256: "2".repeat(64),
            physical_evidence_ref: "evidence:current-physical-envelope".to_owned(),
            physical_evidence_sha256: "3".repeat(64),
            maximum_completion_seconds: 70,
            safety_buffer_seconds: 10,
            gates: all_gates,
            physically_admissible_by_state: vec![true, true],
            service_floor_preserved_by_state: vec![vec![true], vec![true]],
            service_loss_by_state: vec![vec![20], vec![20]],
        };
        problem.observation.arrival_seconds = 30;
        problem.observation.cost_score_units = 0;

        let result = evaluate(&problem).expect("valid delayed fixture");
        assert_eq!(result.baseline_action_id, "act-now");
        assert_eq!(result.outcome_decisions[0].action_id, "hold");
        assert_eq!(result.outcome_decisions[1].action_id, "hold");
        assert_eq!(result.net_value_numerator, Some(-20_000_000_000_000));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
        assert!(!result.post_observation_has_intervention);
    }

    #[test]
    fn rights_ineligible_observation_is_not_used() {
        let mut problem = positive_voi_fixture();
        problem.observation.rights_eligible = false;
        let result = evaluate(&problem).expect("valid rights fixture");
        assert_eq!(
            result.observation_disposition,
            ObservationDisposition::RightsIneligible
        );
        assert!(result.observed_expected_service_losses.is_none());
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }

    #[test]
    fn unqualified_common_mode_model_is_rejected() {
        let mut problem = positive_voi_fixture();
        problem.observation.common_mode_model_qualified = false;
        let result = evaluate(&problem).expect("valid common-mode fixture");
        assert_eq!(
            result.observation_disposition,
            ObservationDisposition::CommonModeUnqualified
        );
        assert!(result.observed_weighted_loss_numerator.is_none());
        assert_eq!(result.baseline_action_id, "hold");
    }

    #[test]
    fn hold_fallback_remains_available_without_intervention_gates() {
        let mut problem = positive_voi_fixture();
        let closed_gates = ActionGates {
            human_approved: false,
            resources_available: false,
            evidence_fresh: false,
            scope_verified: false,
        };
        problem.actions[0].gates = closed_gates;
        problem.actions[1].gates = closed_gates;

        let result = evaluate(&problem).expect("non-intervention HOLD remains available");
        assert_eq!(result.baseline_action_id, "hold");
        assert!(result
            .outcome_decisions
            .iter()
            .all(|decision| decision.action_id == "hold"));
        assert!(!result.execution_authorized);
    }

    #[test]
    fn hold_fallback_does_not_depend_on_physical_admissibility_flags() {
        let mut problem = positive_voi_fixture();
        let closed_gates = ActionGates {
            human_approved: false,
            resources_available: false,
            evidence_fresh: false,
            scope_verified: false,
        };
        problem.actions[0].gates = closed_gates;
        problem.actions[0].physically_admissible_by_state = vec![false, false];
        problem.actions[1].gates = closed_gates;
        let result = evaluate(&problem).expect("HOLD is a non-mutation abstention option");
        assert_eq!(result.baseline_action_id, "hold");
        assert!(result
            .outcome_decisions
            .iter()
            .all(|decision| decision.action_id == "hold"));
        assert!(!result.execution_authorized);
    }

    #[test]
    fn pending_human_authority_blocks_intervention() {
        let mut problem = positive_voi_fixture();
        problem.actions[1].gates.human_approved = false;
        let result = evaluate(&problem).expect("HOLD remains available");
        assert_eq!(result.baseline_action_id, "hold");
        assert!(result.outcome_decisions.iter().all(|decision| {
            decision.action_id == "hold"
        }));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }

    #[test]
    fn unavailable_resources_block_intervention() {
        let mut problem = positive_voi_fixture();
        problem.actions[1].gates.resources_available = false;
        let result = evaluate(&problem).expect("HOLD remains available");
        assert!(result.outcome_decisions.iter().all(|decision| {
            decision.action_id == "hold"
        }));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }

    #[test]
    fn hard_service_floor_blocks_intervention_even_when_physics_is_admissible() {
        let mut problem = positive_voi_fixture();
        problem.actions[1].physically_admissible_by_state = vec![true, true];
        problem.actions[1].service_floor_preserved_by_state = vec![vec![false], vec![false]];
        let result = evaluate(&problem).expect("HOLD remains available");
        assert!(result.outcome_decisions.iter().all(|decision| {
            decision.action_id == "hold"
        }));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }
 
    #[test]
    fn stale_action_evidence_blocks_intervention() {
        let mut problem = positive_voi_fixture();
        problem.actions[1].gates.evidence_fresh = false;
        let result = evaluate(&problem).expect("HOLD remains available");
        assert!(result.outcome_decisions.iter().all(|decision| {
            decision.action_id == "hold"
        }));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }

    #[test]
    fn scope_conflict_blocks_intervention() {
        let mut problem = positive_voi_fixture();
        problem.actions[1].gates.scope_verified = false;
        let result = evaluate(&problem).expect("HOLD remains available");
        assert!(result.outcome_decisions.iter().all(|decision| {
            decision.action_id == "hold"
        }));
        assert_eq!(result.recommendation, Recommendation::UseCurrentEvidence);
    }

    #[test]
    fn observation_after_deadline_is_not_used() {
        let mut problem = positive_voi_fixture();
        problem.observation.arrival_seconds = problem.decision_deadline_seconds;
        let result = evaluate(&problem).expect("current-evidence decision remains available");
        assert_eq!(
            result.observation_disposition,
            ObservationDisposition::ArrivesAfterDeadline
        );
        assert!(result.outcome_decisions.is_empty());
        assert!(result.net_value_numerator.is_none());
        assert_eq!(result.baseline_action_id, "hold");
    }

    #[test]
    fn receipt_is_deterministic_and_never_authorizes_execution() {
        let problem = positive_voi_fixture();
        let first = evaluate(&problem).expect("first evaluation").to_json();
        let second = evaluate(&problem).expect("second evaluation").to_json();
        assert_eq!(first, second);
        assert!(first.contains("\"execution_authorized\":false"));
        assert!(first.contains("\"net_value_numerator\":\"18000000000000\""));
    }

    #[test]
    fn malformed_probability_mass_fails_closed() {
        let mut problem = positive_voi_fixture();
        problem.states[0].prior_weight_ppm = 400_000;
        assert_eq!(
            evaluate(&problem).expect_err("invalid priors"),
            DecisionError::InvalidPriorProbabilities
        );
    }
}
