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
    for row in [previous, incoming] {
        if let Some(values) = row.get("reason_codes").and_then(Value::as_array) {
            for value in values.iter().filter_map(Value::as_str) {
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
) -> Result<(), String> {
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
    let _ = report_rows(base_report, index)?;
    Ok(())
}

fn make_state_pin(
    index: &GraphIndex,
    report: &Value,
    sequence: u64,
    event_head: &str,
    last_recorded_at: &str,
    last_trusted_at: &str,
) -> Result<Value, String> {
    if sequence == 0 || !valid_sha256(event_head) {
        return Err("state pin requires a positive sequence and lowercase SHA-256 event head".into());
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

fn compute_transition(
    index: &GraphIndex,
    base_report: &Value,
    base_pin: &Value,
    event: &Value,
    metadata: &Value,
) -> Result<Transition, String> {
    validate_state_pin(index, base_report, base_pin)?;
    validate_event(event, &index.nodes)?;

    let previous_sequence = base_pin["last_event_sequence"]
        .as_u64()
        .ok_or("state pin sequence is missing")?;
    let expected_sequence = previous_sequence
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
    if event["previous_event_sha256"].as_str()
        != base_pin["event_chain_head_sha256"].as_str()
    {
        return Err("event predecessor does not match pinned event-chain head".into());
    }

    let event_digest = canonical_json_digest(event)?;
    let event_system_time = event.get("system_time").ok_or("event system_time missing")?;
    let recorded_at = required_string(event_system_time, "recorded_at", "event.system_time")?;
    let trusted_at = required_string(event_system_time, "trusted_at", "event.system_time")?;
    if !valid_utc_timestamp(recorded_at)
        || !valid_utc_timestamp(trusted_at)
        || trusted_at < recorded_at
        || recorded_at < base_pin["last_recorded_at"].as_str().unwrap_or_default()
        || trusted_at < base_pin["last_trusted_at"].as_str().unwrap_or_default()
    {
        return Err("event recorded/trusted times violate the pinned temporal high-water mark".into());
    }
    let evaluated_at = required_string(metadata, "evaluated_at", "delta metadata")?;
    let policy_sha = required_string(metadata, "impact_policy_sha256", "delta metadata")?;
    let engine_sha = required_string(metadata, "impact_engine_build_sha256", "delta metadata")?;
    if !valid_utc_timestamp(evaluated_at) || !valid_sha256(policy_sha) || !valid_sha256(engine_sha) {
        return Err("delta metadata has invalid time, policy digest or engine digest".into());
    }
    if base_report["impact_policy_sha256"].as_str() != Some(policy_sha)
        || base_report["impact_engine_build_sha256"].as_str() != Some(engine_sha)
    {
        return Err("policy or engine identity changed; run full reconciliation before applying a delta".into());
    }

    let base_rows = report_rows(base_report, index)?;
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
    let mut next_rows = base_rows.clone();
    for node_id in &affected_node_ids {
        let previous = base_rows
            .get(node_id)
            .ok_or_else(|| format!("base report lacks affected node {node_id}"))?;
        let incoming = impacted_rows
            .get(node_id)
            .ok_or_else(|| format!("impact row evaluator omitted affected node {node_id}"))?;
        let merged = merge_impact_rows(previous, incoming);
        if merged != *previous {
            updates.push(merged.clone());
            next_rows.insert(node_id.clone(), merged);
        }
    }
    updates.sort_by(|left, right| {
        left["node_id"]
            .as_str()
            .cmp(&right["node_id"].as_str())
    });

    // No row outside the independently proven closure may be changed.
    let next_summary = summarize_rows(&next_rows);
    let mut next_report = base_report.clone();
    next_report["report_id"] = json!(format!(
        "impact-{}",
        required_string(event, "event_id", "event")?
    ));
    next_report["invalidation_event_sha256"] = json!(event_digest);
    next_report["evaluated_at"] = json!(evaluated_at);
    next_report["results"] = json!(
        next_rows.values().cloned().collect::<Vec<_>>()
    );
    next_report["summary"] = next_summary;
    let resulting_report_sha = canonical_json_digest(&next_report)?;

    let update_count = updates.len();
    let affected_node_count = independently_reached.len();
    let mut delta = json!({
        "schema_version": DELTA_SCHEMA,
        "delta_id": format!("delta-{}-{}", sequence, required_string(event, "event_id", "event")?),
        "graph_snapshot_sha256": index.graph_snapshot_sha256,
        "graph_index_sha256": index.index_sha256,
        "base_impact_report_sha256": canonical_json_digest(base_report)?,
        "base_event_sequence": previous_sequence,
        "base_event_chain_head_sha256": base_pin["event_chain_head_sha256"],
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
        "resulting_impact_report_sha256": resulting_report_sha,
        "resulting_event_sequence": sequence,
        "resulting_event_chain_head_sha256": event_digest,
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none",
        "delta_sha256": ""
    });
    let delta_sha = canonical_digest_without_field(&delta, "delta_sha256")?;
    delta["delta_sha256"] = json!(delta_sha);

    let state_pin = make_state_pin(
        index,
        &next_report,
        sequence,
        &event_digest,
        recorded_at,
        trusted_at,
    )?;
    Ok(Transition {
        delta,
        report: next_report,
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
    if canonical_json_digest(&expected.report)?
        != delta["resulting_impact_report_sha256"].as_str().unwrap_or_default()
    {
        return Err("resulting report digest does not match sparse delta application".into());
    }
    Ok(expected)
}

fn run_batch(
    graph: &Value,
    base_report: &Value,
    base_pin: &Value,
    delta_input: &Value,
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

    // The exact graph snapshot is validated and indexed once for the whole batch.
    let index = GraphIndex::build(graph)?;
    validate_state_pin(&index, base_report, base_pin)?;
    let mut current_report = base_report.clone();
    let mut current_pin = base_pin.clone();
    let mut deltas = Vec::with_capacity(events.len());

    for event in events {
        let proposed = compute_transition(
            &index,
            &current_report,
            &current_pin,
            event,
            metadata,
        )?;
        // The producer advances its in-memory state from the exact transition
        // it just computed. External consumers use apply_delta to independently
        // check a received sparse update against their pinned base.
        deltas.push(proposed.delta);
        current_report = proposed.report;
        current_pin = proposed.state_pin;
    }

    let batch = json!({
        "schema_version": BATCH_SCHEMA,
        "graph_snapshot_sha256": index.graph_snapshot_sha256,
        "graph_index_sha256": index.index_sha256,
        "graph_index_build_count": 1,
        "graph_index_reused_for_events": events.len(),
        "graph_node_count": index.nodes.len(),
        "graph_edge_count": index.edge_count,
        "event_count": events.len(),
        "delta_count": deltas.len(),
        "deltas": deltas,
        "resulting_impact_report_sha256": canonical_json_digest(&current_report)?,
        "resulting_event_sequence": current_pin["last_event_sequence"],
        "resulting_event_chain_head_sha256": current_pin["event_chain_head_sha256"],
        "resulting_impact_report": current_report,
        "next_state_pin": current_pin,
        "complete": true,
        "claim_ceiling": "incomplete_or_indeterminate",
        "action_authority": "none"
    });
    Ok(batch)
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
    let pin = make_state_pin(
        &index,
        report,
        sequence,
        event_head,
        last_recorded_at,
        last_trusted_at,
    )?;
    validate_state_pin(&index, report, &pin)?;
    Ok(pin)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
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
        eprintln!("usage: industry-impact-delta <graph.json> <base-report.json> <base-pin.json> <delta-input.json> <new-batch-result.json>");
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
        let base_pin = make_state_pin(
            &index,
            &base_report,
            1,
            &first_digest,
            first_system_time["recorded_at"].as_str().unwrap(),
            first_system_time["trusted_at"].as_str().unwrap(),
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
        assert_eq!(batch["delta_count"], 2);
        assert_eq!(batch["next_state_pin"]["last_event_sequence"], 3);
        assert_eq!(
            batch["next_state_pin"]["impact_report_sha256"],
            canonical_json_digest(&batch["resulting_impact_report"]).unwrap()
        );
    }
}
