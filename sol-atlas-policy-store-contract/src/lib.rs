// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Storage-neutral atomic persistence contract for Sol Atlas policy consumption.
//!
//! The core remains renderer- and storage-neutral. This package is an adapter
//! boundary: it validates a transition against a freshly loaded state and asks
//! an implementation to atomically compare-and-set the exact successor.
//!
//! The contract deliberately does not choose a database, transport, locking
//! primitive, or external authorization mechanism.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::{
    RecoveryExecution, RecoveryExecutionResultSnapshotV1,
    RecoveryPolicyConsumptionSnapshotV1, RecoveryPolicyConsumptionStateV1,
    RecoveryPolicyConsumptionTransitionV1, RecoveryPolicyDecisionSnapshotV1,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

/// Persistence outcomes visible to an adapter caller.
///
/// Semantic rejection is distinct from store failure: only Committed means
/// the adapter may report that consumption was persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryPolicyConsumptionPersistenceOutcome {
    /// The store positively acknowledged that the exact successor was committed.
    Committed,
    /// The expected version was no longer current and this operation did not commit.
    Conflict,
    /// The store could not establish whether the atomic commit happened.
    ///
    /// A caller must neither report success nor blindly retry a single-use
    /// action without reconciling the resulting state.
    CommitIndeterminate,
    ReplayDetected,
    MissingState,
    InvalidTransition,
    MalformedDecision,
    MalformedTransition,
    MalformedSuccessor,
    MalformedStoredState,
}

/// Storage failure is intentionally opaque to this contract package.
#[derive(Debug)]
pub enum RecoveryPolicyConsumptionPersistenceError<E> {
    Store(E),
}

/// Error boundary for one operation that reads the authoritative fence store
/// and then mutates a separate effect store. The stores remain independent
/// transaction boundaries; this type does not imply a cross-store transaction.
#[derive(Debug)]
pub enum RecoveryExecutionCrossStorePersistenceError<FenceError, EffectError> {
    FenceStore(FenceError),
    EffectStore(EffectError),
}

/// Result of the single atomic persistence decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPolicyConsumptionCasResult {
    /// The exact supplied successor was committed.
    Committed,
    /// The expected version was not current; this operation did not commit.
    NotCurrent,
    /// The store cannot determine whether the commit happened.
    Indeterminate,
}

/// A minimal compare-and-set store required by the persistence boundary.
///
/// Implementations MUST make compare_and_set atomic with respect to all
/// contenders for the same authorization key.
///
/// Committed means the exact supplied successor was committed.
/// NotCurrent means the expected version was no longer current and this
/// operation did not commit.
/// Indeterminate means the store cannot determine whether the commit
/// happened (for example, the database may have committed before the response
/// was lost). Callers must not report success or blindly retry a single-use
/// action in that state.
pub trait RecoveryPolicyConsumptionStore: Send + Sync {
    type Error;

    fn load(
        &self,
        decision_digest: &str,
    ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error>;

    fn compare_and_set(
        &self,
        decision_digest: &str,
        expected_snapshot_digest: &str,
        next: &RecoveryPolicyConsumptionSnapshotV1,
    ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error>;
}

/// Point-in-time result of reconciling an indeterminate CAS acknowledgement.
///
/// These outcomes describe what a subsequent load observed. They intentionally
/// do not turn a generic read into a durable guarantee about future state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPolicyConsumptionReconciliationOutcome {
    /// The exact successor state is currently stored.
    ObservedCommitted,
    /// The exact expected pre-state is currently stored.
    ObservedExpected,
    /// A different valid state is currently stored.
    ///
    /// Another contender may have won, or an adapter-specific mutation may
    /// have replaced the state. The generic contract does not infer which.
    ObservedDifferentState,
    MissingState,
    InvalidTransition,
    MalformedStoredState,
}

/// Reconcile an indeterminate persistence acknowledgement using the exact
/// successor's content digest as the unique observable side effect.
///
/// This function performs only a read. It never retries CAS and never mutates
/// the store. If the successor is observed, the caller can safely conclude
/// that this exact successor is present at reconciliation time. If the expected
/// pre-state is observed, the caller only knows that the successor is not
/// currently present; an adapter may require stronger store-specific guarantees
/// before treating that as a definitive non-commit result.
pub fn reconcile_indeterminate_consumption<S>(
    store: &S,
    decision: &RecoveryPolicyDecisionSnapshotV1,
    transition: &RecoveryPolicyConsumptionTransitionV1,
    next: &RecoveryPolicyConsumptionSnapshotV1,
) -> Result<
    RecoveryPolicyConsumptionReconciliationOutcome,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryPolicyConsumptionStore,
{
    if !decision.is_well_formed()
        || !transition.is_well_formed()
        || !next.is_well_formed()
        || transition.decision_digest != decision.digest()
        || transition.next_snapshot_digest != next.digest()
        || next.decision_digest != decision.digest()
        || next.state != RecoveryPolicyConsumptionStateV1::Consumed
        || next.consumed_execution_id.as_deref() != Some(transition.execution_id.as_str())
        || next.consumed_at.as_deref() != Some(transition.consumed_at.as_str())
        || next.claim_ceiling != transition.claim_ceiling
    {
        return Ok(RecoveryPolicyConsumptionReconciliationOutcome::InvalidTransition);
    }

    let Some(current) = store
        .load(&decision.digest())
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?
    else {
        return Ok(RecoveryPolicyConsumptionReconciliationOutcome::MissingState);
    };

    if !current.is_well_formed() {
        return Ok(
            RecoveryPolicyConsumptionReconciliationOutcome::MalformedStoredState
        );
    }

    if current.decision_digest != decision.digest() {
        return Ok(
            RecoveryPolicyConsumptionReconciliationOutcome::InvalidTransition
        );
    }

    if current.digest() == next.digest() {
        return Ok(
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedCommitted
        );
    }

    if current.digest() == transition.expected_snapshot_digest {
        return Ok(
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedExpected
        );
    }

    Ok(RecoveryPolicyConsumptionReconciliationOutcome::ObservedDifferentState)
}

/// Pure authorization/claim orchestration observation.
///
/// These states describe only what the caller has observed across the separate
/// authorization and execution-claim stores. They do not imply cross-store
/// atomicity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAuthorizationOrchestrationStateV1 {
    AuthorizationUnconsumedNoClaim,
    AuthorizationUnconsumedSameClaim,
    AuthorizationConsumedSameClaim,
    AuthorizationConsumedNoClaim,
    AuthorizationConsumptionIndeterminate,
}

/// Safe next action for the authorization/claim boundary.
///
/// This is deliberately a decision table, not a mutating operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAuthorizationOrchestrationActionV1 {
    AcquireExecutionClaim,
    ConsumeAuthorization,
    ReconcileAndContinueExecution,
    ReconcileExecutionAndEffectBeforeIrreversibleWork,
    ReconcileAuthorizationConsumption,
}

impl RecoveryAuthorizationOrchestrationStateV1 {
    pub fn next_action(self) -> RecoveryAuthorizationOrchestrationActionV1 {
        match self {
            Self::AuthorizationUnconsumedNoClaim => {
                RecoveryAuthorizationOrchestrationActionV1::AcquireExecutionClaim
            }
            Self::AuthorizationUnconsumedSameClaim => {
                RecoveryAuthorizationOrchestrationActionV1::ConsumeAuthorization
            }
            Self::AuthorizationConsumedSameClaim => {
                RecoveryAuthorizationOrchestrationActionV1::ReconcileAndContinueExecution
            }
            Self::AuthorizationConsumedNoClaim => {
                RecoveryAuthorizationOrchestrationActionV1::
                    ReconcileExecutionAndEffectBeforeIrreversibleWork
            }
            Self::AuthorizationConsumptionIndeterminate => {
                RecoveryAuthorizationOrchestrationActionV1::ReconcileAuthorizationConsumption
            }
        }
    }
}

/// Pure effect orchestration observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryEffectOrchestrationStateV1 {
    InProgressSameAttempt,
    InProgressOtherAttempt,
    Succeeded,
    Failed,
}

/// Safe next action for the external-effect boundary.
///
/// Terminal outcomes are replayable history. Only InProgress is live mutable
/// ownership and therefore requires attempt/fence handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryEffectOrchestrationActionV1 {
    ReconcileWithoutRestart,
    FailClosedOrUseAdapterSpecificRecovery,
    ReturnRecordedSuccess,
    ApplyExplicitRetryPolicy,
}

impl RecoveryEffectOrchestrationStateV1 {
    pub fn next_action(self) -> RecoveryEffectOrchestrationActionV1 {
        match self {
            Self::InProgressSameAttempt => {
                RecoveryEffectOrchestrationActionV1::ReconcileWithoutRestart
            }
            Self::InProgressOtherAttempt => {
                RecoveryEffectOrchestrationActionV1::FailClosedOrUseAdapterSpecificRecovery
            }
            Self::Succeeded => RecoveryEffectOrchestrationActionV1::ReturnRecordedSuccess,
            Self::Failed => RecoveryEffectOrchestrationActionV1::ApplyExplicitRetryPolicy,
        }
    }
}

/// Durable idempotency/ownership record for one concrete execution identity.
///
/// The exact execution-input snapshot is the request fingerprint. The opaque
/// attempt identifier distinguishes the caller/request that first acquired the
/// execution claim from a later competing attempt. This record does not mean
/// the external execution side effect has happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryExecutionClaimV1 {
    pub schema: String,
    pub execution_id: String,
    pub execution_input_snapshot: String,
    pub attempt_id: String,
}

impl RecoveryExecutionClaimV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-execution-claim:v1";

    /// Construct a claim directly from the execution's canonical input snapshot.
    ///
    /// The snapshot digest must already have been established by the execution
    /// admission path; this constructor does not reinterpret execution inputs.
    pub fn for_execution(
        execution: &RecoveryExecution,
        attempt_id: impl Into<String>,
    ) -> Option<Self> {
        if execution.ended_at.is_some() {
            return None;
        }

        let claim = Self {
            schema: Self::SCHEMA.into(),
            execution_id: execution.execution_id.clone(),
            execution_input_snapshot: execution.input_snapshot.clone(),
            attempt_id: attempt_id.into(),
        };
        claim.is_well_formed().then_some(claim)
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.execution_id.is_empty()
            && is_sha256_digest(&self.execution_input_snapshot)
            && !self.attempt_id.is_empty()
    }
}

/// Result of the atomic execution-claim attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionClaimResult {
    /// This attempt atomically acquired the execution claim.
    Acquired,
    /// The same attempt already owns the exact same execution fingerprint.
    ///
    /// This is an idempotent replay of the same logical start request. It does
    /// not establish that the external effect completed.
    AlreadyClaimedSameAttempt,
    /// Another attempt owns the exact same execution fingerprint.
    AlreadyClaimedDifferentAttempt,
    /// The execution identity is already bound to a different fingerprint.
    ///
    /// Reusing an execution identity for a different request is rejected
    /// rather than silently mutating the meaning of the execution.
    ExecutionIdentityReuseMismatch,
    /// The supplied claim is structurally malformed and must not reach the store.
    MalformedClaim,
    /// The store cannot determine whether the claim was acquired.
    Indeterminate,
}

/// Storage-neutral boundary for one execution identity's durable claim.
///
/// Implementations MUST make claim_if_absent atomic for the same
/// execution_id. A conforming implementation never replaces an existing
/// claim with a different attempt or execution fingerprint.
pub trait RecoveryExecutionClaimStore: Send + Sync {
    type Error;

    /// Atomically establish the first durable owner for an execution identity.
    ///
    /// Implementations MUST reject malformed claims with MalformedClaim and
    /// MUST NOT mutate state when rejecting them. An existing claim must never
    /// be replaced by a different attempt or fingerprint.
    fn claim_if_absent(
        &self,
        claim: &RecoveryExecutionClaimV1,
    ) -> Result<RecoveryExecutionClaimResult, Self::Error>;

    fn load_claim(
        &self,
        execution_id: &str,
    ) -> Result<Option<RecoveryExecutionClaimV1>, Self::Error>;
}

/// Typed reconciliation result after an indeterminate execution-claim
/// acknowledgement. This is observation only; it never retries the claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionClaimReconciliationOutcome {
    ObservedOwnedByThisAttempt,
    ObservedOwnedByOtherAttempt,
    ObservedDifferentFingerprint,
    MissingClaim,
    InvalidClaim,
}

/// Reconcile an indeterminate execution claim without mutating the store.
///
/// Observing the exact claim proves ownership of the durable claim record at
/// reconciliation time. It does not prove that an external side effect has
/// completed or that this caller's process is still the one performing it.
pub fn reconcile_execution_claim<S>(
    store: &S,
    claim: &RecoveryExecutionClaimV1,
) -> Result<
    RecoveryExecutionClaimReconciliationOutcome,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionClaimStore,
{
    if !claim.is_well_formed() {
        return Ok(RecoveryExecutionClaimReconciliationOutcome::InvalidClaim);
    }

    let Some(current) = store
        .load_claim(&claim.execution_id)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?
    else {
        return Ok(RecoveryExecutionClaimReconciliationOutcome::MissingClaim);
    };

    if !current.is_well_formed() {
        return Ok(RecoveryExecutionClaimReconciliationOutcome::InvalidClaim);
    }

    if current.execution_input_snapshot != claim.execution_input_snapshot {
        return Ok(
            RecoveryExecutionClaimReconciliationOutcome::ObservedDifferentFingerprint,
        );
    }

    if current.attempt_id == claim.attempt_id {
        return Ok(
            RecoveryExecutionClaimReconciliationOutcome::ObservedOwnedByThisAttempt,
        );
    }

    Ok(RecoveryExecutionClaimReconciliationOutcome::ObservedOwnedByOtherAttempt)
}

/// Attempt to acquire an execution claim.
///
/// This helper is intentionally separate from policy authorization consumption:
/// a persisted authorization winner and an execution-side-effect owner are
/// distinct facts and must not be conflated.
pub fn claim_execution_start<S>(
    store: &S,
    claim: &RecoveryExecutionClaimV1,
) -> Result<
    RecoveryExecutionClaimResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionClaimStore,
{
    if !claim.is_well_formed() {
        return Ok(RecoveryExecutionClaimResult::MalformedClaim);
    }

    store
        .claim_if_absent(claim)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
}

/// Durable receipt state for one external execution effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryExecutionEffectStateV1 {
    InProgress,
    Succeeded,
    Failed,
}

/// Durable idempotency receipt for the external effect of one execution.
///
/// The execution-input snapshot is the semantic fingerprint. The attempt
/// identifier establishes which execution claim may advance an InProgress
/// receipt. Outcome identity is content-addressed; the receipt does not
/// contain or imply the external side effect itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryExecutionEffectReceiptV2 {
    pub schema: String,
    pub execution_id: String,
    pub execution_input_snapshot: String,
    pub attempt_id: String,
    pub fence_epoch: u64,
    pub state: RecoveryExecutionEffectStateV1,
    pub outcome_digest: Option<String>,
}

impl RecoveryExecutionEffectReceiptV2 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-execution-effect-receipt:v2";

    pub fn in_progress(
        execution_id: impl Into<String>,
        execution_input_snapshot: impl Into<String>,
        attempt_id: impl Into<String>,
        fence_epoch: u64,
    ) -> Self {
        Self {
            schema: Self::SCHEMA.into(),
            execution_id: execution_id.into(),
            execution_input_snapshot: execution_input_snapshot.into(),
            attempt_id: attempt_id.into(),
            fence_epoch,
            state: RecoveryExecutionEffectStateV1::InProgress,
            outcome_digest: None,
        }
    }

    /// Construct an effect receipt from an already-established execution fence.
    pub fn in_progress_for_fence(
        fence: &RecoveryExecutionFenceV1,
    ) -> Option<Self> {
        if !fence.is_well_formed() {
            return None;
        }

        let receipt = Self::in_progress(
            fence.execution_id.clone(),
            fence.execution_input_snapshot.clone(),
            fence.attempt_id.clone(),
            fence.fence_epoch,
        );
        receipt.is_well_formed().then_some(receipt)
    }

    /// Check that this receipt is bound exactly to an established execution fence.
    pub fn matches_fence(&self, fence: &RecoveryExecutionFenceV1) -> bool {
        self.is_well_formed()
            && fence.is_well_formed()
            && self.execution_id == fence.execution_id
            && self.execution_input_snapshot == fence.execution_input_snapshot
            && self.attempt_id == fence.attempt_id
            && self.fence_epoch == fence.fence_epoch
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.execution_id.is_empty()
            && is_sha256_digest(&self.execution_input_snapshot)
            && !self.attempt_id.is_empty()
            && self.fence_epoch > 0
            && match self.state {
                RecoveryExecutionEffectStateV1::InProgress => self.outcome_digest.is_none(),
                RecoveryExecutionEffectStateV1::Succeeded
                | RecoveryExecutionEffectStateV1::Failed => self
                    .outcome_digest
                    .as_deref()
                    .is_some_and(is_sha256_digest),
            }
    }

    /// Derive a terminal success receipt from the canonical execution-result
    /// snapshot rather than accepting an independently supplied outcome digest.
    ///
    /// This is a stronger identity path, not proof that an external side effect
    /// succeeded. The execution result must itself satisfy the core success
    /// predicate, and its canonical digest becomes the receipt outcome identity.
    pub fn succeeded_from_execution(
        fence: &RecoveryExecutionFenceV1,
        execution: &RecoveryExecution,
    ) -> Option<Self> {
        let result = RecoveryExecutionResultSnapshotV1::from_execution(execution);
        if !execution.is_successful()
            || !result.is_well_formed()
            || result.execution_id != fence.execution_id
            || result.input_snapshot != fence.execution_input_snapshot
        {
            return None;
        }

        let receipt = Self {
            schema: Self::SCHEMA.into(),
            execution_id: fence.execution_id.clone(),
            execution_input_snapshot: fence.execution_input_snapshot.clone(),
            attempt_id: fence.attempt_id.clone(),
            fence_epoch: fence.fence_epoch,
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(result.digest()),
        };
        receipt.is_well_formed().then_some(receipt)
    }

    /// Derive a terminal failure receipt from the canonical execution-result
    /// snapshot. The result identity is content-addressed and bound to the
    /// exact execution/fingerprint carried by the established fence.
    pub fn failed_from_execution(
        fence: &RecoveryExecutionFenceV1,
        execution: &RecoveryExecution,
    ) -> Option<Self> {
        let result = RecoveryExecutionResultSnapshotV1::from_execution(execution);
        if !execution.is_failed()
            || !result.is_well_formed()
            || result.execution_id != fence.execution_id
            || result.input_snapshot != fence.execution_input_snapshot
        {
            return None;
        }

        let receipt = Self {
            schema: Self::SCHEMA.into(),
            execution_id: fence.execution_id.clone(),
            execution_input_snapshot: fence.execution_input_snapshot.clone(),
            attempt_id: fence.attempt_id.clone(),
            fence_epoch: fence.fence_epoch,
            state: RecoveryExecutionEffectStateV1::Failed,
            outcome_digest: Some(result.digest()),
        };
        receipt.is_well_formed().then_some(receipt)
    }
}

/// Result of atomically beginning an external effect.
///
/// Fence epochs protect only live InProgress ownership. Terminal Succeeded/Failed
/// receipts are immutable history and remain replayable across later fence epochs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionEffectStartResult {
    Started,
    AlreadyInProgressSameAttempt,
    AlreadyInProgressOtherAttempt,
    AlreadySucceededSameRequest,
    AlreadyFailedSameRequest,
    FingerprintMismatch,
    FenceMismatch,
    MalformedReceipt,
    Indeterminate,
}

/// Result of atomically completing an effect receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionEffectCompletionResult {
    Completed,
    AlreadyCompletedSameOutcome,
    AlreadyCompletedDifferentOutcome,
    NotOwner,
    MissingReceipt,
    FingerprintMismatch,
    FenceMismatch,
    MalformedReceipt,
    Indeterminate,
}

/// Result of atomically transferring an InProgress effect receipt to a newer fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionEffectRecoveryResult {
    Recovered,
    AlreadyRecovered,
    StaleExpectedReceipt,
    MissingReceipt,
    FingerprintMismatch,
    MalformedReceipt,
    Indeterminate,
}

/// Storage-neutral contract for durable external-effect idempotency receipts.
pub trait RecoveryExecutionEffectStore: Send + Sync {
    type Error;

    /// Atomically begin an effect receipt.
    ///
    /// Implementations MUST reject malformed receipts with MalformedReceipt
    /// and MUST NOT mutate state when rejecting them.
    fn begin_effect(
        &self,
        receipt: &RecoveryExecutionEffectReceiptV2,
    ) -> Result<RecoveryExecutionEffectStartResult, Self::Error>;

    /// Atomically record a terminal receipt owned by the exact live generation.
    ///
    /// Implementations MUST reject malformed or argument-mismatched terminal
    /// receipts with MalformedReceipt and MUST NOT mutate state when rejecting.
    fn complete_effect(
        &self,
        execution_id: &str,
        execution_input_snapshot: &str,
        attempt_id: &str,
        fence_epoch: u64,
        completed: &RecoveryExecutionEffectReceiptV2,
    ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error>;

    /// Atomically transfer one InProgress receipt to an exact successor fence.
    ///
    /// Implementations MUST reject malformed or non-monotonic transitions with
    /// MalformedReceipt and MUST NOT mutate state when rejecting them.
    fn recover_effect_if_current(
        &self,
        expected: &RecoveryExecutionEffectReceiptV2,
        successor: &RecoveryExecutionEffectReceiptV2,
    ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error>;

    fn load_effect(
        &self,
        execution_id: &str,
    ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error>;
}

/// Point-in-time reconciliation result for an uncertain effect-start or
/// completion acknowledgement.
///
/// Terminal outcomes are historical observations and therefore remain visible
/// across fence generations; fence comparisons apply only while the receipt is
/// still InProgress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionEffectReconciliationOutcome {
    ObservedInProgressOwnedByThisAttempt,
    ObservedInProgressOwnedByOtherAttempt,
    ObservedSucceeded,
    ObservedFailed,
    ObservedStaleFence,
    ObservedOlderFence,
    ObservedDifferentFingerprint,
    MissingReceipt,
    InvalidReceipt,
}

/// Reconcile an uncertain effect-store result without mutating the store.
pub fn reconcile_execution_effect<S>(
    store: &S,
    execution_id: &str,
    expected_input_snapshot: &str,
    attempt_id: &str,
    expected_fence_epoch: u64,
) -> Result<
    RecoveryExecutionEffectReconciliationOutcome,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    if execution_id.is_empty()
        || !is_sha256_digest(expected_input_snapshot)
        || attempt_id.is_empty()
        || expected_fence_epoch == 0
    {
        return Ok(RecoveryExecutionEffectReconciliationOutcome::InvalidReceipt);
    }

    let Some(current) = store
        .load_effect(execution_id)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?
    else {
        return Ok(RecoveryExecutionEffectReconciliationOutcome::MissingReceipt);
    };

    if !current.is_well_formed() {
        return Ok(RecoveryExecutionEffectReconciliationOutcome::InvalidReceipt);
    }

    if current.execution_input_snapshot != expected_input_snapshot {
        return Ok(
            RecoveryExecutionEffectReconciliationOutcome::ObservedDifferentFingerprint,
        );
    }

    match &current.state {
        RecoveryExecutionEffectStateV1::Succeeded => {
            return Ok(RecoveryExecutionEffectReconciliationOutcome::ObservedSucceeded);
        }
        RecoveryExecutionEffectStateV1::Failed => {
            return Ok(RecoveryExecutionEffectReconciliationOutcome::ObservedFailed);
        }
        RecoveryExecutionEffectStateV1::InProgress => {}
    }

    if current.fence_epoch > expected_fence_epoch {
        return Ok(RecoveryExecutionEffectReconciliationOutcome::ObservedStaleFence);
    }

    if current.fence_epoch < expected_fence_epoch {
        return Ok(RecoveryExecutionEffectReconciliationOutcome::ObservedOlderFence);
    }

    Ok(match (&current.state, current.attempt_id == attempt_id) {
        (RecoveryExecutionEffectStateV1::InProgress, true) => {
            RecoveryExecutionEffectReconciliationOutcome::ObservedInProgressOwnedByThisAttempt
        }
        (RecoveryExecutionEffectStateV1::InProgress, false) => {
            RecoveryExecutionEffectReconciliationOutcome::ObservedInProgressOwnedByOtherAttempt
        }
        (RecoveryExecutionEffectStateV1::Succeeded, _) => {
            RecoveryExecutionEffectReconciliationOutcome::ObservedSucceeded
        }
        (RecoveryExecutionEffectStateV1::Failed, _) => {
            RecoveryExecutionEffectReconciliationOutcome::ObservedFailed
        }
    })
}

/// Atomically transfer one InProgress effect receipt to a newer fence generation.
///
/// The successor must preserve the exact execution identity and input fingerprint,
/// be InProgress, use a different attempt identifier, and advance the fence epoch
/// by exactly one. The backing store must enforce exact-current CAS semantics.
/// This operation only transfers durable receipt ownership; the concrete external
/// resource must itself enforce the same fence token to make stale work harmless.
pub fn recover_execution_effect<S>(
    store: &S,
    expected: &RecoveryExecutionEffectReceiptV2,
    successor_fence: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectRecoveryResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    let valid_successor_epoch = expected.fence_epoch.checked_add(1);
    let Some(successor) =
        RecoveryExecutionEffectReceiptV2::in_progress_for_fence(successor_fence)
    else {
        return Ok(RecoveryExecutionEffectRecoveryResult::MalformedReceipt);
    };

    if !expected.is_well_formed()
        || expected.state != RecoveryExecutionEffectStateV1::InProgress
        || successor.state != RecoveryExecutionEffectStateV1::InProgress
        || !successor.matches_fence(successor_fence)
        || successor.execution_id != expected.execution_id
        || successor.execution_input_snapshot != expected.execution_input_snapshot
        || successor.attempt_id == expected.attempt_id
        || valid_successor_epoch
            .is_none_or(|epoch| successor.fence_epoch != epoch)
    {
        return Ok(RecoveryExecutionEffectRecoveryResult::MalformedReceipt);
    }

    store
        .recover_effect_if_current(expected, &successor)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
}

/// Recover an effect from one established fence into a newer established fence.
///
/// Both fence generations must have been positively accepted by the authoritative
/// fence store. The successor receipt is derived from that established fence,
/// avoiding a caller-created copy of its execution identity, fingerprint, attempt,
/// and epoch.
pub fn recover_execution_effect_for_established_fence<S>(
    store: &S,
    expected: &RecoveryExecutionEffectReceiptV2,
    successor_fence: &EstablishedRecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectRecoveryResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    recover_execution_effect(store, expected, successor_fence.fence())
}

/// Attempt to begin one external execution effect.
pub fn begin_execution_effect<S>(
    store: &S,
    receipt: &RecoveryExecutionEffectReceiptV2,
) -> Result<
    RecoveryExecutionEffectStartResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    if !receipt.is_well_formed() {
        return Ok(RecoveryExecutionEffectStartResult::MalformedReceipt);
    }

    store
        .begin_effect(receipt)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
}

/// Attempt to begin an effect using a single fence record as the source of
/// execution identity, fingerprint, attempt, and fence epoch.
///
/// The stronger established-fence helper below should be preferred when the
/// caller has positively acquired ownership from the authoritative fence store.
pub fn begin_execution_effect_for_fence<S>(
    store: &S,
    fence: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectStartResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    let Some(receipt) = RecoveryExecutionEffectReceiptV2::in_progress_for_fence(fence) else {
        return Ok(RecoveryExecutionEffectStartResult::MalformedReceipt);
    };

    begin_execution_effect(store, &receipt)
}

/// Attempt to begin an effect from a provenance-bearing established fence.
///
/// The wrapped fence cannot be fabricated through the public type constructor,
/// closing the normal-path confusion between fence-shaped data and a generation
/// positively accepted by the authoritative store.
pub fn begin_execution_effect_for_established_fence<S>(
    store: &S,
    fence: &EstablishedRecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectStartResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    begin_execution_effect_for_fence(store, fence.fence())
}

/// Complete an effect with an outcome identity derived from the canonical
/// execution-result snapshot.
///
/// The execution must be successful or failed according to core semantics and
/// must be bound to the exact started receipt identity. This helper records the
/// canonical result identity; it does not itself prove that the external side
/// effect happened.
pub fn complete_execution_effect_from_execution<S>(
    store: &S,
    started: &RecoveryExecutionEffectReceiptV2,
    execution: &RecoveryExecution,
    fence: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectCompletionResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    if !started.matches_fence(fence) {
        return Ok(RecoveryExecutionEffectCompletionResult::MalformedReceipt);
    }

    let completed = if execution.is_successful() {
        RecoveryExecutionEffectReceiptV2::succeeded_from_execution(fence, execution)
    } else if execution.is_failed() {
        RecoveryExecutionEffectReceiptV2::failed_from_execution(fence, execution)
    } else {
        None
    };

    let Some(completed) = completed else {
        return Ok(RecoveryExecutionEffectCompletionResult::MalformedReceipt);
    };

    complete_execution_effect(store, started, &completed)
}

/// Complete an effect from an established fence and the canonical execution
/// result, keeping the strongest provenance-bearing path end-to-end.
///
/// The established fence prevents caller-minted fence-shaped data from being
/// treated as authoritative ownership in the normal completion path.
pub fn complete_execution_effect_for_established_fence<S>(
    store: &S,
    started: &RecoveryExecutionEffectReceiptV2,
    execution: &RecoveryExecution,
    fence: &EstablishedRecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectCompletionResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    complete_execution_effect_from_execution(store, started, execution, fence.fence())
}

/// Record a terminal external-effect outcome under the attempt that owns the
/// InProgress receipt. This does not perform the external effect.
pub fn complete_execution_effect<S>(
    store: &S,
    started: &RecoveryExecutionEffectReceiptV2,
    completed: &RecoveryExecutionEffectReceiptV2,
) -> Result<
    RecoveryExecutionEffectCompletionResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
    if !started.is_well_formed()
        || started.state != RecoveryExecutionEffectStateV1::InProgress
        || !completed.is_well_formed()
        || completed.execution_id != started.execution_id
        || completed.execution_input_snapshot != started.execution_input_snapshot
        || completed.attempt_id != started.attempt_id
        || completed.fence_epoch != started.fence_epoch
        || completed.state == RecoveryExecutionEffectStateV1::InProgress
    {
        return Ok(RecoveryExecutionEffectCompletionResult::MalformedReceipt);
    }

    store
        .complete_effect(
            &started.execution_id,
            &started.execution_input_snapshot,
            &started.attempt_id,
            started.fence_epoch,
            completed,
        )
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
}

/// Validate and persist one transition through the external CAS boundary.
///
/// The helper deliberately performs load/validation/CAS as separate operations:
/// only the store's compare-and-set supplies the concurrency guarantee.
/// Consequently a caller can never infer atomicity from this helper alone.
pub fn persist_consumption_transition<S>(
    store: &S,
    decision: &RecoveryPolicyDecisionSnapshotV1,
    execution: &RecoveryExecution,
    transition: &RecoveryPolicyConsumptionTransitionV1,
    next: &RecoveryPolicyConsumptionSnapshotV1,
) -> Result<
    RecoveryPolicyConsumptionPersistenceOutcome,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryPolicyConsumptionStore,
{
    if !decision.is_well_formed() {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::MalformedDecision);
    }
    if !transition.is_well_formed() {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::MalformedTransition);
    }
    if !next.is_well_formed() {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::MalformedSuccessor);
    }

    let Some(current) = store
        .load(&decision.digest())
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?
    else {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::MissingState);
    };

    if !current.is_well_formed() {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::MalformedStoredState);
    }

    let decision_digest = decision.digest();
    if current.decision_digest != decision_digest {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::InvalidTransition);
    }

    if current.digest() != transition.expected_snapshot_digest {
        return Ok(match current.state {
            RecoveryPolicyConsumptionStateV1::Consumed => {
                RecoveryPolicyConsumptionPersistenceOutcome::ReplayDetected
            }
            RecoveryPolicyConsumptionStateV1::Unconsumed => {
                RecoveryPolicyConsumptionPersistenceOutcome::Conflict
            }
        });
    }

    if !transition.matches(&current, decision, execution, next)
        || transition.next_snapshot_digest != next.digest()
    {
        return Ok(RecoveryPolicyConsumptionPersistenceOutcome::InvalidTransition);
    }

    let cas_result = store
        .compare_and_set(
            &decision_digest,
            &transition.expected_snapshot_digest,
            next,
        )
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?;

    Ok(match cas_result {
        RecoveryPolicyConsumptionCasResult::Committed => {
            RecoveryPolicyConsumptionPersistenceOutcome::Committed
        }
        RecoveryPolicyConsumptionCasResult::NotCurrent => {
            RecoveryPolicyConsumptionPersistenceOutcome::Conflict
        }
        RecoveryPolicyConsumptionCasResult::Indeterminate => {
            RecoveryPolicyConsumptionPersistenceOutcome::CommitIndeterminate
        }
    })
}

/// Durable monotonic ownership epoch for recoverable execution claims.
///
/// This is the stronger recovery-capable companion to RecoveryExecutionClaimV1.
/// A fenced adapter uses the epoch as a capability that must also be enforced by
/// the protected resource: an older epoch is stale after a successful takeover.
/// The epoch is scoped to one execution identity and starts at one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryExecutionFenceV1 {
    pub schema: String,
    pub execution_id: String,
    pub execution_input_snapshot: String,
    pub attempt_id: String,
    pub fence_epoch: u64,
}

impl RecoveryExecutionFenceV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-execution-fence:v1";

    /// Create the first fenced ownership generation for an execution claim.
    pub fn for_initial_claim(claim: &RecoveryExecutionClaimV1) -> Option<Self> {
        if !claim.is_well_formed() {
            return None;
        }

        let fence = Self {
            schema: Self::SCHEMA.into(),
            execution_id: claim.execution_id.clone(),
            execution_input_snapshot: claim.execution_input_snapshot.clone(),
            attempt_id: claim.attempt_id.clone(),
            fence_epoch: 1,
        };

        fence.is_well_formed().then_some(fence)
    }

    /// Create the next ownership generation from an exact current fence.
    ///
    /// Takeover is deliberately a different attempt. A same-attempt replay should
    /// continue using the existing generation rather than manufacturing a new one.
    pub fn for_recovery(current: &Self, new_attempt_id: impl Into<String>) -> Option<Self> {
        if !current.is_well_formed() {
            return None;
        }

        let fence_epoch = current.fence_epoch.checked_add(1)?;
        let successor = Self {
            schema: Self::SCHEMA.into(),
            execution_id: current.execution_id.clone(),
            execution_input_snapshot: current.execution_input_snapshot.clone(),
            attempt_id: new_attempt_id.into(),
            fence_epoch,
        };

        (successor.attempt_id != current.attempt_id && successor.is_well_formed())
            .then_some(successor)
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.execution_id.is_empty()
            && is_sha256_digest(&self.execution_input_snapshot)
            && !self.attempt_id.is_empty()
            && self.fence_epoch > 0
    }
}

/// Result of a fenced execution-ownership operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionFenceResult {
    Acquired,
    AlreadyOwnedSameAttempt,
    AlreadyOwnedOtherAttempt,
    Recovered,
    StaleExpectedFence,
    MissingCurrentFence,
    FingerprintMismatch,
    MalformedFence,
    Indeterminate,
}

/// Provenance-bearing capability produced only after the authoritative fence
/// store positively accepts an initial generation for this exact execution.
///
/// The wrapped fence is private so callers cannot fabricate an "established"
/// handle by merely constructing a structurally valid fence value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstablishedRecoveryExecutionFenceV1 {
    fence: RecoveryExecutionFenceV1,
}

impl EstablishedRecoveryExecutionFenceV1 {
    fn from_authoritative(fence: RecoveryExecutionFenceV1) -> Self {
        Self { fence }
    }

    pub fn execution_id(&self) -> &str {
        &self.fence.execution_id
    }

    pub fn execution_input_snapshot(&self) -> &str {
        &self.fence.execution_input_snapshot
    }

    pub fn attempt_id(&self) -> &str {
        &self.fence.attempt_id
    }

    pub fn fence_epoch(&self) -> u64 {
        self.fence.fence_epoch
    }

    fn fence(&self) -> &RecoveryExecutionFenceV1 {
        &self.fence
    }
}

/// Outcome of the stronger initial-fence acquisition path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryExecutionFenceEstablishmentV1 {
    Established(EstablishedRecoveryExecutionFenceV1),
    Rejected(RecoveryExecutionFenceResult),
}

/// Outcome of recovering an already-established fence into a new generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryExecutionFenceRecoveryEstablishmentV1 {
    Established(EstablishedRecoveryExecutionFenceV1),
    Rejected(RecoveryExecutionFenceResult),
}

/// Storage-neutral contract for monotonic execution fencing.
///
/// recover_if_current MUST be atomic for the same execution identity. A
/// successful recovery replaces exactly the expected current owner with the
/// supplied successor whose epoch is exactly current + 1.
///
/// The store owns the authoritative epoch. A caller must never treat a locally
/// chosen epoch as sufficient proof of ownership.
pub trait RecoveryExecutionFenceStore: Send + Sync {
    type Error;

    /// Atomically establish the initial ownership generation.
    ///
    /// Implementations MUST reject a malformed fence. When no record exists
    /// for the execution identity, only fence epoch 1 may be acquired; a
    /// non-initial epoch must return MalformedFence without mutating state.
    /// An existing record must never be replaced by a different owner merely
    /// because the supplied fence was caller-chosen.
    fn acquire_fence(
        &self,
        fence: &RecoveryExecutionFenceV1,
    ) -> Result<RecoveryExecutionFenceResult, Self::Error>;

    /// Atomically transfer ownership from one exact current generation.
    ///
    /// Implementations MUST reject malformed fences and malformed transitions:
    /// the successor must preserve execution identity and fingerprint, use a
    /// different attempt, and advance the expected epoch by exactly one.
    fn recover_if_current(
        &self,
        expected: &RecoveryExecutionFenceV1,
        successor: &RecoveryExecutionFenceV1,
    ) -> Result<RecoveryExecutionFenceResult, Self::Error>;

    fn load_fence(
        &self,
        execution_id: &str,
    ) -> Result<Option<RecoveryExecutionFenceV1>, Self::Error>;
}

/// Point-in-time observation after reconciling a fenced ownership operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionFenceReconciliationOutcome {
    ObservedCurrentOwnedByThisAttempt,
    ObservedCurrentOwnedByOtherAttempt,
    ObservedStaleFence,
    ObservedOlderFence,
    ObservedDifferentFingerprint,
    MissingFence,
    InvalidFence,
}

/// Reconcile an established fence against the authoritative fence store.
///
/// This is intentionally read-only and point-in-time. Current means the exact
/// established generation is observed in the fence store at this read; it does
/// not reserve that generation for a later effect-store operation.
pub fn reconcile_established_execution_fence<S>(
    store: &S,
    expected: &EstablishedRecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionFenceReconciliationOutcome,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionFenceStore,
{
    reconcile_execution_fence(store, expected.fence())
}

/// Result of the explicit fence-freshness preflight for an effect start.
///
/// RejectedByFence is a negative authoritative observation. AttemptedAfterCurrentFenceObservation
/// means the fence was observed current immediately before the effect-store call; it does not
/// make that separate mutation atomic with the fence read or prove that the fence remained current.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionEffectFenceAdmissionResult {
    RejectedByFence(RecoveryExecutionFenceReconciliationOutcome),
    AttemptedAfterCurrentFenceObservation(RecoveryExecutionEffectStartResult),
}

/// Revalidate a provenance-bearing fence immediately before attempting an effect-store start.
///
/// This closes the admission ambiguity where an established handle could be passed directly
/// to an effect store after authoritative ownership had already advanced. It remains a
/// point-in-time preflight because the fence store and effect store are separate transaction
/// boundaries; a fence can advance after this read and before the effect mutation.
pub fn begin_execution_effect_after_fence_revalidation<F, E>(
    fence_store: &F,
    effect_store: &E,
    fence: &EstablishedRecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionEffectFenceAdmissionResult,
    RecoveryExecutionCrossStorePersistenceError<F::Error, E::Error>,
>
where
    F: RecoveryExecutionFenceStore,
    E: RecoveryExecutionEffectStore,
{
    let observation = reconcile_established_execution_fence(fence_store, fence).map_err(
        |error| match error {
            RecoveryPolicyConsumptionPersistenceError::Store(error) => {
                RecoveryExecutionCrossStorePersistenceError::FenceStore(error)
            }
        },
    )?;

    if observation
        != RecoveryExecutionFenceReconciliationOutcome::ObservedCurrentOwnedByThisAttempt
    {
        return Ok(RecoveryExecutionEffectFenceAdmissionResult::RejectedByFence(
            observation,
        ));
    }

    let result = begin_execution_effect_for_established_fence(effect_store, fence).map_err(
        |error| match error {
            RecoveryPolicyConsumptionPersistenceError::Store(error) => {
                RecoveryExecutionCrossStorePersistenceError::EffectStore(error)
            }
        },
    )?;

    Ok(
        RecoveryExecutionEffectFenceAdmissionResult::AttemptedAfterCurrentFenceObservation(
            result,
        ),
    )
}

/// Reconcile a fenced ownership acknowledgement without mutating the store.
pub fn reconcile_execution_fence<S>(
    store: &S,
    expected: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionFenceReconciliationOutcome,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionFenceStore,
{
    if !expected.is_well_formed() {
        return Ok(RecoveryExecutionFenceReconciliationOutcome::InvalidFence);
    }

    let Some(current) = store
        .load_fence(&expected.execution_id)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?
    else {
        return Ok(RecoveryExecutionFenceReconciliationOutcome::MissingFence);
    };

    if !current.is_well_formed() {
        return Ok(RecoveryExecutionFenceReconciliationOutcome::InvalidFence);
    }

    if current.execution_input_snapshot != expected.execution_input_snapshot {
        return Ok(RecoveryExecutionFenceReconciliationOutcome::ObservedDifferentFingerprint);
    }

    match current.fence_epoch.cmp(&expected.fence_epoch) {
        std::cmp::Ordering::Greater => {
            Ok(RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence)
        }
        std::cmp::Ordering::Less => {
            Ok(RecoveryExecutionFenceReconciliationOutcome::ObservedOlderFence)
        }
        std::cmp::Ordering::Equal if current.attempt_id == expected.attempt_id => {
            Ok(RecoveryExecutionFenceReconciliationOutcome::ObservedCurrentOwnedByThisAttempt)
        }
        std::cmp::Ordering::Equal => {
            Ok(RecoveryExecutionFenceReconciliationOutcome::ObservedCurrentOwnedByOtherAttempt)
        }
    }
}

/// Attempt to acquire the initial fenced generation.
pub fn acquire_execution_fence<S>(
    store: &S,
    fence: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionFenceResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionFenceStore,
{
    if !fence.is_well_formed() || fence.fence_epoch != 1 {
        return Ok(RecoveryExecutionFenceResult::MalformedFence);
    }

    store
        .acquire_fence(fence)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
}

/// Atomically transfer fenced ownership from one exact generation to the next.
pub fn recover_execution_fence<S>(
    store: &S,
    expected: &RecoveryExecutionFenceV1,
    successor: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionFenceResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionFenceStore,
{
    if !expected.is_well_formed()
        || !successor.is_well_formed()
        || successor.execution_id != expected.execution_id
        || successor.execution_input_snapshot != expected.execution_input_snapshot
        || successor.attempt_id == expected.attempt_id
        || expected
            .fence_epoch
            .checked_add(1)
            .is_none_or(|next| successor.fence_epoch != next)
    {
        return Ok(RecoveryExecutionFenceResult::MalformedFence);
    }

    store
        .recover_if_current(expected, successor)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
}

/// Acquire the initial fence and, when the authoritative store confirms the
/// acquisition or exact same-attempt replay, return a provenance-bearing handle.
///
/// Indeterminate does not produce an established handle because the caller has
/// no positive acknowledgement that durable ownership exists.
pub fn establish_execution_fence<S>(
    store: &S,
    fence: &RecoveryExecutionFenceV1,
) -> Result<
    RecoveryExecutionFenceEstablishmentV1,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionFenceStore,
{
    match acquire_execution_fence(store, fence)? {
        RecoveryExecutionFenceResult::Acquired
        | RecoveryExecutionFenceResult::AlreadyOwnedSameAttempt => {
            Ok(RecoveryExecutionFenceEstablishmentV1::Established(
                EstablishedRecoveryExecutionFenceV1::from_authoritative(fence.clone()),
            ))
        }
        outcome => Ok(RecoveryExecutionFenceEstablishmentV1::Rejected(outcome)),
    }
}

/// Recover an already-established fence and, only after a positive recovery
/// acknowledgement, produce the next provenance-bearing generation.
///
/// The successor is derived from the established handle, avoiding another
/// caller-supplied copy of execution identity, fingerprint, and current epoch.
pub fn recover_established_execution_fence<S>(
    store: &S,
    current: &EstablishedRecoveryExecutionFenceV1,
    new_attempt_id: impl Into<String>,
) -> Result<
    RecoveryExecutionFenceRecoveryEstablishmentV1,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionFenceStore,
{
    let Some(successor) =
        RecoveryExecutionFenceV1::for_recovery(current.fence(), new_attempt_id)
    else {
        return Ok(RecoveryExecutionFenceRecoveryEstablishmentV1::Rejected(
            RecoveryExecutionFenceResult::MalformedFence,
        ));
    };

    match recover_execution_fence(store, current.fence(), &successor)? {
        RecoveryExecutionFenceResult::Recovered => Ok(
            RecoveryExecutionFenceRecoveryEstablishmentV1::Established(
                EstablishedRecoveryExecutionFenceV1::from_authoritative(successor),
            ),
        ),
        outcome => Ok(RecoveryExecutionFenceRecoveryEstablishmentV1::Rejected(outcome)),
    }
}

/// Validate the numeric fencing invariant at a protected resource.
///
/// The protected resource, not merely the lock holder, must enforce this check.
/// Current is the only value that permits the supplied operation. A future
/// epoch is not automatically accepted: it indicates that the caller has not
/// established durable ownership for that generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionFenceCheck {
    Invalid,
    Stale,
    Current,
    Future,
}

pub fn check_execution_fence(
    current_epoch: u64,
    supplied_epoch: u64,
) -> RecoveryExecutionFenceCheck {
    if current_epoch == 0 || supplied_epoch == 0 {
        RecoveryExecutionFenceCheck::Invalid
    } else if supplied_epoch < current_epoch {
        RecoveryExecutionFenceCheck::Stale
    } else if supplied_epoch == current_epoch {
        RecoveryExecutionFenceCheck::Current
    } else {
        RecoveryExecutionFenceCheck::Future
    }
}

/// Mutation request presented to a protected external resource.
///
/// The request copies only the identity needed by the protected resource.
/// The authoritative fence must still be enforced by the resource at mutation
/// time; this value is not itself proof that the caller is authenticated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryExecutionProtectedMutationV1 {
    pub execution_id: String,
    pub execution_input_snapshot: String,
    pub attempt_id: String,
    pub fence_epoch: u64,
    /// Optional stable external idempotency key.
    ///
    /// Absence means the adapter must not infer retry safety from fencing alone.
    pub idempotency_key: Option<String>,
}

impl RecoveryExecutionProtectedMutationV1 {
    pub fn from_established_fence(
        fence: &EstablishedRecoveryExecutionFenceV1,
        idempotency_key: Option<String>,
    ) -> Option<Self> {
        if idempotency_key.as_deref().is_some_and(str::is_empty) {
            return None;
        }

        Some(Self {
            execution_id: fence.execution_id().to_owned(),
            execution_input_snapshot: fence.fence().execution_input_snapshot.clone(),
            attempt_id: fence.attempt_id().to_owned(),
            fence_epoch: fence.fence_epoch(),
            idempotency_key,
        })
    }

    /// Returns whether this request carries a usable stable external
    /// idempotency key.
    pub fn has_stable_idempotency_key(&self) -> bool {
        self.idempotency_key.as_deref().is_some_and(|key| !key.is_empty())
    }

    pub fn matches_fence(&self, fence: &EstablishedRecoveryExecutionFenceV1) -> bool {
        self.execution_id == fence.execution_id()
            && self.execution_input_snapshot == fence.fence().execution_input_snapshot
            && self.attempt_id == fence.attempt_id()
            && self.fence_epoch == fence.fence_epoch()
            && !self.execution_id.is_empty()
            && is_sha256_digest(&self.execution_input_snapshot)
            && !self.attempt_id.is_empty()
            && self.fence_epoch > 0
            && self.idempotency_key.as_deref().is_none_or(|key| !key.is_empty())
    }
}

/// Result of a protected-resource mutation that atomically enforces fencing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionProtectedMutationResult {
    Applied,
    AlreadyAppliedSameRequest,
    RejectedStaleFence,
    RejectedFutureFence,
    RejectedInvalidFence,
    RejectedIdentityMismatch,
    RejectedOtherAttempt,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionProtectedMutationValidation {
    Valid,
    Invalid,
}

/// Contract for a resource that can enforce an execution fence at its own
/// mutation boundary.
///
/// `mutate_if_fence_is_current` MUST validate the supplied fence and perform
/// the protected mutation as one resource-local atomic decision. An adapter
/// must not implement this by reading its current epoch in one operation and
/// mutating in a later independent operation.
pub trait RecoveryExecutionFencedResource: Send + Sync {
    type Error;

    fn mutate_if_fence_is_current(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> Result<RecoveryExecutionProtectedMutationResult, Self::Error>;
}

/// Point-in-time reconciliation observation for an indeterminate protected mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionProtectedMutationReconciliationOutcome {
    ObservedAppliedSameRequest,
    ObservedNotApplied,
    ObservedDifferentRequest,
    MissingState,
    InvalidState,
}

/// Optional read-only reconciliation contract for protected resources.
///
/// Reconciliation never retries or mutates the resource. ObservedNotApplied
/// means only that the exact mutation is not observed at this read; it must not
/// be interpreted as proof that a prior indeterminate request never took effect.
pub trait RecoveryExecutionProtectedMutationReconciler: Send + Sync {
    type Error;

    fn reconcile_mutation(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> Result<RecoveryExecutionProtectedMutationReconciliationOutcome, Self::Error>;
}

/// Safe next-action classification for protected-resource recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionProtectedMutationNextActionV1 {
    ReturnRecordedOutcome,
    ReconcileIndeterminateMutation,
    RequireIdempotencyOrManualRecovery,
    RetryWithDeclaredIdempotency,
    ManualRecoveryOnly,
    FailClosed,
}

/// Pure orchestration state for an ambiguous protected-resource mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExecutionProtectedMutationOrchestrationStateV1 {
    AppliedSameRequest,
    Indeterminate,
    ObservedAppliedSameRequest,
    ObservedNotApplied,
    ObservedDifferentRequest,
    MissingState,
    InvalidState,
}

impl RecoveryExecutionProtectedMutationOrchestrationStateV1 {
    pub fn next_action(self) -> RecoveryExecutionProtectedMutationNextActionV1 {
        match self {
            Self::AppliedSameRequest | Self::ObservedAppliedSameRequest => {
                RecoveryExecutionProtectedMutationNextActionV1::ReturnRecordedOutcome
            }
            Self::Indeterminate => {
                RecoveryExecutionProtectedMutationNextActionV1::ReconcileIndeterminateMutation
            }
            Self::ObservedNotApplied => {
                RecoveryExecutionProtectedMutationNextActionV1::RequireIdempotencyOrManualRecovery
            }
            Self::ObservedDifferentRequest | Self::MissingState | Self::InvalidState => {
                RecoveryExecutionProtectedMutationNextActionV1::FailClosed
            }
        }
    }
}

/// Capability of an external resource to enforce fencing at its mutation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryExecutionFencingCapabilityV1 {
    EnforcedAtMutationBoundary,
    NotSupported,
    Unknown,
}

/// Capability of an external effect to make repeated identical requests safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryExecutionIdempotencyCapabilityV1 {
    StableKey,
    TransactionallyCoupled,
    NotSupported,
    Unknown,
}

/// Read-back capability after an uncertain external-resource mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryExecutionReconciliationCapabilityV1 {
    StrongReadBack,
    EventuallyConsistentReadBack,
    NotSupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryExecutionEffectRecoveryModeV1 {
    AutomaticTakeover,
    AutomaticRetryOnly,
    ManualRecoveryOnly,
}

/// Explicit external-effect safety profile.
///
/// Unknown capabilities never increase the derived recovery permission.
/// The profile describes adapter guarantees; it does not establish
/// authentication, proof-of-possession, exactly-once execution, or truth of
/// the external resource state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryExecutionEffectSafetyProfileV1 {
    pub schema: String,
    pub fencing: RecoveryExecutionFencingCapabilityV1,
    pub idempotency: RecoveryExecutionIdempotencyCapabilityV1,
    pub reconciliation: RecoveryExecutionReconciliationCapabilityV1,
    pub claim_ceiling: String,
}

impl RecoveryExecutionEffectSafetyProfileV1 {
    pub const SCHEMA: &'static str =
        "sol-atlas:recovery-execution-effect-safety-profile:v1";

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA && !self.claim_ceiling.is_empty()
    }

    /// Canonical content binding for provenance and drift detection.
    pub fn digest(&self) -> String {
        let fencing = match self.fencing {
            RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary => "enforced",
            RecoveryExecutionFencingCapabilityV1::NotSupported => "unsupported",
            RecoveryExecutionFencingCapabilityV1::Unknown => "unknown",
        };
        let idempotency = match self.idempotency {
            RecoveryExecutionIdempotencyCapabilityV1::StableKey => "stable-key",
            RecoveryExecutionIdempotencyCapabilityV1::TransactionallyCoupled => {
                "transactionally-coupled"
            }
            RecoveryExecutionIdempotencyCapabilityV1::NotSupported => "unsupported",
            RecoveryExecutionIdempotencyCapabilityV1::Unknown => "unknown",
        };
        let reconciliation = match self.reconciliation {
            RecoveryExecutionReconciliationCapabilityV1::StrongReadBack => "strong-read-back",
            RecoveryExecutionReconciliationCapabilityV1::EventuallyConsistentReadBack => {
                "eventually-consistent-read-back"
            }
            RecoveryExecutionReconciliationCapabilityV1::NotSupported => "unsupported",
            RecoveryExecutionReconciliationCapabilityV1::Unknown => "unknown",
        };
        let material = format!(
            "schema={}:{}|fencing={}|idempotency={}|reconciliation={}|claim={}:{}",
            self.schema.len(),
            self.schema,
            fencing,
            idempotency,
            reconciliation,
            self.claim_ceiling.len(),
            self.claim_ceiling
        );
        let mut hasher = Sha256::new();
        hasher.update(material.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }

    pub fn automatic_retry_safe(&self) -> bool {
        self.is_well_formed()
            && matches!(
                self.idempotency,
                RecoveryExecutionIdempotencyCapabilityV1::StableKey
                    | RecoveryExecutionIdempotencyCapabilityV1::TransactionallyCoupled
            )
    }

    pub fn automatic_takeover_safe(&self) -> bool {
        self.automatic_retry_safe()
            && matches!(
                self.fencing,
                RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary
            )
    }

    pub fn recovery_mode(&self) -> RecoveryExecutionEffectRecoveryModeV1 {
        if self.automatic_takeover_safe() {
            RecoveryExecutionEffectRecoveryModeV1::AutomaticTakeover
        } else if self.automatic_retry_safe() {
            RecoveryExecutionEffectRecoveryModeV1::AutomaticRetryOnly
        } else {
            RecoveryExecutionEffectRecoveryModeV1::ManualRecoveryOnly
        }
    }

    /// Apply declared adapter capabilities to an observed mutation state.
    ///
    /// This is pure policy classification. It never performs reconciliation,
    /// retries, takeover, or the external mutation itself.
    pub fn automatic_retry_safe_for(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> bool {
        self.automatic_retry_safe()
            && match self.idempotency {
                RecoveryExecutionIdempotencyCapabilityV1::StableKey => {
                    mutation.has_stable_idempotency_key()
                }
                RecoveryExecutionIdempotencyCapabilityV1::TransactionallyCoupled => true,
                RecoveryExecutionIdempotencyCapabilityV1::NotSupported
                | RecoveryExecutionIdempotencyCapabilityV1::Unknown => false,
            }
    }

    pub fn automatic_takeover_safe_for(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> bool {
        self.automatic_takeover_safe()
            && self.automatic_retry_safe_for(mutation)
    }

    pub fn next_action(
        &self,
        state: RecoveryExecutionProtectedMutationOrchestrationStateV1,
    ) -> RecoveryExecutionProtectedMutationNextActionV1 {
        if !self.is_well_formed() {
            return RecoveryExecutionProtectedMutationNextActionV1::FailClosed;
        }

        match state {
            RecoveryExecutionProtectedMutationOrchestrationStateV1::AppliedSameRequest
            | RecoveryExecutionProtectedMutationOrchestrationStateV1::
                ObservedAppliedSameRequest => {
                RecoveryExecutionProtectedMutationNextActionV1::ReturnRecordedOutcome
            }
            RecoveryExecutionProtectedMutationOrchestrationStateV1::Indeterminate => {
                match self.reconciliation {
                    RecoveryExecutionReconciliationCapabilityV1::StrongReadBack
                    | RecoveryExecutionReconciliationCapabilityV1::EventuallyConsistentReadBack => {
                        RecoveryExecutionProtectedMutationNextActionV1::
                            ReconcileIndeterminateMutation
                    }
                    RecoveryExecutionReconciliationCapabilityV1::NotSupported
                    | RecoveryExecutionReconciliationCapabilityV1::Unknown => {
                        RecoveryExecutionProtectedMutationNextActionV1::ManualRecoveryOnly
                    }
                }
            }
            RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied => {
                match self.idempotency {
                    RecoveryExecutionIdempotencyCapabilityV1::TransactionallyCoupled => {
                        RecoveryExecutionProtectedMutationNextActionV1::RetryWithDeclaredIdempotency
                    }
                    RecoveryExecutionIdempotencyCapabilityV1::StableKey
                    | RecoveryExecutionIdempotencyCapabilityV1::NotSupported
                    | RecoveryExecutionIdempotencyCapabilityV1::Unknown => {
                        RecoveryExecutionProtectedMutationNextActionV1::
                            RequireIdempotencyOrManualRecovery
                    }
                }
            }
            RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedDifferentRequest
            | RecoveryExecutionProtectedMutationOrchestrationStateV1::MissingState
            | RecoveryExecutionProtectedMutationOrchestrationStateV1::InvalidState => {
                RecoveryExecutionProtectedMutationNextActionV1::FailClosed
            }
        }
    }

    /// Apply the safety profile to the exact mutation being considered.
    pub fn next_action_for_mutation(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
        state: RecoveryExecutionProtectedMutationOrchestrationStateV1,
    ) -> RecoveryExecutionProtectedMutationNextActionV1 {
        if !self.is_well_formed()
            || mutation.execution_id.is_empty()
            || !is_sha256_digest(&mutation.execution_input_snapshot)
            || mutation.attempt_id.is_empty()
            || mutation.fence_epoch == 0
            || mutation.idempotency_key.as_deref().is_some_and(str::is_empty)
        {
            return RecoveryExecutionProtectedMutationNextActionV1::FailClosed;
        }

        match state {
            RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied => {
                if self.automatic_retry_safe_for(mutation) {
                    RecoveryExecutionProtectedMutationNextActionV1::RetryWithDeclaredIdempotency
                } else {
                    RecoveryExecutionProtectedMutationNextActionV1::
                        RequireIdempotencyOrManualRecovery
                }
            }
            _ => self.next_action(state),
        }
    }

    /// Qualify an adapter capability declaration against executable evidence.
    pub fn supported_by_conformance_evidence(
        &self,
        evidence: &RecoveryExecutionEffectConformanceEvidenceV1,
    ) -> bool {
        self.is_well_formed()
            && match self.fencing {
                RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary => {
                    evidence.current_fence_accepted
                        && evidence.stale_fence_rejected
                        && evidence.future_fence_rejected
                        && evidence.concurrent_fencing_preserved
                }
                RecoveryExecutionFencingCapabilityV1::NotSupported => true,
                RecoveryExecutionFencingCapabilityV1::Unknown => false,
            }
            && match self.idempotency {
                RecoveryExecutionIdempotencyCapabilityV1::StableKey => {
                    evidence.stable_key_replay_safe
                        && evidence.different_request_same_key_rejected
                        && evidence.changed_idempotency_key_rejected
                }
                RecoveryExecutionIdempotencyCapabilityV1::TransactionallyCoupled => {
                    evidence.transactionally_coupled_retry_safe
                }
                RecoveryExecutionIdempotencyCapabilityV1::NotSupported => true,
                RecoveryExecutionIdempotencyCapabilityV1::Unknown => false,
            }
            && match self.reconciliation {
                RecoveryExecutionReconciliationCapabilityV1::StrongReadBack => {
                    evidence.exact_reconciliation
                        && evidence.point_in_time_semantics_explicit
                        && evidence.strong_read_back_verified
                }
                RecoveryExecutionReconciliationCapabilityV1::EventuallyConsistentReadBack => {
                    evidence.exact_reconciliation
                        && evidence.point_in_time_semantics_explicit
                        && evidence.eventually_consistent_read_back_verified
                }
                RecoveryExecutionReconciliationCapabilityV1::NotSupported => true,
                RecoveryExecutionReconciliationCapabilityV1::Unknown => false,
            }
    }

}

/// Runtime evidence emitted by an adapter-specific protected-effect conformance suite.
///
/// The evidence records concrete behaviors exercised against the actual adapter.
/// A capability declaration cannot satisfy this contract by referring to itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryExecutionEffectConformanceEvidenceV1 {
    pub current_fence_accepted: bool,
    pub stale_fence_rejected: bool,
    pub future_fence_rejected: bool,
    pub concurrent_fencing_preserved: bool,
    pub stable_key_replay_safe: bool,
    pub different_request_same_key_rejected: bool,
    pub changed_idempotency_key_rejected: bool,
    pub transactionally_coupled_retry_safe: bool,
    pub exact_reconciliation: bool,
    pub point_in_time_semantics_explicit: bool,
    pub strong_read_back_verified: bool,
    pub eventually_consistent_read_back_verified: bool,
}

impl RecoveryExecutionEffectConformanceEvidenceV1 {
    /// Content digest for the concrete behavioral evidence.
    pub fn digest(&self) -> String {
        let flags = [
            self.current_fence_accepted,
            self.stale_fence_rejected,
            self.future_fence_rejected,
            self.concurrent_fencing_preserved,
            self.stable_key_replay_safe,
            self.different_request_same_key_rejected,
            self.changed_idempotency_key_rejected,
            self.transactionally_coupled_retry_safe,
            self.exact_reconciliation,
            self.point_in_time_semantics_explicit,
            self.strong_read_back_verified,
            self.eventually_consistent_read_back_verified,
        ];
        let material: String = flags
            .into_iter()
            .map(|flag| if flag { '1' } else { '0' })
            .collect();
        let mut hasher = Sha256::new();
        hasher.update(material.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

/// Provenance-bearing conformance artifact.
///
/// The artifact binds the observed evidence to one exact safety profile, the
/// adapter revision that was exercised, and the harness revision that produced
/// the evidence. These bindings detect drift; they are not authentication,
/// proof-of-possession, or proof that an external service is truthful.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryExecutionEffectConformanceReportV1 {
    pub schema: String,
    pub adapter_id: String,
    pub adapter_revision: String,
    pub harness_id: String,
    pub harness_revision: String,
    pub profile_digest: String,
    pub evidence_digest: String,
    pub report_digest: String,
    pub evidence: RecoveryExecutionEffectConformanceEvidenceV1,
}

impl RecoveryExecutionEffectConformanceReportV1 {
    pub const SCHEMA: &'static str =
        "sol-atlas:recovery-execution-effect-conformance-report:v1";

    fn identity_material(&self) -> String {
        format!(
            concat!(
                "schema={}:{}|adapter={}:{}|adapter-revision={}:{}|",
                "harness={}:{}|harness-revision={}:{}|",
                "profile={}|evidence={}|evidence-body={}"
            ),
            self.schema.len(),
            self.schema,
            self.adapter_id.len(),
            self.adapter_id,
            self.adapter_revision.len(),
            self.adapter_revision,
            self.harness_id.len(),
            self.harness_id,
            self.harness_revision.len(),
            self.harness_revision,
            self.profile_digest,
            self.evidence_digest,
            self.evidence.digest(),
        )
    }
    pub fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.identity_material().as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }

    pub fn from_parts(
        adapter_id: impl Into<String>,
        adapter_revision: impl Into<String>,
        harness_id: impl Into<String>,
        harness_revision: impl Into<String>,
        profile: &RecoveryExecutionEffectSafetyProfileV1,
        evidence: RecoveryExecutionEffectConformanceEvidenceV1,
    ) -> Option<Self> {
        if !profile.is_well_formed() {
            return None;
        }
        let adapter_id = adapter_id.into();
        let adapter_revision = adapter_revision.into();
        let harness_id = harness_id.into();
        let harness_revision = harness_revision.into();
        if adapter_id.is_empty()
            || adapter_revision.is_empty()
            || harness_id.is_empty()
            || harness_revision.is_empty()
        {
            return None;
        }
        let mut report = Self {
            schema: Self::SCHEMA.into(),
            adapter_id,
            adapter_revision,
            harness_id,
            harness_revision,
            profile_digest: profile.digest(),
            evidence_digest: evidence.digest(),
            report_digest: String::new(),
            evidence,
        };
        report.report_digest = report.digest();
        Some(report)
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.adapter_id.is_empty()
            && !self.adapter_revision.is_empty()
            && !self.harness_id.is_empty()
            && !self.harness_revision.is_empty()
            && is_sha256_digest(&self.profile_digest)
            && is_sha256_digest(&self.evidence_digest)
            && is_sha256_digest(&self.report_digest)
            && self.evidence_digest == self.evidence.digest()
            && self.report_digest == self.digest()
    }

    /// Qualify the report against the exact profile it claims to exercise.
    ///
    /// The profile digest prevents evidence from another capability declaration
    /// from being silently reused after the profile changes.
    pub fn supports_profile(&self, profile: &RecoveryExecutionEffectSafetyProfileV1) -> bool {
        self.is_well_formed()
            && profile.is_well_formed()
            && self.profile_digest == profile.digest()
            && profile.supported_by_conformance_evidence(&self.evidence)
    }
}

/// Validate a protected mutation request against the established fence before
/// crossing the resource boundary.
pub fn validate_protected_mutation_request(
    mutation: &RecoveryExecutionProtectedMutationV1,
    fence: &EstablishedRecoveryExecutionFenceV1,
) -> RecoveryExecutionProtectedMutationValidation {
    if mutation.matches_fence(fence) {
        RecoveryExecutionProtectedMutationValidation::Valid
    } else {
        RecoveryExecutionProtectedMutationValidation::Invalid
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sol_atlas_core::{
        CapabilityId, CapabilityState, RecoveryPolicyDecisionV1, RecoveryExecution,
    };
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn protected_resource_fence_check_is_fail_closed_for_invalid_and_future_epochs() {
        assert_eq!(
            check_execution_fence(0, 1),
            RecoveryExecutionFenceCheck::Invalid
        );
        assert_eq!(
            check_execution_fence(2, 0),
            RecoveryExecutionFenceCheck::Invalid
        );
        assert_eq!(
            check_execution_fence(2, 3),
            RecoveryExecutionFenceCheck::Future
        );
    }

    #[test]
    fn protected_resource_fence_check_accepts_only_the_current_epoch() {
        assert_eq!(
            check_execution_fence(2, 1),
            RecoveryExecutionFenceCheck::Stale
        );
        assert_eq!(
            check_execution_fence(2, 2),
            RecoveryExecutionFenceCheck::Current
        );
        assert_ne!(
            check_execution_fence(2, 1),
            RecoveryExecutionFenceCheck::Current
        );
        assert_ne!(
            check_execution_fence(2, 3),
            RecoveryExecutionFenceCheck::Current
        );
    }

    #[derive(Default)]
    struct FencedResourceMemoryStore {
        state: Mutex<FencedResourceMemoryState>,
    }

    #[derive(Default)]
    struct FencedResourceMemoryState {
        current_epoch: u64,
        applied: BTreeMap<String, (String, String, u64, Option<String>)>,
    }

    impl RecoveryExecutionProtectedMutationReconciler for FencedResourceMemoryStore {
        type Error = &'static str;

        fn reconcile_mutation(
            &self,
            mutation: &RecoveryExecutionProtectedMutationV1,
        ) -> Result<
            RecoveryExecutionProtectedMutationReconciliationOutcome,
            Self::Error,
        > {
            if mutation.execution_id.is_empty()
                || !is_sha256_digest(&mutation.execution_input_snapshot)
                || mutation.attempt_id.is_empty()
                || mutation.fence_epoch == 0
                || mutation.idempotency_key.as_deref().is_some_and(str::is_empty)
            {
                return Ok(
                    RecoveryExecutionProtectedMutationReconciliationOutcome::InvalidState,
                );
            }

            let state = self.state.lock().map_err(|_| "poisoned")?;
            let Some((fingerprint, attempt_id, epoch, idempotency_key)) =
                state.applied.get(&mutation.execution_id)
            else {
                return Ok(
                    RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedNotApplied,
                );
            };

            if fingerprint == &mutation.execution_input_snapshot
                && attempt_id == &mutation.attempt_id
                && *epoch == mutation.fence_epoch
                && idempotency_key == &mutation.idempotency_key
            {
                return Ok(
                    RecoveryExecutionProtectedMutationReconciliationOutcome::
                        ObservedAppliedSameRequest,
                );
            }

            Ok(
                RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedDifferentRequest,
            )
        }
    }

    impl FencedResourceMemoryStore {
        fn set_epoch(&self, epoch: u64) {
            self.state.lock().expect("resource state lock").current_epoch = epoch;
        }
    }

    impl RecoveryExecutionFencedResource for FencedResourceMemoryStore {
        type Error = &'static str;

        fn mutate_if_fence_is_current(
            &self,
            mutation: &RecoveryExecutionProtectedMutationV1,
        ) -> Result<RecoveryExecutionProtectedMutationResult, Self::Error> {
            if mutation.execution_id.is_empty()
                || !is_sha256_digest(&mutation.execution_input_snapshot)
                || mutation.attempt_id.is_empty()
                || mutation.fence_epoch == 0
            {
                return Ok(RecoveryExecutionProtectedMutationResult::RejectedInvalidFence);
            }

            let mut state = self.state.lock().map_err(|_| "poisoned")?;
            let current_epoch = state.current_epoch;
            if mutation.fence_epoch < current_epoch {
                return Ok(RecoveryExecutionProtectedMutationResult::RejectedStaleFence);
            }
            if mutation.fence_epoch > current_epoch {
                return Ok(RecoveryExecutionProtectedMutationResult::RejectedFutureFence);
            }

            let key = mutation.execution_id.clone();
            if let Some((fingerprint, attempt_id, epoch, idempotency_key)) =
                state.applied.get(&key)
            {
                if fingerprint != &mutation.execution_input_snapshot {
                    return Ok(
                        RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch,
                    );
                }
                if *epoch == mutation.fence_epoch && attempt_id == &mutation.attempt_id {
                    if idempotency_key != &mutation.idempotency_key {
                        return Ok(
                            RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch,
                        );
                    }
                    return Ok(
                        RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest,
                    );
                }
                if *epoch == mutation.fence_epoch {
                    return Ok(RecoveryExecutionProtectedMutationResult::RejectedOtherAttempt);
                }
                return Ok(RecoveryExecutionProtectedMutationResult::RejectedStaleFence);
            }

            state.applied.insert(
                key,
                (
                    mutation.execution_input_snapshot.clone(),
                    mutation.attempt_id.clone(),
                    mutation.fence_epoch,
                    mutation.idempotency_key.clone(),
                ),
            );
            Ok(RecoveryExecutionProtectedMutationResult::Applied)
        }
    }

    #[test]
    fn safety_profile_digest_changes_on_profile_drift() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter-specific evidence required.".into(),
        };
        let original = profile.digest();
        let mut drifted = profile.clone();
        drifted.claim_ceiling = "Different claim ceiling.".into();

        assert_ne!(original, drifted.digest());
    }

    #[test]
    fn conformance_report_binds_exact_profile_and_provenance() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter-specific evidence required.".into(),
        };
        let evidence = RecoveryExecutionEffectConformanceEvidenceV1 {
            current_fence_accepted: true,
            stale_fence_rejected: true,
            future_fence_rejected: true,
            concurrent_fencing_preserved: true,
            stable_key_replay_safe: true,
            different_request_same_key_rejected: true,
            changed_idempotency_key_rejected: true,
            transactionally_coupled_retry_safe: false,
            exact_reconciliation: true,
            point_in_time_semantics_explicit: true,
            strong_read_back_verified: true,
            eventually_consistent_read_back_verified: false,
        };
        let report = RecoveryExecutionEffectConformanceReportV1::from_parts(
            "reference-resource",
            "adapter-revision-1",
            "sol-atlas-protected-effect-conformance",
            "harness-revision-1",
            &profile,
            evidence,
        )
        .expect("report");
        assert!(report.is_well_formed());
        assert!(report.supports_profile(&profile));

        let mut drifted = profile.clone();
        drifted.claim_ceiling = "Changed profile.".into();
        assert!(!report.supports_profile(&drifted));

        let mut altered_revision = report.clone();
        altered_revision.adapter_revision = "adapter-revision-2".into();
        assert!(!altered_revision.is_well_formed());
        assert_ne!(report.digest(), altered_revision.digest());

        let mut altered_harness = report.clone();
        altered_harness.harness_revision = "harness-revision-2".into();
        assert!(!altered_harness.is_well_formed());
        assert_ne!(report.digest(), altered_harness.digest());

        let mut altered_evidence = report.clone();
        altered_evidence.evidence.current_fence_accepted = false;
        assert!(!altered_evidence.is_well_formed());
    }

    #[test]
    fn conformance_report_requires_complete_provenance_fields() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::NotSupported,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::NotSupported,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::NotSupported,
            claim_ceiling: "No automatic external recovery.".into(),
        };
        let evidence = RecoveryExecutionEffectConformanceEvidenceV1 {
            current_fence_accepted: false,
            stale_fence_rejected: false,
            future_fence_rejected: false,
            concurrent_fencing_preserved: false,
            stable_key_replay_safe: false,
            different_request_same_key_rejected: false,
            changed_idempotency_key_rejected: false,
            transactionally_coupled_retry_safe: false,
            exact_reconciliation: false,
            point_in_time_semantics_explicit: false,
            strong_read_back_verified: false,
            eventually_consistent_read_back_verified: false,
        };

        assert!(
            RecoveryExecutionEffectConformanceReportV1::from_parts(
                "",
                "adapter-revision",
                "harness",
                "harness-revision",
                &profile,
                evidence,
            )
            .is_none()
        );
        assert!(
            RecoveryExecutionEffectConformanceReportV1::from_parts(
                "adapter",
                "",
                "harness",
                "harness-revision",
                &profile,
                evidence,
            )
            .is_none()
        );
        assert!(
            RecoveryExecutionEffectConformanceReportV1::from_parts(
                "adapter",
                "adapter-revision",
                "",
                "harness-revision",
                &profile,
                evidence,
            )
            .is_none()
        );
        assert!(
            RecoveryExecutionEffectConformanceReportV1::from_parts(
                "adapter",
                "adapter-revision",
                "harness",
                "",
                &profile,
                evidence,
            )
            .is_none()
        );
    }

    #[test]
    fn external_effect_safety_profile_requires_request_key_for_stable_key_retry() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-profile-key".into(),
            execution_input_snapshot:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            attempt_id: "attempt-profile-key".into(),
            fence_epoch: 2,
            idempotency_key: None,
        };

        assert_eq!(
            profile.next_action(
                RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::RequireIdempotencyOrManualRecovery,
        );
        assert_eq!(
            profile.next_action_for_mutation(
                &mutation,
                RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::RequireIdempotencyOrManualRecovery,
        );

        let keyed = RecoveryExecutionProtectedMutationV1 {
            idempotency_key: Some("stable-effect-key".into()),
            ..mutation
        };
        assert_eq!(
            profile.next_action_for_mutation(
                &keyed,
                RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::RetryWithDeclaredIdempotency,
        );
        assert!(profile.automatic_retry_safe_for(&keyed));
        assert!(profile.automatic_takeover_safe_for(&keyed));
    }

    #[test]
    fn transactionally_coupled_retry_does_not_require_a_key() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::TransactionallyCoupled,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-transactional".into(),
            execution_input_snapshot:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            attempt_id: "attempt-transactional".into(),
            fence_epoch: 2,
            idempotency_key: None,
        };

        assert!(profile.automatic_retry_safe_for(&mutation));
        assert!(profile.automatic_takeover_safe_for(&mutation));
        assert_eq!(
            profile.next_action_for_mutation(
                &mutation,
                RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::RetryWithDeclaredIdempotency,
        );
    }

    #[test]
    fn external_effect_profile_requires_executable_conformance_evidence() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter-specific evidence required.".into(),
        };
        let complete = RecoveryExecutionEffectConformanceEvidenceV1 {
            current_fence_accepted: true,
            stale_fence_rejected: true,
            future_fence_rejected: true,
            concurrent_fencing_preserved: true,
            stable_key_replay_safe: true,
            different_request_same_key_rejected: true,
            changed_idempotency_key_rejected: true,
            transactionally_coupled_retry_safe: false,
            exact_reconciliation: true,
            point_in_time_semantics_explicit: true,
            strong_read_back_verified: true,
            eventually_consistent_read_back_verified: false,
        };
        assert!(profile.supported_by_conformance_evidence(&complete));

        let missing_stale = RecoveryExecutionEffectConformanceEvidenceV1 {
            stale_fence_rejected: false,
            ..complete
        };
        assert!(!profile.supported_by_conformance_evidence(&missing_stale));

        let missing_key_replay = RecoveryExecutionEffectConformanceEvidenceV1 {
            stable_key_replay_safe: false,
            ..complete
        };
        assert!(!profile.supported_by_conformance_evidence(&missing_key_replay));

        let missing_readback = RecoveryExecutionEffectConformanceEvidenceV1 {
            exact_reconciliation: false,
            ..complete
        };
        assert!(!profile.supported_by_conformance_evidence(&missing_readback));

        let missing_point_in_time = RecoveryExecutionEffectConformanceEvidenceV1 {
            point_in_time_semantics_explicit: false,
            ..complete
        };
        assert!(!profile.supported_by_conformance_evidence(
            &missing_point_in_time
        ));
    }

    #[test]
    fn reference_resource_produces_conformance_evidence_for_its_profile() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);

        let current = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-conformance-current".into(),
            execution_input_snapshot:
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
            attempt_id: "attempt-conformance".into(),
            fence_epoch: 2,
            idempotency_key: Some("conformance-key".into()),
        };
        let stale = RecoveryExecutionProtectedMutationV1 {
            fence_epoch: 1,
            ..current.clone()
        };
        let future = RecoveryExecutionProtectedMutationV1 {
            fence_epoch: 3,
            ..current.clone()
        };
        let changed_same_key = RecoveryExecutionProtectedMutationV1 {
            execution_input_snapshot:
                "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                    .into(),
            ..current.clone()
        };
        let changed_idempotency_key = RecoveryExecutionProtectedMutationV1 {
            idempotency_key: Some("different-key".into()),
            ..current.clone()
        };

        let current_fence_accepted = resource
            .mutate_if_fence_is_current(&current)
            .expect("current mutation")
            == RecoveryExecutionProtectedMutationResult::Applied;
        let stable_key_replay_safe = resource
            .mutate_if_fence_is_current(&current)
            .expect("same-key replay")
            == RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest;
        let stale_fence_rejected = resource
            .mutate_if_fence_is_current(&stale)
            .expect("stale mutation")
            == RecoveryExecutionProtectedMutationResult::RejectedStaleFence;
        let future_fence_rejected = resource
            .mutate_if_fence_is_current(&future)
            .expect("future mutation")
            == RecoveryExecutionProtectedMutationResult::RejectedFutureFence;
        let different_request_same_key_rejected = resource
            .mutate_if_fence_is_current(&changed_same_key)
            .expect("different request with same key")
            == RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch;
        let changed_idempotency_key_rejected = resource
            .mutate_if_fence_is_current(&changed_idempotency_key)
            .expect("changed idempotency key")
            == RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch;
        let exact_reconciliation = resource
            .reconcile_mutation(&current)
            .expect("exact reconciliation")
            == RecoveryExecutionProtectedMutationReconciliationOutcome::
                ObservedAppliedSameRequest;

        let concurrent = Arc::new(FencedResourceMemoryStore::default());
        concurrent.set_epoch(2);
        let barrier = Arc::new(Barrier::new(32));
        let stale = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-conformance-concurrent-stale".into(),
            execution_input_snapshot:
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
            attempt_id: "attempt-stale".into(),
            fence_epoch: 1,
            idempotency_key: None,
        };
        let current_concurrent = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-conformance-concurrent-current".into(),
            execution_input_snapshot:
                "sha256:abababababababababababababababababababababababababababababababab"
                    .into(),
            attempt_id: "attempt-current".into(),
            fence_epoch: 2,
            idempotency_key: None,
        };
        let mut joins = Vec::new();
        for index in 0..32 {
            let store = Arc::clone(&concurrent);
            let gate = Arc::clone(&barrier);
            let request = if index == 0 {
                current_concurrent.clone()
            } else {
                stale.clone()
            };
            joins.push(thread::spawn(move || {
                gate.wait();
                store.mutate_if_fence_is_current(&request)
            }));
        }
        let mut stale_rejected_count = 0usize;
        let mut current_applied_count = 0usize;
        for join in joins {
            match join.expect("concurrent thread").expect("concurrent result") {
                RecoveryExecutionProtectedMutationResult::RejectedStaleFence => {
                    stale_rejected_count += 1;
                }
                RecoveryExecutionProtectedMutationResult::Applied => {
                    current_applied_count += 1;
                }
                RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest => {}
                other => panic!("unexpected concurrent outcome: {other:?}"),
            }
        }
        let concurrent_fencing_preserved =
            current_applied_count == 1 && stale_rejected_count == 31;

        let evidence = RecoveryExecutionEffectConformanceEvidenceV1 {
            current_fence_accepted,
            stale_fence_rejected,
            future_fence_rejected,
            concurrent_fencing_preserved,
            stable_key_replay_safe,
            different_request_same_key_rejected,
            changed_idempotency_key_rejected,
            transactionally_coupled_retry_safe: false,
            exact_reconciliation,
            point_in_time_semantics_explicit: true,
            strong_read_back_verified: exact_reconciliation,
            eventually_consistent_read_back_verified: false,
        };
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Reference adapter evidence only.".into(),
        };

        assert!(profile.supported_by_conformance_evidence(&evidence));
    }

    #[test]
    fn external_effect_safety_profile_wires_capabilities_into_actions() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };

        assert_eq!(
            profile.next_action(
                RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::RequireIdempotencyOrManualRecovery,
        );

        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-profile-action".into(),
            execution_input_snapshot:
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
            attempt_id: "attempt-profile-action".into(),
            fence_epoch: 2,
            idempotency_key: Some("stable-profile-action".into()),
        };
        assert_eq!(
            profile.next_action_for_mutation(
                &mutation,
                RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::RetryWithDeclaredIdempotency,
        );
        assert_eq!(
            profile.next_action(
                RecoveryExecutionProtectedMutationOrchestrationStateV1::Indeterminate,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::ReconcileIndeterminateMutation,
        );

        let no_reconciliation = RecoveryExecutionEffectSafetyProfileV1 {
            reconciliation:
                RecoveryExecutionReconciliationCapabilityV1::NotSupported,
            ..profile
        };
        assert_eq!(
            no_reconciliation.next_action(
                RecoveryExecutionProtectedMutationOrchestrationStateV1::Indeterminate,
            ),
            RecoveryExecutionProtectedMutationNextActionV1::ManualRecoveryOnly,
        );
    }

    #[test]
    fn external_effect_safety_profile_derives_conservative_recovery_mode() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };

        assert!(profile.is_well_formed());
        assert!(profile.automatic_retry_safe());
        assert!(profile.automatic_takeover_safe());
        assert_eq!(
            profile.recovery_mode(),
            RecoveryExecutionEffectRecoveryModeV1::AutomaticTakeover,
        );
    }

    #[test]
    fn external_effect_safety_profile_without_fencing_allows_retry_but_not_takeover() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::NotSupported,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };

        assert!(profile.automatic_retry_safe());
        assert!(!profile.automatic_takeover_safe());
        assert_eq!(
            profile.recovery_mode(),
            RecoveryExecutionEffectRecoveryModeV1::AutomaticRetryOnly,
        );
    }

    #[test]
    fn external_effect_safety_profile_without_idempotency_fails_closed() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::NotSupported,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };

        assert!(!profile.automatic_retry_safe());
        assert!(!profile.automatic_takeover_safe());
        assert_eq!(
            profile.recovery_mode(),
            RecoveryExecutionEffectRecoveryModeV1::ManualRecoveryOnly,
        );
    }

    #[test]
    fn unknown_external_effect_capabilities_never_expand_recovery_permission() {
        let profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::Unknown,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::Unknown,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::Unknown,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };

        assert!(!profile.automatic_retry_safe());
        assert!(!profile.automatic_takeover_safe());
        assert_eq!(
            profile.recovery_mode(),
            RecoveryExecutionEffectRecoveryModeV1::ManualRecoveryOnly,
        );
    }

    #[test]
    fn malformed_external_effect_safety_profile_fails_closed() {
        let mut profile = RecoveryExecutionEffectSafetyProfileV1 {
            schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
            fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
            idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
            reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
            claim_ceiling: "Adapter capability declaration only.".into(),
        };
        profile.schema = "wrong-schema".into();

        assert!(!profile.is_well_formed());
        assert!(!profile.automatic_retry_safe());
        assert_eq!(
            profile.recovery_mode(),
            RecoveryExecutionEffectRecoveryModeV1::ManualRecoveryOnly,
        );
    }

    #[test]
    fn protected_mutation_orchestration_is_fail_closed_after_indeterminate_recovery() {
        assert_eq!(
            RecoveryExecutionProtectedMutationOrchestrationStateV1::Indeterminate
                .next_action(),
            RecoveryExecutionProtectedMutationNextActionV1::ReconcileIndeterminateMutation,
        );
        assert_eq!(
            RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied
                .next_action(),
            RecoveryExecutionProtectedMutationNextActionV1::RequireIdempotencyOrManualRecovery,
        );
        assert_eq!(
            RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedAppliedSameRequest
                .next_action(),
            RecoveryExecutionProtectedMutationNextActionV1::ReturnRecordedOutcome,
        );
        assert_eq!(
            RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedDifferentRequest
                .next_action(),
            RecoveryExecutionProtectedMutationNextActionV1::FailClosed,
        );
    }

    #[test]
    fn protected_mutation_orchestration_fails_closed_for_missing_or_invalid_state() {
        for state in [
            RecoveryExecutionProtectedMutationOrchestrationStateV1::MissingState,
            RecoveryExecutionProtectedMutationOrchestrationStateV1::InvalidState,
        ] {
            assert_eq!(
                state.next_action(),
                RecoveryExecutionProtectedMutationNextActionV1::FailClosed,
            );
        }
    }

    #[test]
    fn protected_mutation_reconciliation_is_read_only_and_exact() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-reconcile".into(),
            execution_input_snapshot:
                "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
            attempt_id: "attempt-a".into(),
            fence_epoch: 2,
            idempotency_key: Some("stable-key".into()),
        };

        assert_eq!(
            resource
                .reconcile_mutation(&mutation)
                .expect("missing observation"),
            RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedNotApplied,
        );
        assert_eq!(
            resource
                .mutate_if_fence_is_current(&mutation)
                .expect("apply"),
            RecoveryExecutionProtectedMutationResult::Applied,
        );
        assert_eq!(
            resource
                .reconcile_mutation(&mutation)
                .expect("applied observation"),
            RecoveryExecutionProtectedMutationReconciliationOutcome::
                ObservedAppliedSameRequest,
        );
    }

    #[test]
    fn protected_mutation_indeterminate_ack_reconciles_without_retry() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-lost-ack".into(),
            execution_input_snapshot:
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
            attempt_id: "attempt-lost-ack".into(),
            fence_epoch: 2,
            idempotency_key: Some("stable-lost-ack".into()),
        };

        let applied = resource
            .mutate_if_fence_is_current(&mutation)
            .expect("apply before simulated response loss");
        assert_eq!(applied, RecoveryExecutionProtectedMutationResult::Applied);

        assert_eq!(
            RecoveryExecutionProtectedMutationOrchestrationStateV1::Indeterminate
                .next_action(),
            RecoveryExecutionProtectedMutationNextActionV1::ReconcileIndeterminateMutation,
        );
        assert_eq!(
            resource.reconcile_mutation(&mutation).expect("reconcile"),
            RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest,
        );
        assert_eq!(
            resource.reconcile_mutation(&mutation).expect("second read remains read-only"),
            RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest,
        );
    }

    #[test]
    fn protected_mutation_reconciliation_rejects_identity_drift() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-reconcile-drift".into(),
            execution_input_snapshot:
                "sha256:2222222222222222222222222222222222222222222222222222222222222222".into(),
            attempt_id: "attempt-a".into(),
            fence_epoch: 2,
            idempotency_key: Some("stable-key".into()),
        };

        resource
            .mutate_if_fence_is_current(&mutation)
            .expect("apply");
        let drifted = RecoveryExecutionProtectedMutationV1 {
            idempotency_key: Some("changed-key".into()),
            ..mutation
        };

        assert_eq!(
            resource
                .reconcile_mutation(&drifted)
                .expect("identity drift"),
            RecoveryExecutionProtectedMutationReconciliationOutcome::
                ObservedDifferentRequest,
        );
    }

    #[test]
    fn protected_mutation_reconciliation_rejects_malformed_input() {
        let resource = FencedResourceMemoryStore::default();
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-reconcile-invalid".into(),
            execution_input_snapshot: "not-a-digest".into(),
            attempt_id: "attempt-a".into(),
            fence_epoch: 1,
            idempotency_key: None,
        };

        assert_eq!(
            resource
                .reconcile_mutation(&mutation)
                .expect("invalid observation"),
            RecoveryExecutionProtectedMutationReconciliationOutcome::InvalidState,
        );
    }

    #[test]
    fn protected_mutation_rejects_an_empty_idempotency_key() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("fence");
        let store = FencedExecutionMemoryStore::default();
        let established = match establish_execution_fence(&store, &fence).expect("establish") {
            RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
            _ => panic!("fence must establish"),
        };

        assert!(
            RecoveryExecutionProtectedMutationV1::from_established_fence(
                &established,
                Some(String::new()),
            )
            .is_none()
        );
    }

    #[test]
    fn protected_resource_concurrent_mutations_preserve_fence_invariant() {
        let resource = Arc::new(FencedResourceMemoryStore::default());
        resource.set_epoch(2);

        let current = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-concurrent-current".into(),
            execution_input_snapshot:
                "sha256:3333333333333333333333333333333333333333333333333333333333333333".into(),
            attempt_id: "attempt-current".into(),
            fence_epoch: 2,
            idempotency_key: None,
        };
        let stale = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-concurrent-stale".into(),
            execution_input_snapshot:
                "sha256:4444444444444444444444444444444444444444444444444444444444444444".into(),
            attempt_id: "attempt-stale".into(),
            fence_epoch: 1,
            idempotency_key: None,
        };

        let mut joins = Vec::new();
        for _ in 0..32 {
            let resource = Arc::clone(&resource);
            let request = current.clone();
            joins.push(thread::spawn(move || {
                resource.mutate_if_fence_is_current(&request)
            }));
        }
        for _ in 0..32 {
            let resource = Arc::clone(&resource);
            let request = stale.clone();
            joins.push(thread::spawn(move || {
                resource.mutate_if_fence_is_current(&request)
            }));
        }

        for join in joins {
            let _ = join
                .join()
                .expect("resource mutation thread")
                .expect("resource result");
        }

        assert_eq!(
            resource
                .mutate_if_fence_is_current(&current)
                .expect("current replay"),
            RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest,
        );
        assert_eq!(
            resource
                .mutate_if_fence_is_current(&stale)
                .expect("stale rejection"),
            RecoveryExecutionProtectedMutationResult::RejectedStaleFence,
        );

        let state = resource.state.lock().expect("resource state lock");
        assert!(
            state
                .applied
                .values()
                .all(|(_, _, epoch, _)| *epoch <= state.current_epoch),
        );
        assert!(!state.applied.contains_key(&stale.execution_id));
    }

    #[test]
    fn protected_resource_accepts_only_the_current_fence_atomically() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let mutation = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-protected".into(),
            execution_input_snapshot:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            attempt_id: "attempt-b".into(),
            fence_epoch: 2,
            idempotency_key: Some("effect-key".into()),
        };

        assert_eq!(
            resource
                .mutate_if_fence_is_current(&mutation)
                .expect("current mutation"),
            RecoveryExecutionProtectedMutationResult::Applied
        );
        assert_eq!(
            resource
                .mutate_if_fence_is_current(&mutation)
                .expect("same request replay"),
            RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest
        );
    }

    #[test]
    fn protected_resource_rejects_a_changed_idempotency_key_on_replay() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let first = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-protected-idempotency".into(),
            execution_input_snapshot:
                "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                    .into(),
            attempt_id: "attempt-a".into(),
            fence_epoch: 2,
            idempotency_key: Some("stable-key-a".into()),
        };
        let changed = RecoveryExecutionProtectedMutationV1 {
            idempotency_key: Some("stable-key-b".into()),
            ..first.clone()
        };

        assert_eq!(
            resource
                .mutate_if_fence_is_current(&first)
                .expect("first mutation"),
            RecoveryExecutionProtectedMutationResult::Applied,
        );
        assert_eq!(
            resource
                .mutate_if_fence_is_current(&changed)
                .expect("changed idempotency key"),
            RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch,
        );
    }

    #[test]
    fn protected_resource_rejects_a_different_attempt_at_the_same_epoch() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let first = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-protected-owner".into(),
            execution_input_snapshot:
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .into(),
            attempt_id: "attempt-a".into(),
            fence_epoch: 2,
            idempotency_key: Some("effect-key".into()),
        };
        let second = RecoveryExecutionProtectedMutationV1 {
            attempt_id: "attempt-b".into(),
            ..first.clone()
        };

        assert_eq!(
            resource
                .mutate_if_fence_is_current(&first)
                .expect("first mutation"),
            RecoveryExecutionProtectedMutationResult::Applied
        );
        assert_eq!(
            resource
                .mutate_if_fence_is_current(&second)
                .expect("different attempt"),
            RecoveryExecutionProtectedMutationResult::RejectedOtherAttempt
        );
    }

    #[test]
    fn protected_resource_rejects_stale_and_future_fences() {
        let resource = FencedResourceMemoryStore::default();
        resource.set_epoch(2);
        let base = RecoveryExecutionProtectedMutationV1 {
            execution_id: "execution-protected-boundary".into(),
            execution_input_snapshot:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            attempt_id: "attempt-a".into(),
            fence_epoch: 1,
            idempotency_key: None,
        };

        assert_eq!(
            resource
                .mutate_if_fence_is_current(&base)
                .expect("stale mutation"),
            RecoveryExecutionProtectedMutationResult::RejectedStaleFence
        );

        let future = RecoveryExecutionProtectedMutationV1 {
            fence_epoch: 3,
            ..base
        };
        assert_eq!(
            resource
                .mutate_if_fence_is_current(&future)
                .expect("future mutation"),
            RecoveryExecutionProtectedMutationResult::RejectedFutureFence
        );
    }

    #[test]
    fn protected_mutation_request_cannot_detach_from_established_fence() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("fence");
        let store = FencedExecutionMemoryStore::default();
        let established = match establish_execution_fence(&store, &fence).expect("establish") {
            RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
            _ => panic!("fence must establish"),
        };
        let mutation = RecoveryExecutionProtectedMutationV1::from_established_fence(
            &established,
            Some("stable-effect-key".into()),
        )
        .expect("mutation");

        assert_eq!(
            validate_protected_mutation_request(&mutation, &established),
            RecoveryExecutionProtectedMutationValidation::Valid
        );

        let detached = RecoveryExecutionProtectedMutationV1 {
            fence_epoch: established.fence_epoch() + 1,
            ..mutation
        };
        assert_eq!(
            validate_protected_mutation_request(&detached, &established),
            RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch
        );
    }

    struct IndeterminateOutcomeStore {
        inner: MemoryStore,
        commit_before_indeterminate: bool,
    }

    impl IndeterminateOutcomeStore {
        fn new(
            decision: &RecoveryPolicyDecisionSnapshotV1,
            commit_before_indeterminate: bool,
        ) -> Self {
            Self {
                inner: MemoryStore::new(decision),
                commit_before_indeterminate,
            }
        }
    }

    #[derive(Default)]
    struct MemoryStore {
        values: Mutex<BTreeMap<String, RecoveryPolicyConsumptionSnapshotV1>>,
    }

    impl RecoveryPolicyConsumptionStore for IndeterminateOutcomeStore {
        type Error = &'static str;

        fn load(
            &self,
            decision_digest: &str,
        ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
            self.inner.load(decision_digest)
        }

        fn compare_and_set(
            &self,
            decision_digest: &str,
            expected_snapshot_digest: &str,
            next: &RecoveryPolicyConsumptionSnapshotV1,
        ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
            if self.commit_before_indeterminate {
                let result = self.inner.compare_and_set(
                    decision_digest,
                    expected_snapshot_digest,
                    next,
                )?;
                if result != RecoveryPolicyConsumptionCasResult::Committed {
                    return Ok(result);
                }
            }
            Ok(RecoveryPolicyConsumptionCasResult::Indeterminate)
        }
    }

    impl MemoryStore {
        fn new(decision: &RecoveryPolicyDecisionSnapshotV1) -> Self {
            let mut values = BTreeMap::new();
            values.insert(
                decision.digest(),
                RecoveryPolicyConsumptionSnapshotV1::for_decision(decision),
            );
            Self {
                values: Mutex::new(values),
            }
        }

        fn current(
            &self,
            decision: &RecoveryPolicyDecisionSnapshotV1,
        ) -> RecoveryPolicyConsumptionSnapshotV1 {
            self.values
                .lock()
                .expect("memory store is not poisoned")
                .get(&decision.digest())
                .expect("fixture state")
                .clone()
        }
    }

    struct ContendedMemoryStore {
        inner: MemoryStore,
        load_barrier: Barrier,
    }

    impl ContendedMemoryStore {
        fn new(decision: &RecoveryPolicyDecisionSnapshotV1) -> Self {
            Self {
                inner: MemoryStore::new(decision),
                load_barrier: Barrier::new(2),
            }
        }

        fn current(
            &self,
            decision: &RecoveryPolicyDecisionSnapshotV1,
        ) -> RecoveryPolicyConsumptionSnapshotV1 {
            self.inner.current(decision)
        }
    }

    impl RecoveryPolicyConsumptionStore for ContendedMemoryStore {
        type Error = &'static str;

        fn load(
            &self,
            decision_digest: &str,
        ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
            let current = self.inner.load(decision_digest)?;
            self.load_barrier.wait();
            Ok(current)
        }

        fn compare_and_set(
            &self,
            decision_digest: &str,
            expected_snapshot_digest: &str,
            next: &RecoveryPolicyConsumptionSnapshotV1,
        ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
            self.inner
                .compare_and_set(decision_digest, expected_snapshot_digest, next)
        }
    }

    #[derive(Default)]
    struct ExecutionEffectMemoryStore {
        values: Mutex<BTreeMap<String, RecoveryExecutionEffectReceiptV2>>,
    }

    impl RecoveryExecutionEffectStore for ExecutionEffectMemoryStore {
        type Error = &'static str;

        fn begin_effect(
            &self,
            receipt: &RecoveryExecutionEffectReceiptV2,
        ) -> Result<RecoveryExecutionEffectStartResult, Self::Error> {
            if !receipt.is_well_formed() {
                return Ok(RecoveryExecutionEffectStartResult::MalformedReceipt);
            }

            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&receipt.execution_id) else {
                values.insert(receipt.execution_id.clone(), receipt.clone());
                return Ok(RecoveryExecutionEffectStartResult::Started);
            };

            if !current.is_well_formed() {
                return Ok(RecoveryExecutionEffectStartResult::MalformedReceipt);
            }

            if current.execution_input_snapshot != receipt.execution_input_snapshot {
                return Ok(RecoveryExecutionEffectStartResult::FingerprintMismatch);
            }

            match &current.state {
                RecoveryExecutionEffectStateV1::Succeeded => {
                    return Ok(RecoveryExecutionEffectStartResult::AlreadySucceededSameRequest);
                }
                RecoveryExecutionEffectStateV1::Failed => {
                    return Ok(RecoveryExecutionEffectStartResult::AlreadyFailedSameRequest);
                }
                RecoveryExecutionEffectStateV1::InProgress => {}
            }

            if current.fence_epoch != receipt.fence_epoch {
                return Ok(RecoveryExecutionEffectStartResult::FenceMismatch);
            }

            Ok(match (&current.state, current.attempt_id == receipt.attempt_id) {
                (RecoveryExecutionEffectStateV1::InProgress, true) => {
                    RecoveryExecutionEffectStartResult::AlreadyInProgressSameAttempt
                }
                (RecoveryExecutionEffectStateV1::InProgress, false) => {
                    RecoveryExecutionEffectStartResult::AlreadyInProgressOtherAttempt
                }
                (RecoveryExecutionEffectStateV1::Succeeded, _) => {
                    RecoveryExecutionEffectStartResult::AlreadySucceededSameRequest
                }
                (RecoveryExecutionEffectStateV1::Failed, _) => {
                    RecoveryExecutionEffectStartResult::AlreadyFailedSameRequest
                }
            })
        }

        fn complete_effect(
            &self,
            execution_id: &str,
            execution_input_snapshot: &str,
            attempt_id: &str,
            fence_epoch: u64,
            completed: &RecoveryExecutionEffectReceiptV2,
        ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error> {
            if execution_id.is_empty()
                || !is_sha256_digest(execution_input_snapshot)
                || attempt_id.is_empty()
                || fence_epoch == 0
                || !completed.is_well_formed()
                || completed.execution_id != execution_id
                || completed.execution_input_snapshot != execution_input_snapshot
                || completed.attempt_id != attempt_id
                || completed.fence_epoch != fence_epoch
                || completed.state == RecoveryExecutionEffectStateV1::InProgress
            {
                return Ok(RecoveryExecutionEffectCompletionResult::MalformedReceipt);
            }

            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(execution_id) else {
                return Ok(RecoveryExecutionEffectCompletionResult::MissingReceipt);
            };

            if !current.is_well_formed() {
                return Ok(RecoveryExecutionEffectCompletionResult::MalformedReceipt);
            }

            if current.execution_input_snapshot != execution_input_snapshot {
                return Ok(RecoveryExecutionEffectCompletionResult::FingerprintMismatch);
            }

            if current.state != RecoveryExecutionEffectStateV1::InProgress {
                return Ok(
                    if current.state == completed.state
                        && current.outcome_digest == completed.outcome_digest
                    {
                        RecoveryExecutionEffectCompletionResult::AlreadyCompletedSameOutcome
                    } else {
                        RecoveryExecutionEffectCompletionResult::AlreadyCompletedDifferentOutcome
                    },
                );
            }

            if current.fence_epoch != fence_epoch || current.fence_epoch != completed.fence_epoch {
                return Ok(RecoveryExecutionEffectCompletionResult::FenceMismatch);
            }

            if current.attempt_id != attempt_id {
                return Ok(RecoveryExecutionEffectCompletionResult::NotOwner);
            }

            values.insert(execution_id.to_owned(), completed.clone());
            Ok(RecoveryExecutionEffectCompletionResult::Completed)
        }

        fn recover_effect_if_current(
            &self,
            expected: &RecoveryExecutionEffectReceiptV2,
            successor: &RecoveryExecutionEffectReceiptV2,
        ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error> {
            let valid_successor_epoch = expected.fence_epoch.checked_add(1);
            if !expected.is_well_formed()
                || expected.state != RecoveryExecutionEffectStateV1::InProgress
                || !successor.is_well_formed()
                || successor.state != RecoveryExecutionEffectStateV1::InProgress
                || successor.execution_id != expected.execution_id
                || successor.execution_input_snapshot != expected.execution_input_snapshot
                || successor.attempt_id == expected.attempt_id
                || valid_successor_epoch
                    .is_none_or(|epoch| successor.fence_epoch != epoch)
            {
                return Ok(RecoveryExecutionEffectRecoveryResult::MalformedReceipt);
            }

            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&expected.execution_id) else {
                return Ok(RecoveryExecutionEffectRecoveryResult::MissingReceipt);
            };

            if !current.is_well_formed() {
                return Ok(RecoveryExecutionEffectRecoveryResult::MalformedReceipt);
            }

            if current.execution_input_snapshot != expected.execution_input_snapshot {
                return Ok(RecoveryExecutionEffectRecoveryResult::FingerprintMismatch);
            }

            if current == successor {
                return Ok(RecoveryExecutionEffectRecoveryResult::AlreadyRecovered);
            }

            if current != expected {
                return Ok(RecoveryExecutionEffectRecoveryResult::StaleExpectedReceipt);
            }

            values.insert(expected.execution_id.clone(), successor.clone());
            Ok(RecoveryExecutionEffectRecoveryResult::Recovered)
        }

        fn load_effect(
            &self,
            execution_id: &str,
        ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error> {
            Ok(self
                .values
                .lock()
                .map_err(|_| "poisoned")?
                .get(execution_id)
                .cloned())
        }
    }

    struct FenceAdvancingEffectStore {
        inner: ExecutionEffectMemoryStore,
        fence_store: Arc<FencedExecutionMemoryStore>,
        expected_fence: RecoveryExecutionFenceV1,
        successor_fence: RecoveryExecutionFenceV1,
    }

    impl RecoveryExecutionEffectStore for FenceAdvancingEffectStore {
        type Error = &'static str;

        fn begin_effect(
            &self,
            receipt: &RecoveryExecutionEffectReceiptV2,
        ) -> Result<RecoveryExecutionEffectStartResult, Self::Error> {
            let outcome = self
                .fence_store
                .recover_if_current(&self.expected_fence, &self.successor_fence)?;
            assert_eq!(outcome, RecoveryExecutionFenceResult::Recovered);
            self.inner.begin_effect(receipt)
        }

        fn complete_effect(
            &self,
            execution_id: &str,
            execution_input_snapshot: &str,
            attempt_id: &str,
            fence_epoch: u64,
            completed: &RecoveryExecutionEffectReceiptV2,
        ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error> {
            self.inner.complete_effect(
                execution_id,
                execution_input_snapshot,
                attempt_id,
                fence_epoch,
                completed,
            )
        }

        fn recover_effect_if_current(
            &self,
            expected: &RecoveryExecutionEffectReceiptV2,
            successor: &RecoveryExecutionEffectReceiptV2,
        ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error> {
            self.inner.recover_effect_if_current(expected, successor)
        }

        fn load_effect(
            &self,
            execution_id: &str,
        ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error> {
            self.inner.load_effect(execution_id)
        }
    }

    #[test]
    fn cross_store_revalidation_exposes_race_without_claiming_atomicity() {
        let fence_store = Arc::new(FencedExecutionMemoryStore::default());
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(fence_store.as_ref(), &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };
        let successor = match recover_established_execution_fence(
            fence_store.as_ref(),
            &established,
            "attempt-b",
        )
        .expect("successor")
        {
            RecoveryExecutionFenceRecoveryEstablishmentV1::Established(fence) => fence,
            _ => panic!("successor must establish"),
        };
        let effect_store = FenceAdvancingEffectStore {
            inner: ExecutionEffectMemoryStore::default(),
            fence_store: Arc::clone(&fence_store),
            expected_fence: established.fence().clone(),
            successor_fence: successor.fence().clone(),
        };

        let outcome = begin_execution_effect_after_fence_revalidation(
            fence_store.as_ref(),
            &effect_store,
            &established,
        )
        .expect("cross-store admission");

        assert_eq!(
            outcome,
            RecoveryExecutionEffectFenceAdmissionResult::AttemptedAfterCurrentFenceObservation(
                RecoveryExecutionEffectStartResult::Started,
            ),
        );
        assert_eq!(
            reconcile_established_execution_fence(fence_store.as_ref(), &established)
                .expect("stale original fence"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence,
        );
        assert_eq!(
            effect_store
                .load_effect(&established.execution_id())
                .expect("effect load")
                .map(|receipt| receipt.fence_epoch),
            Some(established.fence_epoch()),
        );
    }

    fn effect_receipt_fixture(        attempt_id: &str,
        input_snapshot: &str,
    ) -> RecoveryExecutionEffectReceiptV2 {
        RecoveryExecutionEffectReceiptV2::in_progress(
            "effect-001",
            input_snapshot,
            attempt_id,
            1,
        )
    }

    #[test]
    fn effect_store_rejects_malformed_start_even_when_called_directly() {
        let store = ExecutionEffectMemoryStore::default();
        let mut receipt = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        receipt.outcome_digest = Some("not-a-digest".into());

        assert_eq!(
            store.begin_effect(&receipt).expect("direct malformed start"),
            RecoveryExecutionEffectStartResult::MalformedReceipt
        );
        assert!(
            store
                .load_effect(&receipt.execution_id)
                .expect("load after rejection")
                .is_none()
        );
    }

    #[test]
    fn effect_store_rejects_malformed_completion_without_mutating_state() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let mut malformed = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        malformed.outcome_digest = None;

        assert_eq!(
            store.begin_effect(&started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            store
                .complete_effect(
                    &started.execution_id,
                    &started.execution_input_snapshot,
                    &started.attempt_id,
                    started.fence_epoch,
                    &malformed,
                )
                .expect("direct malformed completion"),
            RecoveryExecutionEffectCompletionResult::MalformedReceipt
        );
        assert_eq!(
            store.load_effect(&started.execution_id).expect("load"),
            Some(started)
        );
    }

    #[test]
    fn effect_store_rejects_non_monotonic_recovery_without_mutating_state() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let mut successor = RecoveryExecutionEffectReceiptV2::in_progress(
            started.execution_id.clone(),
            started.execution_input_snapshot.clone(),
            "attempt-b",
            2,
        );
        successor.fence_epoch = 7;

        assert_eq!(
            store.begin_effect(&started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            store
                .recover_effect_if_current(&started, &successor)
                .expect("direct malformed recovery"),
            RecoveryExecutionEffectRecoveryResult::MalformedReceipt
        );
        assert_eq!(
            store.load_effect(&started.execution_id).expect("load"),
            Some(started)
        );
    }

    #[test]
    fn first_effect_attempt_is_started_once() {
        let store = ExecutionEffectMemoryStore::default();
        let receipt = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert_eq!(
            begin_execution_effect(&store, &receipt).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            begin_execution_effect(&store, &receipt).expect("duplicate start"),
            RecoveryExecutionEffectStartResult::AlreadyInProgressSameAttempt
        );
    }

    #[test]
    fn competing_effect_attempt_cannot_start_the_same_execution() {
        let store = Arc::new(ExecutionEffectMemoryStore::default());
        let first = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let second = effect_receipt_fixture(
            "attempt-b",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        let left_store = Arc::clone(&store);
        let left_receipt = first.clone();
        let left = thread::spawn(move || {
            begin_execution_effect(left_store.as_ref(), &left_receipt)
        });
        let right_store = Arc::clone(&store);
        let right_receipt = second.clone();
        let right = thread::spawn(move || {
            begin_execution_effect(right_store.as_ref(), &right_receipt)
        });

        let outcomes = [
            left.join().expect("left join").expect("left result"),
            right.join().expect("right join").expect("right result"),
        ];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == RecoveryExecutionEffectStartResult::Started)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| {
                    **outcome == RecoveryExecutionEffectStartResult::AlreadyInProgressOtherAttempt
                })
                .count(),
            1
        );
    }

    #[test]
    fn effect_cannot_reuse_execution_identity_for_different_input() {
        let store = ExecutionEffectMemoryStore::default();
        let first = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let different = effect_receipt_fixture(
            "attempt-b",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        );

        assert_eq!(
            begin_execution_effect(&store, &first).expect("first start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            begin_execution_effect(&store, &different).expect("fingerprint mismatch"),
            RecoveryExecutionEffectStartResult::FingerprintMismatch
        );
    }

    #[test]
    fn stale_effect_fence_cannot_complete_after_ownership_advances() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let current = RecoveryExecutionEffectReceiptV2 {
            attempt_id: "attempt-b".into(),
            fence_epoch: 2,
            ..started.clone()
        };
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        store
            .values
            .lock()
            .expect("memory store")
            .insert(started.execution_id.clone(), current);

        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("stale completion"),
            RecoveryExecutionEffectCompletionResult::FenceMismatch
        );
        assert_eq!(
            begin_execution_effect(&store, &started).expect("stale replay"),
            RecoveryExecutionEffectStartResult::FenceMismatch
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &started.execution_id,
                &started.execution_input_snapshot,
                &started.attempt_id,
                started.fence_epoch,
            )
            .expect("reconcile stale fence"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedStaleFence
        );
    }

    #[test]
    fn failed_effect_result_remains_replayable_after_fence_advances() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let failed = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Failed,
            outcome_digest: Some(
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .into(),
            ),
            ..started.clone()
        };
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor_fence =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-b")
                .expect("successor fence");
        let replay =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&successor_fence)
                .expect("new epoch replay");

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &failed).expect("fail"),
            RecoveryExecutionEffectCompletionResult::Completed
        );
        assert_eq!(
            begin_execution_effect(&store, &replay).expect("failed replay"),
            RecoveryExecutionEffectStartResult::AlreadyFailedSameRequest
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &replay.execution_id,
                &replay.execution_input_snapshot,
                &replay.attempt_id,
                replay.fence_epoch,
            )
            .expect("reconcile failed replay"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedFailed
        );
    }

    #[test]
    fn terminal_effect_completion_replay_remains_observable_after_fence_advances() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor_fence =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-b")
                .expect("successor fence");

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("complete"),
            RecoveryExecutionEffectCompletionResult::Completed
        );

        let replay_started =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&successor_fence)
                .expect("replay receipt");
        let replay_success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: success.outcome_digest.clone(),
            ..replay_started.clone()
        };

        assert_eq!(
            complete_execution_effect(&store, &replay_started, &replay_success)
                .expect("terminal replay"),
            RecoveryExecutionEffectCompletionResult::AlreadyCompletedSameOutcome
        );
    }

    #[test]
    fn terminal_effect_result_remains_replayable_after_fence_advances() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor_fence =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-b")
                .expect("successor fence");
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("complete"),
            RecoveryExecutionEffectCompletionResult::Completed
        );

        let replay_at_new_epoch = RecoveryExecutionEffectReceiptV2::in_progress_for_fence(
            &successor_fence,
        )
        .expect("new-epoch replay");
        assert_eq!(
            begin_execution_effect(&store, &replay_at_new_epoch).expect("terminal replay"),
            RecoveryExecutionEffectStartResult::AlreadySucceededSameRequest
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &replay_at_new_epoch.execution_id,
                &replay_at_new_epoch.execution_input_snapshot,
                &replay_at_new_epoch.attempt_id,
                replay_at_new_epoch.fence_epoch,
            )
            .expect("terminal reconciliation"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedSucceeded
        );
    }

    #[test]
    fn fenced_effect_recovery_advances_receipt_epoch_and_stales_old_owner() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor_fence =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-b")
                .expect("successor fence");
        let successor =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&successor_fence)
                .expect("successor receipt");
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            recover_execution_effect(&store, &started, &successor_fence).expect("recover"),
            RecoveryExecutionEffectRecoveryResult::Recovered
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("stale owner"),
            RecoveryExecutionEffectCompletionResult::FenceMismatch
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &started.execution_id,
                &started.execution_input_snapshot,
                &started.attempt_id,
                started.fence_epoch,
            )
            .expect("reconcile stale owner"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedStaleFence
        );
        assert_eq!(
            begin_execution_effect(&store, &successor).expect("recovered owner"),
            RecoveryExecutionEffectStartResult::AlreadyInProgressSameAttempt
        );
    }

    #[test]
    fn indeterminate_fenced_effect_recovery_reconciles_to_current_generation() {
        struct IndeterminateRecoveryStore {
            inner: ExecutionEffectMemoryStore,
        }

        impl RecoveryExecutionEffectStore for IndeterminateRecoveryStore {
            type Error = &'static str;

            fn begin_effect(
                &self,
                receipt: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectStartResult, Self::Error> {
                self.inner.begin_effect(receipt)
            }

            fn complete_effect(
                &self,
                execution_id: &str,
                execution_input_snapshot: &str,
                attempt_id: &str,
                fence_epoch: u64,
                completed: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error> {
                self.inner.complete_effect(
                    execution_id,
                    execution_input_snapshot,
                    attempt_id,
                    fence_epoch,
                    completed,
                )
            }

            fn recover_effect_if_current(
                &self,
                expected: &RecoveryExecutionEffectReceiptV2,
                successor: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error> {
                let result = self.inner.recover_effect_if_current(expected, successor)?;
                if result == RecoveryExecutionEffectRecoveryResult::Recovered {
                    return Ok(RecoveryExecutionEffectRecoveryResult::Indeterminate);
                }
                Ok(result)
            }

            fn load_effect(
                &self,
                execution_id: &str,
            ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error> {
                self.inner.load_effect(execution_id)
            }
        }

        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor_fence =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-b")
                .expect("successor fence");
        let successor =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&successor_fence)
                .expect("successor receipt");
        let store = IndeterminateRecoveryStore {
            inner: ExecutionEffectMemoryStore::default(),
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            recover_execution_effect(&store, &started, &successor_fence).expect("unknown recovery"),
            RecoveryExecutionEffectRecoveryResult::Indeterminate
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &successor.execution_id,
                &successor.execution_input_snapshot,
                &successor.attempt_id,
                successor.fence_epoch,
            )
            .expect("reconcile recovery"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedInProgressOwnedByThisAttempt
        );
        assert_eq!(
            begin_execution_effect(&store, &successor).expect("recovered replay"),
            RecoveryExecutionEffectStartResult::AlreadyInProgressSameAttempt
        );
    }

    #[test]
    fn concurrent_fenced_effect_recovery_has_one_winner() {
        let store = Arc::new(ExecutionEffectMemoryStore::default());
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor_fence_b =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-b")
                .expect("successor fence b");
        let successor_fence_c =
            RecoveryExecutionFenceV1::for_recovery(&initial_fence, "attempt-c")
                .expect("successor fence c");
        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );

        let left_store = Arc::clone(&store);
        let left_expected = started.clone();
        let left_successor_fence = successor_fence_b.clone();
        let left = thread::spawn(move || {
            recover_execution_effect(
                left_store.as_ref(),
                &left_expected,
                &left_successor_fence,
            )
        });

        let right_store = Arc::clone(&store);
        let right_expected = started.clone();
        let right_successor_fence = successor_fence_c.clone();
        let right = thread::spawn(move || {
            recover_execution_effect(
                right_store.as_ref(),
                &right_expected,
                &right_successor_fence,
            )
        });

        let outcomes = [
            left.join().expect("left join").expect("left result"),
            right.join().expect("right join").expect("right result"),
        ];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == RecoveryExecutionEffectRecoveryResult::Recovered)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| {
                    **outcome == RecoveryExecutionEffectRecoveryResult::StaleExpectedReceipt
                })
                .count(),
            1
        );
    }

    #[test]
    fn only_the_claim_owner_can_complete_an_effect() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let foreign = RecoveryExecutionEffectReceiptV2 {
            attempt_id: "attempt-b".into(),
            ..started.clone()
        };
        let foreign_success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..foreign.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &foreign, &foreign_success).expect("owner check"),
            RecoveryExecutionEffectCompletionResult::NotOwner
        );
    }

    #[test]
    fn completed_effect_replays_same_result_without_reexecution() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("complete"),
            RecoveryExecutionEffectCompletionResult::Completed
        );
        assert_eq!(
            begin_execution_effect(&store, &started).expect("replay"),
            RecoveryExecutionEffectStartResult::AlreadySucceededSameRequest
        );
    }

    #[test]
    fn indeterminate_effect_start_reconciles_without_retry() {
        struct IndeterminateEffectStore {
            inner: ExecutionEffectMemoryStore,
            commit: bool,
        }

        impl RecoveryExecutionEffectStore for IndeterminateEffectStore {
            type Error = &'static str;

            fn begin_effect(
                &self,
                receipt: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectStartResult, Self::Error> {
                if self.commit {
                    let result = self.inner.begin_effect(receipt)?;
                    if result != RecoveryExecutionEffectStartResult::Started {
                        return Ok(result);
                    }
                }
                Ok(RecoveryExecutionEffectStartResult::Indeterminate)
            }

            fn complete_effect(
                &self,
                execution_id: &str,
                execution_input_snapshot: &str,
                attempt_id: &str,
                fence_epoch: u64,
                completed: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error> {
                self.inner.complete_effect(
                    execution_id,
                    execution_input_snapshot,
                    attempt_id,
                    fence_epoch,
                    completed,
                )
            }

            fn recover_effect_if_current(
                &self,
                expected: &RecoveryExecutionEffectReceiptV2,
                successor: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error> {
                self.inner.recover_effect_if_current(expected, successor)
            }

            fn load_effect(
                &self,
                execution_id: &str,
            ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error> {
                self.inner.load_effect(execution_id)
            }
        }

        let receipt = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let store = IndeterminateEffectStore {
            inner: ExecutionEffectMemoryStore::default(),
            commit: true,
        };

        assert_eq!(
            begin_execution_effect(&store, &receipt).expect("indeterminate start"),
            RecoveryExecutionEffectStartResult::Indeterminate
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &receipt.execution_id,
                &receipt.execution_input_snapshot,
                &receipt.attempt_id,
                receipt.fence_epoch,
            )
            .expect("reconcile start"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedInProgressOwnedByThisAttempt
        );
    }

    #[test]
    fn concurrent_effect_completion_persists_only_one_terminal_outcome() {
        let store = Arc::new(ExecutionEffectMemoryStore::default());
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        let failure = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Failed,
            outcome_digest: Some(
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .into(),
            ),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );

        let left_store = Arc::clone(&store);
        let left_started = started.clone();
        let left_success = success.clone();
        let left = thread::spawn(move || {
            complete_execution_effect(&left_store, &left_started, &left_success)
        });

        let right_store = Arc::clone(&store);
        let right_started = started.clone();
        let right_failure = failure.clone();
        let right = thread::spawn(move || {
            complete_execution_effect(&right_store, &right_started, &right_failure)
        });

        let outcomes = [
            left.join().expect("left join").expect("left result"),
            right.join().expect("right join").expect("right result"),
        ];

        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == RecoveryExecutionEffectCompletionResult::Completed)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| {
                    **outcome
                        == RecoveryExecutionEffectCompletionResult::AlreadyCompletedDifferentOutcome
                })
                .count(),
            1
        );
    }

    #[test]
    fn strongest_completion_path_uses_established_fence_and_canonical_result() {
        let fence_store = FencedExecutionMemoryStore::default();
        let effect_store = ExecutionEffectMemoryStore::default();
        let execution = fixture().1;
        let mut claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        claim.execution_id = execution.execution_id.clone();
        let raw_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&fence_store, &raw_fence).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };
        let started =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(established.fence())
                .expect("started receipt");

        assert_eq!(
            begin_execution_effect_for_established_fence(&effect_store, &established)
                .expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect_for_established_fence(
                &effect_store,
                &started,
                &execution,
                &established,
            )
            .expect("complete"),
            RecoveryExecutionEffectCompletionResult::Completed
        );

        let result = RecoveryExecutionResultSnapshotV1::from_execution(&execution);
        let stored = effect_store
            .load_effect(&started.execution_id)
            .expect("load")
            .expect("stored");
        assert_eq!(stored.outcome_digest, Some(result.digest()));
    }

    #[test]
    fn canonical_execution_result_becomes_effect_outcome_identity() {
        let store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let execution = fixture().1;
        let mut claim = claim;
        claim.execution_id = execution.execution_id.clone();
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let started =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&fence)
                .expect("started receipt");

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect_from_execution(&store, &started, &execution, &fence)
                .expect("complete from canonical result"),
            RecoveryExecutionEffectCompletionResult::Completed
        );

        let result = RecoveryExecutionResultSnapshotV1::from_execution(&execution);
        let stored = store
            .load_effect(&started.execution_id)
            .expect("load")
            .expect("stored receipt");
        assert_eq!(stored.outcome_digest, Some(result.digest()));
    }

    #[test]
    fn canonical_failure_result_becomes_failed_effect_identity() {
        let store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let mut execution = fixture().1;
        let mut claim = claim;
        claim.execution_id = execution.execution_id.clone();
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        execution.completed_steps.clear();
        execution.failed_steps = vec!["verify".into()];
        execution.failure_reason = Some("verification failed".into());
        let started =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&fence)
                .expect("started receipt");

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect_from_execution(&store, &started, &execution, &fence)
                .expect("complete failure"),
            RecoveryExecutionEffectCompletionResult::Completed
        );

        let result = RecoveryExecutionResultSnapshotV1::from_execution(&execution);
        let stored = store
            .load_effect(&started.execution_id)
            .expect("load")
            .expect("stored receipt");
        assert_eq!(stored.state, RecoveryExecutionEffectStateV1::Failed);
        assert_eq!(stored.outcome_digest, Some(result.digest()));
    }

    #[test]
    fn canonical_result_path_rejects_execution_fingerprint_mismatch() {
        let store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let mut execution = fixture().1;
        let mut claim = claim;
        claim.execution_id = execution.execution_id.clone();
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        execution.input_snapshot =
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        let started =
            RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&fence)
                .expect("started receipt");

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect_from_execution(&store, &started, &execution, &fence)
                .expect("reject mismatch"),
            RecoveryExecutionEffectCompletionResult::MalformedReceipt
        );
        assert_eq!(
            store.load_effect(&started.execution_id).expect("load"),
            Some(started)
        );
    }

    #[test]
    fn terminal_state_is_part_of_effect_outcome_identity() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        let failure_with_same_digest = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Failed,
            outcome_digest: success.outcome_digest.clone(),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("complete"),
            RecoveryExecutionEffectCompletionResult::Completed
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &failure_with_same_digest)
                .expect("different terminal outcome"),
            RecoveryExecutionEffectCompletionResult::AlreadyCompletedDifferentOutcome
        );
    }

    #[test]
    fn indeterminate_effect_completion_reconciles_to_terminal_success() {
        struct IndeterminateCompletionStore {
            inner: ExecutionEffectMemoryStore,
        }

        impl RecoveryExecutionEffectStore for IndeterminateCompletionStore {
            type Error = &'static str;

            fn begin_effect(
                &self,
                receipt: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectStartResult, Self::Error> {
                self.inner.begin_effect(receipt)
            }

            fn complete_effect(
                &self,
                execution_id: &str,
                execution_input_snapshot: &str,
                attempt_id: &str,
                fence_epoch: u64,
                completed: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error> {
                let result = self.inner.complete_effect(
                    execution_id,
                    execution_input_snapshot,
                    attempt_id,
                    fence_epoch,
                    completed,
                )?;
                if result == RecoveryExecutionEffectCompletionResult::Completed {
                    return Ok(RecoveryExecutionEffectCompletionResult::Indeterminate);
                }
                Ok(result)
            }

            fn recover_effect_if_current(
                &self,
                expected: &RecoveryExecutionEffectReceiptV2,
                successor: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error> {
                self.inner.recover_effect_if_current(expected, successor)
            }

            fn load_effect(
                &self,
                execution_id: &str,
            ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error> {
                self.inner.load_effect(execution_id)
            }
        }

        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        let store = IndeterminateCompletionStore {
            inner: ExecutionEffectMemoryStore::default(),
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("unknown completion"),
            RecoveryExecutionEffectCompletionResult::Indeterminate
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &started.execution_id,
                &started.execution_input_snapshot,
                &started.attempt_id,
                started.fence_epoch,
            )
            .expect("reconcile completion"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedSucceeded
        );
    }

    #[test]
    fn indeterminate_effect_completion_without_commit_remains_in_progress() {
        struct NoCommitCompletionStore {
            inner: ExecutionEffectMemoryStore,
        }

        impl RecoveryExecutionEffectStore for NoCommitCompletionStore {
            type Error = &'static str;

            fn begin_effect(
                &self,
                receipt: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectStartResult, Self::Error> {
                self.inner.begin_effect(receipt)
            }

            fn complete_effect(
                &self,
                _execution_id: &str,
                _execution_input_snapshot: &str,
                _attempt_id: &str,
                _fence_epoch: u64,
                _completed: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error> {
                Ok(RecoveryExecutionEffectCompletionResult::Indeterminate)
            }

            fn recover_effect_if_current(
                &self,
                expected: &RecoveryExecutionEffectReceiptV2,
                successor: &RecoveryExecutionEffectReceiptV2,
            ) -> Result<RecoveryExecutionEffectRecoveryResult, Self::Error> {
                self.inner.recover_effect_if_current(expected, successor)
            }

            fn load_effect(
                &self,
                execution_id: &str,
            ) -> Result<Option<RecoveryExecutionEffectReceiptV2>, Self::Error> {
                self.inner.load_effect(execution_id)
            }
        }

        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        let store = NoCommitCompletionStore {
            inner: ExecutionEffectMemoryStore::default(),
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &success).expect("unknown completion"),
            RecoveryExecutionEffectCompletionResult::Indeterminate
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &started.execution_id,
                &started.execution_input_snapshot,
                &started.attempt_id,
                started.fence_epoch,
            )
            .expect("reconcile in-progress"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedInProgressOwnedByThisAttempt
        );
    }

    #[test]
    fn completed_failure_is_replayed_without_reexecution() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let failed = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Failed,
            outcome_digest: Some(
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .into(),
            ),
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            complete_execution_effect(&store, &started, &failed).expect("complete failure"),
            RecoveryExecutionEffectCompletionResult::Completed
        );
        assert_eq!(
            begin_execution_effect(&store, &started).expect("failure replay"),
            RecoveryExecutionEffectStartResult::AlreadyFailedSameRequest
        );
        assert_eq!(
            reconcile_execution_effect(
                &store,
                &started.execution_id,
                &started.execution_input_snapshot,
                &started.attempt_id,
                started.fence_epoch,
            )
            .expect("reconcile failure"),
            RecoveryExecutionEffectReconciliationOutcome::ObservedFailed
        );
    }

    #[test]
    fn invalid_effect_completion_never_reaches_store() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let success = RecoveryExecutionEffectReceiptV2 {
            state: RecoveryExecutionEffectStateV1::Succeeded,
            outcome_digest: Some(
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .into(),
            ),
            ..started.clone()
        };
        let malformed_started = RecoveryExecutionEffectReceiptV2 {
            attempt_id: "attempt-b".into(),
            ..started
        };

        assert_eq!(
            complete_execution_effect(&store, &malformed_started, &success)
                .expect("malformed completion"),
            RecoveryExecutionEffectCompletionResult::MalformedReceipt
        );
    }

    #[derive(Default)]
    struct ExecutionClaimMemoryStore {
        values: Mutex<BTreeMap<String, RecoveryExecutionClaimV1>>,
    }

    impl RecoveryExecutionClaimStore for ExecutionClaimMemoryStore {
        type Error = &'static str;

        fn claim_if_absent(
            &self,
            claim: &RecoveryExecutionClaimV1,
        ) -> Result<RecoveryExecutionClaimResult, Self::Error> {
            if !claim.is_well_formed() {
                return Ok(RecoveryExecutionClaimResult::MalformedClaim);
            }

            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&claim.execution_id) else {
                values.insert(claim.execution_id.clone(), claim.clone());
                return Ok(RecoveryExecutionClaimResult::Acquired);
            };

            if !current.is_well_formed() {
                return Ok(RecoveryExecutionClaimResult::MalformedClaim);
            }

            if current.execution_input_snapshot != claim.execution_input_snapshot {
                return Ok(RecoveryExecutionClaimResult::ExecutionIdentityReuseMismatch);
            }

            if current.attempt_id == claim.attempt_id {
                return Ok(RecoveryExecutionClaimResult::AlreadyClaimedSameAttempt);
            }

            Ok(RecoveryExecutionClaimResult::AlreadyClaimedDifferentAttempt)
        }

        fn load_claim(
            &self,
            execution_id: &str,
        ) -> Result<Option<RecoveryExecutionClaimV1>, Self::Error> {
            Ok(self
                .values
                .lock()
                .map_err(|_| "poisoned")?
                .get(execution_id)
                .cloned())
        }
    }

    #[derive(Default)]
    struct FencedExecutionMemoryStore {
        values: Mutex<BTreeMap<String, RecoveryExecutionFenceV1>>,
    }

    impl RecoveryExecutionFenceStore for FencedExecutionMemoryStore {
        type Error = &'static str;

        fn acquire_fence(
            &self,
            fence: &RecoveryExecutionFenceV1,
        ) -> Result<RecoveryExecutionFenceResult, Self::Error> {
            if !fence.is_well_formed() || fence.fence_epoch != 1 {
                return Ok(RecoveryExecutionFenceResult::MalformedFence);
            }

            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&fence.execution_id) else {
                values.insert(fence.execution_id.clone(), fence.clone());
                return Ok(RecoveryExecutionFenceResult::Acquired);
            };

            if !current.is_well_formed() {
                return Ok(RecoveryExecutionFenceResult::MalformedFence);
            }

            if current.execution_input_snapshot != fence.execution_input_snapshot {
                return Ok(RecoveryExecutionFenceResult::FingerprintMismatch);
            }

            Ok(if current.attempt_id == fence.attempt_id
                && current.fence_epoch == fence.fence_epoch
            {
                RecoveryExecutionFenceResult::AlreadyOwnedSameAttempt
            } else {
                RecoveryExecutionFenceResult::AlreadyOwnedOtherAttempt
            })
        }

        fn recover_if_current(
            &self,
            expected: &RecoveryExecutionFenceV1,
            successor: &RecoveryExecutionFenceV1,
        ) -> Result<RecoveryExecutionFenceResult, Self::Error> {
            if !expected.is_well_formed()
                || !successor.is_well_formed()
                || successor.execution_id != expected.execution_id
                || successor.execution_input_snapshot != expected.execution_input_snapshot
                || successor.attempt_id == expected.attempt_id
                || expected
                    .fence_epoch
                    .checked_add(1)
                    .is_none_or(|next| successor.fence_epoch != next)
            {
                return Ok(RecoveryExecutionFenceResult::MalformedFence);
            }

            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&expected.execution_id) else {
                return Ok(RecoveryExecutionFenceResult::MissingCurrentFence);
            };

            if !current.is_well_formed() {
                return Ok(RecoveryExecutionFenceResult::MalformedFence);
            }

            if current.execution_input_snapshot != expected.execution_input_snapshot {
                return Ok(RecoveryExecutionFenceResult::FingerprintMismatch);
            }

            if current != expected {
                return Ok(RecoveryExecutionFenceResult::StaleExpectedFence);
            }

            values.insert(expected.execution_id.clone(), successor.clone());
            Ok(RecoveryExecutionFenceResult::Recovered)
        }

        fn load_fence(
            &self,
            execution_id: &str,
        ) -> Result<Option<RecoveryExecutionFenceV1>, Self::Error> {
            Ok(self
                .values
                .lock()
                .map_err(|_| "poisoned")?
                .get(execution_id)
                .cloned())
        }
    }
    struct BrokenStore;

    impl RecoveryPolicyConsumptionStore for BrokenStore {
        type Error = &'static str;

        fn load(
            &self,
            _decision_digest: &str,
        ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
            Err("load failed")
        }

        fn compare_and_set(
            &self,
            _decision_digest: &str,
            _expected_snapshot_digest: &str,
            _next: &RecoveryPolicyConsumptionSnapshotV1,
        ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
            unreachable!("load failure must prevent CAS")
        }
    }

    impl RecoveryPolicyConsumptionStore for MemoryStore {
        type Error = &'static str;

        fn load(
            &self,
            decision_digest: &str,
        ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
            Ok(self
                .values
                .lock()
                .map_err(|_| "poisoned")?
                .get(decision_digest)
                .cloned())
        }

        fn compare_and_set(
            &self,
            decision_digest: &str,
            expected_snapshot_digest: &str,
            next: &RecoveryPolicyConsumptionSnapshotV1,
        ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(decision_digest) else {
                return Ok(RecoveryPolicyConsumptionCasResult::NotCurrent);
            };
            if current.digest() != expected_snapshot_digest {
                return Ok(RecoveryPolicyConsumptionCasResult::NotCurrent);
            }
            values.insert(decision_digest.to_owned(), next.clone());
            Ok(RecoveryPolicyConsumptionCasResult::Committed)
        }
    }

    fn fixture() -> (
        RecoveryPolicyDecisionSnapshotV1,
        RecoveryExecution,
        RecoveryPolicyConsumptionSnapshotV1,
    ) {
        let mut execution = RecoveryExecution {
            plan_id: "plan-store-contract".into(),
            execution_id: "execution-store-contract".into(),
            started_at: "2026-10-02T07:59:00Z".into(),
            ended_at: Some("2026-10-02T08:00:00Z".into()),
            attempted_steps: vec!["verify".into()],
            completed_steps: vec!["verify".into()],
            failed_steps: vec![],
            observed_preconditions: vec![],
            evidence: vec!["store-contract".into()],
            resulting_state: CapabilityState::Demonstrated,
            authorization: None,
            ai_assistance: None,
            input_snapshot:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            failure_reason: None,
            claim_ceiling: "Exact execution scope only.".into(),
        };

        let decision = RecoveryPolicyDecisionSnapshotV1 {
            schema: RecoveryPolicyDecisionSnapshotV1::SCHEMA.into(),
            id: "decision-store-contract".into(),
            decision: RecoveryPolicyDecisionV1::Admitted,
            purpose: "recovery.execute".into(),
            consumer: "operator-001".into(),
            plan_id: execution.plan_id.clone(),
            plan_snapshot: "sha256:plan".into(),
            candidate: CapabilityId("recovery".into()),
            candidate_snapshot: "sha256:candidate".into(),
            execution_id: Some(execution.execution_id.clone()),
            authority_reference: "authority-record".into(),
            issued_at: "2026-10-02T07:50:00Z".into(),
            valid_until: "2026-10-02T08:10:00Z".into(),
            claim_ceiling: "Exact recovery admission only.".into(),
        };

        execution.authorization = Some(decision.digest());
        let current = RecoveryPolicyConsumptionSnapshotV1::for_decision(&decision);
        (decision, execution, current)
    }

    fn transition_fixture(
        decision: &RecoveryPolicyDecisionSnapshotV1,
        execution: &RecoveryExecution,
        current: &RecoveryPolicyConsumptionSnapshotV1,
        consumed_at: &str,
    ) -> (
        RecoveryPolicyConsumptionTransitionV1,
        RecoveryPolicyConsumptionSnapshotV1,
    ) {
        let next = current
            .consumed(decision, execution, consumed_at)
            .expect("valid successor");
        let transition = RecoveryPolicyConsumptionTransitionV1::for_successful_consumption(
            current,
            decision,
            execution,
            consumed_at,
        )
        .expect("valid transition");
        (transition, next)
    }

    fn execution_claim_fixture(attempt_id: &str, input_snapshot: &str) -> RecoveryExecutionClaimV1 {
        RecoveryExecutionClaimV1 {
            schema: RecoveryExecutionClaimV1::SCHEMA.into(),
            execution_id: "execution-claim-001".into(),
            execution_input_snapshot: input_snapshot.into(),
            attempt_id: attempt_id.into(),
        }
    }

    #[test]
    fn authorization_orchestration_matrix_is_fail_closed_and_exhaustive() {
        assert_eq!(
            RecoveryAuthorizationOrchestrationStateV1::AuthorizationUnconsumedNoClaim.next_action(),
            RecoveryAuthorizationOrchestrationActionV1::AcquireExecutionClaim
        );
        assert_eq!(
            RecoveryAuthorizationOrchestrationStateV1::AuthorizationUnconsumedSameClaim
                .next_action(),
            RecoveryAuthorizationOrchestrationActionV1::ConsumeAuthorization
        );
        assert_eq!(
            RecoveryAuthorizationOrchestrationStateV1::AuthorizationConsumedSameClaim.next_action(),
            RecoveryAuthorizationOrchestrationActionV1::ReconcileAndContinueExecution
        );
        assert_eq!(
            RecoveryAuthorizationOrchestrationStateV1::AuthorizationConsumedNoClaim
                .next_action(),
            RecoveryAuthorizationOrchestrationActionV1::
                ReconcileExecutionAndEffectBeforeIrreversibleWork
        );
        assert_eq!(
            RecoveryAuthorizationOrchestrationStateV1::AuthorizationConsumptionIndeterminate
                .next_action(),
            RecoveryAuthorizationOrchestrationActionV1::ReconcileAuthorizationConsumption
        );
    }

    #[test]
    fn effect_orchestration_matrix_preserves_terminal_replay_and_live_ownership_boundaries() {
        assert_eq!(
            RecoveryEffectOrchestrationStateV1::InProgressSameAttempt.next_action(),
            RecoveryEffectOrchestrationActionV1::ReconcileWithoutRestart
        );
        assert_eq!(
            RecoveryEffectOrchestrationStateV1::InProgressOtherAttempt.next_action(),
            RecoveryEffectOrchestrationActionV1::FailClosedOrUseAdapterSpecificRecovery
        );
        assert_eq!(
            RecoveryEffectOrchestrationStateV1::Succeeded.next_action(),
            RecoveryEffectOrchestrationActionV1::ReturnRecordedSuccess
        );
        assert_eq!(
            RecoveryEffectOrchestrationStateV1::Failed.next_action(),
            RecoveryEffectOrchestrationActionV1::ApplyExplicitRetryPolicy
        );
    }

    #[test]
    fn terminal_execution_cannot_acquire_a_start_claim() {
        let execution = fixture().1;
        assert!(
            RecoveryExecutionClaimV1::for_execution(&execution, "attempt-a").is_none()
        );
    }

    #[test]
    fn execution_claim_uses_the_existing_execution_input_fingerprint() {
        let execution = fixture().1;
        let claim = RecoveryExecutionClaimV1::for_execution(&execution, "attempt-a")
            .expect("well-formed execution claim");
        assert_eq!(claim.execution_id, execution.execution_id);
        assert_eq!(claim.execution_input_snapshot, execution.input_snapshot);
        assert_eq!(claim.attempt_id, "attempt-a");
    }

    #[test]
    fn malformed_execution_claim_never_reaches_store() {
        struct NoClaimStore;

        impl RecoveryExecutionClaimStore for NoClaimStore {
            type Error = &'static str;

            fn claim_if_absent(
                &self,
                _claim: &RecoveryExecutionClaimV1,
            ) -> Result<RecoveryExecutionClaimResult, Self::Error> {
                Err("claim store must not be reached")
            }

            fn load_claim(
                &self,
                _execution_id: &str,
            ) -> Result<Option<RecoveryExecutionClaimV1>, Self::Error> {
                Err("claim store must not be reached")
            }
        }

        let mut claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        claim.attempt_id.clear();
        assert_eq!(
            claim_execution_start(&NoClaimStore, &claim).expect("malformed claim outcome"),
            RecoveryExecutionClaimResult::MalformedClaim
        );
    }

    #[test]
    fn claim_store_rejects_malformed_claim_even_when_called_directly() {
        let store = ExecutionClaimMemoryStore::default();
        let mut claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        claim.attempt_id.clear();

        assert_eq!(
            store.claim_if_absent(&claim).expect("direct claim"),
            RecoveryExecutionClaimResult::MalformedClaim
        );
        assert!(
            store
                .load_claim(&claim.execution_id)
                .expect("load after rejection")
                .is_none()
        );
    }

    #[test]
    fn first_execution_claim_is_acquired_atomically() {
        let store = ExecutionClaimMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert_eq!(
            claim_execution_start(&store, &claim).expect("claim"),
            RecoveryExecutionClaimResult::Acquired
        );
    }

    #[test]
    fn same_attempt_replay_is_idempotent_but_not_side_effect_success() {
        let store = ExecutionClaimMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert_eq!(
            claim_execution_start(&store, &claim).expect("first claim"),
            RecoveryExecutionClaimResult::Acquired
        );
        assert_eq!(
            claim_execution_start(&store, &claim).expect("replay claim"),
            RecoveryExecutionClaimResult::AlreadyClaimedSameAttempt
        );
    }

    #[test]
    fn competing_attempt_cannot_steal_the_same_execution() {
        let store = Arc::new(ExecutionClaimMemoryStore::default());
        let claim_a = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let claim_b = execution_claim_fixture(
            "attempt-b",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        let left_store = Arc::clone(&store);
        let left_claim = claim_a.clone();
        let left = thread::spawn(move || claim_execution_start(left_store.as_ref(), &left_claim));

        let right_store = Arc::clone(&store);
        let right_claim = claim_b.clone();
        let right = thread::spawn(move || {
            claim_execution_start(right_store.as_ref(), &right_claim)
        });

        let outcomes = [
            left.join().expect("left join").expect("left result"),
            right.join().expect("right join").expect("right result"),
        ];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == RecoveryExecutionClaimResult::Acquired)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| {
                    **outcome == RecoveryExecutionClaimResult::AlreadyClaimedDifferentAttempt
                })
                .count(),
            1
        );
    }

    #[test]
    fn execution_identity_cannot_be_reused_for_a_different_fingerprint() {
        let store = ExecutionClaimMemoryStore::default();
        let first = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let different = execution_claim_fixture(
            "attempt-a",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        );

        assert_eq!(
            claim_execution_start(&store, &first).expect("first claim"),
            RecoveryExecutionClaimResult::Acquired
        );
        assert_eq!(
            claim_execution_start(&store, &different).expect("reuse"),
            RecoveryExecutionClaimResult::ExecutionIdentityReuseMismatch
        );
    }

    #[test]
    fn indeterminate_execution_claim_reconciles_to_this_attempt() {
        struct IndeterminateClaimStore {
            inner: ExecutionClaimMemoryStore,
        }

        impl RecoveryExecutionClaimStore for IndeterminateClaimStore {
            type Error = &'static str;

            fn claim_if_absent(
                &self,
                claim: &RecoveryExecutionClaimV1,
            ) -> Result<RecoveryExecutionClaimResult, Self::Error> {
                self.inner.claim_if_absent(claim)?;
                Ok(RecoveryExecutionClaimResult::Indeterminate)
            }

            fn load_claim(
                &self,
                execution_id: &str,
            ) -> Result<Option<RecoveryExecutionClaimV1>, Self::Error> {
                self.inner.load_claim(execution_id)
            }
        }

        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let store = IndeterminateClaimStore {
            inner: ExecutionClaimMemoryStore::default(),
        };

        assert_eq!(
            claim_execution_start(&store, &claim).expect("indeterminate claim"),
            RecoveryExecutionClaimResult::Indeterminate
        );
        assert_eq!(
            reconcile_execution_claim(&store, &claim).expect("reconcile"),
            RecoveryExecutionClaimReconciliationOutcome::ObservedOwnedByThisAttempt
        );
    }

    #[test]
    fn indeterminate_execution_claim_can_reconcile_to_missing() {
        struct NoCommitClaimStore;

        impl RecoveryExecutionClaimStore for NoCommitClaimStore {
            type Error = &'static str;

            fn claim_if_absent(
                &self,
                _claim: &RecoveryExecutionClaimV1,
            ) -> Result<RecoveryExecutionClaimResult, Self::Error> {
                Ok(RecoveryExecutionClaimResult::Indeterminate)
            }

            fn load_claim(
                &self,
                _execution_id: &str,
            ) -> Result<Option<RecoveryExecutionClaimV1>, Self::Error> {
                Ok(None)
            }
        }

        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let store = NoCommitClaimStore;

        assert_eq!(
            claim_execution_start(&store, &claim).expect("indeterminate claim"),
            RecoveryExecutionClaimResult::Indeterminate
        );
        assert_eq!(
            reconcile_execution_claim(&store, &claim).expect("reconcile"),
            RecoveryExecutionClaimReconciliationOutcome::MissingClaim
        );
    }

    #[test]
    fn initial_fence_must_start_at_epoch_one() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let mut fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("fence");
        fence.fence_epoch = 2;
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            acquire_execution_fence(&store, &fence).expect("invalid epoch"),
            RecoveryExecutionFenceResult::MalformedFence
        );
    }

    #[test]
    #[test]
    fn fence_store_rejects_non_initial_epoch_even_when_called_directly() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let mut fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        fence.fence_epoch = 2;
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            store.acquire_fence(&fence).expect("direct store acquire"),
            RecoveryExecutionFenceResult::MalformedFence
        );
        assert!(
            store
                .load_fence(&fence.execution_id)
                .expect("load after rejection")
                .is_none()
        );
    }

    #[test]
    fn fence_store_rejects_non_monotonic_successor_even_when_called_directly() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let mut successor =
            RecoveryExecutionFenceV1::for_recovery(&initial, "attempt-b").expect("successor");
        successor.fence_epoch = 7;
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            store.acquire_fence(&initial).expect("initial acquire"),
            RecoveryExecutionFenceResult::Acquired
        );
        assert_eq!(
            store
                .recover_if_current(&initial, &successor)
                .expect("direct recovery"),
            RecoveryExecutionFenceResult::MalformedFence
        );
        assert_eq!(
            store.load_fence(&initial.execution_id).expect("load current"),
            Some(initial)
        );
    }

    #[test]
    fn initial_fence_is_acquired_once_and_replayed_idempotently() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            acquire_execution_fence(&store, &fence).expect("acquire"),
            RecoveryExecutionFenceResult::Acquired
        );
        assert_eq!(
            acquire_execution_fence(&store, &fence).expect("replay"),
            RecoveryExecutionFenceResult::AlreadyOwnedSameAttempt
        );
    }

    #[test]
    fn fence_revalidated_effect_start_rejects_stale_authority_before_mutating_effect_store() {
        let fence_store = FencedExecutionMemoryStore::default();
        let effect_store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&fence_store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        match recover_established_execution_fence(&fence_store, &established, "attempt-b")
            .expect("recover fence")
        {
            RecoveryExecutionFenceRecoveryEstablishmentV1::Established(_) => {}
            _ => panic!("fence recovery must succeed"),
        }

        assert_eq!(
            begin_execution_effect_after_fence_revalidation(
                &fence_store,
                &effect_store,
                &established,
            )
            .expect("stale admission"),
            RecoveryExecutionEffectFenceAdmissionResult::RejectedByFence(
                RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence,
            )
        );
        assert!(
            effect_store
                .load_effect(&established.execution_id())
                .expect("effect load")
                .is_none()
        );
    }

    #[test]
    fn fence_revalidated_effect_start_marks_success_as_point_in_time_observation() {
        let fence_store = FencedExecutionMemoryStore::default();
        let effect_store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&fence_store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        assert_eq!(
            begin_execution_effect_after_fence_revalidation(
                &fence_store,
                &effect_store,
                &established,
            )
            .expect("admission"),
            RecoveryExecutionEffectFenceAdmissionResult::AttemptedAfterCurrentFenceObservation(
                RecoveryExecutionEffectStartResult::Started,
            )
        );
    }

    #[test]
    fn established_fence_freshness_check_is_point_in_time_and_fail_closed() {
        let store = FencedExecutionMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        assert_eq!(
            reconcile_established_execution_fence(&store, &established)
                .expect("current freshness"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedCurrentOwnedByThisAttempt
        );

        let successor =
            match recover_established_execution_fence(&store, &established, "attempt-b")
                .expect("recover")
            {
                RecoveryExecutionFenceRecoveryEstablishmentV1::Established(fence) => fence,
                _ => panic!("recovery must succeed"),
            };
        assert_eq!(successor.fence_epoch(), established.fence_epoch() + 1);

        assert_eq!(
            reconcile_established_execution_fence(&store, &established)
                .expect("stale freshness"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence
        );
        assert_eq!(
            reconcile_established_execution_fence(&store, &successor)
                .expect("current successor freshness"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedCurrentOwnedByThisAttempt
        );
    }

    #[test]
    fn established_fence_freshness_check_reports_missing_and_malformed_state() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&FencedExecutionMemoryStore::default(), &fence)
                .expect("establish")
            {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        let missing = FencedExecutionMemoryStore::default();
        assert_eq!(
            reconcile_established_execution_fence(&missing, &established)
                .expect("missing"),
            RecoveryExecutionFenceReconciliationOutcome::MissingFence
        );

        let malformed = FencedExecutionMemoryStore::default();
        malformed
            .values
            .lock()
            .expect("memory store")
            .insert(
                established.execution_id().to_owned(),
                RecoveryExecutionFenceV1 {
                    schema: RecoveryExecutionFenceV1::SCHEMA.into(),
                    execution_id: established.execution_id().into(),
                    execution_input_snapshot: "not-a-digest".into(),
                    attempt_id: established.attempt_id().into(),
                    fence_epoch: established.fence_epoch(),
                },
            );
        assert_eq!(
            reconcile_established_execution_fence(&malformed, &established)
                .expect("malformed"),
            RecoveryExecutionFenceReconciliationOutcome::InvalidFence
        );
    }

    #[test]
    fn stale_established_fence_cannot_begin_after_effect_recovery() {
        let fence_store = FencedExecutionMemoryStore::default();
        let effect_store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&fence_store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };
        let successor =
            match recover_established_execution_fence(&fence_store, &established, "attempt-b")
                .expect("recover fence")
            {
                RecoveryExecutionFenceRecoveryEstablishmentV1::Established(fence) => fence,
                _ => panic!("fence recovery must succeed"),
            };

        assert_eq!(
            begin_execution_effect_for_established_fence(&effect_store, &established)
                .expect("initial effect start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            recover_execution_effect_for_established_fence(
                &effect_store,
                &RecoveryExecutionEffectReceiptV2::in_progress_for_fence(established.fence())
                    .expect("initial receipt"),
                &successor,
            )
            .expect("effect recovery"),
            RecoveryExecutionEffectRecoveryResult::Recovered
        );
        assert_eq!(
            begin_execution_effect_for_established_fence(&effect_store, &established)
                .expect("stale effect replay"),
            RecoveryExecutionEffectStartResult::FenceMismatch
        );
        assert_eq!(
            begin_execution_effect_for_established_fence(&effect_store, &successor)
                .expect("current effect replay"),
            RecoveryExecutionEffectStartResult::AlreadyInProgressSameAttempt
        );
    }

    #[test]
    fn established_fence_can_start_an_effect_without_reconstructing_the_receipt() {
        let fence_store = FencedExecutionMemoryStore::default();
        let effect_store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&fence_store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        assert_eq!(
            begin_execution_effect_for_established_fence(&effect_store, &established)
                .expect("effect start"),
            RecoveryExecutionEffectStartResult::Started
        );
    }

    #[test]
    fn established_fence_requires_positive_store_acknowledgement() {
        struct IndeterminateFenceStore;

        impl RecoveryExecutionFenceStore for IndeterminateFenceStore {
            type Error = &'static str;

            fn acquire_fence(
                &self,
                _fence: &RecoveryExecutionFenceV1,
            ) -> Result<RecoveryExecutionFenceResult, Self::Error> {
                Ok(RecoveryExecutionFenceResult::Indeterminate)
            }

            fn recover_if_current(
                &self,
                _expected: &RecoveryExecutionFenceV1,
                _successor: &RecoveryExecutionFenceV1,
            ) -> Result<RecoveryExecutionFenceResult, Self::Error> {
                unreachable!("recovery is not reached")
            }

            fn load_fence(
                &self,
                _execution_id: &str,
            ) -> Result<Option<RecoveryExecutionFenceV1>, Self::Error> {
                Ok(None)
            }
        }

        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");

        assert_eq!(
            establish_execution_fence(&IndeterminateFenceStore, &fence).expect("indeterminate"),
            RecoveryExecutionFenceEstablishmentV1::Rejected(
                RecoveryExecutionFenceResult::Indeterminate
            )
        );
    }

    #[test]
    fn established_fence_can_advance_without_recopying_identity_fields() {
        let store = FencedExecutionMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        let successor =
            match recover_established_execution_fence(&store, &established, "attempt-b")
                .expect("recover")
            {
                RecoveryExecutionFenceRecoveryEstablishmentV1::Established(fence) => fence,
                RecoveryExecutionFenceRecoveryEstablishmentV1::Rejected(outcome) => {
                    panic!("unexpected rejection: {outcome:?}")
                }
            };

        assert_eq!(successor.execution_id(), established.execution_id());
        assert_eq!(
            successor.execution_input_snapshot(),
            established.execution_input_snapshot()
        );
        assert_eq!(successor.attempt_id(), "attempt-b");
        assert_eq!(successor.fence_epoch(), established.fence_epoch() + 1);
    }

    #[test]
    fn stale_established_fence_cannot_be_recovered_twice() {
        let store = FencedExecutionMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let established =
            match establish_execution_fence(&store, &initial).expect("establish") {
                RecoveryExecutionFenceEstablishmentV1::Established(fence) => fence,
                _ => panic!("initial establishment must succeed"),
            };

        assert!(matches!(
            recover_established_execution_fence(&store, &established, "attempt-b")
                .expect("first recovery"),
            RecoveryExecutionFenceRecoveryEstablishmentV1::Established(_)
        ));

        assert_eq!(
            recover_established_execution_fence(&store, &established, "attempt-c")
                .expect("stale recovery"),
            RecoveryExecutionFenceRecoveryEstablishmentV1::Rejected(
                RecoveryExecutionFenceResult::StaleExpectedFence
            )
        );
    }

    #[test]
    fn effect_start_can_be_derived_from_one_fence_binding() {
        let store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");

        assert_eq!(
            begin_execution_effect_for_fence(&store, &fence).expect("fenced start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            begin_execution_effect_for_fence(&store, &fence).expect("fenced replay"),
            RecoveryExecutionEffectStartResult::AlreadyInProgressSameAttempt
        );
    }

    #[test]
    fn effect_start_rejects_a_fence_generation_that_is_not_current_in_the_effect_store() {
        let store = ExecutionEffectMemoryStore::default();
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let successor =
            RecoveryExecutionFenceV1::for_recovery(&initial, "attempt-b").expect("successor");

        assert_eq!(
            begin_execution_effect_for_fence(&store, &initial).expect("initial start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            begin_execution_effect_for_fence(&store, &successor).expect("future fence"),
            RecoveryExecutionEffectStartResult::FenceMismatch
        );
    }

    #[test]
    fn effect_receipt_can_be_derived_from_the_current_fence() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let fence = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let receipt = RecoveryExecutionEffectReceiptV2::in_progress_for_fence(&fence)
            .expect("fenced effect receipt");

        assert!(receipt.is_well_formed());
        assert_eq!(receipt.execution_id, fence.execution_id);
        assert_eq!(receipt.execution_input_snapshot, fence.execution_input_snapshot);
        assert_eq!(receipt.attempt_id, fence.attempt_id);
        assert_eq!(receipt.fence_epoch, fence.fence_epoch);
    }

    #[test]
    fn effect_receipt_zero_fence_epoch_fails_closed() {
        let mut receipt = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        receipt.fence_epoch = 0;
        let store = ExecutionEffectMemoryStore::default();

        assert!(!receipt.is_well_formed());
        assert_eq!(
            begin_execution_effect(&store, &receipt).expect("malformed fence"),
            RecoveryExecutionEffectStartResult::MalformedReceipt
        );
    }

    #[test]
    fn fenced_recovery_advances_epoch_and_stales_the_old_owner() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial fence");
        let recovered =
            RecoveryExecutionFenceV1::for_recovery(&initial, "attempt-b").expect("recovery fence");
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            acquire_execution_fence(&store, &initial).expect("initial acquire"),
            RecoveryExecutionFenceResult::Acquired
        );
        assert_eq!(
            recover_execution_fence(&store, &initial, &recovered).expect("recover"),
            RecoveryExecutionFenceResult::Recovered
        );
        assert_eq!(
            check_execution_fence(recovered.fence_epoch, initial.fence_epoch),
            RecoveryExecutionFenceCheck::Stale
        );
        assert_eq!(
            check_execution_fence(recovered.fence_epoch, recovered.fence_epoch),
            RecoveryExecutionFenceCheck::Current
        );
        assert_eq!(
            reconcile_execution_fence(&store, &initial).expect("reconcile stale"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence
        );
        assert_eq!(
            reconcile_execution_fence(&store, &recovered).expect("reconcile current"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedCurrentOwnedByThisAttempt
        );
    }

    #[test]
    fn stale_recovery_cannot_replace_a_newer_owner() {
        let claim_a = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim_a).expect("initial");
        let recovered =
            RecoveryExecutionFenceV1::for_recovery(&initial, "attempt-b").expect("recovery");
        let recovered_again =
            RecoveryExecutionFenceV1::for_recovery(&recovered, "attempt-c").expect("recovery 2");
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            acquire_execution_fence(&store, &initial).expect("acquire"),
            RecoveryExecutionFenceResult::Acquired
        );
        assert_eq!(
            recover_execution_fence(&store, &initial, &recovered).expect("recover"),
            RecoveryExecutionFenceResult::Recovered
        );
        assert_eq!(
            recover_execution_fence(&store, &recovered_again, &recovered).expect("stale"),
            RecoveryExecutionFenceResult::StaleExpectedFence
        );
        assert_eq!(
            reconcile_execution_fence(&store, &initial).expect("stale owner"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence
        );
    }

    #[test]
    fn concurrent_recovery_has_one_winner_for_the_same_expected_fence() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial");
        let recovered_b =
            RecoveryExecutionFenceV1::for_recovery(&initial, "attempt-b").expect("recovery b");
        let recovered_c =
            RecoveryExecutionFenceV1::for_recovery(&initial, "attempt-c").expect("recovery c");
        let store = Arc::new(FencedExecutionMemoryStore::default());

        assert_eq!(
            acquire_execution_fence(store.as_ref(), &initial).expect("acquire"),
            RecoveryExecutionFenceResult::Acquired
        );

        let left_store = Arc::clone(&store);
        let left_expected = initial.clone();
        let left_successor = recovered_b.clone();
        let left = thread::spawn(move || {
            recover_execution_fence(left_store.as_ref(), &left_expected, &left_successor)
        });

        let right_store = Arc::clone(&store);
        let right_expected = initial.clone();
        let right_successor = recovered_c.clone();
        let right = thread::spawn(move || {
            recover_execution_fence(right_store.as_ref(), &right_expected, &right_successor)
        });

        let outcomes = [
            left.join().expect("left join").expect("left result"),
            right.join().expect("right join").expect("right result"),
        ];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == RecoveryExecutionFenceResult::Recovered)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == RecoveryExecutionFenceResult::StaleExpectedFence)
                .count(),
            1
        );
        assert_eq!(
            reconcile_execution_fence(&store, &initial).expect("reconcile old owner"),
            RecoveryExecutionFenceReconciliationOutcome::ObservedStaleFence
        );
    }
    #[test]
    fn fenced_recovery_rejects_fingerprint_reuse_and_non_monotonic_successors() {
        let claim = execution_claim_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let initial = RecoveryExecutionFenceV1::for_initial_claim(&claim).expect("initial");
        let different = execution_claim_fixture(
            "attempt-b",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        );
        let different_fence =
            RecoveryExecutionFenceV1::for_initial_claim(&different).expect("different fence");
        let store = FencedExecutionMemoryStore::default();

        assert_eq!(
            acquire_execution_fence(&store, &initial).expect("acquire"),
            RecoveryExecutionFenceResult::Acquired
        );
        assert_eq!(
            recover_execution_fence(&store, &initial, &different_fence).expect("fingerprint"),
            RecoveryExecutionFenceResult::MalformedFence
        );

        let mut non_monotonic = initial.clone();
        non_monotonic.attempt_id = "attempt-b".into();
        assert_eq!(
            recover_execution_fence(&store, &initial, &non_monotonic).expect("epoch"),
            RecoveryExecutionFenceResult::MalformedFence
        );
    }
    #[test]
    fn concurrent_stale_contenders_race_on_the_same_loaded_version() {
        let (decision, execution, current) = fixture();
        let (first, first_next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let (second, second_next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:01Z");

        assert_ne!(first.next_snapshot_digest, second.next_snapshot_digest);

        let store = Arc::new(ContendedMemoryStore::new(&decision));
        let left_store = Arc::clone(&store);
        let left_decision = decision.clone();
        let left_execution = execution.clone();
        let left_transition = first.clone();
        let left_next = first_next.clone();
        let left = thread::spawn(move || {
            persist_consumption_transition(
                left_store.as_ref(),
                &left_decision,
                &left_execution,
                &left_transition,
                &left_next,
            )
            .expect("left persistence")
        });

        let right_store = Arc::clone(&store);
        let right_decision = decision.clone();
        let right_execution = execution.clone();
        let right_transition = second.clone();
        let right_next = second_next.clone();
        let right = thread::spawn(move || {
            persist_consumption_transition(
                right_store.as_ref(),
                &right_decision,
                &right_execution,
                &right_transition,
                &right_next,
            )
            .expect("right persistence")
        });

        let outcomes = [left.join().expect("left join"), right.join().expect("right join")];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| {
                    **outcome == RecoveryPolicyConsumptionPersistenceOutcome::Committed
                })
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| {
                    **outcome == RecoveryPolicyConsumptionPersistenceOutcome::Conflict
                })
                .count(),
            1
        );

        let current = store.current(&decision);
        assert_eq!(
            current.state,
            RecoveryPolicyConsumptionStateV1::Consumed
        );
        assert!(
            current.digest() == first.next_snapshot_digest
                || current.digest() == second.next_snapshot_digest
        );
    }

    #[test]
    fn missing_state_is_reported_without_cas() {
        struct MissingStore {
            cas_called: Mutex<bool>,
        }

        impl RecoveryPolicyConsumptionStore for MissingStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Ok(None)
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                *self.cas_called.lock().expect("cas lock") = true;
                Ok(RecoveryPolicyConsumptionCasResult::Committed)
            }
        }

        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let store = MissingStore {
            cas_called: Mutex::new(false),
        };

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("missing-state result"),
            RecoveryPolicyConsumptionPersistenceOutcome::MissingState
        );
        assert!(!*store.cas_called.lock().expect("cas lock"));
    }

    #[test]
    fn cas_rejection_is_conflict_and_not_commit() {
        struct RejectingStore {
            loaded: RecoveryPolicyConsumptionSnapshotV1,
        }

        impl RecoveryPolicyConsumptionStore for RejectingStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Ok(Some(self.loaded.clone()))
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                Ok(RecoveryPolicyConsumptionCasResult::NotCurrent)
            }
        }

        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        assert_eq!(
            persist_consumption_transition(
                &RejectingStore { loaded: current },
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("CAS rejection result"),
            RecoveryPolicyConsumptionPersistenceOutcome::Conflict
        );
    }

    #[test]
    fn indeterminate_commit_is_not_reported_as_success() {
        struct IndeterminateStore {
            loaded: RecoveryPolicyConsumptionSnapshotV1,
        }

        impl RecoveryPolicyConsumptionStore for IndeterminateStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Ok(Some(self.loaded.clone()))
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                Ok(RecoveryPolicyConsumptionCasResult::Indeterminate)
            }
        }

        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");

        let outcome = persist_consumption_transition(
            &IndeterminateStore { loaded: current },
            &decision,
            &execution,
            &transition,
            &next,
        )
        .expect("indeterminate CAS result");

        assert_eq!(
            outcome,
            RecoveryPolicyConsumptionPersistenceOutcome::CommitIndeterminate
        );
        assert_ne!(
            outcome,
            RecoveryPolicyConsumptionPersistenceOutcome::Committed
        );
    }

    #[test]
    fn reconciliation_recovers_a_commit_followed_by_an_indeterminate_ack() {
        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let store = IndeterminateOutcomeStore::new(&decision, true);

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("indeterminate persistence result"),
            RecoveryPolicyConsumptionPersistenceOutcome::CommitIndeterminate
        );

        assert_eq!(
            reconcile_indeterminate_consumption(&store, &decision, &transition, &next)
                .expect("reconcile committed successor"),
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedCommitted
        );
    }

    #[test]
    fn reconciliation_recovers_an_indeterminate_ack_without_a_commit() {
        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let store = IndeterminateOutcomeStore::new(&decision, false);

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("indeterminate persistence result"),
            RecoveryPolicyConsumptionPersistenceOutcome::CommitIndeterminate
        );

        assert_eq!(
            reconcile_indeterminate_consumption(&store, &decision, &transition, &next)
                .expect("reconcile expected pre-state"),
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedExpected
        );
    }

    #[test]
    fn reconciliation_observes_exact_successor_without_mutation() {
        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");

        let store = MemoryStore::new(&decision);
        assert_eq!(
            reconcile_indeterminate_consumption(&store, &decision, &transition, &next)
                .expect("expected pre-state"),
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedExpected
        );

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("commit"),
            RecoveryPolicyConsumptionPersistenceOutcome::Committed
        );

        assert_eq!(
            reconcile_indeterminate_consumption(&store, &decision, &transition, &next)
                .expect("committed successor"),
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedCommitted
        );
    }

    #[test]
    fn reconciliation_classifies_a_different_successor_without_guessing_why() {
        let (decision, execution, current) = fixture();
        let (first, first_next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let (second, second_next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:01Z");
        let store = MemoryStore::new(&decision);

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &first,
                &first_next,
            )
            .expect("first commit"),
            RecoveryPolicyConsumptionPersistenceOutcome::Committed
        );

        assert_eq!(
            reconcile_indeterminate_consumption(&store, &decision, &second, &second_next)
                .expect("different successor"),
            RecoveryPolicyConsumptionReconciliationOutcome::ObservedDifferentState
        );
    }

    #[test]
    fn reconciliation_requires_successor_to_match_transition_metadata() {
        let (decision, execution, current) = fixture();
        let (mut transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        transition.execution_id = "different-execution".into();

        let store = MemoryStore::new(&decision);
        assert_eq!(
            reconcile_indeterminate_consumption(&store, &decision, &transition, &next)
                .expect("binding result"),
            RecoveryPolicyConsumptionReconciliationOutcome::InvalidTransition
        );
    }

    #[test]
    fn reconciliation_validates_inputs_before_reading_store() {
        let (decision, execution, current) = fixture();
        let (mut transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        transition.next_snapshot_digest = "sha256:tampered".into();

        struct NoReadStore;

        impl RecoveryPolicyConsumptionStore for NoReadStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Err("read must not be reached")
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                unreachable!("reconciliation never performs CAS")
            }
        }

        assert_eq!(
            reconcile_indeterminate_consumption(&NoReadStore, &decision, &transition, &next)
                .expect("invalid transition outcome"),
            RecoveryPolicyConsumptionReconciliationOutcome::InvalidTransition
        );
    }

    #[test]
    fn stored_decision_binding_mismatch_blocks_cas() {
        struct MismatchedStore {
            loaded: RecoveryPolicyConsumptionSnapshotV1,
            cas_called: Mutex<bool>,
        }

        impl RecoveryPolicyConsumptionStore for MismatchedStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Ok(Some(self.loaded.clone()))
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                *self.cas_called.lock().expect("cas lock") = true;
                Ok(RecoveryPolicyConsumptionCasResult::Committed)
            }
        }

        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let mut loaded = current;
        loaded.decision_digest =
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
        let store = MismatchedStore {
            loaded,
            cas_called: Mutex::new(false),
        };

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("stored-decision mismatch result"),
            RecoveryPolicyConsumptionPersistenceOutcome::InvalidTransition
        );
        assert!(!*store.cas_called.lock().expect("cas lock"));
    }

    #[test]
    fn malformed_stored_state_is_rejected_before_conflict_classification() {
        struct MalformedStore {
            state: RecoveryPolicyConsumptionSnapshotV1,
        }

        impl RecoveryPolicyConsumptionStore for MalformedStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Ok(Some(self.state.clone()))
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                Err("cas must not be reached")
            }
        }

        let (decision, execution, _current) = fixture();
        let mut malformed = RecoveryPolicyConsumptionSnapshotV1::for_decision(&decision);
        malformed.claim_ceiling.clear();
        let (transition, next) = transition_fixture(
            &decision,
            &execution,
            &RecoveryPolicyConsumptionSnapshotV1::for_decision(&decision),
            "2026-10-02T08:00:00Z",
        );

        assert_eq!(
            persist_consumption_transition(
                &MalformedStore { state: malformed },
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("stored-state validation result"),
            RecoveryPolicyConsumptionPersistenceOutcome::MalformedStoredState
        );
    }

    #[test]
    fn replay_does_not_mutate_consumed_state() {
        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let store = MemoryStore::new(&decision);

        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("first persistence"),
            RecoveryPolicyConsumptionPersistenceOutcome::Committed
        );

        let consumed = store.current(&decision);
        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("replay"),
            RecoveryPolicyConsumptionPersistenceOutcome::ReplayDetected
        );
        assert_eq!(store.current(&decision), consumed);
    }

    #[test]
    fn persistence_failure_is_not_reported_as_commit() {
        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");

        assert!(matches!(
            persist_consumption_transition(
                &BrokenStore,
                &decision,
                &execution,
                &transition,
                &next,
            ),
            Err(RecoveryPolicyConsumptionPersistenceError::Store("load failed"))
        ));
    }

    #[test]
    fn malformed_inputs_never_reach_store() {
        struct CountingStore {
            loaded: Mutex<bool>,
            cas_called: Mutex<bool>,
        }

        impl RecoveryPolicyConsumptionStore for CountingStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                *self.loaded.lock().expect("load lock") = true;
                Err("load must not be called")
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                *self.cas_called.lock().expect("cas lock") = true;
                Err("cas must not be called")
            }
        }

        let (decision, execution, current) = fixture();
        let (transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let store = CountingStore {
            loaded: Mutex::new(false),
            cas_called: Mutex::new(false),
        };

        let mut malformed_decision = decision.clone();
        malformed_decision.consumer.clear();
        assert_eq!(
            persist_consumption_transition(
                &store,
                &malformed_decision,
                &execution,
                &transition,
                &next,
            )
            .expect("malformed decision outcome"),
            RecoveryPolicyConsumptionPersistenceOutcome::MalformedDecision
        );

        let mut malformed_transition = transition.clone();
        malformed_transition.execution_id.clear();
        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &malformed_transition,
                &next,
            )
            .expect("malformed transition outcome"),
            RecoveryPolicyConsumptionPersistenceOutcome::MalformedTransition
        );

        let mut malformed_next = next.clone();
        malformed_next.claim_ceiling.clear();
        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &malformed_next,
            )
            .expect("malformed successor outcome"),
            RecoveryPolicyConsumptionPersistenceOutcome::MalformedSuccessor
        );

        assert!(!*store.loaded.lock().expect("load lock"));
        assert!(!*store.cas_called.lock().expect("cas lock"));
    }

    #[test]
    fn invalid_transition_never_reaches_cas() {
        struct RecordingStore {
            loaded: RecoveryPolicyConsumptionSnapshotV1,
            cas_called: Mutex<bool>,
        }

        impl RecoveryPolicyConsumptionStore for RecordingStore {
            type Error = &'static str;

            fn load(
                &self,
                _decision_digest: &str,
            ) -> Result<Option<RecoveryPolicyConsumptionSnapshotV1>, Self::Error> {
                Ok(Some(self.loaded.clone()))
            }

            fn compare_and_set(
                &self,
                _decision_digest: &str,
                _expected_snapshot_digest: &str,
                _next: &RecoveryPolicyConsumptionSnapshotV1,
            ) -> Result<RecoveryPolicyConsumptionCasResult, Self::Error> {
                *self.cas_called.lock().expect("cas lock") = true;
                Ok(RecoveryPolicyConsumptionCasResult::Committed)
            }
        }

        let (decision, execution, current) = fixture();
        let (mut transition, next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        transition.next_snapshot_digest = "sha256:tampered".into();

        let store = RecordingStore {
            loaded: current,
            cas_called: Mutex::new(false),
        };
        assert_eq!(
            persist_consumption_transition(
                &store,
                &decision,
                &execution,
                &transition,
                &next,
            )
            .expect("validation result"),
            RecoveryPolicyConsumptionPersistenceOutcome::InvalidTransition
        );
        assert!(!*store.cas_called.lock().expect("cas lock"));
    }
}
