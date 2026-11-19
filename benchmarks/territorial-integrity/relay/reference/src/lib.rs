//! Synthetic reference model for CTER relay event convergence.
//!
//! This crate classifies event envelopes after an independent verifier has
//! supplied a verification disposition. It does NOT implement cryptography,
//! canonicalization, key management, transport, authorization or legal review.
//!
//! "accepted_events" means eligible for evidence preservation under the
//! supplied dispositions only. It is not a truth, title or legal-effect result.

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
    pub causal_cycle_events: BTreeSet<String>,
    pub conflict_pairs: BTreeSet<(String, String)>,
}

#[derive(Debug, Clone)]
struct ParsedEvent {
    id: String,
    origin: String,
    sequence: u64,
    parents: Vec<String>,
    projection_key: String,
    envelope_sha256: String,
    verification: String,
}

fn nonempty_string(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().filter(|s| !s.trim().is_empty()).map(str::to_owned)
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn parse_event(value: &Value) -> Option<ParsedEvent> {
    let id = nonempty_string(value, "id")?;
    let origin = nonempty_string(value, "origin")?;
    let sequence = value.get("sequence")?.as_u64()?;
    let projection_key = nonempty_string(value, "projection_key")?;
    let envelope_sha256 = nonempty_string(value, "envelope_sha256")?;
    if !is_lower_hex_sha256(&envelope_sha256) {
        return None;
    }
    let verification = nonempty_string(value, "verification")?;
    if !matches!(verification.as_str(), "verified" | "invalid" | "revoked" | "unavailable") {
        return None;
    }
    let visibility = nonempty_string(value, "visibility")?;
    if !matches!(
        visibility.as_str(),
        "public" | "community" | "restricted" | "culturally_protected"
    ) {
        return None;
    }
    let parents_value = value.get("parents")?.as_array()?;
    let mut parents = Vec::with_capacity(parents_value.len());
    for parent in parents_value {
        let parent_id = parent.as_str()?.trim();
        if parent_id.is_empty() {
            return None;
        }
        parents.push(parent_id.to_owned());
    }
    Some(ParsedEvent {
        id,
        origin,
        sequence,
        parents,
        projection_key,
        envelope_sha256,
        verification,
    })
}

fn mark_id(value: &Value, index: usize) -> String {
    value.get("id").and_then(Value::as_str).filter(|s| !s.is_empty())
        .map(str::to_owned).unwrap_or_else(|| format!("#event-{index}"))
}

/// Deterministic set-union merge. Distinct contents under the same event ID are
/// retained as separate variants so evaluate() can mark equivocation.
pub fn merge_event_sets(left: &[Value], right: &[Value]) -> Vec<Value> {
    let mut merged: BTreeMap<(String, String, String), Value> = BTreeMap::new();
    for event in left.iter().chain(right.iter()) {
        let id = event.get("id").and_then(Value::as_str).unwrap_or("");
        let digest = event.get("envelope_sha256").and_then(Value::as_str).unwrap_or("");
        // The serialized value is a final tie-breaker, not a cryptographic hash.
        let serialized = event.to_string();
        merged.insert((id.to_owned(), digest.to_owned(), serialized), event.clone());
    }
    merged.into_values().collect()
}

fn is_ancestor(ancestor: &str, descendant: &str, events: &BTreeMap<String, ParsedEvent>) -> bool {
    let mut stack = vec![descendant.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if let Some(event) = events.get(&current) {
            for parent in &event.parents {
                if parent == ancestor {
                    return true;
                }
                stack.push(parent.clone());
            }
        }
    }
    false
}

fn visit_cycle(
    node: &str,
    unresolved: &BTreeSet<String>,
    events: &BTreeMap<String, ParsedEvent>,
    state: &mut BTreeMap<String, u8>,
    stack: &mut Vec<String>,
    cycle_nodes: &mut BTreeSet<String>,
) {
    state.insert(node.to_owned(), 1);
    stack.push(node.to_owned());
    if let Some(event) = events.get(node) {
        for parent in &event.parents {
            if !unresolved.contains(parent) {
                continue;
            }
            match state.get(parent).copied().unwrap_or(0) {
                0 => visit_cycle(parent, unresolved, events, state, stack, cycle_nodes),
                1 => {
                    if let Some(position) = stack.iter().position(|item| item == parent) {
                        cycle_nodes.extend(stack[position..].iter().cloned());
                    }
                }
                _ => {}
            }
        }
    }
    stack.pop();
    state.insert(node.to_owned(), 2);
}

/// Evaluate a synthetic batch of event envelopes.
///
/// The caller supplies signature/key verification outcomes. This function
/// never upgrades "unavailable" to "verified" and never uses wall-clock time
/// to order events.
pub fn evaluate(input: &[Value]) -> Evaluation {
    let mut result = Evaluation::default();
    let mut variants: BTreeMap<String, BTreeMap<String, (String, ParsedEvent)>> = BTreeMap::new();
    let mut present_ids = BTreeSet::new();

    for (index, value) in input.iter().enumerate() {
        let id_hint = mark_id(value, index);
        match parse_event(value) {
            Some(event) => {
                present_ids.insert(event.id.clone());
                let canonical_text = value.to_string();
                let by_digest = variants.entry(event.id.clone()).or_default();
                if let Some((existing_text, _)) = by_digest.get(&event.envelope_sha256) {
                    if existing_text == &canonical_text {
                        result.duplicate_replays += 1;
                    } else {
                        // Same ID and digest but different canonical content is also equivocation.
                        by_digest.insert(
                            format!("{}:variant:{}", event.envelope_sha256, canonical_text),
                            (canonical_text, event),
                        );
                    }
                } else {
                    by_digest.insert(event.envelope_sha256.clone(), (canonical_text, event));
                }
            }
            None => {
                result.malformed_events.insert(id_hint);
                if let Some(id) = value.get("id").and_then(Value::as_str) {
                    if !id.is_empty() {
                        present_ids.insert(id.to_owned());
                    }
                }
            }
        }
    }

    let mut unique: BTreeMap<String, ParsedEvent> = BTreeMap::new();
    for (id, by_digest) in variants {
        if by_digest.len() != 1 {
            result.equivocation_event_ids.insert(id);
            continue;
        }
        if let Some((_, event)) = by_digest.into_values().next() {
            unique.insert(id, event);
        }
    }

    // Only verifier-accepted events can assert an origin sequence. A forged
    // or unverifiable envelope must not poison another origin's sequence.
    let mut sequence_groups: BTreeMap<(String, u64), BTreeSet<String>> = BTreeMap::new();
    for event in unique.values() {
        if event.verification == "verified" {
            sequence_groups.entry((event.origin.clone(), event.sequence))
                .or_default().insert(event.id.clone());
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
            "verified" if !result.sequence_fork_event_ids.contains(id) => {
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
            let mut missing = false;
            let mut blocked = false;
            let mut all_parents_accepted = true;
            for parent in &event.parents {
                if !present_ids.contains(parent) {
                    missing = true;
                    all_parents_accepted = false;
                    break;
                }
                if result.accepted_events.contains(parent) {
                    continue;
                }
                if !eligible.contains_key(parent) {
                    blocked = true;
                    all_parents_accepted = false;
                    break;
                }
                all_parents_accepted = false;
            }
            if missing {
                result.missing_parent_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
            } else if blocked {
                result.blocked_parent_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
            } else if all_parents_accepted {
                result.accepted_events.insert(id.clone());
                unresolved.remove(&id);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }

    // Remaining nodes are either in a causal cycle or depend on one.
    let mut state: BTreeMap<String, u8> = BTreeMap::new();
    let mut stack = Vec::new();
    let mut cycle_nodes = BTreeSet::new();
    for id in &unresolved {
        if state.get(id).copied().unwrap_or(0) == 0 {
            visit_cycle(id, &unresolved, &eligible, &mut state, &mut stack, &mut cycle_nodes);
        }
    }
    for id in unresolved {
        if cycle_nodes.contains(&id) {
            result.causal_cycle_events.insert(id);
        } else {
            result.blocked_parent_events.insert(id);
        }
    }

    // Concurrent verified events that project onto the same key remain
    // accepted as evidence but require an explicit conflict review.
    let accepted: Vec<String> = result.accepted_events.iter().cloned().collect();
    for left_index in 0..accepted.len() {
        for right_index in (left_index + 1)..accepted.len() {
            let left = &accepted[left_index];
            let right = &accepted[right_index];
            let (Some(left_event), Some(right_event)) = (eligible.get(left), eligible.get(right)) else {
                continue;
            };
            if left_event.projection_key != right_event.projection_key {
                continue;
            }
            if !is_ancestor(left, right, &eligible) && !is_ancestor(right, left, &eligible) {
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
            "causal_cycle_events": self.causal_cycle_events.iter().cloned().collect::<Vec<_>>(),
            "conflict_pairs": pairs
        })
    }
}
