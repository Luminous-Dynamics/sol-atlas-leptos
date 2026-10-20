// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! RFC 8785 payload canonicalization and SHA-256 digest helpers.
//!
//! The caller supplies the original JSON text, not a previously parsed
//! `serde_json::Value`: converting text to a generic value can erase duplicate
//! object member names before a strict canonicalizer can reject them.
//! Digests are computed over the canonical UTF-8 bytes, never over ad hoc
//! serializer output. The canonicalizer rejects duplicate keys and numbers
//! outside its documented interoperable integer policy.

use std::fmt;

use sha2::{Digest, Sha256};

/// Errors from canonicalization or verification of a payload digest.
#[derive(Debug)]
pub enum PayloadDigestError {
    /// Input is not accepted by the selected RFC 8785 canonicalization profile.
    Canonicalization(codec_jcs::JcsError),
    /// The expected digest is not 64 lowercase hexadecimal characters.
    InvalidExpectedDigest,
    /// The payload's computed SHA-256 does not equal the expected digest.
    DigestMismatch,
}

impl fmt::Display for PayloadDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonicalization(error) => {
                write!(formatter, "payload is not valid under the JCS profile: {error}")
            }
            Self::InvalidExpectedDigest => {
                formatter.write_str("expected payload digest must be 64 lowercase hex characters")
            }
            Self::DigestMismatch => formatter.write_str("payload digest does not match canonical payload"),
        }
    }
}

impl std::error::Error for PayloadDigestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Canonicalization(error) => Some(error),
            Self::InvalidExpectedDigest | Self::DigestMismatch => None,
        }
    }
}

/// Canonicalize untrusted JSON to the exact UTF-8 bytes used by the protocol.
///
/// This raw-text entry point intentionally rejects duplicate object member
/// names, malformed/trailing JSON, and integers outside the interoperable
/// range accepted by the pinned `reallyme-codec-jcs` implementation.
///
/// It performs no Unicode normalization: canonically equivalent Unicode
/// strings remain distinct JSON strings, as required by RFC 8785.
pub fn canonicalize_payload_json(input: &str) -> Result<Vec<u8>, PayloadDigestError> {
    codec_jcs::canonicalize_json_text(input)
        .map(String::into_bytes)
        .map_err(PayloadDigestError::Canonicalization)
}

/// Compute lowercase SHA-256 hex over the RFC 8785 canonical UTF-8 bytes.
pub fn payload_digest_sha256(input: &str) -> Result<String, PayloadDigestError> {
    let canonical = canonicalize_payload_json(input)?;
    let digest = Sha256::digest(&canonical);
    Ok(encode_lower_hex(&digest))
}

/// Verify an envelope's declared lowercase SHA-256 digest against raw payload
/// text. This does not authenticate the sender or service; it only establishes
/// that the local payload and the declared digest agree under this profile.
pub fn verify_payload_digest_sha256(
    input: &str,
    expected_digest: &str,
) -> Result<(), PayloadDigestError> {
    if !is_lower_hex_digest(expected_digest) {
        return Err(PayloadDigestError::InvalidExpectedDigest);
    }
    let actual = payload_digest_sha256(input)?;
    if actual == expected_digest {
        Ok(())
    } else {
        Err(PayloadDigestError::DigestMismatch)
    }
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[(byte >> 4) as usize]));
        encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        PayloadDigestError, canonicalize_payload_json, payload_digest_sha256,
        verify_payload_digest_sha256,
    };

    #[test]
    fn object_order_and_whitespace_do_not_change_digest() {
        let first = payload_digest_sha256(r#"{"b":2,"a":1}"#).unwrap();
        let second = payload_digest_sha256(" { \"a\": 1.0, \"b\": 2 } ").unwrap();

        assert_eq!(first, "43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777");
        assert_eq!(first, second);
    }

    #[test]
    fn number_serialization_follows_jcs_profile() {
        let canonical =
            canonicalize_payload_json(r#"{"numbers":[1e21,0.000001,-0.0],"amount":1.0,"from":"€"}"#)
                .unwrap();
        assert_eq!(
            String::from_utf8(canonical).unwrap(),
            r#"{"amount":1,"from":"€","numbers":[1e+21,0.000001,0]}"#
        );
    }

    #[test]
    fn duplicate_object_member_is_rejected_before_hashing() {
        assert!(matches!(
            payload_digest_sha256(r#"{"operation":"one","operation":"two"}"#),
            Err(PayloadDigestError::Canonicalization(_))
        ));
    }

    #[test]
    fn out_of_profile_integer_is_rejected_before_hashing() {
        assert!(matches!(
            payload_digest_sha256(r#"{"asset_revision":9007199254740992}"#),
            Err(PayloadDigestError::Canonicalization(_))
        ));
    }

    #[test]
    fn declared_digest_is_verified_and_mismatch_fails_closed() {
        let digest = "43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777";
        assert!(verify_payload_digest_sha256(r#"{"a":1,"b":2}"#, digest).is_ok());
        assert!(matches!(
            verify_payload_digest_sha256(r#"{"a":1,"b":3}"#, digest),
            Err(PayloadDigestError::DigestMismatch)
        ));
        assert!(matches!(
            verify_payload_digest_sha256(r#"{"a":1}"#, "ABCD"),
            Err(PayloadDigestError::InvalidExpectedDigest)
        ));
    }

    #[test]
    fn unicode_is_not_normalized_during_canonicalization() {
        let composed = payload_digest_sha256(r#"{"value":"é"}"#).unwrap();
        let decomposed = payload_digest_sha256(r#"{"value":"é"}"#).unwrap();
        assert_ne!(composed, decomposed);
    }
}
