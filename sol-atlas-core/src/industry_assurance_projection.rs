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
use crate::built_environment_source_descriptor::{
    validate_source_descriptor_catalog, DescriptorDisposition, DescriptorValidation,
    NativeIdentityComponent, SourceDescriptorCatalog,
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
    /// Source-owner namespaces keyed by evidence:<id>, artifact:<id>,
    /// engineering_receipt:<id>, or decision:<id>. Missing keys use a
    /// visibly incomplete adapter-scoped fallback.
    pub native_identity_namespaces: BTreeMap<String, String>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRecordProjectionMetadata {
    pub media_type: Option<String>,
    pub schema_id: Option<String>,
    pub schema_version: Option<String>,
    pub schema_sha256: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DescriptorProjectionOutput {
    pub descriptor_validation: DescriptorValidation,
    pub projection: ProjectionOutput,
}

struct ProjectionOverrides {
    catalog_sha256: String,
    validation: DescriptorValidation,
    identity_components: BTreeMap<String, BTreeMap<String, String>>,
    source_metadata: BTreeMap<String, SourceRecordProjectionMetadata>,
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
            severity: "invalid",
            code,
            native_key: native_key.into(),
            detail: detail.into(),
        });
    }

    fn disposition(&self) -> ProjectionDisposition {
        if self.rows.iter().any(|row| row.severity == "invalid") {
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
    identity_namespace: &str,
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

    if identity_namespace.trim().is_empty() || identity_namespace.len() > 1024 {
        return Err("native identity namespace must be non-empty and at most 1024 UTF-8 bytes".into());
    }
    let mut encoded = IDENTITY_DOMAIN.to_vec();
    append_identity_field(&mut encoded, node_type)?;
    append_identity_field(&mut encoded, identity_namespace)?;
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
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
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

fn namespace_map_digest(
    namespaces: &BTreeMap<String, String>,
) -> Result<String, String> {
    let bytes = serde_json::to_vec(namespaces)
        .map_err(|error| format!("namespace map serialization failed: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn valid_native_identity_namespace(namespace: &str) -> bool {
    !namespace.trim().is_empty()
        && namespace == namespace.trim()
        && namespace.len() <= 1_024
}

fn resolve_native_identity_namespace(
    record_key: &str,
    context: &ProjectionContext,
    findings: &mut Findings,
) -> String {
    match context.native_identity_namespaces.get(record_key) {
        Some(namespace) if valid_native_identity_namespace(namespace) => namespace.clone()
        Some(_) => {
            findings.invalid(
                "INVALID_NATIVE_IDENTITY_NAMESPACE",
                record_key,
                "native identity namespace must be non-empty, trimmed, and no longer than 1024 UTF-8 bytes",
            );
            IDENTITY_NAMESPACE.to_owned()
        }
        None => {
            findings.incomplete(
                "NATIVE_IDENTITY_NAMESPACE_NOT_SUPPLIED",
                record_key,
                "no owner-native namespace was supplied for this record; adapter-scoped fallback identity is used and cannot be treated as source-owner identity",
            );
            IDENTITY_NAMESPACE.to_owned()
        }
    }
}

struct NodeProjection<'a> {
    node_type: &'a str,
    identity_namespace: &'a str,
    components: BTreeMap<String, String>,
    payload_sha256: &'a str,
    artifact_media_type: &'a str,
    schema_version: &'a str,
    recorded_at: &'a str,
    artifact_created_at: Option<String>,
    case_id: &'a str,
    design_id: Option<&'a str>,
}

fn make_node(spec: NodeProjection<'_>) -> Result<Value, String> {
    if !valid_sha256(spec.payload_sha256) {
        return Err("artifact payload digest is not lowercase SHA-256".into());
    }
    let mut components = spec.components;
    string_component(&mut components, "case_id", spec.case_id);
    let identity_sha = native_identity_sha256(
        spec.node_type,
        spec.identity_namespace,
        &components,
    )?;
    let node_id = format!("{}:identity-sha256:{identity_sha}", spec.node_type);

    let mut subjects = BTreeSet::new();
    subjects.insert(spec.case_id.to_owned());
    if let Some(design_id) = spec.design_id.filter(|value| !value.is_empty()) {
        subjects.insert(design_id.to_owned());
    }

    let identity_components = components
        .iter()
        .map(|(name, value)| json!({"name": name, "value": value}))
        .collect::<Vec<_>>();

    Ok(json!({
        "node_id": node_id,
        "node_type": spec.node_type,
        "identity": {
            "identity_namespace": spec.identity_namespace,
            "components": identity_components
        },
        "identity_sha256": identity_sha,
        "artifact_sha256": spec.payload_sha256,
        "artifact_media_type": spec.artifact_media_type,
        "schema_version": spec.schema_version,
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
            "artifact_created_at": spec.artifact_created_at,
            "recorded_at": spec.recorded_at,
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
    spec: UnmappedRelation<'_>,
) {
    let key = format!(
        "{}:{}:{}:{}:{}",
        spec.source_kind, spec.source_id, spec.relation, spec.referenced_id, spec.code
    );
    findings.incomplete(spec.code, key.clone(), spec.detail);
    unmapped.insert(
        key.clone(),
        json!({
            "unmapped_id": key,
            "source_record_kind": spec.source_kind,
            "source_record_id": spec.source_id,
            "relationship_family": spec.relation,
            "referenced_native_id": spec.referenced_id,
            "reason_code": spec.code,
            "detail": spec.detail
        }),
    );
}

struct EdgeProjection<'a> {
    from_node: &'a str,
    to_node: &'a str,
    relation: &'a str,
    native_relationship_key: &'a str,
    mapping_family: &'a str,
}

fn add_edge(
    edges: &mut BTreeMap<String, Value>,
    edge_mappings: &mut BTreeMap<String, Value>,
    edge_pairs: &mut BTreeSet<(String, String)>,
    findings: &mut Findings,
    spec: EdgeProjection<'_>,
) {
    let mut bytes = Vec::new();
    for field in [
        spec.from_node,
        spec.to_node,
        spec.relation,
        spec.native_relationship_key,
    ] {
        if append_identity_field(&mut bytes, field).is_err() {
            findings.invalid(
                "EDGE_IDENTITY_ENCODING_FAILED",
                spec.native_relationship_key,
                "edge identity could not be length-prefixed",
            );
            return;
        }
    }
    if !edge_pairs.insert((spec.from_node.to_owned(), spec.to_node.to_owned())) {
        findings.invalid(
            "DUPLICATE_PROJECTED_EDGE_ENDPOINT_PAIR",
            spec.native_relationship_key,
            "IEAC graph cannot represent two directed edges with the same endpoint pair",
        );
        return;
    }
    let edge_id = format!("edge-{}", &sha256_hex(&bytes)[..32]);
    let edge = json!({
        "edge_id": edge_id,
        "from_node_id": spec.from_node,
        "to_node_id": spec.to_node,
        "relation": spec.relation
    });
    if edges.insert(edge_id.clone(), edge).is_some() {
        findings.invalid(
            "DUPLICATE_PROJECTED_EDGE",
            spec.native_relationship_key,
            "two native relationships produced the same projected edge identity",
        );
        return;
    }
    edge_mappings.insert(
        edge_id.clone(),
        json!({
            "edge_id": edge_id,
            "mapping_family": spec.mapping_family,
            "native_relationship_key": spec.native_relationship_key,
            "from_node_id": spec.from_node,
            "to_node_id": spec.to_node,
            "relation": spec.relation,
            "mapping_disposition": "exact_native_relationship"
        }),
    );
}

fn has_dependency_cycle(
    nodes: &BTreeMap<String, Value>,
    edges: &BTreeMap<String, Value>,
) -> bool {
    let mut indegree: BTreeMap<String, usize> =
        nodes.keys().map(|node_id| (node_id.clone(), 0)).collect();
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for edge in edges.values() {
        let Some(from) = edge.get("from_node_id").and_then(Value::as_str) else {
            return true;
        };
        let Some(to) = edge.get("to_node_id").and_then(Value::as_str) else {
            return true;
        };
        if !nodes.contains_key(from) || !nodes.contains_key(to) {
            return true;
        }
        adjacency.entry(from.to_owned()).or_default().push(to.to_owned());
        let Some(degree) = indegree.get_mut(to) else {
            return true;
        };
        *degree += 1;
    }

    for destinations in adjacency.values_mut() {
        destinations.sort();
        destinations.dedup();
    }
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter_map(|(node_id, degree)| (*degree == 0).then_some(node_id.clone()))
        .collect();
    let mut visited = 0usize;
    while let Some(node_id) = ready.pop_first() {
        visited += 1;
        if let Some(destinations) = adjacency.get(&node_id) {
            for destination in destinations {
                let Some(degree) = indegree.get_mut(destination) else {
                    return true;
                };
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(destination.clone());
                }
            }
        }
    }
    visited != nodes.len()
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
    project_built_environment_case_inner(case, context, None)
}

pub fn project_built_environment_case_with_descriptors(
    case: &BuiltEnvironmentCase,
    context: &ProjectionContext,
    catalog: &SourceDescriptorCatalog,
) -> DescriptorProjectionOutput {
    let validation = validate_source_descriptor_catalog(case, catalog);
    let catalog_sha256 = serde_json::to_vec(catalog)
        .map(|bytes| sha256_hex(&bytes))
        .unwrap_or_default();

    if validation.disposition == DescriptorDisposition::InvalidInput {
        let case_bytes = serde_json::to_vec(case).unwrap_or_default();
        let case_sha256 = sha256_hex(&case_bytes);
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
                "adapter_fallback_identity_namespace": IDENTITY_NAMESPACE,
                "identity_namespace_basis": "caller_supplied_map_with_adapter_fallback",
                "native_identity_namespaces": context.native_identity_namespaces,
                "native_identity_namespace_map_sha256": namespace_map_digest(&context.native_identity_namespaces).unwrap_or_default(),
                "native_identity_namespace_entries_supplied": context.native_identity_namespaces.len(),
                "native_identity_namespace_entries_expected": case.evidence.len()
                    + case.artifacts.len()
                    + case.engineering_receipts.len()
                    + case.decisions.len(),
                "source_owner_namespace_entries_supplied_for_all_records": false,
                "source_owner_namespace_authenticated": false,
                "source_descriptor_catalog": {
                    "catalog_sha256": catalog_sha256,
                    "validation_disposition": validation.disposition.as_str(),
                    "descriptor_count": validation.descriptor_count,
                    "missing_descriptor_count": validation.missing_descriptor_count,
                    "extra_descriptor_count": validation.extra_descriptor_count,
                    "payload_byte_verification": validation.payload_byte_verification,
                    "rights_authentication": validation.rights_authentication,
                    "source_attestation_authentication": validation.source_attestation_authentication,
                    "action_authority": "none",
                    "findings": validation.findings
                }
            },
            "projected_graph": null,
            "node_mappings": [],
            "edge_mappings": [],
            "projected_payloads": [],
            "unmapped_relationships": [],
            "findings": [{
                "severity": "invalid",
                "code": "SOURCE_DESCRIPTOR_CATALOG_INVALID",
                "native_key": "source_descriptor_catalog",
                "detail": "invalid descriptor catalogue cannot supply projection identity or source metadata"
            }],
            "disposition": "invalid_input",
            "claim_ceiling": "incomplete_or_indeterminate",
            "action_authority": "none",
            "verification_claim": "none",
            "limitations": [
                "Invalid source descriptor catalogue was not used for projection.",
                "The descriptor validator does not authenticate source-owner identities, source bytes, rights, or attestations."
            ]
        });
        if let Ok(receipt_digest) = canonical_json_sha256(&receipt) {
            receipt["projection_receipt_sha256"] = json!(receipt_digest);
        }
        return DescriptorProjectionOutput {
            descriptor_validation: validation,
            projection: projection_output(
                Value::Null,
                receipt,
                ProjectionDisposition::InvalidInput,
            ),
        };
    }

    let mut projection_context = context.clone();
    for (record_key, namespace) in &validation.native_identity_namespaces {
        projection_context
            .native_identity_namespaces
            .insert(record_key.clone(), namespace.clone());
    }

    let identity_components = validation
        .native_identity_components
        .iter()
        .map(|(record_key, components)| {
            (
                record_key.clone(),
                components
                    .iter()
                    .map(|component| (component.name.clone(), component.value.clone()))
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let source_metadata = catalog
        .descriptors
        .iter()
        .map(|descriptor| {
            let key = descriptor
                .record_kind
                .case_key(&descriptor.case_record_id);
            (
                key,
                SourceRecordProjectionMetadata {
                    media_type: descriptor.source_payload.media_type.clone(),
                    schema_id: descriptor.source_payload.schema_id.clone(),
                    schema_version: descriptor.source_payload.schema_version.clone(),
                    schema_sha256: descriptor.source_payload.schema_sha256.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let overrides = ProjectionOverrides {
        catalog_sha256,
        validation: validation.clone(),
        identity_components,
        source_metadata,
    };
    let projection =
        project_built_environment_case_inner(case, &projection_context, Some(&overrides));
    DescriptorProjectionOutput {
        descriptor_validation: validation,
        projection,
    }
}

fn identity_components_for_record(
    native_key: &str,
    fallback: BTreeMap<String, String>,
    overrides: Option<&ProjectionOverrides>,
) -> BTreeMap<String, String> {
    overrides
        .and_then(|value| value.identity_components.get(native_key))
        .cloned()
        .unwrap_or(fallback)
}

fn source_metadata_for_record<'a>(
    native_key: &str,
    overrides: Option<&'a ProjectionOverrides>,
) -> Option<&'a SourceRecordProjectionMetadata> {
    overrides.and_then(|value| value.source_metadata.get(native_key))
}

fn project_built_environment_case_inner(
    case: &BuiltEnvironmentCase,
    context: &ProjectionContext,
    descriptor_overrides: Option<&ProjectionOverrides>,
) -> ProjectionOutput {
    let mut findings = Findings::default();
    let mut unmapped: BTreeMap<String, Value> = BTreeMap::new();
    if let Some(overrides) = descriptor_overrides {
        if overrides.validation.disposition == DescriptorDisposition::IncompleteOrIndeterminate {
            findings.incomplete(
                "SOURCE_DESCRIPTOR_CATALOG_INCOMPLETE",
                "source_descriptor_catalog",
                "descriptor validation found missing or unverified source metadata; supplied mappings remain bounded by its incomplete disposition",
            );
        }
    }

    if case.case_id.trim().is_empty() {
        findings.invalid("EMPTY_CASE_ID", "case", "case_id is required");
    }
    if case.jurisdiction.trim().is_empty() {
        findings.invalid("EMPTY_CASE_JURISDICTION", "case", "case jurisdiction is required");
    }
    let expected_namespace_keys = case
        .evidence
        .iter()
        .map(|record| format!("evidence:{}", record.evidence_id))
        .chain(case.artifacts.iter().map(|record| format!("artifact:{}", record.artifact_id)))
        .chain(case.engineering_receipts.iter().map(|record| format!("engineering_receipt:{}", record.receipt_id)))
        .chain(case.decisions.iter().map(|record| format!("decision:{}", record.event_id)))
        .collect::<BTreeSet<_>>();
    for supplied_key in context.native_identity_namespaces.keys() {
        if !expected_namespace_keys.contains(supplied_key) {
            findings.incomplete(
                "UNUSED_NATIVE_IDENTITY_NAMESPACE",
                supplied_key,
                "identity namespace map contains a key that does not identify a record in this case",
            );
        }
    }
    for record_key in &expected_namespace_keys {
        let _ = resolve_native_identity_namespace(record_key, context, &mut findings);
    }
    let supplied_native_identity_namespace_count = expected_namespace_keys
        .iter()
        .filter(|record_key| {
            context
                .native_identity_namespaces
                .get(*record_key)
                .is_some_and(|namespace| valid_native_identity_namespace(namespace))
        })
        .count();
    let all_native_identity_namespaces_supplied =
        supplied_native_identity_namespace_count == expected_namespace_keys.len();
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
                "adapter_fallback_identity_namespace": IDENTITY_NAMESPACE,
                "identity_namespace_basis": if all_native_identity_namespaces_supplied {
                    "caller_supplied_per_record_namespace_map_not_authenticated"
                } else {
                    "caller_supplied_map_with_adapter_fallback"
                },
                "native_identity_namespaces": context.native_identity_namespaces,
                "native_identity_namespace_map_sha256": namespace_map_digest(&context.native_identity_namespaces).unwrap_or_default(),
                "native_identity_namespace_entries_supplied": supplied_native_identity_namespace_count,
                "native_identity_namespace_entries_expected": expected_namespace_keys.len(),
                "source_owner_namespace_entries_supplied_for_all_records": all_native_identity_namespaces_supplied,
                "source_owner_namespace_authenticated": false
            },
            "projected_graph": null,
            "node_mappings": [],
            "edge_mappings": [],
            "projected_payloads": [],
            "disposition": disposition.as_str(),
            "claim_ceiling": "incomplete_or_indeterminate",
            "action_authority": "none",
            "verification_claim": "none",
            "findings": findings.json(),
            "unmapped_relationships": [],
            "limitations": [
                "Invalid input was not projected into a graph.",
                "Source identity namespaces, if supplied, are caller assertions and are not authenticated here.",
                "No source completeness, engineering verification, or action authority is established."
            ]
        });
        if let Ok(receipt_digest) = canonical_json_sha256(&receipt) {
            receipt["projection_receipt_sha256"] = json!(receipt_digest);
        }
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
        let source_metadata = source_metadata_for_record(&native_key, descriptor_overrides);
        let metadata_complete = source_metadata.is_some_and(|metadata| {
            metadata.media_type.as_deref().is_some_and(|value| !value.trim().is_empty())
                && metadata.schema_id.as_deref().is_some_and(|value| !value.trim().is_empty())
                && metadata.schema_version.as_deref().is_some_and(|value| !value.trim().is_empty())
                && metadata.schema_sha256.as_deref().is_some_and(valid_sha256)
        });
        if !metadata_complete {
            findings.incomplete(
                "SOURCE_EVIDENCE_MEDIA_TYPE_SCHEMA_NOT_MODELED",
                &native_key,
                "authoritative source media type, schema ID/version, or schema digest is not available from a validated descriptor",
            );
        }
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
                UnmappedRelation {
                    code: "SOURCE_PAYLOAD_DIGEST_UNAVAILABLE",
                    source_kind: "evidence",
                    source_id: &evidence.evidence_id,
                    relation: "source_payload",
                    referenced_id: &evidence.source_id,
                    detail: "source evidence has no source_payload_sha256; the typed case record is preserved separately but cannot be represented as a source-payload-addressed graph node",
                },
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
        let components =
            identity_components_for_record(&native_key, components, descriptor_overrides);
        let observed = evidence.observed_at_unix_seconds.and_then(unix_seconds_to_utc);
        if evidence.observed_at_unix_seconds.is_some() && observed.is_none() {
            findings.incomplete(
                "EVIDENCE_OBSERVATION_TIME_UNREPRESENTABLE",
                &native_key,
                "source observation time is outside the supported UTC timestamp range",
            );
        }
        let identity_namespace = resolve_native_identity_namespace(
            &native_key,
            context,
            &mut findings,
        );
        let identity_namespace_basis = if context
            .native_identity_namespaces
            .get(&native_key)
            .is_some_and(|namespace| valid_native_identity_namespace(namespace))
        {
            "caller_supplied"
        } else {
            "adapter_fallback"
        };
        let node = match make_node(NodeProjection {
            node_type: "source_evidence",
            identity_namespace: &identity_namespace,
            components,
            payload_sha256: payload_sha,
            artifact_media_type: source_metadata
                .and_then(|metadata| metadata.media_type.as_deref())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("application/octet-stream"),
            schema_version: source_metadata
                .and_then(|metadata| metadata.schema_version.as_deref())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(SOURCE_SCHEMA_UNKNOWN),
            recorded_at: &context.captured_at_utc,
            artifact_created_at: observed,
            case_id: &case.case_id,
            design_id: None,
        }) {
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
                "native_identity_namespace": identity_namespace,
                "native_identity_namespace_basis": identity_namespace_basis,
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
                "source_media_type": source_metadata.and_then(|metadata| metadata.media_type.as_deref()),
                "source_schema_id": source_metadata.and_then(|metadata| metadata.schema_id.as_deref()),
                "source_schema_version": source_metadata.and_then(|metadata| metadata.schema_version.as_deref()),
                "source_schema_sha256": source_metadata.and_then(|metadata| metadata.schema_sha256.as_deref()),
                "schema_version_claim": source_metadata
                    .and_then(|metadata| metadata.schema_version.as_deref())
                    .unwrap_or("unknown")
            }),
        );
        if let Some(payload) = projected_payloads.get_mut(&record_payload_key) {
            payload["node_id"] = json!(node_id);
        }
    }

    // Artifact nodes use the source artifact content digest unchanged.
    for artifact in &case.artifacts {
        let native_key = format!("artifact:{}", artifact.artifact_id);
        let source_metadata = source_metadata_for_record(&native_key, descriptor_overrides);
        let metadata_complete = source_metadata.is_some_and(|metadata| {
            metadata.media_type.as_deref().is_some_and(|value| !value.trim().is_empty())
                && metadata.schema_id.as_deref().is_some_and(|value| !value.trim().is_empty())
                && metadata.schema_version.as_deref().is_some_and(|value| !value.trim().is_empty())
                && metadata.schema_sha256.as_deref().is_some_and(valid_sha256)
        });
        if !metadata_complete {
            findings.incomplete(
                "ENGINEERING_ARTIFACT_MEDIA_TYPE_SCHEMA_NOT_MODELED",
                &native_key,
                "authoritative source media type, schema ID/version, or schema digest is not available from a validated descriptor",
            );
        }
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
        let components =
            identity_components_for_record(&native_key, components, descriptor_overrides);
        let created = unix_seconds_to_utc(artifact.created_at_unix_seconds);
        if created.is_none() {
            findings.incomplete(
                "ARTIFACT_CREATION_TIME_UNREPRESENTABLE",
                &native_key,
                "artifact creation time is outside the supported UTC timestamp range",
            );
        }
        let identity_namespace = resolve_native_identity_namespace(
            &native_key,
            context,
            &mut findings,
        );
        let identity_namespace_basis = if context
            .native_identity_namespaces
            .get(&native_key)
            .is_some_and(|namespace| valid_native_identity_namespace(namespace))
        {
            "caller_supplied"
        } else {
            "adapter_fallback"
        };
        let node = match make_node(NodeProjection {
            node_type: "engineering_artifact",
            identity_namespace: &identity_namespace,
            components,
            payload_sha256: &artifact.content_sha256,
            artifact_media_type: source_metadata
                .and_then(|metadata| metadata.media_type.as_deref())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("application/octet-stream"),
            schema_version: source_metadata
                .and_then(|metadata| metadata.schema_version.as_deref())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(SOURCE_SCHEMA_UNKNOWN),
            recorded_at: &context.captured_at_utc,
            artifact_created_at: created,
            case_id: &case.case_id,
            design_id: Some(&artifact.design_id),
        }) {
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
                "native_identity_namespace": identity_namespace,
                "native_identity_namespace_basis": identity_namespace_basis,
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
                "source_media_type": source_metadata.and_then(|metadata| metadata.media_type.as_deref()),
                "source_schema_id": source_metadata.and_then(|metadata| metadata.schema_id.as_deref()),
                "source_schema_version": source_metadata.and_then(|metadata| metadata.schema_version.as_deref()),
                "source_schema_sha256": source_metadata.and_then(|metadata| metadata.schema_sha256.as_deref())
            }),
        );
    }

    // Engineering receipts do not currently carry a source payload digest.
    // The node digest therefore identifies the exact projector serialization,
    // and the bytes are retained in projected_payloads for independent replay.
    for receipt in &case.engineering_receipts {
        let native_key = format!("engineering_receipt:{}", receipt.receipt_id);
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
        let components =
            identity_components_for_record(&native_key, components, descriptor_overrides);
        let identity_namespace = resolve_native_identity_namespace(
            &native_key,
            context,
            &mut findings,
        );
        let identity_namespace_basis = if context
            .native_identity_namespaces
            .get(&native_key)
            .is_some_and(|namespace| valid_native_identity_namespace(namespace))
        {
            "caller_supplied"
        } else {
            "adapter_fallback"
        };
        let node = match make_node(NodeProjection {
            node_type: "engineering_receipt",
            identity_namespace: &identity_namespace,
            components,
            payload_sha256: &payload_sha,
            artifact_media_type: "application/vnd.luminous.engineering-receipt+json",
            schema_version: "luminous.sol-atlas.engineering-receipt-struct.v0.1",
            recorded_at: &context.captured_at_utc,
            artifact_created_at: None,
            case_id: &case.case_id,
            design_id: Some(&receipt.design_id),
        }) {
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
                "native_identity_namespace": identity_namespace,
                "native_identity_namespace_basis": identity_namespace_basis,
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
        let components =
            identity_components_for_record(&native_key, components, descriptor_overrides);
        let created = unix_seconds_to_utc(decision.decided_at_unix_seconds);
        if created.is_none() {
            findings.incomplete(
                "DECISION_TIME_UNREPRESENTABLE",
                &native_key,
                "decision timestamp is outside the supported UTC timestamp range",
            );
        }
        let identity_namespace = resolve_native_identity_namespace(
            &native_key,
            context,
            &mut findings,
        );
        let identity_namespace_basis = if context
            .native_identity_namespaces
            .get(&native_key)
            .is_some_and(|namespace| valid_native_identity_namespace(namespace))
        {
            "caller_supplied"
        } else {
            "adapter_fallback"
        };
        let node = match make_node(NodeProjection {
            node_type: "decision",
            identity_namespace: &identity_namespace,
            components,
            payload_sha256: &payload_sha,
            artifact_media_type: "application/vnd.luminous.engineering-decision+json",
            schema_version: "luminous.sol-atlas.engineering-decision-struct.v0.1",
            recorded_at: &context.captured_at_utc,
            artifact_created_at: created,
            case_id: &case.case_id,
            design_id: Some(&decision.design_id),
        }) {
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
                "native_identity_namespace": identity_namespace,
                "native_identity_namespace_basis": identity_namespace_basis,
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
    let mut edge_pairs: BTreeSet<(String, String)> = BTreeSet::new();

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
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: parent_node,
                        to_node: child_node,
                        relation: "derived_from",
                        native_relationship_key: &format!("artifact:{}:parent:{}", artifact.artifact_id, parent.artifact_id),
                        mapping_family: "artifact_lineage",
                    },
                );
                }
                Some(_) => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "PARENT_ARTIFACT_DIGEST_OR_PROJECTION_MISMATCH",
                    source_kind: "artifact",
                    source_id: &artifact.artifact_id,
                    relation: "artifact_lineage",
                    referenced_id: &parent.artifact_id,
                    detail: "parent reference digest does not match the source artifact or its node was not projected",
                },
            ),
                None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "PARENT_ARTIFACT_NOT_FOUND",
                    source_kind: "artifact",
                    source_id: &artifact.artifact_id,
                    relation: "artifact_lineage",
                    referenced_id: &parent.artifact_id,
                    detail: "parent artifact reference does not resolve in the case",
                },
            ),
            }
        }

        for evidence_id in &artifact.evidence_ids {
            match evidence_nodes.get(evidence_id) {
                Some(evidence_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: evidence_node,
                        to_node: child_node,
                        relation: "evidence_for",
                        native_relationship_key: &format!("artifact:{}:evidence:{}", artifact.artifact_id, evidence_id),
                        mapping_family: "source_evidence",
                    },
                ),
                None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "ARTIFACT_EVIDENCE_NOT_PROJECTED",
                    source_kind: "artifact",
                    source_id: &artifact.artifact_id,
                    relation: "source_evidence",
                    referenced_id: evidence_id,
                    detail: "referenced evidence is missing or lacks a source payload digest",
                },
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
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: &artifact_nodes[&receipt.input_artifact_id],
                        to_node: receipt_node,
                        relation: "verified_by",
                        native_relationship_key: &format!("receipt:{}:input:{}", receipt.receipt_id, receipt.input_artifact_id),
                        mapping_family: "solver_inputs",
                    },
                );
            }
            Some(_) => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "RECEIPT_INPUT_DIGEST_OR_PROJECTION_MISMATCH",
                    source_kind: "engineering_receipt",
                    source_id: &receipt.receipt_id,
                    relation: "solver_inputs",
                    referenced_id: &receipt.input_artifact_id,
                    detail: "receipt input digest does not match the named source artifact or its node was not projected",
                },
            ),
            None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "RECEIPT_INPUT_ARTIFACT_NOT_FOUND",
                    source_kind: "engineering_receipt",
                    source_id: &receipt.receipt_id,
                    relation: "solver_inputs",
                    referenced_id: &receipt.input_artifact_id,
                    detail: "receipt input artifact does not resolve in the case",
                },
            ),
        }

        for evidence_id in &receipt.assumption_evidence_ids {
            match evidence_nodes.get(evidence_id) {
                Some(evidence_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: evidence_node,
                        to_node: receipt_node,
                        relation: "evidence_for",
                        native_relationship_key: &format!("receipt:{}:assumption:{}", receipt.receipt_id, evidence_id),
                        mapping_family: "solver_assumptions",
                    },
                ),
                None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "RECEIPT_ASSUMPTION_EVIDENCE_NOT_PROJECTED",
                    source_kind: "engineering_receipt",
                    source_id: &receipt.receipt_id,
                    relation: "solver_assumptions",
                    referenced_id: evidence_id,
                    detail: "assumption evidence is missing or lacks a source payload digest",
                },
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
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: &artifact_nodes[&decision.target_artifact_id],
                        to_node: decision_node,
                        relation: "evidence_for",
                        native_relationship_key: &format!("decision:{}:target:{}", decision.event_id, decision.target_artifact_id),
                        mapping_family: "decision_target",
                    },
                );
            }
            Some(_) => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "DECISION_TARGET_DIGEST_OR_PROJECTION_MISMATCH",
                    source_kind: "decision",
                    source_id: &decision.event_id,
                    relation: "decision_target",
                    referenced_id: &decision.target_artifact_id,
                    detail: "decision target digest does not match the named source artifact or its node was not projected",
                },
            ),
            None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "DECISION_TARGET_ARTIFACT_NOT_FOUND",
                    source_kind: "decision",
                    source_id: &decision.event_id,
                    relation: "decision_target",
                    referenced_id: &decision.target_artifact_id,
                    detail: "decision target artifact does not resolve in the case",
                },
            ),
        }

        for receipt_id in &decision.engineering_receipt_ids {
            match receipt_nodes.get(receipt_id) {
                Some(receipt_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: receipt_node,
                        to_node: decision_node,
                        relation: "evidence_for",
                        native_relationship_key: &format!("decision:{}:receipt:{}", decision.event_id, receipt_id),
                        mapping_family: "decision_receipt",
                    },
                ),
                None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "DECISION_RECEIPT_NOT_FOUND",
                    source_kind: "decision",
                    source_id: &decision.event_id,
                    relation: "decision_receipt",
                    referenced_id: receipt_id,
                    detail: "decision references an engineering receipt that does not resolve",
                },
            ),
            }
        }

        if let Some(authority_id) = &decision.authority_evidence_id {
            match evidence_nodes.get(authority_id) {
                Some(authority_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: authority_node,
                        to_node: decision_node,
                        relation: "authorized_by",
                        native_relationship_key: &format!("decision:{}:authority:{}", decision.event_id, authority_id),
                        mapping_family: "decision_authority",
                    },
                ),
                None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "DECISION_AUTHORITY_EVIDENCE_NOT_PROJECTED",
                    source_kind: "decision",
                    source_id: &decision.event_id,
                    relation: "decision_authority",
                    referenced_id: authority_id,
                    detail: "authority evidence is missing or lacks a source payload digest",
                },
            ),
            }
        } else {
            add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "DECISION_AUTHORITY_EVIDENCE_NOT_DECLARED",
                    source_kind: "decision",
                    source_id: &decision.event_id,
                    relation: "decision_authority",
                    referenced_id: "missing",
                    detail: "decision record has no authority_evidence_id; the projector cannot infer an authority relationship",
                },
            );
        }

        if let Some(superseded_id) = &decision.supersedes_event_id {
            match decision_nodes.get(superseded_id) {
                Some(previous_node) => add_edge(
                    &mut edges,
                    &mut edge_mappings,
                    &mut edge_pairs,
                    &mut findings,
                    EdgeProjection {
                        from_node: previous_node,
                        to_node: decision_node,
                        relation: "superseded_by",
                        native_relationship_key: &format!("decision:{}:supersedes:{}", decision.event_id, superseded_id),
                        mapping_family: "decision_supersession",
                    },
                ),
                None => add_unmapped(
                &mut findings,
                &mut unmapped,
                UnmappedRelation {
                    code: "SUPERSEDED_DECISION_NOT_FOUND",
                    source_kind: "decision",
                    source_id: &decision.event_id,
                    relation: "decision_supersession",
                    referenced_id: superseded_id,
                    detail: "superseded decision event does not resolve in the case",
                },
            ),
            }
        }
    }

    // Graph structural validity is independent from mapping completeness.
    // A source cycle is not repaired by the projector; it is an invalid graph.
    if has_dependency_cycle(&nodes, &edges) {
        findings.invalid(
            "PROJECTED_DEPENDENCY_CYCLE",
            "projected_graph",
            "projected dependency graph contains a directed cycle or an unresolved endpoint",
        );
    }

    // A JSON Schema-valid IEAC graph requires at least one node. Preserve
    // the projection payloads and findings, but do not emit an empty graph.
    if nodes.is_empty() {
        findings.incomplete(
            "NO_GRAPH_NODES_PROJECTABLE",
            "projected_graph",
            "no source-addressed graph node could be projected from the case",
        );
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
                "adapter_fallback_identity_namespace": IDENTITY_NAMESPACE,
                "identity_namespace_basis": if all_native_identity_namespaces_supplied {
                    "caller_supplied_per_record_namespace_map_not_authenticated"
                } else {
                    "caller_supplied_map_with_adapter_fallback"
                },
                "native_identity_namespaces": context.native_identity_namespaces,
                "native_identity_namespace_map_sha256": namespace_map_digest(&context.native_identity_namespaces).unwrap_or_default(),
                "native_identity_namespace_entries_supplied": supplied_native_identity_namespace_count,
                "native_identity_namespace_entries_expected": expected_namespace_keys.len(),
                "source_owner_namespace_entries_supplied_for_all_records": all_native_identity_namespaces_supplied,
                "source_owner_namespace_authenticated": false
            },
            "projected_graph": null,
            "node_mappings": node_mappings.into_values().collect::<Vec<_>>(),
            "edge_mappings": edge_mappings.into_values().collect::<Vec<_>>(),
            "projected_payloads": projected_payloads.into_values().collect::<Vec<_>>(),
            "unmapped_relationships": unmapped.into_values().collect::<Vec<_>>(),
            "findings": findings.json(),
            "disposition": final_disposition.as_str(),
            "claim_ceiling": "incomplete_or_indeterminate",
            "action_authority": "none",
            "verification_claim": "none",
            "limitations": [
                "No source-addressable IEAC graph node could be produced.",
                "Source identity namespaces, if supplied, are caller assertions and are not authenticated here.",
                "No source completeness, engineering verification, or action authority is established."
            ]
        });
        if let Ok(receipt_digest) = canonical_json_sha256(&receipt) {
            receipt["projection_receipt_sha256"] = json!(receipt_digest);
        }
        return projection_output(Value::Null, receipt, final_disposition);
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
        "incomplete_or_indeterminate"
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
            "adapter_fallback_identity_namespace": IDENTITY_NAMESPACE,
            "identity_namespace_basis": if all_native_identity_namespaces_supplied {
                "caller_supplied_per_record_namespace_map_not_authenticated"
            } else {
                "caller_supplied_map_with_adapter_fallback"
            },
            "native_identity_namespaces": context.native_identity_namespaces,
            "native_identity_namespace_map_sha256": namespace_map_digest(&context.native_identity_namespaces).unwrap_or_default(),
            "native_identity_namespace_entries_supplied": supplied_native_identity_namespace_count,
            "native_identity_namespace_entries_expected": expected_namespace_keys.len(),
            "source_owner_namespace_entries_supplied_for_all_records": all_native_identity_namespaces_supplied,
            "source_owner_namespace_authenticated": false,
            "source_descriptor_catalog": descriptor_overrides.map(|value| json!({
                "catalog_sha256": value.catalog_sha256,
                "validation_disposition": value.validation.disposition.as_str(),
                "descriptor_count": value.validation.descriptor_count,
                "missing_descriptor_count": value.validation.missing_descriptor_count,
                "extra_descriptor_count": value.validation.extra_descriptor_count,
                "payload_byte_verification": value.validation.payload_byte_verification,
                "rights_authentication": value.validation.rights_authentication,
                "source_attestation_authentication": value.validation.source_attestation_authentication,
                "action_authority": "none",
                "findings": value.validation.findings
            }))
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
            "incomplete_or_indeterminate"
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
            native_identity_namespaces: BTreeMap::from([
                ("evidence:evidence-design-source".into(), "urn:source-owner:survey-registry".into()),
                ("evidence:evidence-authority".into(), "urn:source-owner:authority-registry".into()),
                ("artifact:artifact-base".into(), "urn:source-owner:design-repository".into()),
                ("artifact:artifact-revision-2".into(), "urn:source-owner:design-repository".into()),
                ("engineering_receipt:receipt-structural-001".into(), "urn:source-owner:solver-registry".into()),
                ("decision:decision-001".into(), "urn:source-owner:authority-registry".into())
            ]),
        }
    }

    fn independent_sha256(bytes: &[u8]) -> String {
        use sha2::Digest as _;
        let digest = sha2::Sha256::digest(bytes);
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn append_independent_identity_field(encoded: &mut Vec<u8>, value: &str) {
        encoded.extend_from_slice(&(value.len() as u64).to_be_bytes());
        encoded.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn graph_identity_and_snapshot_digests_match_independent_recomputation() {
        let output = project_built_environment_case(&sample_case(), &context());
        let nodes = output.graph["nodes"].as_array().unwrap();

        for node in nodes {
            let node_type = node["node_type"].as_str().unwrap();
            let identity_namespace = node["identity"]["identity_namespace"].as_str().unwrap();
            let mut components = node["identity"]["components"]
                .as_array()
                .unwrap()
                .iter()
                .map(|component| (
                    component["name"].as_str().unwrap().to_owned(),
                    component["value"].as_str().unwrap().to_owned(),
                ))
                .collect::<Vec<_>>();
            components.sort_by(|left, right| left.0.cmp(&right.0));

            let mut encoded = b"LUMINOUS-IEAC-NODE-IDENTITY-V0.2\0".to_vec();
            append_independent_identity_field(&mut encoded, node_type);
            append_independent_identity_field(&mut encoded, identity_namespace);
            for (name, value) in components {
                append_independent_identity_field(&mut encoded, &name);
                append_independent_identity_field(&mut encoded, &value);
            }

            let expected_identity = independent_sha256(&encoded);
            assert_eq!(node["identity_sha256"], expected_identity);
            assert_eq!(
                node["node_id"],
                format!("{node_type}:identity-sha256:{expected_identity}")
            );
        }

        let mut graph_core = output.graph.clone();
        graph_core.as_object_mut().unwrap().remove("graph_snapshot_sha256");
        let expected_graph_digest =
            independent_sha256(&serde_json::to_vec(&graph_core).unwrap());
        assert_eq!(
            output.graph["graph_snapshot_sha256"],
            expected_graph_digest
        );
    }

    #[test]
    fn unmodeled_source_media_type_and_schema_are_explicit_projection_losses() {
        let result = project_built_environment_case(&sample_case(), &context());
        let findings = result.projection_receipt["findings"].as_array().unwrap();
        assert!(findings.iter().any(|finding| {
            finding["code"] == "SOURCE_EVIDENCE_MEDIA_TYPE_SCHEMA_NOT_MODELED"
        }));
        assert!(findings.iter().any(|finding| {
            finding["code"] == "ENGINEERING_ARTIFACT_MEDIA_TYPE_SCHEMA_NOT_MODELED"
        }));
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
        assert_eq!(result.graph["claim_ceiling"], "incomplete_or_indeterminate");
    }

    #[test]
    fn source_owner_namespace_is_part_of_native_identity_and_receipt() {
        let result = project_built_environment_case(&sample_case(), &context());
        let evidence_mapping = result.projection_receipt["node_mappings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["record_kind"] == "evidence")
            .unwrap();
        let expected_namespace = "urn:source-owner:survey-registry";
        assert_eq!(evidence_mapping["native_identity_namespace"], expected_namespace);
        let evidence_node = result.graph["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["node_type"] == "source_evidence"
                && node["identity"]["identity_namespace"] == expected_namespace)
            .unwrap();
        assert_eq!(evidence_node["identity"]["identity_namespace"], expected_namespace);
        assert!(valid_sha256(
            result.projection_receipt["projector"]["native_identity_namespace_map_sha256"]
                .as_str()
                .unwrap()
        ));
        assert!(!result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "NATIVE_IDENTITY_NAMESPACE_NOT_SUPPLIED"));
        assert_eq!(
            result.projection_receipt["projector"]["native_identity_namespace_entries_supplied"],
            6
        );
        assert_eq!(
            result.projection_receipt["projector"]["native_identity_namespace_entries_expected"],
            6
        );
        assert_eq!(
            result.projection_receipt["projector"]["source_owner_namespace_entries_supplied_for_all_records"],
            true
        );
        assert_eq!(
            result.projection_receipt["projector"]["source_owner_namespace_authenticated"],
            false
        );
        assert_eq!(
            evidence_mapping["native_identity_namespace_basis"],
            "caller_supplied"
        );
    }

    #[test]
    fn missing_source_owner_namespace_uses_visible_fallback_and_downgrades() {
        let mut projection_context = context();
        projection_context
            .native_identity_namespaces
            .remove("evidence:evidence-design-source");
        let result = project_built_environment_case(&sample_case(), &projection_context);
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
        assert!(result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "NATIVE_IDENTITY_NAMESPACE_NOT_SUPPLIED"));
        let node = result.graph["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["identity"]["components"].as_array().unwrap().iter()
                .any(|component| component["name"] == "native_record_id"
                    && component["value"] == "evidence-design-source"))
            .unwrap();
        assert_eq!(node["identity"]["identity_namespace"], IDENTITY_NAMESPACE);
        assert_eq!(
            result.projection_receipt["projector"]["source_owner_namespace_entries_supplied_for_all_records"],
            false
        );
        assert_eq!(
            result.projection_receipt["projector"]["native_identity_namespace_entries_supplied"],
            5
        );
        let fallback_mapping = result.projection_receipt["node_mappings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["native_key"] == "evidence:evidence-design-source")
            .unwrap();
        assert_eq!(
            fallback_mapping["native_identity_namespace_basis"],
            "adapter_fallback"
        );
    }

    #[test]
    fn empty_source_owner_namespace_is_invalid_not_silently_normalized() {
        let mut projection_context = context();
        projection_context.native_identity_namespaces.insert(
            "evidence:evidence-design-source".into(),
            "   ".into(),
        );
        let result = project_built_environment_case(&sample_case(), &projection_context);
        assert_eq!(result.disposition, ProjectionDisposition::InvalidInput);
        assert!(result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "INVALID_NATIVE_IDENTITY_NAMESPACE"));
    }

    #[test]
    fn same_payload_does_not_collapse_distinct_source_evidence_identities() {
        let result = project_built_environment_case(&sample_case(), &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
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
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
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
        assert_eq!(
            result.graph["claim_ceiling"],
            "incomplete_or_indeterminate"
        );
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
            "incomplete_or_indeterminate"
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
    fn projection_receipt_digest_binds_its_complete_receipt_body() {
        let result = project_built_environment_case(&sample_case(), &context());
        let declared = result.projection_receipt["projection_receipt_sha256"]
            .as_str()
            .expect("receipt digest is present");
        let mut digest_input = result.projection_receipt.clone();
        digest_input
            .as_object_mut()
            .unwrap()
            .remove("projection_receipt_sha256");
        assert_eq!(
            canonical_json_sha256(&digest_input).unwrap(),
            declared
        );
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
    fn case_with_only_undigestible_evidence_has_receipt_but_no_empty_graph() {
        let mut case = sample_case();
        case.artifacts.clear();
        case.engineering_receipts.clear();
        case.decisions.clear();
        for evidence in &mut case.evidence {
            evidence.source_payload_sha256 = None;
        }
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
        assert_eq!(result.graph, Value::Null);
        assert!(result.projection_receipt["projected_graph"].is_null());
        assert_eq!(
            result.projection_receipt["projected_payloads"].as_array().unwrap().len(),
            2
        );
        assert!(result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "NO_GRAPH_NODES_PROJECTABLE"));
    }

    #[test]
    fn artifact_parent_cycle_is_invalid_and_never_claimed_as_record_only() {
        let mut case = sample_case();
        case.artifacts[0].parent_artifacts.push(ArtifactReference {
            artifact_id: "artifact-revision-2".into(),
            content_sha256: digest('c'),
        });
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::InvalidInput);
        assert_eq!(
            result.graph["claim_ceiling"],
            "incomplete_or_indeterminate"
        );
        assert!(result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "PROJECTED_DEPENDENCY_CYCLE"));
    }

    #[test]
    fn duplicate_endpoint_pairs_are_not_silently_coalesced() {
        let mut case = sample_case();
        case.artifacts[0]
            .evidence_ids
            .push("evidence-design-source".into());
        let result = project_built_environment_case(&case, &context());
        assert_eq!(result.disposition, ProjectionDisposition::InvalidInput);
        assert!(result.projection_receipt["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "DUPLICATE_PROJECTED_EDGE_ENDPOINT_PAIR"));
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
