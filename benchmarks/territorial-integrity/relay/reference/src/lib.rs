pub mod archive;

//! Synthetic CTER relay reference model around Mycelix-DeSci signed events.
//!
//! Event identity is immutable. Verification assessments and disclosure /
//! projection annotations are detached, time-scoped context. This model does
//! not verify signatures, key authorization, assessment signatures, BLAKE3,
//! consent, access permissions, or legal authority.
//!
//! "accepted_events" means eligible for evidence preservation under the
//! externally supplied verifier dispositions and stream-chain structure only.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub accepted_events: BTreeSet<String>,
    pub duplicate_replays: u64,
    pub invalid_signatures: BTreeSet<String>,
    pub revoked_keys: BTreeSet<String>,
    pub verification_pending: BTreeSet<String>,
    pub verification_conflict_event_ids: BTreeSet<String>,
    pub malformed_events: BTreeSet<String>,
    pub missing_parent_events: BTreeSet<String>,
    pub blocked_parent_events: BTreeSet<String>,
    pub equivocation_event_ids: BTreeSet<String>,
    pub sequence_fork_event_ids: BTreeSet<String>,
    pub sequence_conflict_pending_event_ids: BTreeSet<String>,
    pub hash_collision_event_ids: BTreeSet<String>,
    pub context_conflict_event_ids: BTreeSet<String>,
    pub conflict_groups: BTreeMap<String, BTreeSet<(String, String)>>,
}

#[derive(Debug, Clone)]
struct ParsedEvent {
    id: String,
    stream_id: String,
    sequence: u64,
    previous_hash: Option<String>,
    event_hash: String,
    canonical_event: String,
    assessments: BTreeSet<(String, String, String, String)>,
    contexts: BTreeSet<(String, String)>,
    equivocation: bool,
}

fn nonempty_string(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().filter(|s| !s.trim().is_empty()).map(str::to_owned)
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Array(items) => format!("[{}]", items.iter().map(canonical_json).collect::<Vec<_>>().join(",")),
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let fields = keys.into_iter().map(|key| {
                format!("{}:{}", serde_json::to_string(key).expect("string key"), canonical_json(&map[key]))
            }).collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
        _ => value.to_string(),
    }
}

fn parse_delivery(value: &Value) -> Option<ParsedEvent> {
    let event = value.get("event")?;
    let assessment = value.get("assessment")?;
    let context = value.get("context")?;
    let payload = value.get("payload")?;
    let _blob_ref = nonempty_string(payload, "blob_ref")?;
    let _blob_media_type = nonempty_string(payload, "media_type")?;
    let blob_sha256 = nonempty_string(payload, "blob_sha256")?;
    if !is_lower_hex_digest(&blob_sha256) {
        return None;
    }

    let id = nonempty_string(event, "event_id")?;
    let stream_id = nonempty_string(event, "stream_id")?;
    let sequence = event.get("sequence")?.as_u64()?;
    let event_hash = nonempty_string(event, "event_hash")?;
    if !is_lower_hex_digest(&event_hash) {
        return None;
    }
    // Actor is part of the canonical signed Mycelix envelope. The adapter
    // checks presence only; actor/key authorization is a separate Mycelix gate.
    let _actor = nonempty_string(event, "actor")?;
    let previous_hash = match event.get("previous_hash")? {
        Value::Null => None,
        Value::String(hash) if is_lower_hex_digest(hash) => Some(hash.clone()),
        _ => return None,
    };
    if (sequence == 0 && previous_hash.is_some()) || (sequence > 0 && previous_hash.is_none()) {
        return None;
    }

    let _assessment_id = nonempty_string(assessment, "assessment_id")?;
    let _verifier_id = nonempty_string(assessment, "verifier_id")?;
    let policy_id = nonempty_string(assessment, "policy_id")?;
    let _assessed_at = nonempty_string(assessment, "assessed_at")?;
    let key_status_snapshot_ref = nonempty_string(assessment, "key_status_snapshot_ref")?;
    let time_basis = nonempty_string(assessment, "event_time_basis")?;
    if !matches!(
        time_basis.as_str(),
        "trusted_append_receipt" | "external_trusted_timestamp" | "signer_claim_only" | "unknown"
    ) {
        return None;
    }
    let time_evidence_ref = nonempty_string(assessment, "time_evidence_ref");
    let raw_status = nonempty_string(assessment, "status")?;
    if !matches!(raw_status.as_str(), "verified" | "invalid" | "revoked" | "unavailable") {
        return None;
    }
    // A signer-controlled occurred_at is not an authenticated timestamp.
    // A trusted basis without its receipt/timestamp evidence reference is
    // also insufficient: downgrade verified/revoked to unavailable.
    let has_trusted_time = matches!(
        time_basis.as_str(),
        "trusted_append_receipt" | "external_trusted_timestamp"
    ) && time_evidence_ref.is_some();
    let status = match raw_status.as_str() {
        "invalid" => "invalid",
        "unavailable" => "unavailable",
        "verified" | "revoked" if has_trusted_time => {
            if raw_status == "verified" { "verified" } else { "revoked" }
        }
        _ => "unavailable",
    }.to_string();

    let projection_key = nonempty_string(context, "projection_key")?;
    let access_class = nonempty_string(context, "access_class")?;
    if !matches!(
        access_class.as_str(),
        "public" | "community" | "restricted" | "culturally_protected"
    ) {
        return None;
    }

    Some(ParsedEvent {
        id,
        stream_id,
        sequence,
        previous_hash,
        event_hash,
        canonical_event: canonical_json(event),
        assessments: BTreeSet::from([(policy_id, status, key_status_snapshot_ref, time_basis)]),
        contexts: BTreeSet::from([(projection_key, access_class)]),
        equivocation: false,
    })
}

/// Deterministic set-union merge of deliveries. Different detached assessments
/// remain separate; evaluate() aggregates them against immutable event identity.
pub fn merge_event_sets(left: &[Value], right: &[Value]) -> Vec<Value> {
    let mut merged: BTreeMap<String, Value> = BTreeMap::new();
    for delivery in left.iter().chain(right.iter()) {
        merged.insert(canonical_json(delivery), delivery.clone());
    }
    merged.into_values().collect()
}

fn effective_status(event: &ParsedEvent, result: &mut Evaluation) -> Option<String> {
    if event.assessments.len() != 1 {
        result.verification_conflict_event_ids.insert(event.id.clone());
        return None;
    }
    event.assessments.iter().next().map(|(_, status, _, _)| status.clone())
}

/// Evaluate a batch of Mycelix-shaped event deliveries.
///
/// `event.event_hash` is the BLAKE3 hash supplied by the independent Mycelix
/// verifier. Detached verifier results and context labels are not part of the
/// signed envelope and cannot make an otherwise identical event equivocate.
pub fn evaluate(input: &[Value]) -> Evaluation {
    let mut result = Evaluation::default();
    let mut variants: BTreeMap<String, BTreeMap<String, ParsedEvent>> = BTreeMap::new();
    let mut hash_to_ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (index, delivery) in input.iter().enumerate() {
        let id_hint = delivery.get("event").and_then(|e| e.get("event_id"))
            .and_then(Value::as_str).filter(|s| !s.is_empty())
            .map(str::to_owned).unwrap_or_else(|| format!("#delivery-{index}"));
        match parse_delivery(delivery) {
            Some(parsed) => {
                hash_to_ids.entry(parsed.event_hash.clone()).or_default().insert(parsed.id.clone());
                let by_hash = variants.entry(parsed.id.clone()).or_default();
                if let Some(existing) = by_hash.get_mut(&parsed.event_hash) {
                    if existing.canonical_event != parsed.canonical_event {
                        existing.equivocation = true;
                    } else {
                        result.duplicate_replays += 1;
                        existing.assessments.extend(parsed.assessments);
                        existing.contexts.extend(parsed.contexts);
                    }
                } else {
                    by_hash.insert(parsed.event_hash.clone(), parsed);
                }
            }
            None => {
                result.malformed_events.insert(id_hint);
            }
        }
    }

    let mut unique: BTreeMap<String, ParsedEvent> = BTreeMap::new();
    for (id, by_hash) in variants {
        if by_hash.len() != 1 {
            result.equivocation_event_ids.insert(id);
            continue;
        }
        if let Some((_, event)) = by_hash.into_iter().next() {
            if event.equivocation {
                result.equivocation_event_ids.insert(id);
            } else {
                unique.insert(id, event);
            }
        }
    }

    // One BLAKE3 event hash under multiple event IDs is an integrity failure.
    for ids in hash_to_ids.values() {
        if ids.len() > 1 {
            result.hash_collision_event_ids.extend(ids.iter().cloned());
        }
    }

    let mut statuses: BTreeMap<String, String> = BTreeMap::new();
    for (id, event) in &unique {
        if let Some(status) = effective_status(event, &mut result) {
            statuses.insert(id.clone(), status);
        }
        if event.contexts.len() > 1 {
            result.context_conflict_event_ids.insert(id.clone());
        }
    }

    // Stream sequence is not resolved by arrival order. A potentially competing
    // event with unresolved verification holds the entire slot.
    let mut sequence_groups: BTreeMap<(String, u64), BTreeSet<String>> = BTreeMap::new();
    for (id, event) in &unique {
        if !result.hash_collision_event_ids.contains(id)
            && statuses.get(id).map(String::as_str) != Some("invalid")
        {
            sequence_groups.entry((event.stream_id.clone(), event.sequence))
                .or_default().insert(id.clone());
        }
    }
    for ids in sequence_groups.values() {
        if ids.len() > 1 {
            let all_verified = ids.iter().all(|id| statuses.get(id).map(String::as_str) == Some("verified"));
            if all_verified {
                result.sequence_fork_event_ids.extend(ids.iter().cloned());
            } else {
                result.sequence_conflict_pending_event_ids.extend(ids.iter().cloned());
            }
        }
    }

    let mut eligible: BTreeMap<String, ParsedEvent> = BTreeMap::new();
    for (id, event) in &unique {
        match statuses.get(id).map(String::as_str) {
            Some("invalid") => { result.invalid_signatures.insert(id.clone()); }
            Some("revoked") => { result.revoked_keys.insert(id.clone()); }
            Some("unavailable") => { result.verification_pending.insert(id.clone()); }
            Some("verified")
                if !result.sequence_fork_event_ids.contains(id)
                    && !result.sequence_conflict_pending_event_ids.contains(id)
                    && !result.hash_collision_event_ids.contains(id) =>
            {
                eligible.insert(id.clone(), event.clone());
            }
            _ => {}
        }
    }

    // Build the predecessor DAG once, then propagate acceptance from genesis
    // events with a queue. Mycelix sequence links are stream-local and must
    // increase by exactly one, so a valid predecessor graph is acyclic.
    let mut children: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut ready = VecDeque::new();
    for (id, event) in &eligible {
        let Some(previous_hash) = event.previous_hash.as_ref() else {
            ready.push_back(id.clone());
            continue;
        };
        let Some(parent_ids) = hash_to_ids.get(previous_hash) else {
            result.missing_parent_events.insert(id.clone());
            continue;
        };
        if parent_ids.len() != 1 {
            result.blocked_parent_events.insert(id.clone());
            continue;
        }
        let parent_id = parent_ids.iter().next().expect("one predecessor");
        let Some(parent) = eligible.get(parent_id) else {
            result.blocked_parent_events.insert(id.clone());
            continue;
        };
        if parent.stream_id != event.stream_id
            || parent.sequence.checked_add(1) != Some(event.sequence)
        {
            result.blocked_parent_events.insert(id.clone());
            continue;
        }
        children.entry(parent_id.clone()).or_default().push(id.clone());
    }

    while let Some(id) = ready.pop_front() {
        if !result.accepted_events.insert(id.clone()) {
            continue;
        }
        if let Some(descendants) = children.get(&id) {
            ready.extend(descendants.iter().cloned());
        }
    }

    // Any structurally eligible descendant whose predecessor chain never
    // reached genesis is blocked. This includes descendants of missing,
    // invalid, revoked, unavailable, equivocated or forked predecessor events.
    for id in eligible.keys() {
        if !result.accepted_events.contains(id)
            && !result.missing_parent_events.contains(id)
            && !result.blocked_parent_events.contains(id)
        {
            result.blocked_parent_events.insert(id.clone());
        }
    }

    // Store one compact group per disputed projection, not every pair of
    // conflicting events. This avoids O(k^2) output for a projection with k
    // independently asserted streams.
    let mut projections: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for id in &result.accepted_events {
        let Some(event) = eligible.get(id) else { continue };
        if event.contexts.len() != 1 {
            continue;
        }
        let context = event.contexts.iter().next().expect("one context");
        let projection_key = context.0.clone();
        projections.entry(projection_key)
            .or_default()
            .insert((id.clone(), event.stream_id.clone()));
    }
    for (projection_key, members) in projections {
        let distinct_streams: BTreeSet<&str> = members.iter()
            .map(|(_, stream_id)| stream_id.as_str())
            .collect();
        if distinct_streams.len() > 1 {
            result.conflict_groups.insert(projection_key, members);
        }
    }
    result
}

impl Evaluation {
    pub fn to_json(&self) -> Value {
        let groups: Vec<Value> = self.conflict_groups.iter().map(|(projection_key, members)| {
            let entries: Vec<Value> = members.iter().map(|(event_id, stream_id)| {
                json!({"event_id": event_id, "stream_id": stream_id})
            }).collect();
            json!({"projection_key": projection_key, "members": entries})
        }).collect();
        json!({
            "accepted_events": self.accepted_events.iter().cloned().collect::<Vec<_>>(),
            "duplicate_replays": self.duplicate_replays,
            "invalid_signatures": self.invalid_signatures.iter().cloned().collect::<Vec<_>>(),
            "revoked_keys": self.revoked_keys.iter().cloned().collect::<Vec<_>>(),
            "verification_pending": self.verification_pending.iter().cloned().collect::<Vec<_>>(),
            "verification_conflict_event_ids": self.verification_conflict_event_ids.iter().cloned().collect::<Vec<_>>(),
            "malformed_events": self.malformed_events.iter().cloned().collect::<Vec<_>>(),
            "missing_parent_events": self.missing_parent_events.iter().cloned().collect::<Vec<_>>(),
            "blocked_parent_events": self.blocked_parent_events.iter().cloned().collect::<Vec<_>>(),
            "equivocation_event_ids": self.equivocation_event_ids.iter().cloned().collect::<Vec<_>>(),
            "sequence_fork_event_ids": self.sequence_fork_event_ids.iter().cloned().collect::<Vec<_>>(),
            "sequence_conflict_pending_event_ids": self.sequence_conflict_pending_event_ids.iter().cloned().collect::<Vec<_>>(),
            "hash_collision_event_ids": self.hash_collision_event_ids.iter().cloned().collect::<Vec<_>>(),
            "context_conflict_event_ids": self.context_conflict_event_ids.iter().cloned().collect::<Vec<_>>(),
            "conflict_groups": groups
        })
    }
}
