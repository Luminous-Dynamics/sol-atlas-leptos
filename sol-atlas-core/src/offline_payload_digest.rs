// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Strict, reproducible JSON payload hashing for the offline journal.
//!
//! Protocol v2 pins payload identity to RFC 8785 JCS canonical UTF-8 bytes,
//! then SHA-256. Do not hash ad-hoc `serde_json` or JavaScript serialization.
//! Raw JSON is accepted so duplicate keys and incorrectly rounded numeric
//! parsing can be rejected before they become an ambiguous operation identity.
//!
//! The journal reducer can validate the *shape* of a digest string, but it
//! cannot verify a payload without the original payload bytes. Call
//! `verify_payload_digest` at the payload ingress boundary before persistence
//! or dispatch, and independently at the authoritative service boundary.

use std::fmt;

use sha2::{Digest, Sha256};

/// Errors from canonicalization or digest verification.
#[derive(Debug)]
pub enum PayloadDigestError {
    /// The payload cannot be represented faithfully under the RFC 8785 profile.
    Canonicalization(jcs::Error),
    /// The expected digest is not 64 lowercase hexadecimal characters.
    InvalidExpectedDigest,
    /// The payload canonicalizes successfully, but does not match the envelope digest.
    DigestMismatch { expected: String, actual: String },
}

impl fmt::Display for PayloadDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonicalization(error) => {
                write!(formatter, "payload is not valid strict JCS input: {error}")
            }
            Self::InvalidExpectedDigest => {
                formatter.write_str("expected digest must be 64 lowercase hexadecimal characters")
            }
            Self::DigestMismatch { expected, actual } => write!(
                formatter,
                "payload digest mismatch: expected {expected}, computed {actual}"
            ),
        }
    }
}

impl std::error::Error for PayloadDigestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Canonicalization(error) => Some(error),
            Self::InvalidExpectedDigest | Self::DigestMismatch { .. } => None,
        }
    }
}

/// Canonicalizes raw JSON to the exact RFC 8785 UTF-8 byte sequence.
///
/// This rejects duplicate keys, malformed JSON, non-finite/out-of-model
/// numbers, and integers that cannot be represented exactly under the JCS
/// IEEE-754 number model. Store large exact integers as strings, as prescribed
/// by RFC 8785 Appendix D.
pub fn canonicalize_payload_json(
    payload_json: &str,
) -> Result<Vec<u8>, PayloadDigestError> {
    jcs::canonicalize_to_vec(payload_json).map_err(PayloadDigestError::Canonicalization)
}

/// Computes lowercase hexadecimal SHA-256 over canonical RFC 8785 UTF-8 bytes.
pub fn canonical_payload_digest_sha256(
    payload_json: &str,
) -> Result<String, PayloadDigestError> {
    let canonical = canonicalize_payload_json(payload_json)?;
    let digest = Sha256::digest(&canonical);
    Ok(lower_hex(&digest))
}

/// Verifies the immutable envelope digest against the original raw JSON payload.
///
/// Verification must happen before the local journal is persisted or a request
/// is dispatched. The server must independently repeat this check; client
/// verification does not replace server-side idempotency or revision checks.
pub fn verify_payload_digest(
    payload_json: &str,
    expected_digest: &str,
) -> Result<(), PayloadDigestError> {
    if !is_sha256_lower_hex(expected_digest) {
        return Err(PayloadDigestError::InvalidExpectedDigest);
    }

    let actual = canonical_payload_digest_sha256(payload_json)?;
    if actual == expected_digest {
        Ok(())
    } else {
        Err(PayloadDigestError::DigestMismatch {
            expected: expected_digest.to_owned(),
            actual,
        })
    }
}

fn is_sha256_lower_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
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
        canonical_payload_digest_sha256, canonicalize_payload_json, verify_payload_digest,
        PayloadDigestError,
    };
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DigestCorpus {
        schema_version: u16,
        vectors: Vec<DigestVector>,
        invalid_cases: Vec<InvalidCase>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DigestVector {
        case_id: String,
        payload_json: String,
        canonical_utf8: String,
        sha256: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct InvalidCase {
        case_id: String,
        payload_json: String,
    }

    #[test]
    fn frozen_rfc8785_digest_corpus_matches_byte_for_byte() {
        let corpus: DigestCorpus = serde_json::from_str(include_str!(
            "../fixtures/offline-payload-digest-v1.json"
        ))
        .expect("payload digest corpus must match its frozen schema");
        assert_eq!(corpus.schema_version, 1);
        assert!(!corpus.vectors.is_empty());

        for vector in corpus.vectors {
            let canonical = canonicalize_payload_json(&vector.payload_json)
                .unwrap_or_else(|error| panic!("{}: {error}", vector.case_id));
            assert_eq!(
                String::from_utf8(canonical).unwrap(),
                vector.canonical_utf8,
                "canonical bytes for {}",
                vector.case_id
            );
            assert_eq!(
                canonical_payload_digest_sha256(&vector.payload_json).unwrap(),
                vector.sha256,
                "digest for {}",
                vector.case_id
            );
            assert!(
                verify_payload_digest(&vector.payload_json, &vector.sha256).is_ok(),
                "verification for {}",
                vector.case_id
            );
        }
    }

    #[test]
    fn strict_jcs_rejects_duplicate_keys_and_out_of_model_values() {
        let corpus: DigestCorpus = serde_json::from_str(include_str!(
            "../fixtures/offline-payload-digest-v1.json"
        ))
        .expect("payload digest corpus must match its frozen schema");

        assert!(!corpus.invalid_cases.is_empty());
        for case in corpus.invalid_cases {
            assert!(
                canonicalize_payload_json(&case.payload_json).is_err(),
                "invalid JCS input unexpectedly accepted: {}",
                case.case_id
            );
        }
    }

    #[test]
    fn whitespace_and_property_order_do_not_change_digest() {
        let left = r#"{ "b": 2, "a": 1.0 }"#;
        let right = r#"{"a":1,"b":2}"#;
        assert_eq!(
            canonical_payload_digest_sha256(left).unwrap(),
            canonical_payload_digest_sha256(right).unwrap()
        );
    }

    #[test]
    fn digest_verification_fails_closed_on_bad_or_mismatched_digest() {
        let payload = r#"{"a":1}"#;
        assert!(matches!(
            verify_payload_digest(payload, "ABC"),
            Err(PayloadDigestError::InvalidExpectedDigest)
        ));
        assert!(matches!(
            verify_payload_digest(payload, &"0".repeat(64)),
            Err(PayloadDigestError::DigestMismatch { .. })
        ));
    }
}
