use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fmt;
use std::fs;
use std::io::Read;
use std::process::ExitCode;

pub(crate) const GRAPH_SCHEMA: &str = "luminous.ieac-assurance-dependency-graph.v0.1";
pub(crate) const EVENT_SCHEMA: &str = "luminous.ieac-assurance-invalidation-event.v0.1";
pub(crate) const REPORT_SCHEMA: &str = "luminous.ieac-assurance-impact-report.v0.1";
const MAX_INPUT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_GRAPH_NODES: usize = 100_000;
const MAX_GRAPH_EDGES: usize = 500_000;
const MAX_EVENT_TARGETS: usize = 10_000;
const MAX_AUTHORITY_EVENTS_PER_BATCH: usize = 10_000;

#[derive(Debug)]
pub(crate) enum StrictJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<StrictJson>),
    Object(BTreeMap<String, StrictJson>),
}

impl StrictJson {
    pub(crate) fn into_value(self) -> Value {
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

pub(crate) fn canonical_json_digest(value: &Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("cannot serialize canonical JSON: {error}"))?;
    Ok(sha256_hex(&bytes))
}

pub(crate) fn required_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| format!("{context}: required non-empty string '{key}' is missing"))
}

pub(crate) fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// Accept only canonical UTC whole-second timestamps (YYYY-MM-DDTHH:MM:SSZ).
/// Lexicographic ordering is chronological only after this normalization.
pub(crate) fn valid_utc_timestamp(value: &str) -> bool {
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

    let number = |start: usize, end: usize| -> Option<u32> {
        bytes[start..end].iter().try_fold(0u32, |value, byte| {
            if byte.is_ascii_digit() {
                Some(value * 10 + u32::from(*byte - b'0'))
            } else {
                None
            }
        })
    };
    let Some(year) = number(0, 4) else {
        return false;
    };
    let Some(month) = number(5, 7) else {
        return false;
    };
    let Some(day) = number(8, 10) else {
        return false;
    };
    let Some(hour) = number(11, 13) else {
        return false;
    };
    let Some(minute) = number(14, 16) else {
        return false;
    };
    let Some(second) = number(17, 19) else {
        return false;
    };

    if year == 0
        || !(1..=12).contains(&month)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return false;
    }
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days_in_month).contains(&day)
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

pub(crate) fn validate_graph(graph: &Value) -> Result<BTreeMap<String, Value>, String> {
    if graph.get("schema_version").and_then(Value::as_str) != Some(GRAPH_SCHEMA) {
        return Err("unsupported dependency graph schema".into());
    }

    // Check declared collection budgets before cloning/canonicalizing the graph.
    let rows = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "graph.nodes must be an array".to_owned())?;
    if rows.len() > MAX_GRAPH_NODES {
        return Err(format!(
            "graph node count {} exceeds v0.1 limit {MAX_GRAPH_NODES}",
            rows.len()
        ));
    }
    let edges = graph
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| "graph.edges must be an array".to_owned())?;
    if edges.len() > MAX_GRAPH_EDGES {
        return Err(format!(
            "graph edge count {} exceeds v0.1 limit {MAX_GRAPH_EDGES}",
            edges.len()
        ));
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

    let mut edge_ids = BTreeSet::new();
    let mut edge_pairs = BTreeSet::new();
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut indegree: BTreeMap<String, usize> = nodes.keys().map(|key| (key.clone(), 0)).collect();

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
            matches!(
                node_type,
                "source_measurement" | "authority_grant" | "signed_statement"
            )
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

pub(crate) fn validate_event(event: &Value, nodes: &BTreeMap<String, Value>) -> Result<(), String> {
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
    if sequence > 1 && !previous.as_str().is_some_and(valid_sha256) {
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
        if !valid_utc_timestamp(effective_at)
            || !valid_utc_timestamp(recorded_at)
            || !valid_utc_timestamp(trusted_at)
            || trusted_at < recorded_at
        {
            return Err(
                "retroactive event times must be ordered, normalized UTC timestamps".into(),
            );
        }
        let timestamp_receipt = required_string(
            system_time,
            "trusted_timestamp_receipt_sha256",
            "system_time",
        )?;
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
    if targets.len() > MAX_EVENT_TARGETS {
        return Err(format!(
            "invalidation target count {} exceeds v0.1 limit {MAX_EVENT_TARGETS}",
            targets.len()
        ));
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
            return Err(format!(
                "{context}: target type or digest does not match graph node"
            ));
        }
        if !target_ids.insert(node_id.to_owned()) {
            return Err(format!("{context}: duplicate invalidation target"));
        }
        if !cause_allows(cause, node_type) {
            return Err(format!(
                "{context}: cause class is incompatible with target node type"
            ));
        }
        if !scope_matches(node, event_scope) {
            return Err(format!(
                "{context}: invalidation authority scope does not cover target"
            ));
        }
    }
    Ok(())
}

/// Verify a complete authority event chain and return its canonical sequence order.
///
/// Exact duplicate events are idempotent. Conflicting events at the same sequence,
/// reused event IDs, sequence gaps, broken predecessor digests, or backwards
/// recorded/trusted time are rejected. This is an in-memory batch verifier only;
/// it is not a persistent anti-rollback high-water store.
fn validate_event_chain(
    events: &[Value],
    nodes: &BTreeMap<String, Value>,
) -> Result<Vec<Value>, String> {
    if events.len() > MAX_AUTHORITY_EVENTS_PER_BATCH {
        return Err(format!(
            "authority event batch size {} exceeds v0.1 limit {MAX_AUTHORITY_EVENTS_PER_BATCH}",
            events.len()
        ));
    }
    let mut by_sequence: BTreeMap<u64, (String, Value)> = BTreeMap::new();
    let mut event_ids: BTreeMap<String, (u64, String)> = BTreeMap::new();

    for event in events {
        validate_event(event, nodes)?;
        let event_id = required_string(event, "event_id", "event")?.to_owned();
        let sequence = event
            .get("authority_sequence")
            .and_then(Value::as_u64)
            .ok_or_else(|| "authority_sequence must be a positive integer".to_owned())?;
        let digest = canonical_json_digest(event)?;

        if let Some((existing_sequence, existing_digest)) = event_ids.get(&event_id) {
            if *existing_sequence != sequence || existing_digest != &digest {
                return Err(format!(
                    "event ID {event_id} is reused with a different sequence or content"
                ));
            }
        }
        if let Some((existing_digest, _)) = by_sequence.get(&sequence) {
            if existing_digest != &digest {
                return Err(format!(
                    "conflicting invalidation events share authority sequence {sequence}"
                ));
            }
            // Exact replay of an already-seen event is idempotent.
            continue;
        }
        event_ids.insert(event_id, (sequence, digest.clone()));
        by_sequence.insert(sequence, (digest, event.clone()));
    }

    let mut ordered = Vec::with_capacity(by_sequence.len());
    let mut previous_digest: Option<String> = None;
    let mut previous_recorded_at: Option<String> = None;
    let mut previous_trusted_at: Option<String> = None;

    for (index, (sequence, (digest, event))) in by_sequence.into_iter().enumerate() {
        let expected_sequence =
            u64::try_from(index + 1).map_err(|_| "event chain index exceeds u64".to_owned())?;
        if sequence != expected_sequence {
            return Err(format!(
                "authority event sequence is not contiguous: expected {expected_sequence}, got {sequence}"
            ));
        }

        let previous = event
            .get("previous_event_sha256")
            .ok_or("previous_event_sha256 is required")?;
        match previous_digest.as_deref() {
            None if !previous.is_null() => {
                return Err("first authority event must have no predecessor".into());
            }
            Some(expected) if previous.as_str() != Some(expected) => {
                return Err(format!(
                    "authority event sequence {sequence} has a broken predecessor digest"
                ));
            }
            _ => {}
        }

        let system_time = event
            .get("system_time")
            .ok_or("event system_time is required for chain ordering")?;
        let recorded_at = required_string(system_time, "recorded_at", "event.system_time")?;
        let trusted_at = required_string(system_time, "trusted_at", "event.system_time")?;
        if !valid_utc_timestamp(recorded_at) || !valid_utc_timestamp(trusted_at) {
            return Err(format!(
                "authority event sequence {sequence} requires normalized UTC recorded/trusted times"
            ));
        }
        if previous_recorded_at
            .as_deref()
            .is_some_and(|previous| recorded_at < previous)
            || previous_trusted_at
                .as_deref()
                .is_some_and(|previous| trusted_at < previous)
        {
            return Err(format!(
                "authority event sequence {sequence} moves recorded/trusted time backwards"
            ));
        }

        previous_digest = Some(digest);
        previous_recorded_at = Some(recorded_at.to_owned());
        previous_trusted_at = Some(trusted_at.to_owned());
        ordered.push(event);
    }

    Ok(ordered)
}

pub(crate) fn signed_statement_precedes_compromise(
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
                valid_utc_timestamp(time)
                    && valid_utc_timestamp(effective_at)
                    && time < effective_at
            })
            && fresh_until.is_some_and(|time| valid_utc_timestamp(time) && time >= evaluated_at)
            && checked_at.is_some_and(|time| valid_utc_timestamp(time) && time <= evaluated_at)
    })
}

pub(crate) fn build_forward_adjacency(
    graph: &Value,
) -> Result<BTreeMap<String, Vec<String>>, String> {
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
    Ok(adjacency)
}

/// Compute only the nodes reachable from the authorized event targets.
/// This is shared by the full report and sparse-delta paths so impact
/// classification semantics do not fork between the two execution modes.
pub(crate) fn build_impacted_results_with_adjacency(
    event: &Value,
    evaluated_at: &str,
    nodes: &BTreeMap<String, Value>,
    adjacency: &BTreeMap<String, Vec<String>>,
) -> Result<BTreeMap<String, Value>, String> {
    let effective_at = required_string(
        event.get("valid_time").ok_or("event valid_time missing")?,
        "effective_at",
        "event.valid_time",
    )?;
    let system_time = event
        .get("system_time")
        .ok_or("event system_time missing")?;
    let trusted_at = required_string(system_time, "trusted_at", "event.system_time")?;
    if !valid_utc_timestamp(evaluated_at)
        || !valid_utc_timestamp(trusted_at)
        || !valid_utc_timestamp(effective_at)
        || trusted_at > evaluated_at
        || effective_at > evaluated_at
    {
        return Err(
            "invalidation event was not trusted/effective at the requested evaluation time".into(),
        );
    }

    let targets = event
        .get("targets")
        .and_then(Value::as_array)
        .ok_or_else(|| "event.targets must be an array".to_owned())?;
    let cause = required_string(event, "cause_class", "event")?;
    let direct_target_ids: BTreeSet<String> = targets
        .iter()
        .map(|target| required_string(target, "node_id", "event target").map(str::to_owned))
        .collect::<Result<_, _>>()?;
    let mut paths: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for target in targets {
        let node_id = required_string(target, "node_id", "event target")?;
        if !nodes.contains_key(node_id) {
            return Err(format!("event target {node_id} is absent from validated graph index"));
        }
        if !paths.contains_key(node_id) {
            paths.insert(node_id.to_owned(), vec![node_id.to_owned()]);
            queue.push_back(node_id.to_owned());
        }
    }

    while let Some(current) = queue.pop_front() {
        let current_path = paths
            .get(&current)
            .cloned()
            .ok_or_else(|| "impact traversal lost its current path".to_owned())?;
        if let Some(destinations) = adjacency.get(&current) {
            for destination in destinations {
                if !nodes.contains_key(destination) {
                    return Err(format!("impact traversal found unknown node {destination}"));
                }
                if paths.contains_key(destination) {
                    continue;
                }
                let mut next_path = current_path.clone();
                next_path.push(destination.clone());
                paths.insert(destination.clone(), next_path);
                queue.push_back(destination.clone());
            }
        }
    }

    let mut results = BTreeMap::new();
    for (node_id, path) in paths {
        let node = nodes
            .get(&node_id)
            .ok_or_else(|| format!("impact traversal lost graph node {node_id}"))?;
        let node_type = required_string(node, "node_type", "graph node")?;
        let artifact_digest = required_string(node, "artifact_sha256", "graph node")?;
        let direct_invalidation = if direct_target_ids.contains(&node_id) {
            match (node_type, cause) {
                ("issuer_key", "issuer_key_compromised") => Some((
                    "ISSUER_KEY_COMPROMISED",
                    "CURRENT_KEY_TRUST_WITHDRAWN",
                )),
                ("issuer_key", "issuer_key_revoked_or_rotated") => Some((
                    "ISSUER_KEY_REVOKED_OR_ROTATED",
                    "CURRENT_KEY_TRUST_WITHDRAWN",
                )),
                ("transparency_service_key", "transparency_service_key_compromised") => Some((
                    "TRANSPARENCY_SERVICE_KEY_COMPROMISED",
                    "CURRENT_TS_KEY_TRUST_WITHDRAWN",
                )),
                ("trust_bundle", "trust_root_changed") => Some((
                    "TRUST_ROOT_CHANGED",
                    "CURRENT_TRUST_BUNDLE_REQUIRES_REPLACEMENT",
                )),
                ("authority_grant", "authority_revoked") => Some((
                    "AUTHORITY_GRANT_REVOKED",
                    "CURRENT_AUTHORITY_WITHDRAWN",
                )),
                _ => None,
            }
        } else {
            None
        };
        let (impact_class, historical, current, reasons): (&str, &str, &str, Vec<&str>) =
            if let Some((direct_reason, reliance_reason)) = direct_invalidation {
                (
                    "claim_invalidated",
                    "unknown",
                    "ineligible",
                    vec![direct_reason, reliance_reason],
                )
            } else if signed_statement_precedes_compromise(&path, nodes, effective_at, evaluated_at)
            {
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

        results.insert(
            node_id.clone(),
            json!({
                "node_id": node_id,
                "node_type": node_type,
                "artifact_sha256": artifact_digest,
                "impact_class": impact_class,
                "historical_authenticity": historical,
                "current_reliance": current,
                "dependency_path": path,
                "reason_codes": reasons
            }),
        );
    }
    Ok(results)
}

pub(crate) fn build_impact_report(
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
    let evaluated_at = required_string(
        expected_metadata,
        "evaluated_at",
        "expected report metadata",
    )?;
    if !valid_utc_timestamp(evaluated_at) {
        return Err("impact evaluation time must be normalized UTC whole-second RFC 3339".into());
    }
    let system_time = event
        .get("system_time")
        .ok_or("event system_time missing")?;
    let trusted_at = required_string(system_time, "trusted_at", "event.system_time")?;
    if !valid_utc_timestamp(trusted_at)
        || !valid_utc_timestamp(effective_at)
        || trusted_at > evaluated_at
        || effective_at > evaluated_at
    {
        return Err(
            "invalidation event was not trusted/effective at the requested evaluation time".into(),
        );
    }
    let adjacency = build_forward_adjacency(graph)?;
    let impacted_results = build_impacted_results_with_adjacency(
        event,
        evaluated_at,
        nodes,
        &adjacency,
    )?;

    let rows = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or("graph.nodes missing")?;
    let mut results = Vec::with_capacity(rows.len());

    for node in rows {
        let node_id = required_string(node, "node_id", "graph node")?;
        if let Some(impact) = impacted_results.get(node_id) {
            results.push(impact.clone());
            continue;
        }
        let node_type = required_string(node, "node_type", "graph node")?;
        let artifact_digest = required_string(node, "artifact_sha256", "graph node")?;
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
    if fixture
        .get("fixture_schema_version")
        .and_then(Value::as_str)
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
    validate_event_chain(std::slice::from_ref(event), &nodes)?;
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
        return Err(
            "derived impact results do not match the frozen expected node classifications".into(),
        );
    }
    if report["summary"] != *expected_summary {
        return Err("derived impact summary does not match the frozen expected counts".into());
    }
    if report["graph_snapshot_sha256"] != expected["graph_snapshot_sha256"] {
        return Err("impact report graph digest does not match expected report binding".into());
    }
    if report["invalidation_event_sha256"] != expected["invalidation_event_sha256"] {
        return Err(
            "impact report invalidation-event digest does not match expected binding".into(),
        );
    }
    for field in [
        "report_id",
        "not_checked",
        "historical_record_preserved",
        "claim_ceiling",
        "action_authority",
    ] {
        if report[field] != expected[field] {
            return Err(format!(
                "impact report field '{field}' does not match the frozen expected contract"
            ));
        }
    }
    Ok(report)
}

fn run(input_path: &str, output_path: &str) -> Result<(), String> {
    let file =
        fs::File::open(input_path).map_err(|error| format!("cannot open fixture: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read fixture: {error}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_INPUT_BYTES {
        return Err(format!(
            "fixture input exceeds v0.1 byte limit {MAX_INPUT_BYTES}"
        ));
    }
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid or duplicate-key JSON fixture: {error}"))?;
    let fixture = parsed.into_value();
    let report = evaluate_fixture(&fixture)?;
    let encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode impact report: {error}"))?;
    fs::write(output_path, encoded)
        .map_err(|error| format!("cannot write impact report: {error}"))?;
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

    fn two_event_chain() -> (BTreeMap<String, Value>, Value, Value) {
        let fixture = fixture();
        let nodes = validate_graph(&fixture["graph"]).unwrap();
        let first = fixture["invalidation_event"].clone();
        let first_digest = canonical_json_digest(&first).unwrap();
        let mut second = first.clone();
        second["event_id"] = json!("evt-key-compromise-followup-002");
        second["authority_sequence"] = json!(2);
        second["previous_event_sha256"] = json!(first_digest);
        second["system_time"]["recorded_at"] = json!("2026-11-02T11:00:00Z");
        second["system_time"]["trusted_at"] = json!("2026-11-02T11:01:00Z");
        (nodes, first, second)
    }

    #[test]
    fn event_chain_sorts_out_of_order_delivery_and_accepts_exact_replay() {
        let (nodes, first, second) = two_event_chain();
        let ordered =
            validate_event_chain(&[second.clone(), first.clone(), first.clone()], &nodes).unwrap();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0]["authority_sequence"], 1);
        assert_eq!(ordered[1]["authority_sequence"], 2);
    }

    #[test]
    fn event_chain_rejects_conflicting_same_sequence_and_reused_ids() {
        let (nodes, first, second) = two_event_chain();
        let mut conflicting = second.clone();
        conflicting["event_id"] = json!("evt-conflicting-sequence-002");
        assert!(validate_event_chain(&[first.clone(), second, conflicting], &nodes).is_err());

        let mut reused_id = two_event_chain().2;
        reused_id["event_id"] = first["event_id"].clone();
        assert!(validate_event_chain(&[first, reused_id], &nodes).is_err());
    }

    #[test]
    fn event_chain_rejects_gaps_broken_predecessors_and_time_rollback() {
        let (nodes, first, second) = two_event_chain();
        let mut gap = second.clone();
        gap["authority_sequence"] = json!(3);
        assert!(validate_event_chain(&[first.clone(), gap], &nodes).is_err());

        let mut broken_link = second.clone();
        broken_link["previous_event_sha256"] =
            json!("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd");
        assert!(validate_event_chain(&[first.clone(), broken_link], &nodes).is_err());

        let mut backdated = second;
        backdated["system_time"]["recorded_at"] = json!("2026-10-31T00:00:00Z");
        backdated["system_time"]["trusted_at"] = json!("2026-10-31T00:00:00Z");
        assert!(validate_event_chain(&[first, backdated], &nodes).is_err());
    }

    #[test]
    fn only_canonical_utc_whole_second_timestamps_are_orderable() {
        assert!(valid_utc_timestamp("2026-11-02T21:29:00Z"));
        assert!(valid_utc_timestamp("2024-02-29T23:59:59Z"));
        assert!(!valid_utc_timestamp("2026-02-29T00:00:00Z"));
        assert!(!valid_utc_timestamp("2026-11-02T24:00:00Z"));
        assert!(!valid_utc_timestamp("2026-11-02T21:29:00+00:00"));
        assert!(!valid_utc_timestamp("2026-11-02T21:29:00.1Z"));
        assert!(!valid_utc_timestamp("2026-13-01T00:00:00Z"));
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
    fn dependency_impact_crosses_domains_but_unconnected_out_of_scope_records_stay_unaffected() {
        let mut fixture = fixture();
        let graph = &mut fixture["graph"];
        let water_source_id = graph["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| {
                node["node_type"] == "source_measurement" && node["scope"]["domain"] == "water"
            })
            .unwrap()["node_id"]
            .clone();
        let water_decision_digest =
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
        let water_decision_id = format!("decision:sha256:{water_decision_digest}");
        let disconnected_control_digest =
            "abababababababababababababababababababababababababababababababab";
        let disconnected_control_id =
            format!("source_measurement:sha256:{disconnected_control_digest}");
        graph["nodes"].as_array_mut().unwrap().extend([
            json!({
                "node_id": water_decision_id,
                "node_type": "decision",
                "artifact_sha256": water_decision_digest,
                "scope": {
                    "domain": "water",
                    "subject_pseudonyms": ["case:water:asset:delta"],
                    "purposes": ["water_operations"]
                }
            }),
            json!({
                "node_id": disconnected_control_id,
                "node_type": "source_measurement",
                "artifact_sha256": disconnected_control_digest,
                "scope": {
                    "domain": "water",
                    "subject_pseudonyms": ["case:water:asset:epsilon"],
                    "purposes": ["water_operations"]
                }
            }),
        ]);
        let issuer_key_id = graph["nodes"][0]["node_id"].clone();
        graph["edges"].as_array_mut().unwrap().extend([
            json!({
                "edge_id": "edge-energy-key-to-water-source",
                "from_node_id": issuer_key_id,
                "to_node_id": water_source_id,
                "relation": "derived_from"
            }),
            json!({
                "edge_id": "edge-cross-domain-water-decision",
                "from_node_id": water_source_id,
                "to_node_id": water_decision_id,
                "relation": "evidence_for"
            }),
        ]);

        let mut digest_input = graph.clone();
        digest_input
            .as_object_mut()
            .unwrap()
            .remove("graph_snapshot_sha256");
        graph["graph_snapshot_sha256"] = json!(canonical_json_digest(&digest_input).unwrap());

        let nodes = validate_graph(graph).unwrap();
        validate_event(&fixture["invalidation_event"], &nodes).unwrap();
        let report = build_impact_report(
            graph,
            &fixture["invalidation_event"],
            &fixture["expected_impact_report"],
            &nodes,
        )
        .unwrap();
        let results = report["results"].as_array().unwrap();
        let water_source = results
            .iter()
            .find(|row| row["node_id"] == water_source_id)
            .unwrap();
        let water_decision = results
            .iter()
            .find(|row| row["node_id"] == water_decision_id)
            .unwrap();
        assert_eq!(water_source["impact_class"], "reverify_required");
        assert_eq!(water_decision["impact_class"], "quarantined_pending_review");

        let disconnected_water_control = results
            .iter()
            .find(|row| row["node_id"] == disconnected_control_id)
            .expect("disconnected water control remains in the report");
        assert_eq!(disconnected_water_control["impact_class"], "no_impact");
        assert_eq!(
            disconnected_water_control["reason_codes"][0],
            "OUTSIDE_INVALIDATION_SCOPE"
        );
    }

    #[test]
    fn graph_node_and_edge_budgets_reject_oversized_collections() {
        let mut graph = fixture()["graph"].clone();
        graph["nodes"] = json!(vec![Value::Null; MAX_GRAPH_NODES + 1]);
        assert!(validate_graph(&graph).unwrap_err().contains("node count"));

        let mut graph = fixture()["graph"].clone();
        graph["edges"] = json!(vec![Value::Null; MAX_GRAPH_EDGES + 1]);
        assert!(validate_graph(&graph).unwrap_err().contains("edge count"));
    }

    #[test]
    fn invalidation_target_budget_rejects_oversized_event() {
        let fixture = fixture();
        let nodes = validate_graph(&fixture["graph"]).unwrap();
        let mut event = fixture["invalidation_event"].clone();
        event["targets"] = json!(vec![Value::Null; MAX_EVENT_TARGETS + 1]);
        assert!(
            validate_event(&event, &nodes)
                .unwrap_err()
                .contains("target count")
        );
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
        let key_id = fixture["graph"]["nodes"][0]["node_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let terminal_outcome_id = fixture["graph"]["nodes"][14]["node_id"]
            .as_str()
            .unwrap()
            .to_owned();
        fixture["graph"]["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "edge_id":"edge-cycle-test",
                "from_node_id":terminal_outcome_id,
                "to_node_id":key_id,
                "relation":"revoked_by"
            }));
        // Recompute the graph digest so the cycle check, not only the digest check, is exercised.
        let mut core = fixture["graph"].clone();
        core.as_object_mut()
            .unwrap()
            .remove("graph_snapshot_sha256");
        fixture["graph"]["graph_snapshot_sha256"] = json!(canonical_json_digest(&core).unwrap());
        assert!(evaluate_fixture(&fixture).is_err());
    }

    #[test]
    fn reachable_but_untargeted_issuer_key_is_not_directly_invalidated() {
        let mut fixture = fixture();
        let graph = &mut fixture["graph"];
        let direct_key_id = graph["nodes"][0]["node_id"].clone();
        let inherited_scope = graph["nodes"][0]["scope"].clone();
        let descendant_digest =
            "abababababababababababababababababababababababababababababababab";
        let descendant_id = format!("issuer_key:sha256:{descendant_digest}");
        graph["nodes"].as_array_mut().unwrap().push(json!({
            "node_id": descendant_id,
            "node_type": "issuer_key",
            "artifact_sha256": descendant_digest,
            "scope": inherited_scope
        }));
        graph["edges"].as_array_mut().unwrap().push(json!({
            "edge_id": "edge-direct-key-to-descendant-key",
            "from_node_id": direct_key_id,
            "to_node_id": descendant_id,
            "relation": "derived_from"
        }));

        let mut digest_input = graph.clone();
        digest_input
            .as_object_mut()
            .unwrap()
            .remove("graph_snapshot_sha256");
        graph["graph_snapshot_sha256"] = json!(canonical_json_digest(&digest_input).unwrap());

        let event = fixture["invalidation_event"].clone();
        let nodes = validate_graph(graph).unwrap();
        validate_event(&event, &nodes).unwrap();
        let report = build_impact_report(
            graph,
            &event,
            &fixture["expected_impact_report"],
            &nodes,
        )
        .unwrap();
        let rows = report["results"].as_array().unwrap();
        let direct = rows
            .iter()
            .find(|row| row["node_id"] == direct_key_id)
            .unwrap();
        let descendant = rows
            .iter()
            .find(|row| row["node_id"] == descendant_id)
            .unwrap();
        assert_eq!(direct["impact_class"], "claim_invalidated");
        assert_eq!(descendant["impact_class"], "reverify_required");
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
        core.as_object_mut()
            .unwrap()
            .remove("graph_snapshot_sha256");
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

    fn scale_node(index: usize, node_type: &str, domain: &str) -> (String, Value) {
        let digest = sha256_hex(format!("ieac-scale-node-v0.1:{index}:{node_type}").as_bytes());
        let node_id = format!("{node_type}:sha256:{digest}");
        let node = json!({
            "node_id": node_id,
            "node_type": node_type,
            "artifact_sha256": digest,
            "scope": {
                "domain": domain,
                "subject_pseudonyms": ["bench:subject:001"],
                "purposes": ["runtime_admission_descriptive"]
            }
        });
        (node_id, node)
    }

    fn scale_graph(total_nodes: usize) -> (Value, Value, Value, usize) {
        assert!(total_nodes >= 4);
        let impacted_nodes = total_nodes * 4 / 5;
        let mut nodes = Vec::with_capacity(total_nodes);
        let mut node_ids = Vec::with_capacity(total_nodes);
        let mut edges = Vec::with_capacity(impacted_nodes.saturating_sub(1));

        let (root_id, root) = scale_node(0, "issuer_key", "energy");
        node_ids.push(root_id.clone());
        nodes.push(root);
        for index in 1..impacted_nodes {
            let domain = if index % 5 == 0 { "water" } else { "energy" };
            let (node_id, node) = scale_node(index, "execution", domain);
            let parent_index = if index == 1 { 0 } else { (index - 1) / 2 };
            edges.push(json!({
                "edge_id": format!("edge-{index:08}"),
                "from_node_id": node_ids[parent_index],
                "to_node_id": node_id,
                "relation": "derived_from"
            }));
            node_ids.push(node_id);
            nodes.push(node);
        }
        for index in impacted_nodes..total_nodes {
            let domain = if index % 2 == 0 { "water" } else { "energy" };
            let (node_id, node) = scale_node(index, "source_measurement", domain);
            node_ids.push(node_id);
            nodes.push(node);
        }

        let mut graph = json!({
            "schema_version": GRAPH_SCHEMA,
            "graph_id": format!("graph-scale-{total_nodes}"),
            "captured_at": "2026-01-03T00:00:00Z",
            "nodes": nodes,
            "edges": edges,
            "graph_snapshot_sha256": ""
        });
        let mut digest_input = graph.clone();
        digest_input
            .as_object_mut()
            .unwrap()
            .remove("graph_snapshot_sha256");
        graph["graph_snapshot_sha256"] = json!(canonical_json_digest(&digest_input).unwrap());

        let event = json!({
            "schema_version": EVENT_SCHEMA,
            "event_id": format!("event-scale-{total_nodes}"),
            "authority_sequence": 1,
            "previous_event_sha256": null,
            "cause_class": "issuer_key_compromised",
            "invalidation_semantics": "retroactive_from_effective_time",
            "dispute_state": "undisputed",
            "valid_time": {
                "effective_at": "2026-01-01T00:00:00Z",
                "effective_time_confidence": "verified"
            },
            "system_time": {
                "recorded_at": "2026-01-02T00:00:00Z",
                "trusted_at": "2026-01-02T00:00:00Z",
                "trusted_timestamp_receipt_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            },
            "scope": {
                "domains": ["energy"],
                "subject_pseudonyms": ["bench:subject:001"],
                "purposes": ["runtime_admission_descriptive"]
            },
            "targets": [{
                "node_id": root_id,
                "node_type": "issuer_key",
                "artifact_sha256": graph["nodes"][0]["artifact_sha256"]
            }]
        });
        let metadata = json!({
            "evaluated_at": "2026-01-03T00:00:00Z",
            "impact_policy_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "impact_engine_build_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        });
        (graph, event, metadata, impacted_nodes)
    }

    #[test]
    fn impact_propagation_scale_ladder_has_exact_precision_and_replay() {
        use std::time::Instant;

        for total_nodes in [100usize, 1_000, 10_000, 100_000] {
            let (graph, event, metadata, expected_affected) = scale_graph(total_nodes);
            let started = Instant::now();
            let nodes = validate_graph(&graph).expect("generated DAG must validate");
            validate_event(&event, &nodes).expect("generated invalidation event must validate");
            let report = build_impact_report(&graph, &event, &metadata, &nodes)
                .expect("impact traversal must complete");
            let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;

            let results = report["results"].as_array().unwrap();
            assert_eq!(results.len(), total_nodes);
            let mut false_positives = 0usize;
            let mut false_negatives = 0usize;
            for (index, result) in results.iter().enumerate() {
                let expected = if index == 0 {
                    "claim_invalidated"
                } else if index < expected_affected {
                    "reverify_required"
                } else {
                    "no_impact"
                };
                let actual = result["impact_class"].as_str().unwrap();
                if expected != actual {
                    if expected == "no_impact" {
                        false_positives += 1;
                    } else {
                        false_negatives += 1;
                    }
                }
            }
            assert_eq!(false_positives, 0, "false positives at {total_nodes} nodes");
            assert_eq!(false_negatives, 0, "false negatives at {total_nodes} nodes");
            assert_eq!(report["summary"]["node_count"], total_nodes);
            assert_eq!(report["summary"]["claim_invalidated_count"], 1);
            assert_eq!(
                report["summary"]["reverify_required_count"],
                expected_affected - 1
            );
            assert_eq!(
                report["summary"]["no_impact_count"],
                total_nodes - expected_affected
            );

            let report_digest = canonical_json_digest(&report).unwrap();
            drop(report);
            drop(nodes);

            let replay_nodes = validate_graph(&graph).unwrap();
            validate_event(&event, &replay_nodes).unwrap();
            let replay = build_impact_report(&graph, &event, &metadata, &replay_nodes).unwrap();
            assert_eq!(
                canonical_json_digest(&replay).unwrap(),
                report_digest,
                "non-deterministic replay at {total_nodes} nodes"
            );

            println!(
                "{}",
                json!({
                    "benchmark": "ieac_bitemporal_impact_scale_v0.1",
                    "synthetic_only": true,
                    "node_count": total_nodes,
                    "edge_count": graph["edges"].as_array().unwrap().len(),
                    "expected_affected_count": expected_affected,
                    "expected_unaffected_out_of_scope_count": total_nodes - expected_affected,
                    "false_positives": false_positives,
                    "false_negatives": false_negatives,
                    "deterministic_replay": true,
                    "measured_validation_and_impact_ms": elapsed_ms,
                    "measurement_scope": "single in-process sample; excludes fixture generation and process startup"
                })
            );
        }
    }
}
