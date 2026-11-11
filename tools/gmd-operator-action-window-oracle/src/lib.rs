//! Conservative GMD mitigation action-window evaluator.
//!
//! All times are elapsed seconds from a declared decision reference instant.
//! This crate evaluates a supplied candidate action; it does not select,
//! authorize, schedule, dispatch or execute actions.

use std::error::Error;
use std::fmt;

/// Broad action category used for stratified benchmark reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionClass {
    Operational,
    Maintenance,
    Capital,
}

/// Reversibility is reported, never converted into an automatic risk score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    Reversible,
    ConditionallyReversible,
    Irreversible,
    Unknown,
}

/// Owner-defined relative cost band. Bands are not comparable across owners
/// unless the benchmark separately defines a common cost basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostBand {
    NoNewSpend,
    Low,
    Moderate,
    High,
    Unknown,
}

/// Range of elapsed duration for one action stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationRange {
    pub minimum_seconds: u64,
    pub maximum_seconds: u64,
}

/// Quality/freshness of the evidence required by the action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceState {
    Fresh,
    Stale,
    Unknown,
}

/// Current state of the required human authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityState {
    Approved,
    Pending,
    Denied,
    Unknown,
}

/// Availability of required crews, equipment, fuel, spares or other resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Available,
    Unavailable,
    Unknown,
}

/// Whether the action is bound to the correct owner and operational scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeState {
    Verified,
    Conflict,
    Unknown,
}

/// One candidate action and its event-relative timing/authority evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionWindowInput {
    pub action_id: String,
    pub owner_ref: String,
    pub procedure_revision: String,
    pub procedure_sha256: String,
    pub forecast_ref: String,
    pub forecast_sha256: String,
    pub action_class: ActionClass,
    pub reversibility: Reversibility,
    pub cost_band: CostBand,
    /// Earliest and latest impact arrival measured from decision t0.
    pub earliest_impact_seconds: u64,
    pub latest_impact_seconds: u64,
    /// Required safety buffer before earliest/latest impact.
    pub safety_buffer_seconds: u64,
    pub approval_duration: DurationRange,
    pub preparation_duration: DurationRange,
    pub mobilisation_duration: DurationRange,
    pub execution_duration: DurationRange,
    pub evidence_state: EvidenceState,
    pub authority_state: AuthorityState,
    pub resource_state: ResourceState,
    pub scope_state: ScopeState,
}

/// Independent readiness gate. A ready result means only ready for operator
/// review; it is never a command or execution authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessDisposition {
    ReadyForOperatorReview,
    AwaitingHumanApproval,
    BlockedByAuthority,
    BlockedByResources,
    StaleEvidence,
    ScopeConflict,
    Indeterminate,
}

/// Timing class based on duration bounds and forecast arrival bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingDisposition {
    ConservativeFit,
    ArrivalDependentAtRisk,
    MissedWindow,
}

/// Qualified result: readiness and timing remain separate dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionWindowResult {
    pub action_id: String,
    pub action_class: ActionClass,
    pub reversibility: Reversibility,
    pub cost_band: CostBand,
    pub minimum_completion_seconds: u64,
    pub maximum_completion_seconds: u64,
    pub conservative_deadline_seconds: u64,
    pub latest_safe_deadline_seconds: u64,
    pub readiness: ReadinessDisposition,
    pub timing: TimingDisposition,
}

/// Fail-closed validation and arithmetic errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionWindowError {
    EmptyRequiredField,
    InvalidDigest,
    InvalidImpactWindow,
    InvalidDurationRange,
    ApprovedActionHasPendingApprovalDuration,
    DurationOverflow,
}

impl fmt::Display for ActionWindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyRequiredField => "action, owner, procedure and forecast references are required",
            Self::InvalidDigest => "procedure and forecast digests must be 64 lowercase hexadecimal characters",
            Self::InvalidImpactWindow => "impact arrival window must be positive and ordered",
            Self::InvalidDurationRange => "each duration range minimum must not exceed its maximum",
            Self::ApprovedActionHasPendingApprovalDuration => "approved action cannot retain a pending approval duration",
            Self::DurationOverflow => "action stage duration sum overflowed",
        };
        f.write_str(message)
    }
}

impl Error for ActionWindowError {}

/// Evaluate a candidate action without granting authority or performing effects.
///
/// The decision reference instant is t0. Forecast arrival bounds and all stage
/// durations are elapsed seconds after t0; no GNSS/UTC timestamp is required.
pub fn evaluate_action_window(
    input: &ActionWindowInput,
) -> Result<ActionWindowResult, ActionWindowError> {
    for field in [
        input.action_id.as_str(),
        input.owner_ref.as_str(),
        input.procedure_revision.as_str(),
        input.forecast_ref.as_str(),
    ] {
        if field.trim().is_empty() || field.trim() != field {
            return Err(ActionWindowError::EmptyRequiredField);
        }
    }
    if !valid_sha256(&input.procedure_sha256) || !valid_sha256(&input.forecast_sha256) {
        return Err(ActionWindowError::InvalidDigest);
    }
    if input.earliest_impact_seconds == 0
        || input.latest_impact_seconds < input.earliest_impact_seconds
    {
        return Err(ActionWindowError::InvalidImpactWindow);
    }
    for duration in [
        input.approval_duration,
        input.preparation_duration,
        input.mobilisation_duration,
        input.execution_duration,
    ] {
        if duration.minimum_seconds > duration.maximum_seconds {
            return Err(ActionWindowError::InvalidDurationRange);
        }
    }
    if input.authority_state == AuthorityState::Approved
        && (input.approval_duration.minimum_seconds != 0
            || input.approval_duration.maximum_seconds != 0)
    {
        return Err(ActionWindowError::ApprovedActionHasPendingApprovalDuration);
    }

    let minimum_completion_seconds = sum_durations([
        input.approval_duration.minimum_seconds,
        input.preparation_duration.minimum_seconds,
        input.mobilisation_duration.minimum_seconds,
        input.execution_duration.minimum_seconds,
    ])?;
    let maximum_completion_seconds = sum_durations([
        input.approval_duration.maximum_seconds,
        input.preparation_duration.maximum_seconds,
        input.mobilisation_duration.maximum_seconds,
        input.execution_duration.maximum_seconds,
    ])?;

    let conservative_deadline_seconds = input
        .earliest_impact_seconds
        .saturating_sub(input.safety_buffer_seconds);
    let latest_safe_deadline_seconds = input
        .latest_impact_seconds
        .saturating_sub(input.safety_buffer_seconds);

    let timing = if maximum_completion_seconds <= conservative_deadline_seconds {
        TimingDisposition::ConservativeFit
    } else if minimum_completion_seconds <= latest_safe_deadline_seconds {
        TimingDisposition::ArrivalDependentAtRisk
    } else {
        TimingDisposition::MissedWindow
    };

    let readiness = assess_readiness(
        input.evidence_state,
        input.authority_state,
        input.resource_state,
        input.scope_state,
    );

    Ok(ActionWindowResult {
        action_id: input.action_id.clone(),
        action_class: input.action_class,
        reversibility: input.reversibility,
        cost_band: input.cost_band,
        minimum_completion_seconds,
        maximum_completion_seconds,
        conservative_deadline_seconds,
        latest_safe_deadline_seconds,
        readiness,
        timing,
    })
}

fn assess_readiness(
    evidence: EvidenceState,
    authority: AuthorityState,
    resources: ResourceState,
    scope: ScopeState,
) -> ReadinessDisposition {
    if scope == ScopeState::Conflict || authority == AuthorityState::Denied {
        return if scope == ScopeState::Conflict {
            ReadinessDisposition::ScopeConflict
        } else {
            ReadinessDisposition::BlockedByAuthority
        };
    }
    if resources == ResourceState::Unavailable {
        return ReadinessDisposition::BlockedByResources;
    }
    if evidence == EvidenceState::Stale {
        return ReadinessDisposition::StaleEvidence;
    }
    if authority == AuthorityState::Pending {
        return ReadinessDisposition::AwaitingHumanApproval;
    }
    if evidence == EvidenceState::Unknown
        || authority == AuthorityState::Unknown
        || resources == ResourceState::Unknown
        || scope == ScopeState::Unknown
    {
        return ReadinessDisposition::Indeterminate;
    }
    ReadinessDisposition::ReadyForOperatorReview
}

fn sum_durations(values: [u64; 4]) -> Result<u64, ActionWindowError> {
    values.into_iter().try_fold(0_u64, |sum, value| {
        sum.checked_add(value).ok_or(ActionWindowError::DurationOverflow)
    })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn range(minimum_seconds: u64, maximum_seconds: u64) -> DurationRange {
        DurationRange {
            minimum_seconds,
            maximum_seconds,
        }
    }

    fn input() -> ActionWindowInput {
        ActionWindowInput {
            action_id: "reduce-transformer-loading".to_owned(),
            owner_ref: "urn:test:owner:grid-operator".to_owned(),
            procedure_revision: "gmd-procedure-r4".to_owned(),
            procedure_sha256: DIGEST.to_owned(),
            forecast_ref: "urn:test:forecast:local-impact-01".to_owned(),
            forecast_sha256: DIGEST.to_owned(),
            action_class: ActionClass::Operational,
            reversibility: Reversibility::ConditionallyReversible,
            cost_band: CostBand::Low,
            earliest_impact_seconds: 100,
            latest_impact_seconds: 140,
            safety_buffer_seconds: 10,
            approval_duration: range(0, 0),
            preparation_duration: range(10, 10),
            mobilisation_duration: range(20, 20),
            execution_duration: range(30, 30),
            evidence_state: EvidenceState::Fresh,
            authority_state: AuthorityState::Approved,
            resource_state: ResourceState::Available,
            scope_state: ScopeState::Verified,
        }
    }

    #[test]
    fn conservative_fit_requires_maximum_duration_before_earliest_safe_deadline() {
        let result = evaluate_action_window(&input()).unwrap();
        assert_eq!(result.minimum_completion_seconds, 60);
        assert_eq!(result.maximum_completion_seconds, 60);
        assert_eq!(result.conservative_deadline_seconds, 90);
        assert_eq!(result.latest_safe_deadline_seconds, 130);
        assert_eq!(result.timing, TimingDisposition::ConservativeFit);
        assert_eq!(result.readiness, ReadinessDisposition::ReadyForOperatorReview);
    }

    #[test]
    fn duration_and_arrival_uncertainty_produce_at_risk_classification() {
        let mut candidate = input();
        candidate.preparation_duration = range(10, 40);
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.minimum_completion_seconds, 60);
        assert_eq!(result.maximum_completion_seconds, 90);
        assert_eq!(result.timing, TimingDisposition::ConservativeFit);
    }

    #[test]
    fn earliest_arrival_failure_but_latest_arrival_fit_is_at_risk() {
        let mut candidate = input();
        candidate.preparation_duration = range(40, 70);
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.minimum_completion_seconds, 90);
        assert_eq!(result.maximum_completion_seconds, 120);
        assert_eq!(result.timing, TimingDisposition::ArrivalDependentAtRisk);
    }

    #[test]
    fn minimum_completion_after_latest_safe_deadline_is_missed() {
        let mut candidate = input();
        candidate.preparation_duration = range(90, 100);
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.minimum_completion_seconds, 140);
        assert_eq!(result.latest_safe_deadline_seconds, 130);
        assert_eq!(result.timing, TimingDisposition::MissedWindow);
    }

    #[test]
    fn exact_earliest_deadline_boundary_is_conservative_fit() {
        let mut candidate = input();
        candidate.preparation_duration = range(40, 60);
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.maximum_completion_seconds, 110);
        assert_eq!(result.conservative_deadline_seconds, 90);
        assert_eq!(result.timing, TimingDisposition::ArrivalDependentAtRisk);
    }

    #[test]
    fn pending_human_approval_never_becomes_ready_to_review() {
        let mut candidate = input();
        candidate.authority_state = AuthorityState::Pending;
        candidate.approval_duration = range(5, 15);
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.readiness, ReadinessDisposition::AwaitingHumanApproval);
        assert_eq!(result.timing, TimingDisposition::ConservativeFit);
    }

    #[test]
    fn denied_authority_blocks_even_a_conservative_timing_fit() {
        let mut candidate = input();
        candidate.authority_state = AuthorityState::Denied;
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.timing, TimingDisposition::ConservativeFit);
        assert_eq!(result.readiness, ReadinessDisposition::BlockedByAuthority);
    }

    #[test]
    fn stale_evidence_blocks_readiness() {
        let mut candidate = input();
        candidate.evidence_state = EvidenceState::Stale;
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.readiness, ReadinessDisposition::StaleEvidence);
    }

    #[test]
    fn unavailable_resources_block_readiness() {
        let mut candidate = input();
        candidate.resource_state = ResourceState::Unavailable;
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.readiness, ReadinessDisposition::BlockedByResources);
    }

    #[test]
    fn unknown_resource_state_is_indeterminate() {
        let mut candidate = input();
        candidate.resource_state = ResourceState::Unknown;
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.readiness, ReadinessDisposition::Indeterminate);
    }

    #[test]
    fn unknown_scope_is_indeterminate_and_conflicting_scope_blocks() {
        let mut candidate = input();
        candidate.scope_state = ScopeState::Unknown;
        assert_eq!(
            evaluate_action_window(&candidate).unwrap().readiness,
            ReadinessDisposition::Indeterminate
        );
        candidate.scope_state = ScopeState::Conflict;
        assert_eq!(
            evaluate_action_window(&candidate).unwrap().readiness,
            ReadinessDisposition::ScopeConflict
        );
    }

    #[test]
    fn invalid_impact_interval_is_rejected() {
        let mut candidate = input();
        candidate.earliest_impact_seconds = 150;
        candidate.latest_impact_seconds = 140;
        assert_eq!(
            evaluate_action_window(&candidate),
            Err(ActionWindowError::InvalidImpactWindow)
        );
    }

    #[test]
    fn invalid_stage_duration_range_is_rejected() {
        let mut candidate = input();
        candidate.mobilisation_duration = range(20, 10);
        assert_eq!(
            evaluate_action_window(&candidate),
            Err(ActionWindowError::InvalidDurationRange)
        );
    }

    #[test]
    fn approved_authority_cannot_retain_pending_approval_time() {
        let mut candidate = input();
        candidate.approval_duration = range(1, 5);
        assert_eq!(
            evaluate_action_window(&candidate),
            Err(ActionWindowError::ApprovedActionHasPendingApprovalDuration)
        );
    }

    #[test]
    fn duration_sum_overflow_is_rejected() {
        let mut candidate = input();
        candidate.preparation_duration = range(u64::MAX, u64::MAX);
        assert_eq!(
            evaluate_action_window(&candidate),
            Err(ActionWindowError::DurationOverflow)
        );
    }

    #[test]
    fn malformed_forecast_digest_is_rejected() {
        let mut candidate = input();
        candidate.forecast_sha256 = "not-a-digest".to_owned();
        assert_eq!(
            evaluate_action_window(&candidate),
            Err(ActionWindowError::InvalidDigest)
        );
    }

    #[test]
    fn excessive_safety_margin_does_not_create_a_false_window() {
        let mut candidate = input();
        candidate.safety_buffer_seconds = 200;
        let result = evaluate_action_window(&candidate).unwrap();
        assert_eq!(result.conservative_deadline_seconds, 0);
        assert_eq!(result.latest_safe_deadline_seconds, 0);
        assert_eq!(result.timing, TimingDisposition::MissedWindow);
    }
}
