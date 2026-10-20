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
    /// Set only when a deliberate new operation resolves an earlier conflict.
    #[serde(default)]
    pub supersedes_operation_id: Option<String>,
}

impl OperationEnvelope {
    pub fn validate(&self) -> Result<(), JournalError> {
        if self.schema_version != OFFLINE_JOURNAL_SCHEMA_VERSION {
            return Err(JournalError::UnsupportedSchemaVersion(self.schema_version));
        }
        require_non_empty(&self.operation_id, "operation_id")?;
        require_non_empty(&self.aggregate_id, "aggregate_id")?;
        require_non_empty(&self.idempotency_key, "idempotency_key")?;
        if let Some(parent) = &self.supersedes_operation_id {
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
    /// Conflict lineage from the immutable request envelope.
    #[serde(default)]
    pub supersedes_operation_id: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    pub envelope: OperationEnvelope,
    pub state: JournalState,
    /// Number of attempts that were durably marked as potentially dispatched.
    pub attempt_count: u32,
}

impl JournalEntry {
    pub fn prepare(envelope: OperationEnvelope) -> Result<Self, JournalError> {
        envelope.validate()?;
        Ok(Self {
            envelope,
            state: JournalState::Prepared,
            attempt_count: 0,
        })
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
        if replacement.supersedes_operation_id.as_deref()
            != Some(self.envelope.operation_id.as_str())
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
        Self::prepare(replacement)
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
    EmptyField(&'static str),
    InvalidPayloadDigest,
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
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidPayloadDigest => {
                formatter.write_str("payload digest must be 64 lowercase hexadecimal characters")
            }
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
        OperationEnvelope, ReconcileOutcome, ServiceReply,
    };
    use serde::Deserialize;
    use std::collections::BTreeSet;

    fn envelope() -> OperationEnvelope {
        OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: "op-001".to_owned(),
            aggregate_id: "job-001".to_owned(),
            expected_revision: 7,
            payload_digest_sha256: "a".repeat(64),
            idempotency_key: "idem-001".to_owned(),
            supersedes_operation_id: None,
        }
    }

    fn operation_binding() -> OperationBinding {
        OperationBinding::from_envelope(&envelope())
    }

    #[test]
    fn dispatch_state_is_unknown_not_success_until_an_authoritative_receipt() {
        let mut entry = JournalEntry::prepare(envelope()).unwrap();
        entry.mark_attempt_started().unwrap();
        assert_eq!(entry.state, JournalState::OutcomeUnknown);
        assert_eq!(entry.attempt_count, 1);
    }

    #[test]
    fn retry_must_reuse_the_exact_immutable_envelope() {
        let entry = JournalEntry::prepare(envelope()).unwrap();
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
        let mut entry = JournalEntry::prepare(envelope()).unwrap();
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
        let mut entry = JournalEntry::prepare(envelope()).unwrap();
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
                6 => binding.supersedes_operation_id = Some("another-parent".to_owned()),
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
    fn malformed_or_non_advanced_conflict_reply_leaves_state_unknown() {
        let mut entry = JournalEntry::prepare(envelope()).unwrap();
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
        let mut original = JournalEntry::prepare(envelope()).unwrap();
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
        replacement.supersedes_operation_id = Some("op-001".to_owned());

        let resolved = original.prepare_conflict_resolution(replacement).unwrap();
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
        let mut original = JournalEntry::prepare(envelope()).unwrap();
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
        replacement.supersedes_operation_id = Some("op-001".to_owned());
        assert!(matches!(
            original.prepare_conflict_resolution(replacement.clone()),
            Err(JournalError::ConflictResolutionMismatch(
                "operation_id_reused"
            ))
        ));
        replacement.operation_id = "op-002".to_owned();
        replacement.idempotency_key = "idem-002".to_owned();
        replacement.expected_revision = 8;
        assert!(matches!(
            original.prepare_conflict_resolution(replacement),
            Err(JournalError::ConflictResolutionMismatch(
                "wrong_base_revision"
            ))
        ));
    }

    #[test]
    fn unknown_future_service_reply_is_rejected_without_promoting_state() {
        let mut entry = JournalEntry::prepare(envelope()).unwrap();
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
            JournalEntry::prepare(bad),
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
            let mut entry = JournalEntry::prepare(envelope()).unwrap();
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
