// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline Ed25519 admission for EV route evidence references.
//!
//! The trust store is supplied separately from the route plan. A plan cannot
//! insert or replace its own trust anchors. Signatures bind the exact evidence
//! reference, digest, source class, validity interval, role, subject and issuer.
//! Authentication proves who signed those claims; it does not prove that the
//! underlying sensor, model or physical world is correct.

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const TRUST_STORE_SCHEMA_VERSION: &str = "ev-route-evidence-trust-store/v0.1";
const SIGNATURE_DOMAIN: &[u8] = b"LUMINOUS_EV_ROUTE_EVIDENCE_ATTESTATION_V0_1\0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAttestation {
    pub evidence_role: String,
    pub subject_id: String,
    pub issuer_id: String,
    pub issuer_role: String,
    pub key_id: String,
    pub issued_at_unix_s: i64,
    #[serde(default)]
    pub signature_ed25519_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedEvidenceKey {
    pub key_id: String,
    pub issuer_id: String,
    pub issuer_role: String,
    pub public_key_ed25519_hex: String,
    pub valid_from_unix_s: i64,
    #[serde(default)]
    pub valid_until_unix_s: Option<i64>,
    pub revoked: bool,
    pub authorized_evidence_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceTrustStore {
    pub schema_version: String,
    pub policy_id: String,
    pub trusted_keys: Vec<TrustedEvidenceKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceVerification {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Inputs bound to one evidence consumer field and the independently admitted
/// trust policy. Keeping these values in one request prevents argument-order
/// mistakes when adding new evidence dimensions.
#[derive(Debug, Clone, Copy)]
pub struct EvidenceVerificationRequest<'a> {
    pub reference_id: &'a str,
    pub digest_sha256: &'a str,
    pub source_class: &'a str,
    pub observed_at_unix_s: Option<i64>,
    pub valid_until_unix_s: Option<i64>,
    pub attestation: Option<&'a EvidenceAttestation>,
    pub expected_evidence_role: &'a str,
    pub expected_subject_id: &'a str,
    pub assessed_at_unix_s: i64,
}

impl EvidenceVerification {
    pub fn authenticated(&self) -> bool {
        self.status == "AUTHENTICATED"
    }
}

#[derive(Serialize)]
struct SignedEvidenceClaims<'a> {
    schema_version: &'static str,
    reference_id: &'a str,
    digest_sha256: &'a str,
    source_class: &'a str,
    observed_at_unix_s: Option<i64>,
    valid_until_unix_s: Option<i64>,
    evidence_role: &'a str,
    subject_id: &'a str,
    issuer_id: &'a str,
    issuer_role: &'a str,
    key_id: &'a str,
    issued_at_unix_s: i64,
}

/// Serialize the exact versioned claim bytes used by producers and verifiers.
/// The domain separator prevents this signature from being replayed as another
/// Luminous protocol's signature.
pub fn signature_message(
    reference_id: &str,
    digest_sha256: &str,
    source_class: &str,
    observed_at_unix_s: Option<i64>,
    valid_until_unix_s: Option<i64>,
    attestation: &EvidenceAttestation,
) -> Result<Vec<u8>, String> {
    let claims = SignedEvidenceClaims {
        schema_version: TRUST_STORE_SCHEMA_VERSION,
        reference_id,
        digest_sha256,
        source_class,
        observed_at_unix_s,
        valid_until_unix_s,
        evidence_role: &attestation.evidence_role,
        subject_id: &attestation.subject_id,
        issuer_id: &attestation.issuer_id,
        issuer_role: &attestation.issuer_role,
        key_id: &attestation.key_id,
        issued_at_unix_s: attestation.issued_at_unix_s,
    };
    let mut message = SIGNATURE_DOMAIN.to_vec();
    let payload = serde_json::to_vec(&claims)
        .map_err(|error| format!("cannot serialize signed evidence claims: {error}"))?;
    message.extend_from_slice(&payload);
    Ok(message)
}

fn decode_hex<const N: usize>(value: &str, label: &str) -> Result<[u8; N], String> {
    if value.len() != N * 2
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{label} must be {N} bytes of lowercase hexadecimal"
        ));
    }
    let mut output = [0u8; N];
    for (index, byte) in output.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&value[start..start + 2], 16)
            .map_err(|_| format!("{label} contains invalid hexadecimal"))?;
    }
    Ok(output)
}

fn failed(
    status: &str,
    reason: &str,
    attestation: Option<&EvidenceAttestation>,
) -> EvidenceVerification {
    EvidenceVerification {
        status: status.to_owned(),
        issuer_id: attestation.map(|value| value.issuer_id.clone()),
        issuer_role: attestation.map(|value| value.issuer_role.clone()),
        key_id: attestation.map(|value| value.key_id.clone()),
        reason: Some(reason.to_owned()),
    }
}

pub fn validate_trust_store(store: &EvidenceTrustStore) -> Result<(), String> {
    if store.schema_version != TRUST_STORE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported trust-store schema_version: expected {TRUST_STORE_SCHEMA_VERSION}"
        ));
    }
    if store.policy_id.trim().is_empty() {
        return Err("trust-store policy_id must not be empty".into());
    }
    if store.trusted_keys.is_empty() {
        return Err("trust store must contain at least one explicitly admitted key".into());
    }

    let mut key_ids = BTreeSet::new();
    for key in &store.trusted_keys {
        if key.key_id.trim().is_empty()
            || key.issuer_id.trim().is_empty()
            || key.issuer_role.trim().is_empty()
            || !key_ids.insert(key.key_id.clone())
        {
            return Err(format!(
                "empty or duplicate trusted evidence key: {}",
                key.key_id
            ));
        }
        if key
            .valid_until_unix_s
            .is_some_and(|end| end < key.valid_from_unix_s)
        {
            return Err(format!(
                "trusted key {} has an inverted validity window",
                key.key_id
            ));
        }
        let bytes = decode_hex::<32>(&key.public_key_ed25519_hex, "trusted Ed25519 public key")?;
        VerifyingKey::from_bytes(&bytes).map_err(|_| {
            format!(
                "trusted key {} is not a valid Ed25519 public key",
                key.key_id
            )
        })?;

        if key.authorized_evidence_roles.is_empty() {
            return Err(format!(
                "trusted key {} has no authorized evidence roles",
                key.key_id
            ));
        }
        let mut roles = BTreeSet::new();
        for role in &key.authorized_evidence_roles {
            if role.trim().is_empty() || !roles.insert(role) {
                return Err(format!(
                    "trusted key {} has an empty or duplicate evidence role",
                    key.key_id
                ));
            }
        }
    }
    Ok(())
}

/// Verify signer identity, key status, validity interval, role authority,
/// subject binding and the Ed25519 signature over the exact evidence claim.
pub fn verify_evidence(
    request: EvidenceVerificationRequest<'_>,
    store: &EvidenceTrustStore,
) -> EvidenceVerification {
    let EvidenceVerificationRequest {
        reference_id,
        digest_sha256,
        source_class,
        observed_at_unix_s,
        valid_until_unix_s,
        attestation,
        expected_evidence_role,
        expected_subject_id,
        assessed_at_unix_s,
    } = request;
    let Some(attestation) = attestation else {
        return failed(
            "UNATTESTED",
            "evidence reference has no Ed25519 attestation",
            None,
        );
    };
    if attestation.evidence_role != expected_evidence_role
        || attestation.subject_id != expected_subject_id
    {
        return failed(
            "REJECTED_SCOPE",
            "signed evidence role or subject does not match the exact consumer field and asset",
            Some(attestation),
        );
    }
    if attestation.issued_at_unix_s > assessed_at_unix_s {
        return failed(
            "REJECTED_FUTURE_ATTESTATION",
            "attestation issue time is after the assessment time",
            Some(attestation),
        );
    }

    let Some(key) = store
        .trusted_keys
        .iter()
        .find(|key| key.key_id == attestation.key_id)
    else {
        return failed(
            "REJECTED_UNKNOWN_KEY",
            "attestation key_id is not present in the separately supplied trust store",
            Some(attestation),
        );
    };
    if key.revoked {
        return failed(
            "REJECTED_REVOKED_KEY",
            "attestation key is revoked under the supplied trust policy",
            Some(attestation),
        );
    }
    if key.issuer_id != attestation.issuer_id || key.issuer_role != attestation.issuer_role {
        return failed(
            "REJECTED_SIGNER_BINDING",
            "signed issuer identity or role does not match the trusted key record",
            Some(attestation),
        );
    }
    if !key
        .authorized_evidence_roles
        .iter()
        .any(|role| role == expected_evidence_role)
    {
        return failed(
            "REJECTED_ROLE_AUTHORITY",
            "trusted key is not authorized to attest this evidence role",
            Some(attestation),
        );
    }
    if attestation.issued_at_unix_s < key.valid_from_unix_s
        || key
            .valid_until_unix_s
            .is_some_and(|end| attestation.issued_at_unix_s > end)
    {
        return failed(
            "REJECTED_KEY_VALIDITY",
            "attestation was issued outside the trusted key validity interval",
            Some(attestation),
        );
    }

    let public_key =
        match decode_hex::<32>(&key.public_key_ed25519_hex, "trusted Ed25519 public key") {
            Ok(value) => value,
            Err(reason) => return failed("REJECTED_KEY_MALFORMED", &reason, Some(attestation)),
        };
    let signature_hex = match attestation.signature_ed25519_hex.as_deref() {
        Some(value) if !value.is_empty() => value,
        _ => {
            return failed(
                "UNATTESTED",
                "Ed25519 signature is absent",
                Some(attestation),
            );
        }
    };
    let signature_bytes = match decode_hex::<64>(signature_hex, "Ed25519 signature") {
        Ok(value) => value,
        Err(reason) => return failed("REJECTED_SIGNATURE", &reason, Some(attestation)),
    };
    let verifying_key = match VerifyingKey::from_bytes(&public_key) {
        Ok(value) => value,
        Err(_) => {
            return failed(
                "REJECTED_KEY_MALFORMED",
                "trusted Ed25519 public key is invalid",
                Some(attestation),
            );
        }
    };
    let signature = Signature::from_bytes(&signature_bytes);
    let message = match signature_message(
        reference_id,
        digest_sha256,
        source_class,
        observed_at_unix_s,
        valid_until_unix_s,
        attestation,
    ) {
        Ok(value) => value,
        Err(reason) => return failed("REJECTED_SIGNATURE", &reason, Some(attestation)),
    };
    if verifying_key.verify_strict(&message, &signature).is_err() {
        return failed(
            "REJECTED_SIGNATURE",
            "signature does not verify over the exact evidence claims",
            Some(attestation),
        );
    }

    EvidenceVerification {
        status: "AUTHENTICATED".into(),
        issuer_id: Some(attestation.issuer_id.clone()),
        issuer_role: Some(attestation.issuer_role.clone()),
        key_id: Some(attestation.key_id.clone()),
        reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn fixture_key() -> SigningKey {
        SigningKey::from_bytes(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ])
    }

    fn fixture_store() -> EvidenceTrustStore {
        let key = fixture_key();
        EvidenceTrustStore {
            schema_version: TRUST_STORE_SCHEMA_VERSION.into(),
            policy_id: "test-only-ev-route-trust-policy".into(),
            trusted_keys: vec![TrustedEvidenceKey {
                key_id: "test-key-001".into(),
                issuer_id: "test-issuer-001".into(),
                issuer_role: "engineering_evidence_issuer".into(),
                public_key_ed25519_hex: key
                    .verifying_key()
                    .to_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
                valid_from_unix_s: 0,
                valid_until_unix_s: Some(20_000),
                revoked: false,
                authorized_evidence_roles: vec!["route_state".into()],
            }],
        }
    }

    fn signed_attestation() -> (
        String,
        String,
        Option<i64>,
        Option<i64>,
        EvidenceAttestation,
    ) {
        let key = fixture_key();
        let reference = "test:route-state";
        let digest = "ab".repeat(32);
        let observed = Some(800);
        let valid_until = Some(2_000);
        let mut attestation = EvidenceAttestation {
            evidence_role: "route_state".into(),
            subject_id: "LEG-1".into(),
            issuer_id: "test-issuer-001".into(),
            issuer_role: "engineering_evidence_issuer".into(),
            key_id: "test-key-001".into(),
            issued_at_unix_s: 900,
            signature_ed25519_hex: None,
        };
        let message = signature_message(
            reference,
            &digest,
            "observed",
            observed,
            valid_until,
            &attestation,
        )
        .expect("signed evidence message");
        attestation.signature_ed25519_hex = Some(
            key.sign(&message)
                .to_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        );
        (reference.into(), digest, observed, valid_until, attestation)
    }

    fn verify_fixture(
        reference: &str,
        digest: &str,
        attestation: Option<&EvidenceAttestation>,
        store: &EvidenceTrustStore,
        expected_role: &str,
        expected_subject: &str,
    ) -> EvidenceVerification {
        verify_evidence(
            EvidenceVerificationRequest {
                reference_id: reference,
                digest_sha256: digest,
                source_class: "observed",
                observed_at_unix_s: Some(800),
                valid_until_unix_s: Some(2_000),
                attestation,
                expected_evidence_role: expected_role,
                expected_subject_id: expected_subject,
                assessed_at_unix_s: 1_000,
            },
            store,
        )
    }

    #[test]
    fn exact_claims_and_trusted_role_authenticate() {
        let store = fixture_store();
        let (reference, digest, _, _, attestation) = signed_attestation();
        let result = verify_fixture(
            &reference,
            &digest,
            Some(&attestation),
            &store,
            "route_state",
            "LEG-1",
        );
        assert!(result.authenticated());
    }

    #[test]
    fn missing_signature_is_unattested() {
        let store = fixture_store();
        let (reference, digest, _, _, mut attestation) = signed_attestation();
        attestation.signature_ed25519_hex = None;
        let result = verify_fixture(&reference, &digest, Some(&attestation), &store, "route_state", "LEG-1");
        assert_eq!(result.status, "UNATTESTED");
    }

    #[test]
    fn digest_tampering_breaks_the_signature() {
        let store = fixture_store();
        let (reference, _digest, _, _, attestation) = signed_attestation();
        let result = verify_fixture(
            &reference,
            &"cd".repeat(32),
            Some(&attestation),
            &store,
            "route_state",
            "LEG-1",
        );
        assert_eq!(result.status, "REJECTED_SIGNATURE");
    }

    #[test]
    fn wrong_subject_or_role_is_rejected_even_with_a_valid_signature() {
        let store = fixture_store();
        let (reference, digest, observed, valid_until, attestation) = signed_attestation();
        let result = verify_fixture(
            &reference,
            &digest,
            Some(&attestation),
            &store,
            "charger_status",
            "CHARGER-1",
        );
        assert_eq!(result.status, "REJECTED_SCOPE");
    }

    #[test]
    fn revoked_key_cannot_authenticate() {
        let mut store = fixture_store();
        store.trusted_keys[0].revoked = true;
        let (reference, digest, observed, valid_until, attestation) = signed_attestation();
        let result = verify_fixture(&reference, &digest, Some(&attestation), &store, "route_state", "LEG-1");
        assert_eq!(result.status, "REJECTED_REVOKED_KEY");
    }

    #[test]
    fn trust_store_rejects_duplicate_keys_and_roles() {
        let mut store = fixture_store();
        store.trusted_keys.push(store.trusted_keys[0].clone());
        assert!(validate_trust_store(&store).is_err());
    }
}
