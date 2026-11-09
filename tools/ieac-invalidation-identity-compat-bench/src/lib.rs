//! Independent synthetic compatibility checker for IEAC graph/event/report v0.2.
//!
//! This is not the production impact engine, signature/SCITT verifier, issuer
//! authenticator, or professional approval system. It checks typed bindings
//! between an exact graph snapshot, a bitemporal invalidation event, and report.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const GRAPH_V01: &str = "luminous.ieac-assurance-dependency-graph.v0.1";
pub const EVENT_V01: &str = "luminous.ieac-assurance-invalidation-event.v0.1";
pub const REPORT_V01: &str = "luminous.ieac-assurance-impact-report.v0.1";
pub const GRAPH_V02: &str = "luminous.ieac-assurance-dependency-graph.v0.2";
pub const EVENT_V02: &str = "luminous.ieac-assurance-invalidation-event.v0.2";
pub const REPORT_V02: &str = "luminous.ieac-assurance-impact-report.v0.2";
pub const CORPUS_ID: &str = "IEAC-INVALIDATION-IDENTITY-COMPAT-CORPUS-V0.1";
pub const CLAIM_CEILING: &str = "synthetic_identity_content_compatibility_only";
const IDENTITY_DOMAIN: &[u8] = b"LUMINOUS-IEAC-NODE-IDENTITY-V0.2\0";

const NODE_TYPES: &[&str] = &[
    "source_measurement",
    "signed_statement",
    "transparency_receipt",
    "issuer_key",
    "transparency_service_key",
    "schema",
    "transformation",
    "model",
    "policy",
    "authority_grant",
    "decision",
    "action_authorization",
    "execution",
    "outcome",
    "source_evidence",
    "engineering_artifact",
    "engineering_receipt",
    "verifier_build",
    "trust_bundle",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub accepted: bool,
    pub compatibility: String,
    pub reason_codes: Vec<String>,
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn add(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn canonical_sha256(value: &Value, excluded_key: &str) -> Result<String, String> {
    let mut core = value.clone();
    let object = core
        .as_object_mut()
        .ok_or_else(|| "CANONICAL_DIGEST_OBJECT_REQUIRED".to_owned())?;
    object.remove(excluded_key);
    let bytes = serde_json::to_vec(&core)
        .map_err(|error| format!("CANONICAL_JSON_SERIALIZATION_FAILED:{error}"))?;
    Ok(sha256_hex(&bytes))
}

fn append_identity_field(buffer: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let length =
        u64::try_from(value.len()).map_err(|_| "IDENTITY_COMPONENT_LENGTH_OVERFLOW".to_owned())?;
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(value.as_bytes());
    Ok(())
}

fn valid_component_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
}

fn identity_digest(node: &Value) -> Result<String, String> {
    let identity = node
        .get("identity")
        .ok_or_else(|| "NATIVE_IDENTITY_OBJECT_MISSING".to_owned())?;
    let namespace = text(identity, "identity_namespace");
    if namespace.is_empty() || namespace.trim() != namespace || namespace.len() > 1024 {
        return Err("NATIVE_IDENTITY_NAMESPACE_INVALID".to_owned());
    }
    let components = identity
        .get("components")
        .and_then(Value::as_array)
        .ok_or_else(|| "NATIVE_IDENTITY_COMPONENTS_MISSING".to_owned())?;
    let mut ordered = BTreeMap::new();
    for component in components {
        let name = text(component, "name");
        let value = text(component, "value");
        if !valid_component_name(name) || value.is_empty() {
            return Err("NATIVE_IDENTITY_COMPONENT_INVALID".to_owned());
        }
        if ordered.insert(name.to_owned(), value.to_owned()).is_some() {
            return Err("DUPLICATE_IDENTITY_COMPONENT".to_owned());
        }
    }
    if ordered.len() < 2 || !ordered.contains_key("native_id") {
        return Err("NATIVE_IDENTITY_NATIVE_ID_MISSING".to_owned());
    }
    if ![
        "native_record_id",
        "native_revision",
        "native_sequence",
        "native_observation_id",
    ]
    .iter()
    .any(|name| ordered.contains_key(*name))
    {
        return Err("NATIVE_IDENTITY_DISCRIMINATOR_MISSING".to_owned());
    }

    let mut encoded = IDENTITY_DOMAIN.to_vec();
    append_identity_field(&mut encoded, text(node, "node_type"))?;
    append_identity_field(&mut encoded, namespace)?;
    for (name, value) in ordered {
        append_identity_field(&mut encoded, &name)?;
        append_identity_field(&mut encoded, &value)?;
    }
    Ok(sha256_hex(&encoded))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn check_graph_node(node: &Value, reasons: &mut BTreeSet<String>) {
    let node_type = text(node, "node_type");
    if !NODE_TYPES.contains(&node_type) {
        add(reasons, "GRAPH_NODE_TYPE_UNSUPPORTED");
        return;
    }
    let declared_identity = text(node, "identity_sha256");
    let artifact = text(node, "artifact_sha256");
    if !valid_digest(declared_identity) || !valid_digest(artifact) {
        add(reasons, "GRAPH_NODE_DIGEST_INVALID");
        return;
    }
    match identity_digest(node) {
        Ok(computed) if computed == declared_identity => {
            let expected_id = format!("{node_type}:identity-sha256:{computed}");
            if text(node, "node_id") != expected_id {
                add(reasons, "GRAPH_NODE_IDENTITY_BINDING_INVALID");
            }
        }
        Ok(_) => add(reasons, "NATIVE_IDENTITY_DIGEST_MISMATCH"),
        Err(code) => add(reasons, &code),
    }
}

fn check_exact_keys(
    value: &Value,
    expected: &[&str],
    reason: &str,
    reasons: &mut BTreeSet<String>,
) {
    let Some(object) = value.as_object() else {
        add(reasons, reason);
        return;
    };
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        add(reasons, reason);
    }
}

fn find_node<'a>(graph: &'a Value, node_id: &str) -> Option<&'a Value> {
    array(graph, "nodes")
        .iter()
        .find(|node| text(node, "node_id") == node_id)
}

fn legacy_v01(graph: &Value, event: &Value, report: &Value) -> Evaluation {
    let mut reasons = BTreeSet::new();
    if array(graph, "nodes").is_empty()
        || array(event, "targets").is_empty()
        || array(report, "results").is_empty()
    {
        add(&mut reasons, "LEGACY_V01_REQUIRED_RECORDS_MISSING");
    }
    for target in array(event, "targets") {
        let matched = array(graph, "nodes").iter().any(|node| {
            text(node, "node_id") == text(target, "node_id")
                && text(node, "node_type") == text(target, "node_type")
                && text(node, "artifact_sha256") == text(target, "artifact_sha256")
        });
        if !matched {
            add(&mut reasons, "LEGACY_V01_TARGET_BINDING_MISMATCH");
        }
    }
    for result in array(report, "results") {
        let matched = array(graph, "nodes").iter().any(|node| {
            text(node, "node_id") == text(result, "node_id")
                && text(node, "node_type") == text(result, "node_type")
                && text(node, "artifact_sha256") == text(result, "artifact_sha256")
        });
        if !matched {
            add(&mut reasons, "LEGACY_V01_RESULT_BINDING_MISMATCH");
        }
    }
    if text(report, "graph_snapshot_sha256") != text(graph, "graph_snapshot_sha256") {
        add(&mut reasons, "LEGACY_V01_GRAPH_SNAPSHOT_MISMATCH");
    }
    if [graph, event, report]
        .iter()
        .any(|item| text(item, "action_authority") != "none")
    {
        add(&mut reasons, "ACTION_AUTHORITY_MUST_REMAIN_NONE");
    }
    Evaluation {
        accepted: reasons.is_empty(),
        compatibility: "legacy_v0_1_preserved_without_coercion".to_owned(),
        reason_codes: reasons.into_iter().collect(),
    }
}

fn evaluate_v02(graph: &Value, event: &Value, report: &Value) -> Evaluation {
    let mut reasons = BTreeSet::new();
    check_exact_keys(
        graph,
        &[
            "schema_version",
            "graph_id",
            "captured_at",
            "graph_snapshot_sha256",
            "nodes",
            "edges",
            "claim_ceiling",
            "action_authority",
        ],
        "GRAPH_ROOT_FIELD_SET_INVALID",
        &mut reasons,
    );
    check_exact_keys(
        event,
        &[
            "schema_version",
            "event_id",
            "issuer_id",
            "authority_sequence",
            "previous_event_sha256",
            "statement_sha256",
            "cause_class",
            "targets",
            "scope",
            "valid_time",
            "system_time",
            "invalidation_semantics",
            "evidence_references",
            "authority",
            "dispute_state",
            "action_authority",
            "graph_schema_version",
            "graph_snapshot_sha256",
            "event_sha256",
        ],
        "EVENT_ROOT_FIELD_SET_INVALID",
        &mut reasons,
    );
    check_exact_keys(
        report,
        &[
            "schema_version",
            "report_id",
            "graph_schema_version",
            "invalidation_event_schema_version",
            "graph_snapshot_sha256",
            "invalidation_event_sha256",
            "impact_policy_sha256",
            "impact_engine_build_sha256",
            "evaluated_at",
            "results",
            "summary",
            "historical_record_preserved",
            "not_checked",
            "claim_ceiling",
            "action_authority",
        ],
        "REPORT_ROOT_FIELD_SET_INVALID",
        &mut reasons,
    );
    if text(graph, "schema_version") != GRAPH_V02
        || text(event, "schema_version") != EVENT_V02
        || text(report, "schema_version") != REPORT_V02
        || text(event, "graph_schema_version") != GRAPH_V02
        || text(report, "graph_schema_version") != GRAPH_V02
        || text(report, "invalidation_event_schema_version") != EVENT_V02
    {
        add(&mut reasons, "VERSION_MATRIX_MISMATCH");
    }
    if [graph, event, report]
        .iter()
        .any(|item| text(item, "action_authority") != "none")
    {
        add(&mut reasons, "ACTION_AUTHORITY_MUST_REMAIN_NONE");
    }

    let graph_sha = canonical_sha256(graph, "graph_snapshot_sha256").unwrap_or_default();
    if !valid_digest(&graph_sha) || text(graph, "graph_snapshot_sha256") != graph_sha {
        add(&mut reasons, "GRAPH_SNAPSHOT_DIGEST_MISMATCH");
    }
    if text(event, "graph_snapshot_sha256") != graph_sha {
        add(&mut reasons, "EVENT_GRAPH_SNAPSHOT_MISMATCH");
    }
    if text(report, "graph_snapshot_sha256") != graph_sha {
        add(&mut reasons, "REPORT_GRAPH_SNAPSHOT_MISMATCH");
    }

    let event_sha = canonical_sha256(event, "event_sha256").unwrap_or_default();
    if !valid_digest(&event_sha) || text(event, "event_sha256") != event_sha {
        add(&mut reasons, "EVENT_DIGEST_MISMATCH");
    }
    if text(report, "invalidation_event_sha256") != event_sha {
        add(&mut reasons, "REPORT_EVENT_DIGEST_MISMATCH");
    }

    let mut graph_ids = HashSet::new();
    for node in array(graph, "nodes") {
        check_exact_keys(
            node,
            &[
                "node_id",
                "node_type",
                "identity",
                "identity_sha256",
                "artifact_sha256",
                "artifact_media_type",
                "schema_version",
                "issuer_id",
                "scope",
                "valid_time",
                "observation_time",
                "verification",
            ],
            "GRAPH_NODE_FIELD_SET_INVALID",
            &mut reasons,
        );
        check_exact_keys(
            node.get("identity").unwrap_or(&Value::Null),
            &["identity_namespace", "components"],
            "GRAPH_NODE_IDENTITY_FIELD_SET_INVALID",
            &mut reasons,
        );
        for component in array(node.get("identity").unwrap_or(&Value::Null), "components") {
            check_exact_keys(
                component,
                &["name", "value"],
                "GRAPH_IDENTITY_COMPONENT_FIELD_SET_INVALID",
                &mut reasons,
            );
        }
        if !graph_ids.insert(text(node, "node_id").to_owned()) {
            add(&mut reasons, "DUPLICATE_GRAPH_NODE_ID");
        }
        check_graph_node(node, &mut reasons);
    }

    let semantics = text(event, "invalidation_semantics");
    let valid_time = event.get("valid_time").unwrap_or(&Value::Null);
    let system_time = event.get("system_time").unwrap_or(&Value::Null);
    let authority = event.get("authority").unwrap_or(&Value::Null);
    check_exact_keys(
        valid_time,
        &[
            "effective_at",
            "effective_until",
            "effective_time_confidence",
        ],
        "EVENT_VALID_TIME_FIELD_SET_INVALID",
        &mut reasons,
    );
    check_exact_keys(
        system_time,
        &[
            "recorded_at",
            "trusted_at",
            "trusted_timestamp_receipt_sha256",
        ],
        "EVENT_SYSTEM_TIME_FIELD_SET_INVALID",
        &mut reasons,
    );
    check_exact_keys(
        authority,
        &[
            "authority_id",
            "key_id",
            "scope_policy_sha256",
            "authorization_receipt_sha256",
            "independent_review",
        ],
        "EVENT_AUTHORITY_FIELD_SET_INVALID",
        &mut reasons,
    );
    if semantics == "retroactive_from_effective_time"
        && (text(valid_time, "effective_time_confidence") != "verified"
            || text(valid_time, "effective_at").is_empty()
            || text(authority, "independent_review") != "approved")
    {
        add(&mut reasons, "INVALIDATION_AUTHORITY_INSUFFICIENT");
    }
    if text(event, "dispute_state") != "undisputed" && semantics != "quarantined_pending_review" {
        add(&mut reasons, "DISPUTED_EVENT_NOT_QUARANTINED");
    }
    if text(authority, "independent_review") == "pending" {
        add(&mut reasons, "INVALIDATION_AUTHORITY_INSUFFICIENT");
    }

    let mut targets = BTreeSet::new();
    let mut resolved_targets = BTreeSet::new();
    for target in array(event, "targets") {
        check_exact_keys(
            target,
            &["node_id", "node_type", "identity_sha256", "artifact_sha256"],
            "EVENT_TARGET_FIELD_SET_INVALID",
            &mut reasons,
        );
        let id = text(target, "node_id");
        if !targets.insert(id.to_owned()) {
            add(&mut reasons, "DUPLICATE_INVALIDATION_TARGET");
            continue;
        }
        let Some(node) = find_node(graph, id) else {
            add(&mut reasons, "INVALIDATION_TARGET_NODE_MISSING");
            continue;
        };
        if text(target, "node_type") != text(node, "node_type")
            || text(target, "identity_sha256") != text(node, "identity_sha256")
            || text(target, "artifact_sha256") != text(node, "artifact_sha256")
        {
            add(&mut reasons, "INVALIDATION_TARGET_NODE_MISMATCH");
        } else {
            resolved_targets.insert(id.to_owned());
        }
    }

    let mut results = BTreeSet::new();
    for result in array(report, "results") {
        check_exact_keys(
            result,
            &[
                "node_id",
                "node_type",
                "identity_sha256",
                "artifact_sha256",
                "impact_class",
                "historical_authenticity",
                "current_reliance",
                "dependency_path",
                "reason_codes",
            ],
            "REPORT_RESULT_FIELD_SET_INVALID",
            &mut reasons,
        );
        let id = text(result, "node_id");
        if !results.insert(id.to_owned()) {
            add(&mut reasons, "DUPLICATE_IMPACT_RESULT_NODE");
            continue;
        }
        let Some(node) = find_node(graph, id) else {
            add(&mut reasons, "IMPACT_RESULT_NODE_MISSING");
            continue;
        };
        if text(result, "node_type") != text(node, "node_type")
            || text(result, "identity_sha256") != text(node, "identity_sha256")
            || text(result, "artifact_sha256") != text(node, "artifact_sha256")
        {
            add(&mut reasons, "IMPACT_RESULT_NODE_MISMATCH");
        }
    }
    if resolved_targets.iter().any(|id| !results.contains(id)) {
        add(&mut reasons, "IMPACT_RESULT_MISSING_INVALIDATION_TARGET");
    }
    if report["summary"]["node_count"].as_u64() != Some(array(report, "results").len() as u64) {
        add(&mut reasons, "IMPACT_REPORT_SUMMARY_COUNT_MISMATCH");
    }

    Evaluation {
        accepted: reasons.is_empty(),
        compatibility: "v0_2_identity_and_content_bound".to_owned(),
        reason_codes: reasons.into_iter().collect(),
    }
}

pub fn evaluate_triplet(graph: &Value, event: &Value, report: &Value) -> Evaluation {
    let versions = (
        text(graph, "schema_version"),
        text(event, "schema_version"),
        text(report, "schema_version"),
    );
    match versions {
        (GRAPH_V01, EVENT_V01, REPORT_V01) => legacy_v01(graph, event, report),
        (GRAPH_V02, EVENT_V02, REPORT_V02) => evaluate_v02(graph, event, report),
        _ => Evaluation {
            accepted: false,
            compatibility: "mixed_or_unsupported_version_triplet".to_owned(),
            reason_codes: vec!["VERSION_MATRIX_MISMATCH".to_owned()],
        },
    }
}

fn seal_v02(graph: &mut Value, event: &mut Value, report: &mut Value) {
    graph["graph_snapshot_sha256"] = json!("");
    let graph_sha = canonical_sha256(graph, "graph_snapshot_sha256").unwrap_or_default();
    graph["graph_snapshot_sha256"] = json!(graph_sha);
    event["graph_snapshot_sha256"] = json!(graph_sha);
    event["event_sha256"] = json!("");
    let event_sha = canonical_sha256(event, "event_sha256").unwrap_or_default();
    event["event_sha256"] = json!(event_sha);
    report["graph_snapshot_sha256"] = json!(graph_sha);
    report["invalidation_event_sha256"] = json!(event_sha);
}

fn make_node(node_type: &str, native_id: &str, revision: &str, payload: &str) -> Value {
    let mut node = json!({
        "node_id": "",
        "node_type": node_type,
        "identity": {
            "identity_namespace": "urn:luminous:synthetic-owner:habitat",
            "components": [
                {"name": "native_id", "value": native_id},
                {"name": "native_revision", "value": revision}
            ]
        },
        "identity_sha256": "",
        "artifact_sha256": sha256_hex(payload.as_bytes()),
        "artifact_media_type": "application/json",
        "schema_version": "owner.synthetic.contract@1.0.0",
        "issuer_id": "urn:luminous:synthetic-owner:habitat",
        "scope": {
            "domain": "habitat",
            "subject_pseudonyms": ["synthetic-dwelling-01"],
            "purposes": ["engineering-assurance"]
        },
        "valid_time": {
            "valid_from": "2026-01-01T00:00:00Z",
            "valid_until": null
        },
        "observation_time": {
            "artifact_created_at": "2026-01-01T00:00:00Z",
            "recorded_at": "2026-01-01T00:01:00Z",
            "trusted_at": "2026-01-01T00:02:00Z"
        },
        "verification": {
            "result": "verified",
            "policy_sha256": sha256_hex(b"policy"),
            "verifier_build_sha256": sha256_hex(b"verifier-build"),
            "trust_bundle_sha256": sha256_hex(b"trust-bundle"),
            "checked_at": "2026-01-01T00:03:00Z",
            "fresh_until": "2027-01-01T00:00:00Z"
        }
    });
    let identity = identity_digest(&node).expect("fixture identity is valid");
    node["identity_sha256"] = json!(identity);
    node["node_id"] = json!(format!("{node_type}:identity-sha256:{identity}"));
    node
}

fn target_from_node(node: &Value) -> Value {
    json!({
        "node_id": node["node_id"],
        "node_type": node["node_type"],
        "identity_sha256": node["identity_sha256"],
        "artifact_sha256": node["artifact_sha256"]
    })
}

fn result_from_node(node: &Value) -> Value {
    json!({
        "node_id": node["node_id"],
        "node_type": node["node_type"],
        "identity_sha256": node["identity_sha256"],
        "artifact_sha256": node["artifact_sha256"],
        "impact_class": "reverify_required",
        "historical_authenticity": "valid_at_time",
        "current_reliance": "reverify_required",
        "dependency_path": ["synthetic-edge-0001"],
        "reason_codes": ["SOURCE_CORRECTED"]
    })
}

fn base_v02() -> (Value, Value, Value) {
    let node = make_node(
        "source_evidence",
        "native-record-001",
        "revision-1",
        "{\"service\":\"water\"}",
    );
    let graph = json!({
        "schema_version": GRAPH_V02,
        "graph_id": "synthetic-ieac-graph-0001",
        "captured_at": "2026-01-01T00:04:00Z",
        "graph_snapshot_sha256": "",
        "nodes": [node.clone()],
        "edges": [],
        "claim_ceiling": "record_only",
        "action_authority": "none"
    });
    let event = json!({
        "schema_version": EVENT_V02,
        "event_id": "synthetic-invalidation-event-0001",
        "issuer_id": "urn:luminous:synthetic-owner:habitat",
        "authority_sequence": 1,
        "previous_event_sha256": null,
        "statement_sha256": sha256_hex(b"synthetic-revocation-statement"),
        "cause_class": "source_corrected",
        "targets": [target_from_node(&node)],
        "scope": {
            "domains": ["habitat"],
            "subject_pseudonyms": ["synthetic-dwelling-01"],
            "purposes": ["engineering-assurance"]
        },
        "valid_time": {
            "effective_at": "2026-01-01T00:05:00Z",
            "effective_until": null,
            "effective_time_confidence": "verified"
        },
        "system_time": {
            "recorded_at": "2026-01-01T00:05:00Z",
            "trusted_at": "2026-01-01T00:05:01Z",
            "trusted_timestamp_receipt_sha256": sha256_hex(b"synthetic-scitt-receipt")
        },
        "invalidation_semantics": "reverify_required",
        "evidence_references": [{
            "evidence_id": "synthetic-evidence-0001",
            "artifact_sha256": sha256_hex(b"synthetic-evidence"),
            "media_type": "application/json"
        }],
        "authority": {
            "authority_id": "urn:luminous:synthetic-owner:habitat",
            "key_id": "synthetic-key-001",
            "scope_policy_sha256": sha256_hex(b"scope-policy"),
            "authorization_receipt_sha256": sha256_hex(b"authorization-receipt"),
            "independent_review": "approved"
        },
        "dispute_state": "undisputed",
        "action_authority": "none",
        "graph_schema_version": GRAPH_V02,
        "graph_snapshot_sha256": "",
        "event_sha256": ""
    });
    let report = json!({
        "schema_version": REPORT_V02,
        "report_id": "synthetic-impact-report-0001",
        "graph_schema_version": GRAPH_V02,
        "invalidation_event_schema_version": EVENT_V02,
        "graph_snapshot_sha256": "",
        "invalidation_event_sha256": "",
        "impact_policy_sha256": sha256_hex(b"impact-policy"),
        "impact_engine_build_sha256": sha256_hex(b"impact-engine"),
        "evaluated_at": "2026-01-01T00:06:00Z",
        "results": [result_from_node(&node)],
        "summary": {
            "node_count": 1,
            "no_impact_count": 0,
            "historical_record_only_count": 0,
            "reverify_required_count": 1,
            "claim_invalidated_count": 0,
            "quarantined_pending_review_count": 0
        },
        "historical_record_preserved": true,
        "not_checked": ["issuer identity authentication", "professional engineering approval"],
        "claim_ceiling": "record_only",
        "action_authority": "none"
    });
    let mut graph = graph;
    let mut event = event;
    let mut report = report;
    seal_v02(&mut graph, &mut event, &mut report);
    (graph, event, report)
}

fn base_v01() -> (Value, Value, Value) {
    let artifact = sha256_hex(b"legacy-v0.1-payload");
    let node_id = format!("source_measurement:sha256:{artifact}");
    let node = json!({
        "node_id": node_id,
        "node_type": "source_measurement",
        "artifact_sha256": artifact,
        "artifact_media_type": "application/json",
        "schema_version": "owner.synthetic.contract@1.0.0",
        "issuer_id": "urn:luminous:synthetic-owner:legacy",
        "scope": {
            "domain": "habitat",
            "subject_pseudonyms": ["synthetic-dwelling-01"],
            "purposes": ["engineering-assurance"]
        },
        "valid_time": {
            "valid_from": "2025-01-01T00:00:00Z",
            "valid_until": null
        },
        "observation_time": {
            "artifact_created_at": "2025-01-01T00:00:00Z",
            "recorded_at": "2025-01-01T00:01:00Z",
            "trusted_at": "2025-01-01T00:02:00Z"
        },
        "verification": {
            "result": "verified",
            "policy_sha256": sha256_hex(b"legacy-policy"),
            "verifier_build_sha256": sha256_hex(b"legacy-verifier"),
            "trust_bundle_sha256": sha256_hex(b"legacy-trust-bundle"),
            "checked_at": "2025-01-01T00:03:00Z",
            "fresh_until": "2026-01-01T00:00:00Z"
        }
    });
    let mut graph = json!({
        "schema_version": GRAPH_V01,
        "graph_id": "legacy-ieac-graph-0001",
        "captured_at": "2025-01-01T00:04:00Z",
        "graph_snapshot_sha256": "",
        "nodes": [node],
        "edges": [],
        "claim_ceiling": "record_only",
        "action_authority": "none"
    });
    let graph_sha = canonical_sha256(&graph, "graph_snapshot_sha256").unwrap_or_default();
    graph["graph_snapshot_sha256"] = json!(graph_sha);

    let event = json!({
        "schema_version": EVENT_V01,
        "event_id": "legacy-invalidation-event-0001",
        "issuer_id": "urn:luminous:synthetic-owner:legacy",
        "authority_sequence": 1,
        "previous_event_sha256": null,
        "statement_sha256": sha256_hex(b"legacy-invalidation-statement"),
        "cause_class": "source_corrected",
        "targets": [{
            "node_id": node_id,
            "node_type": "source_measurement",
            "artifact_sha256": artifact
        }],
        "scope": {
            "domains": ["habitat"],
            "subject_pseudonyms": ["synthetic-dwelling-01"],
            "purposes": ["engineering-assurance"]
        },
        "valid_time": {
            "effective_at": "2025-01-01T00:05:00Z",
            "effective_until": null,
            "effective_time_confidence": "verified"
        },
        "system_time": {
            "recorded_at": "2025-01-01T00:05:00Z",
            "trusted_at": "2025-01-01T00:05:01Z",
            "trusted_timestamp_receipt_sha256": sha256_hex(b"legacy-timestamp-receipt")
        },
        "invalidation_semantics": "reverify_required",
        "evidence_references": [{
            "evidence_id": "legacy-evidence-0001",
            "artifact_sha256": sha256_hex(b"legacy-evidence"),
            "media_type": "application/json"
        }],
        "authority": {
            "authority_id": "urn:luminous:synthetic-owner:legacy",
            "key_id": "legacy-key-001",
            "scope_policy_sha256": sha256_hex(b"legacy-scope-policy"),
            "authorization_receipt_sha256": sha256_hex(b"legacy-authorization"),
            "independent_review": "approved"
        },
        "dispute_state": "undisputed",
        "action_authority": "none"
    });
    let event_bytes = serde_json::to_vec(&event).expect("legacy event canonical serialization");
    let report = json!({
        "schema_version": REPORT_V01,
        "report_id": "legacy-impact-report-0001",
        "graph_snapshot_sha256": graph["graph_snapshot_sha256"],
        "invalidation_event_sha256": sha256_hex(&event_bytes),
        "impact_policy_sha256": sha256_hex(b"legacy-impact-policy"),
        "impact_engine_build_sha256": sha256_hex(b"legacy-impact-engine"),
        "evaluated_at": "2025-01-01T00:06:00Z",
        "results": [{
            "node_id": node_id,
            "node_type": "source_measurement",
            "artifact_sha256": artifact,
            "impact_class": "reverify_required",
            "historical_authenticity": "valid_at_time",
            "current_reliance": "reverify_required",
            "dependency_path": [],
            "reason_codes": ["SOURCE_CORRECTED"]
        }],
        "summary": {
            "node_count": 1,
            "no_impact_count": 0,
            "historical_record_only_count": 0,
            "reverify_required_count": 1,
            "claim_invalidated_count": 0,
            "quarantined_pending_review_count": 0
        },
        "historical_record_preserved": true,
        "not_checked": ["issuer authentication", "engineering correctness"],
        "claim_ceiling": "record_only",
        "action_authority": "none"
    });
    (graph, event, report)
}

fn reseal_event(event: &mut Value, report: &mut Value) {
    event["event_sha256"] = json!("");
    let digest = canonical_sha256(event, "event_sha256").unwrap_or_default();
    event["event_sha256"] = json!(digest);
    report["invalidation_event_sha256"] = json!(digest);
}

pub fn build_triplet(mutation: &str) -> (Value, Value, Value) {
    if mutation == "legacy_v0_1_triplet" {
        return base_v01();
    }
    let mut triplet = base_v02();
    let (graph, event, report) = (&mut triplet.0, &mut triplet.1, &mut triplet.2);

    match mutation {
        "valid_engineering_artifact" | "valid_engineering_receipt" => {
            let node_type = if mutation == "valid_engineering_artifact" {
                "engineering_artifact"
            } else {
                "engineering_receipt"
            };
            let node = make_node(
                node_type,
                "native-record-001",
                "revision-1",
                "{\"service\":\"water\"}",
            );
            graph["nodes"][0] = node.clone();
            event["targets"][0] = target_from_node(&node);
            report["results"][0] = result_from_node(&node);
        }
        "event_identity_mismatch" => {
            event["targets"][0]["identity_sha256"] = json!(sha256_hex(b"wrong-identity"));
        }
        "event_artifact_mismatch" => {
            event["targets"][0]["artifact_sha256"] = json!(sha256_hex(b"wrong-payload"));
        }
        "event_type_mismatch" => {
            event["targets"][0]["node_type"] = json!("source_measurement");
        }
        "unsupported_graph_node_type" => {
            graph["nodes"][0]["node_type"] = json!("unknown_future_node");
        }
        "absent_target" => {
            event["targets"][0]["node_id"] = json!(format!(
                "source_evidence:identity-sha256:{}",
                sha256_hex(b"absent")
            ));
        }
        "event_graph_snapshot_mismatch" => {
            event["graph_snapshot_sha256"] = json!(sha256_hex(b"other-graph"));
        }
        "report_graph_snapshot_mismatch" => {
            report["graph_snapshot_sha256"] = json!(sha256_hex(b"other-graph"));
        }
        "report_event_digest_mismatch" => {
            report["invalidation_event_sha256"] = json!(sha256_hex(b"other-event"));
        }
        "report_identity_mismatch" => {
            report["results"][0]["identity_sha256"] = json!(sha256_hex(b"wrong-result-identity"));
        }
        "report_artifact_mismatch" => {
            report["results"][0]["artifact_sha256"] = json!(sha256_hex(b"wrong-result-payload"));
        }
        "native_component_mutated" => {
            graph["nodes"][0]["identity"]["components"][0]["value"] = json!("different-native-id");
        }
        "duplicate_identity_component" => {
            let component = graph["nodes"][0]["identity"]["components"][0].clone();
            graph["nodes"][0]["identity"]["components"]
                .as_array_mut()
                .expect("identity components")
                .push(component);
        }
        "mixed_v02_graph_v01_event" => {
            event["schema_version"] = json!(EVENT_V01);
        }
        "mixed_v01_graph_v02_event" => {
            graph["schema_version"] = json!(GRAPH_V01);
        }
        "retroactive_unknown_effective_time" => {
            event["invalidation_semantics"] = json!("retroactive_from_effective_time");
            event["valid_time"]["effective_time_confidence"] = json!("unknown");
        }
        "pending_review_with_scitt_receipt" => {
            event["authority"]["independent_review"] = json!("pending");
            event["evidence_references"][0]["media_type"] = json!("application/scitt-receipt+json");
        }
        "duplicate_graph_node_identity" => {
            let duplicate = graph["nodes"][0].clone();
            graph["nodes"]
                .as_array_mut()
                .expect("graph nodes")
                .push(duplicate);
        }
        "report_extra_unbound_node" => {
            let mut extra = report["results"][0].clone();
            extra["node_id"] = json!(format!(
                "engineering_receipt:identity-sha256:{}",
                sha256_hex(b"not-present")
            ));
            extra["node_type"] = json!("engineering_receipt");
            extra["identity_sha256"] = json!(sha256_hex(b"not-present-identity"));
            extra["artifact_sha256"] = json!(sha256_hex(b"not-present-payload"));
            report["results"]
                .as_array_mut()
                .expect("report results")
                .push(extra);
            report["summary"]["node_count"] = json!(2);
            report["summary"]["reverify_required_count"] = json!(2);
        }
        "missing_impact_result_for_existing_target" => {
            report["results"] = json!([]);
            report["summary"]["node_count"] = json!(0);
            report["summary"]["reverify_required_count"] = json!(0);
        }
        "legacy_v01_report_in_v02_triplet" => {
            report["schema_version"] = json!(REPORT_V01);
        }
        "component_order_permuted" => {
            graph["nodes"][0]["identity"]["components"]
                .as_array_mut()
                .expect("identity components")
                .reverse();
        }
        "same_payload_distinct_native_identities" => {
            let mut second = make_node(
                "source_evidence",
                "native-record-002",
                "revision-1",
                "{\"service\":\"water\"}",
            );
            second["artifact_sha256"] = graph["nodes"][0]["artifact_sha256"].clone();
            let identity = identity_digest(&second).expect("second identity");
            second["identity_sha256"] = json!(identity);
            second["node_id"] = json!(format!("source_evidence:identity-sha256:{identity}"));
            graph["nodes"]
                .as_array_mut()
                .expect("graph nodes")
                .push(second);
        }
        _ => {}
    }

    // Recompute outer digests so each negative control isolates the intended
    // binding failure instead of failing merely because a fixture checksum is stale.
    graph["graph_snapshot_sha256"] = json!("");
    let graph_sha = canonical_sha256(graph, "graph_snapshot_sha256").unwrap_or_default();
    graph["graph_snapshot_sha256"] = json!(graph_sha);
    if mutation != "event_graph_snapshot_mismatch" {
        event["graph_snapshot_sha256"] = json!(graph_sha);
    }
    report["graph_snapshot_sha256"] = json!(graph_sha);
    reseal_event(event, report);
    if mutation == "report_graph_snapshot_mismatch" {
        report["graph_snapshot_sha256"] = json!(sha256_hex(b"other-graph"));
    }
    if mutation == "report_event_digest_mismatch" {
        report["invalidation_event_sha256"] = json!(sha256_hex(b"other-event"));
    }
    (triplet.0, triplet.1, triplet.2)
}

pub fn evaluate_mutation(mutation: &str) -> Evaluation {
    let (graph, event, report) = build_triplet(mutation);
    evaluate_triplet(&graph, &event, &report)
}

pub fn evaluate_corpus_case(case: &Value) -> Value {
    let evaluation = evaluate_mutation(text(case, "mutation"));
    json!({
        "case_id": text(case, "case_id"),
        "mutation": text(case, "mutation"),
        "accepted": evaluation.accepted,
        "compatibility": evaluation.compatibility,
        "reason_codes": evaluation.reason_codes
    })
}

pub fn validate_corpus(corpus: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if text(corpus, "corpus_id") != CORPUS_ID
        || text(corpus, "schema_version") != "0.1.0"
        || text(corpus, "classification") != "synthetic_fixture_only"
        || text(corpus, "claim_ceiling") != CLAIM_CEILING
    {
        errors.push("CORPUS_ID_VERSION_CLASSIFICATION_OR_CLAIM_CEILING_INVALID".to_owned());
    }
    let cases = array(corpus, "cases");
    if cases.len() != 25 {
        errors.push("CORPUS_CASE_COUNT_NOT_25".to_owned());
    }
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = text(case, "case_id");
        if id.is_empty() || !seen.insert(id.to_owned()) {
            errors.push("CORPUS_CASE_ID_EMPTY_OR_DUPLICATE".to_owned());
        }
        if text(case, "mutation").is_empty() {
            errors.push(format!("{id}:MUTATION_MISSING"));
        }
        if !case.get("expected").is_some_and(Value::is_object)
            || case["expected"]
                .get("accepted")
                .and_then(Value::as_bool)
                .is_none()
            || !case["expected"]
                .get("reason_codes")
                .is_some_and(Value::is_array)
        {
            errors.push(format!("{id}:EXPECTED_OUTPUT_INVALID"));
        }
    }
    errors.sort();
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS: &str = include_str!(
        "../../../docs/research/ieac-identity-compat/IEAC_INVALIDATION_IDENTITY_COMPAT_CORPUS_V0_1.json"
    );

    fn corpus() -> Value {
        serde_json::from_str(CORPUS).expect("frozen compatibility corpus parses")
    }

    #[test]
    fn all_frozen_identity_compatibility_cases_match() {
        let data = corpus();
        let errors = validate_corpus(&data);
        assert!(errors.is_empty(), "{errors:?}");
        for case in array(&data, "cases") {
            let actual = evaluate_corpus_case(case);
            assert!(
                actual == case["expected"],
                "{} actual={} expected={}",
                text(case, "case_id"),
                actual,
                case["expected"]
            );
        }
    }

    #[test]
    fn v0_1_triplet_is_preserved_without_coercion() {
        let result = evaluate_mutation("legacy_v0_1_triplet");
        assert!(result.accepted);
        assert_eq!(
            result.compatibility,
            "legacy_v0_1_preserved_without_coercion"
        );
        assert!(result.reason_codes.is_empty());
    }

    #[test]
    fn identical_payloads_do_not_merge_different_native_identities() {
        let (graph, event, report) = build_triplet("same_payload_distinct_native_identities");
        assert_eq!(array(&graph, "nodes").len(), 2);
        assert_eq!(
            graph["nodes"][0]["artifact_sha256"],
            graph["nodes"][1]["artifact_sha256"]
        );
        assert_ne!(
            graph["nodes"][0]["identity_sha256"],
            graph["nodes"][1]["identity_sha256"]
        );
        assert!(evaluate_triplet(&graph, &event, &report).accepted);
    }

    #[test]
    fn component_input_order_does_not_change_the_native_identity() {
        assert!(evaluate_mutation("component_order_permuted").accepted);
    }

    #[test]
    fn scitt_receipt_does_not_replace_owner_authority_review() {
        let result = evaluate_mutation("pending_review_with_scitt_receipt");
        assert!(!result.accepted);
        assert!(
            result
                .reason_codes
                .contains(&"INVALIDATION_AUTHORITY_INSUFFICIENT".to_owned())
        );
    }
}
