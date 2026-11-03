// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned sidecar descriptors for source-native built-environment identity
//! and payload metadata. This module validates descriptor consistency only;
//! source-owner authenticity, bytes, rights, and signatures remain separate
//! trust and content-verification obligations.

use crate::built_environment_case::BuiltEnvironmentCase;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const CATALOG_SCHEMA: &str = "luminous.built-environment-source-descriptor-catalog.v0.1";
const IDENTITY_DOMAIN: &[u8] = b"LUMINOUS-IEAC-NODE-IDENTITY-V0.2\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRecordKind {
    Evidence,
    Artifact,
    EngineeringReceipt,
    Decision,
}

impl SourceRecordKind {
    pub fn node_type(self) -> &'static str {
        match self {
            Self::Evidence => "source_evidence",
            Self::Artifact => "engineering_artifact",
            Self::EngineeringReceipt => "engineering_receipt",
            Self::Decision => "decision",
        }
    }

    pub fn case_key(self, case_record_id: &str) -> String {
        let prefix = match self {
            Self::Evidence => "evidence",
            Self::Artifact => "artifact",
            Self::EngineeringReceipt => "engineering_receipt",
            Self::Decision => "decision",
        };
        format!("{prefix}:{case_record_id}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadDigestBasis {
    LocallyRetrievedAndHashed,
    SourceOwnerAttested,
    ExternalReference,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRightsState {
    Verified,
    Restricted,
    Unknown,
    NotApplicable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeIdentityComponent {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePayloadDescriptor {
    pub sha256: Option<String>,
    pub digest_basis: PayloadDigestBasis,
    pub bytes_retrieved: bool,
    pub byte_length: Option<u64>,
    /// Opaque content-addressed object reference, never an arbitrary URL.
    pub payload_object_ref: Option<String>,
    pub media_type: Option<String>,
    pub schema_id: Option<String>,
    pub schema_version: Option<String>,
    pub schema_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshotDescriptor {
    pub source_id: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub captured_at: String,
    pub retrieved_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservationTime {
    pub observed_at: Option<String>,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRightsDescriptor {
    pub state: SourceRightsState,
    pub permitted_uses: Vec<String>,
    pub rights_evidence_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAttestationDescriptor {
    pub statement_ref: Option<String>,
    pub key_id: Option<String>,
    /// The descriptor layer cannot authenticate signatures or trust roots.
    pub trust_evaluation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordDescriptor {
    pub record_kind: SourceRecordKind,
    pub case_record_id: String,
    pub owner_namespace: String,
    pub native_identity_components: Vec<NativeIdentityComponent>,
    pub source_payload: SourcePayloadDescriptor,
    pub source_snapshot: SourceSnapshotDescriptor,
    pub observation_time: SourceObservationTime,
    pub rights: SourceRightsDescriptor,
    pub attestation: SourceAttestationDescriptor,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDescriptorCatalog {
    pub schema_version: String,
    pub catalog_id: String,
    pub case_id: String,
    pub case_representation_sha256: String,
    pub captured_at: String,
    pub descriptors: Vec<SourceRecordDescriptor>,
    pub claim_ceiling: String,
    pub action_authority: String,
    pub limitations: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DescriptorDisposition {
    InternallyConsistentForDeclaredCase,
    IncompleteOrIndeterminate,
    InvalidInput,
}

impl DescriptorDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InternallyConsistentForDeclaredCase => "internally_consistent_for_declared_case",
            Self::IncompleteOrIndeterminate => "incomplete_or_indeterminate",
            Self::InvalidInput => "invalid_input",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DescriptorFinding {
    pub severity: String,
    pub code: String,
    pub record_key: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DescriptorValidation {
    pub disposition: DescriptorDisposition,
    pub case_id: String,
    pub catalog_id: String,
    pub catalog_representation_sha256: String,
    pub descriptor_count: usize,
    pub missing_descriptor_count: usize,
    pub extra_descriptor_count: usize,
    pub identity_sha256_by_case_record: BTreeMap<String, String>,
    pub native_identity_namespaces: BTreeMap<String, String>,
    pub native_identity_components: BTreeMap<String, Vec<NativeIdentityComponent>>,
    pub payload_byte_verification: String,
    pub rights_authentication: String,
    pub source_attestation_authentication: String,
    pub action_authority: String,
    pub findings: Vec<DescriptorFinding>,
}

#[derive(Default)]
struct Findings {
    rows: BTreeSet<DescriptorFinding>,
}

impl Findings {
    fn invalid(&mut self, code: &str, key: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(DescriptorFinding {
            severity: "invalid".into(),
            code: code.into(),
            record_key: key.into(),
            detail: detail.into(),
        });
    }

    fn incomplete(&mut self, code: &str, key: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(DescriptorFinding {
            severity: "incomplete".into(),
            code: code.into(),
            record_key: key.into(),
            detail: detail.into(),
        });
    }

    fn disposition(&self) -> DescriptorDisposition {
        if self.rows.iter().any(|row| row.severity == "invalid") {
            DescriptorDisposition::InvalidInput
        } else if !self.rows.is_empty() {
            DescriptorDisposition::IncompleteOrIndeterminate
        } else {
            DescriptorDisposition::InternallyConsistentForDeclaredCase
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_component_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'-')
        })
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
    let at = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|part| part.parse::<u32>().ok())
    };
    let year = at(0, 4).unwrap_or(0);
    let month = at(5, 7).unwrap_or(0);
    let day = at(8, 10).unwrap_or(0);
    let hour = at(11, 13).unwrap_or(99);
    let minute = at(14, 16).unwrap_or(99);
    let second = at(17, 19).unwrap_or(99);
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

fn append_identity_field(encoded: &mut Vec<u8>, value: &str) {
    encoded.extend_from_slice(&(value.len() as u64).to_be_bytes());
    encoded.extend_from_slice(value.as_bytes());
}

fn identity_digest(
    descriptor: &SourceRecordDescriptor,
    case_id: &str,
) -> Result<String, String> {
    if descriptor.owner_namespace.trim().is_empty()
        || descriptor.owner_namespace.len() > 1024
        || descriptor.owner_namespace.as_str() != descriptor.owner_namespace.trim()
    {
        return Err("owner namespace must be non-empty, trimmed, and at most 1024 UTF-8 bytes".into());
    }

    let mut components = BTreeMap::new();
    for component in &descriptor.native_identity_components {
        if !valid_component_name(&component.name) {
            return Err(format!("invalid native identity component name: {}", component.name));
        }
        if component.value.is_empty() {
            return Err(format!("native identity component {} is empty", component.name));
        }
        if components
            .insert(component.name.clone(), component.value.clone())
            .is_some()
        {
            return Err(format!("duplicate native identity component: {}", component.name));
        }
    }
    if !components.contains_key("native_id") {
        return Err("native identity requires native_id".into());
    }
    if ![
        "native_record_id",
        "native_revision",
        "native_sequence",
        "native_observation_id",
    ]
    .iter()
    .any(|name| components.contains_key(*name))
    {
        return Err("native identity requires a record, revision, sequence, or observation discriminator".into());
    }

    if components.contains_key("case_id") {
        return Err("case_id is a projector-owned identity component and must not be supplied by the source descriptor".into());
    }
    components.insert("case_id".into(), case_id.to_owned());
    let mut encoded = IDENTITY_DOMAIN.to_vec();
    append_identity_field(&mut encoded, descriptor.record_kind.node_type());
    append_identity_field(&mut encoded, &descriptor.owner_namespace);
    for (name, value) in components {
        append_identity_field(&mut encoded, &name);
        append_identity_field(&mut encoded, &value);
    }
    Ok(sha256_hex(&encoded))
}

fn expected_case_records(case: &BuiltEnvironmentCase) -> BTreeSet<(SourceRecordKind, String)> {
    case.evidence
        .iter()
        .map(|record| (SourceRecordKind::Evidence, record.evidence_id.clone()))
        .chain(
            case.artifacts
                .iter()
                .map(|record| (SourceRecordKind::Artifact, record.artifact_id.clone())),
        )
        .chain(
            case.engineering_receipts
                .iter()
                .map(|record| (SourceRecordKind::EngineeringReceipt, record.receipt_id.clone())),
        )
        .chain(
            case.decisions
                .iter()
                .map(|record| (SourceRecordKind::Decision, record.event_id.clone())),
        )
        .collect()
}

fn expected_identity_components(
    case: &BuiltEnvironmentCase,
    kind: SourceRecordKind,
    record_id: &str,
) -> Option<BTreeMap<String, String>> {
    let mut components = BTreeMap::new();
    match kind {
        SourceRecordKind::Evidence => {
            let record = case.evidence.iter().find(|row| row.evidence_id == record_id)?;
            components.insert("native_id".into(), record.source_id.clone());
            components.insert("native_record_id".into(), record.evidence_id.clone());
            components.insert("field_path".into(), record.field_path.clone());
            components.insert("subject_id".into(), record.subject_id.clone());
            components.insert("jurisdiction".into(), record.jurisdiction.clone());
        }
        SourceRecordKind::Artifact => {
            let record = case.artifacts.iter().find(|row| row.artifact_id == record_id)?;
            components.insert("native_id".into(), record.design_id.clone());
            components.insert("native_record_id".into(), record.artifact_id.clone());
            components.insert("native_revision".into(), record.revision.to_string());
            components.insert("artifact_family".into(), record.family_id.clone());
            components.insert("jurisdiction".into(), record.jurisdiction.clone());
        }
        SourceRecordKind::EngineeringReceipt => {
            let record = case.engineering_receipts.iter().find(|row| row.receipt_id == record_id)?;
            components.insert("native_id".into(), record.design_id.clone());
            components.insert("native_record_id".into(), record.receipt_id.clone());
        }
        SourceRecordKind::Decision => {
            let record = case.decisions.iter().find(|row| row.event_id == record_id)?;
            components.insert("native_id".into(), record.design_id.clone());
            components.insert("native_record_id".into(), record.event_id.clone());
        }
    }
    Some(components)
}

fn case_payload_digest(
    case: &BuiltEnvironmentCase,
    kind: SourceRecordKind,
    record_id: &str,
) -> Option<String> {
    match kind {
        SourceRecordKind::Evidence => case
            .evidence
            .iter()
            .find(|record| record.evidence_id == record_id)
            .and_then(|record| record.source_payload_sha256.clone()),
        SourceRecordKind::Artifact => case
            .artifacts
            .iter()
            .find(|record| record.artifact_id == record_id)
            .map(|record| record.content_sha256.clone()),
        SourceRecordKind::EngineeringReceipt | SourceRecordKind::Decision => None,
    }
}

fn expected_cas_reference(sha256: &str) -> String {
    format!("cas:sha256:{sha256}")
}

const PERMITTED_USES: &[&str] = &[
    "impact_analysis",
    "design_review",
    "engineering_review",
    "construction_release_review",
    "maintenance_review",
    "benchmarking",
];

fn validate_payload(
    descriptor: &SourceRecordDescriptor,
    expected_case_digest: Option<&str>,
    findings: &mut Findings,
    key: &str,
) {
    let payload = &descriptor.source_payload;
    if let Some(digest) = payload.sha256.as_deref() {
        if !valid_sha256(digest) {
            findings.invalid("INVALID_SOURCE_PAYLOAD_DIGEST", key, "source payload digest is not lowercase SHA-256");
        }
    }

    match payload.digest_basis {
        PayloadDigestBasis::LocallyRetrievedAndHashed => {
            if !payload.bytes_retrieved
                || payload.sha256.is_none()
                || payload.byte_length.is_none()
                || payload.payload_object_ref.as_deref().is_none_or(str::is_empty)
            {
                findings.invalid(
                    "LOCAL_PAYLOAD_DIGEST_BASIS_INCONSISTENT",
                    key,
                    "locally retrieved and hashed payload requires digest, byte retrieval, byte length, and content-addressed object reference",
                );
            }
            if let (Some(digest), Some(object_ref)) =
                (payload.sha256.as_deref(), payload.payload_object_ref.as_deref())
            {
                if object_ref != expected_cas_reference(digest) {
                    findings.invalid(
                        "PAYLOAD_OBJECT_REFERENCE_DIGEST_MISMATCH",
                        key,
                        "content-addressed object reference must bind the exact declared SHA-256",
                    );
                }
            }
        }
        PayloadDigestBasis::SourceOwnerAttested => {
            if payload.bytes_retrieved || payload.sha256.is_none() {
                findings.invalid(
                    "SOURCE_ATTESTED_DIGEST_BASIS_INCONSISTENT",
                    key,
                    "source-owner-attested digest requires a digest and must not claim local byte retrieval",
                );
            }
            if descriptor.attestation.statement_ref.as_deref().is_none_or(str::is_empty) {
                findings.incomplete(
                    "SOURCE_PAYLOAD_ATTESTATION_REFERENCE_MISSING",
                    key,
                    "source-owner-attested digest has no statement reference",
                );
            }
            findings.incomplete(
                "SOURCE_PAYLOAD_DIGEST_NOT_LOCALLY_REVERIFIED",
                key,
                "the descriptor cites an owner-attested digest; the descriptor validator does not fetch or hash source bytes",
            );
        }
        PayloadDigestBasis::ExternalReference => {
            if payload.bytes_retrieved
                || payload.sha256.is_none()
                || payload.payload_object_ref.as_deref().is_none_or(str::is_empty)
            {
                findings.invalid(
                    "EXTERNAL_PAYLOAD_REFERENCE_INCONSISTENT",
                    key,
                    "external reference requires a digest and object reference and cannot claim local byte retrieval",
                );
            }
            if let (Some(digest), Some(object_ref)) =
                (payload.sha256.as_deref(), payload.payload_object_ref.as_deref())
            {
                if object_ref != expected_cas_reference(digest) {
                    findings.invalid(
                        "PAYLOAD_OBJECT_REFERENCE_DIGEST_MISMATCH",
                        key,
                        "content-addressed object reference must bind the exact declared SHA-256",
                    );
                }
            }
            findings.incomplete(
                "EXTERNAL_PAYLOAD_BYTES_NOT_RETRIEVED",
                key,
                "payload is referenced but its bytes have not been retrieved and independently hashed by this verifier",
            );
        }
        PayloadDigestBasis::Unavailable => {
            if payload.sha256.is_some()
                || payload.bytes_retrieved
                || payload.byte_length.is_some()
                || payload.payload_object_ref.is_some()
            {
                findings.invalid(
                    "UNAVAILABLE_PAYLOAD_BASIS_INCONSISTENT",
                    key,
                    "unavailable payload must not claim a digest, byte retrieval, byte length, or object reference",
                );
            }
            findings.incomplete(
                "SOURCE_PAYLOAD_UNAVAILABLE",
                key,
                "no source payload digest or bytes are available",
            );
        }
    }

    match (expected_case_digest, payload.sha256.as_deref()) {
        (Some(expected), Some(actual)) if expected != actual => findings.invalid(
            "CASE_SOURCE_DIGEST_DISAGREEMENT",
            key,
            "descriptor source payload digest disagrees with the digest already present in BuiltEnvironmentCase",
        ),
        (Some(_), None) => findings.incomplete(
            "DESCRIPTOR_PAYLOAD_DIGEST_MISSING",
            key,
            "BuiltEnvironmentCase supplies a source digest but the descriptor does not bind it",
        ),
        _ => {}
    }

    if payload.media_type.as_deref().is_none_or(str::is_empty) {
        findings.incomplete(
            "SOURCE_MEDIA_TYPE_UNAVAILABLE",
            key,
            "source media type is not represented by this descriptor",
        );
    }
    if payload.schema_id.as_deref().is_none_or(str::is_empty)
        || payload.schema_version.as_deref().is_none_or(str::is_empty)
    {
        findings.incomplete(
            "SOURCE_SCHEMA_IDENTITY_UNAVAILABLE",
            key,
            "source schema ID or version is not represented by this descriptor",
        );
    }
    if payload.schema_id.is_some() && payload.schema_sha256.as_deref().is_none_or(|v| !valid_sha256(v)) {
        findings.incomplete(
            "SOURCE_SCHEMA_DIGEST_UNAVAILABLE",
            key,
            "source schema is named but its exact schema digest is missing or invalid",
        );
    }
}

fn validate_rights(descriptor: &SourceRecordDescriptor, findings: &mut Findings, key: &str) {
    let mut seen_uses = BTreeSet::new();
    for permitted_use in &descriptor.rights.permitted_uses {
        if !PERMITTED_USES.contains(&permitted_use.as_str()) {
            findings.invalid(
                "UNSUPPORTED_PERMITTED_USE",
                key,
                format!("unsupported permitted use: {permitted_use}"),
            );
        }
        if !seen_uses.insert(permitted_use) {
            findings.invalid(
                "DUPLICATE_PERMITTED_USE",
                key,
                format!("permitted use {permitted_use} appears more than once"),
            );
        }
    }
    match descriptor.rights.state {
        SourceRightsState::Verified => {
            if descriptor.rights.permitted_uses.is_empty()
                || descriptor.rights.rights_evidence_ref.as_deref().is_none_or(str::is_empty)
            {
                findings.incomplete(
                    "VERIFIED_RIGHTS_LACK_SCOPE_EVIDENCE",
                    key,
                    "verified rights require at least one permitted use and an evidence reference",
                );
            }
        }
        SourceRightsState::Unknown => {
            if !descriptor.rights.permitted_uses.is_empty() {
                findings.invalid(
                    "UNKNOWN_RIGHTS_WITH_PERMITTED_USES",
                    key,
                    "unknown rights cannot carry permitted uses",
                );
            }
            findings.incomplete(
                "SOURCE_RIGHTS_UNKNOWN",
                key,
                "source rights are unknown",
            );
        }
        SourceRightsState::Restricted => findings.incomplete(
            "SOURCE_RIGHTS_RESTRICTED",
            key,
            "source is restricted; permitted-use adjudication is outside this descriptor verifier",
        ),
        SourceRightsState::NotApplicable => {}
    }
}

fn validate_time(descriptor: &SourceRecordDescriptor, catalog: &SourceDescriptorCatalog, findings: &mut Findings, key: &str) {
    let snapshot = &descriptor.source_snapshot;
    if snapshot.source_id.trim().is_empty() || snapshot.snapshot_id.trim().is_empty() {
        findings.invalid(
            "EMPTY_SOURCE_SNAPSHOT_IDENTITY",
            key,
            "source_id and snapshot_id must be non-empty",
        );
    }
    if !valid_sha256(&snapshot.snapshot_sha256) {
        findings.invalid("INVALID_SOURCE_SNAPSHOT_DIGEST", key, "source snapshot digest is invalid");
    }
    for (name, time) in [
        ("snapshot.captured_at", snapshot.captured_at.as_str()),
        ("snapshot.retrieved_at", snapshot.retrieved_at.as_str()),
        ("observation.recorded_at", descriptor.observation_time.recorded_at.as_str()),
    ] {
        if !valid_utc_timestamp(time) {
            findings.invalid("INVALID_SOURCE_TIME", key, format!("{name} is not canonical UTC whole-second time"));
        }
    }
    if let Some(observed_at) = descriptor.observation_time.observed_at.as_deref() {
        if !valid_utc_timestamp(observed_at) {
            findings.invalid("INVALID_SOURCE_TIME", key, "observed_at is not canonical UTC whole-second time");
        } else if observed_at > descriptor.observation_time.recorded_at.as_str() {
            findings.incomplete(
                "OBSERVATION_TIME_AFTER_RECORDING_TIME",
                key,
                "source observation time is later than its recorded time",
            );
        }
    }
    if snapshot.captured_at > snapshot.retrieved_at {
        findings.invalid("SOURCE_SNAPSHOT_TIME_REVERSAL", key, "source retrieval precedes snapshot capture");
    }
    if snapshot.retrieved_at > catalog.captured_at {
        findings.incomplete(
            "SOURCE_RETRIEVED_AFTER_CATALOG_CAPTURE",
            key,
            "source snapshot was retrieved after the catalogue capture time",
        );
    }
    if descriptor.observation_time.recorded_at > catalog.captured_at {
        findings.incomplete(
            "SOURCE_RECORD_AFTER_CATALOG_CAPTURE",
            key,
            "source record was recorded after the catalogue capture time",
        );
    }
}

/// Validate a descriptor catalogue against one exact typed case.
///
/// A consistent result means only that the descriptor catalogue has one
/// internally coherent descriptor per typed case record. This does not fetch
/// payload bytes, authenticate owner namespaces, evaluate rights, verify
/// attestations, or establish that the case/profile is complete for a real
/// engineering decision.
pub fn validate_source_descriptor_catalog(
    case: &BuiltEnvironmentCase,
    catalog: &SourceDescriptorCatalog,
) -> DescriptorValidation {
    let mut findings = Findings::default();
    if catalog.schema_version != CATALOG_SCHEMA {
        findings.invalid("UNSUPPORTED_DESCRIPTOR_SCHEMA", "catalog", "unsupported source descriptor schema version");
    }
    if catalog.case_id != case.case_id {
        findings.invalid("CATALOG_CASE_ID_MISMATCH", "catalog", "catalogue case ID differs from BuiltEnvironmentCase");
    }
    if catalog.claim_ceiling != "descriptor_integrity_only" {
        findings.invalid("DESCRIPTOR_CLAIM_CEILING_ESCALATION", "catalog", "catalogue claim ceiling must be descriptor_integrity_only");
    }
    if catalog.action_authority != "none" {
        findings.invalid("DESCRIPTOR_ACTION_AUTHORITY_ESCALATION", "catalog", "descriptor catalogue grants no action authority");
    }
    if !valid_utc_timestamp(&catalog.captured_at) {
        findings.invalid("INVALID_CATALOG_CAPTURE_TIME", "catalog", "catalog captured_at must be canonical UTC whole-second time");
    }
    if !valid_sha256(&catalog.case_representation_sha256) {
        findings.invalid("INVALID_CASE_REPRESENTATION_DIGEST", "catalog", "case representation digest must be lowercase SHA-256");
    }
    if catalog.limitations.is_empty() {
        findings.invalid("DESCRIPTOR_LIMITATIONS_MISSING", "catalog", "descriptor catalogue must state its limitations");
    }

    match serde_json::to_vec(case) {
        Ok(bytes) if sha256_hex(&bytes) != catalog.case_representation_sha256 => findings.invalid(
            "CASE_REPRESENTATION_DIGEST_MISMATCH",
            "catalog",
            "catalogue is not bound to the exact typed BuiltEnvironmentCase representation",
        ),
        Err(error) => findings.invalid(
            "CASE_REPRESENTATION_SERIALIZATION_FAILED",
            "catalog",
            format!("cannot serialize BuiltEnvironmentCase: {error}"),
        ),
        _ => {}
    }

    let expected = expected_case_records(case);
    let total_case_records = case.evidence.len()
        + case.artifacts.len()
        + case.engineering_receipts.len()
        + case.decisions.len();
    if expected.len() != total_case_records {
        findings.invalid(
            "DUPLICATE_CASE_RECORD_KEY",
            "case",
            "one or more typed case records reuse the same record-kind and native case ID",
        );
    }
    let mut seen = BTreeSet::new();
    let mut native_identities = BTreeMap::<String, (String, Option<String>)>::new();
    let mut identity_sha256_by_case_record = BTreeMap::new();
    let mut native_identity_namespaces = BTreeMap::new();
    let mut native_identity_components = BTreeMap::new();

    for descriptor in &catalog.descriptors {
        let key = descriptor
            .record_kind
            .case_key(&descriptor.case_record_id);
        let record_key = (descriptor.record_kind, descriptor.case_record_id.clone());
        if descriptor.case_record_id.trim().is_empty() {
            findings.invalid("EMPTY_CASE_RECORD_ID", &key, "case_record_id must be non-empty");
        }
        if !expected.contains(&record_key) {
            findings.invalid("DESCRIPTOR_WITHOUT_CASE_RECORD", &key, "descriptor has no matching typed case record");
        }
        if !seen.insert(record_key.clone()) {
            findings.invalid("DUPLICATE_CASE_RECORD_DESCRIPTOR", &key, "each typed case record may have only one descriptor");
        }

        let expected_components =
            expected_identity_components(case, descriptor.record_kind, &descriptor.case_record_id);
        let mut supplied_components = descriptor
            .native_identity_components
            .iter()
            .map(|component| (component.name.clone(), component.value.clone()))
            .collect::<BTreeMap<_, _>>();
        if let Some(expected_components) = expected_components.as_ref() {
            for (name, expected_value) in expected_components {
                if supplied_components.get(name) != Some(expected_value) {
                    findings.invalid(
                        "NATIVE_IDENTITY_COMPONENT_CASE_DISAGREEMENT",
                        &key,
                        format!("native identity component {name} disagrees with the typed case record"),
                    );
                }
            }
        }
        match identity_digest(descriptor, &case.case_id) {
            Ok(identity_sha256) => {
                if let Some((other_key, previous_digest)) = native_identities.get(&identity_sha256) {
                    if other_key != &key {
                        findings.invalid(
                            "DUPLICATE_NATIVE_IDENTITY",
                            &key,
                            format!("native identity also occurs at {other_key}"),
                        );
                    }
                    if previous_digest != &descriptor.source_payload.sha256 {
                        findings.invalid(
                            "DUPLICATE_NATIVE_IDENTITY_CONFLICTING_DIGEST",
                            &key,
                            "the same owner-native identity is bound to conflicting payload digests",
                        );
                    }
                } else {
                    native_identities.insert(
                        identity_sha256.clone(),
                        (key.clone(), descriptor.source_payload.sha256.clone()),
                    );
                }
                identity_sha256_by_case_record.insert(key.clone(), identity_sha256);
                native_identity_namespaces.insert(key.clone(), descriptor.owner_namespace.clone());
                supplied_components.insert("case_id".into(), case.case_id.clone());
                native_identity_components.insert(
                    key.clone(),
                    supplied_components
                        .into_iter()
                        .map(|(name, value)| NativeIdentityComponent { name, value })
                        .collect(),
                );
            }
            Err(error) => findings.invalid("INVALID_NATIVE_IDENTITY", &key, error),
        }

        if descriptor.owner_namespace.trim().is_empty()
            || descriptor.owner_namespace.len() > 1024
            || descriptor.owner_namespace.as_str() != descriptor.owner_namespace.trim()
        {
            findings.invalid("INVALID_OWNER_NAMESPACE", &key, "owner namespace must be non-empty, trimmed, and at most 1024 UTF-8 bytes");
        }
        validate_payload(
            descriptor,
            case_payload_digest(case, descriptor.record_kind, &descriptor.case_record_id).as_deref(),
            &mut findings,
            &key,
        );
        validate_rights(descriptor, &mut findings, &key);
        validate_time(descriptor, catalog, &mut findings, &key);
        if descriptor.attestation.trust_evaluation != "not_evaluated_by_descriptor_catalog" {
            findings.invalid(
                "ATTESTATION_TRUST_PROMOTION",
                &key,
                "descriptor catalogue must not claim to authenticate signatures or trust roots",
            );
        }
        if descriptor.attestation.statement_ref.is_some() && descriptor.attestation.key_id.is_none() {
            findings.incomplete(
                "ATTESTATION_KEY_ID_MISSING",
                &key,
                "attestation reference is present without a key identity",
            );
        }
    }

    let missing = expected
        .difference(&seen)
        .map(|(kind, record_id)| {
            let key = kind.case_key(record_id);
            findings.incomplete(
                "SOURCE_DESCRIPTOR_MISSING",
                key.clone(),
                "typed case record has no owner-native source descriptor",
            );
            key
        })
        .collect::<Vec<_>>();

    if catalog.descriptors.is_empty() {
        findings.invalid("EMPTY_SOURCE_DESCRIPTOR_CATALOG", "catalog", "empty descriptor catalogue is not meaningful");
    }

    let disposition = findings.disposition();
    let findings = findings.rows.into_iter().collect();
    DescriptorValidation {
        disposition,
        case_id: case.case_id.clone(),
        catalog_id: catalog.catalog_id.clone(),
        catalog_representation_sha256: serde_json::to_vec(catalog)
            .map(|bytes| sha256_hex(&bytes))
            .unwrap_or_default(),
        descriptor_count: catalog.descriptors.len(),
        missing_descriptor_count: missing.len(),
        extra_descriptor_count: catalog.descriptors.len().saturating_sub(expected.len()),
        identity_sha256_by_case_record,
        native_identity_namespaces,
        native_identity_components,
        payload_byte_verification: "not_performed".into(),
        rights_authentication: "not_performed".into(),
        source_attestation_authentication: "not_performed".into(),
        action_authority: "none".into(),
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_environment_case::{
        ArtifactKind, ArtifactRecord, BuiltEnvironmentCase, DataRights, DecisionDisposition,
        DecisionRecord, EngineeringConclusion, EngineeringDiscipline, EngineeringReceipt,
        EvidenceRecord, EvidenceState, ReviewLevel, SolverRunStatus,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn case() -> BuiltEnvironmentCase {
        BuiltEnvironmentCase {
            case_id: "case-descriptor-001".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence: vec![EvidenceRecord {
                evidence_id: "evidence-001".into(),
                subject_id: "design-001".into(),
                field_path: "geometry.revision".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: Some(digest('a')),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_000),
            }],
            artifacts: vec![ArtifactRecord {
                artifact_id: "artifact-001".into(),
                design_id: "design-001".into(),
                family_id: "design-model".into(),
                revision: 2,
                kind: ArtifactKind::DesignModel,
                content_sha256: digest('b'),
                parent_artifacts: vec![],
                evidence_ids: vec!["evidence-001".into()],
                jurisdiction: "synthetic-jurisdiction".into(),
                created_at_unix_seconds: 1_762_000_100,
            }],
            engineering_receipts: vec![EngineeringReceipt {
                receipt_id: "receipt-001".into(),
                design_id: "design-001".into(),
                input_artifact_id: "artifact-001".into(),
                input_artifact_sha256: digest('b'),
                discipline: EngineeringDiscipline::Structural,
                solver_id: Some("synthetic-solver".into()),
                solver_version: Some("1.0.0".into()),
                run_status: SolverRunStatus::Completed,
                conclusion: EngineeringConclusion::PassWithinDeclaredScope,
                review_level: ReviewLevel::IndependentlyChecked,
                reviewer_id: Some("synthetic-reviewer".into()),
                assumption_evidence_ids: vec!["evidence-001".into()],
                declared_scope: "synthetic-only".into(),
            }],
            decisions: vec![DecisionRecord {
                event_id: "decision-001".into(),
                design_id: "design-001".into(),
                target_artifact_id: "artifact-001".into(),
                target_artifact_sha256: digest('b'),
                disposition: DecisionDisposition::Conditional,
                actor_id: "synthetic-actor".into(),
                authority_evidence_id: Some("evidence-001".into()),
                engineering_receipt_ids: vec!["receipt-001".into()],
                decided_at_unix_seconds: 1_762_000_300,
                conditions: vec!["synthetic-only".into()],
                supersedes_event_id: None,
            }],
        }
    }

    fn descriptor(
        kind: SourceRecordKind,
        case_record_id: &str,
        native_id: &str,
        native_record_id: &str,
        namespace: &str,
        payload_sha: char,
    ) -> SourceRecordDescriptor {
        SourceRecordDescriptor {
            record_kind: kind,
            case_record_id: case_record_id.into(),
            owner_namespace: namespace.into(),
            native_identity_components: vec![
                NativeIdentityComponent {
                    name: "native_id".into(),
                    value: native_id.into(),
                },
                NativeIdentityComponent {
                    name: "native_record_id".into(),
                    value: native_record_id.into(),
                },
                NativeIdentityComponent {
                    name: "native_revision".into(),
                    value: if kind == SourceRecordKind::Artifact { "2" } else { "1" }.into(),
                },
            ],
            source_payload: SourcePayloadDescriptor {
                sha256: Some(digest(payload_sha)),
                digest_basis: PayloadDigestBasis::LocallyRetrievedAndHashed,
                bytes_retrieved: true,
                byte_length: Some(128),
                payload_object_ref: Some(format!("cas:sha256:{}", digest(payload_sha))),
                media_type: Some("application/json".into()),
                schema_id: Some("urn:synthetic:schema:record".into()),
                schema_version: Some("1.0".into()),
                schema_sha256: Some(digest('e')),
            },
            source_snapshot: SourceSnapshotDescriptor {
                source_id: namespace.into(),
                snapshot_id: "snapshot-001".into(),
                snapshot_sha256: digest('f'),
                captured_at: "2026-11-03T10:00:00Z".into(),
                retrieved_at: "2026-11-03T10:01:00Z".into(),
            },
            observation_time: SourceObservationTime {
                observed_at: Some("2026-11-03T09:59:00Z".into()),
                recorded_at: "2026-11-03T10:00:00Z".into(),
            },
            rights: SourceRightsDescriptor {
                state: SourceRightsState::Verified,
                permitted_uses: vec!["impact_analysis".into(), "engineering_review".into()],
                rights_evidence_ref: Some("rights-evidence:001".into()),
            },
            attestation: SourceAttestationDescriptor {
                statement_ref: Some("scitt:statement:001".into()),
                key_id: Some("urn:synthetic:key:001".into()),
                trust_evaluation: "not_evaluated_by_descriptor_catalog".into(),
            },
        }
    }

    fn catalog(case: &BuiltEnvironmentCase) -> SourceDescriptorCatalog {
        let bytes = serde_json::to_vec(case).unwrap();
        SourceDescriptorCatalog {
            schema_version: CATALOG_SCHEMA.into(),
            catalog_id: "catalog-synthetic-001".into(),
            case_id: case.case_id.clone(),
            case_representation_sha256: sha256_hex(&bytes),
            captured_at: "2026-11-03T10:02:00Z".into(),
            descriptors: vec![
                descriptor(
                    SourceRecordKind::Evidence,
                    "evidence-001",
                    "survey-registry",
                    "survey-record-001",
                    "urn:synthetic:survey-owner",
                    'a',
                ),
                descriptor(
                    SourceRecordKind::Artifact,
                    "artifact-001",
                    "design-001",
                    "model-001",
                    "urn:synthetic:design-owner",
                    'b',
                ),
                descriptor(
                    SourceRecordKind::EngineeringReceipt,
                    "receipt-001",
                    "design-001",
                    "solver-receipt-001",
                    "urn:synthetic:solver-owner",
                    'c',
                ),
                descriptor(
                    SourceRecordKind::Decision,
                    "decision-001",
                    "design-001",
                    "decision-record-001",
                    "urn:synthetic:authority-owner",
                    'd',
                ),
            ],
            claim_ceiling: "descriptor_integrity_only".into(),
            action_authority: "none".into(),
            limitations: vec!["Synthetic descriptor data; trust and byte checks are separate.".into()],
        }
    }

    #[test]
    fn complete_synthetic_descriptor_catalog_reconciles_all_case_records() {
        let case = case();
        let result = validate_source_descriptor_catalog(&case, &catalog(&case));
        assert_eq!(
            result.disposition,
            DescriptorDisposition::InternallyConsistentForDeclaredCase
        );
        assert_eq!(result.descriptor_count, 4);
        assert_eq!(result.missing_descriptor_count, 0);
        assert_eq!(result.identity_sha256_by_case_record.len(), 4);
        assert_eq!(result.native_identity_namespaces.len(), 4);
        assert_eq!(result.payload_byte_verification, "not_performed");
        assert_eq!(result.rights_authentication, "not_performed");
        assert_eq!(result.source_attestation_authentication, "not_performed");
        assert_eq!(result.action_authority, "none");
    }

    #[test]
    fn same_payload_from_different_owner_namespaces_keeps_distinct_identities() {
        let mut case = case();
        let mut second = case.evidence[0].clone();
        second.evidence_id = "evidence-002".into();
        second.subject_id = "design-002".into();
        case.evidence.push(second);

        let mut catalog = catalog(&case);
        let mut second_descriptor = descriptor(
            SourceRecordKind::Evidence,
            "evidence-002",
            "survey-project-002",
            "survey-record-002",
            "urn:synthetic:second-survey-owner",
            'a',
        );
        second_descriptor.source_snapshot.snapshot_id = "snapshot-002".into();
        catalog.descriptors.push(second_descriptor);
        catalog.case_representation_sha256 = sha256_hex(&serde_json::to_vec(&case).unwrap());

        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(
            result.disposition,
            DescriptorDisposition::InternallyConsistentForDeclaredCase
        );
        let first = result.identity_sha256_by_case_record["evidence:evidence-001"].clone();
        let second = result.identity_sha256_by_case_record["evidence:evidence-002"].clone();
        assert_ne!(first, second);
    }

    #[test]
    fn same_owner_namespace_native_revision_replays_and_revision_change_changes_identity() {
        let original = descriptor(
            SourceRecordKind::Artifact,
            "artifact-001",
            "design-001",
            "ifc-model-001",
            "urn:synthetic:design-owner",
            'b',
        );
        let replay = original.clone();
        assert_eq!(identity_digest(&original).unwrap(), identity_digest(&replay).unwrap());

        let mut revised = original.clone();
        let component = revised
            .native_identity_components
            .iter_mut()
            .find(|component| component.name == "native_revision")
            .unwrap();
        component.value = "3".into();
        assert_ne!(identity_digest(&original).unwrap(), identity_digest(&revised).unwrap());
    }

    #[test]
    fn duplicate_native_identity_with_conflicting_payload_digest_is_invalid() {
        let mut case = case();
        let mut second = case.evidence[0].clone();
        second.evidence_id = "evidence-002".into();
        second.source_payload_sha256 = Some(digest('f'));
        case.evidence.push(second);

        let mut catalog = catalog(&case);
        let mut duplicate = catalog.descriptors[0].clone();
        duplicate.case_record_id = "evidence-002".into();
        duplicate.source_payload.sha256 = Some(digest('f'));
        catalog.descriptors.push(duplicate);
        catalog.case_representation_sha256 = sha256_hex(&serde_json::to_vec(&case).unwrap());

        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "DUPLICATE_NATIVE_IDENTITY_CONFLICTING_DIGEST"
        }));
    }

    #[test]
    fn duplicate_typed_case_record_key_is_invalid_even_before_descriptor_reconciliation() {
        let mut case = case();
        case.evidence.push(case.evidence[0].clone());
        let catalog = catalog(&case);
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "DUPLICATE_CASE_RECORD_KEY"
        }));
    }

    #[test]
    fn missing_descriptor_stays_incomplete_not_fabricated() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors.retain(|descriptor| {
            !(descriptor.record_kind == SourceRecordKind::Evidence
                && descriptor.case_record_id == "evidence-001")
        });
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(
            result.disposition,
            DescriptorDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(result.missing_descriptor_count, 1);
        assert!(result.findings.iter().any(|finding| finding.code == "SOURCE_DESCRIPTOR_MISSING"));
    }

    #[test]
    fn source_digest_disagreement_is_invalid() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors[0].source_payload.sha256 = Some(digest('f'));
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| finding.code == "CASE_SOURCE_DIGEST_DISAGREEMENT"));
    }

    #[test]
    fn duplicate_owner_native_identity_is_invalid_even_when_payload_matches() {
        let mut case = case();
        let mut second = case.evidence[0].clone();
        second.evidence_id = "evidence-002".into();
        case.evidence.push(second);
        let mut catalog = catalog(&case);
        let mut duplicate = catalog.descriptors[0].clone();
        duplicate.case_record_id = "evidence-002".into();
        catalog.descriptors.push(duplicate);
        catalog.case_representation_sha256 = sha256_hex(&serde_json::to_vec(&case).unwrap());

        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| finding.code == "DUPLICATE_NATIVE_IDENTITY"));
    }

    #[test]
    fn unknown_rights_cannot_carry_permitted_uses() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors[0].rights.state = SourceRightsState::Unknown;
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| finding.code == "UNKNOWN_RIGHTS_WITH_PERMITTED_USES"));
    }

    #[test]
    fn content_addressed_object_reference_must_match_payload_digest() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors[0].source_payload.payload_object_ref =
            Some(format!("cas:sha256:{}", digest('f')));
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "PAYLOAD_OBJECT_REFERENCE_DIGEST_MISMATCH"
        }));
    }

    #[test]
    fn unsupported_rights_purpose_is_rejected() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors[0]
            .rights
            .permitted_uses
            .push("automatic_construction_release".into());
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(result.disposition, DescriptorDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "UNSUPPORTED_PERMITTED_USE"
        }));
    }

    #[test]
    fn observation_after_recording_is_loss_accounted() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors[0].observation_time.observed_at =
            Some("2026-11-03T10:01:00Z".into());
        catalog.descriptors[0].observation_time.recorded_at =
            "2026-11-03T10:00:00Z".into();
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(
            result.disposition,
            DescriptorDisposition::IncompleteOrIndeterminate
        );
        assert!(result.findings.iter().any(|finding| {
            finding.code == "OBSERVATION_TIME_AFTER_RECORDING_TIME"
        }));
    }

    #[test]
    fn absent_schema_or_media_type_is_loss_accounted() {
        let case = case();
        let mut catalog = catalog(&case);
        catalog.descriptors[0].source_payload.media_type = None;
        catalog.descriptors[0].source_payload.schema_id = None;
        catalog.descriptors[0].source_payload.schema_version = None;
        let result = validate_source_descriptor_catalog(&case, &catalog);
        assert_eq!(
            result.disposition,
            DescriptorDisposition::IncompleteOrIndeterminate
        );
        assert!(result.findings.iter().any(|finding| finding.code == "SOURCE_MEDIA_TYPE_UNAVAILABLE"));
        assert!(result.findings.iter().any(|finding| finding.code == "SOURCE_SCHEMA_IDENTITY_UNAVAILABLE"));
    }
}
