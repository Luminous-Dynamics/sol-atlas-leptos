use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::process::ExitCode;

const REPORT_SCHEMA: &str = "luminous.ieac-scitt-event-history-audit-report.v0.1";
const FIXTURE_SCHEMA: &str = "luminous.ieac-scitt-fixture-corpus.v0.1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug)]
struct Event {
    id: String,
    issuer: String,
    subject: String,
    case_id: String,
    stream_id: String,
    sequence: u64,
    previous: Option<String>,
    digest: String,
    parents: Vec<String>,
}

#[derive(Default)]
struct Audit {
    findings: Vec<Value>,
    events: Vec<Event>,
    digest_to_index: BTreeMap<String, usize>,
    id_to_index: BTreeMap<String, usize>,
    streams: BTreeMap<(String, String, String), BTreeMap<u64, usize>>,
    edges: Vec<(usize, usize)>,
    fatal: bool,
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

// This profile's event payloads are restricted to ASCII strings and safe integers.
// For that subset, UTF-16 property order is identical to bytewise ASCII order and
// ECMAScript number serialization is the ordinary base-10 integer representation.
// Non-ASCII strings and non-integral numbers are explicitly unsupported here.
fn append_jcs_subset(value: &Value, output: &mut Vec<u8>) -> Result<(), String> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(value) => output.extend_from_slice(if *value { b"true" } else { b"false" }),
        Value::Number(number) => append_safe_integer(number, output)?,
        Value::String(value) => append_ascii_string(value, output)?,
        Value::Array(values) => {
            output.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index > 0 { output.push(b','); }
                append_jcs_subset(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut entries: Vec<(&String, &Value)> = values.iter().collect();
            for (key, _) in &entries {
                if !key.is_ascii() {
                    return Err("non-ASCII object keys are unsupported by the v0.1 JCS subset".into());
                }
            }
            entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            for (index, (key, item)) in entries.iter().enumerate() {
                if index > 0 { output.push(b','); }
                append_ascii_string(key, output)?;
                output.push(b':');
                append_jcs_subset(item, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn append_safe_integer(number: &Number, output: &mut Vec<u8>) -> Result<(), String> {
    let rendered = if let Some(value) = number.as_u64() {
        if value > MAX_SAFE_INTEGER {
            return Err("integer exceeds the interoperable IEEE-754 safe-integer range".into());
        }
        value.to_string()
    } else if let Some(value) = number.as_i64() {
        if value.unsigned_abs() > MAX_SAFE_INTEGER {
            return Err("integer exceeds the interoperable IEEE-754 safe-integer range".into());
        }
        value.to_string()
    } else {
        return Err("non-integral JSON numbers are unsupported by the v0.1 JCS subset".into());
    };
    output.extend_from_slice(rendered.as_bytes());
    Ok(())
}

fn append_ascii_string(value: &str, output: &mut Vec<u8>) -> Result<(), String> {
    if !value.is_ascii() {
        return Err("non-ASCII strings are unsupported by the v0.1 JCS subset".into());
    }
    output.push(b'"');
    for byte in value.bytes() {
        match byte {
            b'"' => output.extend_from_slice(b"\\\""),
            b'\\' => output.extend_from_slice(b"\\\\"),
            0x08 => output.extend_from_slice(b"\\b"),
            0x09 => output.extend_from_slice(b"\\t"),
            0x0a => output.extend_from_slice(b"\\n"),
            0x0c => output.extend_from_slice(b"\\f"),
            0x0d => output.extend_from_slice(b"\\r"),
            0x00..=0x1f => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                output.extend_from_slice(b"\\u00");
                output.push(HEX[(byte >> 4) as usize]);
                output.push(HEX[(byte & 0x0f) as usize]);
            }
            _ => output.push(byte),
        }
    }
    output.push(b'"');
    Ok(())
}

fn canonical_jcs_subset(value: &Value) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    append_jcs_subset(value, &mut output)?;
    Ok(output)
}

fn required_string(value: &Value, key: &str, context: &str) -> Result<String, String> {
    value.get(key).and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{context}: required non-empty string '{key}' is missing"))
}

fn required_u64(value: &Value, key: &str, context: &str) -> Result<u64, String> {
    value.get(key).and_then(Value::as_u64)
        .ok_or_else(|| format!("{context}: required unsigned integer '{key}' is missing"))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn finding(code: &str, severity: &str, message: impl Into<String>) -> Value {
    json!({"code": code, "severity": severity, "message": message.into()})
}

fn add_finding(audit: &mut Audit, code: &str, severity: &str, message: impl Into<String>) {
    if severity == "invalid" { audit.fatal = true; }
    audit.findings.push(finding(code, severity, message));
}

fn parse_events(input: &Value, audit: &mut Audit) -> Result<(), String> {
    if input.get("fixture_schema_version").and_then(Value::as_str) != Some(FIXTURE_SCHEMA) {
        return Err("unsupported fixture schema; expected the SCITT synthetic event corpus v0.1".into());
    }
    if input.get("fixture_status").and_then(Value::as_str) != Some("synthetic_only") {
        return Err("v0.1 auditor accepts only fixtures explicitly marked synthetic_only".into());
    }
    let rows = input.get("events").and_then(Value::as_array)
        .ok_or_else(|| "fixture.events must be an array".to_owned())?;
    let expected = input.pointer("/expected/event_payload_sha256").and_then(Value::as_object)
        .ok_or_else(|| "fixture.expected.event_payload_sha256 must be an object".to_owned())?;

    for (index, payload) in rows.iter().enumerate() {
        let context = format!("event[{index}]");
        if payload.get("schema_version").and_then(Value::as_str)
            != Some("luminous.ieac-scitt-decision-statement.v0.1")
        {
            return Err(format!("{context}: unsupported event payload schema"));
        }
        let id = required_string(payload, "event_id", &context)?;
        let issuer = required_string(payload, "issuer_id", &context)?;
        let subject = required_string(payload, "subject_id", &context)?;
        let case_id = required_string(payload, "decision_case_id", &context)?;
        let stream_id = required_string(payload, "issuer_stream_id", &context)?;
        let sequence = required_u64(payload, "issuer_sequence", &context)?;
        if sequence == 0 || sequence > MAX_SAFE_INTEGER {
            return Err(format!("{context}: issuer_sequence outside safe positive integer range"));
        }
        if payload.get("claim_ceiling").and_then(Value::as_str) != Some("record_only")
            || payload.get("action_authority").and_then(Value::as_str) != Some("none")
        {
            return Err(format!("{context}: claim ceiling/action authority exceeds record-only profile"));
        }

        let mut canonical = canonical_jcs_subset(payload)
            .map_err(|error| format!("{context}: unsupported JCS input: {error}"))?;
        let digest = sha256_hex(&canonical);
        canonical.clear();
        let expected_digest = expected.get(&id).and_then(Value::as_str)
            .ok_or_else(|| format!("{context}: no frozen expected digest for event_id '{id}'"))?;
        if !valid_sha256(expected_digest) || digest != expected_digest {
            add_finding(
                audit, "EVENT_PAYLOAD_DIGEST_MISMATCH", "invalid",
                format!("{id}: computed {digest}, fixture expected {expected_digest}"),
            );
        }
        if audit.id_to_index.contains_key(&id) {
            add_finding(audit, "DUPLICATE_EVENT_ID", "invalid", format!("duplicate event_id {id}"));
            continue;
        }
        if audit.digest_to_index.contains_key(&digest) {
            add_finding(audit, "DUPLICATE_EVENT_PAYLOAD", "invalid", format!("multiple rows resolve to payload digest {digest}"));
            continue;
        }

        let previous = match payload.get("previous_event_payload_sha256") {
            Some(Value::Null) => None,
            Some(Value::String(value)) if valid_sha256(value) => Some(value.clone()),
            _ => return Err(format!("{context}: previous_event_payload_sha256 must be null or lowercase SHA-256")),
        };
        if sequence == 1 && previous.is_some() {
            add_finding(audit, "FIRST_EVENT_HAS_PREDECESSOR", "invalid", format!("{id}: sequence 1 must have no predecessor"));
        }
        if sequence > 1 && previous.is_none() {
            add_finding(audit, "LATER_EVENT_MISSING_PREDECESSOR", "invalid", format!("{id}: sequence > 1 requires a predecessor digest"));
        }

        let parents = payload.get("causal_parent_payload_sha256").and_then(Value::as_array)
            .ok_or_else(|| format!("{context}: causal_parent_payload_sha256 must be an array"))?
            .iter()
            .map(|item| item.as_str().filter(|value| valid_sha256(value)).map(str::to_owned)
                .ok_or_else(|| format!("{context}: causal parent must be lowercase SHA-256")))
            .collect::<Result<Vec<_>, _>>()?;

        let event = Event {
            id: id.clone(), issuer: issuer.clone(), subject: subject.clone(), case_id,
            stream_id: stream_id.clone(), sequence, previous, digest: digest.clone(), parents,
        };
        let event_index = audit.events.len();
        audit.id_to_index.insert(id, event_index);
        audit.digest_to_index.insert(digest, event_index);
        audit.streams.entry((issuer, subject, stream_id))
            .or_default().insert(sequence, event_index);
        audit.events.push(event);
    }
    Ok(())
}

fn audit_streams(audit: &mut Audit) {
    let groups: Vec<_> = audit.streams.iter()
        .map(|(key, rows)| (key.clone(), rows.clone()))
        .collect();
    for ((issuer, subject, stream), rows) in groups {
        let first = rows.keys().next().copied().unwrap_or(0);
        let last = rows.keys().next_back().copied().unwrap_or(0);
        if first != 1 {
            add_finding(
                audit, "STREAM_PREFIX_NOT_PRESENT", "incomplete",
                format!("{issuer}/{subject}/{stream}: first observed sequence is {first}, not 1"),
            );
        }
        let mut previous_observed: Option<(u64, String)> = None;
        for (sequence, index) in &rows {
            let event = &audit.events[*index];
            if let Some((prior_sequence, prior_digest)) = &previous_observed {
                if *sequence == prior_sequence + 1 {
                    if event.previous.as_deref() != Some(prior_digest.as_str()) {
                        add_finding(
                            audit, "PREDECESSOR_DIGEST_MISMATCH", "invalid",
                            format!("{} does not reference the immediately preceding event in its issuer stream", event.id),
                        );
                    } else {
                        audit.edges.push((*audit.digest_to_index.get(prior_digest).expect("prior event digest indexed"), *index));
                    }
                } else if *sequence > prior_sequence + 1 {
                    add_finding(
                        audit, "STREAM_SEQUENCE_GAP", "incomplete",
                        format!("{issuer}/{subject}/{stream}: missing sequences {}..{}", prior_sequence + 1, sequence - 1),
                    );
                }
            } else if *sequence > 1 {
                add_finding(
                    audit, "STREAM_PREFIX_NOT_PRESENT", "incomplete",
                    format!("{} begins at sequence {sequence}; earlier issuer history is not in this bundle", event.id),
                );
            }
            previous_observed = Some((*sequence, event.digest.clone()));
        }
        if last < first {
            add_finding(audit, "EMPTY_STREAM", "invalid", "empty issuer stream");
        }
    }
}

fn audit_causal_graph(audit: &mut Audit) {
    let count = audit.events.len();
    for child in 0..count {
        let parents = audit.events[child].parents.clone();
        for parent_digest in parents {
            let Some(parent) = audit.digest_to_index.get(&parent_digest).copied() else {
                add_finding(
                    audit, "CAUSAL_PARENT_NOT_INCLUDED", "incomplete",
                    format!("{} references a parent not present in this event bundle: {parent_digest}", audit.events[child].id),
                );
                continue;
            };
            let parent_event = &audit.events[parent];
            let child_event = &audit.events[child];
            if parent == child {
                add_finding(audit, "CAUSAL_SELF_REFERENCE", "invalid", format!("{} references itself", child_event.id));
            } else if parent_event.case_id != child_event.case_id || parent_event.subject != child_event.subject {
                add_finding(
                    audit, "CAUSAL_SCOPE_MISMATCH", "invalid",
                    format!("{} references an event from a different case or subject", child_event.id),
                );
            } else {
                audit.edges.push((parent, child));
            }
        }
    }

    // Kahn's algorithm: every event must be reachable in an acyclic directed graph.
    let mut indegree = vec![0usize; count];
    let mut outgoing = vec![Vec::<usize>::new(); count];
    let mut unique_edges = BTreeSet::new();
    for (from, to) in &audit.edges {
        if unique_edges.insert((*from, *to)) {
            indegree[*to] += 1;
            outgoing[*from].push(*to);
        }
    }
    let mut ready: BTreeSet<usize> = indegree.iter().enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index)).collect();
    let mut visited = 0usize;
    while let Some(node) = ready.pop_first() {
        visited += 1;
        for next in &outgoing[node] {
            indegree[*next] -= 1;
            if indegree[*next] == 0 { ready.insert(*next); }
        }
    }
    if visited != count {
        add_finding(audit, "CAUSAL_GRAPH_CYCLE", "invalid", "combined predecessor and causal-parent edges contain a cycle");
    }
    audit.edges = unique_edges.into_iter().collect();
}

fn build_report(input: &Value, mut audit: Audit) -> Value {
    let stream_count = audit.streams.len();
    let mut unique_edges = BTreeSet::new();
    for edge in &audit.edges { unique_edges.insert(*edge); }
    let invalid_count = audit.findings.iter().filter(|item| item["severity"] == "invalid").count();
    let incomplete_count = audit.findings.iter().filter(|item| item["severity"] == "incomplete").count();
    let status = if invalid_count > 0 {
        "invalid"
    } else if incomplete_count > 0 || input.get("checkpoint_observations").and_then(Value::as_array).is_none_or(|values| values.is_empty()) {
        "valid_incomplete"
    } else {
        // This early profile intentionally never claims checkpoint authentication:
        // checkpoint_observations are descriptive inputs, not verified signatures.
        "valid_incomplete"
    };
    audit.findings.sort_by(|left, right| {
        left["code"].as_str().cmp(&right["code"].as_str())
            .then_with(|| left["message"].as_str().cmp(&right["message"].as_str()))
    });
    json!({
        "schema_version": REPORT_SCHEMA,
        "status": status,
        "structural_integrity": if invalid_count == 0 { "valid" } else { "invalid" },
        "history_completeness": "not_established",
        "event_count": audit.events.len(),
        "issuer_stream_count": stream_count,
        "verified_event_payload_digests": audit.events.len() - if invalid_count > 0 { 1 } else { 0 },
        "causal_and_predecessor_edge_count": unique_edges.len(),
        "invalid_finding_count": invalid_count,
        "incomplete_finding_count": incomplete_count,
        "findings": audit.findings,
        "not_checked": [
            "SCITT Signed Statement COSE signatures",
            "SCITT Receipt signatures and receipt-to-statement bindings",
            "Transparency Service VDS inclusion and consistency proofs",
            "issuer or Transparency Service trust-root authorization and revocation",
            "authenticity of any checkpoint or stream-head claim",
            "proof that an issuer created no statements beyond those supplied",
            "truth of source measurements or fitness of physical models"
        ],
        "claim_ceiling": "record_only",
        "action_authority": "none"
    })
}

fn run(input_path: &str, output_path: &str) -> Result<i32, String> {
    let bytes = fs::read(input_path).map_err(|error| format!("cannot read input: {error}"))?;
    let input: Value = serde_json::from_slice(&bytes).map_err(|error| format!("invalid JSON input: {error}"))?;
    let mut audit = Audit::default();
    parse_events(&input, &mut audit)?;
    audit_streams(&mut audit);
    audit_causal_graph(&mut audit);
    let report = build_report(&input, audit);
    let encoded = serde_json::to_vec_pretty(&report).map_err(|error| format!("cannot encode report: {error}"))?;
    fs::write(output_path, encoded).map_err(|error| format!("cannot write report: {error}"))?;
    match report["status"].as_str() {
        Some("valid_incomplete") => Ok(3),
        Some("invalid") => Ok(2),
        _ => Ok(4),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: industry-event-history-audit <scitt-event-fixture.json> <report.json>");
        return ExitCode::from(4);
    }
    match run(&args[1], &args[2]) {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("event-history audit cannot evaluate: {error}");
            ExitCode::from(4)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jcs_ascii_subset_orders_keys_and_escapes_controls() {
        let value: Value = serde_json::from_str(r#"{"z":1,"a":"line\nnext","quote":"\""}"#).unwrap();
        let bytes = canonical_jcs_subset(&value).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), r#"{"a":"line\nnext","quote":"\"","z":1}"#);
    }

    #[test]
    fn jcs_subset_refuses_non_ascii_and_fractional_values() {
        assert!(canonical_jcs_subset(&json!({"name":"Johannesburg"})).is_ok());
        assert!(canonical_jcs_subset(&json!({"name":"München"})).is_err());
        assert!(canonical_jcs_subset(&json!({"number":0.5})).is_err());
    }

    #[test]
    fn frozen_scitt_fixture_has_matching_event_digests_and_is_incomplete() {
        let input: Value = serde_json::from_str(include_str!("../../../../schemas/examples/ieac-scitt-decision-statement-synthetic-v0.1.json")).unwrap();
        let mut audit = Audit::default();
        parse_events(&input, &mut audit).unwrap();
        audit_streams(&mut audit);
        audit_causal_graph(&mut audit);
        let report = build_report(&input, audit);
        assert_eq!(report["status"], "valid_incomplete");
        assert_eq!(report["event_count"], 4);
        assert_eq!(report["issuer_stream_count"], 3);
        assert_eq!(report["causal_and_predecessor_edge_count"], 3);
        assert_eq!(report["invalid_finding_count"], 0);
        assert_eq!(report["history_completeness"], "not_established");
    }

    #[test]
    fn digest_mismatch_is_invalid_not_incomplete() {
        let mut input: Value = serde_json::from_str(include_str!("../../../../schemas/examples/ieac-scitt-decision-statement-synthetic-v0.1.json")).unwrap();
        input["expected"]["event_payload_sha256"]["event-verdict-001"] = json!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut audit = Audit::default();
        parse_events(&input, &mut audit).unwrap();
        assert!(audit.fatal);
        assert!(audit.findings.iter().any(|item| item["code"] == "EVENT_PAYLOAD_DIGEST_MISMATCH"));
    }
}
