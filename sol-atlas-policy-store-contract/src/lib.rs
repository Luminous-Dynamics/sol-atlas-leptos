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

use sol_atlas_core::{
    RecoveryExecution, RecoveryPolicyConsumptionSnapshotV1,
    RecoveryPolicyConsumptionStateV1, RecoveryPolicyConsumptionTransitionV1,
    RecoveryPolicyDecisionSnapshotV1,
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
}

/// Result of atomically beginning an external effect.
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

    fn begin_effect(
        &self,
        receipt: &RecoveryExecutionEffectReceiptV2,
    ) -> Result<RecoveryExecutionEffectStartResult, Self::Error>;

    fn complete_effect(
        &self,
        execution_id: &str,
        execution_input_snapshot: &str,
        attempt_id: &str,
        fence_epoch: u64,
        completed: &RecoveryExecutionEffectReceiptV2,
    ) -> Result<RecoveryExecutionEffectCompletionResult, Self::Error>;

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
    successor: &RecoveryExecutionEffectReceiptV2,
) -> Result<
    RecoveryExecutionEffectRecoveryResult,
    RecoveryPolicyConsumptionPersistenceError<S::Error>,
>
where
    S: RecoveryExecutionEffectStore,
{
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

    store
        .recover_effect_if_current(expected, successor)
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)
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

    fn acquire_fence(
        &self,
        fence: &RecoveryExecutionFenceV1,
    ) -> Result<RecoveryExecutionFenceResult, Self::Error>;

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
    if !fence.is_well_formed() {
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
#[cfg(test)]
mod tests {
    use super::*;
    use sol_atlas_core::{
        CapabilityId, CapabilityState, RecoveryPolicyDecisionV1, RecoveryExecution,
    };
    use std::sync::{Arc, Barrier};
    use std::thread;

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
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&receipt.execution_id) else {
                values.insert(receipt.execution_id.clone(), receipt.clone());
                return Ok(RecoveryExecutionEffectStartResult::Started);
            };

            if current.execution_input_snapshot != receipt.execution_input_snapshot {
                return Ok(RecoveryExecutionEffectStartResult::FingerprintMismatch);
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
                (RecoveryExecutionEffectStateV1::Succeeded, _ ) => {
                    RecoveryExecutionEffectStartResult::AlreadySucceededSameRequest
                }
                (RecoveryExecutionEffectStateV1::Failed, _ ) => {
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
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(execution_id) else {
                return Ok(RecoveryExecutionEffectCompletionResult::MissingReceipt);
            };

            if current.execution_input_snapshot != execution_input_snapshot {
                return Ok(RecoveryExecutionEffectCompletionResult::FingerprintMismatch);
            }

            if current.fence_epoch != fence_epoch || current.fence_epoch != completed.fence_epoch {
                return Ok(RecoveryExecutionEffectCompletionResult::FenceMismatch);
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
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&expected.execution_id) else {
                return Ok(RecoveryExecutionEffectRecoveryResult::MissingReceipt);
            };

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

    fn effect_receipt_fixture(
        attempt_id: &str,
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
    fn fenced_effect_recovery_advances_receipt_epoch_and_stales_old_owner() {
        let store = ExecutionEffectMemoryStore::default();
        let started = effect_receipt_fixture(
            "attempt-a",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        let successor = RecoveryExecutionEffectReceiptV2 {
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
        assert_eq!(
            recover_execution_effect(&store, &started, &successor).expect("recover"),
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
        let successor = RecoveryExecutionEffectReceiptV2 {
            attempt_id: "attempt-b".into(),
            fence_epoch: 2,
            ..started.clone()
        };
        let store = IndeterminateRecoveryStore {
            inner: ExecutionEffectMemoryStore::default(),
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );
        assert_eq!(
            recover_execution_effect(&store, &started, &successor).expect("unknown recovery"),
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
        let successor_b = RecoveryExecutionEffectReceiptV2 {
            attempt_id: "attempt-b".into(),
            fence_epoch: 2,
            ..started.clone()
        };
        let successor_c = RecoveryExecutionEffectReceiptV2 {
            attempt_id: "attempt-c".into(),
            fence_epoch: 2,
            ..started.clone()
        };

        assert_eq!(
            begin_execution_effect(&store, &started).expect("start"),
            RecoveryExecutionEffectStartResult::Started
        );

        let left_store = Arc::clone(&store);
        let left_expected = started.clone();
        let left_successor = successor_b.clone();
        let left = thread::spawn(move || {
            recover_execution_effect(left_store.as_ref(), &left_expected, &left_successor)
        });

        let right_store = Arc::clone(&store);
        let right_expected = started.clone();
        let right_successor = successor_c.clone();
        let right = thread::spawn(move || {
            recover_execution_effect(right_store.as_ref(), &right_expected, &right_successor)
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
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&claim.execution_id) else {
                values.insert(claim.execution_id.clone(), claim.clone());
                return Ok(RecoveryExecutionClaimResult::Acquired);
            };

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
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&fence.execution_id) else {
                values.insert(fence.execution_id.clone(), fence.clone());
                return Ok(RecoveryExecutionFenceResult::Acquired);
            };

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
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(&expected.execution_id) else {
                return Ok(RecoveryExecutionFenceResult::MissingCurrentFence);
            };

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
