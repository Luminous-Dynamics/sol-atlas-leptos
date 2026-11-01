use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fmt;
use std::fs;
use std::process::ExitCode;

const GRAPH_SCHEMA: &str = "luminous.ieac-assurance-dependency-graph.v0.1";
const EVENT_SCHEMA: &str = "luminous.ieac-assurance-invalidation-event.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-assurance-impact-report.v0.1";

#[derive(Debug)]
enum StrictJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<StrictJson>),
    Object(BTreeMap<String, StrictJson>),
}

impl StrictJson {
    fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(value),
            Self::Number(value) => Value::Number(value),
            Self::String(value) => Value::String(value),
            Self::Array(values) => Value::Array(values.into_iter().map(Self::into_value).collect()),
            Self::Object(values) => {
                let mut object = Map::new();
                for (key, value) in values {
                    object.insert(key, value.into_value());
                }
                Value::Object(object)
            }
        }
    }
}

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrictVisitor;

        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON with unique object member names")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Null)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Null)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Number(Number::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Number(Number::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Number::from_f64(value)
                    .map(StrictJson::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::String(value))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<StrictJson>()? {
                    values.push(value);
                }
                Ok(StrictJson::Array(values))
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, StrictJson>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!(
                            "duplicate JSON object member: {key}"
                        )));
                    }
                    values.insert(key, value);
                }
                Ok(StrictJson::Object(values))
            }
        }

        deserializer.deserialize_any(StrictVisitor)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn canonical_json_digest(value: &Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("cannot serialize canonical JSON: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn required_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| format!("{context}: required non-empty string '{key}' is missing"))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn scope_matches(node: &Value, event_scope: &Value) -> bool {
    let Some(node_scope) = node.get("scope") else {
        return false;
    };
    let Some(domain) = node_scope.get("domain").and_then(Value::as_str) else {
        return false;
    };
    let Some(domains) = event_scope.get("domains").and_then(Value::as_array) else {
        return false;
    };
    if !domains.iter().any(|item| item.as_str() == Some(domain)) {
        return false;
    }

    let node_subjects = node_scope
        .get("subject_pseudonyms")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let event_subjects = event_scope
        .get("subject_pseudonyms")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !event_subjects.is_empty()
        && !node_subjects
            .iter()
            .any(|item| event_subjects.contains(item))
    {
        return false;
    }

    let node_purposes = node_scope
        .get("purposes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let event_purposes = event_scope
        .get("purposes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    event_purposes.is_empty()
        || node_purposes
            .iter()
            .any(|item| event_purposes.contains(item))
}

fn validate_graph(graph: &Value) -> Result<BTreeMap<String, Value>, String> {
    if graph.get("schema_version").and_then(Value::as_str) != Some(GRAPH_SCHEMA) {
        return Err("unsupported dependency graph schema".into());
    }

    let claimed_digest = required_string(graph, "graph_snapshot_sha256", "graph")?;
    if !valid_sha256(claimed_digest) {
        return Err("graph_snapshot_sha256 is not lowercase SHA-256".into());
    }
    let mut digest_input = graph.clone();
    digest_input
        .as_object_mut()
        .ok_or_else(|| "graph must be an object".to_owned())?
        .remove("graph_snapshot_sha256");
    let actual_digest = canonical_json_digest(&digest_input)?;
    if actual_digest != claimed_digest {
        return Err(format!(
            "graph snapshot digest mismatch: expected {claimed_digest}, computed {actual_digest}"
        ));
    }

    let rows = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "graph.nodes must be an array".to_owned())?;
    let mut nodes = BTreeMap::new();
    for (index, node) in rows.iter().enumerate() {
        let context = format!("graph.nodes[{index}]");
        let node_id = required_string(node, "node_id", &context)?;
        let node_type = required_string(node, "node_type", &context)?;
        let artifact_digest = required_string(node, "artifact_sha256", &context)?;
        if !valid_sha256(artifact_digest) {
            return Err(format!("{context}: artifact_sha256 is invalid"));
        }
        let expected_id = format!("{node_type}:sha256:{artifact_digest}");
        if node_id != expected_id {
            return Err(format!(
                "{context}: typed node ID does not match node type and artifact digest"
            ));
        }
        if nodes.insert(node_id.to_owned(), node.clone()).is_some() {
            return Err(format!("duplicate dependency node ID: {node_id}"));
        }
    }

    let edges = graph
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| "graph.edges must be an array".to_owned())?;
    let mut edge_ids = BTreeSet::new();
    let mut edge_pairs = BTreeSet::new();
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut indegree: BTreeMap<String, usize> =
        nodes.keys().map(|key| (key.clone(), 0)).collect();

    for (index, edge) in edges.iter().enumerate() {
        let context = format!("graph.edges[{index}]");
        let edge_id = required_string(edge, "edge_id", &context)?;
        let from = required_string(edge, "from_node_id", &context)?;
        let to = required_string(edge, "to_node_id", &context)?;
        if !edge_ids.insert(edge_id.to_owned()) {
            return Err(format!("duplicate dependency edge ID: {edge_id}"));
        }
        if from == to {
            return Err(format!("{context}: self-edge is invalid"));
        }
        if !edge_pairs.insert((from.to_owned(), to.to_owned())) {
            return Err(format!("{context}: duplicate directed dependency edge"));
        }
        if !nodes.contains_key(from) || !nodes.contains_key(to) {
            return Err(format!("{context}: edge endpoint does not resolve"));
        }
        adjacency
            .entry(from.to_owned())
            .or_default()
            .push(to.to_owned());
        *indegree
            .get_mut(to)
            .ok_or_else(|| format!("{context}: missing indegree entry"))? += 1;
    }

    for destinations in adjacency.values_mut() {
        destinations.sort();
        destinations.dedup();
    }

    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter_map(|(node, degree)| (*degree == 0).then_some(node.clone()))
        .collect();
    let mut visited = 0usize;
    while let Some(node) = ready.pop_first() {
        visited += 1;
        if let Some(destinations) = adjacency.get(&node) {
            for destination in destinations {
                let degree = indegree
                    .get_mut(destination)
                    .ok_or_else(|| "dependency graph has an unknown edge endpoint".to_owned())?;
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(destination.clone());
                }
            }
        }
    }
    if visited != nodes.len() {
        return Err("dependency graph contains a cycle".into());
    }

    Ok(nodes)
}

fn cause_allows(cause: &str, node_type: &str) -> bool {
    match cause {
        "issuer_key_compromised" | "issuer_key_revoked_or_rotated" => node_type == "issuer_key",
        "transparency_service_key_compromised" => node_type == "transparency_service_key",
        "trust_root_changed" => node_type == "trust_bundle",
        "source_corrected" => node_type == "source_measurement",
        "rights_revoked_or_expired" => {
            matches!(node_type, "source_measurement" | "authority_grant" | "signed_statement")
        }
        "model_superseded" => node_type == "model",
        "schema_superseded" => node_type == "schema",
        "verifier_vulnerable" => node_type == "verifier_build",
        "policy_changed" => node_type == "policy",
        "authority_revoked" => node_type == "authority_grant",
        "execution_corrected" => node_type == "execution",
        "outcome_corrected" => node_type == "outcome",
        "evidence_disputed" => true,
        _ => false,
    }
}

fn validate_event(event: &Value, nodes: &BTreeMap<String, Value>) -> Result<(), String> {
    if event.get("schema_version").and_then(Value::as_str) != Some(EVENT_SCHEMA) {
        return Err("unsupported invalidation event schema".into());
    }
    let sequence = event
        .get("authority_sequence")
        .and_then(Value::as_u64)
        .ok_or_else(|| "authority_sequence must be a positive integer".to_owned())?;
    let previous = event
        .get("previous_event_sha256")
        .ok_or_else(|| "previous_event_sha256 is required".to_owned())?;
    if sequence == 1 && !previous.is_null() {
        return Err("first authority event must have no predecessor".into());
    }
    if sequence > 1
        && !previous
            .as_str()
            .is_some_and(valid_sha256)
    {
        return Err("later authority event requires a predecessor digest".into());
    }

    let confidence = required_string(
        event.get("valid_time").ok_or("valid_time missing")?,
        "effective_time_confidence",
        "valid_time",
    )?;
    let semantics = required_string(event, "invalidation_semantics", "event")?;
    let dispute = required_string(event, "dispute_state", "event")?;
    if semantics == "retroactive_from_effective_time" && confidence != "verified" {
        return Err("retroactive invalidation requires a verified effective-time boundary".into());
    }
    if dispute != "undisputed" && semantics != "quarantined_pending_review" {
        return Err("disputed invalidation must remain quarantined pending review".into());
    }
    if semantics == "retroactive_from_effective_time" {
        let valid_time = event.get("valid_time").ok_or("valid_time missing")?;
        let effective_at = required_string(valid_time, "effective_at", "valid_time")?;
        let system_time = event.get("system_time").ok_or("system_time missing")?;
        let recorded_at = required_string(system_time, "recorded_at", "system_time")?;
        let trusted_at = required_string(system_time, "trusted_at", "system_time")?;
        if !effective_at.ends_with('Z')
            || !recorded_at.ends_with('Z')
            || !trusted_at.ends_with('Z')
            || trusted_at < recorded_at
        {
            return Err("retroactive event times must be ordered, normalized UTC timestamps".into());
        }
        let timestamp_receipt =
            required_string(system_time, "trusted_timestamp_receipt_sha256", "system_time")?;
        if !valid_sha256(timestamp_receipt) {
            return Err("trusted timestamp receipt digest is invalid".into());
        }
    }

    let event_scope = event.get("scope").ok_or("event scope missing")?;
    let cause = required_string(event, "cause_class", "event")?;
    let targets = event
        .get("targets")
        .and_then(Value::as_array)
        .ok_or_else(|| "event.targets must be an array".to_owned())?;
    if targets.is_empty() {
        return Err("invalidation event has no target nodes".into());
    }
    let mut target_ids = BTreeSet::new();
    for (index, target) in targets.iter().enumerate() {
        let context = format!("event.targets[{index}]");
        let node_id = required_string(target, "node_id", &context)?;
        let node_type = required_string(target, "node_type", &context)?;
        let artifact_digest = required_string(target, "artifact_sha256", &context)?;
        let node = nodes
            .get(node_id)
            .ok_or_else(|| format!("{context}: target node does not resolve"))?;
        if node.get("node_type").and_then(Value::as_str) != Some(node_type)
            || node.get("artifact_sha256").and_then(Value::as_str) != Some(artifact_digest)
        {
            return Err(format!("{context}: target type or digest does not match graph node"));
        }
        if !target_ids.insert(node_id.to_owned()) {
            return Err(format!("{context}: duplicate invalidation target"));
        }
        if !cause_allows(cause, node_type) {
            return Err(format!("{context}: cause class is incompatible with target node type"));
        }
        if !scope_matches(node, event_scope) {
            return Err(format!("{context}: invalidation authority scope does not cover target"));
        }
    }
    Ok(())
}

fn signed_statement_precedes_compromise(
    path: &[String],
    nodes: &BTreeMap<String, Value>,
    effective_at: &str,
    evaluated_at: &str,
) -> bool {
    path.iter().any(|node_id| {
        let Some(node) = nodes.get(node_id) else {
            return false;
        };
        if node.get("node_type").and_then(Value::as_str) != Some("signed_statement") {
            return false;
        }
        let trusted_at = node
            .pointer("/observation_time/trusted_at")
            .and_then(Value::as_str);
        let verification = node.get("verification");
        let verified = verification
            .and_then(|item| item.get("result"))
            .and_then(Value::as_str)
            == Some("verified");
        let fresh_until = verification
            .and_then(|item| item.get("fresh_until"))
            .and_then(Value::as_str);
        let checked_at = verification
            .and_then(|item| item.get("checked_at"))
            .and_then(Value::as_str);
        verified
            && trusted_at.is_some_and(|time| {
                time.ends_with('Z') && effective_at.ends_with('Z') && time < effective_at
            })
            && fresh_until.is_some_and(|time| time.ends_with('Z') && time >= evaluated_at)
            && checked_at.is_some_and(|time| time.ends_with('Z') && time <= evaluated_at)
    })
}

fn build_impact_report(
    graph: &Value,
    event: &Value,
    expected_metadata: &Value,
    nodes: &BTreeMap<String, Value>,
) -> Result<Value, String> {
    let graph_digest = required_string(graph, "graph_snapshot_sha256", "graph")?;
    let event_digest = canonical_json_digest(event)?;
    let effective_at = required_string(
        event.get("valid_time").ok_or("event valid_time missing")?,
        "effective_at",
        "event.valid_time",
    )?;
    let evaluated_at = required_string(expected_metadata, "evaluated_at", "expected report metadata")?;
    if !evaluated_at.ends_with('Z') {
        return Err("impact evaluation time must be normalized UTC".into());
    }
    let system_time = event.get("system_time").ok_or("event system_time missing")?;
    let trusted_at = required_string(system_time, "trusted_at", "event.system_time")?;
    if trusted_at > evaluated_at || effective_at > evaluated_at {
        return Err("invalidation event was not trusted/effective at the requested evaluation time".into());
    }
    let event_scope = event.get("scope").ok_or("event scope missing")?;
    let targets = event
        .get("targets")
        .and_then(Value::as_array)
        .ok_or_else(|| "event.targets must be an array".to_owned())?;

    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for edge in graph
        .get("edges")
        .and_then(Value::as_array)
        .ok_or("graph.edges missing")?
    {
        let from = required_string(edge, "from_node_id", "edge")?;
        let to = required_string(edge, "to_node_id", "edge")?;
        adjacency
            .entry(from.to_owned())
            .or_default()
            .push(to.to_owned());
    }
    for destinations in adjacency.values_mut() {
        destinations.sort();
        destinations.dedup();
    }

    let mut paths: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for target in targets {
        let node_id = required_string(target, "node_id", "event target")?;
        paths
            .entry(node_id.to_owned())
            .or_insert_with(|| vec![node_id.to_owned()]);
        queue.push_back(node_id.to_owned());
    }
    while let Some(current) = queue.pop_front() {
        let current_path = paths
            .get(&current)
            .cloned()
            .ok_or_else(|| "impact traversal lost its current path".to_owned())?;
        if let Some(destinations) = adjacency.get(&current) {
            for destination in destinations {
                let node = nodes
                    .get(destination)
                    .ok_or_else(|| "impact traversal found an unknown node".to_owned())?;
                if !scope_matches(node, event_scope) || paths.contains_key(destination) {
                    continue;
                }
                let mut next_path = current_path.clone();
                next_path.push(destination.clone());
                paths.insert(destination.clone(), next_path);
                queue.push_back(destination.clone());
            }
        }
    }

    let rows = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or("graph.nodes missing")?;
    let mut results = Vec::with_capacity(rows.len());

    for node in rows {
        let node_id = required_string(node, "node_id", "graph node")?;
        let node_type = required_string(node, "node_type", "graph node")?;
        let artifact_digest = required_string(node, "artifact_sha256", "graph node")?;
        let Some(path) = paths.get(node_id) else {
            let reason = if scope_matches(node, event_scope) {
                "NO_DEPENDENCY_PATH"
            } else {
                "OUTSIDE_INVALIDATION_SCOPE"
            };
            results.push(json!({
                "node_id": node_id,
                "node_type": node_type,
                "artifact_sha256": artifact_digest,
                "impact_class": "no_impact",
                "historical_authenticity": "valid_at_time",
                "current_reliance": "eligible",
                "dependency_path": [],
                "reason_codes": [reason]
            }));
            continue;
        };

        let (impact_class, historical, current, reasons): (&str, &str, &str, Vec<&str>) =
            if node_type == "issuer_key" {
                (
                    "claim_invalidated",
                    "unknown",
                    "ineligible",
                    vec!["ISSUER_KEY_COMPROMISED", "CURRENT_KEY_TRUST_WITHDRAWN"],
                )
            } else if signed_statement_precedes_compromise(path, nodes, effective_at, evaluated_at) {
                (
                    "historical_record_only",
                    "valid_at_time",
                    "eligible",
                    vec![
                        "TRUSTED_REGISTRATION_PRECEDES_EFFECTIVE_COMPROMISE",
                        "PRESERVE_HISTORICAL_RECORD",
                    ],
                )
            } else if matches!(
                node_type,
                "signed_statement" | "transparency_receipt" | "decision"
            ) {
                let reason = match node_type {
                    "signed_statement" => "SIGNATURE_TIME_NOT_ESTABLISHED",
                    "transparency_receipt" => "ISSUER_SIGNATURE_TIME_NOT_ESTABLISHED",
                    _ => "DECISION_DEPENDS_ON_QUARANTINED_STATEMENT",
                };
                (
                    "quarantined_pending_review",
                    "unknown",
                    "quarantined",
                    vec![
                        "REGISTRATION_AFTER_COMPROMISE_BOUNDARY",
                        reason,
                        "QUARANTINE_UNTIL_REVERIFIED",
                    ],
                )
            } else {
                let reason = match node_type {
                    "action_authorization" => "AUTHORIZATION_DEPENDS_ON_QUARANTINED_DECISION",
                    "execution" => "PHYSICAL_EXECUTION_RECORD_PRESERVED",
                    "outcome" => "OUTCOME_OBSERVATION_PRESERVED",
                    _ => "DEPENDENT_CLAIM_REQUIRES_REVERIFICATION",
                };
                let trailing = match node_type {
                    "action_authorization" => "REVIEW_EXECUTION_SCOPE",
                    "execution" => "POST_EVENT_REVIEW_REQUIRED",
                    "outcome" => "ATTRIBUTION_REQUIRES_REVALIDATION",
                    _ => "CURRENT_RELIANCE_REQUIRES_REVIEW",
                };
                (
                    "reverify_required",
                    "valid_at_time",
                    "quarantined",
                    vec![reason, trailing],
                )
            };

        results.push(json!({
            "node_id": node_id,
            "node_type": node_type,
            "artifact_sha256": artifact_digest,
            "impact_class": impact_class,
            "historical_authenticity": historical,
            "current_reliance": current,
            "dependency_path": path,
            "reason_codes": reasons
        }));
    }

    let count = |class: &str| -> usize {
        results
            .iter()
            .filter(|item| item["impact_class"].as_str() == Some(class))
            .count()
    };
    let summary = json!({
        "node_count": results.len(),
        "no_impact_count": count("no_impact"),
        "historical_record_only_count": count("historical_record_only"),
        "reverify_required_count": count("reverify_required"),
        "claim_invalidated_count": count("claim_invalidated"),
        "quarantined_pending_review_count": count("quarantined_pending_review")
    });

    Ok(json!({
        "schema_version": REPORT_SCHEMA,
        "report_id": format!("impact-{}", required_string(event, "event_id", "event")?),
        "graph_snapshot_sha256": graph_digest,
        "invalidation_event_sha256": event_digest,
        "impact_policy_sha256": required_string(expected_metadata, "impact_policy_sha256", "expected report metadata")?,
        "impact_engine_build_sha256": required_string(expected_metadata, "impact_engine_build_sha256", "expected report metadata")?,
        "evaluated_at": required_string(expected_metadata, "evaluated_at", "expected report metadata")?,
        "results": results,
        "summary": summary,
        "historical_record_preserved": true,
        "not_checked": [
            "invalidation event COSE signature and revoker authorization receipt",
            "SCITT trusted timestamp receipt and proof of issuer signature creation time",
            "external key-compromise incident evidence truth and effective-time adjudication",
            "operator identity and administrative independence",
            "SLSA/in-toto provenance of this impact auditor executable",
            "source measurement truth, model fitness and physical consequence"
        ],
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none"
    }))
}

fn evaluate_fixture(fixture: &Value) -> Result<Value, String> {
    if fixture.get("fixture_schema_version").and_then(Value::as_str)
        != Some("luminous.ieac-trust-lifecycle-fixture.v0.1")
        || fixture.get("fixture_status").and_then(Value::as_str) != Some("synthetic_only")
    {
        return Err("only the frozen synthetic trust-lifecycle fixture is accepted by v0.1".into());
    }
    let graph = fixture.get("graph").ok_or("fixture.graph is missing")?;
    let event = fixture
        .get("invalidation_event")
        .ok_or("fixture.invalidation_event is missing")?;
    let nodes = validate_graph(graph)?;
    validate_event(event, &nodes)?;
    let expected = fixture
        .get("expected_impact_report")
        .ok_or("fixture.expected_impact_report is missing")?;
    let report = build_impact_report(graph, event, expected, &nodes)?;

    let expected_results = expected
        .get("results")
        .ok_or("expected report results are missing")?;
    let expected_summary = expected
        .get("summary")
        .ok_or("expected report summary is missing")?;
    if report["results"] != *expected_results {
        return Err("derived impact results do not match the frozen expected node classifications".into());
    }
    if report["summary"] != *expected_summary {
        return Err("derived impact summary does not match the frozen expected counts".into());
    }
    if report["graph_snapshot_sha256"] != expected["graph_snapshot_sha256"] {
        return Err("impact report graph digest does not match expected report binding".into());
    }
    if report["invalidation_event_sha256"] != expected["invalidation_event_sha256"] {
        return Err("impact report invalidation-event digest does not match expected binding".into());
    }
    for field in [
        "report_id",
        "not_checked",
        "historical_record_preserved",
        "claim_ceiling",
        "action_authority",
    ] {
        if report[field] != expected[field] {
            return Err(format!("impact report field '{field}' does not match the frozen expected contract"));
        }
    }
    Ok(report)
}

fn run(input_path: &str, output_path: &str) -> Result<(), String> {
    let bytes = fs::read(input_path).map_err(|error| format!("cannot read fixture: {error}"))?;
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid or duplicate-key JSON fixture: {error}"))?;
    let fixture = parsed.into_value();
    let report = evaluate_fixture(&fixture)?;
    let encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode impact report: {error}"))?;
    fs::write(output_path, encoded).map_err(|error| format!("cannot write impact report: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: industry-trust-lifecycle-audit <fixture.json> <impact-report.json>");
        return ExitCode::from(4);
    }
    match run(&args[1], &args[2]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("trust-lifecycle impact audit failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../schemas/examples/ieac-trust-lifecycle-key-compromise-synthetic-v0.1.json"
        ))
        .unwrap()
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        assert!(serde_json::from_str::<StrictJson>(r#"{"a":1,"a":2}"#).is_err());
    }

    #[test]
    fn frozen_key_compromise_fixture_replays_to_expected_report() {
        let fixture = fixture();
        let report = evaluate_fixture(&fixture).unwrap();
        assert_eq!(report["summary"]["node_count"], 17);
        assert_eq!(report["summary"]["claim_invalidated_count"], 1);
        assert_eq!(report["summary"]["quarantined_pending_review_count"], 3);
        assert_eq!(report["summary"]["reverify_required_count"], 3);
        assert_eq!(report["summary"]["historical_record_only_count"], 6);
        assert_eq!(report["summary"]["no_impact_count"], 4);
    }

    #[test]
    fn graph_snapshot_mutation_is_rejected() {
        let mut fixture = fixture();
        fixture["graph"]["nodes"][0]["issuer_id"] = json!("urn:example:attacker");
        assert!(evaluate_fixture(&fixture).is_err());
    }

    #[test]
    fn dependency_cycle_is_rejected() {
        let mut fixture = fixture();
        let key_id = fixture["graph"]["nodes"][0]["node_id"].as_str().unwrap().to_owned();
        let terminal_outcome_id = fixture["graph"]["nodes"][14]["node_id"].as_str().unwrap().to_owned();
        fixture["graph"]["edges"].as_array_mut().unwrap().push(json!({
            "edge_id":"edge-cycle-test",
            "from_node_id":terminal_outcome_id,
            "to_node_id":key_id,
            "relation":"revoked_by"
        }));
        // Recompute the graph digest so the cycle check, not only the digest check, is exercised.
        let mut core = fixture["graph"].clone();
        core.as_object_mut().unwrap().remove("graph_snapshot_sha256");
        fixture["graph"]["graph_snapshot_sha256"] = json!(canonical_json_digest(&core).unwrap());
        assert!(evaluate_fixture(&fixture).is_err());
    }

    #[test]
    fn invalidation_cause_must_match_target_type() {
        let mut fixture = fixture();
        fixture["invalidation_event"]["cause_class"] = json!("model_superseded");
        assert!(evaluate_fixture(&fixture).is_err());
    }

    #[test]
    fn invalidation_scope_must_cover_target_node() {
        let mut fixture = fixture();
        fixture["graph"]["nodes"][0]["scope"]["domain"] = json!("water");
        let mut core = fixture["graph"].clone();
        core.as_object_mut().unwrap().remove("graph_snapshot_sha256");
        fixture["graph"]["graph_snapshot_sha256"] = json!(canonical_json_digest(&core).unwrap());
        assert!(evaluate_fixture(&fixture).is_err());
    }

    #[test]
    fn uncertain_effective_time_cannot_be_retroactive() {
        let mut fixture = fixture();
        fixture["invalidation_event"]["valid_time"]["effective_time_confidence"] = json!("unknown");
        assert!(evaluate_fixture(&fixture).is_err());
    }

    #[test]
    fn disputed_event_must_be_quarantined() {
        let mut fixture = fixture();
        fixture["invalidation_event"]["dispute_state"] = json!("disputed");
        assert!(evaluate_fixture(&fixture).is_err());
    }
}
