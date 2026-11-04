// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bounded streaming verification of descriptor-referenced source payload bytes.
//!
//! This module verifies that bytes obtained through a caller-owned resolver
//! match a descriptor's declared SHA-256 and optional byte length. It does not
//! authenticate the resolver, object-store origin, owner identity, rights,
//! signatures, source inventory completeness, or engineering correctness.

use crate::built_environment_case::BuiltEnvironmentCase;
use crate::built_environment_source_descriptor::{
    DescriptorDisposition, DescriptorFinding, DescriptorValidation, PayloadDigestBasis,
    SourceDescriptorCatalog, SourceRecordDescriptor, validate_source_descriptor_catalog,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::{self, Read};

const VERIFIER_SCHEMA: &str = "luminous.source-payload-byte-verification.v0.1";
const BUFFER_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadByteStatus {
    Verified,
    SourceBytesUnavailable,
    ReadFailed,
    ByteLimitExceeded,
    InvalidDescriptor,
    DigestMismatch,
    LengthMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadByteCatalogDisposition {
    AllResolvablePayloadsVerified,
    IncompleteOrIndeterminate,
    InvalidInput,
}

impl PayloadByteCatalogDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllResolvablePayloadsVerified => "all_resolvable_payloads_verified",
            Self::IncompleteOrIndeterminate => "incomplete_or_indeterminate",
            Self::InvalidInput => "invalid_input",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PayloadByteFinding {
    pub severity: String,
    pub code: String,
    pub record_key: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayloadByteRecordResult {
    pub record_key: String,
    pub status: PayloadByteStatus,
    pub digest_basis: PayloadDigestBasis,
    pub object_ref: Option<String>,
    pub expected_sha256: Option<String>,
    pub actual_sha256: Option<String>,
    pub expected_byte_length: Option<u64>,
    pub actual_byte_length: Option<u64>,
    /// Number of bytes consumed from the resolver stream. On a limit breach,
    /// this may be one byte greater than the configured limit.
    pub bytes_read: u64,
    pub read_error_kind: Option<String>,
    pub findings: Vec<PayloadByteFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayloadByteVerificationContext {
    /// Canonical UTC whole-second time at which this verifier evaluated bytes.
    pub verified_at_utc: String,
    /// Hard per-object bound. Zero permits only an empty object.
    pub max_bytes_per_payload: u64,
    /// Digest of the exact verifier executable/build artifact.
    pub verifier_build_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PayloadByteCatalogReport {
    pub schema_version: String,
    pub disposition: PayloadByteCatalogDisposition,
    pub case_id: String,
    pub case_representation_sha256: String,
    pub catalog_id: String,
    pub catalog_representation_sha256: String,
    pub verifier_build_sha256: Option<String>,
    pub verified_at_utc: String,
    pub max_bytes_per_payload: u64,
    pub descriptor_validation_disposition: String,
    pub descriptor_validation_findings: Vec<DescriptorFinding>,
    pub descriptor_count: usize,
    pub verified_count: usize,
    pub unavailable_count: usize,
    pub read_failed_count: usize,
    pub byte_limit_exceeded_count: usize,
    pub invalid_count: usize,
    pub records: Vec<PayloadByteRecordResult>,
    pub findings: Vec<PayloadByteFinding>,
    pub source_owner_authentication: String,
    pub source_snapshot_authentication: String,
    pub rights_authentication: String,
    pub attestation_authentication: String,
    pub source_inventory_completeness: String,
    pub engineering_correctness: String,
    pub action_authority: String,
    pub no_impact_eligibility: String,
}

/// Verify one exact descriptor payload using a stream already resolved by the
/// caller. The stream is read in fixed-size chunks and is never buffered whole.
///
/// A successful result means only that the observed bytes match the digest
/// (and byte length, when provided) stated in this descriptor.
pub fn verify_descriptor_payload_stream<R: Read>(
    descriptor: &SourceRecordDescriptor,
    record_key: &str,
    reader: R,
    max_bytes: u64,
) -> PayloadByteRecordResult {
    let payload = &descriptor.source_payload;
    let mut findings = Vec::new();

    let result =
        |status, actual_sha256, actual_byte_length, bytes_read, read_error_kind, findings| {
            PayloadByteRecordResult {
                record_key: record_key.to_owned(),
                status,
                digest_basis: payload.digest_basis,
                object_ref: payload.payload_object_ref.clone(),
                expected_sha256: payload.sha256.clone(),
                actual_sha256,
                expected_byte_length: payload.byte_length,
                actual_byte_length,
                bytes_read,
                read_error_kind,
                findings,
            }
        };

    let Some(expected_sha256) = payload.sha256.as_deref() else {
        findings.push(finding(
            "incomplete",
            "SOURCE_PAYLOAD_DIGEST_UNAVAILABLE",
            record_key,
            "no expected payload SHA-256 is available; byte equality cannot be established",
        ));
        return result(
            PayloadByteStatus::SourceBytesUnavailable,
            None,
            None,
            0,
            None,
            findings,
        );
    };
    if !valid_sha256(expected_sha256) {
        findings.push(finding(
            "invalid",
            "INVALID_EXPECTED_PAYLOAD_DIGEST",
            record_key,
            "expected payload digest is not lowercase SHA-256",
        ));
        return result(
            PayloadByteStatus::InvalidDescriptor,
            None,
            None,
            0,
            None,
            findings,
        );
    }
    let Some(object_ref) = payload.payload_object_ref.as_deref() else {
        findings.push(finding(
            "incomplete",
            "SOURCE_PAYLOAD_OBJECT_REFERENCE_UNAVAILABLE",
            record_key,
            "descriptor has no content-addressed object reference to resolve",
        ));
        return result(
            PayloadByteStatus::SourceBytesUnavailable,
            None,
            None,
            0,
            None,
            findings,
        );
    };
    if object_ref != cas_reference(expected_sha256) {
        findings.push(finding(
            "invalid",
            "PAYLOAD_OBJECT_REFERENCE_DIGEST_MISMATCH",
            record_key,
            "object reference does not bind the declared payload SHA-256",
        ));
        return result(
            PayloadByteStatus::InvalidDescriptor,
            None,
            None,
            0,
            None,
            findings,
        );
    }
    if payload.digest_basis == PayloadDigestBasis::Unavailable {
        findings.push(finding(
            "invalid",
            "UNAVAILABLE_PAYLOAD_HAS_OBJECT_REFERENCE",
            record_key,
            "unavailable digest basis cannot be resolved through an object reference",
        ));
        return result(
            PayloadByteStatus::InvalidDescriptor,
            None,
            None,
            0,
            None,
            findings,
        );
    }

    let mut reader = reader;
    let mut hasher = Sha256::new();
    let mut bytes_read = 0_u64;
    let mut buffer = [0_u8; BUFFER_BYTES];

    loop {
        let remaining = max_bytes.saturating_sub(bytes_read);
        // Read at most one byte beyond the configured limit so an oversized
        // object is distinguishable from an exact-limit object ending at EOF.
        let read_cap = if remaining == u64::MAX {
            BUFFER_BYTES
        } else {
            (remaining + 1).min(BUFFER_BYTES as u64) as usize
        };
        match reader.read(&mut buffer[..read_cap]) {
            Ok(0) => break,
            Ok(count) => {
                let Some(next_total) = bytes_read.checked_add(count as u64) else {
                    findings.push(finding(
                        "incomplete",
                        "SOURCE_PAYLOAD_BYTE_COUNT_OVERFLOW",
                        record_key,
                        "stream byte count overflowed u64",
                    ));
                    return result(
                        PayloadByteStatus::ByteLimitExceeded,
                        None,
                        None,
                        bytes_read,
                        None,
                        findings,
                    );
                };
                if next_total > max_bytes {
                    bytes_read = next_total;
                    findings.push(finding(
                        "incomplete",
                        "SOURCE_PAYLOAD_BYTE_LIMIT_EXCEEDED",
                        record_key,
                        "payload exceeded the configured per-object byte limit; no digest conclusion was made",
                    ));
                    return result(
                        PayloadByteStatus::ByteLimitExceeded,
                        None,
                        None,
                        bytes_read,
                        None,
                        findings,
                    );
                }
                hasher.update(&buffer[..count]);
                bytes_read = next_total;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                let kind = format!("{:?}", error.kind());
                findings.push(finding(
                    "incomplete",
                    "SOURCE_PAYLOAD_READ_FAILED",
                    record_key,
                    format!("payload stream read failed with IO kind {kind}"),
                ));
                return result(
                    PayloadByteStatus::ReadFailed,
                    None,
                    Some(bytes_read),
                    bytes_read,
                    Some(kind),
                    findings,
                );
            }
        }
    }

    let actual_sha256 = hex_digest(hasher.finalize().as_slice());
    let mut mismatch = false;
    if actual_sha256 != expected_sha256 {
        mismatch = true;
        findings.push(finding(
            "invalid",
            "SOURCE_PAYLOAD_SHA256_MISMATCH",
            record_key,
            "recomputed payload SHA-256 differs from the descriptor's expected digest",
        ));
    }
    if let Some(expected_length) = payload.byte_length {
        if expected_length != bytes_read {
            mismatch = true;
            findings.push(finding(
                "invalid",
                "SOURCE_PAYLOAD_BYTE_LENGTH_MISMATCH",
                record_key,
                "recomputed payload byte length differs from the descriptor's expected length",
            ));
        }
    }
    if mismatch {
        let status = if actual_sha256 != expected_sha256 {
            PayloadByteStatus::DigestMismatch
        } else {
            PayloadByteStatus::LengthMismatch
        };
        return result(
            status,
            Some(actual_sha256),
            Some(bytes_read),
            bytes_read,
            None,
            findings,
        );
    }

    result(
        PayloadByteStatus::Verified,
        Some(actual_sha256),
        Some(bytes_read),
        bytes_read,
        None,
        findings,
    )
}

/// Verify every resolvable payload in one descriptor catalogue.
///
/// The resolver is the only storage-specific component. It must return the
/// bytes for the exact opaque content-addressed object reference requested.
/// The verifier rehashes the returned stream itself.
pub fn verify_catalog_payload_bytes<R, F>(
    case: &BuiltEnvironmentCase,
    catalog: &SourceDescriptorCatalog,
    context: &PayloadByteVerificationContext,
    mut open_payload: F,
) -> PayloadByteCatalogReport
where
    R: Read,
    F: FnMut(&str) -> io::Result<R>,
{
    let case_bytes = serde_json::to_vec(case).unwrap_or_default();
    let catalog_bytes = serde_json::to_vec(catalog).unwrap_or_default();
    let case_sha256 = sha256_hex(&case_bytes);
    let catalog_sha256 = sha256_hex(&catalog_bytes);
    let descriptor_validation = validate_source_descriptor_catalog(case, catalog);
    let mut report_findings = BTreeSet::new();

    if !valid_sha256(&case_sha256) || !valid_sha256(&catalog_sha256) {
        report_findings.insert(finding(
            "invalid",
            "REPORT_BINDING_DIGEST_FAILED",
            "catalog",
            "case or descriptor catalogue representation could not be bound",
        ));
    }
    if !valid_utc_timestamp(&context.verified_at_utc) {
        report_findings.insert(finding(
            "invalid",
            "INVALID_VERIFICATION_TIME",
            "verifier",
            "verified_at_utc must be a canonical UTC whole-second timestamp",
        ));
    }
    match context.verifier_build_sha256.as_deref() {
        None => {
            report_findings.insert(finding(
                "incomplete",
                "VERIFIER_BUILD_IDENTITY_UNAVAILABLE",
                "verifier",
                "exact verifier build digest was not supplied",
            ));
        }
        Some(digest) if !valid_sha256(digest) => {
            report_findings.insert(finding(
                "invalid",
                "INVALID_VERIFIER_BUILD_DIGEST",
                "verifier",
                "verifier build digest is not lowercase SHA-256",
            ));
        }
        Some(_) => {}
    }

    let invalid_verifier_context = !valid_utc_timestamp(&context.verified_at_utc)
        || context
            .verifier_build_sha256
            .as_deref()
            .is_some_and(|digest| !valid_sha256(digest));
    if invalid_verifier_context {
        return catalog_report(
            &case.case_id,
            catalog,
            context,
            case_sha256,
            catalog_sha256,
            &descriptor_validation,
            Vec::new(),
            report_findings,
        );
    }

    if descriptor_validation.disposition == DescriptorDisposition::InvalidInput {
        report_findings.insert(finding(
            "invalid",
            "SOURCE_DESCRIPTOR_CATALOG_INVALID",
            "catalog",
            "invalid descriptor catalogue was rejected before any payload object was opened",
        ));
        return catalog_report(
            catalog,
            context,
            case_sha256,
            catalog_sha256,
            &descriptor_validation,
            Vec::new(),
            report_findings,
        );
    }

    let mut records = Vec::with_capacity(catalog.descriptors.len());
    for descriptor in &catalog.descriptors {
        let record_key = descriptor.record_kind.case_key(&descriptor.case_record_id);
        let Some(object_ref) = descriptor.source_payload.payload_object_ref.as_deref() else {
            let result = unavailable_result(
                descriptor,
                &record_key,
                "SOURCE_PAYLOAD_OBJECT_REFERENCE_UNAVAILABLE",
                "no content-addressed payload object reference is available",
            );
            report_findings.extend(result.findings.iter().cloned());
            records.push(result);
            continue;
        };

        match open_payload(object_ref) {
            Ok(reader) => {
                let result = verify_descriptor_payload_stream(
                    descriptor,
                    &record_key,
                    reader,
                    context.max_bytes_per_payload,
                );
                report_findings.extend(result.findings.iter().cloned());
                records.push(result);
            }
            Err(error) => {
                let error_kind = format!("{:?}", error.kind());
                let result = PayloadByteRecordResult {
                    record_key: record_key.clone(),
                    status: PayloadByteStatus::SourceBytesUnavailable,
                    digest_basis: descriptor.source_payload.digest_basis,
                    object_ref: Some(object_ref.to_owned()),
                    expected_sha256: descriptor.source_payload.sha256.clone(),
                    actual_sha256: None,
                    expected_byte_length: descriptor.source_payload.byte_length,
                    actual_byte_length: None,
                    bytes_read: 0,
                    read_error_kind: Some(error_kind.clone()),
                    findings: vec![finding(
                        "incomplete",
                        "SOURCE_PAYLOAD_OBJECT_OPEN_FAILED",
                        &record_key,
                        format!("payload resolver could not open object (IO kind {error_kind})"),
                    )],
                };
                report_findings.extend(result.findings.iter().cloned());
                records.push(result);
            }
        }
    }

    records.sort_by(|left, right| left.record_key.cmp(&right.record_key));
    catalog_report(
        &case.case_id,
        catalog,
        context,
        case_sha256,
        catalog_sha256,
        &descriptor_validation,
        records,
        report_findings,
    )
}

fn catalog_report(
    case_id: &str,
    catalog: &SourceDescriptorCatalog,
    context: &PayloadByteVerificationContext,
    case_sha256: String,
    catalog_sha256: String,
    descriptor_validation: &DescriptorValidation,
    records: Vec<PayloadByteRecordResult>,
    findings: BTreeSet<PayloadByteFinding>,
) -> PayloadByteCatalogReport {
    let verified_count = records
        .iter()
        .filter(|record| record.status == PayloadByteStatus::Verified)
        .count();
    let unavailable_count = records
        .iter()
        .filter(|record| record.status == PayloadByteStatus::SourceBytesUnavailable)
        .count();
    let read_failed_count = records
        .iter()
        .filter(|record| record.status == PayloadByteStatus::ReadFailed)
        .count();
    let byte_limit_exceeded_count = records
        .iter()
        .filter(|record| record.status == PayloadByteStatus::ByteLimitExceeded)
        .count();
    let invalid_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.status,
                PayloadByteStatus::InvalidDescriptor
                    | PayloadByteStatus::DigestMismatch
                    | PayloadByteStatus::LengthMismatch
            )
        })
        .count();

    let invalid_descriptor =
        descriptor_validation.disposition == DescriptorDisposition::InvalidInput;
    let disposition = if invalid_descriptor || invalid_count > 0 {
        PayloadByteCatalogDisposition::InvalidInput
    } else if descriptor_validation.disposition == DescriptorDisposition::IncompleteOrIndeterminate
        || verified_count + unavailable_count + read_failed_count + byte_limit_exceeded_count
            != records.len()
        || unavailable_count > 0
        || read_failed_count > 0
        || byte_limit_exceeded_count > 0
        || findings
            .iter()
            .any(|finding| finding.code == "VERIFIER_BUILD_IDENTITY_UNAVAILABLE")
    {
        PayloadByteCatalogDisposition::IncompleteOrIndeterminate
    } else {
        PayloadByteCatalogDisposition::AllResolvablePayloadsVerified
    };

    let report = PayloadByteCatalogReport {
        schema_version: VERIFIER_SCHEMA.into(),
        disposition,
        case_id: case_id.to_owned(),
        case_representation_sha256: case_sha256,
        catalog_id: catalog.catalog_id.clone(),
        catalog_representation_sha256: catalog_sha256,
        verifier_build_sha256: context.verifier_build_sha256.clone(),
        verified_at_utc: context.verified_at_utc.clone(),
        max_bytes_per_payload: context.max_bytes_per_payload,
        descriptor_validation_disposition: descriptor_validation.disposition.as_str().into(),
        descriptor_validation_findings: descriptor_validation.findings.clone(),
        descriptor_count: catalog.descriptors.len(),
        verified_count,
        unavailable_count,
        read_failed_count,
        byte_limit_exceeded_count,
        invalid_count,
        records,
        findings: findings.iter().cloned().collect(),
        source_owner_authentication: "not_performed".into(),
        source_snapshot_authentication: "not_performed".into(),
        rights_authentication: "not_performed".into(),
        attestation_authentication: "not_performed".into(),
        source_inventory_completeness: "not_performed".into(),
        engineering_correctness: "not_performed".into(),
        action_authority: "none".into(),
        no_impact_eligibility: "none".into(),
    };
    report
}

fn unavailable_result(
    descriptor: &SourceRecordDescriptor,
    record_key: &str,
    code: &str,
    detail: &str,
) -> PayloadByteRecordResult {
    PayloadByteRecordResult {
        record_key: record_key.to_owned(),
        status: PayloadByteStatus::SourceBytesUnavailable,
        digest_basis: descriptor.source_payload.digest_basis,
        object_ref: descriptor.source_payload.payload_object_ref.clone(),
        expected_sha256: descriptor.source_payload.sha256.clone(),
        actual_sha256: None,
        expected_byte_length: descriptor.source_payload.byte_length,
        actual_byte_length: None,
        bytes_read: 0,
        read_error_kind: None,
        findings: vec![finding("incomplete", code, record_key, detail)],
    }
}

fn finding(
    severity: &str,
    code: &str,
    record_key: &str,
    detail: impl Into<String>,
) -> PayloadByteFinding {
    PayloadByteFinding {
        severity: severity.into(),
        code: code.into(),
        record_key: record_key.into(),
        detail: detail.into(),
    }
}

fn cas_reference(sha256: &str) -> String {
    format!("cas:sha256:{sha256}")
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
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= max_day
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn sha256_hex(bytes: &[u8]) -> String {
    {
        let digest = Sha256::digest(bytes);
        hex_digest(&digest)
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
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
    use std::io::Cursor;

    fn sha(bytes: &[u8]) -> String {
        sha256_hex(bytes)
    }

    fn test_case(bytes: &[u8]) -> BuiltEnvironmentCase {
        BuiltEnvironmentCase {
            case_id: "case-byte-verifier-001".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence: vec![EvidenceRecord {
                evidence_id: "evidence-001".into(),
                subject_id: "design-001".into(),
                field_path: "geometry.revision".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: Some(sha(bytes)),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_000),
            }],
            artifacts: Vec::new(),
            engineering_receipts: Vec::new(),
            decisions: Vec::new(),
        }
    }

    fn descriptor(bytes: &[u8]) -> SourceRecordDescriptor {
        let digest = sha(bytes);
        SourceRecordDescriptor {
            record_kind: SourceRecordKind::Evidence,
            case_record_id: "evidence-001".into(),
            owner_namespace: "urn:synthetic:survey-owner".into(),
            native_identity_components: vec![
                NativeIdentityComponent {
                    name: "native_id".into(),
                    value: "survey-registry".into(),
                },
                NativeIdentityComponent {
                    name: "native_record_id".into(),
                    value: "evidence-001".into(),
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
                sha256: Some(digest.clone()),
                digest_basis: PayloadDigestBasis::LocallyRetrievedAndHashed,
                bytes_retrieved: true,
                byte_length: Some(bytes.len() as u64),
                payload_object_ref: Some(cas_reference(&digest)),
                media_type: Some("application/json".into()),
                schema_id: Some("urn:synthetic:survey-evidence".into()),
                schema_version: Some("1.0".into()),
                schema_sha256: Some("e".repeat(64)),
            },
            source_snapshot: SourceSnapshotDescriptor {
                source_id: "urn:synthetic:survey-owner".into(),
                snapshot_id: "snapshot-001".into(),
                snapshot_sha256: "f".repeat(64),
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

    fn catalog(
        case: &BuiltEnvironmentCase,
        descriptor: SourceRecordDescriptor,
    ) -> SourceDescriptorCatalog {
        SourceDescriptorCatalog {
            schema_version: "luminous.built-environment-source-descriptor-catalog.v0.1".into(),
            catalog_id: "catalog-byte-verifier-001".into(),
            case_id: case.case_id.clone(),
            case_representation_sha256: sha(&serde_json::to_vec(case).unwrap()),
            captured_at: "2026-11-03T10:02:00Z".into(),
            descriptors: vec![descriptor],
            claim_ceiling: "descriptor_integrity_only".into(),
            action_authority: "none".into(),
            limitations: vec!["Synthetic fixture; no source trust claim.".into()],
        }
    }

    fn context() -> PayloadByteVerificationContext {
        PayloadByteVerificationContext {
            verified_at_utc: "2026-11-04T05:45:00Z".into(),
            max_bytes_per_payload: 1024 * 1024,
            verifier_build_sha256: Some("d".repeat(64)),
        }
    }

    #[test]
    fn sha256_known_vector_matches_independent_standard_value() {
        assert_eq!(
            sha(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    struct ShortReader {
        bytes: Cursor<Vec<u8>>,
    }

    impl Read for ShortReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let cap = output.len().min(1);
            self.bytes.read(&mut output[..cap])
        }
    }

    #[test]
    fn partial_reads_do_not_change_streaming_hash() {
        let bytes = b"short read source payload";
        let descriptor = descriptor(bytes);
        let result = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            ShortReader {
                bytes: Cursor::new(bytes.to_vec()),
            },
            1024,
        );
        assert_eq!(result.status, PayloadByteStatus::Verified);
        assert_eq!(result.actual_sha256.as_deref(), Some(sha(bytes).as_str()));
    }

    struct InterruptedOnce {
        interrupted: bool,
        bytes: Cursor<Vec<u8>>,
    }

    impl Read for InterruptedOnce {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            self.bytes.read(output)
        }
    }

    #[test]
    fn interrupted_read_is_retried_without_losing_bytes() {
        let bytes = b"interrupted source stream";
        let descriptor = descriptor(bytes);
        let result = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            InterruptedOnce {
                interrupted: false,
                bytes: Cursor::new(bytes.to_vec()),
            },
            1024,
        );
        assert_eq!(result.status, PayloadByteStatus::Verified);
        assert_eq!(result.actual_byte_length, Some(bytes.len() as u64));
    }

    #[test]
    fn exact_payload_stream_matches_expected_sha256_and_length() {
        let bytes = b"engineering source bytes";
        let descriptor = descriptor(bytes);
        let result = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(bytes.to_vec()),
            1024,
        );
        assert_eq!(result.status, PayloadByteStatus::Verified);
        assert_eq!(result.actual_sha256.as_deref(), Some(sha(bytes).as_str()));
        assert_eq!(result.actual_byte_length, Some(bytes.len() as u64));
        assert_eq!(result.bytes_read, bytes.len() as u64);
    }

    #[test]
    fn one_byte_mutation_is_detected_even_when_object_reference_is_unchanged() {
        let descriptor = descriptor(b"original bytes");
        let result = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(b"original byteS".to_vec()),
            1024,
        );
        assert_eq!(result.status, PayloadByteStatus::DigestMismatch);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "SOURCE_PAYLOAD_SHA256_MISMATCH" })
        );
    }

    #[test]
    fn truncation_and_extension_are_detected_by_digest_and_length() {
        let descriptor = descriptor(b"complete source bytes");
        let truncated = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(b"complete source".to_vec()),
            1024,
        );
        assert_eq!(truncated.status, PayloadByteStatus::DigestMismatch);
        assert!(
            truncated
                .findings
                .iter()
                .any(|finding| { finding.code == "SOURCE_PAYLOAD_BYTE_LENGTH_MISMATCH" })
        );

        let extended = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(b"complete source bytes!".to_vec()),
            1024,
        );
        assert_eq!(extended.status, PayloadByteStatus::DigestMismatch);
        assert!(
            extended
                .findings
                .iter()
                .any(|finding| { finding.code == "SOURCE_PAYLOAD_BYTE_LENGTH_MISMATCH" })
        );
    }

    #[test]
    fn exact_limit_is_allowed_but_one_byte_over_limit_is_incomplete() {
        let bytes = b"four";
        let descriptor = descriptor(bytes);
        let at_limit = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(bytes.to_vec()),
            4,
        );
        assert_eq!(at_limit.status, PayloadByteStatus::Verified);

        let over_limit = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(b"fours".to_vec()),
            4,
        );
        assert_eq!(over_limit.status, PayloadByteStatus::ByteLimitExceeded);
        assert_eq!(over_limit.bytes_read, 5);
    }

    struct BrokenReader;
    impl Read for BrokenReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic denial",
            ))
        }
    }

    #[test]
    fn read_error_is_incomplete_and_never_claims_a_digest() {
        let descriptor = descriptor(b"payload");
        let result = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            BrokenReader,
            1024,
        );
        assert_eq!(result.status, PayloadByteStatus::ReadFailed);
        assert!(result.actual_sha256.is_none());
        assert_eq!(result.read_error_kind.as_deref(), Some("PermissionDenied"));
    }

    #[test]
    fn empty_payload_is_verified_without_special_casing() {
        let descriptor = descriptor(b"");
        let result = verify_descriptor_payload_stream(
            &descriptor,
            "evidence:evidence-001",
            Cursor::new(Vec::<u8>::new()),
            0,
        );
        assert_eq!(result.status, PayloadByteStatus::Verified);
        assert_eq!(result.actual_byte_length, Some(0));
    }

    #[test]
    fn catalog_verifier_rehashes_actual_resolved_bytes_and_binds_report() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let descriptor = descriptor(bytes);
        let catalog = catalog(&case, descriptor.clone());
        let expected_ref = descriptor
            .source_payload
            .payload_object_ref
            .clone()
            .unwrap();
        let result = verify_catalog_payload_bytes(&case, &catalog, &context(), move |reference| {
            if reference == expected_ref {
                Ok(Cursor::new(bytes.to_vec()))
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "wrong ref"))
            }
        });
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::AllResolvablePayloadsVerified
        );
        assert_eq!(result.verified_count, 1);
        assert_eq!(result.invalid_count, 0);
        assert_eq!(result.records[0].status, PayloadByteStatus::Verified);
        assert_eq!(result.source_owner_authentication, "not_performed");
        assert_eq!(result.rights_authentication, "not_performed");
        assert_eq!(result.action_authority, "none");
        assert_eq!(result.no_impact_eligibility, "none");
        assert!(valid_sha256(&result.case_representation_sha256));
        assert!(valid_sha256(&result.catalog_representation_sha256));
    }

    #[test]
    fn resolved_object_bytes_are_rehashed_not_trusted_from_the_reference_name() {
        let declared_bytes = b"declared source payload";
        let substituted_bytes = b"different source payload";
        let case = test_case(declared_bytes);
        let catalog = catalog(&case, descriptor(declared_bytes));
        let result = verify_catalog_payload_bytes(&case, &catalog, &context(), |_reference| {
            Ok(Cursor::new(substituted_bytes.to_vec()))
        });
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::InvalidInput
        );
        assert_eq!(result.invalid_count, 1);
        assert_eq!(result.records[0].status, PayloadByteStatus::DigestMismatch);
        assert_ne!(
            result.records[0].actual_sha256,
            result.records[0].expected_sha256
        );
    }

    #[test]
    fn resolver_open_failure_is_incomplete_not_digest_mismatch_or_success() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let catalog = catalog(&case, descriptor(bytes));
        let result = verify_catalog_payload_bytes(&case, &catalog, &context(), |_reference| {
            Err::<Cursor<Vec<u8>>, _>(io::Error::new(
                io::ErrorKind::NotFound,
                "missing",
            ))
        });
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(result.unavailable_count, 1);
        assert_eq!(
            result.records[0].status,
            PayloadByteStatus::SourceBytesUnavailable
        );
    }

    #[test]
    fn invalid_descriptor_catalog_is_rejected_before_resolver_is_called() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let mut catalog = catalog(&case, descriptor(bytes));
        catalog.case_representation_sha256 = "0".repeat(64);
        let mut resolver_called = false;
        let result = verify_catalog_payload_bytes(&case, &catalog, &context(), |_reference| {
            resolver_called = true;
            Ok(Cursor::new(bytes.to_vec()))
        });
        assert!(!resolver_called);
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::InvalidInput
        );
        assert_eq!(result.records.len(), 0);
        assert_eq!(result.invalid_count, 0);
    }

    #[test]
    fn invalid_verification_time_is_invalid_input_not_incomplete_success() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let catalog = catalog(&case, descriptor(bytes));
        let mut context = context();
        context.verified_at_utc = "2026-02-31T25:61:61Z".into();
        let result = verify_catalog_payload_bytes(&case, &catalog, &context, |_reference| {
            Ok(Cursor::new(bytes.to_vec()))
        });
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::InvalidInput
        );
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVALID_VERIFICATION_TIME" })
        );
    }

    #[test]
    fn invalid_verifier_build_digest_prevents_payload_resolution() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let catalog = catalog(&case, descriptor(bytes));
        let mut context = context();
        context.verifier_build_sha256 = Some("not-a-digest".into());
        let mut resolver_called = false;
        let result = verify_catalog_payload_bytes(&case, &catalog, &context, |_reference| {
            resolver_called = true;
            Ok(Cursor::new(bytes.to_vec()))
        });
        assert!(!resolver_called);
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::InvalidInput
        );
        assert_eq!(result.records.len(), 0);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVALID_VERIFIER_BUILD_DIGEST" })
        );
    }

    #[test]
    fn descriptor_validation_findings_are_preserved_in_byte_report() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let mut catalog = catalog(&case, descriptor(bytes));
        catalog.descriptors[0].rights.state = SourceRightsState::Unknown;
        catalog.descriptors[0].rights.permitted_uses.clear();
        catalog.descriptors[0].rights.rights_evidence_ref = None;
        let result = verify_catalog_payload_bytes(&case, &catalog, &context(), |_reference| {
            Ok(Cursor::new(bytes.to_vec()))
        });
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
        assert!(
            result
                .descriptor_validation_findings
                .iter()
                .any(|finding| { finding.code == "SOURCE_RIGHTS_UNKNOWN" })
        );
        assert_eq!(result.no_impact_eligibility, "none");
    }

    #[test]
    fn missing_build_digest_downgrades_report_without_changing_per_record_hash() {
        let bytes = b"actual source payload";
        let case = test_case(bytes);
        let catalog = catalog(&case, descriptor(bytes));
        let mut context = context();
        context.verifier_build_sha256 = None;
        let result = verify_catalog_payload_bytes(&case, &catalog, &context, |_reference| {
            Ok(Cursor::new(bytes.to_vec()))
        });
        assert_eq!(result.verified_count, 1);
        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "VERIFIER_BUILD_IDENTITY_UNAVAILABLE" })
        );
    }
}
