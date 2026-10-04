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

#[cfg(test)]
mod tests {
    use super::*;
    use sol_atlas_core::{
        CapabilityId, CapabilityState, RecoveryPolicyDecisionV1, RecoveryExecution,
    };
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[derive(Default)]
    struct MemoryStore {
        values: Mutex<BTreeMap<String, RecoveryPolicyConsumptionSnapshotV1>>,
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
            input_snapshot: "sha256:execution-input".into(),
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
        loaded.decision_digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
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
