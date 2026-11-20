//! Synthetic paired-world privacy fixture evaluator.
//!
//! This is a corpus semantics checker, not a production information-flow
//! analyzer. It compares the observable JSON transcript declared for two
//! synthetic worlds and scans for explicit synthetic canary strings.

use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyEvaluation {
    pub disposition: &'static str,
    pub different_channels: BTreeSet<String>,
    pub undeclared_channels: BTreeSet<String>,
    pub canary_hit_count: usize,
}

fn collect_canary_hits(value: &Value, canaries: &BTreeSet<String>, count: &mut usize) {
    match value {
        Value::String(text) => {
            for canary in canaries {
                if text.contains(canary) {
                    *count += 1;
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_canary_hits(child, canaries, count);
            }
        }
        Value::Object(object) => {
            for child in object.values() {
                collect_canary_hits(child, canaries, count);
            }
        }
        _ => {}
    }
}

/// Classify one paired-world synthetic fixture using exact observable equality.
///
/// Unknown channels take precedence over the leak/non-disclosure disposition:
/// without an inventory entry, the test boundary is incomplete.
pub fn classify_paired_world_case(
    case: &Value,
    declared_channels: &BTreeSet<String>,
) -> PrivacyEvaluation {
    let empty = Value::Object(Default::default());
    let world_a = case
        .get("world_a")
        .and_then(|value| value.get("observables"))
        .unwrap_or(&empty);
    let world_b = case
        .get("world_b")
        .and_then(|value| value.get("observables"))
        .unwrap_or(&empty);

    let mut observed_channels = BTreeSet::new();
    if let Some(object) = world_a.as_object() {
        observed_channels.extend(object.keys().cloned());
    }
    if let Some(object) = world_b.as_object() {
        observed_channels.extend(object.keys().cloned());
    }

    let undeclared_channels: BTreeSet<String> = observed_channels
        .difference(declared_channels)
        .cloned()
        .collect();

    if !undeclared_channels.is_empty() {
        return PrivacyEvaluation {
            disposition: "inventory_incomplete",
            different_channels: BTreeSet::new(),
            undeclared_channels,
            canary_hit_count: 0,
        };
    }

    let canaries: BTreeSet<String> = case
        .get("canaries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .collect();

    let mut canary_hit_count = 0;
    collect_canary_hits(world_a, &canaries, &mut canary_hit_count);
    collect_canary_hits(world_b, &canaries, &mut canary_hit_count);

    let different_channels: BTreeSet<String> = observed_channels
        .iter()
        .filter(|channel| world_a.get(channel.as_str()) != world_b.get(channel.as_str()))
        .cloned()
        .collect();

    let disposition = if canary_hit_count > 0 || !different_channels.is_empty() {
        "leak_detected"
    } else {
        "non_disclosure"
    };

    PrivacyEvaluation {
        disposition,
        different_channels,
        undeclared_channels,
        canary_hit_count,
    }
}
