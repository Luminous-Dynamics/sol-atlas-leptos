#![forbid(unsafe_code)]
//! Finite reference model for independent per-interval action admission.
//!
//! This crate does not optimize or actuate equipment. It checks one expiring
//! proposal against every declared current state and every declared one-step
//! future operating condition. Its proof is conditional on the supplied
//! finite state/uncertainty envelope and the equations below.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    pub const fn from_byte(value: u8) -> Self {
        Self([value; 32])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryContract {
    pub capacity_quanta: u16,
    pub charge_input_limit_quanta: u16,
    pub discharge_withdrawal_limit_quanta: u16,
    pub charge_efficiency_numerator: u16,
    pub charge_efficiency_denominator: u16,
    pub discharge_efficiency_numerator: u16,
    pub discharge_efficiency_denominator: u16,
}

impl BatteryContract {
    fn is_valid(self, reserve_quanta: u16) -> bool {
        self.capacity_quanta > 0
            && reserve_quanta <= self.capacity_quanta
            && self.charge_efficiency_numerator > 0
            && self.charge_efficiency_denominator > 0
            && self.charge_efficiency_numerator <= self.charge_efficiency_denominator
            && self.discharge_efficiency_numerator > 0
            && self.discharge_efficiency_denominator > 0
            && self.discharge_efficiency_numerator <= self.discharge_efficiency_denominator
    }

    fn charge_gain(self, charge_input_quanta: u16) -> u16 {
        (u64::from(charge_input_quanta)
            * u64::from(self.charge_efficiency_numerator)
            / u64::from(self.charge_efficiency_denominator)) as u16
    }

    fn discharge_delivered(self, withdrawal_quanta: u16) -> u16 {
        (u64::from(withdrawal_quanta)
            * u64::from(self.discharge_efficiency_numerator)
            / u64::from(self.discharge_efficiency_denominator)) as u16
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationStatus {
    Present,
    Missing,
    Conflicting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedState {
    pub state_id: String,
    pub soc_quanta: u16,
    pub grid_outage: bool,
    pub pv_available_quanta: u16,
    pub grid_import_limit_quanta: u16,
    pub essential_demand_quanta: u16,
    pub flexible_demand_quanta: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub status: ObservationStatus,
    pub observation_digest: Digest,
    pub source_digest: Digest,
    pub calibration_digest: Digest,
    pub epoch: u64,
    pub sequence: u64,
    pub sampled_at_unix_seconds: u64,
    pub received_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    /// A finite set of possible states admitted by the observation policy.
    /// Every state in this set must pass; a favourable representative is not
    /// sufficient.
    pub possible_states: Vec<ObservedState>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlAction {
    pub grid_import_quanta: u16,
    pub charge_input_quanta: u16,
    pub soc_withdrawal_quanta: u16,
    pub essential_served_quanta: u16,
    pub flexible_served_quanta: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionProposal {
    pub proposal_digest: Digest,
    pub request_digest: Digest,
    pub schedule_digest: Digest,
    pub action_digest: Digest,
    pub fallback_action_digest: Option<Digest>,
    pub interval_id: String,
    pub issued_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    /// The SOC envelope predicted by the previously admitted schedule prefix.
    pub expected_soc_min_quanta: u16,
    pub expected_soc_max_quanta: u16,
    pub proposed_action: ControlAction,
    pub fallback_action: Option<ControlAction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FutureOperatingCondition {
    pub scenario_id: String,
    pub grid_outage: bool,
    pub pv_available_quanta: u16,
    pub grid_import_limit_quanta: u16,
    pub essential_demand_quanta: u16,
    pub essential_service_floor_quanta: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionPolicy {
    /// Digests are verified against exact serialized bytes by the trusted caller.
    pub policy_digest: Digest,
    pub battery_model_digest: Digest,
    pub uncertainty_profile_digest: Digest,
    pub future_envelope_digest: Digest,
    pub expected_source_digest: Digest,
    pub expected_calibration_digest: Digest,
    pub expected_epoch: u64,
    /// Sequence number of the last observation already admitted by the caller.
    pub last_accepted_sequence: u64,
    pub max_observation_age_seconds: u64,
    pub current_essential_service_floor_quanta: u16,
    pub future_reserve_quanta: u16,
    /// Explicit one-step disturbance envelope. Each possible next condition
    /// must retain an essential-service continuation.
    pub future_conditions: Vec<FutureOperatingCondition>,
    pub monitor_digest: Digest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionDisposition {
    Admit,
    FallbackAdmitted,
    Replan,
    Abstain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionFailure {
    SocOutsideCapacity,
    StateOutsideProposalEnvelope,
    GridImportDuringOutage,
    GridImportLimitExceeded,
    ChargeLimitExceeded,
    DischargeLimitExceeded,
    SimultaneousChargeAndDischarge,
    SocUnderflow,
    SocOverflow,
    EssentialDemandOverServed,
    EssentialServiceFloorMissed,
    FlexibleDemandOverServed,
    NonPvSupplyExceedsUse,
    InsufficientPv,
    NoSafeContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionReason {
    ProposedActionAdmitted,
    FallbackActionAdmitted,
    ProposalExpired,
    InvalidProposalEnvelope,
    ObservationMissing,
    ObservationConflicting,
    SourceMismatch,
    CalibrationMismatch,
    EpochMismatch,
    SequenceReplayOrReorder,
    InvalidTimestampOrder,
    StaleObservation,
    ObservationExpired,
    EmptyStateSet,
    DuplicateStateId,
    InvalidBatteryContract,
    InvalidPolicy,
    EmptyFutureEnvelope,
    StateOutsideProposalEnvelope,
    ProposedActionRejected(ActionFailure),
    FallbackUnavailable,
    FallbackRejected {
        proposed: ActionFailure,
        fallback: ActionFailure,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionReport {
    pub disposition: AdmissionDisposition,
    pub reason: AdmissionReason,
    /// Decision timestamp and the earliest proposal/observation expiry.
    pub checked_at_unix_seconds: u64,
    pub valid_until_unix_seconds: u64,
    pub proposal_digest: Digest,
    pub request_digest: Digest,
    pub schedule_digest: Digest,
    /// Digest of the originally proposed action, even when a fallback is selected.
    pub action_digest: Digest,
    /// Digest of the actual action selected by the monitor.
    pub selected_action_digest: Option<Digest>,
    pub observation_digest: Digest,
    pub policy_digest: Digest,
    pub battery_model_digest: Digest,
    pub uncertainty_profile_digest: Digest,
    pub future_envelope_digest: Digest,
    pub monitor_digest: Digest,
    pub selected_action: Option<ControlAction>,
    pub current_states_checked: usize,
    pub future_conditions_checked: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Evaluation {
    failure: Option<ActionFailure>,
    current_states_checked: usize,
    future_conditions_checked: usize,
}

fn report(
    proposal: &ActionProposal,
    observation: &Observation,
    policy: &AdmissionPolicy,
    checked_at_unix_seconds: u64,
    disposition: AdmissionDisposition,
    reason: AdmissionReason,
    selected_action: Option<ControlAction>,
    selected_action_digest: Option<Digest>,
    evaluation: Evaluation,
) -> AdmissionReport {
    AdmissionReport {
        disposition,
        reason,
        proposal_digest: proposal.proposal_digest,
        request_digest: proposal.request_digest,
        schedule_digest: proposal.schedule_digest,
        action_digest: proposal.action_digest,
        selected_action_digest,
        observation_digest: observation.observation_digest,
        policy_digest: policy.policy_digest,
        battery_model_digest: policy.battery_model_digest,
        uncertainty_profile_digest: policy.uncertainty_profile_digest,
        future_envelope_digest: policy.future_envelope_digest,
        monitor_digest: policy.monitor_digest,
        checked_at_unix_seconds,
        valid_until_unix_seconds: proposal
            .expires_at_unix_seconds
            .min(observation.expires_at_unix_seconds),
        selected_action,
        current_states_checked: evaluation.current_states_checked,
        future_conditions_checked: evaluation.future_conditions_checked,
    }
}

fn abstain(
    proposal: &ActionProposal,
    observation: &Observation,
    policy: &AdmissionPolicy,
    checked_at_unix_seconds: u64,
    reason: AdmissionReason,
) -> AdmissionReport {
    report(
        proposal,
        observation,
        policy,
        checked_at_unix_seconds,
        AdmissionDisposition::Abstain,
        reason,
        None,
        None,
        Evaluation::default(),
    )
}

fn replan(
    proposal: &ActionProposal,
    observation: &Observation,
    policy: &AdmissionPolicy,
    checked_at_unix_seconds: u64,
    reason: AdmissionReason,
    evaluation: Evaluation,
) -> AdmissionReport {
    report(
        proposal,
        observation,
        policy,
        checked_at_unix_seconds,
        AdmissionDisposition::Replan,
        reason,
        None,
        None,
        evaluation,
    )
}

fn validate_observation(
    observation: &Observation,
    policy: &AdmissionPolicy,
    now: u64,
) -> Result<(), AdmissionReason> {
    match observation.status {
        ObservationStatus::Present => {}
        ObservationStatus::Missing => return Err(AdmissionReason::ObservationMissing),
        ObservationStatus::Conflicting => return Err(AdmissionReason::ObservationConflicting),
    }
    if observation.source_digest != policy.expected_source_digest {
        return Err(AdmissionReason::SourceMismatch);
    }
    if observation.calibration_digest != policy.expected_calibration_digest {
        return Err(AdmissionReason::CalibrationMismatch);
    }
    if observation.epoch != policy.expected_epoch {
        return Err(AdmissionReason::EpochMismatch);
    }
    if observation.sequence <= policy.last_accepted_sequence {
        return Err(AdmissionReason::SequenceReplayOrReorder);
    }
    if observation.sampled_at_unix_seconds > observation.received_at_unix_seconds
        || observation.received_at_unix_seconds > now
        || observation.expires_at_unix_seconds <= observation.sampled_at_unix_seconds
    {
        return Err(AdmissionReason::InvalidTimestampOrder);
    }
    if now >= observation.expires_at_unix_seconds {
        return Err(AdmissionReason::ObservationExpired);
    }
    if now.saturating_sub(observation.sampled_at_unix_seconds)
        > policy.max_observation_age_seconds
    {
        return Err(AdmissionReason::StaleObservation);
    }
    if observation.possible_states.is_empty() {
        return Err(AdmissionReason::EmptyStateSet);
    }
    let mut state_ids = BTreeSet::new();
    if observation
        .possible_states
        .iter()
        .any(|state| state.state_id.trim().is_empty() || !state_ids.insert(state.state_id.as_str()))
    {
        return Err(AdmissionReason::DuplicateStateId);
    }
    Ok(())
}

fn validate_policy(
    battery: BatteryContract,
    policy: &AdmissionPolicy,
) -> Result<(), AdmissionReason> {
    if !battery.is_valid(policy.future_reserve_quanta) {
        return Err(AdmissionReason::InvalidBatteryContract);
    }
    if policy.max_observation_age_seconds == 0 {
        return Err(AdmissionReason::InvalidPolicy);
    }
    if policy.future_conditions.is_empty() {
        return Err(AdmissionReason::EmptyFutureEnvelope);
    }
    let mut ids = BTreeSet::new();
    for condition in &policy.future_conditions {
        if condition.scenario_id.trim().is_empty()
            || !ids.insert(condition.scenario_id.as_str())
            || condition.essential_service_floor_quanta > condition.essential_demand_quanta
        {
            return Err(AdmissionReason::InvalidPolicy);
        }
    }
    Ok(())
}

fn check_immediate_action(
    battery: BatteryContract,
    state: &ObservedState,
    action: ControlAction,
    essential_service_floor_quanta: u16,
) -> Result<u16, ActionFailure> {
    if state.soc_quanta > battery.capacity_quanta {
        return Err(ActionFailure::SocOutsideCapacity);
    }
    if state.grid_outage && action.grid_import_quanta > 0 {
        return Err(ActionFailure::GridImportDuringOutage);
    }
    if action.grid_import_quanta > state.grid_import_limit_quanta {
        return Err(ActionFailure::GridImportLimitExceeded);
    }
    if action.charge_input_quanta > battery.charge_input_limit_quanta {
        return Err(ActionFailure::ChargeLimitExceeded);
    }
    if action.soc_withdrawal_quanta > battery.discharge_withdrawal_limit_quanta
        || action.soc_withdrawal_quanta > state.soc_quanta
    {
        return Err(ActionFailure::DischargeLimitExceeded);
    }
    if action.charge_input_quanta > 0 && action.soc_withdrawal_quanta > 0 {
        return Err(ActionFailure::SimultaneousChargeAndDischarge);
    }
    if action.essential_served_quanta > state.essential_demand_quanta {
        return Err(ActionFailure::EssentialDemandOverServed);
    }
    if action.essential_served_quanta < essential_service_floor_quanta {
        return Err(ActionFailure::EssentialServiceFloorMissed);
    }
    if action.flexible_served_quanta > state.flexible_demand_quanta {
        return Err(ActionFailure::FlexibleDemandOverServed);
    }

    let charge_gain = battery.charge_gain(action.charge_input_quanta);
    let soc_after = u32::from(state.soc_quanta)
        .checked_sub(u32::from(action.soc_withdrawal_quanta))
        .ok_or(ActionFailure::SocUnderflow)?
        .checked_add(u32::from(charge_gain))
        .ok_or(ActionFailure::SocOverflow)?;
    if soc_after > u32::from(battery.capacity_quanta) {
        return Err(ActionFailure::SocOverflow);
    }

    let discharge_delivered = battery.discharge_delivered(action.soc_withdrawal_quanta);
    let non_pv_supply =
        u32::from(action.grid_import_quanta) + u32::from(discharge_delivered);
    let bus_load_and_charge = u32::from(action.essential_served_quanta)
        + u32::from(action.flexible_served_quanta)
        + u32::from(action.charge_input_quanta);
    if bus_load_and_charge < non_pv_supply {
        return Err(ActionFailure::NonPvSupplyExceedsUse);
    }
    let pv_used = bus_load_and_charge - non_pv_supply;
    if pv_used > u32::from(state.pv_available_quanta) {
        return Err(ActionFailure::InsufficientPv);
    }
    Ok(soc_after as u16)
}

fn has_safe_next_service(
    battery: BatteryContract,
    soc_after: u16,
    reserve_quanta: u16,
    condition: &FutureOperatingCondition,
) -> bool {
    let usable_soc = soc_after.saturating_sub(reserve_quanta);
    let maximum_withdrawal = usable_soc.min(battery.discharge_withdrawal_limit_quanta);
    let battery_delivery = u32::from(battery.discharge_delivered(maximum_withdrawal));
    let external_delivery = u32::from(condition.pv_available_quanta)
        + if condition.grid_outage {
            0
        } else {
            u32::from(condition.grid_import_limit_quanta)
        };
    external_delivery + battery_delivery
        >= u32::from(condition.essential_service_floor_quanta)
}

fn evaluate_action(
    battery: BatteryContract,
    proposal: &ActionProposal,
    observation: &Observation,
    policy: &AdmissionPolicy,
    action: ControlAction,
    enforce_proposal_soc_envelope: bool,
) -> Evaluation {
    let mut evaluation = Evaluation::default();
    for state in &observation.possible_states {
        evaluation.current_states_checked += 1;
        if enforce_proposal_soc_envelope
            && (state.soc_quanta < proposal.expected_soc_min_quanta
                || state.soc_quanta > proposal.expected_soc_max_quanta)
        {
            evaluation.failure = Some(ActionFailure::StateOutsideProposalEnvelope);
            return evaluation;
        }

        let soc_after = match check_immediate_action(
            battery,
            state,
            action,
            policy.current_essential_service_floor_quanta,
        ) {
            Ok(soc_after) => soc_after,
            Err(failure) => {
                evaluation.failure = Some(failure);
                return evaluation;
            }
        };

        for condition in &policy.future_conditions {
            evaluation.future_conditions_checked += 1;
            if !has_safe_next_service(
                battery,
                soc_after,
                policy.future_reserve_quanta,
                condition,
            ) {
                evaluation.failure = Some(ActionFailure::NoSafeContinuation);
                return evaluation;
            }
        }
    }
    evaluation
}

/// Independently re-admit one expiring action proposal against every possible
/// admitted current state and every declared next-interval operating condition.
///
/// The monitor never trusts a solver-provided SOC-after value or safety flag.
/// Charge/discharge losses use integer-floor rational arithmetic matching the
/// declared finite dispatch contract. A command receipt or hardware interlock
/// is outside this reference model.
pub fn admit_action(
    battery: BatteryContract,
    proposal: &ActionProposal,
    observation: &Observation,
    policy: &AdmissionPolicy,
    now: u64,
) -> AdmissionReport {
    if let Err(reason) = validate_policy(battery, policy) {
        return abstain(proposal, observation, policy, now, reason);
    }
    if let Err(reason) = validate_observation(observation, policy, now) {
        return abstain(proposal, observation, policy, now, reason);
    }
    if proposal.interval_id.trim().is_empty()
        || proposal.expires_at_unix_seconds <= proposal.issued_at_unix_seconds
        || proposal.expected_soc_min_quanta > proposal.expected_soc_max_quanta
        || proposal.expected_soc_max_quanta > battery.capacity_quanta
        || proposal.issued_at_unix_seconds > now
        || proposal.fallback_action.is_some() != proposal.fallback_action_digest.is_some()
    {
        return abstain(
            proposal,
            observation,
            policy,
            now,
            AdmissionReason::InvalidProposalEnvelope,
        );
    }
    if now >= proposal.expires_at_unix_seconds {
        return replan(
            proposal,
            observation,
            policy,
            now,
            AdmissionReason::ProposalExpired,
            Evaluation::default(),
        );
    }

    let proposed = evaluate_action(
        battery,
        proposal,
        observation,
        policy,
        proposal.proposed_action,
        true,
    );
    if proposed.failure.is_none() {
        return report(
            proposal,
            observation,
            policy,
            now,
            AdmissionDisposition::Admit,
            AdmissionReason::ProposedActionAdmitted,
            Some(proposal.proposed_action),
            Some(proposal.action_digest),
            proposed,
        );
    }
    let proposed_failure = proposed.failure.expect("checked above");

    let Some(fallback) = proposal.fallback_action else {
        return replan(
            proposal,
            observation,
            policy,
            now,
            AdmissionReason::ProposedActionRejected(proposed_failure),
            proposed,
        );
    };
    let fallback_evaluation = evaluate_action(
        battery,
        proposal,
        observation,
        policy,
        fallback,
        false,
    );
    if fallback_evaluation.failure.is_none() {
        return report(
            proposal,
            observation,
            policy,
            now,
            AdmissionDisposition::FallbackAdmitted,
            AdmissionReason::FallbackActionAdmitted,
            Some(fallback),
            proposal.fallback_action_digest,
            fallback_evaluation,
        );
    }
    replan(
        proposal,
        observation,
        policy,
        AdmissionReason::FallbackRejected {
            proposed: proposed_failure,
            fallback: fallback_evaluation
                .failure
                .expect("checked above"),
        },
        fallback_evaluation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn battery() -> BatteryContract {
        BatteryContract {
            capacity_quanta: 10,
            charge_input_limit_quanta: 3,
            discharge_withdrawal_limit_quanta: 3,
            charge_efficiency_numerator: 1,
            charge_efficiency_denominator: 1,
            discharge_efficiency_numerator: 1,
            discharge_efficiency_denominator: 1,
        }
    }

    fn state(id: &str, soc: u16) -> ObservedState {
        ObservedState {
            state_id: id.to_owned(),
            soc_quanta: soc,
            grid_outage: true,
            pv_available_quanta: 0,
            grid_import_limit_quanta: 0,
            essential_demand_quanta: 2,
            flexible_demand_quanta: 0,
        }
    }

    fn observation(states: Vec<ObservedState>) -> Observation {
        Observation {
            status: ObservationStatus::Present,
            observation_digest: Digest::from_byte(4),
            source_digest: Digest::from_byte(1),
            calibration_digest: Digest::from_byte(2),
            epoch: 7,
            sequence: 11,
            sampled_at_unix_seconds: 95,
            received_at_unix_seconds: 96,
            expires_at_unix_seconds: 110,
            possible_states: states,
        }
    }

    fn future(id: &str, demand: u16) -> FutureOperatingCondition {
        FutureOperatingCondition {
            scenario_id: id.to_owned(),
            grid_outage: true,
            pv_available_quanta: 0,
            grid_import_limit_quanta: 0,
            essential_demand_quanta: demand,
            essential_service_floor_quanta: demand,
        }
    }

    fn policy(future_conditions: Vec<FutureOperatingCondition>) -> AdmissionPolicy {
        AdmissionPolicy {
            policy_digest: Digest::from_byte(20),
            battery_model_digest: Digest::from_byte(21),
            uncertainty_profile_digest: Digest::from_byte(22),
            future_envelope_digest: Digest::from_byte(23),
            expected_source_digest: Digest::from_byte(1),
            expected_calibration_digest: Digest::from_byte(2),
            expected_epoch: 7,
            last_accepted_sequence: 10,
            max_observation_age_seconds: 10,
            current_essential_service_floor_quanta: 2,
            future_reserve_quanta: 1,
            future_conditions,
            monitor_digest: Digest::from_byte(3),
        }
    }

    fn proposal(action: ControlAction) -> ActionProposal {
        ActionProposal {
            proposal_digest: Digest::from_byte(9),
            request_digest: Digest::from_byte(10),
            schedule_digest: Digest::from_byte(11),
            action_digest: Digest::from_byte(12),
            fallback_action_digest: None,
            interval_id: "interval-001".to_owned(),
            issued_at_unix_seconds: 97,
            expires_at_unix_seconds: 105,
            expected_soc_min_quanta: 3,
            expected_soc_max_quanta: 5,
            proposed_action: action,
            fallback_action: None,
        }
    }

    fn discharge_two() -> ControlAction {
        ControlAction {
            grid_import_quanta: 0,
            charge_input_quanta: 0,
            soc_withdrawal_quanta: 2,
            essential_served_quanta: 2,
            flexible_served_quanta: 0,
        }
    }

    #[test]
    fn admits_only_when_all_current_states_and_future_conditions_pass() {
        let result = admit_action(
            battery(),
            &proposal(discharge_two()),
            &observation(vec![state("current", 4)]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Admit);
        assert_eq!(result.reason, AdmissionReason::ProposedActionAdmitted);
        assert_eq!(result.selected_action, Some(discharge_two()));
        assert_eq!(result.current_states_checked, 1);
        assert_eq!(result.future_conditions_checked, 1);
    }

    #[test]
    fn locally_valid_soc_at_reserve_is_rejected_without_next_step_viability() {
        let result = admit_action(
            battery(),
            &proposal(discharge_two()),
            &observation(vec![state("current", 3)]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(
            result.reason,
            AdmissionReason::ProposedActionRejected(ActionFailure::NoSafeContinuation)
        );
    }

    #[test]
    fn one_unsafe_possible_state_rejects_the_robust_action() {
        let result = admit_action(
            battery(),
            &proposal(discharge_two()),
            &observation(vec![state("high-soc", 4), state("low-soc", 3)]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(
            result.reason,
            AdmissionReason::ProposedActionRejected(ActionFailure::NoSafeContinuation)
        );
    }

    #[test]
    fn stale_and_replayed_observations_abstain() {
        let mut stale = observation(vec![state("current", 4)]);
        stale.sampled_at_unix_seconds = 80;
        stale.received_at_unix_seconds = 81;
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &stale,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::StaleObservation
        );

        let mut replayed = observation(vec![state("current", 4)]);
        replayed.sequence = 10;
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &replayed,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::SequenceReplayOrReorder
        );
    }

    #[test]
    fn conflicting_missing_or_wrong_source_observations_abstain() {
        let mut conflicting = observation(vec![state("current", 4)]);
        conflicting.status = ObservationStatus::Conflicting;
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &conflicting,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::ObservationConflicting
        );

        let mut missing = observation(vec![state("current", 4)]);
        missing.status = ObservationStatus::Missing;
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &missing,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::ObservationMissing
        );

        let mut wrong_source = observation(vec![state("current", 4)]);
        wrong_source.source_digest = Digest::from_byte(88);
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &wrong_source,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::SourceMismatch
        );
    }

    #[test]
    fn state_outside_the_prior_schedule_envelope_requires_replan() {
        let result = admit_action(
            battery(),
            &proposal(discharge_two()),
            &observation(vec![state("out-of-envelope", 2)]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(
            result.reason,
            AdmissionReason::ProposedActionRejected(ActionFailure::StateOutsideProposalEnvelope)
        );
    }

    #[test]
    fn exact_bus_balance_is_recomputed_instead_of_trusting_proposal() {
        let mut invalid = proposal(ControlAction {
            grid_import_quanta: 2,
            charge_input_quanta: 0,
            soc_withdrawal_quanta: 0,
            essential_served_quanta: 1,
            flexible_served_quanta: 0,
        });
        invalid.expected_soc_min_quanta = 4;
        invalid.expected_soc_max_quanta = 4;
        let mut connected = state("connected", 4);
        connected.grid_outage = false;
        connected.grid_import_limit_quanta = 3;
        let result = admit_action(
            battery(),
            &invalid,
            &observation(vec![connected]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(
            result.reason,
            AdmissionReason::ProposedActionRejected(ActionFailure::NonPvSupplyExceedsUse)
        );
    }

    #[test]
    fn fallback_is_checked_against_the_same_current_and_future_envelope() {
        let mut candidate = proposal(ControlAction {
            grid_import_quanta: 2,
            charge_input_quanta: 0,
            soc_withdrawal_quanta: 0,
            essential_served_quanta: 1,
            flexible_served_quanta: 0,
        });
        candidate.expected_soc_min_quanta = 1;
        candidate.expected_soc_max_quanta = 1;
        candidate.fallback_action_digest = Some(Digest::from_byte(13));
        candidate.fallback_action = Some(ControlAction {
            grid_import_quanta: 0,
            charge_input_quanta: 2,
            soc_withdrawal_quanta: 0,
            essential_served_quanta: 1,
            flexible_served_quanta: 0,
        });
        let mut current = state("pv-surplus", 1);
        current.grid_outage = false;
        current.pv_available_quanta = 3;
        current.essential_demand_quanta = 1;
        let mut next = future("next-outage", 2);
        next.essential_demand_quanta = 2;

        let result = admit_action(
            battery(),
            &candidate,
            &observation(vec![current]),
            &policy(vec![next]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::FallbackAdmitted);
        assert_eq!(result.reason, AdmissionReason::FallbackActionAdmitted);
        assert_eq!(result.selected_action, candidate.fallback_action);
        assert_eq!(result.selected_action_digest, candidate.fallback_action_digest);
    }

    #[test]
    fn empty_future_envelope_and_expired_proposals_never_admit() {
        let empty = admit_action(
            battery(),
            &proposal(discharge_two()),
            &observation(vec![state("current", 4)]),
            &policy(Vec::new()),
            100,
        );
        assert_eq!(empty.disposition, AdmissionDisposition::Abstain);
        assert_eq!(empty.reason, AdmissionReason::EmptyFutureEnvelope);

        let mut expired = proposal(discharge_two());
        expired.expires_at_unix_seconds = 100;
        let result = admit_action(
            battery(),
            &expired,
            &observation(vec![state("current", 4)]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(result.reason, AdmissionReason::ProposalExpired);
    }

    #[test]
    fn rational_efficiency_uses_conservative_integer_floor() {
        let battery = BatteryContract {
            discharge_efficiency_numerator: 3,
            discharge_efficiency_denominator: 4,
            ..battery()
        };
        let mut next = future("next-outage", 2);
        let mut current = state("current", 4);
        current.essential_demand_quanta = 1;
        let action = ControlAction {
            grid_import_quanta: 0,
            charge_input_quanta: 0,
            soc_withdrawal_quanta: 2,
            essential_served_quanta: 1,
            flexible_served_quanta: 0,
        };
        let mut configured = policy(vec![next.clone()]);
        configured.current_essential_service_floor_quanta = 1;
        let result = admit_action(
            battery,
            &proposal(action),
            &observation(vec![current]),
            &configured,
            100,
        );
        // 2 internal quanta yield floor(2 * 3 / 4) = 1 delivered quantum;
        // after the withdrawal only 2 SOC quanta remain, one is reserved.
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        next.essential_service_floor_quanta = 1;
        configured.future_conditions = vec![next];
        assert_eq!(
            admit_action(
                battery,
                &proposal(action),
                &observation(vec![state("current", 5)]),
                &configured,
                100,
            )
            .disposition,
            AdmissionDisposition::Admit
        );
    }

    #[test]
    fn every_future_scenario_must_retain_an_essential_service_continuation() {
        let result = admit_action(
            battery(),
            &proposal(discharge_two()),
            &observation(vec![state("current", 4)]),
            &policy(vec![
                future("short-outage", 1),
                future("prolonged-outage", 2),
            ]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(
            result.reason,
            AdmissionReason::ProposedActionRejected(ActionFailure::NoSafeContinuation)
        );
        assert_eq!(result.future_conditions_checked, 2);
    }

    #[test]
    fn fallback_without_its_own_digest_is_rejected_before_evaluation() {
        let mut candidate = proposal(ControlAction {
            grid_import_quanta: 2,
            charge_input_quanta: 0,
            soc_withdrawal_quanta: 0,
            essential_served_quanta: 1,
            flexible_served_quanta: 0,
        });
        candidate.expected_soc_min_quanta = 1;
        candidate.expected_soc_max_quanta = 1;
        candidate.fallback_action = Some(ControlAction {
            grid_import_quanta: 0,
            charge_input_quanta: 2,
            soc_withdrawal_quanta: 0,
            essential_served_quanta: 1,
            flexible_served_quanta: 0,
        });
        let mut current = state("pv-surplus", 1);
        current.grid_outage = false;
        current.pv_available_quanta = 3;
        current.essential_demand_quanta = 1;
        let result = admit_action(
            battery(),
            &candidate,
            &observation(vec![current]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Abstain);
        assert_eq!(result.reason, AdmissionReason::InvalidProposalEnvelope);
        assert_eq!(result.selected_action, None);
    }

    #[test]
    fn observation_expiry_and_future_sample_timestamp_fail_closed() {
        let mut expired_observation = observation(vec![state("current", 4)]);
        expired_observation.expires_at_unix_seconds = 100;
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &expired_observation,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::ObservationExpired
        );

        let mut future_sample = observation(vec![state("current", 4)]);
        future_sample.sampled_at_unix_seconds = 101;
        future_sample.received_at_unix_seconds = 101;
        assert_eq!(
            admit_action(
                battery(),
                &proposal(discharge_two()),
                &future_sample,
                &policy(vec![future("next-outage", 1)]),
                100,
            )
            .reason,
            AdmissionReason::InvalidTimestampOrder
        );
    }

    #[test]
    fn missing_fallback_is_replan_not_an_implicit_admission() {
        let mut candidate = proposal(ControlAction {
            grid_import_quanta: 0,
            charge_input_quanta: 0,
            soc_withdrawal_quanta: 4,
            essential_served_quanta: 2,
            flexible_served_quanta: 0,
        });
        candidate.expected_soc_min_quanta = 4;
        candidate.expected_soc_max_quanta = 4;
        let result = admit_action(
            battery(),
            &candidate,
            &observation(vec![state("current", 4)]),
            &policy(vec![future("next-outage", 1)]),
            100,
        );
        assert_eq!(result.disposition, AdmissionDisposition::Replan);
        assert_eq!(
            result.reason,
            AdmissionReason::ProposedActionRejected(ActionFailure::DischargeLimitExceeded)
        );
        assert_eq!(result.selected_action, None);
    }

    #[test]
    fn report_binds_the_exact_proposal_observation_and_monitor() {
        let candidate = proposal(discharge_two());
        let observed = observation(vec![state("current", 4)]);
        let configured = policy(vec![future("next-outage", 1)]);
        let result = admit_action(battery(), &candidate, &observed, &configured, 100);
        assert_eq!(result.proposal_digest, candidate.proposal_digest);
        assert_eq!(result.request_digest, candidate.request_digest);
        assert_eq!(result.schedule_digest, candidate.schedule_digest);
        assert_eq!(result.action_digest, candidate.action_digest);
        assert_eq!(result.observation_digest, observed.observation_digest);
        assert_eq!(result.policy_digest, configured.policy_digest);
        assert_eq!(result.battery_model_digest, configured.battery_model_digest);
        assert_eq!(result.uncertainty_profile_digest, configured.uncertainty_profile_digest);
        assert_eq!(result.future_envelope_digest, configured.future_envelope_digest);
        assert_eq!(result.monitor_digest, configured.monitor_digest);
    }
}
