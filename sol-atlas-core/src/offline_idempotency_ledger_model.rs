// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Executable in-memory reference model for the authoritative idempotency contract.
//!
//! This model specifies key reservation, request-fingerprint conflicts, in-flight
//! duplicates, stale reservation fencing, and exact outcome replay. It is not a
//! durable server ledger and must not be used as production persistence. A real
//! service must implement the same state machine with database-enforced unique
//! constraints and atomically commit the domain effect plus outcome (or use an
//! outbox for external effects).

use std::{collections::BTreeMap, error::Error, fmt};

use sha2::{Digest, Sha256};

use crate::{
    offline_idempotency_fingerprint::{
        REQUEST_FINGERPRINT_SCHEMA_VERSION, canonical_request_fingerprint_sha256,
    },
    offline_journal::OperationEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedIdempotencyKey {
    tenant_scope: String,
    operation_namespace: String,
    idempotency_key: String,
}

impl ScopedIdempotencyKey {
    pub fn new(
        tenant_scope: &str,
        operation_namespace: &str,
        idempotency_key: &str,
    ) -> Result<Self, LedgerModelError> {
        if tenant_scope.trim().is_empty() {
            return Err(LedgerModelError::EmptyScope("tenant_scope"));
        }
        if operation_namespace.trim().is_empty() {
            return Err(LedgerModelError::EmptyScope("operation_namespace"));
        }
        if idempotency_key.trim().is_empty() {
            return Err(LedgerModelError::EmptyScope("idempotency_key"));
        }
        Ok(Self {
            tenant_scope: tenant_scope.to_owned(),
            operation_namespace: operation_namespace.to_owned(),
            idempotency_key: idempotency_key.to_owned(),
        })
    }

    pub fn tenant_scope(&self) -> &str {
        &self.tenant_scope
    }

    pub fn operation_namespace(&self) -> &str {
        &self.operation_namespace
    }

    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredOutcome {
    status_code: u16,
    receipt_id: String,
    response_body: Vec<u8>,
    response_body_sha256: String,
}

impl StoredOutcome {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    /// Exact response bytes to replay after a lost response.
    pub fn response_body(&self) -> &[u8] {
        &self.response_body
    }

    pub fn response_body_sha256(&self) -> &str {
        &self.response_body_sha256
    }

    fn new(
        status_code: u16,
        receipt_id: String,
        response_body: Vec<u8>,
    ) -> Result<Self, LedgerModelError> {
        if !(100..=599).contains(&status_code) {
            return Err(LedgerModelError::InvalidOutcome("invalid_status_code"));
        }
        if receipt_id.trim().is_empty() {
            return Err(LedgerModelError::InvalidOutcome("empty_receipt_id"));
        }
        let digest = Sha256::digest(&response_body);
        Ok(Self {
            status_code,
            receipt_id,
            response_body_sha256: lower_hex(&digest),
            response_body,
        })
    }
}

/// Opaque, in-process reservation authority. A database-backed service needs
/// its own transactional fencing/version primitive rather than this local counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationTicket {
    key: ScopedIdempotencyKey,
    request_fingerprint_schema_version: u16,
    request_fingerprint_sha256: String,
    generation: u64,
}

impl ReservationTicket {
    pub fn key(&self) -> &ScopedIdempotencyKey {
        &self.key
    }

    pub fn request_fingerprint_schema_version(&self) -> u16 {
        self.request_fingerprint_schema_version
    }

    pub fn request_fingerprint_sha256(&self) -> &str {
        &self.request_fingerprint_sha256
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReserveResult {
    /// This call created the one current reservation for this scoped key.
    Reserved(ReservationTicket),
    /// Another request with the same scoped key/fingerprint is still in progress.
    AlreadyInProgress,
    /// The effect/outcome was already committed; return this exact outcome.
    Replay(StoredOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitResult {
    Committed(StoredOutcome),
    /// A repeated completion of the same ticket/outcome is idempotent.
    AlreadyCommitted(StoredOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerModelError {
    EmptyScope(&'static str),
    InvalidEnvelope(String),
    InvalidFingerprint(String),
    RequestFingerprintConflict,
    StaleReservation,
    ConflictingFinalization,
    SequenceExhausted,
    InvalidOutcome(&'static str),
}

impl fmt::Display for LedgerModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScope(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidEnvelope(reason) => {
                write!(formatter, "invalid operation envelope: {reason}")
            }
            Self::InvalidFingerprint(reason) => {
                write!(formatter, "cannot fingerprint request: {reason}")
            }
            Self::RequestFingerprintConflict => formatter
                .write_str("idempotency key is already bound to a different request fingerprint"),
            Self::StaleReservation => {
                formatter.write_str("reservation is stale, missing, or no longer authoritative")
            }
            Self::ConflictingFinalization => formatter
                .write_str("completed idempotency record cannot be finalized with another outcome"),
            Self::SequenceExhausted => {
                formatter.write_str("reservation generation counter exhausted")
            }
            Self::InvalidOutcome(reason) => {
                write!(formatter, "invalid authoritative outcome: {reason}")
            }
        }
    }
}

impl Error for LedgerModelError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecordState {
    InProgress { generation: u64 },
    Completed(StoredOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LedgerRecord {
    fingerprint_schema_version: u16,
    fingerprint_sha256: String,
    state: RecordState,
}

/// In-memory state-machine oracle for contract tests. The BTreeMap serializes
/// calls only inside this model; production concurrency control must come from
/// the backing database's uniqueness constraints and transaction isolation.
#[derive(Debug, Default)]
pub struct ReferenceIdempotencyLedger {
    records: BTreeMap<ScopedIdempotencyKey, LedgerRecord>,
    next_generation: u64,
}

impl ReferenceIdempotencyLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserve a scoped idempotency key after independently verifying the
    /// envelope, payload digest and full immutable request fingerprint.
    pub fn reserve(
        &mut self,
        tenant_scope: &str,
        operation_namespace: &str,
        envelope: &OperationEnvelope,
        payload_json: &str,
    ) -> Result<ReserveResult, LedgerModelError> {
        let key = ScopedIdempotencyKey::new(
            tenant_scope,
            operation_namespace,
            &envelope.idempotency_key,
        )?;
        envelope
            .validate()
            .map_err(|error| LedgerModelError::InvalidEnvelope(error.to_string()))?;
        let fingerprint = canonical_request_fingerprint_sha256(
            tenant_scope,
            operation_namespace,
            envelope,
            payload_json,
        )
        .map_err(|error| LedgerModelError::InvalidFingerprint(error.to_string()))?;

        if let Some(existing) = self.records.get(&key) {
            if existing.fingerprint_schema_version != REQUEST_FINGERPRINT_SCHEMA_VERSION
                || existing.fingerprint_sha256 != fingerprint
            {
                return Err(LedgerModelError::RequestFingerprintConflict);
            }
            return match &existing.state {
                RecordState::InProgress { .. } => Ok(ReserveResult::AlreadyInProgress),
                RecordState::Completed(outcome) => Ok(ReserveResult::Replay(outcome.clone())),
            };
        }

        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or(LedgerModelError::SequenceExhausted)?;
        let generation = self.next_generation;
        self.records.insert(
            key.clone(),
            LedgerRecord {
                fingerprint_schema_version: REQUEST_FINGERPRINT_SCHEMA_VERSION,
                fingerprint_sha256: fingerprint.clone(),
                state: RecordState::InProgress { generation },
            },
        );
        Ok(ReserveResult::Reserved(ReservationTicket {
            key,
            request_fingerprint_schema_version: REQUEST_FINGERPRINT_SCHEMA_VERSION,
            request_fingerprint_sha256: fingerprint,
            generation,
        }))
    }

    /// Commit one replayable outcome for the current reservation.
    ///
    /// In a real service this transition must be in the same database transaction
    /// as the domain mutation or transactional outbox insertion.
    pub fn commit(
        &mut self,
        ticket: &ReservationTicket,
        status_code: u16,
        receipt_id: &str,
        response_body: &[u8],
    ) -> Result<CommitResult, LedgerModelError> {
        if ticket.request_fingerprint_schema_version != REQUEST_FINGERPRINT_SCHEMA_VERSION {
            return Err(LedgerModelError::StaleReservation);
        }
        let proposed =
            StoredOutcome::new(status_code, receipt_id.to_owned(), response_body.to_vec())?;
        let record = self
            .records
            .get_mut(&ticket.key)
            .ok_or(LedgerModelError::StaleReservation)?;
        if record.fingerprint_schema_version != ticket.request_fingerprint_schema_version
            || record.fingerprint_sha256 != ticket.request_fingerprint_sha256
        {
            return Err(LedgerModelError::StaleReservation);
        }

        match record.state.clone() {
            RecordState::InProgress { generation } if generation == ticket.generation => {
                record.state = RecordState::Completed(proposed.clone());
                Ok(CommitResult::Committed(proposed))
            }
            RecordState::Completed(existing) if existing == proposed => {
                Ok(CommitResult::AlreadyCommitted(existing))
            }
            RecordState::InProgress { .. } => Err(LedgerModelError::StaleReservation),
            RecordState::Completed(_) => Err(LedgerModelError::ConflictingFinalization),
        }
    }

    /// Abandon a reservation only when the authoritative transaction proves
    /// no domain effect committed. Never use this to clear an unknown remote
    /// result; resolve/replay the ledger instead.
    pub fn abort_before_effect(
        &mut self,
        ticket: &ReservationTicket,
    ) -> Result<(), LedgerModelError> {
        let can_abort = self.records.get(&ticket.key).is_some_and(|record| {
            record.fingerprint_schema_version == ticket.request_fingerprint_schema_version
                && record.fingerprint_sha256 == ticket.request_fingerprint_sha256
                && matches!(
                    &record.state,
                    RecordState::InProgress { generation } if *generation == ticket.generation
                )
        });
        if !can_abort {
            return Err(LedgerModelError::StaleReservation);
        }
        self.records.remove(&ticket.key);
        Ok(())
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn completed_count(&self) -> usize {
        self.records
            .values()
            .filter(|record| matches!(record.state, RecordState::Completed(_)))
            .count()
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{CommitResult, LedgerModelError, ReferenceIdempotencyLedger, ReserveResult};
    use crate::offline_journal::{
        OFFLINE_JOURNAL_SCHEMA_VERSION, OperationEnvelope, RequiredNullableOperationId,
    };
    use crate::offline_payload_digest::canonical_payload_digest_sha256;
    use std::sync::{Arc, Barrier, Mutex};

    const TENANT: &str = "tenant-001";
    const NAMESPACE: &str = "work-order.apply";
    const PAYLOAD: &str = r#"{ "quantity": 2 }"#;

    fn envelope(
        operation_id: &str,
        aggregate_id: &str,
        expected_revision: u64,
    ) -> OperationEnvelope {
        OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: operation_id.to_owned(),
            aggregate_id: aggregate_id.to_owned(),
            expected_revision,
            payload_digest_sha256: canonical_payload_digest_sha256(PAYLOAD).unwrap(),
            idempotency_key: "idem-key-001".to_owned(),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        }
    }

    fn reserved(result: ReserveResult) -> super::ReservationTicket {
        match result {
            ReserveResult::Reserved(ticket) => ticket,
            other => panic!("expected reservation ticket, got {other:?}"),
        }
    }

    #[test]
    fn same_request_is_reserved_once_and_completed_outcome_is_replayed_exactly() {
        let mut ledger = ReferenceIdempotencyLedger::new();
        let envelope = envelope("op-001", "aggregate-001", 4);
        let ticket = reserved(
            ledger
                .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
                .unwrap(),
        );

        assert_eq!(
            ledger.reserve(TENANT, NAMESPACE, &envelope, PAYLOAD),
            Ok(ReserveResult::AlreadyInProgress)
        );
        let outcome = match ledger
            .commit(&ticket, 201, "receipt-001", br#"{"revision":5,"ok":true}"#)
            .unwrap()
        {
            CommitResult::Committed(outcome) => outcome,
            other => panic!("expected committed outcome, got {other:?}"),
        };
        assert_eq!(
            ledger.reserve(TENANT, NAMESPACE, &envelope, PAYLOAD),
            Ok(ReserveResult::Replay(outcome.clone()))
        );
        assert_eq!(
            ledger
                .commit(&ticket, 201, "receipt-001", br#"{"revision":5,"ok":true}"#)
                .unwrap(),
            CommitResult::AlreadyCommitted(outcome.clone())
        );
        assert_eq!(outcome.response_body(), br#"{"revision":5,"ok":true}"#);
        assert_eq!(outcome.response_body_sha256().len(), 64);
        assert_eq!(ledger.record_count(), 1);
        assert_eq!(ledger.completed_count(), 1);
    }

    #[test]
    fn same_scoped_key_with_changed_immutable_binding_is_rejected() {
        let mut ledger = ReferenceIdempotencyLedger::new();
        let original = envelope("op-001", "aggregate-001", 4);
        let _ticket = reserved(
            ledger
                .reserve(TENANT, NAMESPACE, &original, PAYLOAD)
                .unwrap(),
        );

        let changed_aggregate = envelope("op-001", "aggregate-002", 4);
        assert_eq!(
            ledger.reserve(TENANT, NAMESPACE, &changed_aggregate, PAYLOAD),
            Err(LedgerModelError::RequestFingerprintConflict)
        );
        let changed_revision = envelope("op-001", "aggregate-001", 5);
        assert_eq!(
            ledger.reserve(TENANT, NAMESPACE, &changed_revision, PAYLOAD),
            Err(LedgerModelError::RequestFingerprintConflict)
        );
        assert_eq!(ledger.record_count(), 1);
    }

    #[test]
    fn raw_key_is_independent_between_authenticated_scopes_and_namespaces() {
        let mut ledger = ReferenceIdempotencyLedger::new();
        let envelope_a = envelope("op-001", "aggregate-001", 4);
        let envelope_b = envelope("op-002", "aggregate-001", 4);
        let first = reserved(
            ledger
                .reserve("tenant-a", "work-order.apply", &envelope_a, PAYLOAD)
                .unwrap(),
        );
        let second = reserved(
            ledger
                .reserve("tenant-b", "work-order.apply", &envelope_a, PAYLOAD)
                .unwrap(),
        );
        let third = reserved(
            ledger
                .reserve("tenant-a", "work-order.cancel", &envelope_b, PAYLOAD)
                .unwrap(),
        );
        assert_ne!(first.key(), second.key());
        assert_ne!(first.key(), third.key());
        assert_eq!(ledger.record_count(), 3);
    }

    #[test]
    fn stale_reservation_cannot_commit_after_safe_abort_and_reacquire() {
        let mut ledger = ReferenceIdempotencyLedger::new();
        let envelope = envelope("op-001", "aggregate-001", 4);
        let stale = reserved(
            ledger
                .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
                .unwrap(),
        );
        ledger.abort_before_effect(&stale).unwrap();
        let current = reserved(
            ledger
                .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
                .unwrap(),
        );
        assert_ne!(stale, current);
        assert_eq!(
            ledger.commit(&stale, 201, "old-receipt", b"{}"),
            Err(LedgerModelError::StaleReservation)
        );
        assert!(matches!(
            ledger.commit(&current, 201, "new-receipt", b"{}"),
            Ok(CommitResult::Committed(_))
        ));
    }

    #[test]
    fn outcome_cannot_be_rewritten_after_commit() {
        let mut ledger = ReferenceIdempotencyLedger::new();
        let envelope = envelope("op-001", "aggregate-001", 4);
        let ticket = reserved(
            ledger
                .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
                .unwrap(),
        );
        ledger
            .commit(&ticket, 201, "receipt-001", b"{\"ok\":true}")
            .unwrap();
        assert_eq!(
            ledger.commit(&ticket, 409, "receipt-002", b"{\"ok\":false}"),
            Err(LedgerModelError::ConflictingFinalization)
        );
        assert_eq!(ledger.completed_count(), 1);
    }

    #[test]
    fn simulated_response_loss_never_reexecutes_a_committed_request() {
        let mut ledger = ReferenceIdempotencyLedger::new();
        let envelope = envelope("op-001", "aggregate-001", 4);
        let ticket = reserved(
            ledger
                .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
                .unwrap(),
        );
        let response_body = br#"{"revision":5,"receipt":"receipt-001"}"#;
        let committed = ledger
            .commit(&ticket, 200, "receipt-001", response_body)
            .unwrap();
        assert!(matches!(committed, CommitResult::Committed(_)));

        // Model a lost network response: the client retries without knowing the
        // commit result. It receives the exact stored bytes and receipt.
        let replay = ledger
            .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
            .unwrap();
        match replay {
            ReserveResult::Replay(outcome) => {
                assert_eq!(outcome.receipt_id(), "receipt-001");
                assert_eq!(outcome.response_body(), response_body);
            }
            other => panic!("expected response replay after lost response, got {other:?}"),
        }
        assert_eq!(ledger.completed_count(), 1);
    }

    #[test]
    fn concurrent_duplicate_claims_yield_one_reservation_in_the_reference_model() {
        const CONTENDERS: usize = 64;
        let ledger = Arc::new(Mutex::new(ReferenceIdempotencyLedger::new()));
        let barrier = Arc::new(Barrier::new(CONTENDERS));
        let envelope = envelope("op-concurrent", "aggregate-001", 4);
        let mut workers = Vec::with_capacity(CONTENDERS);

        for _ in 0..CONTENDERS {
            let ledger = Arc::clone(&ledger);
            let barrier = Arc::clone(&barrier);
            let envelope = envelope.clone();
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                ledger
                    .lock()
                    .unwrap()
                    .reserve(TENANT, NAMESPACE, &envelope, PAYLOAD)
                    .unwrap()
            }));
        }

        let mut reservations = Vec::new();
        let mut in_progress = 0;
        for worker in workers {
            match worker.join().unwrap() {
                ReserveResult::Reserved(ticket) => reservations.push(ticket),
                ReserveResult::AlreadyInProgress => in_progress += 1,
                ReserveResult::Replay(_) => {
                    panic!("nothing should be committed before the reservation batch finishes")
                }
            }
        }
        assert_eq!(reservations.len(), 1);
        assert_eq!(in_progress, CONTENDERS - 1);

        let ticket = reservations.pop().unwrap();
        ledger
            .lock()
            .unwrap()
            .commit(&ticket, 200, "receipt-concurrent", b"{\"applied\":true}")
            .unwrap();
        assert_eq!(ledger.lock().unwrap().completed_count(), 1);
    }
}
