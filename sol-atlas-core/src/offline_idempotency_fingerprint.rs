// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Versioned, deterministic request fingerprint for an authoritative idempotency ledger.
//!
//! The payload digest commits to the canonical body only. This fingerprint
//! additionally binds the tenant/namespace scope and all immutable operation
//! fields so a service can detect idempotency-key reuse across a different
//! aggregate, revision, operation, or conflict lineage.
//!
//! This is a shared fingerprint primitive/reference contract, not a server-side
//! idempotency ledger. The authoritative service must derive tenant and route
//! scope from authenticated request context and enforce unique constraints and
//! effect/outcome atomicity in its database.

use std::{error::Error, fmt};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::offline_journal::{OperationEnvelope, RequiredNullableOperationId};
use crate::offline_payload_digest::{CanonicalPayload, canonicalize_payload_json};

/// Version of the request-fingerprint material encoding, independent of the
/// journal envelope and snapshot schema versions.
pub const REQUEST_FINGERPRINT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug)]
pub enum RequestFingerprintError {
    EmptyScope(&'static str),
    InvalidEnvelope(String),
    PayloadCanonicalization(String),
    PayloadDigestMismatch { expected: String, actual: String },
    FingerprintMaterialSerialization(String),
}

impl fmt::Display for RequestFingerprintError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScope(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidEnvelope(reason) => {
                write!(formatter, "request envelope is invalid: {reason}")
            }
            Self::PayloadCanonicalization(reason) => {
                write!(formatter, "request payload cannot be represented under strict JCS: {reason}")
            }
            Self::PayloadDigestMismatch { expected, actual } => write!(
                formatter,
                "request payload digest mismatch: expected {expected}, computed {actual}"
            ),
            Self::FingerprintMaterialSerialization(reason) => {
                write!(formatter, "request fingerprint material serialization failed: {reason}")
            }
        }
    }
}

impl Error for RequestFingerprintError {}

#[derive(Serialize)]
struct RequestFingerprintMaterial<'a> {
    fingerprint_schema_version: u16,
    tenant_scope: &'a str,
    operation_namespace: &'a str,
    envelope_schema_version: u16,
    operation_id: &'a str,
    aggregate_id: &'a str,
    /// Decimal string preserves exact u64 revisions without placing unsafe
    /// integers into the JCS/ECMAScript number model.
    expected_revision_decimal: String,
    payload_digest_sha256: &'a str,
    /// Explicit JSON null is part of the fingerprint material when no parent exists.
    supersedes_operation_id: Option<&'a str>,
}

/// Builds canonical JCS UTF-8 bytes for the scoped immutable request fingerprint.
///
/// The idempotency key is intentionally not repeated inside the fingerprint:
/// it is the separately indexed ledger key. The server must store the key and
/// fingerprint together under a unique scope. All other immutable request
/// bindings are included.
pub fn canonical_request_fingerprint_material(
    tenant_scope: &str,
    operation_namespace: &str,
    envelope: &OperationEnvelope,
    payload_json: &str,
) -> Result<Vec<u8>, RequestFingerprintError> {
    if tenant_scope.trim().is_empty() {
        return Err(RequestFingerprintError::EmptyScope("tenant_scope"));
    }
    if operation_namespace.trim().is_empty() {
        return Err(RequestFingerprintError::EmptyScope("operation_namespace"));
    }
    envelope
        .validate()
        .map_err(|error| RequestFingerprintError::InvalidEnvelope(error.to_string()))?;

    let payload = CanonicalPayload::from_json(payload_json).map_err(|error| {
        RequestFingerprintError::PayloadCanonicalization(error.to_string())
    })?;
    if payload.sha256_hex() != envelope.payload_digest_sha256 {
        return Err(RequestFingerprintError::PayloadDigestMismatch {
            expected: envelope.payload_digest_sha256.clone(),
            actual: payload.sha256_hex().to_owned(),
        });
    }

    let supersedes_operation_id = match &envelope.supersedes_operation_id {
        RequiredNullableOperationId::Value(value) => Some(value.as_str()),
        RequiredNullableOperationId::Null(()) => None,
    };
    let material = RequestFingerprintMaterial {
        fingerprint_schema_version: REQUEST_FINGERPRINT_SCHEMA_VERSION,
        tenant_scope,
        operation_namespace,
        envelope_schema_version: envelope.schema_version,
        operation_id: &envelope.operation_id,
        aggregate_id: &envelope.aggregate_id,
        expected_revision_decimal: envelope.expected_revision.to_string(),
        payload_digest_sha256: &envelope.payload_digest_sha256,
        supersedes_operation_id,
    };
    let material_json = serde_json::to_string(&material).map_err(|error| {
        RequestFingerprintError::FingerprintMaterialSerialization(error.to_string())
    })?;
    canonicalize_payload_json(&material_json).map_err(|error| {
        RequestFingerprintError::PayloadCanonicalization(error.to_string())
    })
}

/// Computes lowercase hexadecimal SHA-256 over canonical request-fingerprint bytes.
pub fn canonical_request_fingerprint_sha256(
    tenant_scope: &str,
    operation_namespace: &str,
    envelope: &OperationEnvelope,
    payload_json: &str,
) -> Result<String, RequestFingerprintError> {
    let material = canonical_request_fingerprint_material(
        tenant_scope,
        operation_namespace,
        envelope,
        payload_json,
    )?;
    let digest = Sha256::digest(&material);
    Ok(lower_hex(&digest))
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
        RequestFingerprintError, canonical_request_fingerprint_material,
        canonical_request_fingerprint_sha256,
    };
    use crate::offline_journal::{
        OFFLINE_JOURNAL_SCHEMA_VERSION, OperationEnvelope, RequiredNullableOperationId,
    };
    use crate::offline_payload_digest::canonical_payload_digest_sha256;

    const PAYLOAD: &str = r#"{ "b": 2, "a": 1.0 }"#;

    fn envelope() -> OperationEnvelope {
        OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: "op-001".to_owned(),
            aggregate_id: "job-001".to_owned(),
            expected_revision: 7,
            payload_digest_sha256: canonical_payload_digest_sha256(PAYLOAD).unwrap(),
            idempotency_key: "idem-001".to_owned(),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        }
    }

    fn fingerprint(envelope: &OperationEnvelope, payload: &str) -> String {
        canonical_request_fingerprint_sha256(
            "tenant-001",
            "work-order.apply",
            envelope,
            payload,
        )
        .unwrap()
    }

    #[test]
    fn frozen_fingerprint_vector_matches_exact_canonical_bytes_and_hash() {
        let envelope = envelope();
        let material = canonical_request_fingerprint_material(
            "tenant-001",
            "work-order.apply",
            &envelope,
            PAYLOAD,
        )
        .unwrap();
        let material = String::from_utf8(material).unwrap();
        assert_eq!(
            material,
            r#"{"aggregate_id":"job-001","envelope_schema_version":2,"expected_revision_decimal":"7","fingerprint_schema_version":1,"operation_id":"op-001","operation_namespace":"work-order.apply","payload_digest_sha256":"43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777","supersedes_operation_id":null,"tenant_scope":"tenant-001"}"#
        );
        assert_eq!(
            fingerprint(&envelope, PAYLOAD),
            "b9bbfd55bce0d1c29224872f235b2750d124ab0f5bf5ae3382dcc4072c82a1d3"
        );
    }

    #[test]
    fn whitespace_and_property_order_do_not_change_request_fingerprint() {
        let envelope = envelope();
        assert_eq!(
            fingerprint(&envelope, PAYLOAD),
            fingerprint(&envelope, r#"{"a":1,"b":2}"#)
        );
    }

    #[test]
    fn every_immutable_binding_and_scope_changes_the_fingerprint() {
        let base = envelope();
        let original = fingerprint(&base, PAYLOAD);

        let mut changed = base.clone();
        changed.operation_id = "op-other".to_owned();
        assert_ne!(fingerprint(&changed, PAYLOAD), original);

        let mut changed = base.clone();
        changed.aggregate_id = "job-other".to_owned();
        assert_ne!(fingerprint(&changed, PAYLOAD), original);

        let mut changed = base.clone();
        changed.expected_revision += 1;
        assert_ne!(fingerprint(&changed, PAYLOAD), original);

        let mut changed = base.clone();
        changed.supersedes_operation_id =
            RequiredNullableOperationId::Value("op-parent".to_owned());
        assert_ne!(fingerprint(&changed, PAYLOAD), original);

        assert_ne!(
            canonical_request_fingerprint_sha256(
                "tenant-other",
                "work-order.apply",
                &base,
                PAYLOAD
            )
            .unwrap(),
            original
        );
        assert_ne!(
            canonical_request_fingerprint_sha256(
                "tenant-001",
                "work-order.cancel",
                &base,
                PAYLOAD
            )
            .unwrap(),
            original
        );
    }

    #[test]
    fn idempotency_key_is_a_separate_ledger_key_not_fingerprint_material() {
        let mut original = envelope();
        let original_fingerprint = fingerprint(&original, PAYLOAD);
        original.idempotency_key = "idem-retry-key".to_owned();
        assert_eq!(fingerprint(&original, PAYLOAD), original_fingerprint);
    }

    #[test]
    fn changed_payload_with_the_old_envelope_digest_fails_closed() {
        let envelope = envelope();
        assert!(matches!(
            canonical_request_fingerprint_sha256(
                "tenant-001",
                "work-order.apply",
                &envelope,
                r#"{"a":1,"b":3}"#
            ),
            Err(RequestFingerprintError::PayloadDigestMismatch { .. })
        ));
    }

    #[test]
    fn empty_scope_is_rejected() {
        let envelope = envelope();
        assert!(matches!(
            canonical_request_fingerprint_sha256("", "work-order.apply", &envelope, PAYLOAD),
            Err(RequestFingerprintError::EmptyScope("tenant_scope"))
        ));
        assert!(matches!(
            canonical_request_fingerprint_sha256("tenant-001", " ", &envelope, PAYLOAD),
            Err(RequestFingerprintError::EmptyScope("operation_namespace"))
        ));
    }

    #[test]
    fn exact_u64_revision_is_encoded_as_decimal_text() {
        let mut envelope = envelope();
        envelope.expected_revision = u64::MAX;
        let material = canonical_request_fingerprint_material(
            "tenant-001",
            "work-order.apply",
            &envelope,
            PAYLOAD,
        )
        .unwrap();
        let material = String::from_utf8(material).unwrap();
        assert!(material.contains(r#""expected_revision_decimal":"18446744073709551615""#));
    }
}
