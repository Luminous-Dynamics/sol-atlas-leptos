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
    Committed,
    Conflict,
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

/// A minimal compare-and-set store required by the persistence boundary.
///
/// Implementations MUST make compare_and_set atomic with respect to all
/// contenders for the same authorization key. Returning true means the exact
/// supplied successor was committed; returning false means the expected
/// version was no longer current and the caller must not report success.
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
    ) -> Result<bool, Self::Error>;
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

    let committed = store
        .compare_and_set(
            &decision_digest,
            &transition.expected_snapshot_digest,
            next,
        )
        .map_err(RecoveryPolicyConsumptionPersistenceError::Store)?;

    if committed {
        Ok(RecoveryPolicyConsumptionPersistenceOutcome::Committed)
    } else {
        Ok(RecoveryPolicyConsumptionPersistenceOutcome::Conflict)
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
        ) -> Result<bool, Self::Error> {
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
        ) -> Result<bool, Self::Error> {
            let mut values = self.values.lock().map_err(|_| "poisoned")?;
            let Some(current) = values.get(decision_digest) else {
                return Ok(false);
            };
            if current.digest() != expected_snapshot_digest {
                return Ok(false);
            }
            values.insert(decision_digest.to_owned(), next.clone());
            Ok(true)
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
    fn stale_contenders_allow_exactly_one_commit() {
        let (decision, execution, current) = fixture();
        let (first, first_next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:00Z");
        let (second, second_next) =
            transition_fixture(&decision, &execution, &current, "2026-10-02T08:00:01Z");

        assert_ne!(first.next_snapshot_digest, second.next_snapshot_digest);

        let store = Arc::new(MemoryStore::new(&decision));
        let barrier = Arc::new(Barrier::new(3));

        let left_store = Arc::clone(&store);
        let left_barrier = Arc::clone(&barrier);
        let left_decision = decision.clone();
        let left_execution = execution.clone();
        let left_transition = first.clone();
        let left_next = first_next.clone();
        let left = thread::spawn(move || {
            left_barrier.wait();
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
        let right_barrier = Arc::clone(&barrier);
        let right_decision = decision.clone();
        let right_execution = execution.clone();
        let right_transition = second.clone();
        let right_next = second_next.clone();
        let right = thread::spawn(move || {
            right_barrier.wait();
            persist_consumption_transition(
                right_store.as_ref(),
                &right_decision,
                &right_execution,
                &right_transition,
                &right_next,
            )
            .expect("right persistence")
        });

        barrier.wait();

        let outcomes = [left.join().expect("left join"), right.join().expect("right join")];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome
                    == RecoveryPolicyConsumptionPersistenceOutcome::Committed)
                .count(),
            1
        );
        assert!(outcomes.iter().any(|outcome| {
            matches!(
                outcome,
                RecoveryPolicyConsumptionPersistenceOutcome::Conflict
                    | RecoveryPolicyConsumptionPersistenceOutcome::ReplayDetected
            )
        }));

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
            ) -> Result<bool, Self::Error> {
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
            ) -> Result<bool, Self::Error> {
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
            ) -> Result<bool, Self::Error> {
                *self.cas_called.lock().expect("cas lock") = true;
                Ok(true)
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
