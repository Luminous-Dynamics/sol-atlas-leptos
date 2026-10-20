// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic decision logic for an offline operation journal.
//!
//! This module does not provide durable storage or a server. A caller must
//! persist an entry before dispatching a request, persist the transition to
//! outcome_unknown before sending it, and require the server to enforce
//! idempotency for the operation identity and payload digest.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const OFFLINE_JOURNAL_SCHEMA_VERSION: u16 = 2;
/// Version of the durable snapshot representation, independent of wire protocol schema.
pub const JOURNAL_ENTRY_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationEnvelope {
    pub schema_version: u16,
    pub operation_id: String,
    pub aggregate_id: String,
    pub expected_revision: u64,
    /// SHA-256 of canonical payload bytes, encoded as 64 lowercase hex characters.
    pub payload_digest_sha256: String,
    /// Stable across retries; the service must bind it to this immutable envelope.
    pub idempotency_key: String,
    /// Conflict lineage; null is explicit and the field must be present on the wire.
    pub supersedes_operation_id: RequiredNullableOperationId,
}

fn validate_snapshot_state(
    envelope: &OperationEnvelope,
    state: &JournalState,
    attempt_count: u32,
) -> Result<(), JournalError> {
    match state {
        JournalState::Prepared if attempt_count == 0 => Ok(()),
        JournalState::Prepared => Err(JournalError::InvalidSnapshot("prepared_with_attempts")),
        JournalState::OutcomeUnknown if attempt_count > 0 => Ok(()),
        JournalState::OutcomeUnknown => Err(JournalError::InvalidSnapshot("unknown_without_attempt")),
        JournalState::Received {
            receipt_ref,
            applied_revision,
        } if attempt_count > 0
            && !receipt_ref.trim().is_empty()
            && *applied_revision >= envelope.expected_revision =>
        {
            Ok(())
        }
        JournalState::Received { .. } => {
            Err(JournalError::InvalidSnapshot("invalid_received_state"))
        }
        JournalState::Rejected { reason_code }
            if attempt_count > 0 && !reason_code.trim().is_empty() =>
        {
            Ok(())
        }
        JournalState::Rejected { .. } => {
            Err(JournalError::InvalidSnapshot("invalid_rejected_state"))
        }
        JournalState::ConflictNeedsReview {
            current_revision,
            reason_code,
        } if attempt_count > 0
            && !reason_code.trim().is_empty()
            && *current_revision > envelope.expected_revision =>
        {
            Ok(())
        }
        JournalState::ConflictNeedsReview { .. } => {
            Err(JournalError::InvalidSnapshot("invalid_conflict_state"))
        }
    }
}

/// A nullable string that must still be present on the wire.
///
/// Unlike `Option<String>` as a struct field, this non-optional untagged enum
/// accepts JSON null while making an omitted field a deserialization error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredNullableOperationId {
    Value(String),
    Null(()),
}

impl OperationEnvelope {
    /// Verify the supplied raw JSON payload against this envelope's digest.
    ///
    /// Call before persisting the operation or dispatching the request. The
    /// digest's shape-only validation in `validate` is not a substitute for
    /// recomputing it from the original payload bytes.
    pub fn verify_payload_json(
        &self,
        payload_json: &str,
    ) -> Result<(), crate::offline_payload_digest::PayloadDigestError> {
        crate::offline_payload_digest::verify_payload_digest(
            payload_json,
            &self.payload_digest_sha256,
        )
    }

    pub fn validate(&self) -> Result<(), JournalError> {
        if self.schema_version != OFFLINE_JOURNAL_SCHEMA_VERSION {
            return Err(JournalError::UnsupportedSchemaVersion(self.schema_version));
        }
        require_non_empty(&self.operation_id, "operation_id")?;
        require_non_empty(&self.aggregate_id, "aggregate_id")?;
        require_non_empty(&self.idempotency_key, "idempotency_key")?;
        if let RequiredNullableOperationId::Value(parent) = &self.supersedes_operation_id {
            require_non_empty(parent, "supersedes_operation_id")?;
        }
        let digest = self.payload_digest_sha256.as_bytes();
        if digest.len() != 64
            || !digest
                .iter()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        {
            return Err(JournalError::InvalidPayloadDigest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationBinding {
    pub envelope_schema_version: u16,
    pub operation_id: String,
    pub aggregate_id: String,
    pub expected_revision: u64,
    pub payload_digest_sha256: String,
    pub idempotency_key: String,
    /// Conflict lineage from the immutable request envelope; null is explicit.
    pub supersedes_operation_id: RequiredNullableOperationId,
}

impl OperationBinding {
    pub fn from_envelope(envelope: &OperationEnvelope) -> Self {
        Self {
            envelope_schema_version: envelope.schema_version,
            operation_id: envelope.operation_id.clone(),
            aggregate_id: envelope.aggregate_id.clone(),
            expected_revision: envelope.expected_revision,
            payload_digest_sha256: envelope.payload_digest_sha256.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            supersedes_operation_id: envelope.supersedes_operation_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JournalState {
    Prepared,
    /// A request may have reached the service; no authoritative outcome is known locally.
    OutcomeUnknown,
    Received {
        receipt_ref: String,
        applied_revision: u64,
    },
    Rejected {
        reason_code: String,
    },
    ConflictNeedsReview {
        current_revision: u64,
        reason_code: String,
    },
}

impl JournalState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::OutcomeUnknown => "outcome_unknown",
            Self::Received { .. } => "received",
            Self::Rejected { .. } => "rejected",
            Self::ConflictNeedsReview { .. } => "conflict_needs_review",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum ServiceReply {
    Received {
        binding: OperationBinding,
        receipt_ref: String,
        applied_revision: u64,
    },
    Rejected {
        binding: OperationBinding,
        reason_code: String,
    },
    Conflict {
        binding: OperationBinding,
        current_revision: u64,
        reason_code: String,
    },
}

impl ServiceReply {
    fn binding(&self) -> &OperationBinding {
        match self {
            Self::Received { binding, .. }
            | Self::Rejected { binding, .. }
            | Self::Conflict { binding, .. } => binding,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileOutcome {
    Applied,
    DuplicateAcknowledgement,
    DuplicateTerminalReply,
}

/// Versioned persistence representation of a complete immutable journal operation.
///
/// The payload string must contain canonical RFC 8785 JSON, not an arbitrary
/// source spelling. Recovery re-canonicalizes it, demands byte-for-byte equality,
/// and recomputes its digest before trusting either the payload or stored state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalEntrySnapshotV1 {
    pub schema_version: u16,
    pub envelope: OperationEnvelope,
    pub canonical_payload_json: String,
    pub state: JournalState,
    pub attempt_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    envelope: OperationEnvelope,
    payload: crate::offline_payload_digest::CanonicalPayload,
    state: JournalState,
    /// Number of attempts that were durably marked as potentially dispatched.
    attempt_count: u32,
}

impl JournalEntry {
    /// Prepare an operation only after validating the digest against its raw payload.
    ///
    /// The canonical UTF-8 bytes are retained with the envelope so a retry can
    /// resend exactly the payload that was hashed, rather than reconstructing it
    /// from potentially edited UI state.
    pub fn prepare(envelope: OperationEnvelope, payload_json: &str) -> Result<Self, JournalError> {
        envelope.validate()?;
        let payload = crate::offline_payload_digest::CanonicalPayload::from_json(payload_json)
            .map_err(|error| JournalError::PayloadCanonicalization(error.to_string()))?;
        if payload.sha256_hex() != envelope.payload_digest_sha256 {
            return Err(JournalError::PayloadDigestMismatch);
        }
        Ok(Self {
            envelope,
            payload,
            state: JournalState::Prepared,
            attempt_count: 0,
        })
    }

    /// Export the entry as a versioned, deterministic storage snapshot.
    ///
    /// Persist this snapshot atomically with the pending indexes used by the
    /// backend. The snapshot is data, not authority: recovery must call
    /// `from_snapshot` and validate it before resuming dispatch or displaying
    /// a terminal state as authoritative.
    pub fn to_snapshot(&self) -> Result<JournalEntrySnapshotV1, JournalError> {
        let canonical_payload_json = std::str::from_utf8(self.payload.canonical_utf8())
            .map_err(|_| JournalError::InvalidSnapshot("canonical_payload_not_utf8"))?
            .to_owned();
        Ok(JournalEntrySnapshotV1 {
            schema_version: JOURNAL_ENTRY_SNAPSHOT_SCHEMA_VERSION,
            envelope: self.envelope.clone(),
            canonical_payload_json,
            state: self.state.clone(),
            attempt_count: self.attempt_count,
        })
    }

    /// Rehydrate a snapshot only after rechecking schema, canonical bytes,
    /// digest binding, and state/attempt invariants.
    pub fn from_snapshot(snapshot: JournalEntrySnapshotV1) -> Result<Self, JournalError> {
        if snapshot.schema_version != JOURNAL_ENTRY_SNAPSHOT_SCHEMA_VERSION {
            return Err(JournalError::UnsupportedSnapshotSchemaVersion(
                snapshot.schema_version,
            ));
        }
        snapshot.envelope.validate()?;

        let payload =
            crate::offline_payload_digest::CanonicalPayload::from_json(&snapshot.canonical_payload_json)
                .map_err(|error| JournalError::PayloadCanonicalization(error.to_string()))?;
        if payload.canonical_utf8() != snapshot.canonical_payload_json.as_bytes() {
            return Err(JournalError::NonCanonicalSnapshotPayload);
        }
        if payload.sha256_hex() != snapshot.envelope.payload_digest_sha256 {
            return Err(JournalError::PayloadDigestMismatch);
        }
        validate_snapshot_state(
            &snapshot.envelope,
            &snapshot.state,
            snapshot.attempt_count,
        )?;

        Ok(Self {
            envelope: snapshot.envelope,
            payload,
            state: snapshot.state,
            attempt_count: snapshot.attempt_count,
        })
    }

    /// Returns the immutable operation envelope.
    pub fn envelope(&self) -> &OperationEnvelope {
        &self.envelope
    }

    /// Returns the canonical payload bytes that must be persisted and retried unchanged.
    pub fn canonical_payload_utf8(&self) -> &[u8] {
        self.payload.canonical_utf8()
    }

    /// Returns the current reducer state without permitting external mutation.
    pub fn state(&self) -> &JournalState {
        &self.state
    }

    /// Number of attempts durably marked as possibly dispatched.
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Validate that a retry is exactly the same logical operation.
    ///
    /// The durable journal should store the envelope immutably; this explicit
    /// check protects adapters that reconstruct a request from editable UI state.
    pub fn ensure_retry_identity(&self, candidate: &OperationEnvelope) -> Result<(), JournalError> {
        candidate.validate()?;
        if candidate != &self.envelope {
            return Err(JournalError::OperationIdentityChanged);
        }
        Ok(())
    }

    /// Mark the operation as possibly dispatched before performing network I/O.
    ///
    /// The caller must persist the updated entry before sending. A crash after
    /// this transition is intentionally represented as outcome_unknown.
    pub fn mark_attempt_started(&mut self) -> Result<(), JournalError> {
        match &self.state {
            JournalState::Prepared => self.attempt_count = 1,
            JournalState::OutcomeUnknown => {
                self.attempt_count = self
                    .attempt_count
                    .checked_add(1)
                    .ok_or(JournalError::AttemptsExhausted)?;
            }
            _ => return Err(JournalError::InvalidTransition),
        }
        self.state = JournalState::OutcomeUnknown;
        Ok(())
    }

    /// Apply an authoritative service reply or safely recognize a duplicate reply.
    ///
    /// Unknown/malformed reply data must not be converted into a ServiceReply.
    /// The adapter should leave the entry unchanged if decoding fails.
    pub fn reconcile(&mut self, reply: &ServiceReply) -> Result<ReconcileOutcome, JournalError> {
        validate_reply_shape(reply)?;
        if reply.binding() != &OperationBinding::from_envelope(&self.envelope) {
            return Err(JournalError::ReplyBindingMismatch);
        }

        match (self.state.clone(), reply) {
            (
                JournalState::OutcomeUnknown,
                ServiceReply::Received {
                    receipt_ref,
                    applied_revision,
                    ..
                },
            ) => {
                if *applied_revision < self.envelope.expected_revision {
                    return Err(JournalError::InvalidReply);
                }
                self.state = JournalState::Received {
                    receipt_ref: receipt_ref.clone(),
                    applied_revision: *applied_revision,
                };
                Ok(ReconcileOutcome::Applied)
            }
            (JournalState::OutcomeUnknown, ServiceReply::Rejected { reason_code, .. }) => {
                self.state = JournalState::Rejected {
                    reason_code: reason_code.clone(),
                };
                Ok(ReconcileOutcome::Applied)
            }
            (
                JournalState::OutcomeUnknown,
                ServiceReply::Conflict {
                    current_revision,
                    reason_code,
                    ..
                },
            ) => {
                if *current_revision <= self.envelope.expected_revision {
                    return Err(JournalError::InvalidReply);
                }
                self.state = JournalState::ConflictNeedsReview {
                    current_revision: *current_revision,
                    reason_code: reason_code.clone(),
                };
                Ok(ReconcileOutcome::Applied)
            }
            (
                JournalState::Received {
                    receipt_ref,
                    applied_revision,
                },
                ServiceReply::Received {
                    receipt_ref: reply_ref,
                    applied_revision: reply_revision,
                    ..
                },
            ) if receipt_ref == *reply_ref && applied_revision == *reply_revision => {
                Ok(ReconcileOutcome::DuplicateAcknowledgement)
            }
            (
                JournalState::Rejected { reason_code },
                ServiceReply::Rejected {
                    reason_code: reply_code,
                    ..
                },
            ) if reason_code == *reply_code => Ok(ReconcileOutcome::DuplicateTerminalReply),
            (
                JournalState::ConflictNeedsReview {
                    current_revision,
                    reason_code,
                },
                ServiceReply::Conflict {
                    current_revision: reply_revision,
                    reason_code: reply_code,
                    ..
                },
            ) if current_revision == *reply_revision && reason_code == *reply_code => {
                Ok(ReconcileOutcome::DuplicateTerminalReply)
            }
            (JournalState::Prepared, _) => Err(JournalError::InvalidTransition),
            _ => Err(JournalError::InvalidTransition),
        }
    }

    /// Create a new operation after a conflict, leaving the original entry intact.
    ///
    /// The caller still needs a durable uniqueness constraint for operation IDs
    /// and idempotency keys across the whole journal, not just this entry.
    pub fn prepare_conflict_resolution(
        &self,
        replacement: OperationEnvelope,
        replacement_payload_json: &str,
    ) -> Result<Self, JournalError> {
        let current_revision = match &self.state {
            JournalState::ConflictNeedsReview {
                current_revision, ..
            } => *current_revision,
            _ => {
                return Err(JournalError::ConflictResolutionMismatch(
                    "original_not_conflicted",
                ));
            }
        };
        replacement.validate()?;
        if replacement.operation_id == self.envelope.operation_id {
            return Err(JournalError::ConflictResolutionMismatch(
                "operation_id_reused",
            ));
        }
        if replacement.aggregate_id != self.envelope.aggregate_id {
            return Err(JournalError::ConflictResolutionMismatch(
                "aggregate_changed",
            ));
        }
        if replacement.expected_revision != current_revision {
            return Err(JournalError::ConflictResolutionMismatch(
                "wrong_base_revision",
            ));
        }
        if replacement.supersedes_operation_id
            != RequiredNullableOperationId::Value(self.envelope.operation_id.clone())
        {
            return Err(JournalError::ConflictResolutionMismatch(
                "supersedes_reference_missing",
            ));
        }
        if replacement.idempotency_key == self.envelope.idempotency_key {
            return Err(JournalError::ConflictResolutionMismatch(
                "idempotency_key_reused",
            ));
        }
        Self::prepare(replacement, replacement_payload_json)
    }
}

fn validate_reply_shape(reply: &ServiceReply) -> Result<(), JournalError> {
    match reply {
        ServiceReply::Received { receipt_ref, .. } => {
            require_non_empty(receipt_ref, "receipt_ref").map_err(|_| JournalError::InvalidReply)
        }
        ServiceReply::Rejected { reason_code, .. } | ServiceReply::Conflict { reason_code, .. } => {
            require_non_empty(reason_code, "reason_code").map_err(|_| JournalError::InvalidReply)
        }
    }
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), JournalError> {
    if value.trim().is_empty() {
        Err(JournalError::EmptyField(field))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalError {
    UnsupportedSchemaVersion(u16),
    UnsupportedSnapshotSchemaVersion(u16),
    InvalidSnapshot(&'static str),
    NonCanonicalSnapshotPayload,
    EmptyField(&'static str),
    InvalidPayloadDigest,
    PayloadCanonicalization(String),
    PayloadDigestMismatch,
    AttemptsExhausted,
    OperationIdentityChanged,
    InvalidReply,
    ReplyBindingMismatch,
    InvalidTransition,
    ConflictResolutionMismatch(&'static str),
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported journal schema version {version}")
            }
            Self::UnsupportedSnapshotSchemaVersion(version) => {
                write!(formatter, "unsupported journal snapshot schema version {version}")
            }
            Self::InvalidSnapshot(reason) => {
                write!(formatter, "journal snapshot is inconsistent: {reason}")
            }
            Self::NonCanonicalSnapshotPayload => {
                formatter.write_str("persisted payload is valid JSON but not canonical JCS bytes")
            }
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidPayloadDigest => {
                formatter.write_str("payload digest must be 64 lowercase hexadecimal characters")
            }
            Self::PayloadCanonicalization(reason) => write!(
                formatter,
                "payload cannot be represented under strict JCS: {reason}"
            ),
            Self::PayloadDigestMismatch => formatter
                .write_str("canonical payload digest does not match the operation envelope"),
            Self::AttemptsExhausted => formatter.write_str("journal attempt counter exhausted"),
            Self::OperationIdentityChanged => {
                formatter.write_str("retry changed immutable operation identity or payload")
            }
            Self::InvalidReply => formatter
                .write_str("service reply is malformed or inconsistent with expected revision"),
            Self::ReplyBindingMismatch => formatter
                .write_str("service reply does not bind to the immutable operation envelope"),
            Self::InvalidTransition => formatter.write_str("journal transition is not allowed"),
            Self::ConflictResolutionMismatch(reason) => {
                write!(
                    formatter,
                    "conflict resolution does not match the original: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for JournalError {}

#[cfg(test)]
mod tests {
    use super::{
        JournalEntry, JournalError, JournalState, OFFLINE_JOURNAL_SCHEMA_VERSION, OperationBinding,
        OperationEnvelope, ReconcileOutcome, RequiredNullableOperationId, ServiceReply,
    };
    use serde::Deserialize;
    use std::collections::BTreeSet;

    fn payload_json() -> &'static str {
        r#"{"inspection":"unit-test","quantity":2}"#
    }

    fn envelope() -> OperationEnvelope {
        OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: "op-001".to_owned(),
            aggregate_id: "job-001".to_owned(),
            expected_revision: 7,
            payload_digest_sha256: crate::offline_payload_digest::canonical_payload_digest_sha256(
                payload_json(),
            )
            .unwrap(),
            idempotency_key: "idem-001".to_owned(),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        }
    }

    fn prepared_entry() -> JournalEntry {
        JournalEntry::prepare(envelope(), payload_json()).unwrap()
    }

    fn operation_binding() -> OperationBinding {
        OperationBinding::from_envelope(&envelope())
    }

    #[test]
    fn prepared_entry_retains_exact_canonical_payload_bytes() {
        let entry = prepared_entry();
        assert_eq!(
            std::str::from_utf8(entry.canonical_payload_utf8()).unwrap(),
            r#"{"inspection":"unit-test","quantity":2}"#
        );
        assert_eq!(
            entry.envelope().payload_digest_sha256,
            crate::offline_payload_digest::canonical_payload_digest_sha256(payload_json()).unwrap()
        );
        assert_eq!(entry.attempt_count(), 0);
        assert_eq!(entry.state(), &JournalState::Prepared);
    }

    #[test]
    fn prepare_rejects_payload_digest_mismatch_before_creating_a_journal_entry() {
        let envelope = envelope();
        assert!(matches!(
            JournalEntry::prepare(envelope, r#"{"inspection":"different","quantity":2}"#),
            Err(JournalError::PayloadDigestMismatch)
        ));
    }

    #[test]
    fn prepare_rejects_non_jcs_payload_before_creating_a_journal_entry() {
        let envelope = envelope();
        assert!(matches!(
            JournalEntry::prepare(envelope, r#"{"a":1,"a":2}"#),
            Err(JournalError::PayloadCanonicalization(_))
        ));
    }

    #[test]
    fn dispatch_state_is_unknown_not_success_until_an_authoritative_receipt() {
        let mut entry = prepared_entry();
        entry.mark_attempt_started().unwrap();
        assert_eq!(entry.state, JournalState::OutcomeUnknown);
        assert_eq!(entry.attempt_count, 1);
    }

    #[test]
    fn retry_must_reuse_the_exact_immutable_envelope() {
        let entry = prepared_entry();
        assert!(entry.ensure_retry_identity(&entry.envelope.clone()).is_ok());

        let mut changed = entry.envelope.clone();
        changed.payload_digest_sha256 = "b".repeat(64);
        assert_eq!(
            entry.ensure_retry_identity(&changed),
            Err(JournalError::OperationIdentityChanged)
        );

        changed = entry.envelope.clone();
        changed.expected_revision += 1;
        assert_eq!(
            entry.ensure_retry_identity(&changed),
            Err(JournalError::OperationIdentityChanged)
        );
    }

    #[test]
    fn duplicate_authoritative_receipt_is_idempotent_but_a_different_reply_is_not() {
        let mut entry = prepared_entry();
        entry.mark_attempt_started().unwrap();
        let receipt = ServiceReply::Received {
            binding: operation_binding(),
            receipt_ref: "receipt-001".to_owned(),
            applied_revision: 8,
        };
        assert_eq!(entry.reconcile(&receipt), Ok(ReconcileOutcome::Applied));
        assert_eq!(
            entry.reconcile(&receipt),
            Ok(ReconcileOutcome::DuplicateAcknowledgement)
        );
        assert_eq!(
            entry.reconcile(&ServiceReply::Rejected {
                binding: operation_binding(),
                reason_code: "stale-rejection".to_owned(),
            }),
            Err(JournalError::InvalidTransition)
        );
        assert_eq!(
            entry.state,
            JournalState::Received {
                receipt_ref: "receipt-001".to_owned(),
                applied_revision: 8,
            }
        );
    }

    #[test]
    fn receipt_bound_to_a_different_operation_cannot_reconcile() {
        let mut entry = prepared_entry();
        entry.mark_attempt_started().unwrap();

        for mismatch in 0..7 {
            let mut binding = operation_binding();
            match mismatch {
                0 => binding.envelope_schema_version = 1,
                1 => binding.operation_id = "op-from-another-request".to_owned(),
                2 => binding.aggregate_id = "another-job".to_owned(),
                3 => binding.expected_revision += 1,
                4 => binding.payload_digest_sha256 = "b".repeat(64),
                5 => binding.idempotency_key = "another-key".to_owned(),
                6 => {
                    binding.supersedes_operation_id =
                        RequiredNullableOperationId::Value("another-parent".to_owned())
                }
                _ => unreachable!(),
            }
            let reply = ServiceReply::Received {
                binding,
                receipt_ref: "receipt-001".to_owned(),
                applied_revision: 8,
            };

            assert_eq!(
                entry.reconcile(&reply),
                Err(JournalError::ReplyBindingMismatch),
                "mismatched binding field index {mismatch}"
            );
            assert_eq!(entry.state, JournalState::OutcomeUnknown);
        }
    }

    #[test]
    fn operation_envelope_verifies_digest_against_raw_payload() {
        let payload = r#"{ "b": 2, "a": 1.0 }"#;
        let mut candidate = envelope();
        candidate.payload_digest_sha256 =
            crate::offline_payload_digest::canonical_payload_digest_sha256(payload).unwrap();

        assert!(candidate.verify_payload_json(payload).is_ok());
        assert!(matches!(
            candidate.verify_payload_json(r#"{"a":1,"b":3}"#),
            Err(crate::offline_payload_digest::PayloadDigestError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn operation_envelope_requires_explicit_null_for_missing_conflict_parent() {
        let envelope = envelope();
        let encoded = serde_json::to_value(&envelope).unwrap();
        assert_eq!(encoded["supersedes_operation_id"], serde_json::Value::Null);
        assert_eq!(
            serde_json::from_value::<OperationEnvelope>(encoded.clone()).unwrap(),
            envelope
        );

        let mut omitted = encoded;
        omitted
            .as_object_mut()
            .unwrap()
            .remove("supersedes_operation_id");
        assert!(
            serde_json::from_value::<OperationEnvelope>(omitted).is_err(),
            "omitting nullable request lineage must not be accepted as explicit null"
        );
    }

    #[test]
    fn operation_binding_requires_explicit_null_for_missing_conflict_parent() {
        let binding = operation_binding();
        let encoded = serde_json::to_value(&binding).unwrap();
        assert_eq!(encoded["supersedes_operation_id"], serde_json::Value::Null);
        assert_eq!(
            serde_json::from_value::<OperationBinding>(encoded.clone()).unwrap(),
            binding
        );

        let mut omitted = encoded;
        omitted
            .as_object_mut()
            .unwrap()
            .remove("supersedes_operation_id");
        assert!(
            serde_json::from_value::<OperationBinding>(omitted).is_err(),
            "omitting nullable lineage must not be accepted as explicit null"
        );
    }

    #[test]
    fn malformed_or_non_advanced_conflict_reply_leaves_state_unknown() {
        let mut entry = prepared_entry();
        entry.mark_attempt_started().unwrap();
        assert_eq!(
            entry.reconcile(&ServiceReply::Conflict {
                binding: operation_binding(),
                current_revision: 7,
                reason_code: "revision_conflict".to_owned(),
            }),
            Err(JournalError::InvalidReply)
        );
        assert_eq!(
            entry.reconcile(&ServiceReply::Received {
                binding: operation_binding(),
                receipt_ref: " ".to_owned(),
                applied_revision: 8,
            }),
            Err(JournalError::InvalidReply)
        );
        assert_eq!(entry.state, JournalState::OutcomeUnknown);
    }

    #[test]
    fn conflict_resolution_is_a_new_revision_bound_operation() {
        let mut original = prepared_entry();
        original.mark_attempt_started().unwrap();
        original
            .reconcile(&ServiceReply::Conflict {
                binding: operation_binding(),
                current_revision: 9,
                reason_code: "concurrent_edit".to_owned(),
            })
            .unwrap();

        let mut replacement = envelope();
        replacement.operation_id = "op-002".to_owned();
        replacement.idempotency_key = "idem-002".to_owned();
        replacement.expected_revision = 9;
        replacement.supersedes_operation_id =
            RequiredNullableOperationId::Value("op-001".to_owned());

        let resolved = original
            .prepare_conflict_resolution(replacement, payload_json())
            .unwrap();
        assert_eq!(resolved.state, JournalState::Prepared);
        assert_eq!(resolved.envelope.expected_revision, 9);
        assert_eq!(
            original.state,
            JournalState::ConflictNeedsReview {
                current_revision: 9,
                reason_code: "concurrent_edit".to_owned(),
            }
        );
    }

    #[test]
    fn conflict_resolution_rejects_same_identity_or_wrong_revision() {
        let mut original = prepared_entry();
        original.mark_attempt_started().unwrap();
        original
            .reconcile(&ServiceReply::Conflict {
                binding: operation_binding(),
                current_revision: 9,
                reason_code: "concurrent_edit".to_owned(),
            })
            .unwrap();

        let mut replacement = envelope();
        replacement.expected_revision = 9;
        replacement.supersedes_operation_id =
            RequiredNullableOperationId::Value("op-001".to_owned());
        assert!(matches!(
            original.prepare_conflict_resolution(replacement.clone(), payload_json()),
            Err(JournalError::ConflictResolutionMismatch(
                "operation_id_reused"
            ))
        ));
        replacement.operation_id = "op-002".to_owned();
        replacement.idempotency_key = "idem-002".to_owned();
        replacement.expected_revision = 8;
        assert!(matches!(
            original.prepare_conflict_resolution(replacement, payload_json()),
            Err(JournalError::ConflictResolutionMismatch(
                "wrong_base_revision"
            ))
        ));
    }

    #[test]
    fn unknown_future_service_reply_is_rejected_without_promoting_state() {
        let mut entry = prepared_entry();
        entry.mark_attempt_started().unwrap();
        let decoded: Result<ServiceReply, _> =
            serde_json::from_str(r#"{"reply":"future_server_status"}"#);
        assert!(decoded.is_err());
        assert_eq!(entry.state, JournalState::OutcomeUnknown);
    }

    #[test]
    fn digest_requires_exactly_64_lowercase_hex_characters() {
        let mut bad = envelope();
        bad.payload_digest_sha256 = "A".repeat(64);
        assert_eq!(
            JournalEntry::prepare(bad, payload_json()),
            Err(JournalError::InvalidPayloadDigest)
        );
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FaultCorpus {
        schema_version: u16,
        description: String,
        cases: Vec<FaultCase>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FaultCase {
        case_id: String,
        attempts: u32,
        reply: Option<ServiceReply>,
        repeat_reply: Option<ServiceReply>,
        expected_state: String,
        expected_result: String,
        expected_repeat_result: String,
    }

    fn result_label(result: Result<ReconcileOutcome, JournalError>) -> &'static str {
        match result {
            Ok(ReconcileOutcome::Applied) => "applied",
            Ok(ReconcileOutcome::DuplicateAcknowledgement) => "duplicate_acknowledgement",
            Ok(ReconcileOutcome::DuplicateTerminalReply) => "duplicate_terminal_reply",
            Err(JournalError::InvalidReply) => "invalid_reply",
            Err(JournalError::ReplyBindingMismatch) => "reply_binding_mismatch",
            Err(JournalError::InvalidTransition) => "invalid_transition",
            Err(_) => "other_error",
        }
    }

    #[test]
    fn frozen_fault_corpus_executes_against_the_reducer() {
        let corpus: FaultCorpus = serde_json::from_str(include_str!(
            "../fixtures/offline-journal-fault-corpus-v2.json"
        ))
        .expect("fault corpus must be valid JSON matching the frozen schema");
        assert_eq!(corpus.schema_version, OFFLINE_JOURNAL_SCHEMA_VERSION);
        assert!(!corpus.description.trim().is_empty());
        assert!(!corpus.cases.is_empty());

        let mut seen_ids = BTreeSet::new();
        for case in corpus.cases {
            assert!(
                seen_ids.insert(case.case_id.clone()),
                "duplicate case id: {}",
                case.case_id
            );
            let mut entry = prepared_entry();
            for _ in 0..case.attempts {
                entry.mark_attempt_started().unwrap();
            }

            let first_result = match case.reply.as_ref() {
                None => "no_reply",
                Some(reply) => result_label(entry.reconcile(reply)),
            };
            assert_eq!(first_result, case.expected_result, "case {}", case.case_id);

            let repeat_result = match case.repeat_reply.as_ref() {
                None => "not_applicable",
                Some(reply) => result_label(entry.reconcile(reply)),
            };
            assert_eq!(
                repeat_result, case.expected_repeat_result,
                "repeat reply in case {}",
                case.case_id
            );
            assert_eq!(
                entry.state.as_str(),
                case.expected_state,
                "case {}",
                case.case_id
            );
        }
    }
}
