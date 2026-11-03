#![recursion_limit = "256"]

#[allow(dead_code)]
#[path = "../main.rs"]
mod lifecycle;

use lifecycle::{
    REPORT_SCHEMA, StrictJson, build_forward_adjacency,
    build_impacted_results_with_adjacency, canonical_json_digest, required_string,
    valid_sha256, valid_utc_timestamp, validate_event, validate_graph,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process::ExitCode;

const DELTA_SCHEMA: &str = "luminous.ieac-assurance-impact-delta.v0.1";
const STATE_PIN_SCHEMA: &str = "luminous.ieac-assurance-impact-state-pin.v0.1";
const BATCH_SCHEMA: &str = "luminous.ieac-assurance-impact-delta-batch.v0.1";
const DELTA_INPUT_SCHEMA: &str = "luminous.ieac-assurance-impact-delta-input.v0.1";
const MAX_INPUT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_BATCH_EVENTS: usize = 10_000;
const MAX_BATCH_CLOSURE_NODE_MEMBERSHIPS: usize = 2_000_000;

#[derive(Debug)]
struct GraphIndex {
    graph_snapshot_sha256: String,
    index_sha256: String,
    nodes: BTreeMap<String, Value>,
    forward: BTreeMap<String, Vec<String>>,
    reverse: BTreeMap<String, Vec<String>>,
    edge_count: usize,
}

impl GraphIndex {
    fn build(graph: &Value) -> Result<Self, String> {
        let nodes = validate_graph(graph)?;
        let forward = build_forward_adjacency(graph)?;
        let mut reverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (from, destinations) in &forward {
            for to in destinations {
                reverse.entry(to.clone()).or_default().push(from.clone());
            }
        }
        for sources in reverse.values_mut() {
            sources.sort();
            sources.dedup();
        }
        for (to, sources) in &reverse {
            for from in sources {
                if !forward.get(from).is_some_and(|destinations| destinations.contains(to)) {
                    return Err("forward/reverse graph index consistency check failed".into());
                }
            }
        }

        let graph_snapshot_sha256 =
            required_string(graph, "graph_snapshot_sha256", "graph")?.to_owned();
        let edge_count = graph
            .get("edges")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        let index_identity = json!({
            "index_version": "validated-forward-reverse-adjacency-v0.1",
            "graph_snapshot_sha256": graph_snapshot_sha256.clone(),
            "node_count": nodes.len(),
            "edge_count": edge_count
        });
        let index_sha256 = canonical_json_digest(&index_identity)?;

        Ok(Self {
            graph_snapshot_sha256,
            index_sha256,
            nodes,
            forward,
            reverse,
            edge_count,
        })
    }

    fn independent_reachable_set(&self, event: &Value) -> Result<BTreeSet<String>, String> {
        // Iterative depth-first closure is intentionally different from the
        // lifecycle engine's breadth-first shortest-path traversal.
        let targets = event
            .get("targets")
            .and_then(Value::as_array)
            .ok_or("event.targets must be an array")?;
        let mut visited = BTreeSet::new();
        let mut pending = Vec::with_capacity(targets.len());
        for target in targets {
            let id = required_string(target, "node_id", "event target")?;
            if !self.nodes.contains_key(id) {
                return Err(format!("independent closure found unknown target {id}"));
            }
            if visited.insert(id.to_owned()) {
                pending.push(id.to_owned());
            }
        }

        while let Some(current) = pending.pop() {
            if let Some(destinations) = self.forward.get(&current) {
                for destination in destinations.iter().rev() {
                    if !self.nodes.contains_key(destination) {
                        return Err(format!(
                            "independent closure found unknown edge endpoint {destination}"
                        ));
                    }
                    if !self
                        .reverse
                        .get(destination)
                        .is_some_and(|sources| sources.binary_search(&current).is_ok())
                    {
                        return Err(format!(
                            "forward/reverse adjacency disagreement for edge {current} -> {destination}"
                        ));
                    }
                    if visited.insert(destination.clone()) {
                        pending.push(destination.clone());
                    }
                }
            }
        }
        Ok(visited)
    }
}

fn parse_strict(path: &Path) -> Result<Value, String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_INPUT_BYTES {
        return Err(format!(
            "input {} exceeds byte limit {MAX_INPUT_BYTES}",
            path.display()
        ));
    }
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid or duplicate-member JSON in {}: {error}", path.display()))?;
    Ok(parsed.into_value())
}

fn write_new(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("cannot encode {}: {error}", path.display()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("refusing to overwrite or cannot create {}: {error}", path.display()))?;
    file.write_all(&bytes)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("cannot sync {}: {error}", path.display()))?;
    Ok(())
}

fn report_rows(report: &Value, index: &GraphIndex) -> Result<BTreeMap<String, Value>, String> {
    if report.get("schema_version").and_then(Value::as_str) != Some(REPORT_SCHEMA) {
        return Err("base impact report has an unsupported schema version".into());
    }
    if report.get("graph_snapshot_sha256").and_then(Value::as_str)
        != Some(index.graph_snapshot_sha256.as_str())
    {
        return Err("base impact report is bound to a different graph snapshot".into());
    }
    if report.get("historical_record_preserved").and_then(Value::as_bool) != Some(true)
        || report.get("claim_ceiling").and_then(Value::as_str)
            != Some("incomplete_or_indeterminate")
        || report.get("action_authority").and_then(Value::as_str) != Some("none")
        || report.get("not_checked").and_then(Value::as_array).is_none_or(Vec::is_empty)
    {
        return Err("base report claim ceiling or historical-record disposition is invalid".into());
    }
    let report_rows = report
        .get("results")
        .and_then(Value::as_array)
        .ok_or("base impact report results must be an array")?;
    if report_rows.len() != index.nodes.len() {
        return Err("base impact report does not contain exactly one row per graph node".into());
    }

    let mut rows = BTreeMap::new();
    for row in report_rows {
        let node_id = required_string(row, "node_id", "base report row")?;
        let node = index
            .nodes
            .get(node_id)
            .ok_or_else(|| format!("base report contains unknown node {node_id}"))?;
        if row.get("node_type") != node.get("node_type")
            || row.get("artifact_sha256") != node.get("artifact_sha256")
        {
            return Err(format!(
                "base report row {node_id} does not preserve graph node identity"
            ));
        }
        if !matches!(
            row.get("impact_class").and_then(Value::as_str),
            Some(
                "no_impact"
                    | "historical_record_only"
                    | "reverify_required"
                    | "claim_invalidated"
                    | "quarantined_pending_review"
            )
        ) {
            return Err(format!("base report row {node_id} has an unknown impact class"));
        }
        if !matches!(
            row.get("historical_authenticity").and_then(Value::as_str),
            Some("valid_at_time" | "invalid_at_time" | "unknown" | "not_applicable")
        ) {
            return Err(format!(
                "base report row {node_id} has an unknown historical-authenticity state"
            ));
        }
        if !matches!(
            row.get("current_reliance").and_then(Value::as_str),
            Some("eligible" | "reverify_required" | "ineligible" | "quarantined" | "not_applicable")
        ) {
            return Err(format!(
                "base report row {node_id} has an unknown current-reliance state"
            ));
        }
        let dependency_path = row
            .get("dependency_path")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("base report row {node_id} has no dependency path array"))?;
        if dependency_path.iter().any(|item| item.as_str().is_none_or(str::is_empty))
            || (row["impact_class"] == "no_impact" && !dependency_path.is_empty())
            || (row["impact_class"] != "no_impact" && dependency_path.is_empty())
        {
            return Err(format!("base report row {node_id} has an inconsistent dependency path"));
        }
        let reasons = row
            .get("reason_codes")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("base report row {node_id} has no reason-code array"))?;
        if reasons.is_empty()
            || reasons.iter().any(|item| item.as_str().is_none_or(str::is_empty))
        {
            return Err(format!("base report row {node_id} has invalid reason codes"));
        }
        if rows.insert(node_id.to_owned(), row.clone()).is_some() {
            return Err(format!("base report contains duplicate node row {node_id}"));
        }
    }
    if rows.len() != index.nodes.len()
        || index.nodes.keys().any(|node_id| !rows.contains_key(node_id))
    {
        return Err("base impact report is missing graph nodes".into());
    }

    let summary = summarize_rows(&rows);
    if report.get("summary") != Some(&summary) {
        return Err("base impact report summary does not reconstruct from its rows".into());
    }
    Ok(rows)
}

fn summarize_rows(rows: &BTreeMap<String, Value>) -> Value {
    let count = |class: &str| {
        rows.values()
            .filter(|row| row.get("impact_class").and_then(Value::as_str) == Some(class))
            .count()
    };
    json!({
        "node_count": rows.len(),
        "no_impact_count": count("no_impact"),
        "historical_record_only_count": count("historical_record_only"),
        "reverify_required_count": count("reverify_required"),
        "claim_invalidated_count": count("claim_invalidated"),
        "quarantined_pending_review_count": count("quarantined_pending_review")
    })
}

fn impact_rank(value: &Value) -> u8 {
    match value.get("impact_class").and_then(Value::as_str) {
        Some("no_impact") => 0,
        Some("historical_record_only") => 1,
        Some("reverify_required") => 2,
        Some("quarantined_pending_review") => 3,
        Some("claim_invalidated") => 4,
        _ => 0,
    }
}

fn historical_rank(value: &Value) -> u8 {
    match value.get("historical_authenticity").and_then(Value::as_str) {
        Some("not_applicable") => 0,
        Some("valid_at_time") => 1,
        Some("unknown") => 2,
        Some("invalid_at_time") => 3,
        _ => 0,
    }
}

fn reliance_rank(value: &Value) -> u8 {
    match value.get("current_reliance").and_then(Value::as_str) {
        Some("not_applicable") => 0,
        Some("eligible") => 1,
        Some("reverify_required") => 2,
        Some("quarantined") => 3,
        Some("ineligible") => 4,
        _ => 0,
    }
}

/// Conservative monotonic join for append-only invalidations. This v0.1
/// transition never clears a prior quarantine, invalidation or review need.
/// Corrections/reversals require a later full lifecycle policy and are not
/// inferred from mere absence in the next event's closure.
fn merge_impact_rows(previous: &Value, incoming: &Value) -> Value {
    let mut merged = previous.clone();
    if impact_rank(incoming) > impact_rank(previous) {
        merged["impact_class"] = incoming["impact_class"].clone();
        merged["dependency_path"] = incoming["dependency_path"].clone();
    } else if impact_rank(incoming) == impact_rank(previous) {
        let left_path = previous["dependency_path"].as_array().cloned().unwrap_or_default();
        let right_path = incoming["dependency_path"].as_array().cloned().unwrap_or_default();
        let right_path_order = right_path
            .iter()
            .filter_map(Value::as_str)
            .cmp(left_path.iter().filter_map(Value::as_str));
        if right_path.len() > left_path.len()
            || (right_path.len() == left_path.len() && right_path_order.is_lt())
        {
            merged["dependency_path"] = incoming["dependency_path"].clone();
        }
    }

    if historical_rank(incoming) > historical_rank(previous) {
        merged["historical_authenticity"] = incoming["historical_authenticity"].clone();
    }
    if reliance_rank(incoming) > reliance_rank(previous) {
        merged["current_reliance"] = incoming["current_reliance"].clone();
    }

    let mut reasons = BTreeSet::new();
    let state_is_impacted = impact_rank(previous).max(impact_rank(incoming)) > 0;
    for row in [previous, incoming] {
        if let Some(values) = row.get("reason_codes").and_then(Value::as_array) {
            for value in values.iter().filter_map(Value::as_str) {
                if state_is_impacted
                    && matches!(value, "NO_DEPENDENCY_PATH" | "OUTSIDE_INVALIDATION_SCOPE")
                {
                    continue;
                }
                reasons.insert(value.to_owned());
            }
        }
    }
    merged["reason_codes"] = json!(reasons.into_iter().collect::<Vec<_>>());
    merged
}

fn validate_state_pin(
    index: &GraphIndex,
    base_report: &Value,
    pin: &Value,
) -> Result<BTreeMap<String, Value>, String> {
    if pin.get("schema_version").and_then(Value::as_str) != Some(STATE_PIN_SCHEMA) {
        return Err("base state pin has an unsupported schema version".into());
    }
    let report_sha = canonical_json_digest(base_report)?;
    if pin.get("impact_report_sha256").and_then(Value::as_str) != Some(report_sha.as_str()) {
        return Err("base state pin does not match the exact canonical base report".into());
    }
    if pin.get("graph_snapshot_sha256").and_then(Value::as_str)
        != Some(index.graph_snapshot_sha256.as_str())
        || base_report.get("graph_snapshot_sha256").and_then(Value::as_str)
            != Some(index.graph_snapshot_sha256.as_str())
    {
        return Err("base report/state pin is bound to a different graph snapshot".into());
    }
    let head = required_string(pin, "event_chain_head_sha256", "base state pin")?;
    if !valid_sha256(head)
        || base_report.get("invalidation_event_sha256").and_then(Value::as_str) != Some(head)
    {
        return Err("base state pin event-chain head does not match the base report".into());
    }
    let sequence = pin
        .get("last_event_sequence")
        .and_then(Value::as_u64)
        .ok_or("base state pin last_event_sequence must be an unsigned integer")?;
    if sequence == 0 {
        return Err("base state pin sequence must be positive".into());
    }
    if pin.get("impact_policy_sha256") != base_report.get("impact_policy_sha256")
        || pin.get("impact_engine_build_sha256") != base_report.get("impact_engine_build_sha256")
    {
        return Err("base state pin policy/engine identity differs from the base report".into());
    }
    let pinned_at = required_string(pin, "pinned_at", "base state pin")?;
    let last_recorded_at = required_string(pin, "last_recorded_at", "base state pin")?;
    let last_trusted_at = required_string(pin, "last_trusted_at", "base state pin")?;
    if !valid_utc_timestamp(pinned_at)
        || !valid_utc_timestamp(last_recorded_at)
        || !valid_utc_timestamp(last_trusted_at)
        || last_recorded_at > pinned_at
        || last_trusted_at < last_recorded_at
        || last_trusted_at > pinned_at
    {
        return Err("base state pin has invalid or non-monotonic event times".into());
    }
    if pin["high_water_storage"] != "caller_managed_monotonic_external_store_required"
        || pin["authentication_status"] != "not_checked"
        || pin["claim_ceiling"] != "incomplete_or_indeterminate"
        || pin["action_authority"] != "none"
    {
        return Err("base state pin claim ceiling or storage semantics are invalid".into());
    }
    let state_sha256 = required_string(pin, "impact_state_sha256", "base state pin")?;
    if !valid_sha256(state_sha256) {
        return Err("base state pin has an invalid impact-state digest".into());
    }
    let rows = report_rows(base_report, index)?;
    if sequence == 1 && initial_state_digest(base_report)? != state_sha256 {
        return Err("genesis state pin does not match the canonical initial report state".into());
    }
    Ok(rows)
}

fn initial_state_digest(report: &Value) -> Result<String, String> {
    let report_sha = canonical_json_digest(report)?;
    canonical_json_digest(&json!({
        "schema_version": "luminous.ieac-assurance-impact-state-root.v0.1",
        "genesis_report_sha256": report_sha
    }))
}

fn next_state_digest(
    base_state_sha256: &str,
    event_sha256: &str,
    event_sequence: u64,
    updates: &[Value],
) -> Result<String, String> {
    if !valid_sha256(base_state_sha256) || !valid_sha256(event_sha256) {
        return Err("state transition requires valid base-state and event digests".into());
    }
    let updates_sha256 = canonical_json_digest(&Value::Array(updates.to_vec()))?;
    canonical_json_digest(&json!({
        "schema_version": "luminous.ieac-assurance-impact-state-transition.v0.1",
        "base_state_sha256": base_state_sha256,
        "event_sha256": event_sha256,
        "event_sequence": event_sequence,
        "updates_sha256": updates_sha256
    }))
}

fn make_state_pin(
    index: &GraphIndex,
    report: &Value,
    sequence: u64,
    event_head: &str,
    last_recorded_at: &str,
    last_trusted_at: &str,
    state_sha256: &str,
) -> Result<Value, String> {
    if sequence == 0 || !valid_sha256(event_head) || !valid_sha256(state_sha256) {
        return Err("state pin requires a positive sequence and valid lowercase SHA-256 digests".into());
    }
    if report.get("invalidation_event_sha256").and_then(Value::as_str) != Some(event_head) {
        return Err("state pin event head differs from report invalidation digest".into());
    }
    let pinned_at = required_string(report, "evaluated_at", "base report")?;
    if !valid_utc_timestamp(pinned_at)
        || !valid_utc_timestamp(last_recorded_at)
        || !valid_utc_timestamp(last_trusted_at)
        || last_trusted_at < last_recorded_at
        || last_trusted_at > pinned_at
    {
        return Err("state pin requires ordered, normalized recorded/trusted/evaluation times".into());
    }
    let report_sha = canonical_json_digest(report)?;
    Ok(json!({
        "schema_version": STATE_PIN_SCHEMA,
        "graph_snapshot_sha256": index.graph_snapshot_sha256,
        "impact_report_sha256": report_sha,
        "impact_state_sha256": state_sha256,
        "last_event_sequence": sequence,
        "event_chain_head_sha256": event_head,
        "last_recorded_at": last_recorded_at,
        "last_trusted_at": last_trusted_at,
        "impact_policy_sha256": report["impact_policy_sha256"],
        "impact_engine_build_sha256": report["impact_engine_build_sha256"],
        "pinned_at": pinned_at,
        "high_water_storage": "caller_managed_monotonic_external_store_required",
        "authentication_status": "not_checked",
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none"
    }))
}

struct Transition {
    delta: Value,
    report: Value,
    state_pin: Value,
}

#[derive(Clone, Debug)]
struct EventCursor {
    sequence: u64,
    event_head_sha256: String,
    last_recorded_at: String,
    last_trusted_at: String,
    pinned_at: String,
    state_sha256: String,
}

fn cursor_from_pin(pin: &Value) -> Result<EventCursor, String> {
    Ok(EventCursor {
        sequence: pin["last_event_sequence"]
            .as_u64()
            .ok_or("state pin sequence is missing")?,
        event_head_sha256: required_string(pin, "event_chain_head_sha256", "state pin")?.to_owned(),
        last_recorded_at: required_string(pin, "last_recorded_at", "state pin")?.to_owned(),
        last_trusted_at: required_string(pin, "last_trusted_at", "state pin")?.to_owned(),
        pinned_at: required_string(pin, "pinned_at", "state pin")?.to_owned(),
        state_sha256: required_string(pin, "impact_state_sha256", "state pin")?.to_owned(),
    })
}

fn materialize_report(
    base_report: &Value,
    rows: &BTreeMap<String, Value>,
    final_event: &Value,
    evaluated_at: &str,
) -> Result<Value, String> {
    let mut report = base_report.clone();
    report["report_id"] = json!(format!(
        "impact-{}",
        required_string(final_event, "event_id", "final event")?
    ));
    report["invalidation_event_sha256"] = json!(canonical_json_digest(final_event)?);
    report["evaluated_at"] = json!(evaluated_at);
    report["results"] = json!(rows.values().cloned().collect::<Vec<_>>());
    report["summary"] = summarize_rows(rows);
    Ok(report)
}

fn advance_state(
    index: &GraphIndex,
    rows: &mut BTreeMap<String, Value>,
    base_report_sha256: &str,
    base_report: &Value,
    cursor: &EventCursor,
    event: &Value,
    metadata: &Value,
) -> Result<(Value, EventCursor), String> {
    validate_event(event, &index.nodes)?;

    let expected_sequence = cursor
        .sequence
        .checked_add(1)
        .ok_or("authority sequence overflow")?;
    let sequence = event["authority_sequence"]
        .as_u64()
        .ok_or("event authority_sequence must be an unsigned integer")?;
    if sequence != expected_sequence {
        return Err(format!(
            "event sequence mismatch: expected {expected_sequence}, got {sequence}"
        ));
    }
    if event["previous_event_sha256"].as_str() != Some(cursor.event_head_sha256.as_str()) {
        return Err("event predecessor does not match pinned event-chain head".into());
    }

    let event_digest = canonical_json_digest(event)?;
    let event_system_time = event.get("system_time").ok_or("event system_time missing")?;
    let recorded_at = required_string(event_system_time, "recorded_at", "event.system_time")?;
    let trusted_at = required_string(event_system_time, "trusted_at", "event.system_time")?;
    if !valid_utc_timestamp(recorded_at)
        || !valid_utc_timestamp(trusted_at)
        || trusted_at < recorded_at
        || recorded_at < cursor.last_recorded_at.as_str()
        || trusted_at < cursor.last_trusted_at.as_str()
    {
        return Err("event recorded/trusted times violate the pinned temporal high-water mark".into());
    }

    let evaluated_at = required_string(metadata, "evaluated_at", "delta metadata")?;
    let policy_sha = required_string(metadata, "impact_policy_sha256", "delta metadata")?;
    let engine_sha = required_string(metadata, "impact_engine_build_sha256", "delta metadata")?;
    if !valid_utc_timestamp(evaluated_at)
        || !valid_sha256(policy_sha)
        || !valid_sha256(engine_sha)
        || evaluated_at < cursor.pinned_at.as_str()
    {
        return Err("delta evaluation time, policy or engine identity is invalid or backdated".into());
    }
    if base_report["impact_policy_sha256"].as_str() != Some(policy_sha)
        || base_report["impact_engine_build_sha256"].as_str() != Some(engine_sha)
    {
        return Err("policy or engine identity changed; run full reconciliation before applying a delta".into());
    }
    let direct_target_set = event["targets"]
        .as_array()
        .ok_or("event.targets must be an array")?
        .iter()
        .map(|target| required_string(target, "node_id", "event target").map(str::to_owned))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let impacted_rows = build_impacted_results_with_adjacency(
        event,
        evaluated_at,
        &index.nodes,
        &index.forward,
    )?;

    let independently_reached = index.independent_reachable_set(event)?;
    let bfs_reached: BTreeSet<String> = impacted_rows.keys().cloned().collect();
    if independently_reached != bfs_reached {
        return Err("independent DFS closure disagrees with lifecycle BFS closure".into());
    }
    for (node_id, row) in &impacted_rows {
        if row["impact_class"] == "claim_invalidated" && !direct_target_set.contains(node_id) {
            return Err(format!(
                "dependency descendant {node_id} was incorrectly classified as directly invalidated"
            ));
        }
    }
    let affected_node_ids = bfs_reached.iter().cloned().collect::<Vec<_>>();
    if direct_target_set.iter().any(|node_id| !bfs_reached.contains(node_id)) {
        return Err("direct target is missing from the calculated impact closure".into());
    }
    let direct_target_node_ids = direct_target_set.iter().cloned().collect::<Vec<_>>();

    let mut updates = Vec::new();
    for node_id in &affected_node_ids {
        let previous = rows
            .get(node_id)
            .ok_or_else(|| format!("base state lacks affected node {node_id}"))?;
        let incoming = impacted_rows
            .get(node_id)
            .ok_or_else(|| format!("impact row evaluator omitted affected node {node_id}"))?;
        let merged = merge_impact_rows(previous, incoming);
        if merged != *previous {
            updates.push(merged.clone());
            rows.insert(node_id.clone(), merged);
        }
    }
    updates.sort_by(|left, right| {
        left["node_id"]
            .as_str()
            .cmp(&right["node_id"].as_str())
    });

    let resulting_state_sha256 = next_state_digest(
        &cursor.state_sha256,
        &event_digest,
        sequence,
        &updates,
    )?;
    let update_count = updates.len();
    let affected_node_count = independently_reached.len();
    let mut delta = json!({
        "schema_version": DELTA_SCHEMA,
        "delta_id": format!("delta-{}-{}", sequence, required_string(event, "event_id", "event")?),
        "graph_snapshot_sha256": index.graph_snapshot_sha256,
        "graph_index_sha256": index.index_sha256,
        "base_impact_report_sha256": base_report_sha256,
        "base_state_sha256": cursor.state_sha256,
        "resulting_state_sha256": resulting_state_sha256,
        "base_event_sequence": cursor.sequence,
        "base_event_chain_head_sha256": cursor.event_head_sha256,
        "event_id": event["event_id"],
        "event_sequence": sequence,
        "event_sha256": event_digest,
        "previous_event_sha256": event["previous_event_sha256"],
        "impact_policy_sha256": policy_sha,
        "impact_engine_build_sha256": engine_sha,
        "evaluated_at": evaluated_at,
        "impact_merge_policy": "monotonic_conservative_join_v0.1",
        "direct_target_node_ids": direct_target_node_ids,
        "affected_node_ids": affected_node_ids,
        "affected_node_count": affected_node_count,
        "updates": updates,
        "update_count": update_count,
        "closure_verifier": "independent_iterative_dfs_v0.1",
        "closure_complete": true,
        "resulting_event_sequence": sequence,
        "resulting_event_chain_head_sha256": event_digest,
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none",
        "delta_sha256": ""
    });
    let delta_sha = canonical_digest_without_field(&delta, "delta_sha256")?;
    delta["delta_sha256"] = json!(delta_sha);

    let next_cursor = EventCursor {
        sequence,
        event_head_sha256: event_digest,
        last_recorded_at: recorded_at.to_owned(),
        last_trusted_at: trusted_at.to_owned(),
        pinned_at: evaluated_at.to_owned(),
        state_sha256: resulting_state_sha256,
    };
    Ok((delta, next_cursor))
}

fn compute_transition(
    index: &GraphIndex,
    base_report: &Value,
    base_pin: &Value,
    event: &Value,
    metadata: &Value,
) -> Result<Transition, String> {
    let mut rows = validate_state_pin(index, base_report, base_pin)?;
    let cursor = cursor_from_pin(base_pin)?;
    let base_report_sha256 = canonical_json_digest(base_report)?;
    let (delta, next_cursor) = advance_state(
        index,
        &mut rows,
        &base_report_sha256,
        base_report,
        &cursor,
        event,
        metadata,
    )?;
    let evaluated_at = required_string(metadata, "evaluated_at", "delta metadata")?;
    let report = materialize_report(base_report, &rows, event, evaluated_at)?;
    let state_pin = make_state_pin(
        index,
        &report,
        next_cursor.sequence,
        &next_cursor.event_head_sha256,
        &next_cursor.last_recorded_at,
        &next_cursor.last_trusted_at,
        &next_cursor.state_sha256,
    )?;
    Ok(Transition {
        delta,
        report,
        state_pin,
    })
}

fn canonical_digest_without_field(value: &Value, field: &str) -> Result<String, String> {
    let mut unsigned = value.clone();
    let object = unsigned
        .as_object_mut()
        .ok_or("digest-bound object must be a JSON object")?;
    object.remove(field);
    canonical_json_digest(&unsigned)
}

fn apply_delta(
    index: &GraphIndex,
    base_report: &Value,
    base_pin: &Value,
    event: &Value,
    metadata: &Value,
    delta: &Value,
) -> Result<Transition, String> {
    if delta.get("schema_version").and_then(Value::as_str) != Some(DELTA_SCHEMA) {
        return Err("impact delta schema version is unsupported".into());
    }
    if delta.get("closure_complete").and_then(Value::as_bool) != Some(true) {
        return Err("impact delta does not assert a complete dependency closure".into());
    }
    let claimed_delta_sha = required_string(delta, "delta_sha256", "impact delta")?;
    if !valid_sha256(claimed_delta_sha)
        || canonical_digest_without_field(delta, "delta_sha256")? != claimed_delta_sha
    {
        return Err("impact delta digest mismatch".into());
    }

    let expected = compute_transition(index, base_report, base_pin, event, metadata)?;
    if expected.delta != *delta {
        return Err(
            "impact delta does not equal the complete deterministic transition from its exact base".into(),
        );
    }
    if expected.state_pin["impact_state_sha256"] != delta["resulting_state_sha256"] {
        return Err("resulting state root does not match sparse delta application".into());
    }
    Ok(expected)
}

fn process_batch(
    graph: &Value,
    base_report: &Value,
    base_pin: &Value,
    delta_input: &Value,
    received_batch: Option<&Value>,
) -> Result<Value, String> {
    if delta_input.get("schema_version").and_then(Value::as_str)
        != Some(DELTA_INPUT_SCHEMA)
    {
        return Err("impact delta input has an unsupported schema version".into());
    }
    let metadata = delta_input.get("metadata").ok_or("impact delta metadata is missing")?;
    let events = delta_input
        .get("events")
        .and_then(Value::as_array)
        .ok_or("events input must contain an events array")?;
    if events.is_empty() || events.len() > MAX_BATCH_EVENTS {
        return Err(format!("event batch must contain 1..={MAX_BATCH_EVENTS} events"));
    }

    let index = GraphIndex::build(graph)?;
    let mut rows = validate_state_pin(&index, base_report, base_pin)?;
    let mut cursor = cursor_from_pin(base_pin)?;
    let base_report_sha256 = canonical_json_digest(base_report)?;
    let mut deltas = Vec::with_capacity(events.len());
    let mut total_closure_node_memberships = 0usize;

    for (event_index, event) in events.iter().enumerate() {
        let (delta, next_cursor) = advance_state(
            &index,
            &mut rows,
            &base_report_sha256,
            base_report,
            &cursor,
            event,
            metadata,
        )?;

        if let Some(received) = received_batch {
            let received_deltas = received
                .get("deltas")
                .and_then(Value::as_array)
                .ok_or("received delta batch has no deltas array")?;
            let received_delta = received_deltas
                .get(event_index)
                .ok_or_else(|| format!("received delta batch is missing event {event_index}"))?;
            if received_delta != &delta {
                return Err(format!(
                    "received sparse delta {event_index} does not match the exact base-bound transition"
                ));
            }
        }

        let affected = delta["affected_node_count"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("delta affected-node count is invalid")?;
        total_closure_node_memberships = total_closure_node_memberships
            .checked_add(affected)
            .ok_or("batch closure-membership counter overflow")?;
        if total_closure_node_memberships > MAX_BATCH_CLOSURE_NODE_MEMBERSHIPS {
            return Err(format!(
                "batch closure-membership budget exceeded: {total_closure_node_memberships} > {MAX_BATCH_CLOSURE_NODE_MEMBERSHIPS}"
            ));
        }
        deltas.push(delta);
        cursor = next_cursor;
    }

    if let Some(received) = received_batch {
        if received.get("schema_version").and_then(Value::as_str) != Some(BATCH_SCHEMA)
            || received.get("complete").and_then(Value::as_bool) != Some(true)
            || received.get("graph_index_build_count").and_then(Value::as_u64) != Some(1)
            || received.get("graph_index_reused_for_events").and_then(Value::as_u64)
                != u64::try_from(events.len()).ok()
            || received.get("state_row_map_initializations").and_then(Value::as_u64) != Some(1)
            || received.get("state_row_map_reused_for_events").and_then(Value::as_u64)
                != u64::try_from(events.len()).ok()
            || received.get("full_report_materializations").and_then(Value::as_u64) != Some(1)
            || received.get("event_count").and_then(Value::as_u64)
                != u64::try_from(events.len()).ok()
            || received.get("delta_count").and_then(Value::as_u64)
                != u64::try_from(events.len()).ok()
        {
            return Err("received delta batch has invalid completeness or work-shape metadata".into());
        }
    }

    // Materialize and hash the full reconciliation representation once, at
    // the batch boundary, rather than once per event.
    let final_event = events.last().ok_or("event batch unexpectedly empty")?;
    let evaluated_at = required_string(metadata, "evaluated_at", "delta metadata")?;
    let resulting_report = materialize_report(base_report, &rows, final_event, evaluated_at)?;
    let resulting_report_sha256 = canonical_json_digest(&resulting_report)?;
    let next_state_pin = make_state_pin(
        &index,
        &resulting_report,
        cursor.sequence,
        &cursor.event_head_sha256,
        &cursor.last_recorded_at,
        &cursor.last_trusted_at,
        &cursor.state_sha256,
    )?;

    let expected_batch = json!({
        "schema_version": BATCH_SCHEMA,
        "graph_snapshot_sha256": index.graph_snapshot_sha256,
        "graph_index_sha256": index.index_sha256,
        "graph_index_build_count": 1,
        "graph_index_reused_for_events": events.len(),
        "state_row_map_initializations": 1,
        "state_row_map_reused_for_events": events.len(),
        "full_report_materializations": 1,
        "total_closure_node_memberships": total_closure_node_memberships,
        "graph_node_count": index.nodes.len(),
        "graph_edge_count": index.edge_count,
        "event_count": events.len(),
        "delta_count": deltas.len(),
        "deltas": deltas,
        "resulting_impact_report_sha256": resulting_report_sha256,
        "resulting_event_sequence": cursor.sequence,
        "resulting_event_chain_head_sha256": cursor.event_head_sha256,
        "resulting_impact_report": resulting_report,
        "next_state_pin": next_state_pin,
        "complete": true,
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none"
    });

    if let Some(received) = received_batch {
        if received != &expected_batch {
            return Err(
                "received delta batch does not match the exact base, event chain, sparse state and final report"
                    .into(),
            );
        }
    }
    Ok(expected_batch)
}

fn run_batch(
    graph: &Value,
    base_report: &Value,
    base_pin: &Value,
    delta_input: &Value,
) -> Result<Value, String> {
    process_batch(graph, base_report, base_pin, delta_input, None)
}

fn verify_batch(
    graph: &Value,
    base_report: &Value,
    base_pin: &Value,
    delta_input: &Value,
    received_batch: &Value,
) -> Result<Value, String> {
    process_batch(
        graph,
        base_report,
        base_pin,
        delta_input,
        Some(received_batch),
    )
}

fn bootstrap_pin(
    graph: &Value,
    report: &Value,
    sequence: u64,
    event_head: &str,
    last_recorded_at: &str,
    last_trusted_at: &str,
) -> Result<Value, String> {
    let index = GraphIndex::build(graph)?;
    let _ = report_rows(report, &index)?;
    let state_sha256 = initial_state_digest(report)?;
    let pin = make_state_pin(
        &index,
        report,
        sequence,
        event_head,
        last_recorded_at,
        last_trusted_at,
        &state_sha256,
    )?;
    validate_state_pin(&index, report, &pin)?;
    Ok(pin)
}


fn build_scaled_benchmark_case(
    total_nodes: usize,
    impact_basis_points: usize,
) -> Result<(Value, Value, Value, Value), String> {
    if total_nodes == 0 || total_nodes > 100_000 || impact_basis_points > 10_000 {
        return Err("benchmark graph size or impact fraction is outside the declared bounds".into());
    }

    let fixture: Value = {
        let parsed: StrictJson = serde_json::from_str(include_str!(
            "../../../../schemas/examples/ieac-trust-lifecycle-key-compromise-synthetic-v0.1.json"
        ))
        .map_err(|error| format!("cannot parse frozen benchmark fixture: {error}"))?;
        parsed.into_value()
    };
    let fixture_graph = &fixture["graph"];
    let root_template = fixture_graph["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["node_type"] == "issuer_key"))
        .ok_or("benchmark fixture lacks an issuer-key root node")?;
    let statement_template = fixture_graph["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["node_type"] == "signed_statement"))
        .ok_or("benchmark fixture lacks a signed-statement node template")?;

    let affected_count = total_nodes
        .checked_mul(impact_basis_points)
        .ok_or("benchmark impact-fraction arithmetic overflow")?
        .checked_div(10_000)
        .ok_or("benchmark impact-fraction division failed")?
        .max(1)
        .min(total_nodes);

    let root_id = required_string(root_template, "node_id", "benchmark root")?.to_owned();
    let root_digest = required_string(root_template, "artifact_sha256", "benchmark root")?.to_owned();
    let mut nodes = Vec::with_capacity(total_nodes);
    nodes.push(root_template.clone());
    let mut edges = Vec::with_capacity(affected_count.saturating_sub(1));

    for index in 1..total_nodes {
        let mut node = statement_template.clone();
        let digest = format!("{:064x}", index + 1);
        let node_id = format!("signed_statement:sha256:{digest}");
        node["node_id"] = json!(node_id);
        node["artifact_sha256"] = json!(digest);
        nodes.push(node);
        if index < affected_count {
            edges.push(json!({
                "edge_id": format!("bench-edge-{index:08}"),
                "from_node_id": root_id,
                "to_node_id": node_id,
                "relation": "signed_by"
            }));
        }
    }

    let mut graph = json!({
        "schema_version": fixture_graph["schema_version"],
        "graph_snapshot_sha256": "0".repeat(64),
        "nodes": nodes,
        "edges": edges
    });
    let mut digest_input = graph.clone();
    digest_input
        .as_object_mut()
        .ok_or("benchmark graph must be an object")?
        .remove("graph_snapshot_sha256");
    graph["graph_snapshot_sha256"] = json!(canonical_json_digest(&digest_input)?);

    let mut event = fixture["invalidation_event"].clone();
    event["event_id"] = json!(format!("bench-key-compromise-{total_nodes}-{impact_basis_points}"));
    event["authority_sequence"] = json!(1);
    event["previous_event_sha256"] = Value::Null;
    event["cause_class"] = json!("issuer_key_compromised");
    event["targets"] = json!([{
        "node_id": root_id,
        "node_type": "issuer_key",
        "artifact_sha256": root_digest
    }]);

    let nodes_by_id = validate_graph(&graph)?;
    validate_event(&event, &nodes_by_id)?;
    let mut base_metadata = fixture["expected_impact_report"].clone();
    base_metadata["evaluated_at"] = json!("2026-11-03T06:00:00Z");
    let base_report = lifecycle::build_impact_report(
        &graph,
        &event,
        &base_metadata,
        &nodes_by_id,
    )?;
    let event_digest = canonical_json_digest(&event)?;
    let system_time = &event["system_time"];
    let index = GraphIndex::build(&graph)?;
    let state_root = initial_state_digest(&base_report)?;
    let base_pin = make_state_pin(
        &index,
        &base_report,
        1,
        &event_digest,
        required_string(system_time, "recorded_at", "benchmark event.system_time")?,
        required_string(system_time, "trusted_at", "benchmark event.system_time")?,
        &state_root,
    )?;

    let mut event_two = event.clone();
    event_two["event_id"] = json!(format!("bench-key-rotation-{total_nodes}-{impact_basis_points}"));
    event_two["authority_sequence"] = json!(2);
    event_two["previous_event_sha256"] = json!(event_digest);
    event_two["cause_class"] = json!("issuer_key_revoked_or_rotated");
    event_two["system_time"]["recorded_at"] = json!("2026-11-02T12:00:00Z");
    event_two["system_time"]["trusted_at"] = json!("2026-11-02T12:01:00Z");

    let event_two_digest = canonical_json_digest(&event_two)?;
    let mut event_three = event_two.clone();
    event_three["event_id"] = json!(format!("bench-followup-{total_nodes}-{impact_basis_points}"));
    event_three["authority_sequence"] = json!(3);
    event_three["previous_event_sha256"] = json!(event_two_digest);
    event_three["system_time"]["recorded_at"] = json!("2026-11-02T13:00:00Z");
    event_three["system_time"]["trusted_at"] = json!("2026-11-02T13:01:00Z");

    let delta_input = json!({
        "schema_version": DELTA_INPUT_SCHEMA,
        "events": [event_two, event_three],
        "metadata": {
            "evaluated_at": "2026-11-03T07:00:00Z",
            "impact_policy_sha256": base_report["impact_policy_sha256"],
            "impact_engine_build_sha256": base_report["impact_engine_build_sha256"]
        }
    });
    Ok((graph, base_report, base_pin, delta_input))
}

fn full_recompute_batch_reference(
    graph: &Value,
    base_report: &Value,
    base_pin: &Value,
    delta_input: &Value,
) -> Result<Value, String> {
    let nodes = validate_graph(graph)?;
    let metadata = delta_input.get("metadata").ok_or("benchmark metadata missing")?;
    let events = delta_input
        .get("events")
        .and_then(Value::as_array)
        .ok_or("benchmark events array missing")?;
    let mut rows = BTreeMap::new();
    for row in base_report
        .get("results")
        .and_then(Value::as_array)
        .ok_or("benchmark base report results missing")?
    {
        let node_id = required_string(row, "node_id", "benchmark base row")?;
        if rows.insert(node_id.to_owned(), row.clone()).is_some() {
            return Err(format!("duplicate benchmark base row {node_id}"));
        }
    }
    if rows.len() != nodes.len() {
        return Err("benchmark base report does not cover the exact graph node set".into());
    }
    for event in events {
        let full_event_report =
            lifecycle::build_impact_report(graph, event, metadata, &nodes)?;
        for row in full_event_report["results"]
            .as_array()
            .ok_or("full reference report rows missing")?
        {
            let node_id = required_string(row, "node_id", "full reference row")?;
            let previous = rows
                .get(node_id)
                .ok_or_else(|| format!("full reference state lacks node {node_id}"))?;
            rows.insert(node_id.to_owned(), merge_impact_rows(previous, row));
        }
        // The B0 path materializes and hashes a complete report after every
        // event; this is intentionally the work being compared with B2.
        let _ = materialize_report(
            base_report,
            &rows,
            event,
            required_string(metadata, "evaluated_at", "benchmark metadata")?,
        )?;
    }
    let final_event = events.last().ok_or("benchmark event batch empty")?;
    materialize_report(
        base_report,
        &rows,
        final_event,
        required_string(metadata, "evaluated_at", "benchmark metadata")?,
    )
    .map_err(Into::into)
}

fn percentile_ns(samples: &[u128], percentile: usize) -> Result<u128, String> {
    if samples.is_empty() || percentile > 100 {
        return Err("benchmark percentile requires non-empty samples and 0..=100".into());
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = (sorted.len() - 1)
        .checked_mul(percentile)
        .ok_or("percentile index overflow")?
        / 100;
    sorted
        .get(index)
        .copied()
        .ok_or_else(|| "percentile index out of range".to_owned())
}

fn run_delta_benchmark(smoke: bool) -> Result<Value, String> {
    use std::time::Instant;

    let graph_sizes: &[usize] = if smoke {
        &[100, 1_000, 10_000]
    } else {
        &[100, 1_000, 10_000, 100_000]
    };
    let impact_fractions_bps: &[usize] = &[10, 100, 1_000, 5_000, 8_000];
    let repetitions = if smoke { 3 } else { 5 };
    let mut cases = Vec::new();

    for total_nodes in graph_sizes {
        for impact_bps in impact_fractions_bps {
            let (graph, base_report, base_pin, delta_input) =
                build_scaled_benchmark_case(*total_nodes, *impact_bps)?;
            let mut b0_ns = Vec::with_capacity(repetitions);
            let mut b2_ns = Vec::with_capacity(repetitions);
            let mut final_batch = Value::Null;
            let mut reference_report = Value::Null;

            for _ in 0..repetitions {
                let b0_start = Instant::now();
                let reference = full_recompute_batch_reference(
                    &graph,
                    &base_report,
                    &base_pin,
                    &delta_input,
                )?;
                b0_ns.push(b0_start.elapsed().as_nanos());

                let b2_start = Instant::now();
                let batch = run_batch(&graph, &base_report, &base_pin, &delta_input)?;
                b2_ns.push(b2_start.elapsed().as_nanos());

                if batch["resulting_impact_report"] != reference {
                    return Err(format!(
                        "full-recompute and sparse-delta reports diverged at nodes={} impact_bps={}",
                        total_nodes, impact_bps
                    ));
                }
                reference_report = reference;
                final_batch = batch;
            }

            let affected = final_batch["deltas"]
                .as_array()
                .and_then(|deltas| deltas.first())
                .and_then(|delta| delta["affected_node_count"].as_u64())
                .ok_or("benchmark affected-node count missing")?;
            let full_bytes = serde_json::to_vec(&reference_report)
                .map_err(|error| format!("cannot size full report: {error}"))?
                .len();
            let delta_bytes = serde_json::to_vec(&final_batch["deltas"])
                .map_err(|error| format!("cannot size sparse deltas: {error}"))?
                .len();
            let full_p50 = percentile_ns(&b0_ns, 50)?;
            let full_p95 = percentile_ns(&b0_ns, 95)?;
            let delta_p50 = percentile_ns(&b2_ns, 50)?;
            let delta_p95 = percentile_ns(&b2_ns, 95)?;

            cases.push(json!({
                "graph_node_count": total_nodes,
                "graph_edge_count": graph["edges"].as_array().map_or(0, Vec::len),
                "impact_fraction_basis_points": impact_bps,
                "event_count": delta_input["events"].as_array().map_or(0, Vec::len),
                "affected_node_count_per_event": affected,
                "repetitions": repetitions,
                "full_recompute_p50_ns": full_p50,
                "full_recompute_p95_ns": full_p95,
                "sparse_batch_p50_ns": delta_p50,
                "sparse_batch_p95_ns": delta_p95,
                "p50_speedup_ratio": if delta_p50 > 0 {
                    full_p50 as f64 / delta_p50 as f64
                } else {
                    0.0
                },
                "full_report_bytes": full_bytes,
                "sparse_delta_bytes": delta_bytes,
                "delta_to_report_byte_ratio": if full_bytes > 0 {
                    delta_bytes as f64 / full_bytes as f64
                } else {
                    0.0
                },
                "graph_index_build_count": final_batch["graph_index_build_count"],
                "state_row_map_initializations": final_batch["state_row_map_initializations"],
                "full_report_materializations": final_batch["full_report_materializations"],
                "total_closure_node_memberships": final_batch["total_closure_node_memberships"],
                "exact_report_digest_agreement": true
            }));
        }
    }

    Ok(json!({
        "schema_version": "luminous.ieac-assurance-impact-delta-benchmark.v0.1",
        "classification": "synthetic_only",
        "benchmark_mode": if smoke { "smoke_100_to_10000_nodes" } else { "full_100_to_100000_nodes" },
        "baseline": "B0_full_recompute_after_every_event",
        "candidate": "B2_reused_index_mutable_rows_one_final_report",
        "repetitions_per_case": repetitions,
        "case_count": cases.len(),
        "cases": cases,
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none",
        "limitations": [
            "Synthetic star dependency graphs only; graph topology, depth and fan-in require separate strata.",
            "The B0 and B2 paths share the same impact-row classification policy.",
            "No signature or authority authentication is performed.",
            "Reported timings are single-runner observations and not a SOTA claim."
        ]
    }))
}
fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().is_some_and(|item| item == "benchmark") {
        let (smoke, output_arg) = match args.as_slice() {
            [_, output] => (false, output.as_str()),
            [_, mode, output] if mode == "--smoke" => (true, output.as_str()),
            _ => {
                eprintln!("usage: industry-impact-delta benchmark [--smoke] <new-report.json>");
                return ExitCode::from(4);
            }
        };
        let result = (|| {
            let report = run_delta_benchmark(smoke)?;
            write_new(Path::new(output_arg), &report)
        })();
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("impact delta benchmark failed closed: {error}");
                ExitCode::from(2)
            }
        };
    }

    if args.first().is_some_and(|item| item == "verify-batch") {
        if args.len() != 7 {
            eprintln!("usage: industry-impact-delta verify-batch <graph.json> <base-report.json> <base-pin.json> <delta-input.json> <received-batch.json> <new-verified-result.json>");
            return ExitCode::from(4);
        }
        let result = (|| {
            let graph = parse_strict(Path::new(&args[1]))?;
            let base_report = parse_strict(Path::new(&args[2]))?;
            let base_pin = parse_strict(Path::new(&args[3]))?;
            let delta_input = parse_strict(Path::new(&args[4]))?;
            let received_batch = parse_strict(Path::new(&args[5]))?;
            let verified = verify_batch(
                &graph,
                &base_report,
                &base_pin,
                &delta_input,
                &received_batch,
            )?;
            write_new(Path::new(&args[6]), &verified)
        })();
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("sparse impact delta batch verification failed closed: {error}");
                ExitCode::from(2)
            }
        };
    }

    if args.first().is_some_and(|item| item == "apply") {
        if args.len() != 8 {
            eprintln!("usage: industry-impact-delta apply <graph.json> <base-report.json> <base-pin.json> <event.json> <metadata.json> <delta.json> <new-apply-result.json>");
            return ExitCode::from(4);
        }
        let result = (|| {
            let graph = parse_strict(Path::new(&args[1]))?;
            let base_report = parse_strict(Path::new(&args[2]))?;
            let base_pin = parse_strict(Path::new(&args[3]))?;
            let event = parse_strict(Path::new(&args[4]))?;
            let metadata = parse_strict(Path::new(&args[5]))?;
            let delta = parse_strict(Path::new(&args[6]))?;
            let index = GraphIndex::build(&graph)?;
            let applied = apply_delta(
                &index,
                &base_report,
                &base_pin,
                &event,
                &metadata,
                &delta,
            )?;
            let report_sha256 = canonical_json_digest(&applied.report)?;
            let result = json!({
                "schema_version": BATCH_SCHEMA,
                "graph_snapshot_sha256": index.graph_snapshot_sha256,
                "graph_index_sha256": index.index_sha256,
                "graph_index_build_count": 1,
                "graph_index_reused_for_events": 1,
                "state_row_map_initializations": 1,
                "state_row_map_reused_for_events": 1,
                "full_report_materializations": 1,
                "total_closure_node_memberships": applied.delta["affected_node_count"],
                "graph_node_count": index.nodes.len(),
                "graph_edge_count": index.edge_count,
                "event_count": 1,
                "delta_count": 1,
                "deltas": [applied.delta],
                "resulting_impact_report_sha256": report_sha256,
                "resulting_event_sequence": applied.state_pin["last_event_sequence"],
                "resulting_event_chain_head_sha256": applied.state_pin["event_chain_head_sha256"],
                "resulting_impact_report": applied.report,
                "next_state_pin": applied.state_pin,
                "complete": true,
                "claim_ceiling": "incomplete_or_indeterminate",
                "action_authority": "none"
            });
            write_new(Path::new(&args[7]), &result)
        })();
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("sparse impact delta application failed closed: {error}");
                ExitCode::from(2)
            }
        };
    }

    if args.first().is_some_and(|item| item == "pin") {
        if args.len() != 8 {
            eprintln!("usage: industry-impact-delta pin <graph.json> <base-report.json> <last-sequence> <event-chain-head-sha256> <last-recorded-at> <last-trusted-at> <new-pin.json>");
            return ExitCode::from(4);
        }
        let result = (|| {
            let graph = parse_strict(Path::new(&args[1]))?;
            let report = parse_strict(Path::new(&args[2]))?;
            let sequence = args[3]
                .parse::<u64>()
                .map_err(|error| format!("invalid last event sequence: {error}"))?;
            let pin = bootstrap_pin(
                &graph,
                &report,
                sequence,
                &args[4],
                &args[5],
                &args[6],
            )?;
            write_new(Path::new(&args[7]), &pin)
        })();
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("impact state pin creation failed closed: {error}");
                ExitCode::from(2)
            }
        };
    }

    if args.len() != 5 {
        eprintln!("usage: industry-impact-delta <graph.json> <base-report.json> <base-pin.json> <delta-input.json> <new-batch-result.json> (or subcommands: benchmark, verify-batch, apply, pin)");
        return ExitCode::from(4);
    }
    let result = (|| {
        let graph = parse_strict(Path::new(&args[0]))?;
        let base_report = parse_strict(Path::new(&args[1]))?;
        let base_pin = parse_strict(Path::new(&args[2]))?;
        let delta_input = parse_strict(Path::new(&args[3]))?;
        let batch = run_batch(&graph, &base_report, &base_pin, &delta_input)?;
        // One create-new result file carries the deltas, final report and next pin.
        // Consumers never observe a partially written three-file state tuple.
        write_new(Path::new(&args[4]), &batch)?;
        Ok::<(), String>(())
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("impact delta processing failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        parse_fixture(include_str!(
            "../../../schemas/examples/ieac-trust-lifecycle-key-compromise-synthetic-v0.1.json"
        ))
    }

    fn parse_fixture(source: &str) -> Value {
        let parsed: StrictJson = serde_json::from_str(source).unwrap();
        parsed.into_value()
    }

    fn setup() -> (Value, Value, Value, Value, GraphIndex) {
        let fixture = fixture();
        let graph = fixture["graph"].clone();
        let index = GraphIndex::build(&graph).unwrap();
        let first_event = fixture["invalidation_event"].clone();
        let metadata = fixture["expected_impact_report"].clone();
        let base_report = lifecycle::build_impact_report(
            &graph,
            &first_event,
            &metadata,
            &index.nodes,
        )
        .unwrap();
        let first_digest = canonical_json_digest(&first_event).unwrap();
        let first_system_time = &first_event["system_time"];
        let genesis_state_sha256 = initial_state_digest(&base_report).unwrap();
        let base_pin = make_state_pin(
            &index,
            &base_report,
            1,
            &first_digest,
            first_system_time["recorded_at"].as_str().unwrap(),
            first_system_time["trusted_at"].as_str().unwrap(),
            &genesis_state_sha256,
        )
        .unwrap();
        (graph, base_report, base_pin, metadata, index)
    }

    fn source_correction_event(base_pin: &Value) -> Value {
        let original = fixture()["invalidation_event"].clone();
        let node_id = "source_measurement:sha256:0202020202020202020202020202020202020202020202020202020202020202";
        let mut event = original;
        event["event_id"] = json!("evt-source-correction-002");
        event["authority_sequence"] = json!(2);
        event["previous_event_sha256"] = base_pin["event_chain_head_sha256"].clone();
        event["cause_class"] = json!("source_corrected");
        event["invalidation_semantics"] = json!("reverify_required");
        event["valid_time"]["effective_at"] = json!("2026-11-02T10:30:00Z");
        event["system_time"]["recorded_at"] = json!("2026-11-02T11:00:00Z");
        event["system_time"]["trusted_at"] = json!("2026-11-02T11:01:00Z");
        event["targets"] = json!([{
            "node_id": node_id,
            "node_type": "source_measurement",
            "artifact_sha256": "0202020202020202020202020202020202020202020202020202020202020202"
        }]);
        event
    }

    fn metadata_for_next_event(base: &Value) -> Value {
        json!({
            "evaluated_at": "2026-11-03T07:00:00Z",
            "impact_policy_sha256": base["impact_policy_sha256"],
            "impact_engine_build_sha256": base["impact_engine_build_sha256"]
        })
    }

    #[test]
    fn transitioning_from_no_impact_removes_stale_no_dependency_reason() {
        let previous = json!({
            "node_id": "decision:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "impact_class": "no_impact",
            "historical_authenticity": "valid_at_time",
            "current_reliance": "eligible",
            "dependency_path": [],
            "reason_codes": ["NO_DEPENDENCY_PATH", "OUTSIDE_INVALIDATION_SCOPE"]
        });
        let incoming = json!({
            "node_id": "decision:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "impact_class": "quarantined_pending_review",
            "historical_authenticity": "unknown",
            "current_reliance": "quarantined",
            "dependency_path": ["issuer_key:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"],
            "reason_codes": ["DECISION_DEPENDS_ON_QUARANTINED_STATEMENT"]
        });
        let merged = merge_impact_rows(&previous, &incoming);
        let reasons = merged["reason_codes"].as_array().unwrap();
        assert!(reasons.iter().all(|reason| {
            !matches!(
                reason.as_str(),
                Some("NO_DEPENDENCY_PATH" | "OUTSIDE_INVALIDATION_SCOPE")
            )
        }));
        assert!(reasons.iter().any(|reason| {
            reason == "DECISION_DEPENDS_ON_QUARANTINED_STATEMENT"
        }));
    }

    #[test]
    fn sparse_delta_applies_exactly_and_matches_full_graph_transition() {
        let (graph, base_report, base_pin, _, index) = setup();
        let event = source_correction_event(&base_pin);
        let metadata = metadata_for_next_event(&base_report);
        let transition = compute_transition(&index, &base_report, &base_pin, &event, &metadata)
            .unwrap();
        let applied = apply_delta(
            &index,
            &base_report,
            &base_pin,
            &event,
            &metadata,
            &transition.delta,
        )
        .unwrap();

        assert_eq!(applied.report, transition.report);
        assert_eq!(applied.state_pin, transition.state_pin);
        assert_eq!(
            transition.delta["closure_verifier"],
            "independent_iterative_dfs_v0.1"
        );
        assert_eq!(transition.delta["closure_complete"], true);
        assert!(transition.delta["affected_node_count"].as_u64().unwrap() > 0);
        assert!(
            transition.delta["update_count"].as_u64().unwrap()
                <= transition.delta["affected_node_count"].as_u64().unwrap()
        );
        assert!(
            transition.delta["updates"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["impact_class"] != "no_impact")
        );

        // Full-report reconciliation oracle: evaluate every graph node for the
        // event, then conservatively join that report with the prior full state.
        let nodes = validate_graph(&graph).unwrap();
        let full_event_report = lifecycle::build_impact_report(
            &graph,
            &event,
            &metadata,
            &nodes,
        )
        .unwrap();
        let mut expected_rows = report_rows(&base_report, &index).unwrap();
        for row in full_event_report["results"].as_array().unwrap() {
            let id = row["node_id"].as_str().unwrap();
            let previous = expected_rows.get(id).unwrap().clone();
            expected_rows.insert(id.to_owned(), merge_impact_rows(&previous, row));
        }
        let mut expected_report = base_report.clone();
        expected_report["report_id"] = full_event_report["report_id"].clone();
        expected_report["invalidation_event_sha256"] =
            full_event_report["invalidation_event_sha256"].clone();
        expected_report["evaluated_at"] = full_event_report["evaluated_at"].clone();
        expected_report["results"] = json!(expected_rows.values().cloned().collect::<Vec<_>>());
        expected_report["summary"] = summarize_rows(&expected_rows);
        assert_eq!(applied.report, expected_report);
    }

    #[test]
    fn stale_base_report_graph_sequence_and_predecessor_are_rejected() {
        let (graph, base_report, base_pin, _, index) = setup();
        let event = source_correction_event(&base_pin);
        let metadata = metadata_for_next_event(&base_report);
        let transition = compute_transition(&index, &base_report, &base_pin, &event, &metadata)
            .unwrap();

        let mut stale_report = base_report.clone();
        stale_report["report_id"] = json!("mutated-base-report");
        assert!(apply_delta(
            &index,
            &stale_report,
            &base_pin,
            &event,
            &metadata,
            &transition.delta
        )
        .is_err());

        let mut wrong_graph_pin = base_pin.clone();
        wrong_graph_pin["graph_snapshot_sha256"] = json!("f".repeat(64));
        assert!(validate_state_pin(&index, &base_report, &wrong_graph_pin).is_err());

        let mut gap = event.clone();
        gap["authority_sequence"] = json!(3);
        assert!(compute_transition(&index, &base_report, &base_pin, &gap, &metadata).is_err());

        let mut broken_predecessor = event.clone();
        broken_predecessor["previous_event_sha256"] = json!("d".repeat(64));
        assert!(
            compute_transition(&index, &base_report, &base_pin, &broken_predecessor, &metadata)
                .is_err()
        );

        let _ = graph;
    }

    #[test]
    fn event_recorded_or_trusted_time_cannot_roll_back_behind_the_state_pin() {
        let (_graph, base_report, base_pin, _, index) = setup();
        let event = source_correction_event(&base_pin);
        let metadata = metadata_for_next_event(&base_report);

        let mut backdated = event.clone();
        backdated["system_time"]["recorded_at"] = json!("2026-11-01T09:59:00Z");
        backdated["system_time"]["trusted_at"] = json!("2026-11-01T10:00:00Z");
        assert!(
            compute_transition(&index, &base_report, &base_pin, &backdated, &metadata).is_err()
        );

        let mut trusted_backdated = event;
        trusted_backdated["system_time"]["recorded_at"] = json!("2026-11-02T11:00:00Z");
        trusted_backdated["system_time"]["trusted_at"] = json!("2026-11-01T10:00:59Z");
        assert!(
            compute_transition(
                &index,
                &base_report,
                &base_pin,
                &trusted_backdated,
                &metadata
            )
            .is_err()
        );
    }

    #[test]
    fn evaluation_time_cannot_roll_back_behind_the_pinned_report_time() {
        let (_graph, base_report, base_pin, _, index) = setup();
        let mut event = source_correction_event(&base_pin);
        event["valid_time"]["effective_at"] = json!("2026-11-01T10:07:00Z");
        event["system_time"]["recorded_at"] = json!("2026-11-01T10:05:00Z");
        event["system_time"]["trusted_at"] = json!("2026-11-01T10:06:00Z");
        let mut metadata = metadata_for_next_event(&base_report);
        metadata["evaluated_at"] = json!("2026-11-01T10:10:00Z");
        assert!(
            compute_transition(&index, &base_report, &base_pin, &event, &metadata).is_err()
        );
    }

    #[test]
    fn delta_tampering_is_rejected_even_when_payload_digest_is_resealed() {
        let (graph, base_report, base_pin, _, index) = setup();
        let event = source_correction_event(&base_pin);
        let metadata = metadata_for_next_event(&base_report);
        let transition = compute_transition(&index, &base_report, &base_pin, &event, &metadata)
            .unwrap();
        let mut tampered = transition.delta.clone();
        if let Some(first) = tampered["updates"].as_array_mut().and_then(|rows| rows.first_mut()) {
            first["impact_class"] = json!("claim_invalidated");
        }
        let digest = canonical_digest_without_field(&tampered, "delta_sha256").unwrap();
        tampered["delta_sha256"] = json!(digest);
        assert!(apply_delta(&index, &base_report, &base_pin, &event, &metadata, &tampered).is_err());
        let _ = graph;
    }

    #[test]
    fn exact_replay_is_idempotent_but_replay_after_high_water_advance_is_rejected() {
        let (graph, base_report, base_pin, _, index) = setup();
        let event = source_correction_event(&base_pin);
        let metadata = metadata_for_next_event(&base_report);
        let transition = compute_transition(&index, &base_report, &base_pin, &event, &metadata)
            .unwrap();
        let first = apply_delta(
            &index,
            &base_report,
            &base_pin,
            &event,
            &metadata,
            &transition.delta,
        )
        .unwrap();
        let exact_replay = apply_delta(
            &index,
            &base_report,
            &base_pin,
            &event,
            &metadata,
            &transition.delta,
        )
        .unwrap();
        assert_eq!(first.report, exact_replay.report);
        assert_eq!(first.state_pin, exact_replay.state_pin);
        assert!(apply_delta(
            &index,
            &first.report,
            &first.state_pin,
            &event,
            &metadata,
            &transition.delta
        )
        .is_err());
        let _ = graph;
    }

    #[test]
    fn batched_events_reuse_one_validated_graph_index() {
        let (graph, base_report, base_pin, _, _) = setup();
        let event_two = source_correction_event(&base_pin);
        let mut event_three = event_two.clone();
        let event_two_sha = canonical_json_digest(&event_two).unwrap();
        event_three["event_id"] = json!("evt-source-correction-003");
        event_three["authority_sequence"] = json!(3);
        event_three["previous_event_sha256"] = json!(event_two_sha);
        event_three["system_time"]["recorded_at"] = json!("2026-11-02T12:00:00Z");
        event_three["system_time"]["trusted_at"] = json!("2026-11-02T12:01:00Z");
        let metadata = metadata_for_next_event(&base_report);
        let envelope = json!({
            "schema_version": DELTA_INPUT_SCHEMA,
            "events": [event_two,event_three],
            "metadata": metadata
        });
        let batch = run_batch(&graph, &base_report, &base_pin, &envelope).unwrap();
        assert_eq!(batch["graph_index_build_count"], 1);
        assert_eq!(batch["graph_index_reused_for_events"], 2);
        assert_eq!(batch["state_row_map_initializations"], 1);
        assert_eq!(batch["state_row_map_reused_for_events"], 2);
        assert_eq!(batch["full_report_materializations"], 1);
        assert!(batch["total_closure_node_memberships"].as_u64().unwrap() <= 2_000_000);
        assert_eq!(batch["delta_count"], 2);
        assert_eq!(batch["next_state_pin"]["last_event_sequence"], 3);
        assert_eq!(
            batch["deltas"][0]["base_state_sha256"],
            base_pin["impact_state_sha256"]
        );
        assert_eq!(
            batch["deltas"][1]["base_state_sha256"],
            batch["deltas"][0]["resulting_state_sha256"]
        );
        assert_eq!(
            batch["next_state_pin"]["impact_state_sha256"],
            batch["deltas"][1]["resulting_state_sha256"]
        );
        assert_eq!(
            batch["next_state_pin"]["impact_report_sha256"],
            canonical_json_digest(&batch["resulting_impact_report"]).unwrap()
        );
        assert_eq!(
            verify_batch(&graph, &base_report, &base_pin, &envelope, &batch).unwrap(),
            batch
        );

        let mut tampered = batch.clone();
        tampered["deltas"][1]["resulting_state_sha256"] = json!("f".repeat(64));
        let resealed_delta =
            canonical_digest_without_field(&tampered["deltas"][1], "delta_sha256").unwrap();
        tampered["deltas"][1]["delta_sha256"] = json!(resealed_delta);
        assert!(verify_batch(&graph, &base_report, &base_pin, &envelope, &tampered).is_err());
    }
}
