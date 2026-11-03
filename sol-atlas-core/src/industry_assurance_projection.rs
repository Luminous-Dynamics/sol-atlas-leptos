// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Loss-accounted, read-only projection from BuiltEnvironmentCase into IEAC v0.2.
//!
//! The projector preserves source-native identifiers and payload digests where
//! the source model provides them. It never infers verification, issuer
//! authority, trusted time, or action permission from successful projection.

use crate::built_environment_case::{
    ArtifactRecord, BuiltEnvironmentCase, DecisionRecord, EngineeringReceipt, EvidenceRecord,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const GRAPH_SCHEMA_V02: &str = "luminous.ieac-assurance-dependency-graph.v0.2";
const PROJECTION_SCHEMA_V01: &str = "luminous.built-environment-ieac-projection.v0.1";
const IDENTITY_DOMAIN: &[u8] = b"LUMINOUS-IEAC-NODE-IDENTITY-V0.2\0";
const IDENTITY_NAMESPACE: &str = "urn:luminous:sol-atlas:built-environment-case";
const PROJECTOR_ISSUER: &str = "urn:luminous:sol-atlas:built-environment-projection";
const SOURCE_SCHEMA_UNKNOWN: &str = "unknown-source-payload-schema";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionDisposition {
    ProjectedWithoutKnownLoss,
    ProjectedWithKnownLoss,
    InvalidInput,
}

impl ProjectionDisposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProjectedWithoutKnownLoss => "projected_without_known_loss",
            Self::ProjectedWithKnownLoss => "projected_with_known_loss",
            Self::InvalidInput => "invalid_input",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProjectionContext {
    /// Explicit projection/evaluation instant in canonical UTC seconds.
    pub captured_at_utc: String,
    /// Exact digest of the compiled projector binary, not merely the source tree.
    pub projector_build_sha256: Option<String>,
    /// Digest of the frozen identity, mapping, and loss-accounting profile.
    pub mapping_profile_sha256: String,
}

#[derive(Clone, Debug)]
pub struct ProjectionOutput {
    /// IEAC v0.2 dependency graph. Its claim ceiling is never higher than
    /// record_only, and becomes incomplete_or_indeterminate on any loss.
    pub graph: Value,
    /// Receipt explaining native identity mappings, edge mappings, losses,
    /// generated payloads, exact consumed-case representation, and digests.
    pub projection_receipt: Value,
    pub disposition: ProjectionDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    severity: &'static str,
    code: &'static str,
    native_key: String,
    detail: String,
}

#[derive(Default)]
struct Findings {
    rows: BTreeSet<Finding>,
}

impl Findings {
    fn incomplete(&mut self, code: &'static str, native_key: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(Finding {
            severity: "incomplete",
            code,
            native_key: native_key.into(),
            detail: detail.into(),
        });
    }

    fn invalid(&mut self, code: &'static str, native_key: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(Finding {
            severity: "invalid_input",
            code,
            native_key: native_key.into(),
            detail: detail.into(),
        });
    }

    fn disposition(&self) -> ProjectionDisposition {
        if self.rows.iter().any(|row| row.severity == "invalid_input") {
            ProjectionDisposition::InvalidInput
        } else if !self.rows.is_empty() {
            ProjectionDisposition::ProjectedWithKnownLoss
        } else {
            ProjectionDisposition::ProjectedWithoutKnownLoss
        }
    }

    fn json(&self) -> Vec<Value> {
        self.rows
            .iter()
            .map(|row| {
                json!({
                    "severity": row.severity,
                    "code": row.code,
                    "native_key": row.native_key,
                    "detail": row.detail
                })
            })
            .collect()
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

fn canonical_json_sha256(value: &Value) -> Result<String, String> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| format!("JSON serialization failed: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn append_identity_field(buffer: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let length =
        u64::try_from(value.len()).map_err(|_| "identity component length exceeds u64".to_owned())?;
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(value.as_bytes());
    Ok(())
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

fn native_identity_sha256(
    node_type: &str,
    components: &BTreeMap<String, String>,
) -> Result<String, String> {
    if components.len() < 2 {
        return Err("native identity requires native_id and a record/revision discriminator".into());
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
        return Err("native identity requires a native record, revision, sequence, or observation discriminator".into());
    }
    if components.keys().any(|name| !valid_component_name(name)) {
        return Err("native identity component name is not a canonical ASCII identifier".into());
    }

    let mut encoded = IDENTITY_DOMAIN.to_vec();
    append_identity_field(&mut encoded, node_type)?;
    append_identity_field(&mut encoded, IDENTITY_NAMESPACE)?;
    for (name, value) in components {
        if value.is_empty() {
            return Err(format!("native identity component {name} is empty"));
        }
        append_identity_field(&mut encoded, name)?;
        append_identity_field(&mut encoded, value)?;
    }
    Ok(sha256_hex(&encoded))
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
    let value_at = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|part| part.parse::<u32>().ok())
    };
    let year = value_at(0, 4).unwrap_or(0);
    let month = value_at(5, 7).unwrap_or(0);
    let day = value_at(8, 10).unwrap_or(0);
    let hour = value_at(11, 13).unwrap_or(99);
    let minute = value_at(14, 16).unwrap_or(99);
    let second = value_at(17, 19).unwrap_or(99);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= days
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era
        - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    if month <= 2 {
        year += 1;
    }
    (year, month as u32, day as u32)
}

fn unix_seconds_to_utc(seconds: i64) -> Option<String> {
    let days = seconds.div_euclid(86_400);
    let within_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    if !(1..=9_999).contains(&year) {
        return None;
    }
    let hour = within_day / 3_600;
    let minute = (within_day % 3_600) / 60;
    let second = within_day % 60;
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

fn string_component(components: &mut BTreeMap<String, String>, name: &str, value: &str) {
    components.insert(name.to_owned(), value.to_owned());
}

fn make_node(
    node_type: &str,
    mut components: BTreeMap<String, String>,
    payload_sha256: &str,
    artifact_media_type: &str,
    schema_version: &str,
    recorded_at: &str,
    artifact_created_at: Option<String>,
    case_id: &str,
    design_id: Option<&str>,
) -> Result<Value, String> {
    if !valid_sha256(payload_sha256) {
        return Err("artifact payload digest is not lowercase SHA-256".into());
    }
    string_component(&mut components, "case_id", case_id);
    let identity_sha = native_identity_sha256(node_type, &components)?;
    let node_id = format!("{node_type}:identity-sha256:{identity_sha}");

    let mut subjects = BTreeSet::new();
    subjects.insert(case_id.to_owned());
    if let Some(design_id) = design_id {
        if !design_id.is_empty() {
            subjects.insert(design_id.to_owned());
        }
    }

    let identity_components = components
        .iter()
        .map(|(name, value)| json!({"name": name, "value": value}))
        .collect::<Vec<_>>();

    Ok(json!({
        "node_id": node_id,
        "node_type": node_type,
        "identity": {
            "identity_namespace": IDENTITY_NAMESPACE,
            "components": identity_components
        },
        "identity_sha256": identity_sha,
        "artifact_sha256": payload_sha256,
        "artifact_media_type": artifact_media_type,
        "schema_version": schema_version,
        "issuer_id": PROJECTOR_ISSUER,
        "scope": {
            "domain": "built_environment",
            "subject_pseudonyms": subjects.into_iter().collect::<Vec<_>>(),
            "purposes": ["engineering_change_impact"]
        },
        "valid_time": {
            "valid_from": null,
            "valid_until": null
        },
        "observation_time": {
            "artifact_created_at": artifact_created_at,
            "recorded_at": recorded_at,
            "trusted_at": null
        },
        "verification": {
            "result": "unknown",
            "policy_sha256": null,
            "verifier_build_sha256": null,
            "trust_bundle_sha256": null,
            "checked_at": null,
            "fresh_until": null
        }
    }))
}

fn add_unmapped(
    findings: &mut Findings,
    unmapped: &mut BTreeMap<String, Value>,
    code: &'static str,
    source_kind: &str,
    source_id: &str,
    relation: &str,
    referenced_id: &str,
    detail: &str,
) {
    let key = format!("{source_kind}:{source_id}:{relation}:{referenced_id}:{code}");
    findings.incomplete(code, key.clone(), detail);
    unmapped.insert(
        key.clone(),
        json!({
            "unmapped_id": key,
            "source_record_kind": source_kind,
            "source_record_id": source_id,
            "relationship_family": relation,
            "referenced_native_id": referenced_id,
            "reason_code": code,
            "detail": detail
        }),
    );
}

fn add_edge(
    edges: &mut BTreeMap<String, Value>,
    edge_mappings: &mut BTreeMap<String, Value>,
    findings: &mut Findings,
    from_node: &str,
    to_node: &str,
    relation: &str,
    native_relationship_key: &str,
    mapping_family: &str,
) {
    let mut bytes = Vec::new();
    for field in [from_node, to_node, relation, native_relationship_key] {
        if append_identity_field(&mut bytes, field).is_err() {
            findings.invalid(
                "EDGE_IDENTITY_ENCODING_FAILED",
                native_relationship_key,
                "edge identity could not be length-prefixed",
            );
            return;
        }
    }
    let edge_id = format!("edge-{}", &sha256_hex(&bytes)[..32]);
    let edge = json!({
        "edge_id": edge_id,
        "from_node_id": from_node,
        "to_node_id": to_node,
        "relation": relation
    });
    if edges.insert(edge_id.clone(), edge).is_some() {
        findings.invalid(
            "DUPLICATE_PROJECTED_EDGE",
            native_relationship_key,
            "two native relationships produced the same projected edge identity",
        );
        return;
    }
    edge_mappings.insert(
        edge_id.clone(),
        json!({
            "edge_id": edge_id,
            "mapping_family": mapping_family,
            "native_relationship_key": native_relationship_key,
            "from_node_id": from_node,
            "to_node_id": to_node,
            "relation": relation,
            "mapping_disposition": "exact_native_relationship"
        }),
    );
}

fn record_payload<T: Serialize>(value: &T) -> Result<(String, String), String> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| format!("record serialization failed: {error}"))?;
    let text = String::from_utf8(bytes.clone())
        .map_err(|error| format!("record serialization was not UTF-8: {error}"))?;
    Ok((sha256_hex(&bytes), text))
}

fn projection_output(
    graph: Value,
    receipt: Value,
    disposition: ProjectionDisposition,
) -> ProjectionOutput {
    ProjectionOutput {
        graph,
        projection_receipt: receipt,
        disposition,
    }
}

/// Project a typed built-environment case into an IEAC v0.2 identity graph.
///
/// The function is read-only. It does not approve artifacts, authenticate
/// source owners, verify engineering receipts, assign trusted timestamps, or
/// grant action authority. Missing source evidence digests and unresolved
/// native relationships are explicit losses and downgrade the result.
pub fn project_built_environment_case(
    case: &BuiltEnvironmentCase,
    context: &ProjectionContext,
) -> ProjectionOutput {
    let mut findings = Findings::default();
    let mut unmapped: BTreeMap<String, Value> = BTreeMap::new();

    if case.case_id.trim().is_empty() {
        findings.invalid("EMPTY_CASE_ID", "case", "case_id is required");
    }
    if case.jurisdiction.trim().is_empty() {
        findings.invalid("EMPTY_CASE_JURISDICTION", "case", "case jurisdiction is required");
    }
    if case.evidence.is_empty()
        && case.artifacts.is_empty()
        && case.engineering_receipts.is_empty()
        && case.decisions.is_empty()
    {
        findings.invalid("EMPTY_CASE_GRAPH", "case", "an empty case cannot produce a dependency graph");
    }
    if !valid_utc_timestamp(&context.captured_at_utc) {
        findings.invalid(
            "INVALID_PROJECTION_TIME",
            "projection_context",
            "captured_at_utc must be a valid canonical UTC timestamp at whole-second precision",
        );
    }
    if !valid_sha256(&context.mapping_profile_sha256) {
        findings.invalid(
            "INVALID_MAPPING_PROFILE_DIGEST",
            "projection_context",
            "mapping_profile_sha256 must be lowercase SHA-256",
        );
    }
    match context.projector_build_sha256.as_deref() {
        Some(value) if valid_sha256(value) => {}
        Some(_) => findings.invalid(
            "INVALID_PROJECTOR_BUILD_DIGEST",
            "projection_context",
            "projector_build_sha256 must be lowercase SHA-256",
        ),
        None => findings.incomplete(
            "PROJECTOR_BUILD_IDENTITY_UNAVAILABLE",
            "projection_context",
            "exact compiled projector build digest was not supplied",
        ),
    }

    let case_bytes = match serde_json::to_vec(case) {
        Ok(bytes) => bytes,
        Err(error) => {
            findings.invalid(
                "CASE_SERIALIZATION_FAILED",
                "case",
                format!("typed case representation could not be serialized: {error}"),
            );
            Vec::new()
        }
    };
    let case_sha256 = sha256_hex(&case_bytes);
    if findings.disposition() == ProjectionDisposition::InvalidInput {
        let disposition = findings.disposition();
        let receipt = json!({
            "schema_version": PROJECTION_SCHEMA_V01,
            "case_id": case.case_id,
            "source_case_representation_sha256": case_sha256,
            "projector_build_sha256": context.projector_build_sha256,
            "mapping_profile_sha256": context.mapping_profile_sha256,
            "disposition": disposition.as_str(),
            "action_authority": "none",
            "findings": findings.json(),
            "unmapped_relationships": []
        });
        return projection_output(Value::Null, receipt, disposition);
    }

    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    let mut node_mappings: BTreeMap<String, Value> = BTreeMap::new();
    let mut projected_payloads: BTreeMap<String, Value> = BTreeMap::new();
    let mut evidence_nodes: BTreeMap<String, String> = BTreeMap::new();
    let mut artifact_nodes: BTreeMap<String, String> = BTreeMap::new();
    let mut receipt_nodes: BTreeMap<String, String> = BTreeMap::new();
    let mut decision_nodes: BTreeMap<String, String> = BTreeMap::new();

    let mut evidence_by_id: BTreeMap<String, &EvidenceRecord> = BTreeMap::new();
    for evidence in &case.evidence {
        if evidence.evidence_id.trim().is_empty() {
            findings.invalid(
                "EMPTY_EVIDENCE_ID",
                "evidence",
                "evidence_id is required",
            );
            continue;
        }
        if evidence_by_id
            .insert(evidence.evidence_id.clone(), evidence)
            .is_some()
        {
            findings.invalid(
                "DUPLICATE_EVIDENCE_ID",
                &evidence.evidence_id,
                "evidence IDs must be unique within a case",
            );
        }
    }

    let mut artifacts_by_id: BTreeMap<String, &ArtifactRecord> = BTreeMap::new();
    for artifact in &case.artifacts {
        if artifact.artifact_id.trim().is_empty() {
            findings.invalid(
                "EMPTY_ARTIFACT_ID",
                "artifact",
                "artifact_id is required",
            );
            continue;
        }
        if artifacts_by_id
            .insert(artifact.artifact_id.clone(), artifact)
            .is_some()
        {
            findings.invalid(
                "DUPLICATE_ARTIFACT_ID",
                &artifact.artifact_id,
                "artifact IDs must be unique within a case",
            );
        }
    }

    let mut receipts_by_id: BTreeMap<String, &EngineeringReceipt> = BTreeMap::new();
    for receipt in &case.engineering_receipts {
        if receipt.receipt_id.trim().is_empty() {
            findings.invalid(
                "EMPTY_RECEIPT_ID",
                "engineering_receipt",
                "receipt_id is required",
            );
            continue;
        }
        if receipts_by_id
            .insert(receipt.receipt_id.clone(), receipt)
            .is_some()
        {
            findings.invalid(
                "DUPLICATE_RECEIPT_ID",
                &receipt.receipt_id,
                "engineering receipt IDs must be unique within a case",
            );
        }
    }

    let mut decisions_by_id: BTreeMap<String, &DecisionRecord> = BTreeMap::new();
    for decision in &case.decisions {
        if decision.event_id.trim().is_empty() {
            findings.invalid(
                "EMPTY_DECISION_ID",
                "decision",
                "event_id is required",
            );
            continue;
        }
        if decisions_by_id
            .insert(decision.event_id.clone(), decision)
            .is_some()
        {
            findings.invalid(
                "DUPLICATE_DECISION_ID",
                &decision.event_id,
                "decision event IDs must be unique within a case",
            );
        }
    }

    // Preserve every typed evidence record as a separate projection payload,
    // including records that cannot become source-payload-addressed graph nodes.
    for evidence in &case.evidence {
        let native_key = format!("evidence:{}", evidence.evidence_id);
        if evidence.jurisdiction != case.jurisdiction {
            findings.invalid(
                "EVIDENCE_CASE_JURISDICTION_MISMATCH",
                &native_key,
                "evidence jurisdiction does not match the built-environment case jurisdiction",
            );
        }
        let (record_sha256, record_bytes) = match record_payload(evidence) {
            Ok(payload) => payload,
            Err(error) => {
                findings.invalid("EVIDENCE_RECORD_SERIALIZATION_FAILED", &native_key, error);
                continue;
            }
        };
        let record_payload_key = format!("case-record:{native_key}");
        let source_payload_sha256 = evidence.source_payload_sha256.clone();
        projected_payloads.insert(
            record_payload_key.clone(),
            json!({
                "record_key": native_key,
                "node_id": null,
                "media_type": "application/json",
                "payload_sha256": record_sha256,
                "payload_digest_basis": "sha256_of_typed_case_record_representation",
                "payload_bytes_utf8": record_bytes,
                "source_payload_sha256": source_payload_sha256,
                "source_payload_bytes_included": false
            }),
        );
        let Some(payload_sha) = evidence.source_payload_sha256.as_deref() else {
            add_unmapped(
                &mut findings,
                &mut unmapped,
                "SOURCE_PAYLOAD_DIGEST_UNAVAILABLE",
                "evidence",
                &evidence.evidence_id,
                "source_payload",
                &evidence.source_id,
                "source evidence has no source_payload_sha256; the typed case record is preserved separately but cannot be represented as a source-payload-addressed graph node",
            );
            continue;
        };
        if !valid_sha256(payload_sha) {
            findings.invalid(
                "INVALID_EVIDENCE_PAYLOAD_DIGEST",
                &native_key,
                "source_payload_sha256 is not lowercase SHA-256",
            );
            continue;
        }
        let mut components = BTreeMap::new();
        string_component(&mut components, "native_id", &evidence.source_id);
        string_component(&mut components, "native_record_id", &evidence.evidence_id);
        string_component(&mut components, "field_path", &evidence.field_path);
        string_component(&mut components, "subject_id", &evidence.subject_id);
        string_component(&mut components, "jurisdiction", &evidence.jurisdiction);
        let observed = evidence.observed_at_unix_seconds.and_then(unix_seconds_to_utc);
        if evidence.observed_at_unix_seconds.is_some() && observed.is_none() {
            findings.incomplete(
                "EVIDENCE_OBSERVATION_TIME_UNREPRESENTABLE",
                &native_key,
                "source observation time is outside the supported UTC timestamp range",
            );
        }
        let node = match make_node(
            "source_evidence",
            components,
            payload_sha,
            "application/octet-stream",
            SOURCE_SCHEMA_UNKNOWN,
            &context.captured_at_utc,
            observed,
            &case.case_id,
            None,
        ) {
            Ok(node) => node,
            Err(error) => {
                findings.invalid("EVIDENCE_IDENTITY_INVALID", &native_key, error);
                continue;
            }
        };
        let node_id = node["node_id"].as_str().unwrap_or_default().to_owned();
        let identity_sha = node["identity_sha256"].as_str().unwrap_or_default().to_owned();
        if nodes.insert(node_id.clone(), node).is_some() {
            findings.invalid(
                "PROJECTED_NODE_ID_COLLISION",
                &native_key,
                "two records produced the same identity node ID",
            );
            continue;
        }
        evidence_nodes.insert(evidence.evidence_id.clone(), node_id.clone());
        node_mappings.insert(
            node_id.clone(),
            json!({
                "record_kind": "evidence",
                "native_key": native_key,
                "native_source_id": evidence.source_id,
                "native_record_id": evidence.evidence_id,
                "node_type": "source_evidence",
                "node_id": node_id,
                "identity_sha256": identity_sha,
                "source_payload_sha256": payload_sha,
                "projected_payload_sha256": payload_sha,
                "payload_digest_basis": "source_payload_sha256",
                "case_record_representation_sha256": record_sha256,
                "source_subject_id": evidence.subject_id,
                "source_field_path": evidence.field_path,
                "source_jurisdiction": evidence.jurisdiction,
                "source_evidence_state": serde_json::to_value(evidence.evidence_state).unwrap_or(Value::Null),
                "source_data_rights": serde_json::to_value(evidence.data_rights).unwrap_or(Value::Null),
                "source_license_id": evidence.license_id,
                "source_observed_at_unix_seconds": evidence.observed_at_unix_seconds,
                "schema_version_claim": "unknown"
            }),
        );
        if let Some(payload) = projected_payloads.get_mut(&record_payload_key) {
            payload["node_id"] = json!(node_id);
        }
    }

    // Artifact nodes use the source artifact content digest unchanged.
    for artifact in &case.artifacts {
        let native_key = format!("artifact:{}", artifact.artifact_id);
        if artifact.jurisdiction != case.jurisdiction {
            findings.invalid(
                "ARTIFACT_CASE_JURISDICTION_MISMATCH",
                &native_key,
                "artifact jurisdiction does not match the built-environment case jurisdiction",
            );
        }
        if !valid_sha256(&artifact.content_sha256) {
            findings.invalid(
                "INVALID_ARTIFACT_CONTENT_DIGEST",
                &native_key,
                "artifact content_sha256 is not lowercase SHA-256",
            );
            continue;
        }
        let mut components = BTreeMap::new();
        string_component(&mut components, "native_id", &artifact.design_id);
        string_component(&mut components, "native_record_id", &artifact.artifact_id);
        string_component(
            &mut components,
            "native_revision",
            &artifact.revision.to_string(),
        );
        string_component(&mut components, "artifact_family", &artifact.family_id);
        string_component(&mut components, "jurisdiction", &artifact.jurisdiction);
        let created = unix_seconds_to_utc(artifact.created_at_unix_seconds);
        if created.is_none() {
            findings.incomplete(
                "ARTIFACT_CREATION_TIME_UNREPRESENTABLE",
                &native_key,
                "artifact creation time is outside the supported UTC timestamp range",
            );
        }
        let node = match make_node(
            "engineering_artifact",
            components,
            &artifact.content_sha256,
            "application/octet-stream",
            SOURCE_SCHEMA_UNKNOWN,
            &context.captured_at_utc,
            created,
            &case.case_id,
            Some(&artifact.design_id),
        ) {
            Ok(node) => node,
            Err(error) => {
                findings.invalid("ARTIFACT_IDENTITY_INVALID", &native_key, error);
                continue;
            }
        };
        let node_id = node["node_id"].as_str().unwrap_or_default().to_owned();
        let identity_sha = node["identity_sha256"].as_str().unwrap_or_default().to_owned();
        if nodes.insert(node_id.clone(), node).is_some() {
            findings.invalid(
                "PROJECTED_NODE_ID_COLLISION",
                &native_key,
                "two records produced the same identity node ID",
            );
            continue;
        }
        artifact_nodes.insert(artifact.artifact_id.clone(), node_id.clone());
        node_mappings.insert(
            node_id.clone(),
            json!({
                "record_kind": "artifact",
                "native_key": native_key,
                "native_design_id": artifact.design_id,
                "native_record_id": artifact.artifact_id,
                "native_family_id": artifact.family_id,
                "native_revision": artifact.revision,
                "source_artifact_kind": serde_json::to_value(artifact.kind).unwrap_or(Value::Null),
                "source_jurisdiction": artifact.jurisdiction,
                "source_created_at_unix_seconds": artifact.created_at_unix_seconds,
                "source_evidence_ids": artifact.evidence_ids,
                "source_parent_artifacts": artifact.parent_artifacts.iter().map(|parent| json!({
                    "artifact_id": parent.artifact_id,
                    "content_sha256": parent.content_sha256
                })).collect::<Vec<_>>(),
                "node_type": "engineering_artifact",
                "node_id": node_id,
                "identity_sha256": identity_sha,
                "source_payload_sha256": artifact.content_sha256,
                "projected_payload_sha256": artifact.content_sha256,
                "payload_digest_basis": "artifact_content_sha256",
            }),
        );
    }

    // Engineering receipts do not currently carry a source payload digest.
    // The node digest therefore identifies the exact projector serialization,
    // and the bytes are retained in projected_payloads for independent replay.
    for receipt in &case.engineering_receipts {
        let native_key = format!("receipt:{}", receipt.receipt_id);
        let (payload_sha, payload_bytes) = match record_payload(receipt) {
            Ok(payload) => payload,
            Err(error) => {
                findings.invalid("RECEIPT_SERIALIZATION_FAILED", &native_key, error);
                continue;
            }
        };
        let mut components = BTreeMap::new();
        string_component(&mut components, "native_id", &receipt.design_id);
        string_component(
            &mut components,
            "native_record_id",
            &receipt.receipt_id,
        );
        let node = match make_node(
            "engineering_receipt",
            components,
            &payload_sha,
            "application/vnd.luminous.engineering-receipt+json",
            "luminous.sol-atlas.engineering-receipt-struct.v0.1",
            &context.captured_at_utc,
            None,
            &case.case_id,
            Some(&receipt.design_id),
        ) {
            Ok(node) => node,
            Err(error) => {
                findings.invalid("RECEIPT_IDENTITY_INVALID", &native_key, error);
                continue;
            }
        };
        let node_id = node["node_id"].as_str().unwrap_or_default().to_owned();
        let identity_sha = node["identity_sha256"].as_str().unwrap_or_default().to_owned();
        if nodes.insert(node_id.clone(), node).is_some() {
            findings.invalid(
                "PROJECTED_NODE_ID_COLLISION",
                &native_key,
                "two records produced the same identity node ID",
            );
            continue;
        }
        receipt_nodes.insert(receipt.receipt_id.clone(), node_id.clone());
        node_mappings.insert(
            node_id.clone(),
            json!({
                "record_kind": "engineering_receipt",
                "native_key": native_key,
                "native_design_id": receipt.design_id,
                "native_record_id": receipt.receipt_id,
                "node_type": "engineering_receipt",
                "node_id": node_id,
                "identity_sha256": identity_sha,
                "source_payload_sha256": null,
                "projected_payload_sha256": payload_sha,
                "payload_digest_basis": "sha256_of_projector_serialized_struct_representation"
            }),
        );
        projected_payloads.insert(
            node_id.clone(),
            json!({
                "node_id": node_id,
                "media_type": "application/vnd.luminous.engineering-receipt+json",
                "payload_sha256": payload_sha,
                "payload_bytes_utf8": payload_bytes
            }),
        );
    }

    // Decisions are events, not endorsements. They are projected as records
    // with unknown verification and no action authority.
    for decision in &case.decisions {
        let native_key = format!("decision:{}", decision.event_id);
        let (payload_sha, payload_bytes) = match record_payload(decision) {
            Ok(payload) => payload,
            Err(error) => {
                findings.invalid("DECISION_SERIALIZATION_FAILED", &native_key, error);
                continue;
            }
        };
        let mut components = BTreeMap::new();
        string_component(&mut components, "native_id", &decision.design_id);
        string_component(
            &mut components,
            "native_record_id",
            &decision.event_id,
        );
        let created = unix_seconds_to_utc(decision.decided_at_unix_seconds);
        if created.is_none() {
            findings.incomplete(
                "DECISION_TIME_UNREPRESENTABLE",
                &native_key,
                "decision timestamp is outside the supported UTC timestamp range",
            );
        }
        let node = match make_node(
            "decision",
            components,
            &payload_sha,
            "application/vnd.luminous.engineering-decision+json",
            "luminous.sol-atlas.engineering-decision-struct.v0.1",
            &context.captured_at_utc,
            created,
            &case.case_id,
            Some(&decision.design_id),
        ) {
            Ok(node) => node,
            Err(error) => {
                findings.invalid("DECISION_IDENTITY_INVALID", &native_key, error);
                continue;
            }
        };
        let node_id = node["node_id"].as_str().unwrap_or_default().to_owned();
        let identity_sha = node["identity_sha256"].as_str().unwrap_or_default().to_owned();
        if nodes.insert(node_id.clone(), node).is_some() {
            findings.invalid(
                "PROJECTED_NODE_ID_COLLISION",
                &native_key,
                "two records produced the same identity node ID",
            );
            continue;
        }
        decision_nodes.insert(decision.event_id.clone(), node_id.clone());
        node_mappings.insert(
            node_id.clone(),
            json!({
                "record_kind": "decision",
                "native_key": native_key,
                "native_design_id": decision.design_id,
                "native_record_id": decision.event_id,
                "node_type": "decision",
                "node_id": node_id,
                "identity_sha256": identity_sha,
                "source_payload_sha256": null,
                "projected_payload_sha256": payload_sha,
                "payload_digest_basis": "sha256_of_projector_serialized_struct_representation"
            }),
        );
        projected_payloads.insert(
            node_id.clone(),
            json!({
                "node_id": node_id,
                "media_type": "application/vnd.luminous.engineering-decision+json",
                "payload_sha256": payload_sha,
                "payload_bytes_utf8": payload_bytes
            }),
        );
    }

    let mut edges: BTreeMap<String, Value> = BTreeMap::new();
    let mut edge_mappings: BTreeMap<String, Value> = BTreeMap::new();

    // Parent artifact relationships: parent -> child (child derives from parent).
    for artifact in &case.artifacts {
        let Some(child_node) = artifact_nodes.get(&artifact.artifact_id) else {
            continue;
        };
        for parent in &artifact.parent_artifacts {
            match artifacts_by_id.get(&parent.artifact_id) {
                Some(parent_record)
                    if parent_record.content_sha256 == parent.content_sha256
                        && artifact_nodes.contains_key(&parent.artifact_id) =>
                {
                    let parent_node = &artifact_nodes[&parent.artifact_id];
                    add_edge(
                        &mut edges,
                        &mut edge_mappings,
                        &mut findings,
                        parent_node,
                        child_node,
                        "derived_from",
                        &format!("artifact:{}:parent:{}", artifact.artifact_id, parent.artifact_id),
                        "artifact_lineage",
                    );
                }
                Some(_) => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "PARENT_ARTIFACT_DIGEST_OR_PROJECTION_MISMATCH",
                    "artifact",
                    &artifact.artifact_id,
                    "artifact_lineage",
                    &parent.artifact_id,
                    "parent reference digest does not match the source artifact or its node was not projected",
                ),
                None => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "PARENT_ARTIFACT_NOT_FOUND",
                    "artifact",
                    &artifact.artifact_id,
                    "artifact_lineage",
                    &parent.artifact_id,
                    "parent artifact reference does not resolve in the case",
                ),
            }
        }

        for evidence_id in &artifact.evidence_ids {
            match evidence_nodes.get(evidence_id) {
                Some(evidence_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    evidence_node,
                    child_node,
                    "evidence_for",
                    &format!("artifact:{}:evidence:{}", artifact.artifact_id, evidence_id),
                    "source_evidence",
                ),
                None => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "ARTIFACT_EVIDENCE_NOT_PROJECTED",
                    "artifact",
                    &artifact.artifact_id,
                    "source_evidence",
                    evidence_id,
                    "referenced evidence is missing or lacks a source payload digest",
                ),
            }
        }
    }

    for receipt in &case.engineering_receipts {
        let Some(receipt_node) = receipt_nodes.get(&receipt.receipt_id) else {
            continue;
        };
        match artifacts_by_id.get(&receipt.input_artifact_id) {
            Some(artifact)
                if artifact.content_sha256 == receipt.input_artifact_sha256
                    && artifact_nodes.contains_key(&receipt.input_artifact_id) =>
            {
                add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    &artifact_nodes[&receipt.input_artifact_id],
                    receipt_node,
                    "verified_by",
                    &format!("receipt:{}:input:{}", receipt.receipt_id, receipt.input_artifact_id),
                    "solver_inputs",
                );
            }
            Some(_) => add_unmapped(
                &mut findings,
                &mut unmapped,
                "RECEIPT_INPUT_DIGEST_OR_PROJECTION_MISMATCH",
                "engineering_receipt",
                &receipt.receipt_id,
                "solver_inputs",
                &receipt.input_artifact_id,
                "receipt input digest does not match the named source artifact or its node was not projected",
            ),
            None => add_unmapped(
                &mut findings,
                &mut unmapped,
                "RECEIPT_INPUT_ARTIFACT_NOT_FOUND",
                "engineering_receipt",
                &receipt.receipt_id,
                "solver_inputs",
                &receipt.input_artifact_id,
                "receipt input artifact does not resolve in the case",
            ),
        }

        for evidence_id in &receipt.assumption_evidence_ids {
            match evidence_nodes.get(evidence_id) {
                Some(evidence_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    evidence_node,
                    receipt_node,
                    "evidence_for",
                    &format!("receipt:{}:assumption:{}", receipt.receipt_id, evidence_id),
                    "solver_assumptions",
                ),
                None => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "RECEIPT_ASSUMPTION_EVIDENCE_NOT_PROJECTED",
                    "engineering_receipt",
                    &receipt.receipt_id,
                    "solver_assumptions",
                    evidence_id,
                    "assumption evidence is missing or lacks a source payload digest",
                ),
            }
        }
    }

    for decision in &case.decisions {
        let Some(decision_node) = decision_nodes.get(&decision.event_id) else {
            continue;
        };
        match artifacts_by_id.get(&decision.target_artifact_id) {
            Some(artifact)
                if artifact.content_sha256 == decision.target_artifact_sha256
                    && artifact_nodes.contains_key(&decision.target_artifact_id) =>
            {
                add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    &artifact_nodes[&decision.target_artifact_id],
                    decision_node,
                    "evidence_for",
                    &format!("decision:{}:target:{}", decision.event_id, decision.target_artifact_id),
                    "decision_target",
                );
            }
            Some(_) => add_unmapped(
                &mut findings,
                &mut unmapped,
                "DECISION_TARGET_DIGEST_OR_PROJECTION_MISMATCH",
                "decision",
                &decision.event_id,
                "decision_target",
                &decision.target_artifact_id,
                "decision target digest does not match the named source artifact or its node was not projected",
            ),
            None => add_unmapped(
                &mut findings,
                &mut unmapped,
                "DECISION_TARGET_ARTIFACT_NOT_FOUND",
                "decision",
                &decision.event_id,
                "decision_target",
                &decision.target_artifact_id,
                "decision target artifact does not resolve in the case",
            ),
        }

        for receipt_id in &decision.engineering_receipt_ids {
            match receipt_nodes.get(receipt_id) {
                Some(receipt_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    receipt_node,
                    decision_node,
                    "evidence_for",
                    &format!("decision:{}:receipt:{}", decision.event_id, receipt_id),
                    "decision_receipt",
                ),
                None => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "DECISION_RECEIPT_NOT_FOUND",
                    "decision",
                    &decision.event_id,
                    "decision_receipt",
                    receipt_id,
                    "decision references an engineering receipt that does not resolve",
                ),
            }
        }

        if let Some(authority_id) = &decision.authority_evidence_id {
            match evidence_nodes.get(authority_id) {
                Some(authority_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    authority_node,
                    decision_node,
                    "authorized_by",
                    &format!("decision:{}:authority:{}", decision.event_id, authority_id),
                    "decision_authority",
                ),
                None => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "DECISION_AUTHORITY_EVIDENCE_NOT_PROJECTED",
                    "decision",
                    &decision.event_id,
                    "decision_authority",
                    authority_id,
                    "authority evidence is missing or lacks a source payload digest",
                ),
            }
        } else {
            add_unmapped(
                &mut findings,
                &mut unmapped,
                "DECISION_AUTHORITY_EVIDENCE_NOT_DECLARED",
                "decision",
                &decision.event_id,
                "decision_authority",
                "missing",
                "decision record has no authority_evidence_id; the projector cannot infer an authority relationship",
            );
        }

        if let Some(superseded_id) = &decision.supersedes_event_id {
            match decision_nodes.get(superseded_id) {
                Some(previous_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut findings,
                    previous_node,
                    decision_node,
                    "superseded_by",
                    &format!("decision:{}:supersedes:{}", decision.event_id, superseded_id),
                    "decision_supersession",
                ),
                None => add_unmapped(
                    &mut findings,
                    &mut unmapped,
                    "SUPERSEDED_DECISION_NOT_FOUND",
                    "decision",
                    &decision.event_id,
                    "decision_supersession",
                    superseded_id,
                    "superseded decision event does not resolve in the case",
                ),
            }
        }
    }

    // Source maps are deterministic and preserve every native record key.
    let mut node_rows = nodes.into_values().collect::<Vec<_>>();
    node_rows.sort_by(|left, right| {
        left["node_id"]
            .as_str()
            .cmp(&right["node_id"].as_str())
    });
    let mut edge_rows = edges.into_values().collect::<Vec<_>>();
    edge_rows.sort_by(|left, right| {
        left["edge_id"]
            .as_str()
            .cmp(&right["edge_id"].as_str())
    });

    let disposition = findings.disposition();
    let claim_ceiling = if disposition == ProjectionDisposition::ProjectedWithoutKnownLoss {
        "record_only"
    } else {
        "projected_with_known_loss"
    };
    let graph_id = format!("built-environment-case-{}", &case_sha256[..24]);
    let mut graph = json!({
        "schema_version": GRAPH_SCHEMA_V02,
        "graph_id": graph_id,
        "captured_at": context.captured_at_utc,
        "graph_snapshot_sha256": null,
        "nodes": node_rows,
        "edges": edge_rows,
        "claim_ceiling": claim_ceiling,
        "action_authority": "none"
    });
    let mut graph_digest_input = graph.clone();
    graph_digest_input
        .as_object_mut()
        .expect("projected graph is an object")
        .remove("graph_snapshot_sha256");
    let graph_digest = match canonical_json_sha256(&graph_digest_input) {
        Ok(digest) => digest,
        Err(error) => {
            findings.invalid(
                "GRAPH_DIGEST_FAILED",
                "projected_graph",
                error,
            );
            String::new()
        }
    };
    graph["graph_snapshot_sha256"] = json!(graph_digest);

    let node_mapping_rows = node_mappings.into_values().collect::<Vec<_>>();
    let edge_mapping_rows = edge_mappings.into_values().collect::<Vec<_>>();
    let unmapped_rows = unmapped.into_values().collect::<Vec<_>>();
    let payload_rows = projected_payloads.into_values().collect::<Vec<_>>();
    let final_disposition = findings.disposition();

    let mut receipt = json!({
        "schema_version": PROJECTION_SCHEMA_V01,
        "case_id": case.case_id,
        "source_case_representation": {
            "media_type": "application/json",
            "serialization_profile": "serde_json_struct_field_order_v1",
            "sha256": case_sha256
        },
        "projector": {
            "id": "sol-atlas-built-environment-ieac-projector",
            "version": env!("CARGO_PKG_VERSION"),
            "build_sha256": context.projector_build_sha256,
            "mapping_profile_sha256": context.mapping_profile_sha256,
            "identity_namespace": IDENTITY_NAMESPACE
        },
        "projected_graph": {
            "schema_version": GRAPH_SCHEMA_V02,
            "graph_id": graph_id,
            "graph_snapshot_sha256": graph_digest,
            "node_count": graph["nodes"].as_array().map_or(0, Vec::len),
            "edge_count": graph["edges"].as_array().map_or(0, Vec::len)
        },
        "node_mappings": node_mapping_rows,
        "edge_mappings": edge_mapping_rows,
        "projected_payloads": payload_rows,
        "unmapped_relationships": unmapped_rows,
        "findings": findings.json(),
        "disposition": final_disposition.as_str(),
        "claim_ceiling": if final_disposition == ProjectionDisposition::ProjectedWithoutKnownLoss {
            "record_only"
        } else {
            "projected_with_known_loss"
        },
        "action_authority": "none",
        "verification_claim": "none",
        "limitations": [
            "Projection is not source authentication or engineering verification.",
            "The compiled build digest is caller-supplied and must be checked against an independently retained build artifact.",
            "Evidence without a source payload digest is not projected as a source-authenticated evidence node.",
            "Artifact media type and payload schema remain explicitly unknown because the source case model does not carry them.",
            "Engineering receipt and decision payload digests identify exact projector-generated struct serializations, not source-native payload bytes.",
            "The source case representation digest is over this typed Rust serialization, not a raw upstream source export."
        ]
    });
    if let Ok(receipt_digest) = canonical_json_sha256(&receipt) {
        receipt["projection_receipt_sha256"] = json!(receipt_digest);
    }

    projection_output(graph, receipt, final_disposition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_environment_case::{
        ArtifactKind, ArtifactReference, DataRights, DecisionDisposition, EngineeringConclusion,
        EngineeringDiscipline, EvidenceState, ReviewLevel, SolverRunStatus,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn sample_case() -> BuiltEnvironmentCase {
        let evidence = vec![
            EvidenceRecord {
                evidence_id: "evidence-design-source".into(),
                subject_id: "design-001".into(),
                field_path: "geometry.revision".into(),
                source_id: "source-registry-01".into(),
                source_payload_sha256: Some(digest('a')),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_000),
            },
            EvidenceRecord {
                evidence_id: "evidence-authority".into(),
                subject_id: "decision-001".into(),
                field_path: "authority.approval".into(),
                source_id: "authority-registry-01".into(),
                source_payload_sha256: Some(digest('a')),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_001),
            },
        ];
        let artifacts = vec![
            ArtifactRecord {
                artifact_id: "artifact-base".into(),
                design_id: "design-001".into(),
                family_id: "design-model".into(),
                revision: 1,
                kind: ArtifactKind::DesignModel,
                content_sha256: digest('b'),
                parent_artifacts: vec![],
                evidence_ids: vec!["evidence-design-source".into()],
                jurisdiction: "synthetic-jurisdiction".into(),
                created_at_unix_seconds: 1_762_000_100,
            },
            ArtifactRecord {
                artifact_id: "artifact-revision-2".into(),
                design_id: "design-001".into(),
                family_id: "design-model".into(),
                revision: 2,
                kind: ArtifactKind::DesignModel,
                content_sha256: digest('c'),
                parent_artifacts: vec![ArtifactReference {
                    artifact_id: "artifact-base".into(),
                    content_sha256: digest('b'),
                }],
                evidence_ids: vec!["evidence-design-source".into()],
                jurisdiction: "synthetic-jurisdiction".into(),
                created_at_unix_seconds: 1_762_000_200,
            },
        ];
        let receipts = vec![EngineeringReceipt {
            receipt_id: "receipt-structural-001".into(),
            design_id: "design-001".into(),
            input_artifact_id: "artifact-revision-2".into(),
            input_artifact_sha256: digest('c'),
            discipline: EngineeringDiscipline::Structural,
            solver_id: Some("solver-frame-v1".into()),
            solver_version: Some("1.2.3".into()),
            run_status: SolverRunStatus::Completed,
            conclusion: EngineeringConclusion::PassWithinDeclaredScope,
            review_level: ReviewLevel::IndependentlyChecked,
            reviewer_id: Some("reviewer-01".into()),
            assumption_evidence_ids: vec!["evidence-design-source".into()],
            declared_scope: "synthetic frame analysis".into(),
        }];
        let decisions = vec![DecisionRecord {
            event_id: "decision-001".into(),
            design_id: "design-001".into(),
            target_artifact_id: "artifact-revision-2".into(),
            target_artifact_sha256: digest('c'),
            disposition: DecisionDisposition::Approve,
            actor_id: "actor-authority-01".into(),
            authority_evidence_id: Some("evidence-authority".into()),
            engineering_receipt_ids: vec!["receipt-structural-001".into()],
            decided_at_unix_seconds: 1_762_000_300,
            conditions: vec!["synthetic-only".into()],
            supersedes_event_id: None,
        }];
        BuiltEnvironmentCase {
            case_id: "case-pavilion-001".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence,
            artifacts,
            engineering_receipts: receipts,
            decisions,
        }
    }

    fn context() -> ProjectionContext {
        ProjectionContext {
            captured_at_utc: "2026-11-03T12:00:00Z".into(),
            projector_build_sha256: Some(digest('d')),
            mapping_profile_sha256: digest('e'),
        }
    }

    #[test]
    fn same_payload_does_not_collapse_distinct_source_evidence_identities() {
        let result = project_built_environment_case(&sample_case(), &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithoutKnownLoss);
        let evidence_nodes = result.graph["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| node["node_type"] == "source_evidence")
            .collect::<Vec<_>>();
        assert_eq!(evidence_nodes.len(), 2);
        assert_eq!(
            evidence_nodes[0]["artifact_sha256"],
            evidence_nodes[1]["artifact_sha256"]
        );
        assert_ne!(evidence_nodes[0]["identity_sha256"], evidence_nodes[1]["identity_sha256"]);
        assert_ne!(evidence_nodes[0]["node_id"], evidence_nodes[1]["node_id"]);
    }

    #[test]
    fn projector_retains_parent_evidence_receipt_authority_and_decision_edges() {
        let result = project_built_environment_case(&sample_case(), &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithoutKnownLoss);
        let edges = result.graph["edges"].as_array().unwrap();
        for relation in [
            "derived_from",
            "evidence_for",
            "verified_by",
            "authorized_by",
        ] {
            assert!(edges.iter().any(|edge| edge["relation"] == relation), "missing {relation}");
        }
        assert!(result.graph["nodes"].as_array().unwrap().iter().all(|node| {
            node["verification"]["result"] == "unknown"
                && node["observation_time"]["trusted_at"].is_null()
        }));
        assert_eq!(result.graph["action_authority"], "none");
        assert_eq!(result.graph["claim_ceiling"], "record_only");
    }

    #[test]
    fn missing_source_evidence_digest_is_visible_and_downgrades_projection() {
        let mut case = sample_case();
        case.evidence[0].source_payload_sha256 = None;
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
        assert!(
            result.projection_receipt["unmapped_relationships"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["reason_code"] == "SOURCE_PAYLOAD_DIGEST_UNAVAILABLE")
        );
        assert_eq!(
            result.graph["claim_ceiling"],
            "projected_with_known_loss"
        );
    }

    #[test]
    fn mismatched_parent_or_receipt_input_digest_is_not_silently_mapped() {
        let mut case = sample_case();
        case.artifacts[1].parent_artifacts[0].content_sha256 = digest('f');
        case.engineering_receipts[0].input_artifact_sha256 = digest('f');
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
        let unmapped = result.projection_receipt["unmapped_relationships"]
            .as_array()
            .unwrap();
        assert!(unmapped.iter().any(|row| {
            row["reason_code"] == "PARENT_ARTIFACT_DIGEST_OR_PROJECTION_MISMATCH"
        }));
        assert!(unmapped.iter().any(|row| {
            row["reason_code"] == "RECEIPT_INPUT_DIGEST_OR_PROJECTION_MISMATCH"
        }));
    }

    #[test]
    fn output_is_deterministic_under_source_vector_reordering() {
        let case = sample_case();
        let mut reordered = case.clone();
        reordered.evidence.reverse();
        reordered.artifacts.reverse();
        reordered.engineering_receipts.reverse();
        reordered.decisions.reverse();
        let first = project_built_environment_case(&case, &context());
        let second = project_built_environment_case(&reordered, &context());
        assert_eq!(
            first.graph["graph_snapshot_sha256"],
            second.graph["graph_snapshot_sha256"]
        );
        assert_eq!(
            first.projection_receipt["projected_graph"]["graph_snapshot_sha256"],
            second.projection_receipt["projected_graph"]["graph_snapshot_sha256"]
        );
        assert_ne!(
            first.projection_receipt["source_case_representation"]["sha256"],
            second.projection_receipt["source_case_representation"]["sha256"],
            "the receipt binds exact typed input bytes, including input vector order"
        );
        assert_ne!(
            first.projection_receipt["projection_receipt_sha256"],
            second.projection_receipt["projection_receipt_sha256"],
            "reordered source input has a different exact-input receipt"
        );
    }

    #[test]
    fn evidence_mapping_preserves_status_rights_license_and_case_record_bytes() {
        let result = project_built_environment_case(&sample_case(), &context());
        let mappings = result.projection_receipt["node_mappings"]
            .as_array()
            .unwrap();
        let evidence_mapping = mappings
            .iter()
            .find(|row| row["record_kind"] == "evidence")
            .unwrap();
        assert_eq!(evidence_mapping["source_evidence_state"], "verified");
        assert_eq!(evidence_mapping["source_data_rights"], "public_domain");
        assert_eq!(evidence_mapping["source_license_id"], "CC0-1.0");
        assert!(valid_sha256(
            evidence_mapping["case_record_representation_sha256"]
                .as_str()
                .unwrap()
        ));
        let payloads = result.projection_receipt["projected_payloads"]
            .as_array()
            .unwrap();
        assert!(payloads.iter().any(|row| {
            row["record_key"] == "evidence:evidence-design-source"
                && row["payload_digest_basis"] == "sha256_of_typed_case_record_representation"
                && row["source_payload_bytes_included"] == false
        }));
    }

    #[test]
    fn missing_evidence_digest_still_preserves_the_typed_record_representation() {
        let mut case = sample_case();
        case.evidence[0].source_payload_sha256 = None;
        let result = project_built_environment_case(&case, &context());
        let payloads = result.projection_receipt["projected_payloads"]
            .as_array()
            .unwrap();
        assert!(payloads.iter().any(|row| {
            row["record_key"] == "evidence:evidence-design-source"
                && row["node_id"].is_null()
                && row["source_payload_sha256"].is_null()
                && row["payload_bytes_utf8"]
                    .as_str()
                    .is_some_and(|bytes| bytes.contains("evidence-design-source"))
        }));
    }

    #[test]
    fn empty_case_cannot_emit_a_valid_empty_ieac_graph() {
        let case = BuiltEnvironmentCase {
            case_id: "case-empty".into(),
            jurisdiction: "synthetic-jurisdiction".into(),
            evidence: vec![],
            artifacts: vec![],
            engineering_receipts: vec![],
            decisions: vec![],
        };
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::InvalidInput);
        assert_eq!(result.graph, Value::Null);
        assert!(result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "EMPTY_CASE_GRAPH"));
    }

    #[test]
    fn missing_decision_authority_is_explicitly_incomplete() {
        let mut case = sample_case();
        case.decisions[0].authority_evidence_id = None;
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
        assert!(result.projection_receipt["unmapped_relationships"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["reason_code"] == "DECISION_AUTHORITY_EVIDENCE_NOT_DECLARED"));
    }

    #[test]
    fn unix_timestamp_conversion_is_exact_at_epoch_and_leap_day() {
        assert_eq!(
            unix_seconds_to_utc(0).as_deref(),
            Some("1970-01-01T00:00:00Z")
        );
        let leap_day = 1_709_164_800; // 2024-02-29T00:00:00Z
        assert_eq!(
            unix_seconds_to_utc(leap_day).as_deref(),
            Some("2024-02-29T00:00:00Z")
        );
    }
}
