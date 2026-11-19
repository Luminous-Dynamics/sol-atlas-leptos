//! Synthetic reference model for CTER relay convergence over the existing
//! Mycelix-DeSci signed event stream.
//!
//! This crate classifies event records after an independent verifier supplies
//! signature/key status and a BLAKE3 event hash. It does NOT implement signing,
//! canonicalization, BLAKE3, key management, transport, authorization or legal
//! review. Access-class metadata is a relay-policy input, not an access-control
//! implementation.
//!
//! "accepted_events" means eligible for evidence preservation under the
//! supplied verifier dispositions and stream-chain structure only.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub accepted_events: BTreeSet<String>,
    pub duplicate_replays: u64,
    pub invalid_signatures: BTreeSet<String>,
    pub revoked_keys: BTreeSet<String>,
    pub verification_pending: BTreeSet<String>,
    pub malformed_events: BTreeSet<String>,
    pub missing_parent_events: BTreeSet<String>,
    pub blocked_parent_events: BTreeSet<String>,
    pub equivocation_event_ids: BTreeSet<String>,
    pub sequence_fork_event_ids: BTreeSet<String>,
    pub hash_collision_event_ids: BTreeSet<String>,
    pub conflict_pairs: BTreeSet<(String, String)>,
}

#[derive(Debug, Clone)]
struct ParsedEvent {
    id: String,
    stream_id: String,
    sequence: u64,
    previous_hash: Option<String>,
    event_hash: String,
    actor: String,
    projection_key: String,
    verification: String,
}

fn nonempty_string(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().filter(|s| !s.trim().is_empty()).map(str::to_owned)
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn parse_event(value: &Value) -> Option<ParsedEvent> {
    let id = nonempty_string(value, "event_id")?;
    let stream_id = nonempty_string(value, "stream_id")?;
    let sequence = value.get("sequence")?.as_u64()?;
    let event_hash = nonempty_string(value, "event_hash")?;
    let actor = nonempty_string(value, "actor")?;
    let projection_key = nonempty_string(value, "projection_key")?;
    let verification = nonempty_string(value, "verification")?;
    if !is_lower_hex_digest(&event_hash)
        || !matches!(verification.as_str(), "verified" | "invalid" | "revoked" | "unavailable")
    {
        return None;
    }
    let previous_hash = match value.get("previous_hash")? {
        Value::Null => None,
        Value::String(hash) if is_lower_hex_digest(hash) => Some(hash.clone()),
        _ => return None,
    };
    let access_class = nonempty_string(value, "access_class")?;
    if !matches!(
        access_class.as_str(),
        "public" | "community" | "restricted" | "culturally_protected"
    ) {
        return None;
    }
    // Match the existing Mycelix stream-envelope invariant: genesis has no
    // previous hash; every non-genesis event must name one.
    if (sequence == 0 && previous_hash.is_some()) || (sequence > 0 && previous_hash.is_none()) {
        return None;
    }
    Some(ParsedEvent {
        id,
        stream_id,
        sequence,
        previous_hash,
        event_hash,
        actor,
        projection_key,
        verification,
    })
}

/// Deterministic set-union merge. Distinct contents under the same event ID
/// are retained as separate variants so evaluate() can mark equivocation.
pub fn merge_event_sets(left: &[Value], right: &[Value]) -> Vec<Value> {
    let mut merged: BTreeMap<(String, String, String), Value> = BTreeMap::new();
    for event in left.iter().chain(right.iter()) {
        let id = event.get("event_id").and_then(Value::as_str).unwrap_or("");
        let hash = event.get("event_hash").and_then(Value::as_str).unwrap_or("");
        let serialized = event.to_string();
        merged.insert((id.to_owned(), hash.to_owned(), serialized), event.clone());
    }
    merged.into_values().collect()
}

/// Evaluate a batch of Mycelix-shaped signed-event references.
///
/// `event_hash` is the Mycelix BLAKE3 hash of the signed canonical event,
/// supplied by the independent Mycelix verifier. This function checks only
/// its shape and linkage; it does not recompute the digest or verify Ed25519.
pub fn evaluate(input: &[Value]) -> Evaluation {
    let mut result = Evaluation::default();
    let mut variants: BTreeMap<String, BTreeMap<String, (String, ParsedEvent)>> = BTreeMap::new();
    let mut hash_to_ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (index, value) in input.iter().enumerate() {
        let id_hint = value.get("event_id").and_then(Value::as_str)
            .filter(|s| !s.is_empty()).map(str::to_owned)
            .unwrap_or_else(|| format!("#event-{index}"));
        match parse_event(value) {
            Some(event) => {
                hash_to_ids.entry(event.event_hash.clone()).or_default().insert(event.id.clone());
                let canonical_text = value.to_string();
                let by_hash = variants.entry(event.id.clone()).or_default();
                if let Some((existing_text, _)) = by_hash.get(&event.event_hash) {
                    if existing_text == &canonical_text {
                        result.duplicate_replays += 1;
                    } else {
                        let variant_key = format!("{}:variant:{}", event.event_hash, canonical_text);
                        by_hash.insert(variant_key, (canonical_text, event));
                    }
                } else {
                    by_hash.insert(event.event_hash.clone(), (canonical_text, event));
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
        if let Some((_, event)) = by_hash.into_values().next() {
            unique.insert(id, event);
        }
    }

    // BLAKE3 event hashes are globally content-addressed. The same digest
    // under different event IDs is treated as an integrity collision, not
    // as two interchangeable parents.
    for ids in hash_to_ids.values() {
        if ids.len() > 1 {
            result.hash_collision_event_ids.extend(ids.iter().cloned());
        }
    }

    // Sequence is per Mycelix claim stream, not per signer/relay/device.
    // Invalid or unverifiable signatures must not poison a valid stream head.
    let mut sequence_groups: BTreeMap<(String, u64), BTreeSet<String>> = BTreeMap::new();
    for (id, event) in &unique {
        if event.verification == "verified" && !result.hash_collision_event_ids.contains(id) {
            sequence_groups.entry((event.stream_id.clone(), event.sequence))
                .or_default().insert(id.clone());
        }
    }
    for ids in sequence_groups.values() {
        if ids.len() > 1 {
            result.sequence_fork_event_ids.extend(ids.iter().cloned());
        }
    }

    let mut eligible = BTreeMap::new();
    for (id, event) in &unique {
        match event.verification.as_str() {
            "invalid" => { result.invalid_signatures.insert(id.clone()); }
            "revoked" => { result.revoked_keys.insert(id.clone()); }
            "unavailable" => { result.verification_pending.insert(id.clone()); }
            "verified"
                if !result.sequence_fork_event_ids.contains(id)
                    && !result.hash_collision_event_ids.contains(id) =>
            {
                eligible.insert(id.clone(), event.clone());
            }
            _ => {}
        }
    }

    let mut unresolved: BTreeSet<String> = eligible.keys().cloned().collect();
    loop {
        let mut progressed = false;
        let current: Vec<String> = unresolved.iter().cloned().collect();
        for id in current {
            let Some(event) = eligible.get(&id) else { continue };
            let Some(previous_hash) = event.previous_hash.as_ref() else {
                // parse_event only permits this for sequence zero.
                result.accepted_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
                continue;
            };
            let Some(parent_ids) = hash_to_ids.get(previous_hash) else {
                result.missing_parent_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
                continue;
            };
            if parent_ids.len() != 1 {
                result.blocked_parent_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
                continue;
            }
            let parent_id = parent_ids.iter().next().expect("one parent id");
            let Some(parent) = eligible.get(parent_id) else {
                result.blocked_parent_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
                continue;
            };
            if parent.stream_id != event.stream_id || parent.sequence.checked_add(1) != Some(event.sequence) {
                result.blocked_parent_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
                continue;
            }
            if result.accepted_events.contains(parent_id) {
                result.accepted_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }

    // Any residual event has a parent that is present but cannot be accepted,
    // or participates in an invalid chain. A valid Mycelix chain is ordered by
    // stream-local sequence, so cycles are structurally invalid rather than a
    // legitimate merge conflict.
    result.blocked_parent_events.extend(unresolved);

    // Different streams can describe the same territorial projection. They
    // remain accepted as evidence but are emitted as an unresolved conflict.
    let accepted: Vec<String> = result.accepted_events.iter().cloned().collect();
    for left_index in 0..accepted.len() {
        for right_index in (left_index + 1)..accepted.len() {
            let left = &accepted[left_index];
            let right = &accepted[right_index];
            let (Some(left_event), Some(right_event)) = (eligible.get(left), eligible.get(right)) else {
                continue;
            };
            if left_event.projection_key == right_event.projection_key
                && left_event.stream_id != right_event.stream_id
            {
                result.conflict_pairs.insert((left.clone(), right.clone()));
            }
        }
    }

    result
}

impl Evaluation {
    pub fn to_json(&self) -> Value {
        let pairs: Vec<Value> = self.conflict_pairs.iter()
            .map(|(left, right)| json!([left, right]))
            .collect();
        json!({
            "accepted_events": self.accepted_events.iter().cloned().collect::<Vec<_>>(),
            "duplicate_replays": self.duplicate_replays,
            "invalid_signatures": self.invalid_signatures.iter().cloned().collect::<Vec<_>>(),
            "revoked_keys": self.revoked_keys.iter().cloned().collect::<Vec<_>>(),
            "verification_pending": self.verification_pending.iter().cloned().collect::<Vec<_>>(),
            "malformed_events": self.malformed_events.iter().cloned().collect::<Vec<_>>(),
            "missing_parent_events": self.missing_parent_events.iter().cloned().collect::<Vec<_>>(),
            "blocked_parent_events": self.blocked_parent_events.iter().cloned().collect::<Vec<_>>(),
            "equivocation_event_ids": self.equivocation_event_ids.iter().cloned().collect::<Vec<_>>(),
            "sequence_fork_event_ids": self.sequence_fork_event_ids.iter().cloned().collect::<Vec<_>>(),
            "hash_collision_event_ids": self.hash_collision_event_ids.iter().cloned().collect::<Vec<_>>(),
            "conflict_pairs": pairs
        })
    }
}
