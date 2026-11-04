// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bounded streaming verification of source payload bytes referenced by
//! built-environment descriptor catalogues.
//!
//! This module computes SHA-256 over bytes supplied by a caller-owned
//! resolver. It does not fetch over the network, authenticate the resolver,
//! establish source-owner identity, evaluate rights, verify signatures, or
//! establish engineering correctness.

use crate::built_environment_case::BuiltEnvironmentCase;
use crate::built_environment_source_descriptor::{
    DescriptorDisposition, PayloadDigestBasis, SourceDescriptorCatalog,
    SourceRecordDescriptor, validate_source_descriptor_catalog,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;

const MAX_BUFFER_SIZE_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadResolveError {
    NotFound,
    Rejected(String),
    IoError(String),
}

/// The resolver is the only I/O boundary. It may use a local CAS, object store,
/// or another explicitly configured source. This crate performs no network
/// request itself and never treats an object reference as proof of bytes.
pub trait ContentAddressedPayloadResolver {
    type Reader: Read;

    fn open(
        &mut self,
        descriptor: &SourceRecordDescriptor,
    ) -> Result<Self::Reader, PayloadResolveError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayloadVerificationLimits {
    /// Maximum bytes hashed for any one payload.
    pub max_bytes_per_payload: u64,
    /// Maximum bytes hashed across the complete catalogue verification.
    pub max_total_bytes: u64,
    /// Fixed streaming buffer size. Must be in 1..=1 MiB.
    pub buffer_size_bytes: usize,
}

impl PayloadVerificationLimits {
    pub fn validate(self) -> Result<(), String> {
        if self.max_bytes_per_payload == 0 {
            return Err("max_bytes_per_payload must be non-zero".into());
        }
        if self.max_total_bytes == 0 {
            return Err("max_total_bytes must be non-zero".into());
        }
        if self.buffer_size_bytes == 0 || self.buffer_size_bytes > MAX_BUFFER_SIZE_BYTES {
            return Err(format!(
                "buffer_size_bytes must be between 1 and {MAX_BUFFER_SIZE_BYTES}"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadByteVerificationDisposition {
    VerifiedMatch,
    DigestMismatch,
    ByteLengthMismatch,
    DigestAndByteLengthMismatch,
    DeclaredUnavailable,
    ObjectNotFound,
    ResourceLimitReached,
    ResolverRejected,
    ResolverIoError,
    StreamReadError,
    InvalidDescriptor,
}

impl PayloadByteVerificationDisposition {
    fn is_invalid(self) -> bool {
        matches!(
            self,
            Self::DigestMismatch
                | Self::ByteLengthMismatch
                | Self::DigestAndByteLengthMismatch
                | Self::ResolverRejected
                | Self::InvalidDescriptor
        )
    }

    fn is_verified(self) -> bool {
        self == Self::VerifiedMatch
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayloadByteVerification {
    pub record_key: String,
    pub digest_basis: String,
    pub object_ref: Option<String>,
    pub expected_sha256: Option<String>,
    pub observed_sha256: Option<String>,
    pub expected_byte_length: Option<u64>,
    pub observed_byte_length: Option<u64>,
    pub bytes_hashed: u64,
    pub digest_matches: Option<bool>,
    pub byte_length_matches: Option<bool>,
    pub resolver_open_attempted: bool,
    pub disposition: PayloadByteVerificationDisposition,
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadCatalogVerificationDisposition {
    AllAvailablePayloadsVerified,
    IncompleteOrIndeterminate,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayloadCatalogVerificationReport {
    pub disposition: PayloadCatalogVerificationDisposition,
    pub case_id: String,
    pub catalog_id: String,
    pub case_representation_sha256: String,
    pub catalog_representation_sha256: String,
    pub evaluation_time_utc: String,
    pub verifier_build_sha256: Option<String>,
    pub catalog_validation: crate::built_environment_source_descriptor::DescriptorValidation,
    pub context_findings: Vec<String>,
    pub limits: PayloadVerificationLimits,
    pub payload_count: usize,
    pub verified_payload_count: usize,
    pub unavailable_payload_count: usize,
    pub resource_limited_payload_count: usize,
    pub invalid_payload_count: usize,
    pub total_bytes_hashed: u64,
    pub payload_results: Vec<PayloadByteVerification>,
    pub payload_byte_verification: String,
    pub rights_authentication: String,
    pub source_attestation_authentication: String,
    pub action_authority: String,
    pub limitations: Vec<String>,
}

enum StreamHashOutcome {
    Complete {
        sha256: String,
        byte_length: u64,
    },
    LimitReached {
        bytes_hashed: u64,
    },
    ReadError {
        bytes_hashed: u64,
        detail: String,
    },
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        if !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit() {
            return false;
        }
    }
    let part = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
    };
    let year = part(0, 4).unwrap_or(0);
    let month = part(5, 7).unwrap_or(0);
    let day = part(8, 10).unwrap_or(0);
    let hour = part(11, 13).unwrap_or(99);
    let minute = part(14, 16).unwrap_or(99);
    let second = part(17, 19).unwrap_or(99);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let last_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    (1..=last_day).contains(&day)
}

fn expected_cas_reference(sha256: &str) -> String {
    format!("cas:sha256:{sha256}")
}

/// Hash no more than max_bytes from the stream. Reaching the exact configured
/// limit is reported as indeterminate because this bounded reader does not
/// consume another byte merely to test for EOF. Configure limits above the
/// maximum admissible payload size to obtain a conclusive result at the
/// expected boundary.
fn hash_bounded_reader<R: Read>(
    mut reader: R,
    max_bytes: u64,
    buffer_size: usize,
) -> StreamHashOutcome {
    if max_bytes == 0 {
        return StreamHashOutcome::LimitReached { bytes_hashed: 0 };
    }

    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; buffer_size];
    let mut bytes_hashed = 0_u64;

    loop {
        if bytes_hashed == max_bytes {
            return StreamHashOutcome::LimitReached { bytes_hashed };
        }
        let remaining = max_bytes - bytes_hashed;
        let read_limit = usize::try_from(remaining)
            .unwrap_or(usize::MAX)
            .min(buffer.len());

        match reader.read(&mut buffer[..read_limit]) {
            Ok(0) => {
                let digest = hasher
                    .finalize()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect();
                return StreamHashOutcome::Complete {
                    sha256: digest,
                    byte_length: bytes_hashed,
                };
            }
            Ok(count) => {
                hasher.update(&buffer[..count]);
                bytes_hashed += count as u64;
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return StreamHashOutcome::ReadError {
                    bytes_hashed,
                    detail: error.to_string(),
                };
            }
        }
    }
}

fn base_result(
    descriptor: &SourceRecordDescriptor,
) -> PayloadByteVerification {
    PayloadByteVerification {
        record_key: descriptor
            .record_kind
            .case_key(&descriptor.case_record_id),
        digest_basis: descriptor.source_payload.digest_basis.as_str().into(),
        object_ref: descriptor.source_payload.payload_object_ref.clone(),
        expected_sha256: descriptor.source_payload.sha256.clone(),
        observed_sha256: None,
        expected_byte_length: descriptor.source_payload.byte_length,
        observed_byte_length: None,
        bytes_hashed: 0,
        digest_matches: None,
        byte_length_matches: None,
        resolver_open_attempted: false,
        disposition: PayloadByteVerificationDisposition::InvalidDescriptor,
        detail: None,
    }
}

fn verify_one<R: ContentAddressedPayloadResolver>(
    descriptor: &SourceRecordDescriptor,
    resolver: &mut R,
    limits: PayloadVerificationLimits,
    remaining_total_budget: u64,
) -> PayloadByteVerification {
    let mut result = base_result(descriptor);
    let payload = &descriptor.source_payload;

    if payload.digest_basis == PayloadDigestBasis::Unavailable {
        result.disposition = PayloadByteVerificationDisposition::DeclaredUnavailable;
        result.detail = Some("descriptor declares no source payload digest or bytes".into());
        return result;
    }

    let Some(expected_sha256) = payload.sha256.as_deref() else {
        result.detail = Some("payload digest is required for this digest basis".into());
        return result;
    };
    if !valid_sha256(expected_sha256) {
        result.detail = Some("expected payload digest is not lowercase SHA-256".into());
        return result;
    }

    if matches!(
        payload.digest_basis,
        PayloadDigestBasis::LocallyRetrievedAndHashed | PayloadDigestBasis::ExternalReference
    ) {
        let expected_ref = expected_cas_reference(expected_sha256);
        if payload.payload_object_ref.as_deref() != Some(expected_ref.as_str()) {
            result.detail = Some(
                "content-addressed reference does not bind the exact expected SHA-256".into(),
            );
            return result;
        }
    }

    if let Some(expected_length) = payload.byte_length {
        if expected_length > limits.max_bytes_per_payload
            || expected_length > remaining_total_budget
        {
            result.disposition = PayloadByteVerificationDisposition::ResourceLimitReached;
            result.detail = Some(
                "declared byte length exceeds the remaining configured verification budget".into(),
            );
            return result;
        }
    }

    if remaining_total_budget == 0 {
        result.disposition = PayloadByteVerificationDisposition::ResourceLimitReached;
        result.detail = Some("catalogue-wide byte budget is exhausted".into());
        return result;
    }

    let max_for_payload = limits
        .max_bytes_per_payload
        .min(remaining_total_budget);
    result.resolver_open_attempted = true;
    let reader = match resolver.open(descriptor) {
        Ok(reader) => reader,
        Err(PayloadResolveError::NotFound) => {
            result.disposition = PayloadByteVerificationDisposition::ObjectNotFound;
            result.detail = Some("resolver could not locate the source payload bytes".into());
            return result;
        }
        Err(PayloadResolveError::Rejected(detail)) => {
            result.disposition = PayloadByteVerificationDisposition::ResolverRejected;
            result.detail = Some(detail);
            return result;
        }
        Err(PayloadResolveError::IoError(detail)) => {
            result.disposition = PayloadByteVerificationDisposition::ResolverIoError;
            result.detail = Some(detail);
            return result;
        }
    };

    match hash_bounded_reader(
        reader,
        max_for_payload,
        limits.buffer_size_bytes,
    ) {
        StreamHashOutcome::Complete {
            sha256,
            byte_length,
        } => {
            result.observed_sha256 = Some(sha256.clone());
            result.observed_byte_length = Some(byte_length);
            result.bytes_hashed = byte_length;
            let digest_matches = sha256 == expected_sha256;
            let byte_length_matches = payload
                .byte_length
                .map(|expected| expected == byte_length)
                .unwrap_or(true);
            result.digest_matches = Some(digest_matches);
            result.byte_length_matches = payload.byte_length.map(|_| byte_length_matches);
            result.disposition = match (digest_matches, byte_length_matches) {
                (true, true) => PayloadByteVerificationDisposition::VerifiedMatch,
                (false, true) => PayloadByteVerificationDisposition::DigestMismatch,
                (true, false) => PayloadByteVerificationDisposition::ByteLengthMismatch,
                (false, false) => {
                    PayloadByteVerificationDisposition::DigestAndByteLengthMismatch
                }
            };
            if !digest_matches {
                result.detail = Some("recomputed SHA-256 differs from descriptor claim".into());
            } else if !byte_length_matches {
                result.detail = Some("recomputed byte length differs from descriptor claim".into());
            }
        }
        StreamHashOutcome::LimitReached { bytes_hashed } => {
            result.bytes_hashed = bytes_hashed;
            result.disposition = PayloadByteVerificationDisposition::ResourceLimitReached;
            result.detail = Some(
                "configured byte limit was reached before end-of-stream could be established"
                    .into(),
            );
        }
        StreamHashOutcome::ReadError {
            bytes_hashed,
            detail,
        } => {
            result.bytes_hashed = bytes_hashed;
            result.disposition = PayloadByteVerificationDisposition::StreamReadError;
            result.detail = Some(detail);
        }
    }
    result
}

/// Rehash every available descriptor payload using a caller-owned resolver.
///
/// Invalid catalogues are rejected before the resolver is called. A matching
/// digest proves byte equality against the descriptor claim only. A caller
/// must separately authenticate the claim, source owner, rights and
/// attestation, and qualify source inventory coverage.
pub fn verify_source_descriptor_catalog_payloads<R: ContentAddressedPayloadResolver>(
    case: &BuiltEnvironmentCase,
    catalog: &SourceDescriptorCatalog,
    resolver: &mut R,
    limits: PayloadVerificationLimits,
    evaluation_time_utc: impl Into<String>,
    verifier_build_sha256: Option<String>,
) -> PayloadCatalogVerificationReport {
    let validation = validate_source_descriptor_catalog(case, catalog);
    let case_bytes = serde_json::to_vec(case).unwrap_or_default();
    let catalog_bytes = serde_json::to_vec(catalog).unwrap_or_default();
    let case_sha256 = sha256_hex(&case_bytes);
    let catalog_sha256 = sha256_hex(&catalog_bytes);
    let evaluation_time_utc = evaluation_time_utc.into();

    let limits_valid = limits.validate().is_ok();
    let mut context_findings = Vec::new();
    let evaluation_time_valid = valid_utc_timestamp(&evaluation_time_utc);
    if !evaluation_time_valid {
        context_findings.push("INVALID_EVALUATION_TIME".into());
    }
    let build_identity_state = match verifier_build_sha256.as_deref() {
        Some(value) if valid_sha256(value) => "valid",
        Some(_) => {
            context_findings.push("INVALID_VERIFIER_BUILD_DIGEST".into());
            "invalid"
        }
        None => {
            context_findings.push("VERIFIER_BUILD_IDENTITY_UNAVAILABLE".into());
            "missing"
        }
    };
    if !limits_valid {
        context_findings.push("INVALID_VERIFICATION_LIMITS".into());
    }
    let mut invalid_input = validation.disposition == DescriptorDisposition::InvalidInput
        || !limits_valid
        || !evaluation_time_valid
        || build_identity_state == "invalid";
    let mut incomplete = !limits_valid || build_identity_state == "missing";
    let mut results = Vec::new();
    let mut total_bytes_hashed = 0_u64;

    if validation.disposition != DescriptorDisposition::InvalidInput && limits_valid {
        let mut descriptors = catalog.descriptors.iter().collect::<Vec<_>>();
        descriptors.sort_by_key(|descriptor| {
            descriptor
                .record_kind
                .case_key(&descriptor.case_record_id)
        });

        for descriptor in descriptors {
            let remaining = limits.max_total_bytes.saturating_sub(total_bytes_hashed);
            let result = verify_one(descriptor, resolver, limits, remaining);
            total_bytes_hashed = total_bytes_hashed.saturating_add(result.bytes_hashed);

            if result.disposition.is_invalid() {
                invalid_input = true;
            } else if !result.disposition.is_verified() {
                incomplete = true;
            }
            results.push(result);
        }
    }

    let verified_record_keys = results
        .iter()
        .filter(|result| result.disposition.is_verified())
        .map(|result| result.record_key.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let unresolved_catalog_findings = validation.findings.iter().any(|finding| {
        if finding.severity != "incomplete" {
            return false;
        }
        let can_be_resolved_by_byte_rehash = matches!(
            finding.code.as_str(),
            "SOURCE_PAYLOAD_DIGEST_NOT_LOCALLY_REVERIFIED"
                | "EXTERNAL_PAYLOAD_BYTES_NOT_RETRIEVED"
        );
        !(can_be_resolved_by_byte_rehash
            && verified_record_keys.contains(finding.record_key.as_str()))
    });
    incomplete |= unresolved_catalog_findings;

    let disposition = if invalid_input {
        PayloadCatalogVerificationDisposition::InvalidInput
    } else if incomplete || results.is_empty() {
        PayloadCatalogVerificationDisposition::IncompleteOrIndeterminate
    } else {
        PayloadCatalogVerificationDisposition::AllAvailablePayloadsVerified
    };

    let verified_payload_count = results
        .iter()
        .filter(|result| result.disposition.is_verified())
        .count();
    let unavailable_payload_count = results
        .iter()
        .filter(|result| {
            matches!(
                result.disposition,
                PayloadByteVerificationDisposition::DeclaredUnavailable
                    | PayloadByteVerificationDisposition::ObjectNotFound
            )
        })
        .count();
    let resource_limited_payload_count = results
        .iter()
        .filter(|result| {
            result.disposition == PayloadByteVerificationDisposition::ResourceLimitReached
        })
        .count();
    let invalid_payload_count = results
        .iter()
        .filter(|result| result.disposition.is_invalid())
        .count();

    PayloadCatalogVerificationReport {
        disposition,
        case_id: case.case_id.clone(),
        catalog_id: catalog.catalog_id.clone(),
        case_representation_sha256: case_sha256,
        catalog_representation_sha256: catalog_sha256,
        evaluation_time_utc,
        verifier_build_sha256,
        catalog_validation: validation,
        context_findings,
        limits,
        payload_count: results.len(),
        verified_payload_count,
        unavailable_payload_count,
        resource_limited_payload_count,
        invalid_payload_count,
        total_bytes_hashed,
        payload_results: results,
        payload_byte_verification: "recomputed_over_resolver_bytes".into(),
        rights_authentication: "not_performed".into(),
        source_attestation_authentication: "not_performed".into(),
        action_authority: "none".into(),
        limitations: vec![
            "A matching digest proves byte equality against the descriptor claim, not source authorship or owner identity.".into(),
            "The resolver is caller supplied; resolver identity, storage authenticity and transport security are not authenticated here.".into(),
            "Rights evidence and source attestations are not authenticated by byte hashing.".into(),
            "Catalogue validation and byte verification do not establish source inventory completeness or engineering correctness.".into(),
            "A payload that exactly reaches a configured byte limit is indeterminate because EOF is not read beyond the hard limit.".into(),
            "No engineering approval, action authority, or NoImpact eligibility is produced.".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_environment_case::{DataRights, EvidenceRecord, EvidenceState};
    use crate::built_environment_source_descriptor::{
        NativeIdentityComponent, SourceAttestationDescriptor, SourceObservationTime,
        SourcePayloadDescriptor, SourceRecordKind, SourceRightsDescriptor, SourceRightsState,
        SourceSnapshotDescriptor,
    };
    use std::collections::BTreeMap;
    use std::io::Cursor;

    #[derive(Default)]
    struct MemoryResolver {
        objects: BTreeMap<String, Vec<u8>>,
        open_count: usize,
    }

    impl ContentAddressedPayloadResolver for MemoryResolver {
        type Reader = Cursor<Vec<u8>>;

        fn open(
            &mut self,
            descriptor: &SourceRecordDescriptor,
        ) -> Result<Self::Reader, PayloadResolveError> {
            self.open_count += 1;
            let object_ref = descriptor
                .source_payload
                .payload_object_ref
                .clone()
                .or_else(|| {
                    descriptor
                        .source_payload
                        .sha256
                        .as_deref()
                        .map(expected_cas_reference)
                })
                .ok_or(PayloadResolveError::NotFound)?;
            self.objects
                .get(&object_ref)
                .cloned()
                .map(Cursor::new)
                .ok_or(PayloadResolveError::NotFound)
        }
    }

    fn digest(bytes: &[u8]) -> String {
        sha256_hex(bytes)
    }

    fn case(bytes_sha256: &str) -> BuiltEnvironmentCase {
        BuiltEnvironmentCase {
            case_id: "case-payload-verification-001".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence: vec![EvidenceRecord {
                evidence_id: "evidence-payload-001".into(),
                subject_id: "design-001".into(),
                field_path: "geometry.revision".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: Some(bytes_sha256.into()),
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

    fn descriptor(bytes: &[u8]) -> SourceRecordDescriptor {
        let sha = digest(bytes);
        SourceRecordDescriptor {
            record_kind: SourceRecordKind::Evidence,
            case_record_id: "evidence-payload-001".into(),
            owner_namespace: "urn:synthetic:survey-owner".into(),
            native_identity_components: vec![
                NativeIdentityComponent {
                    name: "native_id".into(),
                    value: "survey-registry".into(),
                },
                NativeIdentityComponent {
                    name: "native_record_id".into(),
                    value: "evidence-payload-001".into(),
                },
                NativeIdentityComponent {
                    name: "native_revision".into(),
                    value: "1".into(),
                },
                NativeIdentityComponent {
                    name: "field_path".into(),
                    value: "geometry.revision".into(),
                },
                NativeIdentityComponent {
                    name: "subject_id".into(),
                    value: "design-001".into(),
                },
                NativeIdentityComponent {
                    name: "jurisdiction".into(),
                    value: "synthetic-jurisdiction".into(),
                },
            ],
            source_payload: SourcePayloadDescriptor {
                sha256: Some(sha.clone()),
                digest_basis: PayloadDigestBasis::LocallyRetrievedAndHashed,
                bytes_retrieved: true,
                byte_length: Some(bytes.len() as u64),
                payload_object_ref: Some(expected_cas_reference(&sha)),
                media_type: Some("application/json".into()),
                schema_id: Some("urn:synthetic:survey-evidence".into()),
                schema_version: Some("1.0".into()),
                schema_sha256: Some(digest(b"synthetic-schema")),
            },
            source_snapshot: SourceSnapshotDescriptor {
                source_id: "urn:synthetic:survey-owner".into(),
                snapshot_id: "snapshot-001".into(),
                snapshot_sha256: digest(b"snapshot-001"),
                captured_at: "2026-11-04T10:00:00Z".into(),
                retrieved_at: "2026-11-04T10:01:00Z".into(),
            },
            observation_time: SourceObservationTime {
                observed_at: Some("2026-11-04T09:59:00Z".into()),
                recorded_at: "2026-11-04T10:00:00Z".into(),
            },
            rights: SourceRightsDescriptor {
                state: SourceRightsState::Verified,
                permitted_uses: vec!["impact_analysis".into()],
                rights_evidence_ref: Some("rights-evidence:synthetic-001".into()),
            },
            attestation: SourceAttestationDescriptor {
                statement_ref: Some("attestation:synthetic-001".into()),
                key_id: Some("urn:synthetic:key:001".into()),
                trust_evaluation: "not_evaluated_by_descriptor_catalog".into(),
            },
        }
    }

    fn catalog(
        case: &BuiltEnvironmentCase,
        descriptor: SourceRecordDescriptor,
    ) -> SourceDescriptorCatalog {
        SourceDescriptorCatalog {
            schema_version: "luminous.built-environment-source-descriptor-catalog.v0.1".into(),
            catalog_id: "catalog-payload-verification-001".into(),
            case_id: case.case_id.clone(),
            case_representation_sha256: sha256_hex(&serde_json::to_vec(case).unwrap()),
            captured_at: "2026-11-04T10:02:00Z".into(),
            descriptors: vec![descriptor],
            claim_ceiling: "descriptor_integrity_only".into(),
            action_authority: "none".into(),
            limitations: vec!["Synthetic only; trust and engineering proof are separate.".into()],
        }
    }

    fn limits() -> PayloadVerificationLimits {
        PayloadVerificationLimits {
            max_bytes_per_payload: 1024,
            max_total_bytes: 2048,
            buffer_size_bytes: 64,
        }
    }

    #[test]
    fn matching_payload_bytes_are_rehashed_and_reported_verified() {
        let bytes = b"exact synthetic source payload";
        let case = case(&digest(bytes));
        let descriptor = descriptor(bytes);
        let catalog = catalog(&case, descriptor.clone());
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(
            descriptor.source_payload.payload_object_ref.clone().unwrap(),
            bytes.to_vec(),
        );

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            limits(),
            "2026-11-04T10:03:00Z",
            Some(digest(b"verifier-build")),
        );
        assert_eq!(
            report.disposition,
            PayloadCatalogVerificationDisposition::AllAvailablePayloadsVerified
        );
        assert_eq!(report.verified_payload_count, 1);
        assert_eq!(report.total_bytes_hashed, bytes.len() as u64);
        assert_eq!(
            report.payload_results[0].observed_sha256.as_deref(),
            Some(digest(bytes).as_str())
        );
        assert_eq!(report.rights_authentication, "not_performed");
        assert_eq!(report.action_authority, "none");
    }

    #[test]
    fn source_owner_attested_digest_becomes_byte_verified_when_resolver_supplies_bytes() {
        let bytes = b"source-owner-attested payload";
        let case = case(&digest(bytes));
        let mut descriptor = descriptor(bytes);
        descriptor.source_payload.digest_basis = PayloadDigestBasis::SourceOwnerAttested;
        descriptor.source_payload.bytes_retrieved = false;
        descriptor.source_payload.byte_length = None;
        descriptor.source_payload.payload_object_ref = None;
        let catalog = catalog(&case, descriptor.clone());
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(
            expected_cas_reference(descriptor.source_payload.sha256.as_deref().unwrap()),
            bytes.to_vec(),
        );

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            limits(),
            "2026-11-04T10:03:00Z",
            Some(digest(b"verifier-build")),
        );
        assert_eq!(
            report.disposition,
            PayloadCatalogVerificationDisposition::AllAvailablePayloadsVerified
        );
        assert_eq!(report.verified_payload_count, 1);
        assert!(report.catalog_validation.findings.iter().any(|finding| {
            finding.code == "SOURCE_PAYLOAD_DIGEST_NOT_LOCALLY_REVERIFIED"
        }));
        assert_eq!(report.rights_authentication, "not_performed");
        assert_eq!(report.source_attestation_authentication, "not_performed");
    }

    #[test]
    fn claimed_byte_length_mismatch_is_invalid() {
        let bytes = b"length-check";
        let case = case(&digest(bytes));
        let mut descriptor = descriptor(bytes);
        descriptor.source_payload.byte_length = Some(bytes.len() as u64 + 1);
        let catalog = catalog(&case, descriptor.clone());
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(
            descriptor.source_payload.payload_object_ref.clone().unwrap(),
            bytes.to_vec(),
        );

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            limits(),
            "2026-11-04T10:03:00Z",
            Some(digest(b"verifier-build")),
        );
        assert_eq!(report.disposition, PayloadCatalogVerificationDisposition::InvalidInput);
        assert_eq!(
            report.payload_results[0].disposition,
            PayloadByteVerificationDisposition::ByteLengthMismatch
        );
    }

    #[test]
    fn one_byte_payload_mutation_is_invalid_not_incomplete() {
        let expected = b"exact synthetic source payload";
        let observed = b"Exact synthetic source payload";
        let case = case(&digest(expected));
        let descriptor = descriptor(expected);
        let catalog = catalog(&case, descriptor.clone());
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(
            descriptor.source_payload.payload_object_ref.clone().unwrap(),
            observed.to_vec(),
        );

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            limits(),
            "2026-11-04T10:03:00Z",
            None,
        );
        assert_eq!(report.disposition, PayloadCatalogVerificationDisposition::InvalidInput);
        assert_eq!(
            report.payload_results[0].disposition,
            PayloadByteVerificationDisposition::DigestMismatch
        );
    }

    #[test]
    fn resource_limit_is_incomplete_and_never_reports_a_digest_match() {
        let bytes = b"payload longer than configured limit";
        let case = case(&digest(bytes));
        let descriptor = descriptor(bytes);
        let catalog = catalog(&case, descriptor.clone());
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(
            descriptor.source_payload.payload_object_ref.clone().unwrap(),
            bytes.to_vec(),
        );
        let mut configured = limits();
        configured.max_bytes_per_payload = 8;
        configured.max_total_bytes = 8;

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            configured,
            "2026-11-04T10:03:00Z",
            None,
        );
        assert_eq!(
            report.disposition,
            PayloadCatalogVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.payload_results[0].disposition,
            PayloadByteVerificationDisposition::ResourceLimitReached
        );
        assert_eq!(report.payload_results[0].digest_matches, None);
        assert_eq!(report.total_bytes_hashed, 8);
    }

    #[test]
    fn invalid_catalogue_is_rejected_before_resolver_access() {
        let bytes = b"exact synthetic source payload";
        let case = case(&digest(bytes));
        let descriptor = descriptor(bytes);
        let mut catalog = catalog(&case, descriptor.clone());
        catalog.case_representation_sha256 = digest(b"different case bytes");
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(
            descriptor.source_payload.payload_object_ref.clone().unwrap(),
            bytes.to_vec(),
        );

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            limits(),
            "2026-11-04T10:03:00Z",
            None,
        );
        assert_eq!(report.disposition, PayloadCatalogVerificationDisposition::InvalidInput);
        assert_eq!(resolver.open_count, 0);
        assert!(report.payload_results.is_empty());
    }

    #[test]
    fn missing_object_is_incomplete_not_a_digest_success() {
        let bytes = b"exact synthetic source payload";
        let case = case(&digest(bytes));
        let descriptor = descriptor(bytes);
        let catalog = catalog(&case, descriptor);
        let mut resolver = MemoryResolver::default();

        let report = verify_source_descriptor_catalog_payloads(
            &case,
            &catalog,
            &mut resolver,
            limits(),
            "2026-11-04T10:03:00Z",
            None,
        );
        assert_eq!(
            report.disposition,
            PayloadCatalogVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.payload_results[0].disposition,
            PayloadByteVerificationDisposition::ObjectNotFound
        );
    }
}
