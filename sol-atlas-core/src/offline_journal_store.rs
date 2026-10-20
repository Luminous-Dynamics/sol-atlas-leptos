// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Transaction-shaped persistence contract for the offline operation journal.
//!
//! This module defines the guarantees a durable backend must provide; it is not
//! itself a storage implementation. In particular, implementations must commit
//! each state transition atomically before returning success. Callers must not
//! send a request until `begin_attempt` has returned successfully.
//!
//! The asynchronous API supports browser IndexedDB and native storage adapters
//! without requiring the futures to be `Send` (which is not generally possible
//! for browser-bound JavaScript objects).

use std::fmt;

use crate::offline_journal::{
    JournalEntry, JournalError, JournalState, OperationEnvelope, ReconcileOutcome, ServiceReply,
};

/// A page of pending operations, in lexicographic operation-ID order.
///
/// The cursor is the last returned operation ID, and is present only when the
/// backend knows another matching row exists. Cursors are not snapshot tokens:
/// after completing a scan, callers should start the next recovery pass with no
/// cursor so newly-pending operations cannot remain hidden behind an old cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalPage {
    pub entries: Vec<JournalEntry>,
    pub next_after_operation_id: Option<String>,
}

/// Persistence failures that adapters must report without inventing a state
/// transition. Backend-specific failures are preserved as `Backend(E)`.
#[derive(Debug)]
pub enum JournalStoreError<E> {
    /// The operation ID already exists with a different immutable envelope.
    OperationIdCollision,
    /// The idempotency key is already bound to a different operation.
    IdempotencyKeyCollision,
    /// No operation with the requested ID exists.
    OperationNotFound,
    /// A page size of zero cannot make forward progress.
    InvalidPageSize,
    /// Validation or state-machine failure from the pure journal reducer.
    Journal(JournalError),
    /// Database, quota, transaction-abort, serialization, or other adapter error.
    Backend(E),
}

impl<E: fmt::Display> fmt::Display for JournalStoreError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationIdCollision => {
                formatter.write_str("operation ID is already bound to a different envelope")
            }
            Self::IdempotencyKeyCollision => {
                formatter.write_str("idempotency key is already bound to another operation")
            }
            Self::OperationNotFound => formatter.write_str("journal operation was not found"),
            Self::InvalidPageSize => {
                formatter.write_str("journal page size must be greater than zero")
            }
            Self::Journal(error) => write!(formatter, "journal transition failed: {error}"),
            Self::Backend(error) => write!(formatter, "journal storage backend failed: {error}"),
        }
    }
}

impl<E> std::error::Error for JournalStoreError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Journal(error) => Some(error),
            Self::Backend(error) => Some(error),
            _ => None,
        }
    }
}

/// Durable journal storage port.
///
/// Implementations must satisfy the transaction semantics documented on each
/// method. A method returning `Ok` means the transaction committed according
/// to the backend's advertised durability policy; an aborted or indeterminate
/// transaction must return an error and be reconciled by reading the stored
/// state, never by assuming the write succeeded.
///
/// No method accepts an arbitrary caller-mutated `JournalEntry`. State changes
/// are requested through reducer operations so adapters can validate and commit
/// a state transition inside one transaction.
#[allow(async_fn_in_trait)]
pub trait OfflineJournalStore {
    /// Storage-specific error type.
    type BackendError: std::error::Error + 'static;

    /// Atomically insert a validated `Prepared` entry and enforce global
    /// uniqueness for both operation ID and idempotency key.
    ///
    /// Repeating this call with the same operation ID and byte-for-byte equal
    /// immutable envelope is idempotent: return the existing entry at its
    /// current state without resetting attempts or overwriting a receipt.
    /// Reusing that operation ID with a different envelope, or reusing its
    /// idempotency key for another operation, must fail without changing data.
    async fn insert_prepared(
        &mut self,
        envelope: OperationEnvelope,
    ) -> Result<JournalEntry, JournalStoreError<Self::BackendError>>;

    /// Atomically persist `OutcomeUnknown` and increment the attempt count
    /// *before* the caller performs network I/O.
    ///
    /// A successful return is the only permission this API gives a caller to
    /// dispatch. If the process crashes immediately after the commit, recovery
    /// sees `OutcomeUnknown` and retries the same immutable envelope. If the
    /// commit fails or is indeterminate, the caller must not dispatch.
    async fn begin_attempt(
        &mut self,
        operation_id: &str,
    ) -> Result<JournalEntry, JournalStoreError<Self::BackendError>>;

    /// Atomically apply a service reply through `JournalEntry::reconcile`.
    ///
    /// The adapter must load the current row, reconcile a private copy, and
    /// commit the resulting state in one atomic transaction. A validation error
    /// (including a mismatched receipt binding) leaves the stored row unchanged.
    /// Duplicate identical terminal replies may return their duplicate outcome
    /// without rewriting the row. Concurrent writers must use compare-and-swap
    /// or transaction isolation so an update based on stale state cannot win.
    async fn apply_reply(
        &mut self,
        operation_id: &str,
        reply: &ServiceReply,
    ) -> Result<ReconcileOutcome, JournalStoreError<Self::BackendError>>;

    /// Read one operation. Reads must not mutate journal state.
    async fn load(
        &self,
        operation_id: &str,
    ) -> Result<Option<JournalEntry>, JournalStoreError<Self::BackendError>>;

    /// List only `Prepared` and `OutcomeUnknown` entries, ordered by
    /// operation ID. Results must be stable within each query and bounded by
    /// `limit`; a zero limit is an error. This is a pending/retry view, not an
    /// instruction to send every returned row without calling `begin_attempt`.
    async fn list_pending(
        &self,
        after_operation_id: Option<&str>,
        limit: usize,
    ) -> Result<JournalPage, JournalStoreError<Self::BackendError>>;
}

/// Shared state predicate for storage adapters' pending-query implementations.
pub fn is_pending(state: &JournalState) -> bool {
    matches!(state, JournalState::Prepared | JournalState::OutcomeUnknown)
}

#[cfg(test)]
mod tests {
    use super::is_pending;
    use crate::offline_journal::JournalState;

    #[test]
    fn pending_filter_excludes_all_terminal_and_review_states() {
        assert!(is_pending(&JournalState::Prepared));
        assert!(is_pending(&JournalState::OutcomeUnknown));
        assert!(!is_pending(&JournalState::Received {
            receipt_ref: "receipt".to_owned(),
            applied_revision: 1,
        }));
        assert!(!is_pending(&JournalState::Rejected {
            reason_code: "invalid".to_owned(),
        }));
        assert!(!is_pending(&JournalState::ConflictNeedsReview {
            current_revision: 2,
            reason_code: "concurrent_edit".to_owned(),
        }));
    }
}
