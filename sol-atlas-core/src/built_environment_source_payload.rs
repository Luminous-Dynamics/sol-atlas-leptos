// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bounded streaming verification of source payload bytes referenced by a
//! source descriptor catalogue.
//!
//! A successful result proves byte equality against the descriptor's declared
//! SHA-256 and optional byte length. It does not authenticate the source owner,
//! rights, signatures, snapshot completeness, or engineering correctness.

use crate::built_environment_case::BuiltEnvironmentCase;
use crate::built_environment_source_descriptor::{
    DescriptorDisposition, PayloadDigestBasis, SourceDescriptorCatalog, SourceRecordKind,
    validate_source_descriptor_catalog,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};

const MAX_BUFFER_BYTES: usize = 1024 * 1024;

pub trait ContentAddressedPayloadResolver {
    type Reader: Read;

    /// Open one exact content-addressed object reference. The resolver must
    /// not silently substitute another object or follow unbounded redirects.
    fn open(&mut self, object_reference: &str) -> io::Result<Self::Reader>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadVerificationLimits {
    pub buffer_size_bytes: usize,
    pub max_bytes_per_payload: u64,
    pub max_total_bytes: u64,
}

impl Default for PayloadVerificationLimits {
    fn default() -> Self {
        Self {
            buffer_size_bytes: 256 * 1024,
            max_bytes_per_payload: 1024 * 1024 * 1024,
            max_total_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadByteDisposition {
    Verified,
    BytesUnavailable,
    ObjectNotFound,
    ReadFailed,
    ByteLimitExceeded,
    DigestMismatch,
    ByteLengthMismatch,
    InvalidDescriptor,
    NotAttemptedInvalidCatalog,
    NotAttemptedAggregateLimit,
}

impl PayloadByteDisposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::BytesUnavailable => "bytes_unavailable",
            Self::ObjectNotFound => "object_not_found",
            Self::ReadFailed => "read_failed",
            Self::ByteLimitExceeded => "byte_limit_exceeded",
            Self::DigestMismatch => "digest_mismatch",
            Self::ByteLengthMismatch => "byte_length_mismatch",
            Self::InvalidDescriptor => "invalid_descriptor",
            Self::NotAttemptedInvalidCatalog => "not_attempted_invalid_catalog",
            Self::NotAttemptedAggregateLimit => "not_attempted_aggregate_limit",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadFindingSeverity {
    Incomplete,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PayloadByteFinding {
    pub severity: PayloadFindingSeverity,
    pub code: String,
    pub record_key: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayloadRecordVerification {
    pub record_key: String,
    pub digest_basis: String,
    pub object_reference: Option<String>,
    pub expected_sha256: Option<String>,
    pub observed_sha256: Option<String>,
    pub expected_byte_length: Option<u64>,
    pub observed_byte_length: Option<u64>,
    /// Includes a single sentinel byte when a hard byte limit is exceeded.
    pub bytes_read: u64,
    pub disposition: PayloadByteDisposition,
    pub findings: Vec<PayloadByteFinding>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadVerificationDisposition {
    FullyVerified,
    IncompleteOrIndeterminate,
    InvalidInput,
}

impl PayloadVerificationDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FullyVerified => "fully_verified",
            Self::IncompleteOrIndeterminate => "incomplete_or_indeterminate",
            Self::InvalidInput => "invalid_input",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayloadByteVerificationReport {
    pub schema_version: String,
    pub case_id: String,
    pub case_representation_sha256: String,
    pub catalog_id: String,
    pub catalog_sha256: String,
    pub descriptor_validation_disposition: String,
    pub verifier_version: String,
    /// Caller-supplied build identity; it is not authenticated by this module.
    pub verifier_build_sha256: Option<String>,
    pub verifier_build_identity_authenticated: bool,
    pub limits: PayloadVerificationLimits,
    pub disposition: PayloadVerificationDisposition,
    pub records: Vec<PayloadRecordVerification>,
    pub bytes_read_total: u64,
    pub payload_count: usize,
    pub verified_count: usize,
    pub unavailable_count: usize,
    pub failed_count: usize,
    pub action_authority: String,
    pub source_owner_authentication: String,
    pub rights_authentication: String,
    pub source_attestation_authentication: String,
    pub report_sha256: String,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadVerificationContext {
    pub verifier_build_sha256: Option<String>,
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

fn expected_cas_reference(digest: &str) -> String {
    format!("cas:sha256:{digest}")
}

fn record_key(kind: SourceRecordKind, record_id: &str) -> String {
    kind.case_key(record_id)
}

fn finding(
    severity: PayloadFindingSeverity,
    code: &str,
    key: &str,
    detail: &str,
) -> PayloadByteFinding {
    PayloadByteFinding {
        severity,
        code: code.into(),
        record_key: key.into(),
        detail: detail.into(),
    }
}

fn unavailable_record(
    descriptor: &crate::built_environment_source_descriptor::SourceRecordDescriptor,
    disposition: PayloadByteDisposition,
    code: &str,
    detail: &str,
    severity: PayloadFindingSeverity,
) -> PayloadRecordVerification {
    let key = record_key(descriptor.record_kind, &descriptor.case_record_id);
    PayloadRecordVerification {
        record_key: key.clone(),
        digest_basis: descriptor.source_payload.digest_basis.as_str().into(),
        object_reference: descriptor.source_payload.payload_object_ref.clone(),
        expected_sha256: descriptor.source_payload.sha256.clone(),
        observed_sha256: None,
        expected_byte_length: descriptor.source_payload.byte_length,
        observed_byte_length: None,
        bytes_read: 0,
        disposition,
        findings: vec![finding(severity, code, &key, detail)],
    }
}

#[derive(Debug)]
struct StreamResult {
    digest: Option<String>,
    bytes_read: u64,
    disposition: PayloadByteDisposition,
    findings: Vec<(PayloadFindingSeverity, &'static str, &'static str)>,
}

/// Hash a reader using a fixed-size buffer. The verifier reads at most one
/// sentinel byte beyond the applicable limit to distinguish an exact-size
/// payload from a truncated read.
fn hash_reader<R: Read>(
    reader: &mut R,
    expected_sha256: &str,
    expected_byte_length: Option<u64>,
    max_allowed_bytes: u64,
    limit_is_aggregate: bool,
    buffer_size_bytes: usize,
) -> StreamResult {
    let mut buffer = vec![0u8; buffer_size_bytes];
    let mut hasher = Sha256::new();
    let mut bytes_read = 0u64;

    loop {
        let remaining = max_allowed_bytes.saturating_sub(bytes_read);
        let read_cap = remaining.saturating_add(1).min(buffer_size_bytes as u64);
        let read_cap = usize::try_from(read_cap).unwrap_or(buffer_size_bytes);
        match reader.read(&mut buffer[..read_cap]) {
            Ok(0) => break,
            Ok(count) => {
                let count_u64 = u64::try_from(count).unwrap_or(u64::MAX);
                bytes_read = match bytes_read.checked_add(count_u64) {
                    Some(total) => total,
                    None => {
                        return StreamResult {
                            digest: None,
                            bytes_read: u64::MAX,
                            disposition: PayloadByteDisposition::ByteLimitExceeded,
                            findings: vec![(
                                PayloadFindingSeverity::Incomplete,
                                "PAYLOAD_BYTE_COUNT_OVERFLOW",
                                "streamed byte count exceeded u64",
                            )],
                        };
                    }
                };
                if bytes_read > max_allowed_bytes {
                    return StreamResult {
                        digest: None,
                        bytes_read,
                        disposition: PayloadByteDisposition::ByteLimitExceeded,
                        findings: vec![(
                            PayloadFindingSeverity::Incomplete,
                            if limit_is_aggregate {
                                "PAYLOAD_AGGREGATE_BYTE_LIMIT_EXCEEDED"
                            } else {
                                "PAYLOAD_PER_OBJECT_BYTE_LIMIT_EXCEEDED"
                            },
                            "stream exceeded the configured hard byte limit; no digest is accepted",
                        )],
                    };
                }
                hasher.update(&buffer[..count]);
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => {
                return StreamResult {
                    digest: None,
                    bytes_read,
                    disposition: PayloadByteDisposition::ReadFailed,
                    findings: vec![(
                        PayloadFindingSeverity::Incomplete,
                        "PAYLOAD_STREAM_READ_FAILED",
                        "payload stream ended with an I/O error",
                    )],
                };
            }
        }
    }

    let observed_sha256 = sha256_hex(&hasher.finalize());
    let mut findings = Vec::new();
    let mut disposition = PayloadByteDisposition::Verified;

    if observed_sha256 != expected_sha256 {
        disposition = PayloadByteDisposition::DigestMismatch;
        findings.push((
            PayloadFindingSeverity::Invalid,
            "SOURCE_PAYLOAD_DIGEST_MISMATCH",
            "streamed payload bytes do not match the descriptor SHA-256",
        ));
    }
    if expected_byte_length.is_some_and(|expected| expected != bytes_read) {
        disposition = PayloadByteDisposition::ByteLengthMismatch;
        findings.push((
            PayloadFindingSeverity::Invalid,
            "SOURCE_PAYLOAD_BYTE_LENGTH_MISMATCH",
            "streamed payload byte length does not match the descriptor length",
        ));
    }

    StreamResult {
        digest: Some(observed_sha256),
        bytes_read,
        disposition,
        findings,
    }
}

/// Verify all descriptor-backed payloads against exact bytes exposed by a
/// caller-supplied content-addressed resolver.
///
/// Invalid descriptor catalogues are rejected before the resolver is called.
/// Incomplete catalogues may still yield useful per-object byte checks, but
/// the aggregate report can never become fully_verified.
pub fn verify_source_descriptor_payloads<R: ContentAddressedPayloadResolver>(
    case: &BuiltEnvironmentCase,
    catalog: &SourceDescriptorCatalog,
    context: &PayloadVerificationContext,
    limits: &PayloadVerificationLimits,
    resolver: &mut R,
) -> PayloadByteVerificationReport {
    let descriptor_validation = validate_source_descriptor_catalog(case, catalog);
    let case_bytes = serde_json::to_vec(case).unwrap_or_default();
    let catalog_bytes = serde_json::to_vec(catalog).unwrap_or_default();
    let case_sha256 = sha256_hex(&case_bytes);
    let catalog_sha256 = sha256_hex(&catalog_bytes);

    let mut report_findings = Vec::new();
    let mut records = Vec::new();
    let mut bytes_read_total = 0u64;

    let limits_valid = limits.buffer_size_bytes > 0
        && limits.buffer_size_bytes <= MAX_BUFFER_BYTES
        && limits.max_bytes_per_payload > 0
        && limits.max_total_bytes > 0;
    if !limits_valid {
        report_findings.push(finding(
            PayloadFindingSeverity::Invalid,
            "PAYLOAD_VERIFICATION_LIMITS_INVALID",
            "limits",
            "buffer size must be 1..=1 MiB and both byte limits must be positive",
        ));
    }
    if let Some(build_sha256) = context.verifier_build_sha256.as_deref()
        && !valid_sha256(build_sha256)
    {
        report_findings.push(finding(
            PayloadFindingSeverity::Invalid,
            "PAYLOAD_VERIFIER_BUILD_DIGEST_INVALID",
            "verification_context",
            "supplied verifier build digest must be lowercase SHA-256",
        ));
    }

    if descriptor_validation.disposition == DescriptorDisposition::InvalidInput {
        for descriptor in &catalog.descriptors {
            records.push(unavailable_record(
                descriptor,
                PayloadByteDisposition::NotAttemptedInvalidCatalog,
                "INVALID_DESCRIPTOR_CATALOG_NOT_READ",
                "invalid descriptor catalogue was rejected before any payload object was opened",
                PayloadFindingSeverity::Invalid,
            ));
        }
        report_findings.push(finding(
            PayloadFindingSeverity::Invalid,
            "INVALID_DESCRIPTOR_CATALOG_NOT_READ",
            "descriptor_catalog",
            "payload resolution is prohibited for an invalid descriptor catalogue",
        ));
    } else if !limits_valid
        || context
            .verifier_build_sha256
            .as_deref()
            .is_some_and(|digest| !valid_sha256(digest))
    {
        for descriptor in &catalog.descriptors {
            records.push(unavailable_record(
                descriptor,
                PayloadByteDisposition::InvalidDescriptor,
                "PAYLOAD_VERIFICATION_NOT_STARTED",
                "invalid verifier configuration prevented payload resolution",
                PayloadFindingSeverity::Invalid,
            ));
        }
    } else {
        let mut aggregate_limit_reached = false;
        for descriptor in &catalog.descriptors {
            let key = record_key(descriptor.record_kind, &descriptor.case_record_id);
            if aggregate_limit_reached {
                records.push(unavailable_record(
                    descriptor,
                    PayloadByteDisposition::NotAttemptedAggregateLimit,
                    "PAYLOAD_AGGREGATE_BYTE_LIMIT_EXCEEDED",
                    "aggregate byte budget was exhausted before this record was opened",
                    PayloadFindingSeverity::Incomplete,
                ));
                continue;
            }

            let payload = &descriptor.source_payload;
            if payload.digest_basis == PayloadDigestBasis::Unavailable {
                records.push(unavailable_record(
                    descriptor,
                    PayloadByteDisposition::BytesUnavailable,
                    "SOURCE_PAYLOAD_BYTES_UNAVAILABLE",
                    "descriptor explicitly declares that source payload bytes are unavailable",
                    PayloadFindingSeverity::Incomplete,
                ));
                continue;
            }

            let Some(expected_sha256) = payload.sha256.as_deref() else {
                records.push(unavailable_record(
                    descriptor,
                    PayloadByteDisposition::InvalidDescriptor,
                    "SOURCE_PAYLOAD_DIGEST_MISSING",
                    "a byte-verifiable digest basis requires an expected SHA-256",
                    PayloadFindingSeverity::Invalid,
                ));
                continue;
            };
            if !valid_sha256(expected_sha256) {
                records.push(unavailable_record(
                    descriptor,
                    PayloadByteDisposition::InvalidDescriptor,
                    "SOURCE_PAYLOAD_DIGEST_INVALID",
                    "expected payload digest is not lowercase SHA-256",
                    PayloadFindingSeverity::Invalid,
                ));
                continue;
            }

            let Some(object_reference) = payload.payload_object_ref.as_deref() else {
                records.push(unavailable_record(
                    descriptor,
                    PayloadByteDisposition::BytesUnavailable,
                    "SOURCE_PAYLOAD_OBJECT_REFERENCE_UNAVAILABLE",
                    "no content-addressed object reference is available to the byte verifier",
                    PayloadFindingSeverity::Incomplete,
                ));
                continue;
            };
            if object_reference != expected_cas_reference(expected_sha256) {
                records.push(unavailable_record(
                    descriptor,
                    PayloadByteDisposition::InvalidDescriptor,
                    "SOURCE_PAYLOAD_OBJECT_REFERENCE_DIGEST_MISMATCH",
                    "content-addressed object reference does not bind the expected SHA-256",
                    PayloadFindingSeverity::Invalid,
                ));
                continue;
            }

            let mut reader = match resolver.open(object_reference) {
                Ok(reader) => reader,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    records.push(unavailable_record(
                        descriptor,
                        PayloadByteDisposition::ObjectNotFound,
                        "SOURCE_PAYLOAD_OBJECT_NOT_FOUND",
                        "content-addressed payload object was not found",
                        PayloadFindingSeverity::Incomplete,
                    ));
                    continue;
                }
                Err(_) => {
                    records.push(unavailable_record(
                        descriptor,
                        PayloadByteDisposition::ReadFailed,
                        "SOURCE_PAYLOAD_OBJECT_OPEN_FAILED",
                        "content-addressed payload object could not be opened",
                        PayloadFindingSeverity::Incomplete,
                    ));
                    continue;
                }
            };

            let remaining_total = limits.max_total_bytes.saturating_sub(bytes_read_total);
            let effective_limit = limits.max_bytes_per_payload.min(remaining_total);
            let limit_is_aggregate = remaining_total < limits.max_bytes_per_payload;
            let streamed = hash_reader(
                &mut reader,
                expected_sha256,
                payload.byte_length,
                effective_limit,
                limit_is_aggregate,
                limits.buffer_size_bytes,
            );
            bytes_read_total = match bytes_read_total.checked_add(streamed.bytes_read) {
                Some(total) => total,
                None => {
                    report_findings.push(finding(
                        PayloadFindingSeverity::Invalid,
                        "PAYLOAD_TOTAL_BYTE_COUNT_OVERFLOW",
                        &key,
                        "aggregate streamed byte count overflowed u64",
                    ));
                    u64::MAX
                }
            };
            if limit_is_aggregate
                && streamed.disposition == PayloadByteDisposition::ByteLimitExceeded
            {
                aggregate_limit_reached = true;
            }

            let mut findings = streamed
                .findings
                .iter()
                .map(|(severity, code, detail)| finding(*severity, code, &key, detail))
                .collect::<Vec<_>>();
            let disposition = streamed.disposition;
            // Authentication and authority limitations are reported at
            // report level; they must not downgrade a successful byte hash.
            records.push(PayloadRecordVerification {
                record_key: key,
                digest_basis: payload.digest_basis.as_str().into(),
                object_reference: Some(object_reference.into()),
                expected_sha256: Some(expected_sha256.into()),
                observed_sha256: streamed.digest,
                expected_byte_length: payload.byte_length,
                observed_byte_length: Some(streamed.bytes_read),
                bytes_read: streamed.bytes_read,
                disposition,
                findings: std::mem::take(&mut findings),
            });
        }
    }

    for record in &records {
        report_findings.extend(record.findings.iter().cloned());
    }

    let invalid = descriptor_validation.disposition == DescriptorDisposition::InvalidInput
        || !limits_valid
        || context
            .verifier_build_sha256
            .as_deref()
            .is_some_and(|digest| !valid_sha256(digest))
        || report_findings
            .iter()
            .any(|finding| finding.severity == PayloadFindingSeverity::Invalid);
    let incomplete = descriptor_validation.disposition
        == DescriptorDisposition::IncompleteOrIndeterminate
        || records
            .iter()
            .any(|record| record.disposition != PayloadByteDisposition::Verified)
        || context.verifier_build_sha256.is_none()
        || report_findings
            .iter()
            .any(|finding| finding.severity == PayloadFindingSeverity::Incomplete);

    let disposition = if invalid {
        PayloadVerificationDisposition::InvalidInput
    } else if incomplete {
        PayloadVerificationDisposition::IncompleteOrIndeterminate
    } else {
        PayloadVerificationDisposition::FullyVerified
    };

    let verified_count = records
        .iter()
        .filter(|record| record.disposition == PayloadByteDisposition::Verified)
        .count();
    let unavailable_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.disposition,
                PayloadByteDisposition::BytesUnavailable
                    | PayloadByteDisposition::ObjectNotFound
                    | PayloadByteDisposition::NotAttemptedAggregateLimit
            )
        })
        .count();
    let failed_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.disposition,
                PayloadByteDisposition::ReadFailed
                    | PayloadByteDisposition::ByteLimitExceeded
                    | PayloadByteDisposition::DigestMismatch
                    | PayloadByteDisposition::ByteLengthMismatch
                    | PayloadByteDisposition::InvalidDescriptor
                    | PayloadByteDisposition::NotAttemptedInvalidCatalog
            )
        })
        .count();

    let mut report = PayloadByteVerificationReport {
        schema_version: "luminous.source-payload-byte-verification.v0.1".into(),
        case_id: case.case_id.clone(),
        case_representation_sha256: case_sha256,
        catalog_id: catalog.catalog_id.clone(),
        catalog_sha256,
        descriptor_validation_disposition: descriptor_validation.disposition.as_str().into(),
        verifier_version: env!("CARGO_PKG_VERSION").into(),
        verifier_build_sha256: context.verifier_build_sha256.clone(),
        verifier_build_identity_authenticated: false,
        limits: limits.clone(),
        disposition,
        records,
        bytes_read_total,
        payload_count: catalog.descriptors.len(),
        verified_count,
        unavailable_count,
        failed_count,
        action_authority: "none".into(),
        source_owner_authentication: "not_performed".into(),
        rights_authentication: "not_performed".into(),
        source_attestation_authentication: "not_performed".into(),
        report_sha256: String::new(),
        limitations: vec![
            "A matching digest proves byte equality with the descriptor's declared digest only.".into(),
            "The resolver's storage custody, transport security and object-retention policy are outside this verifier.".into(),
            "Source-owner identity, signatures, rights, snapshot completeness, source truth and engineering correctness are not authenticated.".into(),
            "A caller-supplied verifier build digest is recorded but not independently authenticated here.".into(),
            "No action authority or NoImpact eligibility is granted by byte verification.".into(),
        ],
    };
    if let Ok(value) = serde_json::to_value(&report) {
        let mut core = value;
        if let Some(object) = core.as_object_mut() {
            object.remove("report_sha256");
        }
        if let Ok(bytes) = serde_json::to_vec(&core) {
            report.report_sha256 = sha256_hex(&bytes);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_environment_case::{DataRights, EvidenceRecord, EvidenceState};
    use crate::built_environment_source_descriptor::{
        NativeIdentityComponent, SourceAttestationDescriptor, SourceObservationTime,
        SourcePayloadDescriptor, SourceRecordDescriptor, SourceRightsDescriptor, SourceRightsState,
        SourceSnapshotDescriptor,
    };
    use std::collections::BTreeMap;
    use std::io::Cursor;

    struct FailingReader {
        emitted_partial: bool,
    }

    impl Read for FailingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if !self.emitted_partial {
                self.emitted_partial = true;
                let partial = b"partial";
                let count = buffer.len().min(partial.len());
                buffer[..count].copy_from_slice(&partial[..count]);
                return Ok(count);
            }
            Err(io::Error::other("synthetic stream read failure"))
        }
    }

    struct FailingReadResolver;

    impl ContentAddressedPayloadResolver for FailingReadResolver {
        type Reader = FailingReader;

        fn open(&mut self, _object_reference: &str) -> io::Result<Self::Reader> {
            Ok(FailingReader {
                emitted_partial: false,
            })
        }
    }

    #[derive(Default)]
    struct MemoryResolver {
        objects: BTreeMap<String, Vec<u8>>,
        fail_open: bool,
        open_count: usize,
    }

    impl ContentAddressedPayloadResolver for MemoryResolver {
        type Reader = Cursor<Vec<u8>>;

        fn open(&mut self, object_reference: &str) -> io::Result<Self::Reader> {
            self.open_count += 1;
            if self.fail_open {
                return Err(io::Error::other("synthetic resolver failure"));
            }
            self.objects
                .get(object_reference)
                .cloned()
                .map(Cursor::new)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "object missing"))
        }
    }

    fn sha256(bytes: &[u8]) -> String {
        super::sha256_hex(bytes)
    }

    fn case() -> BuiltEnvironmentCase {
        BuiltEnvironmentCase {
            case_id: "payload-verification-case-001".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence: vec![EvidenceRecord {
                evidence_id: "evidence-payload-001".into(),
                subject_id: "design-001".into(),
                field_path: "geometry.source".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: None,
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
        let digest = sha256(bytes);
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
                    name: "field_path".into(),
                    value: "geometry.source".into(),
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
                sha256: Some(digest.clone()),
                digest_basis: PayloadDigestBasis::LocallyRetrievedAndHashed,
                bytes_retrieved: true,
                byte_length: Some(bytes.len() as u64),
                payload_object_ref: Some(format!("cas:sha256:{digest}")),
                media_type: Some("application/octet-stream".into()),
                schema_id: Some("urn:synthetic:payload".into()),
                schema_version: Some("1".into()),
                schema_sha256: Some(sha256(b"schema")),
            },
            source_snapshot: SourceSnapshotDescriptor {
                source_id: "survey-registry".into(),
                snapshot_id: "snapshot-001".into(),
                snapshot_sha256: sha256(b"snapshot"),
                captured_at: "2026-11-03T10:00:00Z".into(),
                retrieved_at: "2026-11-03T10:01:00Z".into(),
            },
            observation_time: SourceObservationTime {
                observed_at: Some("2026-11-03T09:59:00Z".into()),
                recorded_at: "2026-11-03T10:00:00Z".into(),
            },
            rights: SourceRightsDescriptor {
                state: SourceRightsState::Verified,
                permitted_uses: vec!["impact_analysis".into()],
                rights_evidence_ref: Some("synthetic:rights:001".into()),
            },
            attestation: SourceAttestationDescriptor {
                statement_ref: Some("synthetic:attestation:001".into()),
                key_id: Some("urn:synthetic:key:001".into()),
                trust_evaluation: "not_evaluated_by_descriptor_catalog".into(),
            },
        }
    }

    fn catalog(case: &BuiltEnvironmentCase, payload: &[u8]) -> SourceDescriptorCatalog {
        SourceDescriptorCatalog {
            schema_version: "luminous.built-environment-source-descriptor-catalog.v0.1".into(),
            catalog_id: "payload-catalog-001".into(),
            case_id: case.case_id.clone(),
            case_representation_sha256: sha256(&serde_json::to_vec(case).unwrap()),
            captured_at: "2026-11-03T10:02:00Z".into(),
            descriptors: vec![descriptor(payload)],
            claim_ceiling: "descriptor_integrity_only".into(),
            action_authority: "none".into(),
            limitations: vec!["synthetic-only".into()],
        }
    }

    fn context() -> PayloadVerificationContext {
        PayloadVerificationContext {
            verifier_build_sha256: Some(sha256(b"synthetic-verifier-build")),
        }
    }

    fn limits() -> PayloadVerificationLimits {
        PayloadVerificationLimits {
            buffer_size_bytes: 64,
            max_bytes_per_payload: 1024,
            max_total_bytes: 1024,
        }
    }

    #[test]
    fn exact_payload_bytes_verify_digest_and_length() {
        let bytes = b"exact source payload bytes";
        let case = case();
        let catalog = catalog(&case, bytes);
        let reference = format!("cas:sha256:{}", sha256(bytes));
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, bytes.to_vec());

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::FullyVerified,
            "unexpected byte verification report: {report:#?}"
        );
        assert_eq!(report.verified_count, 1);
        assert_eq!(
            report.records[0].observed_sha256.as_deref(),
            Some(sha256(bytes).as_str())
        );
        assert_eq!(
            report.records[0].observed_byte_length,
            Some(bytes.len() as u64)
        );
        assert_eq!(report.action_authority, "none");
        assert_eq!(report.source_owner_authentication, "not_performed");
        assert!(valid_sha256(&report.report_sha256));
    }

    #[test]
    fn one_byte_mutation_fails_the_exact_digest_check() {
        let bytes = b"original";
        let mutated = b"originaL";
        let case = case();
        let catalog = catalog(&case, bytes);
        let reference = format!("cas:sha256:{}", sha256(bytes));
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, mutated.to_vec());

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::InvalidInput
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::DigestMismatch
        );
    }

    #[test]
    fn correct_digest_with_wrong_declared_length_is_invalid() {
        let bytes = b"exact";
        let case = case();
        let mut catalog = catalog(&case, bytes);
        catalog.descriptors[0].source_payload.byte_length = Some(999);
        let reference = format!("cas:sha256:{}", sha256(bytes));
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, bytes.to_vec());

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::InvalidInput
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::ByteLengthMismatch
        );
    }

    #[test]
    fn missing_object_is_incomplete_not_verified() {
        let bytes = b"missing";
        let case = case();
        let catalog = catalog(&case, bytes);
        let mut resolver = MemoryResolver::default();

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::ObjectNotFound
        );
    }

    #[test]
    fn io_error_is_incomplete_not_a_digest_mismatch() {
        let bytes = b"read failure";
        let case = case();
        let catalog = catalog(&case, bytes);
        let mut resolver = MemoryResolver {
            objects: BTreeMap::new(),
            fail_open: true,
            open_count: 0,
        };

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::ReadFailed
        );
    }

    #[test]
    fn stream_read_error_after_partial_bytes_is_not_misreported_as_digest_mismatch() {
        let bytes = b"partial-and-more";
        let case = case();
        let catalog = catalog(&case, bytes);
        let mut resolver = FailingReadResolver;

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::ReadFailed
        );
        assert_eq!(report.records[0].bytes_read, 7);
        assert!(report.records[0].observed_sha256.is_none());
    }

    #[test]
    fn max_byte_limit_reads_only_one_sentinel_byte_beyond_limit() {
        let bytes = vec![b'x'; 100];
        let case = case();
        let catalog = catalog(&case, &bytes);
        let reference = format!("cas:sha256:{}", sha256(&bytes));
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, bytes);

        let limited = PayloadVerificationLimits {
            buffer_size_bytes: 16,
            max_bytes_per_payload: 10,
            max_total_bytes: 1024,
        };
        let report =
            verify_source_descriptor_payloads(&case, &catalog, &context(), &limited, &mut resolver);
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::ByteLimitExceeded
        );
        assert_eq!(report.records[0].bytes_read, 11);
    }

    #[test]
    fn invalid_descriptor_catalog_is_rejected_before_resolver_access() {
        let bytes = b"invalid catalog";
        let case = case();
        let mut catalog = catalog(&case, bytes);
        catalog.descriptors[0].source_payload.sha256 = Some("not-a-digest".into());
        let mut resolver = MemoryResolver::default();

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::InvalidInput
        );
        assert_eq!(resolver.open_count, 0);
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::NotAttemptedInvalidCatalog
        );
    }

    #[test]
    fn incomplete_catalog_never_upgrades_after_successful_byte_checks() {
        let bytes = b"valid bytes but incomplete metadata";
        let case = case();
        let mut catalog = catalog(&case, bytes);
        catalog.descriptors[0].source_payload.schema_sha256 = None;
        let reference = format!("cas:sha256:{}", sha256(bytes));
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, bytes.to_vec());

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::Verified
        );
    }

    #[test]
    fn source_owner_attested_without_local_object_remains_unavailable() {
        let bytes = b"owner attested";
        let case = case();
        let mut catalog = catalog(&case, bytes);
        catalog.descriptors[0].source_payload.digest_basis =
            PayloadDigestBasis::SourceOwnerAttested;
        catalog.descriptors[0].source_payload.bytes_retrieved = false;
        catalog.descriptors[0].source_payload.payload_object_ref = None;
        let mut resolver = MemoryResolver::default();

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(resolver.open_count, 0);
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::BytesUnavailable
        );
    }

    #[test]
    fn owner_attested_digest_can_be_independently_rehashed_when_bytes_are_available() {
        let bytes = b"owner attested but independently available";
        let case = case();
        let mut catalog = catalog(&case, bytes);
        catalog.descriptors[0].source_payload.digest_basis =
            PayloadDigestBasis::SourceOwnerAttested;
        catalog.descriptors[0].source_payload.bytes_retrieved = false;
        let reference = format!("cas:sha256:{}", sha256(bytes));
        catalog.descriptors[0].source_payload.payload_object_ref = Some(reference.clone());
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, bytes.to_vec());

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::Verified
        );
        assert_eq!(report.source_attestation_authentication, "not_performed");
    }

    #[test]
    fn unavailable_payload_never_causes_a_resolver_open() {
        let case = case();
        let mut catalog = catalog(&case, b"unused");
        let payload = &mut catalog.descriptors[0].source_payload;
        payload.sha256 = None;
        payload.digest_basis = PayloadDigestBasis::Unavailable;
        payload.bytes_retrieved = false;
        payload.byte_length = None;
        payload.payload_object_ref = None;
        let mut resolver = MemoryResolver::default();

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(resolver.open_count, 0);
        assert_eq!(
            report.records[0].disposition,
            PayloadByteDisposition::BytesUnavailable
        );
    }

    #[test]
    fn zero_length_payload_is_verified() {
        let bytes: &[u8] = b"";
        let case = case();
        let catalog = catalog(&case, bytes);
        let reference = format!("cas:sha256:{}", sha256(bytes));
        let mut resolver = MemoryResolver::default();
        resolver.objects.insert(reference, Vec::new());

        let report = verify_source_descriptor_payloads(
            &case,
            &catalog,
            &context(),
            &limits(),
            &mut resolver,
        );
        assert_eq!(
            report.disposition,
            PayloadVerificationDisposition::FullyVerified
        );
        assert_eq!(report.records[0].observed_byte_length, Some(0));
    }
}
