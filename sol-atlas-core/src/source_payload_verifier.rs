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
    SkippedBudget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadByteCatalogDisposition {
    AllDeclaredPayloadReferencesVerified,
    IncompleteOrIndeterminate,
    InvalidInput,
}

impl PayloadByteCatalogDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllDeclaredPayloadReferencesVerified => {
                "all_declared_payload_references_verified"
            }
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
    pub descriptor_missing_count: usize,
    pub descriptor_extra_count: usize,
    pub verified_count: usize,
    pub unavailable_count: usize,
    pub read_failed_count: usize,
    pub byte_limit_exceeded_count: usize,
    pub skipped_budget_count: usize,
    pub invalid_count: usize,
    pub invalid_finding_count: usize,
    pub incomplete_finding_count: usize,
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

struct PayloadByteReportContext<'a> {
    case_id: &'a str,
    catalog: &'a SourceDescriptorCatalog,
    verification_context: &'a PayloadByteVerificationContext,
    case_sha256: String,
    catalog_sha256: String,
    descriptor_validation: &'a DescriptorValidation,
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
    if let Some(expected_length) = payload.byte_length
        && expected_length != bytes_read
    {
        mismatch = true;
        findings.push(finding(
            "invalid",
            "SOURCE_PAYLOAD_BYTE_LENGTH_MISMATCH",
            record_key,
            "recomputed payload byte length differs from the descriptor's expected length",
        ));
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
            PayloadByteReportContext {
                case_id: &case.case_id,
                catalog,
                verification_context: context,
                case_sha256,
                catalog_sha256,
                descriptor_validation: &descriptor_validation,
            },
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
            PayloadByteReportContext {
                case_id: &case.case_id,
                catalog,
                verification_context: context,
                case_sha256,
                catalog_sha256,
                descriptor_validation: &descriptor_validation,
            },
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
        PayloadByteReportContext {
            case_id: &case.case_id,
            catalog,
            verification_context: context,
            case_sha256,
            catalog_sha256,
            descriptor_validation: &descriptor_validation,
        },
        records,
        report_findings,
    )
}

fn catalog_report(
    binding: PayloadByteReportContext<'_>,
    records: Vec<PayloadByteRecordResult>,
    findings: BTreeSet<PayloadByteFinding>,
) -> PayloadByteCatalogReport {
    let PayloadByteReportContext {
        case_id,
        catalog,
        verification_context,
        case_sha256,
        catalog_sha256,
        descriptor_validation,
    } = binding;
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
    let skipped_budget_count = records
        .iter()
        .filter(|record| record.status == PayloadByteStatus::SkippedBudget)
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
    let invalid_finding_count = findings
        .iter()
        .filter(|finding| finding.severity == "invalid")
        .count();
    let incomplete_finding_count = findings
        .iter()
        .filter(|finding| finding.severity == "incomplete")
        .count();

    let invalid_descriptor =
        descriptor_validation.disposition == DescriptorDisposition::InvalidInput;
    let invalid_report_finding = findings.iter().any(|finding| finding.severity == "invalid");
    let disposition = if invalid_descriptor || invalid_count > 0 || invalid_report_finding {
        PayloadByteCatalogDisposition::InvalidInput
    } else if verified_count + unavailable_count + read_failed_count + byte_limit_exceeded_count
        != records.len()
        || unavailable_count > 0
        || read_failed_count > 0
        || byte_limit_exceeded_count > 0
        || skipped_budget_count > 0
        || descriptor_validation.disposition
            == DescriptorDisposition::IncompleteOrIndeterminate
        || findings
            .iter()
            .any(|finding| finding.code == "VERIFIER_BUILD_IDENTITY_UNAVAILABLE")
    {
        PayloadByteCatalogDisposition::IncompleteOrIndeterminate
    } else {
        PayloadByteCatalogDisposition::AllDeclaredPayloadReferencesVerified
    };

    PayloadByteCatalogReport {
        schema_version: VERIFIER_SCHEMA.into(),
        disposition,
        case_id: case_id.to_owned(),
        case_representation_sha256: case_sha256,
        catalog_id: catalog.catalog_id.clone(),
        catalog_representation_sha256: catalog_sha256,
        verifier_build_sha256: verification_context.verifier_build_sha256.clone(),
        verified_at_utc: verification_context.verified_at_utc.clone(),
        max_bytes_per_payload: verification_context.max_bytes_per_payload,
        descriptor_validation_disposition: descriptor_validation.disposition.as_str().into(),
        descriptor_validation_findings: descriptor_validation.findings.clone(),
        descriptor_count: catalog.descriptors.len(),
        descriptor_missing_count: descriptor_validation.missing_descriptor_count,
        descriptor_extra_count: descriptor_validation.extra_descriptor_count,
        verified_count,
        unavailable_count,
        read_failed_count,
        byte_limit_exceeded_count,
        skipped_budget_count,
        invalid_count,
        invalid_finding_count,
        incomplete_finding_count,
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
    }
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


/// Deterministic aggregate resource policy for one source-payload catalogue.
/// The verifier processes records sequentially in canonical record-key order;
/// no parallel resolver work is performed in v0.1.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CatalogResourceBudget {
    pub max_descriptors: usize,
    /// Maximum payload bytes incorporated into incremental SHA-256 streams.
    pub max_total_payload_bytes: u64,
    /// Maximum bytes returned by all resolver streams, including over-limit
    /// one-byte probes used to distinguish exact boundaries from extensions.
    pub max_total_bytes_read: u64,
    pub max_bytes_per_payload: u64,
    pub max_resolver_open_attempts: usize,
    /// Retries are disabled in v0.1. This is an upper bound, not a target.
    pub max_retries_per_payload: usize,
    /// Maximum serialized size of this report, including truncation metadata.
    pub max_report_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetedPayloadByteCatalogReport {
    pub schema_version: String,
    pub report: PayloadByteCatalogReport,
    pub budget: CatalogResourceBudget,
    pub descriptor_count: usize,
    pub evaluated_descriptor_count: usize,
    pub skipped_budget_count: usize,
    pub total_payload_bytes_hashed: u64,
    pub total_bytes_read: u64,
    pub resolver_open_attempts: usize,
    pub resolver_retries: usize,
    pub max_concurrency_used: usize,
    pub report_truncated: bool,
    pub omitted_record_count: usize,
    pub omitted_records_sha256: Option<String>,
    pub omitted_finding_count: usize,
    pub omitted_findings_sha256: Option<String>,
    pub omitted_descriptor_validation_finding_count: usize,
    pub omitted_descriptor_validation_findings_sha256: Option<String>,
    pub serialized_report_bytes: usize,
}

const BUDGET_REPORT_SCHEMA: &str = "luminous.source-payload-catalog-budget-report.v0.1";
const MIN_BUDGET_REPORT_BYTES: usize = 8 * 1024;

fn skipped_budget_result(
    descriptor: &SourceRecordDescriptor,
    record_key: &str,
    code: &str,
    detail: &str,
) -> PayloadByteRecordResult {
    PayloadByteRecordResult {
        record_key: record_key.to_owned(),
        status: PayloadByteStatus::SkippedBudget,
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

fn digest_serialized<T: Serialize>(hasher: &mut Sha256, value: &T) {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn finalize_optional_digest(count: usize, hasher: Sha256) -> Option<String> {
    (count > 0).then(|| hex_digest(&hasher.finalize()))
}

/// Verify a catalogue under explicit aggregate resource limits.
///
/// Records are evaluated sequentially in canonical record-key order. The
/// maximum read budget includes the one-byte per-object probe used by the
/// stream verifier; hashed payload bytes are counted separately. A skipped
/// record is never reported as verified.
pub fn verify_catalog_payload_bytes_with_budget<R, F>(
    case: &BuiltEnvironmentCase,
    catalog: &SourceDescriptorCatalog,
    context: &PayloadByteVerificationContext,
    budget: &CatalogResourceBudget,
    mut open_payload: F,
) -> BudgetedPayloadByteCatalogReport
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

    let invalid_budget = budget.max_descriptors == 0
        || budget.max_resolver_open_attempts == 0
        || budget.max_report_bytes < MIN_BUDGET_REPORT_BYTES
        || budget.max_retries_per_payload > 0;
    if invalid_budget {
        report_findings.insert(finding(
            "invalid",
            "INVALID_CATALOG_RESOURCE_BUDGET",
            "budget",
            format!(
                "v0.1 requires max_descriptors>0, max_resolver_open_attempts>0, max_report_bytes>={MIN_BUDGET_REPORT_BYTES}, and retries=0"
            ),
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

    let mut records = Vec::with_capacity(catalog.descriptors.len());
    let mut total_payload_bytes_hashed = 0_u64;
    let mut total_bytes_read = 0_u64;
    let mut resolver_open_attempts = 0_usize;
    let mut skipped_budget_count = 0_usize;

    let invalid_context = invalid_budget
        || !valid_utc_timestamp(&context.verified_at_utc)
        || context
            .verifier_build_sha256
            .as_deref()
            .is_some_and(|digest| !valid_sha256(digest));

    if descriptor_validation.disposition == DescriptorDisposition::InvalidInput
        || invalid_context
    {
        if descriptor_validation.disposition == DescriptorDisposition::InvalidInput {
            report_findings.insert(finding(
                "invalid",
                "SOURCE_DESCRIPTOR_CATALOG_INVALID",
                "catalog",
                "invalid descriptor catalogue was rejected before any payload object was opened",
            ));
        }
    } else {
        let mut ordered = catalog.descriptors.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|descriptor| {
            descriptor.record_kind.case_key(&descriptor.case_record_id)
        });

        for (index, descriptor) in ordered.iter().enumerate() {
            let record_key = descriptor.record_kind.case_key(&descriptor.case_record_id);
            let skip = if index >= budget.max_descriptors {
                Some((
                    "CATALOG_DESCRIPTOR_BUDGET_EXCEEDED",
                    "descriptor was not evaluated because max_descriptors was exhausted",
                ))
            } else if resolver_open_attempts >= budget.max_resolver_open_attempts {
                Some((
                    "CATALOG_RESOLVER_OPEN_BUDGET_EXCEEDED",
                    "descriptor was not opened because max_resolver_open_attempts was exhausted",
                ))
            } else if total_bytes_read >= budget.max_total_bytes_read {
                Some((
                    "CATALOG_TOTAL_READ_BUDGET_EXCEEDED",
                    "descriptor was not opened because max_total_bytes_read was exhausted",
                ))
            } else {
                None
            };

            if let Some((code, detail)) = skip {
                let result = skipped_budget_result(descriptor, &record_key, code, detail);
                report_findings.extend(result.findings.iter().cloned());
                records.push(result);
                skipped_budget_count += 1;
                continue;
            }

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

            // Reserve one byte from the aggregate read budget for the stream
            // verifier's EOF/extension probe. It never reads more than the
            // remaining aggregate read budget.
            let remaining_read = budget.max_total_bytes_read - total_bytes_read;
            let read_probe_reserve = remaining_read.saturating_sub(1);
            let remaining_payload =
                budget.max_total_payload_bytes.saturating_sub(total_payload_bytes_hashed);
            let per_payload_cap = budget
                .max_bytes_per_payload
                .min(remaining_payload)
                .min(read_probe_reserve);

            resolver_open_attempts += 1;
            match open_payload(object_ref) {
                Ok(reader) => {
                    let mut result = verify_descriptor_payload_stream(
                        descriptor,
                        &record_key,
                        reader,
                        per_payload_cap,
                    );
                    let bytes_read = result.bytes_read;
                    let bytes_hashed = if result.status == PayloadByteStatus::ByteLimitExceeded {
                        bytes_read.saturating_sub(1)
                    } else {
                        bytes_read
                    };
                    total_bytes_read = total_bytes_read.saturating_add(bytes_read);
                    total_payload_bytes_hashed =
                        total_payload_bytes_hashed.saturating_add(bytes_hashed);

                    if result.status == PayloadByteStatus::ByteLimitExceeded {
                        let code = if per_payload_cap < budget.max_bytes_per_payload {
                            if remaining_payload <= budget.max_bytes_per_payload {
                                "CATALOG_TOTAL_PAYLOAD_BUDGET_EXCEEDED"
                            } else {
                                "CATALOG_TOTAL_READ_BUDGET_EXCEEDED"
                            }
                        } else {
                            "PAYLOAD_PER_OBJECT_BUDGET_EXCEEDED"
                        };
                        let budget_finding = finding(
                            "incomplete",
                            code,
                            &record_key,
                            "stream stopped at a configured aggregate or per-payload byte boundary",
                        );
                        result.findings.push(budget_finding.clone());
                        report_findings.insert(budget_finding);
                    }
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
    }

    records.sort_by(|left, right| left.record_key.cmp(&right.record_key));
    let base_report = catalog_report(
        PayloadByteReportContext {
            case_id: &case.case_id,
            catalog,
            verification_context: context,
            case_sha256,
            catalog_sha256,
            descriptor_validation: &descriptor_validation,
        },
        records,
        report_findings,
    );

    let mut output = BudgetedPayloadByteCatalogReport {
        schema_version: BUDGET_REPORT_SCHEMA.into(),
        report: base_report,
        budget: budget.clone(),
        descriptor_count: catalog.descriptors.len(),
        evaluated_descriptor_count: 0,
        skipped_budget_count,
        total_payload_bytes_hashed,
        total_bytes_read,
        resolver_open_attempts,
        resolver_retries: 0,
        max_concurrency_used: 1,
        report_truncated: false,
        omitted_record_count: 0,
        omitted_records_sha256: None,
        omitted_finding_count: 0,
        omitted_findings_sha256: None,
        omitted_descriptor_validation_finding_count: 0,
        omitted_descriptor_validation_findings_sha256: None,
        serialized_report_bytes: 0,
    };
    output.evaluated_descriptor_count =
        output.report.records.len().saturating_sub(output.skipped_budget_count);

    let mut omitted_records_hasher = Sha256::new();
    let mut omitted_findings_hasher = Sha256::new();
    let mut omitted_descriptor_findings_hasher = Sha256::new();
    let mut truncation_finding_added = false;

    loop {
        let serialized = serde_json::to_vec(&output).unwrap_or_default();
        if serialized.len().saturating_add(16) <= budget.max_report_bytes {
            output.serialized_report_bytes = serialized.len();
            break;
        }
        output.report_truncated = true;
        if !truncation_finding_added {
            truncation_finding_added = true;
            if output.report.disposition != PayloadByteCatalogDisposition::InvalidInput {
                output.report.disposition =
                    PayloadByteCatalogDisposition::IncompleteOrIndeterminate;
            }
            output.report.incomplete_finding_count += 1;
            output.report.findings.push(finding(
                "incomplete",
                "REPORT_TRUNCATED_BY_RESOURCE_BUDGET",
                "report",
                "deterministic report sections were truncated to respect max_report_bytes",
            ));
        } else if let Some(record) = output.report.records.pop() {
            digest_serialized(&mut omitted_records_hasher, &record);
            output.omitted_record_count += 1;
            output.omitted_records_sha256 = Some("0".repeat(64));
        } else if let Some(finding) = output.report.findings.pop() {
            digest_serialized(&mut omitted_findings_hasher, &finding);
            output.omitted_finding_count += 1;
            output.omitted_findings_sha256 = Some("0".repeat(64));
        } else if let Some(finding) = output.report.descriptor_validation_findings.pop() {
            digest_serialized(&mut omitted_descriptor_findings_hasher, &finding);
            output.omitted_descriptor_validation_finding_count += 1;
            output.omitted_descriptor_validation_findings_sha256 = Some("0".repeat(64));
        } else {
            break;
        }
    }

    if output.omitted_record_count > 0 {
        output.omitted_records_sha256 = finalize_optional_digest(
            output.omitted_record_count,
            omitted_records_hasher,
        );
    }
    if output.omitted_finding_count > 0 {
        output.omitted_findings_sha256 = finalize_optional_digest(
            output.omitted_finding_count,
            omitted_findings_hasher,
        );
    }
    if output.omitted_descriptor_validation_finding_count > 0 {
        output.omitted_descriptor_validation_findings_sha256 = finalize_optional_digest(
            output.omitted_descriptor_validation_finding_count,
            omitted_descriptor_findings_hasher,
        );
    }

    let mut serialized_len = serde_json::to_vec(&output).unwrap_or_default().len();
    if output.serialized_report_bytes != serialized_len {
        output.serialized_report_bytes = serialized_len;
        serialized_len = serde_json::to_vec(&output).unwrap_or_default().len();
        output.serialized_report_bytes = serialized_len;
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
            PayloadByteCatalogDisposition::AllDeclaredPayloadReferencesVerified
        );
        assert_eq!(result.verified_count, 1);
        assert_eq!(result.invalid_count, 0);
        assert_eq!(result.invalid_finding_count, 0);
        assert_eq!(result.incomplete_finding_count, 0);
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
            Err::<Cursor<Vec<u8>>, _>(io::Error::new(io::ErrorKind::NotFound, "missing"))
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
        assert!(result.invalid_finding_count > 0);
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
        assert!(result.invalid_finding_count > 0);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVALID_VERIFIER_BUILD_DIGEST" })
        );
    }

    #[test]
    fn byte_pass_does_not_upgrade_case_with_missing_source_descriptor() {
        let bytes = b"declared evidence bytes";
        let mut case = test_case(bytes);
        case.evidence.push(EvidenceRecord {
            evidence_id: "evidence-002".into(),
            subject_id: "design-002".into(),
            field_path: "material.lot".into(),
            source_id: "supplier-registry".into(),
            source_payload_sha256: Some("f".repeat(64)),
            evidence_state: EvidenceState::Verified,
            data_rights: DataRights::PublicDomain,
            license_id: Some("CC0-1.0".into()),
            jurisdiction: "synthetic-jurisdiction".into(),
            observed_at_unix_seconds: Some(1_762_000_001),
        });
        let catalog = catalog(&case, descriptor(bytes));
        let result = verify_catalog_payload_bytes(&case, &catalog, &context(), |_reference| {
            Ok(Cursor::new(bytes.to_vec()))
        });

        assert_eq!(
            result.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(result.verified_count, 1);
        assert_eq!(result.descriptor_count, 1);
        assert_eq!(result.descriptor_missing_count, 1);
        assert_eq!(
            result.descriptor_validation_disposition,
            "incomplete_or_indeterminate"
        );
        assert_eq!(result.no_impact_eligibility, "none");
        assert_eq!(result.action_authority, "none");
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
        assert_eq!(
            result.descriptor_validation_disposition, "incomplete_or_indeterminate",
            "byte equality must not silently upgrade descriptor rights"
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

    fn resource_budget() -> CatalogResourceBudget {
        CatalogResourceBudget {
            max_descriptors: 100,
            max_total_payload_bytes: 1024 * 1024,
            max_total_bytes_read: 1024 * 1024 + 100,
            max_bytes_per_payload: 1024 * 1024,
            max_resolver_open_attempts: 100,
            max_retries_per_payload: 0,
            max_report_bytes: MIN_BUDGET_REPORT_BYTES,
        }
    }

    fn multi_case_and_catalog(payloads: &[&[u8]]) -> (BuiltEnvironmentCase, SourceDescriptorCatalog) {
        let mut case = test_case(payloads[0]);
        case.evidence.clear();
        let mut descriptors = Vec::new();
        for (index, bytes) in payloads.iter().enumerate() {
            let id = format!("evidence-{:03}", index + 1);
            let subject = format!("design-{:03}", index + 1);
            case.evidence.push(EvidenceRecord {
                evidence_id: id.clone(),
                subject_id: subject.clone(),
                field_path: "geometry.revision".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: Some(sha(bytes)),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_000 + index as i64),
            });
            let mut row = descriptor(bytes);
            row.case_record_id = id.clone();
            for component in &mut row.native_identity_components {
                match component.name.as_str() {
                    "native_record_id" => component.value = id.clone(),
                    "subject_id" => component.value = subject.clone(),
                    _ => {}
                }
            }
            row.source_payload.payload_object_ref =
                Some(cas_reference(row.source_payload.sha256.as_deref().unwrap()));
            descriptors.push(row);
        }
        let catalog = SourceDescriptorCatalog {
            schema_version: "luminous.built-environment-source-descriptor-catalog.v0.1".into(),
            catalog_id: "catalog-budget-synthetic-001".into(),
            case_id: case.case_id.clone(),
            case_representation_sha256: sha(&serde_json::to_vec(&case).unwrap()),
            captured_at: "2026-11-04T06:00:00Z".into(),
            descriptors,
            claim_ceiling: "descriptor_integrity_only".into(),
            action_authority: "none".into(),
            limitations: vec!["Synthetic budget fixture only.".into()],
        };
        (case, catalog)
    }

    #[test]
    fn aggregate_read_budget_is_a_hard_cap_including_probe_bytes() {
        let (case, catalog) = multi_case_and_catalog(&[b"four", b"fives"]);
        let mut budget = resource_budget();
        budget.max_total_payload_bytes = 9;
        budget.max_total_bytes_read = 5;
        let expected = BTreeSet::from([
            cas_reference(&sha(b"four")),
            cas_reference(&sha(b"fives")),
        ]);
        let result = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &budget,
            move |reference| {
                if expected.contains(reference) && reference == cas_reference(&sha(b"four")) {
                    Ok(Cursor::new(b"four".to_vec()))
                } else {
                    Ok(Cursor::new(b"fives".to_vec()))
                }
            },
        );
        assert!(result.total_bytes_read <= budget.max_total_bytes_read);
        assert_eq!(result.total_bytes_read, 5);
        assert_eq!(result.total_payload_bytes_hashed, 4);
        assert_eq!(result.report.records[0].status, PayloadByteStatus::Verified);
        assert_eq!(result.report.records[1].status, PayloadByteStatus::ByteLimitExceeded);
        assert_eq!(
            result.report.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
    }

    #[test]
    fn descriptor_and_open_attempt_budgets_skip_records_without_resolving_them() {
        let (case, catalog) = multi_case_and_catalog(&[b"one", b"two", b"three"]);
        let mut budget = resource_budget();
        budget.max_descriptors = 3;
        budget.max_resolver_open_attempts = 1;
        let mut opens = Vec::new();
        let result = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &budget,
            |reference| {
                opens.push(reference.to_owned());
                let bytes = match reference {
                    value if value == cas_reference(&sha(b"one")) => b"one".to_vec(),
                    value if value == cas_reference(&sha(b"two")) => b"two".to_vec(),
                    _ => b"three".to_vec(),
                };
                Ok(Cursor::new(bytes))
            },
        );
        assert_eq!(opens.len(), 1);
        assert_eq!(result.resolver_open_attempts, 1);
        assert_eq!(result.skipped_budget_count, 2);
        assert_eq!(
            result.report.records.iter().filter(|row| row.status == PayloadByteStatus::SkippedBudget).count(),
            2
        );
        assert_eq!(result.report.records[0].status, PayloadByteStatus::Verified);
        assert!(result.report.records[1..]
            .iter()
            .all(|row| row.status == PayloadByteStatus::SkippedBudget));
        assert_eq!(
            result.report.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
    }

    #[test]
    fn descriptor_count_budget_is_deterministic_under_input_vector_reordering() {
        let (case, mut catalog) = multi_case_and_catalog(&[b"one", b"two", b"three"]);
        let mut budget = resource_budget();
        budget.max_descriptors = 2;
        let mut first_order = Vec::new();
        let first = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &budget,
            |reference| {
                first_order.push(reference.to_owned());
                let bytes = if reference == cas_reference(&sha(b"one")) {
                    b"one".to_vec()
                } else {
                    b"two".to_vec()
                };
                Ok(Cursor::new(bytes))
            },
        );
        catalog.descriptors.reverse();
        let mut second_order = Vec::new();
        let second = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &budget,
            |reference| {
                second_order.push(reference.to_owned());
                let bytes = if reference == cas_reference(&sha(b"one")) {
                    b"one".to_vec()
                } else {
                    b"two".to_vec()
                };
                Ok(Cursor::new(bytes))
            },
        );
        assert_eq!(first_order, second_order);
        assert_eq!(
            first.report.records.iter().map(|row| (&row.record_key, &row.status)).collect::<Vec<_>>(),
            second.report.records.iter().map(|row| (&row.record_key, &row.status)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn report_budget_truncates_deterministically_and_preserves_denominators() {
        let payloads = vec![b"same synthetic payload".as_slice(); 100];
        let (case, catalog) = multi_case_and_catalog(&payloads);
        let mut budget = resource_budget();
        budget.max_descriptors = 100;
        budget.max_resolver_open_attempts = 100;
        budget.max_total_payload_bytes = 100_000;
        budget.max_total_bytes_read = 100_100;
        budget.max_report_bytes = MIN_BUDGET_REPORT_BYTES;
        let expected_ref = cas_reference(&sha(b"same synthetic payload"));
        let result = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &budget,
            move |reference| {
                if reference == expected_ref {
                    Ok(Cursor::new(b"same synthetic payload".to_vec()))
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "unknown object"))
                }
            },
        );
        assert!(result.report_truncated);
        assert!(result.omitted_record_count > 0 || result.omitted_finding_count > 0);
        assert!(result.serialized_report_bytes <= budget.max_report_bytes);
        assert_eq!(result.report.descriptor_count, 100);
        assert_eq!(result.descriptor_count, 100);
        assert_eq!(
            result.omitted_records_sha256.is_some(),
            result.omitted_record_count > 0
        );
        assert_eq!(
            result.report.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
    }

    #[test]
    fn invalid_aggregate_budget_never_opens_a_payload() {
        let bytes = b"budget invalid";
        let case = test_case(bytes);
        let catalog = catalog(&case, descriptor(bytes));
        let mut budget = resource_budget();
        budget.max_retries_per_payload = 1;
        let mut opened = false;
        let result = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &budget,
            |_reference| {
                opened = true;
                Ok(Cursor::new(bytes.to_vec()))
            },
        );
        assert!(!opened);
        assert_eq!(
            result.report.disposition,
            PayloadByteCatalogDisposition::InvalidInput
        );
        assert!(result.report.findings.iter().any(|row| row.code == "INVALID_CATALOG_RESOURCE_BUDGET"));
        assert_eq!(result.resolver_open_attempts, 0);
    }

    #[test]
    fn descriptor_incompleteness_cannot_be_upgraded_by_byte_verification() {
        let bytes = b"descriptor incomplete";
        let mut case = test_case(bytes);
        case.evidence.push(EvidenceRecord {
            evidence_id: "evidence-missing".into(),
            subject_id: "design-missing".into(),
            field_path: "material.lot".into(),
            source_id: "supplier-registry".into(),
            source_payload_sha256: Some("f".repeat(64)),
            evidence_state: EvidenceState::Verified,
            data_rights: DataRights::PublicDomain,
            license_id: Some("CC0-1.0".into()),
            jurisdiction: "synthetic-jurisdiction".into(),
            observed_at_unix_seconds: Some(1_762_000_001),
        });
        let catalog = catalog(&case, descriptor(bytes));
        let result = verify_catalog_payload_bytes_with_budget(
            &case,
            &catalog,
            &context(),
            &resource_budget(),
            |_reference| Ok(Cursor::new(bytes.to_vec())),
        );
        assert_eq!(
            result.report.disposition,
            PayloadByteCatalogDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(result.report.descriptor_validation_disposition, "incomplete_or_indeterminate");
        assert_eq!(result.report.verified_count, 1);
        assert_eq!(result.report.descriptor_missing_count, 1);
        assert_eq!(result.report.no_impact_eligibility, "none");
    }

}
