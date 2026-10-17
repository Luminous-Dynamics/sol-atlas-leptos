use sol_atlas_policy_store_contract::{
    RecoveryExecutionFencedResource, RecoveryExecutionProtectedMutationReconciler,
    RecoveryExecutionProtectedMutationReconciliationOutcome,
    RecoveryExecutionProtectedMutationResult, RecoveryExecutionProtectedMutationV1,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct ReferenceFencedResource {
    state: Mutex<ReferenceResourceState>,
}

#[derive(Default)]
struct ReferenceResourceState {
    current_epoch: u64,
    applied: BTreeMap<String, AppliedMutation>,
    idempotency_index: BTreeMap<String, String>,
}

#[derive(Clone)]
struct AppliedMutation {
    execution_input_snapshot: String,
    attempt_id: String,
    fence_epoch: u64,
    idempotency_key: Option<String>,
}

impl ReferenceFencedResource {
    pub fn set_epoch(&self, epoch: u64) {
        self.state
            .lock()
            .expect("resource state lock")
            .current_epoch = epoch;
    }

    pub fn current_epoch(&self) -> u64 {
        self.state
            .lock()
            .expect("resource state lock")
            .current_epoch
    }
}

impl RecoveryExecutionFencedResource for ReferenceFencedResource {
    type Error = &'static str;

    fn mutate_if_fence_is_current(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> Result<RecoveryExecutionProtectedMutationResult, Self::Error> {
        if mutation.execution_id.is_empty()
            || mutation.execution_input_snapshot.len() != 71
            || !mutation.execution_input_snapshot.starts_with("sha256:")
            || mutation.attempt_id.is_empty()
            || mutation.fence_epoch == 0
            || mutation
                .idempotency_key
                .as_deref()
                .is_some_and(str::is_empty)
        {
            return Ok(RecoveryExecutionProtectedMutationResult::RejectedInvalidFence);
        }

        let mut state = self.state.lock().map_err(|_| "poisoned")?;
        let current_epoch = state.current_epoch;

        if current_epoch == 0 {
            return Ok(RecoveryExecutionProtectedMutationResult::RejectedInvalidFence);
        }
        if mutation.fence_epoch < current_epoch {
            return Ok(RecoveryExecutionProtectedMutationResult::RejectedStaleFence);
        }
        if mutation.fence_epoch > current_epoch {
            return Ok(RecoveryExecutionProtectedMutationResult::RejectedFutureFence);
        }

        if let Some(idempotency_key) = mutation.idempotency_key.as_deref() {
            if let Some(existing_execution_id) = state.idempotency_index.get(idempotency_key) {
                if existing_execution_id != &mutation.execution_id {
                    return Ok(RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch);
                }
            }
        }

        let key = mutation.execution_id.clone();
        if let Some(existing) = state.applied.get(&key) {
            if existing.execution_input_snapshot != mutation.execution_input_snapshot {
                return Ok(RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch);
            }

            if existing.fence_epoch == mutation.fence_epoch
                && existing.attempt_id == mutation.attempt_id
            {
                if existing.idempotency_key != mutation.idempotency_key {
                    return Ok(RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch);
                }

                return Ok(RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest);
            }

            if existing.fence_epoch == mutation.fence_epoch {
                return Ok(RecoveryExecutionProtectedMutationResult::RejectedOtherAttempt);
            }

            return Ok(RecoveryExecutionProtectedMutationResult::RejectedStaleFence);
        }

        state.applied.insert(
            key,
            AppliedMutation {
                execution_input_snapshot: mutation.execution_input_snapshot.clone(),
                attempt_id: mutation.attempt_id.clone(),
                fence_epoch: mutation.fence_epoch,
                idempotency_key: mutation.idempotency_key.clone(),
            },
        );
        if let Some(idempotency_key) = mutation.idempotency_key.as_deref() {
            state
                .idempotency_index
                .insert(idempotency_key.to_owned(), mutation.execution_id.clone());
        }
        Ok(RecoveryExecutionProtectedMutationResult::Applied)
    }
}

impl RecoveryExecutionProtectedMutationReconciler for ReferenceFencedResource {
    type Error = &'static str;

    fn reconcile_mutation(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> Result<RecoveryExecutionProtectedMutationReconciliationOutcome, Self::Error> {
        if mutation.execution_id.is_empty()
            || mutation.execution_input_snapshot.len() != 71
            || !mutation.execution_input_snapshot.starts_with("sha256:")
            || mutation.attempt_id.is_empty()
            || mutation.fence_epoch == 0
            || mutation
                .idempotency_key
                .as_deref()
                .is_some_and(str::is_empty)
        {
            return Ok(RecoveryExecutionProtectedMutationReconciliationOutcome::InvalidState);
        }

        let state = self.state.lock().map_err(|_| "poisoned")?;
        if let Some(idempotency_key) = mutation.idempotency_key.as_deref() {
            if let Some(existing_execution_id) = state.idempotency_index.get(idempotency_key) {
                if existing_execution_id != &mutation.execution_id {
                    return Ok(
                        RecoveryExecutionProtectedMutationReconciliationOutcome::
                            ObservedDifferentRequest,
                    );
                }
            }
        }

        let Some(existing) = state.applied.get(&mutation.execution_id) else {
            return Ok(RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedNotApplied);
        };

        if existing.execution_input_snapshot == mutation.execution_input_snapshot
            && existing.attempt_id == mutation.attempt_id
            && existing.fence_epoch == mutation.fence_epoch
            && existing.idempotency_key == mutation.idempotency_key
        {
            return Ok(
                RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest,
            );
        }

        Ok(RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedDifferentRequest)
    }
}

#[derive(Default)]
pub struct LostAckResource {
    inner: ReferenceFencedResource,
    lose_next_ack: Mutex<bool>,
}

impl LostAckResource {
    pub fn arm_lost_ack(&self) {
        *self.lose_next_ack.lock().expect("lost-ack state lock") = true;
    }

    pub fn set_epoch(&self, epoch: u64) {
        self.inner.set_epoch(epoch);
    }
}

impl RecoveryExecutionFencedResource for LostAckResource {
    type Error = &'static str;

    fn mutate_if_fence_is_current(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> Result<RecoveryExecutionProtectedMutationResult, Self::Error> {
        let result = self.inner.mutate_if_fence_is_current(mutation)?;
        let mut lose_next_ack = self.lose_next_ack.lock().map_err(|_| "poisoned")?;
        if *lose_next_ack {
            *lose_next_ack = false;
            return Ok(RecoveryExecutionProtectedMutationResult::Indeterminate);
        }
        Ok(result)
    }
}

impl RecoveryExecutionProtectedMutationReconciler for LostAckResource {
    type Error = &'static str;

    fn reconcile_mutation(
        &self,
        mutation: &RecoveryExecutionProtectedMutationV1,
    ) -> Result<RecoveryExecutionProtectedMutationReconciliationOutcome, Self::Error> {
        self.inner.reconcile_mutation(mutation)
    }
}
