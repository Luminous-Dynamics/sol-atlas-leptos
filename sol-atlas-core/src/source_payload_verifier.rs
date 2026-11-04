// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Independent byte-level verification for source descriptor claims.
//!
//! This module hashes caller-supplied byte buffers. It does not retrieve URLs,
//! establish where bytes came from, authenticate source owners or attestations,
//! adjudicate rights, prove source-universe completeness, or validate domain
//! semantics. A matching digest proves only that supplied bytes match the
//! digest recorded in the exact descriptor catalogue.

use crate::built_environment_case::BuiltEnvironmentCase;
use crate::built_environment_source_descriptor::{
    DescriptorDisposition, PayloadDigestBasis, SourceDescriptorCatalog,
    SourceRecordKind, validate_source_descriptor_catalog,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const RECEIPT_SCHEMA: &str = "luminous.source-payload-byte-verification-receipt.v0.1";
const PAYLOAD_INPUT_DOMAIN: &[u8] = b"LUMINOUS-SOURCE-PAYLOAD-INPUT-MANIFEST-V0.1\0";
const SCHEMA_INPUT_DOMAIN: &[u8] = b"LUMINOUS-SOURCE-SCHEMA-INPUT-MANIFEST-V0.1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ByteVerificationDisposition {
    CompleteForSuppliedClaims,
    IncompleteOrIndeterminate,
    InvalidInput,
}

impl ByteVerificationDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompleteForSuppliedClaims => "complete_for_supplied_claims",
            Self::IncompleteOrIndeterminate => "incomplete_or_indeterminate",
            Self::InvalidInput => "invalid_input",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ByteVerificationFinding {
    pub severity: String,
    pub code: String,
    pub record_key: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordByteVerification {
    pub record_kind: String,
    pub record_key: String,
    pub digest_basis: String,
    pub expected_payload_sha256: Option<String>,
    pub computed_payload_sha256: Option<String>,
    pub expected_byte_length: Option<u64>,
    pub actual_byte_length: Option<u64>,
    pub payload_result: String,
    pub schema_id: Option<String>,
    pub schema_version: Option<String>,
    pub expected_schema_sha256: Option<String>,
    pub computed_schema_sha256: Option<String>,
    pub schema_result: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayloadByteVerificationReceipt {
    pub schema_version: String,
    pub case_id: String,
    pub catalog_id: String,
    pub catalog_sha256: String,
    pub case_representation_sha256: String,
    pub descriptor_validation_disposition: String,
    pub payload_input_manifest_sha256: String,
    pub schema_input_manifest_sha256: String,
    pub records: Vec<RecordByteVerification>,
    pub record_count: usize,
    pub payload_bytes_verified: usize,
    pub payload_bytes_missing: usize,
    pub payload_digests_unavailable: usize,
    pub payload_digest_mismatches: usize,
    pub payload_length_mismatches: usize,
    pub schema_bytes_verified: usize,
    pub schema_bytes_missing: usize,
    pub schema_digest_mismatches: usize,
    pub unreferenced_payload_input_count: usize,
    pub unreferenced_schema_input_count: usize,
    pub disposition: ByteVerificationDisposition,
    pub payload_byte_verification: String,
    pub source_owner_authentication: String,
    pub source_attestation_authentication: String,
    pub rights_authentication: String,
    pub source_universe_completeness: String,
    pub engineering_correctness: String,
    pub action_authority: String,
    pub findings: Vec<ByteVerificationFinding>,
    pub limitations: Vec<String>,
    pub receipt_sha256: String,
}

#[derive(Default)]
struct Findings {
    rows: BTreeSet<ByteVerificationFinding>,
}

impl Findings {
    fn invalid(&mut self, code: &str, key: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(ByteVerificationFinding {
            severity: "invalid".into(),
            code: code.into(),
            record_key: key.into(),
            detail: detail.into(),
        });
    }

    fn incomplete(&mut self, code: &str, key: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(ByteVerificationFinding {
            severity: "incomplete".into(),
            code: code.into(),
            record_key: key.into(),
            detail: detail.into(),
        });
    }

    fn disposition(&self) -> ByteVerificationDisposition {
        if self.rows.iter().any(|finding| finding.severity == "invalid") {
            ByteVerificationDisposition::InvalidInput
        } else if !self.rows.is_empty() {
            ByteVerificationDisposition::IncompleteOrIndeterminate
        } else {
            ByteVerificationDisposition::CompleteForSuppliedClaims
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn append_field(encoded: &mut Vec<u8>, value: &[u8]) -> Result<(), String> {
    let length = u64::try_from(value.len())
        .map_err(|_| "input-manifest field exceeds u64 length".to_owned())?;
    encoded.extend_from_slice(&length.to_be_bytes());
    encoded.extend_from_slice(value);
    Ok(())
}

fn payload_input_manifest_digest(
    payloads: &BTreeMap<String, Vec<u8>>,
) -> Result<String, String> {
    let mut encoded = PAYLOAD_INPUT_DOMAIN.to_vec();
    for (key, bytes) in payloads {
        append_field(&mut encoded, key.as_bytes())?;
        append_field(&mut encoded, &(bytes.len() as u64).to_be_bytes())?;
        append_field(&mut encoded, sha256_hex(bytes).as_bytes())?;
    }
    Ok(sha256_hex(&encoded))
}

fn schema_input_manifest_digest(
    schemas: &BTreeMap<(String, String), Vec<u8>>,
) -> Result<String, String> {
    let mut encoded = SCHEMA_INPUT_DOMAIN.to_vec();
    for ((schema_id, schema_version), bytes) in schemas {
        append_field(&mut encoded, schema_id.as_bytes())?;
        append_field(&mut encoded, schema_version.as_bytes())?;
        append_field(&mut encoded, &(bytes.len() as u64).to_be_bytes())?;
        append_field(&mut encoded, sha256_hex(bytes).as_bytes())?;
    }
    Ok(sha256_hex(&encoded))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn record_key(kind: SourceRecordKind, id: &str) -> String {
    kind.case_key(id)
}

/// Verify payload and schema bytes for one exact case-bound descriptor
/// catalogue.
///
/// payload_inputs is keyed by the descriptor's stable typed record key
/// (evidence:<id>, artifact:<id>, engineering_receipt:<id>, or decision:<id>).
/// schema_inputs is keyed by (schema_id, schema_version). The verifier hashes
/// supplied byte slices and does not fetch any network resource or trust an
/// input filename/URL.
///
/// This is a bounded in-memory v0.1 implementation. A streaming Read/async
/// interface and content-addressed object-store adapters are future work.
pub fn verify_catalog_payload_bytes(
    case: &BuiltEnvironmentCase,
    catalog: &SourceDescriptorCatalog,
    payload_inputs: &BTreeMap<String, Vec<u8>>,
    schema_inputs: &BTreeMap<(String, String), Vec<u8>>,
) -> PayloadByteVerificationReceipt {
    let descriptor_validation = validate_source_descriptor_catalog(case, catalog);
    let catalog_bytes = serde_json::to_vec(catalog).unwrap_or_default();
    let case_bytes = serde_json::to_vec(case).unwrap_or_default();
    let catalog_sha256 = sha256_hex(&catalog_bytes);
    let case_sha256 = sha256_hex(&case_bytes);
    let payload_manifest_sha256 =
        payload_input_manifest_digest(payload_inputs).unwrap_or_default();
    let schema_manifest_sha256 =
        schema_input_manifest_digest(schema_inputs).unwrap_or_default();

    let mut findings = Findings::default();
    if descriptor_validation.disposition == DescriptorDisposition::InvalidInput {
        findings.invalid(
            "DESCRIPTOR_CATALOG_INVALID",
            "catalog",
            "payload verification will not run against an invalid descriptor catalogue",
        );
    } else if descriptor_validation.disposition == DescriptorDisposition::IncompleteOrIndeterminate {
        findings.incomplete(
            "DESCRIPTOR_CATALOG_INCOMPLETE",
            "catalog",
            "byte verification is bounded by an incomplete descriptor catalogue",
        );
    }

    let mut records = Vec::with_capacity(catalog.descriptors.len());
    let mut referenced_payload_keys = BTreeSet::new();
    let mut referenced_schema_keys = BTreeSet::new();
    let mut payload_bytes_verified = 0usize;
    let mut payload_bytes_missing = 0usize;
    let mut payload_digests_unavailable = 0usize;
    let mut payload_digest_mismatches = 0usize;
    let mut payload_length_mismatches = 0usize;
    let mut schema_bytes_verified = 0usize;
    let mut schema_bytes_missing = 0usize;
    let mut schema_digest_mismatches = 0usize;

    if descriptor_validation.disposition != DescriptorDisposition::InvalidInput {
        for descriptor in &catalog.descriptors {
            let key = record_key(descriptor.record_kind, &descriptor.case_record_id);
            referenced_payload_keys.insert(key.clone());

            let expected_payload_sha256 = descriptor.source_payload.sha256.clone();
            let expected_byte_length = descriptor.source_payload.byte_length;
            let digest_basis = descriptor.source_payload.digest_basis.as_str().to_owned();
            let payload_bytes = payload_inputs.get(&key);
            let (computed_payload_sha256, actual_byte_length, payload_result) =
                match (expected_payload_sha256.as_deref(), payload_bytes) {
                    (Some(expected), Some(bytes)) => {
                        let computed = sha256_hex(bytes);
                        let actual_length = bytes.len() as u64;
                        let expected_digest_valid = valid_sha256(expected);
                        let digest_matches = expected_digest_valid && computed == expected;
                        let length_matches = expected_byte_length
                            .is_none_or(|expected_length| expected_length == actual_length);

                        if !expected_digest_valid {
                            payload_digest_mismatches += 1;
                            findings.invalid(
                                "INVALID_EXPECTED_PAYLOAD_DIGEST",
                                &key,
                                "descriptor payload digest is not lowercase SHA-256",
                            );
                        } else if !digest_matches {
                            payload_digest_mismatches += 1;
                            findings.invalid(
                                "SOURCE_PAYLOAD_DIGEST_MISMATCH",
                                &key,
                                "SHA-256 of supplied payload bytes does not match the descriptor claim",
                            );
                        }

                        if !length_matches {
                            payload_length_mismatches += 1;
                            findings.invalid(
                                "SOURCE_PAYLOAD_LENGTH_MISMATCH",
                                &key,
                                "supplied payload byte length does not match the descriptor claim",
                            );
                        }

                        if descriptor.source_payload.digest_basis == PayloadDigestBasis::Unavailable {
                            payload_digests_unavailable += 1;
                            findings.invalid(
                                "UNAVAILABLE_PAYLOAD_HAS_DIGEST",
                                &key,
                                "unavailable digest basis unexpectedly contains a digest",
                            );
                            (Some(computed), Some(actual_length), "invalid_unavailable_digest_state")
                        } else if !expected_digest_valid {
                            (Some(computed), Some(actual_length), "invalid_expected_digest")
                        } else if !digest_matches {
                            (Some(computed), Some(actual_length), "digest_mismatch")
                        } else if !length_matches {
                            (Some(computed), Some(actual_length), "length_mismatch")
                        } else {
                            payload_bytes_verified += 1;
                            let result = match descriptor.source_payload.digest_basis {
                                PayloadDigestBasis::LocallyRetrievedAndHashed => {
                                    "verified_supplied_bytes_match_local_digest_claim"
                                }
                                PayloadDigestBasis::SourceOwnerAttested => {
                                    "supplied_bytes_match_owner_attested_digest"
                                }
                                PayloadDigestBasis::ExternalReference => {
                                    "verified_supplied_bytes_match_external_digest_claim"
                                }
                                PayloadDigestBasis::Unavailable => {
                                    "invalid_unavailable_digest_state"
                                }
                            };
                            (Some(computed), Some(actual_length), result)
                        }
                    }
                    (Some(_), None) => {
                        payload_bytes_missing += 1;
                        findings.incomplete(
                            "SOURCE_PAYLOAD_BYTES_NOT_SUPPLIED",
                            &key,
                            "descriptor contains a source digest but no corresponding byte buffer was supplied",
                        );
                        (None, None, "missing_bytes")
                    }
                    (None, Some(bytes)) => {
                        payload_digests_unavailable += 1;
                        findings.incomplete(
                            "BYTES_OBSERVED_WITHOUT_SOURCE_DIGEST_CLAIM",
                            &key,
                            "supplied bytes were hashed for observation only; the computed digest is not promoted to a source-native claim",
                        );
                        (
                            Some(sha256_hex(bytes)),
                            Some(bytes.len() as u64),
                            "observed_bytes_without_source_digest_claim",
                        )
                    }
                    (None, None) => {
                        payload_digests_unavailable += 1;
                        findings.incomplete(
                            "SOURCE_PAYLOAD_DIGEST_UNAVAILABLE",
                            &key,
                            "the descriptor has no source payload digest claim",
                        );
                        (None, None, "source_digest_unavailable")
                    }
                };

            let schema_id = descriptor.source_payload.schema_id.clone();
            let schema_version = descriptor.source_payload.schema_version.clone();
            let expected_schema_sha256 = descriptor.source_payload.schema_sha256.clone();
            let schema_key = schema_id
                .as_ref()
                .zip(schema_version.as_ref())
                .map(|(id, version)| (id.clone(), version.clone()));
            let (computed_schema_sha256, schema_result) =
                match (schema_key.as_ref(), expected_schema_sha256.as_deref()) {
                    (Some(schema_key), Some(expected)) => {
                        referenced_schema_keys.insert(schema_key.clone());
                        match schema_inputs.get(schema_key) {
                            Some(bytes) => {
                                let computed = sha256_hex(bytes);
                                if computed == expected && valid_sha256(expected) {
                                    schema_bytes_verified += 1;
                                    (Some(computed), "schema_bytes_match_declared_digest")
                                } else {
                                    schema_digest_mismatches += 1;
                                    findings.invalid(
                                        "SOURCE_SCHEMA_DIGEST_MISMATCH",
                                        &key,
                                        "SHA-256 of supplied schema bytes does not match the descriptor claim",
                                    );
                                    (Some(computed), "schema_digest_mismatch")
                                }
                            }
                            None => {
                                schema_bytes_missing += 1;
                                findings.incomplete(
                                    "SOURCE_SCHEMA_BYTES_NOT_SUPPLIED",
                                    &key,
                                    "schema ID/version and digest are present, but schema bytes were not supplied",
                                );
                                (None, "schema_bytes_missing")
                            }
                        }
                    }
                    (Some(schema_key), None) => {
                        referenced_schema_keys.insert(schema_key.clone());
                        schema_bytes_missing += 1;
                        findings.incomplete(
                            "SOURCE_SCHEMA_DIGEST_UNAVAILABLE",
                            &key,
                            "schema ID/version are present but no schema digest is declared",
                        );
                        (None, "schema_digest_unavailable")
                    }
                    _ => {
                        schema_bytes_missing += 1;
                        findings.incomplete(
                            "SOURCE_SCHEMA_IDENTITY_UNAVAILABLE",
                            &key,
                            "schema ID/version are unavailable, so schema bytes cannot be independently bound",
                        );
                        (None, "schema_identity_unavailable")
                    }
                };

            records.push(RecordByteVerification {
                record_kind: descriptor.record_kind.node_type().into(),
                record_key: key,
                digest_basis,
                expected_payload_sha256,
                computed_payload_sha256,
                expected_byte_length,
                actual_byte_length,
                payload_result: payload_result.into(),
                schema_id,
                schema_version,
                expected_schema_sha256,
                computed_schema_sha256,
                schema_result: schema_result.into(),
            });
        }
    }

    let unreferenced_payload_keys = payload_inputs
        .keys()
        .filter(|key| !referenced_payload_keys.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    for key in &unreferenced_payload_keys {
        findings.incomplete(
            "UNREFERENCED_PAYLOAD_BYTE_INPUT",
            key,
            "payload byte input does not match a descriptor record key and was not used",
        );
    }

    let unreferenced_schema_keys = schema_inputs
        .keys()
        .filter(|key| !referenced_schema_keys.contains(*key))
        .map(|(id, version)| format!("{id}@{version}"))
        .collect::<Vec<_>>();
    for key in &unreferenced_schema_keys {
        findings.incomplete(
            "UNREFERENCED_SCHEMA_BYTE_INPUT",
            key,
            "schema byte input is not referenced by any descriptor and was not used",
        );
    }

    records.sort_by(|left, right| left.record_key.cmp(&right.record_key));
    let disposition = findings.disposition();
    let payload_byte_verification = if payload_bytes_verified > 0
        && payload_bytes_missing == 0
        && payload_digests_unavailable == 0
        && payload_digest_mismatches == 0
        && payload_length_mismatches == 0
        && disposition == ByteVerificationDisposition::CompleteForSuppliedClaims
    {
        "complete_for_supplied_payload_claims"
    } else if payload_bytes_verified > 0 {
        "partial_or_incomplete"
    } else {
        "not_established"
    };

    let mut receipt = PayloadByteVerificationReceipt {
        schema_version: RECEIPT_SCHEMA.into(),
        case_id: case.case_id.clone(),
        catalog_id: catalog.catalog_id.clone(),
        catalog_sha256,
        case_representation_sha256: case_sha256,
        descriptor_validation_disposition: descriptor_validation.disposition.as_str().into(),
        payload_input_manifest_sha256: payload_manifest_sha256,
        schema_input_manifest_sha256: schema_manifest_sha256,
        records,
        record_count: catalog.descriptors.len(),
        payload_bytes_verified,
        payload_bytes_missing,
        payload_digests_unavailable,
        payload_digest_mismatches,
        payload_length_mismatches,
        schema_bytes_verified,
        schema_bytes_missing,
        schema_digest_mismatches,
        unreferenced_payload_input_count: unreferenced_payload_keys.len(),
        unreferenced_schema_input_count: unreferenced_schema_keys.len(),
        disposition,
        payload_byte_verification: payload_byte_verification.into(),
        source_owner_authentication: "not_performed".into(),
        source_attestation_authentication: "not_performed".into(),
        rights_authentication: "not_performed".into(),
        source_universe_completeness: "not_performed".into(),
        engineering_correctness: "not_performed".into(),
        action_authority: "none".into(),
        findings: findings.rows.into_iter().collect(),
        limitations: vec![
            "The verifier hashes caller-supplied byte buffers; it does not retrieve source bytes or authenticate their origin.".into(),
            "A matching payload digest proves byte equality with the descriptor claim, not source-owner identity or attestation authenticity.".into(),
            "Rights, signatures, trust roots, revocation, source-universe completeness, and engineering correctness are outside this verifier.".into(),
            "The v0.1 implementation is bounded and in-memory; streaming and object-store readers are future work.".into(),
            "Schema byte hashing proves exact schema artifact bytes only; it does not prove parser conformance or domain semantics.".into(),
        ],
        receipt_sha256: String::new(),
    };
    let receipt_bytes = serde_json::to_vec(&receipt).unwrap_or_default();
    receipt.receipt_sha256 = sha256_hex(&receipt_bytes);
    receipt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_environment_case::{
        BuiltEnvironmentCase, DataRights, EvidenceRecord, EvidenceState,
    };
    use crate::built_environment_source_descriptor::{
        NativeIdentityComponent, PayloadDigestBasis, SourceAttestationDescriptor,
        SourceDescriptorCatalog, SourceObservationTime, SourcePayloadDescriptor,
        SourceRecordDescriptor, SourceRecordKind, SourceRightsDescriptor, SourceRightsState,
        SourceSnapshotDescriptor,
    };

    fn digest(bytes: &[u8]) -> String {
        sha256_hex(bytes)
    }

    fn case(payload: &[u8]) -> BuiltEnvironmentCase {
        BuiltEnvironmentCase {
            case_id: "case-byte-verify-001".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence: vec![EvidenceRecord {
                evidence_id: "evidence-001".into(),
                subject_id: "design-001".into(),
                field_path: "geometry.revision".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: Some(digest(payload)),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_000),
            }],
            artifacts: vec![],
            engineering_receipts: vec![],
            decisions: vec![],
        }
    }

    fn catalog(
        case: &BuiltEnvironmentCase,
        bytes: &[u8],
        schema_bytes: &[u8],
    ) -> SourceDescriptorCatalog {
        let schema_sha256 = digest(schema_bytes);
        let payload_sha256 = digest(bytes);
        SourceDescriptorCatalog {
            schema_version: "luminous.built-environment-source-descriptor-catalog.v0.1".into(),
            catalog_id: "catalog-byte-verify-001".into(),
            case_id: case.case_id.clone(),
            case_representation_sha256: sha256_hex(&serde_json::to_vec(case).unwrap()),
            captured_at: "2026-11-04T12:00:00Z".into(),
            descriptors: vec![SourceRecordDescriptor {
                record_kind: SourceRecordKind::Evidence,
                case_record_id: "evidence-001".into(),
                owner_namespace: "urn:synthetic:survey-owner".into(),
                native_identity_components: vec![
                    NativeIdentityComponent { name: "native_id".into(), value: "survey-registry".into() },
                    NativeIdentityComponent { name: "native_record_id".into(), value: "evidence-001".into() },
                    NativeIdentityComponent { name: "native_revision".into(), value: "1".into() },
                    NativeIdentityComponent { name: "field_path".into(), value: "geometry.revision".into() },
                    NativeIdentityComponent { name: "subject_id".into(), value: "design-001".into() },
                    NativeIdentityComponent { name: "jurisdiction".into(), value: "synthetic-jurisdiction".into() },
                ],
                source_payload: SourcePayloadDescriptor {
                    sha256: Some(payload_sha256.clone()),
                    digest_basis: PayloadDigestBasis::LocallyRetrievedAndHashed,
                    bytes_retrieved: true,
                    byte_length: Some(bytes.len() as u64),
                    payload_object_ref: Some(format!("cas:sha256:{payload_sha256}")),
                    media_type: Some("application/json".into()),
                    schema_id: Some("urn:synthetic:schema:evidence".into()),
                    schema_version: Some("1.0".into()),
                    schema_sha256: Some(schema_sha256),
                },
                source_snapshot: SourceSnapshotDescriptor {
                    source_id: "survey-registry".into(),
                    snapshot_id: "snapshot-001".into(),
                    snapshot_sha256: digest(b"snapshot"),
                    captured_at: "2026-11-04T11:00:00Z".into(),
                    retrieved_at: "2026-11-04T11:01:00Z".into(),
                },
                observation_time: SourceObservationTime {
                    observed_at: Some("2026-11-04T10:59:00Z".into()),
                    recorded_at: "2026-11-04T11:00:00Z".into(),
                },
                rights: SourceRightsDescriptor {
                    state: SourceRightsState::Verified,
                    permitted_uses: vec!["impact_analysis".into()],
                    rights_evidence_ref: Some("synthetic:rights:001".into()),
                },
                attestation: SourceAttestationDescriptor {
                    statement_ref: Some("synthetic:statement:001".into()),
                    key_id: Some("urn:synthetic:key:001".into()),
                    trust_evaluation: "not_evaluated_by_descriptor_catalog".into(),
                },
            }],
            claim_ceiling: "descriptor_integrity_only".into(),
            action_authority: "none".into(),
            limitations: vec!["Synthetic fixture only.".into()],
        }
    }

    fn inputs(
        payload: &[u8],
        schema: &[u8],
    ) -> (
        BTreeMap<String, Vec<u8>>,
        BTreeMap<(String, String), Vec<u8>>,
    ) {
        (
            BTreeMap::from([("evidence:evidence-001".into(), payload.to_vec())]),
            BTreeMap::from([(
                ("urn:synthetic:schema:evidence".into(), "1.0".into()),
                schema.to_vec(),
            )]),
        )
    }

    #[test]
    fn exact_payload_and_schema_bytes_are_independently_hashed() {
        let payload = br#"{"measurement":42,"unit":"mm"}"#;
        let schema =
            br#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object"}"#;
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (payloads, schemas) = inputs(payload, schema);

        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(
            receipt.disposition,
            ByteVerificationDisposition::CompleteForSuppliedClaims
        );
        assert_eq!(receipt.payload_bytes_verified, 1);
        assert_eq!(receipt.schema_bytes_verified, 1);
        assert_eq!(
            receipt.payload_byte_verification,
            "complete_for_supplied_payload_claims"
        );
        assert_eq!(receipt.source_owner_authentication, "not_performed");
        assert_eq!(receipt.rights_authentication, "not_performed");
        assert_eq!(receipt.action_authority, "none");
        assert!(valid_sha256(&receipt.receipt_sha256));
    }

    #[test]
    fn one_byte_mutation_is_a_digest_mismatch() {
        let payload = b"original payload";
        let schema = b"schema bytes";
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (payloads, schemas) = inputs(b"mutated payload", schema);
        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(receipt.disposition, ByteVerificationDisposition::InvalidInput);
        assert_eq!(receipt.payload_digest_mismatches, 1);
        assert!(receipt.findings.iter().any(|finding| {
            finding.code == "SOURCE_PAYLOAD_DIGEST_MISMATCH"
        }));
        assert_eq!(schemas.len(), 1);
    }

    #[test]
    fn declared_byte_length_mismatch_is_invalid_even_when_digest_matches() {
        let payload = b"payload";
        let schema = b"schema";
        let case = case(payload);
        let mut catalog = catalog(&case, payload, schema);
        catalog.descriptors[0].source_payload.byte_length = Some(999);
        let (payloads, schemas) = inputs(payload, schema);
        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(receipt.disposition, ByteVerificationDisposition::InvalidInput);
        assert_eq!(receipt.payload_length_mismatches, 1);
    }

    #[test]
    fn missing_payload_bytes_remain_incomplete() {
        let payload = b"payload";
        let schema = b"schema";
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (_, schemas) = inputs(payload, schema);
        let receipt = verify_catalog_payload_bytes(&case, &catalog, &BTreeMap::new(), &schemas);
        assert_eq!(
            receipt.disposition,
            ByteVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(receipt.payload_bytes_missing, 1);
    }

    #[test]
    fn schema_digest_mismatch_is_invalid() {
        let payload = b"payload";
        let schema = b"schema";
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (payloads, _) = inputs(payload, schema);
        let wrong_schemas = BTreeMap::from([(
            ("urn:synthetic:schema:evidence".into(), "1.0".into()),
            b"tampered schema".to_vec(),
        )]);
        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &wrong_schemas);
        assert_eq!(receipt.disposition, ByteVerificationDisposition::InvalidInput);
        assert_eq!(receipt.schema_digest_mismatches, 1);
    }

    #[test]
    fn owner_attested_digest_is_corroborated_without_authenticating_owner() {
        let payload = b"owner-attested bytes";
        let schema = b"schema bytes";
        let case = case(payload);
        let mut catalog = catalog(&case, payload, schema);
        let descriptor = &mut catalog.descriptors[0].source_payload;
        descriptor.digest_basis = PayloadDigestBasis::SourceOwnerAttested;
        descriptor.bytes_retrieved = false;
        descriptor.byte_length = None;
        descriptor.payload_object_ref = None;
        let (payloads, schemas) = inputs(payload, schema);

        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(receipt.disposition, ByteVerificationDisposition::IncompleteOrIndeterminate);
        assert_eq!(
            receipt.records[0].payload_result,
            "supplied_bytes_match_owner_attested_digest"
        );
        assert_eq!(receipt.source_owner_authentication, "not_performed");
        assert_eq!(receipt.source_attestation_authentication, "not_performed");
    }

    #[test]
    fn external_reference_digest_is_checked_against_supplied_bytes() {
        let payload = b"external object bytes";
        let schema = b"schema bytes";
        let case = case(payload);
        let mut catalog = catalog(&case, payload, schema);
        let descriptor = &mut catalog.descriptors[0].source_payload;
        descriptor.digest_basis = PayloadDigestBasis::ExternalReference;
        descriptor.bytes_retrieved = false;
        descriptor.byte_length = None;
        let (payloads, schemas) = inputs(payload, schema);

        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(
            receipt.records[0].payload_result,
            "verified_supplied_bytes_match_external_digest_claim"
        );
        assert_eq!(receipt.disposition, ByteVerificationDisposition::IncompleteOrIndeterminate);
    }

    #[test]
    fn unavailable_source_digest_never_promotes_a_computed_digest() {
        let payload = b"observed but unbound bytes";
        let schema = b"schema bytes";
        let mut case = case(payload);
        case.evidence[0].source_payload_sha256 = None;
        let mut catalog = catalog(&case, payload, schema);
        let descriptor = &mut catalog.descriptors[0].source_payload;
        descriptor.sha256 = None;
        descriptor.digest_basis = PayloadDigestBasis::Unavailable;
        descriptor.bytes_retrieved = false;
        descriptor.byte_length = None;
        descriptor.payload_object_ref = None;
        catalog.case_representation_sha256 =
            sha256_hex(&serde_json::to_vec(&case).unwrap());
        let (payloads, schemas) = inputs(payload, schema);

        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(receipt.disposition, ByteVerificationDisposition::IncompleteOrIndeterminate);
        assert_eq!(receipt.payload_bytes_verified, 0);
        assert_eq!(
            receipt.records[0].payload_result,
            "observed_bytes_without_source_digest_claim"
        );
        assert_ne!(
            receipt.records[0].computed_payload_sha256,
            receipt.records[0].expected_payload_sha256
        );
    }

    #[test]
    fn missing_schema_bytes_remain_incomplete() {
        let payload = b"payload";
        let schema = b"schema bytes";
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (payloads, _) = inputs(payload, schema);

        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &BTreeMap::new());
        assert_eq!(receipt.schema_bytes_missing, 1);
        assert_eq!(receipt.disposition, ByteVerificationDisposition::IncompleteOrIndeterminate);
        assert_eq!(receipt.records[0].schema_result, "schema_bytes_missing");
    }

    #[test]
    fn unreferenced_payload_and_schema_inputs_are_not_silently_ignored() {
        let payload = b"payload";
        let schema = b"schema";
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (mut payloads, mut schemas) = inputs(payload, schema);
        payloads.insert("artifact:unreferenced".into(), b"extra".to_vec());
        schemas.insert(
            ("urn:unused:schema".into(), "9.9".into()),
            b"unused".to_vec(),
        );
        let receipt = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(
            receipt.disposition,
            ByteVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(receipt.unreferenced_payload_input_count, 1);
        assert_eq!(receipt.unreferenced_schema_input_count, 1);
    }

    #[test]
    fn caller_input_order_does_not_change_manifest_or_receipt() {
        let payload = b"payload";
        let schema = b"schema";
        let case = case(payload);
        let catalog = catalog(&case, payload, schema);
        let (payloads, schemas) = inputs(payload, schema);
        let first = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        let second = verify_catalog_payload_bytes(&case, &catalog, &payloads, &schemas);
        assert_eq!(
            first.payload_input_manifest_sha256,
            second.payload_input_manifest_sha256
        );
        assert_eq!(
            first.schema_input_manifest_sha256,
            second.schema_input_manifest_sha256
        );
        assert_eq!(first.receipt_sha256, second.receipt_sha256);
    }
}
