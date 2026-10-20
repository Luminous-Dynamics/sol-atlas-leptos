// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Portable, integrity-checked offline journal bundle codec.
//!
//! The bundle preserves unresolved state and original owner/namespace bindings
//! across device transfer. Its unkeyed digest detects accidental corruption;
//! it is not a signature or an authentication mechanism. Importers must compare
//! the bundle owner with authenticated local context and merge records without
//! overwriting conflicts. A bundle never grants authority to dispatch.

use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    offline_journal::{JournalEntry, JournalEntrySnapshotV1},
    offline_payload_digest::canonicalize_payload_json,
};

pub const OFFLINE_JOURNAL_BUNDLE_SCHEMA_VERSION: u16 = 1;
/// JCS represents JSON numbers using IEEE-754 binary64. Revision counters in
/// the bundle's canonical numeric fields must stay within the safe-integer range.
pub const MAX_JCS_SAFE_REVISION: u64 = 9_007_199_254_740_991;

#[derive(Debug)]
pub enum OfflineJournalBundleError {
    EmptyOwnerScope,
    EmptyNamespace {
        operation_id: String,
    },
    UnsupportedSchemaVersion(u16),
    InvalidSnapshot {
        operation_id: String,
        reason: String,
    },
    DuplicateOperationId(String),
    DuplicateScopedIdempotencyKey {
        operation_id: String,
    },
    RevisionOutOfRange {
        operation_id: String,
        field: &'static str,
    },
    NonCanonicalRecordOrder,
    OwnerScopeMismatch {
        expected: String,
        bundle: String,
    },
    IntegrityDigestMismatch {
        expected: String,
        actual: String,
    },
    Serialization(String),
    InvalidJson(String),
}

impl fmt::Display for OfflineJournalBundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyOwnerScope => formatter.write_str("bundle owner scope must not be empty"),
            Self::EmptyNamespace { operation_id } => {
                write!(formatter, "operation {operation_id} has an empty namespace")
            }
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported offline journal bundle schema {version}"
                )
            }
            Self::InvalidSnapshot {
                operation_id,
                reason,
            } => write!(
                formatter,
                "operation {operation_id} snapshot is invalid: {reason}"
            ),
            Self::DuplicateOperationId(operation_id) => {
                write!(formatter, "bundle repeats operation ID {operation_id}")
            }
            Self::DuplicateScopedIdempotencyKey { operation_id } => write!(
                formatter,
                "operation {operation_id} reuses a scoped idempotency key"
            ),
            Self::RevisionOutOfRange {
                operation_id,
                field,
            } => write!(
                formatter,
                "operation {operation_id} field {field} exceeds the JCS safe-integer range"
            ),
            Self::NonCanonicalRecordOrder => {
                formatter.write_str("bundle records must be sorted by operation ID")
            }
            Self::OwnerScopeMismatch { expected, bundle } => write!(
                formatter,
                "bundle owner scope {bundle:?} does not match authenticated import scope {expected:?}"
            ),
            Self::IntegrityDigestMismatch { expected, actual } => write!(
                formatter,
                "bundle integrity digest mismatch: expected {expected}, computed {actual}"
            ),
            Self::Serialization(reason) => {
                write!(formatter, "bundle serialization failed: {reason}")
            }
            Self::InvalidJson(reason) => write!(formatter, "bundle JSON is invalid: {reason}"),
        }
    }
}

impl Error for OfflineJournalBundleError {}

/// One portable operation. Owner scope is also repeated in the bundle header,
/// avoiding ambiguous cross-scope imports while keeping namespace per operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfflineJournalBundleRecordV1 {
    pub operation_namespace: String,
    pub snapshot: JournalEntrySnapshotV1,
}

impl OfflineJournalBundleRecordV1 {
    pub fn operation_id(&self) -> &str {
        &self.snapshot.envelope.operation_id
    }

    fn validate(&self) -> Result<(), OfflineJournalBundleError> {
        let operation_id = self.operation_id().to_owned();
        if self.operation_namespace.trim().is_empty() {
            return Err(OfflineJournalBundleError::EmptyNamespace { operation_id });
        }
        let entry = JournalEntry::from_snapshot(self.snapshot.clone()).map_err(|error| {
            OfflineJournalBundleError::InvalidSnapshot {
                operation_id: operation_id.clone(),
                reason: error.to_string(),
            }
        })?;
        if entry.envelope().expected_revision > MAX_JCS_SAFE_REVISION {
            return Err(OfflineJournalBundleError::RevisionOutOfRange {
                operation_id,
                field: "expected_revision",
            });
        }
        match entry.state() {
            crate::offline_journal::JournalState::Received {
                applied_revision, ..
            } if *applied_revision > MAX_JCS_SAFE_REVISION => {
                return Err(OfflineJournalBundleError::RevisionOutOfRange {
                    operation_id,
                    field: "applied_revision",
                });
            }
            crate::offline_journal::JournalState::ConflictNeedsReview {
                current_revision, ..
            } if *current_revision > MAX_JCS_SAFE_REVISION => {
                return Err(OfflineJournalBundleError::RevisionOutOfRange {
                    operation_id,
                    field: "current_revision",
                });
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OfflineJournalBundleUnsignedV1 {
    schema_version: u16,
    owner_scope: String,
    records: Vec<OfflineJournalBundleRecordV1>,
}

/// Canonical, versioned export of one owner's offline journal.
///
/// The public codec rejects unknown fields and invalid snapshots, requires
/// operation-ID sorting, and protects the complete unsigned manifest using
/// SHA-256 over canonical JCS UTF-8. This is corruption detection only: an
/// attacker who can rewrite the bundle can recompute the unkeyed digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfflineJournalBundleV1 {
    pub schema_version: u16,
    pub owner_scope: String,
    pub records: Vec<OfflineJournalBundleRecordV1>,
    pub integrity_sha256: String,
}

impl OfflineJournalBundleV1 {
    /// Creates a normalized bundle. Records are sorted by operation ID before
    /// the checksum is calculated; duplicate IDs remain a hard error.
    pub fn create(
        owner_scope: impl Into<String>,
        mut records: Vec<OfflineJournalBundleRecordV1>,
    ) -> Result<Self, OfflineJournalBundleError> {
        let owner_scope = owner_scope.into();
        if owner_scope.trim().is_empty() {
            return Err(OfflineJournalBundleError::EmptyOwnerScope);
        }
        records.sort_by(|left, right| left.operation_id().cmp(right.operation_id()));
        let bundle = Self {
            schema_version: OFFLINE_JOURNAL_BUNDLE_SCHEMA_VERSION,
            owner_scope,
            records,
            integrity_sha256: String::new(),
        };
        bundle.validate_records()?;
        let integrity_sha256 = bundle.computed_integrity_sha256()?;
        Ok(Self {
            integrity_sha256,
            ..bundle
        })
    }

    /// Parses a bundle, revalidates every snapshot and verifies the manifest
    /// digest. Input whitespace/property order do not affect validation.
    pub fn from_json(input: &str) -> Result<Self, OfflineJournalBundleError> {
        let bundle: Self = serde_json::from_str(input)
            .map_err(|error| OfflineJournalBundleError::InvalidJson(error.to_string()))?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Validates the bundle envelope, every record, uniqueness/order and digest.
    pub fn validate(&self) -> Result<(), OfflineJournalBundleError> {
        if self.schema_version != OFFLINE_JOURNAL_BUNDLE_SCHEMA_VERSION {
            return Err(OfflineJournalBundleError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.owner_scope.trim().is_empty() {
            return Err(OfflineJournalBundleError::EmptyOwnerScope);
        }
        self.validate_records()?;
        let actual = self.computed_integrity_sha256()?;
        if actual != self.integrity_sha256 {
            return Err(OfflineJournalBundleError::IntegrityDigestMismatch {
                expected: self.integrity_sha256.clone(),
                actual,
            });
        }
        Ok(())
    }

    /// Refuses to treat a self-declared bundle owner as proof of identity.
    /// Caller must supply the owner scope from its authenticated account context.
    pub fn validate_for_owner(
        &self,
        authenticated_owner_scope: &str,
    ) -> Result<(), OfflineJournalBundleError> {
        self.validate()?;
        if authenticated_owner_scope.trim().is_empty()
            || authenticated_owner_scope != self.owner_scope
        {
            return Err(OfflineJournalBundleError::OwnerScopeMismatch {
                expected: authenticated_owner_scope.to_owned(),
                bundle: self.owner_scope.clone(),
            });
        }
        Ok(())
    }

    /// Serializes as canonical JCS JSON suitable for storage or transfer.
    pub fn to_json(&self) -> Result<String, OfflineJournalBundleError> {
        self.validate()?;
        let json = serde_json::to_string(self)
            .map_err(|error| OfflineJournalBundleError::Serialization(error.to_string()))?;
        let canonical = canonicalize_payload_json(&json)
            .map_err(|error| OfflineJournalBundleError::Serialization(error.to_string()))?;
        String::from_utf8(canonical)
            .map_err(|error| OfflineJournalBundleError::Serialization(error.to_string()))
    }

    fn validate_records(&self) -> Result<(), OfflineJournalBundleError> {
        let mut seen = HashSet::with_capacity(self.records.len());
        let mut seen_scoped_idempotency = HashSet::with_capacity(self.records.len());
        let mut previous: Option<&str> = None;
        for record in &self.records {
            record.validate()?;
            let operation_id = record.operation_id();
            if !seen.insert(operation_id.to_owned()) {
                return Err(OfflineJournalBundleError::DuplicateOperationId(
                    operation_id.to_owned(),
                ));
            }
            let scoped_key = (
                record.operation_namespace.clone(),
                record.snapshot.envelope.idempotency_key.clone(),
            );
            if !seen_scoped_idempotency.insert(scoped_key) {
                return Err(OfflineJournalBundleError::DuplicateScopedIdempotencyKey {
                    operation_id: operation_id.to_owned(),
                });
            }
            if previous.is_some_and(|value| value >= operation_id) {
                return Err(OfflineJournalBundleError::NonCanonicalRecordOrder);
            }
            previous = Some(operation_id);
        }
        Ok(())
    }

    /// Returns the exact RFC 8785 canonical UTF-8 bytes covered by the bundle's
    /// integrity digest. This allows independent implementations to verify the
    /// manifest bytes rather than trusting a Rust-specific serialization path.
    ///
    /// The returned bytes exclude integrity_sha256 itself. The caller should
    /// hash these bytes with SHA-256 and compare to integrity_sha256. This method
    /// validates schema, owner scope and every record, but intentionally does not
    /// require the stored integrity digest to match so verifiers can inspect it.
    pub fn canonical_unsigned_manifest_utf8(&self) -> Result<Vec<u8>, OfflineJournalBundleError> {
        if self.schema_version != OFFLINE_JOURNAL_BUNDLE_SCHEMA_VERSION {
            return Err(OfflineJournalBundleError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.owner_scope.trim().is_empty() {
            return Err(OfflineJournalBundleError::EmptyOwnerScope);
        }
        self.validate_records()?;
        self.serialize_unsigned_manifest_utf8()
    }

    fn serialize_unsigned_manifest_utf8(&self) -> Result<Vec<u8>, OfflineJournalBundleError> {
        let unsigned = OfflineJournalBundleUnsignedV1 {
            schema_version: self.schema_version,
            owner_scope: self.owner_scope.clone(),
            records: self.records.clone(),
        };
        let json = serde_json::to_string(&unsigned)
            .map_err(|error| OfflineJournalBundleError::Serialization(error.to_string()))?;
        canonicalize_payload_json(&json)
            .map_err(|error| OfflineJournalBundleError::Serialization(error.to_string()))
    }

    fn computed_integrity_sha256(&self) -> Result<String, OfflineJournalBundleError> {
        let canonical = self.serialize_unsigned_manifest_utf8()?;
        let digest = Sha256::digest(&canonical);
        Ok(lower_hex(&digest))
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
    use super::{
        OFFLINE_JOURNAL_BUNDLE_SCHEMA_VERSION, OfflineJournalBundleError,
        OfflineJournalBundleRecordV1, OfflineJournalBundleV1,
    };
    use crate::{
        offline_journal::{
            JournalEntry, JournalEntrySnapshotV1, OFFLINE_JOURNAL_SCHEMA_VERSION,
            OperationEnvelope, RequiredNullableOperationId,
        },
        offline_payload_digest::{canonical_payload_digest_sha256, canonicalize_payload_json},
    };
    use serde::Deserialize;
    use sha2::{Digest, Sha256};

    const PAYLOAD: &str = r#"{"label":"transfer","amount":12}"#;

    fn record(operation_id: &str, namespace: &str) -> OfflineJournalBundleRecordV1 {
        let envelope = OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: operation_id.to_owned(),
            aggregate_id: format!("aggregate-{operation_id}"),
            expected_revision: 0,
            payload_digest_sha256: canonical_payload_digest_sha256(PAYLOAD).unwrap(),
            idempotency_key: format!("idem-{operation_id}"),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        };
        let entry = JournalEntry::prepare(envelope, PAYLOAD).unwrap();
        OfflineJournalBundleRecordV1 {
            operation_namespace: namespace.to_owned(),
            snapshot: entry.to_snapshot().unwrap(),
        }
    }

    fn bundle() -> OfflineJournalBundleV1 {
        OfflineJournalBundleV1::create(
            "tenant-001",
            vec![
                record("op-2", "work-order.apply"),
                record("op-1", "work-order.apply"),
            ],
        )
        .unwrap()
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BundleConformanceVector {
        schema_version: u16,
        description: String,
        algorithm: String,
        unsigned_manifest_canonical_utf8: String,
        integrity_sha256: String,
        canonical_bundle_utf8: String,
    }

    #[test]
    fn frozen_bundle_vector_matches_unsigned_bytes_digest_and_final_wire_bytes() {
        let vector: BundleConformanceVector =
            serde_json::from_str(include_str!("../fixtures/offline-journal-bundle-v1.json"))
                .expect("bundle vector must match its frozen schema");
        assert_eq!(vector.schema_version, 1);
        assert!(!vector.description.trim().is_empty());
        assert!(!vector.algorithm.trim().is_empty());

        let bundle = OfflineJournalBundleV1::from_json(&vector.canonical_bundle_utf8)
            .expect("frozen bundle must pass the public validator");
        assert_eq!(bundle.integrity_sha256, vector.integrity_sha256);
        assert_eq!(
            String::from_utf8(bundle.canonical_unsigned_manifest_utf8().unwrap()).unwrap(),
            vector.unsigned_manifest_canonical_utf8
        );
        assert_eq!(bundle.to_json().unwrap(), vector.canonical_bundle_utf8);
    }

    #[test]
    fn creation_sorts_records_and_serializes_canonical_json() {
        let bundle = bundle();
        assert_eq!(bundle.schema_version, OFFLINE_JOURNAL_BUNDLE_SCHEMA_VERSION);
        assert_eq!(bundle.records[0].operation_id(), "op-1");
        assert_eq!(bundle.records[1].operation_id(), "op-2");
        let json = bundle.to_json().unwrap();
        assert_eq!(OfflineJournalBundleV1::from_json(&json).unwrap(), bundle);
        assert!(json.starts_with(r#"{"integrity_sha256":"#));
    }

    #[test]
    fn whitespace_and_property_order_do_not_change_bundle_validation() {
        let expected = bundle();
        let json = expected.to_json().unwrap();
        let with_whitespace = format!(" \n  {json}\n");
        assert_eq!(
            OfflineJournalBundleV1::from_json(&with_whitespace).unwrap(),
            expected
        );

        // Reorder the top-level properties deliberately. Canonicalization,
        // not the incoming source spelling, defines the digest bytes.
        let decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        let reordered = format!(
            r#"{{"schema_version":{},"records":{},"owner_scope":{},"integrity_sha256":{}}}"#,
            decoded["schema_version"],
            decoded["records"],
            decoded["owner_scope"],
            decoded["integrity_sha256"]
        );
        assert_ne!(reordered, json);
        assert_eq!(
            OfflineJournalBundleV1::from_json(&reordered).unwrap(),
            expected
        );
        assert_eq!(
            OfflineJournalBundleV1::from_json(&reordered)
                .unwrap()
                .to_json()
                .unwrap(),
            json
        );
    }

    #[test]
    fn any_record_tampering_fails_integrity_or_snapshot_validation() {
        let json = bundle().to_json().unwrap();
        let mut decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        decoded["records"][0]["operation_namespace"] =
            serde_json::Value::String("work-order.cancel".to_owned());
        let tampered = serde_json::to_string(&decoded).unwrap();
        assert!(OfflineJournalBundleV1::from_json(&tampered).is_err());

        let mut decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        decoded["records"][0]["snapshot"]["canonical_payload_json"] =
            serde_json::Value::String(r#"{"label":"tampered","amount":12}"#.to_owned());
        let tampered = serde_json::to_string(&decoded).unwrap();
        assert!(OfflineJournalBundleV1::from_json(&tampered).is_err());
    }

    fn reseal_untrusted_bundle_json(mut value: serde_json::Value) -> String {
        value
            .as_object_mut()
            .expect("bundle JSON must be an object")
            .remove("integrity_sha256");
        let unsigned_json = serde_json::to_string(&value).unwrap();
        let canonical_unsigned = canonicalize_payload_json(&unsigned_json).unwrap();
        let digest = Sha256::digest(&canonical_unsigned);
        let digest_hex = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        value["integrity_sha256"] = serde_json::Value::String(digest_hex);
        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn recomputed_outer_digest_cannot_authorize_an_invalid_inner_snapshot() {
        let valid_json = bundle().to_json().unwrap();

        // A malicious editor can recompute the unkeyed outer checksum, so the
        // inner journal snapshot must be independently validated after reseal.
        let mut forged_state: serde_json::Value = serde_json::from_str(&valid_json).unwrap();
        forged_state["records"][0]["snapshot"]["state"] = serde_json::json!({
            "state": "received",
            "receipt_ref": "",
            "applied_revision": 0
        });
        let resealed_invalid_state = reseal_untrusted_bundle_json(forged_state);
        assert!(matches!(
            OfflineJournalBundleV1::from_json(&resealed_invalid_state),
            Err(OfflineJournalBundleError::InvalidSnapshot { .. })
        ));

        let mut forged_payload: serde_json::Value = serde_json::from_str(&valid_json).unwrap();
        forged_payload["records"][0]["snapshot"]["canonical_payload_json"] =
            serde_json::Value::String(r#"{"a":999}"#.to_owned());
        let resealed_invalid_payload = reseal_untrusted_bundle_json(forged_payload);
        assert!(matches!(
            OfflineJournalBundleV1::from_json(&resealed_invalid_payload),
            Err(OfflineJournalBundleError::InvalidSnapshot { .. })
        ));

        let mut forged_owner: serde_json::Value = serde_json::from_str(&valid_json).unwrap();
        forged_owner["owner_scope"] = serde_json::Value::String("tenant-002".to_owned());
        let resealed_wrong_owner = reseal_untrusted_bundle_json(forged_owner);
        let decoded = OfflineJournalBundleV1::from_json(&resealed_wrong_owner).unwrap();
        assert!(matches!(
            decoded.validate_for_owner("tenant-001"),
            Err(OfflineJournalBundleError::OwnerScopeMismatch { .. })
        ));
    }

    #[test]
    fn bundle_scope_must_match_authenticated_import_scope() {
        let bundle = bundle();
        assert!(bundle.validate_for_owner("tenant-001").is_ok());
        assert!(matches!(
            bundle.validate_for_owner("tenant-002"),
            Err(OfflineJournalBundleError::OwnerScopeMismatch { .. })
        ));
        assert!(bundle.validate_for_owner("").is_err());
    }

    #[test]
    fn duplicate_operation_ids_and_empty_namespaces_are_rejected() {
        let duplicate = record("same-op", "work-order.apply");
        let duplicate_again = duplicate.clone();
        assert!(matches!(
            OfflineJournalBundleV1::create("tenant-001", vec![duplicate, duplicate_again]),
            Err(OfflineJournalBundleError::DuplicateOperationId(_))
        ));
        assert!(matches!(
            OfflineJournalBundleV1::create("tenant-001", vec![record("op-1", "  ")]),
            Err(OfflineJournalBundleError::EmptyNamespace { .. })
        ));
    }

    #[test]
    fn stale_or_invalid_snapshot_state_is_not_exportable() {
        let mut invalid = record("op-invalid", "work-order.apply");
        invalid.snapshot.schema_version += 1;
        assert!(matches!(
            OfflineJournalBundleV1::create("tenant-001", vec![invalid]),
            Err(OfflineJournalBundleError::InvalidSnapshot { .. })
        ));

        let mut invalid = record("op-invalid", "work-order.apply");
        invalid.snapshot.state = crate::offline_journal::JournalState::Received {
            receipt_ref: String::new(),
            applied_revision: 0,
        };
        let snapshot: JournalEntrySnapshotV1 = invalid.snapshot.clone();
        assert!(JournalEntry::from_snapshot(snapshot).is_err());
        assert!(OfflineJournalBundleV1::create("tenant-001", vec![invalid]).is_err());
    }

    #[test]
    fn duplicate_scoped_idempotency_keys_are_rejected_but_namespaces_are_isolated() {
        let first = record("op-1", "work-order.apply");
        let mut second = record("op-2", "work-order.apply");
        second.snapshot.envelope.idempotency_key = first.snapshot.envelope.idempotency_key.clone();
        assert!(matches!(
            OfflineJournalBundleV1::create("tenant-001", vec![first.clone(), second]),
            Err(OfflineJournalBundleError::DuplicateScopedIdempotencyKey { .. })
        ));

        let mut other_namespace = record("op-3", "work-order.cancel");
        other_namespace.snapshot.envelope.idempotency_key =
            first.snapshot.envelope.idempotency_key.clone();
        assert!(OfflineJournalBundleV1::create("tenant-001", vec![first, other_namespace]).is_ok());
    }

    #[test]
    fn revision_fields_outside_jcs_safe_integer_range_fail_explicitly() {
        let mut invalid = record("op-large-revision", "work-order.apply");
        invalid.snapshot.envelope.expected_revision = u64::MAX;
        assert!(matches!(
            OfflineJournalBundleV1::create("tenant-001", vec![invalid]),
            Err(OfflineJournalBundleError::RevisionOutOfRange {
                field: "expected_revision",
                ..
            })
        ));
    }

    #[test]
    fn unknown_bundle_fields_and_schema_versions_fail_closed() {
        let json = bundle().to_json().unwrap();
        let unknown = json.trim_end_matches('}').to_owned() + r#","surprise":true}"#;
        assert!(OfflineJournalBundleV1::from_json(&unknown).is_err());

        let mut invalid = bundle();
        invalid.schema_version += 1;
        assert!(matches!(
            invalid.validate(),
            Err(OfflineJournalBundleError::UnsupportedSchemaVersion(_))
        ));
    }

    #[test]
    fn empty_owner_scope_is_rejected() {
        assert!(matches!(
            OfflineJournalBundleV1::create(" ", vec![record("op-1", "work-order.apply")]),
            Err(OfflineJournalBundleError::EmptyOwnerScope)
        ));
    }
}
