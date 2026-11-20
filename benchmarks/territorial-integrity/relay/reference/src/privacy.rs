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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryEvaluation {
    pub disposition: &'static str,
    pub reason_codes: BTreeSet<String>,
    pub declared_channel_refs: BTreeSet<String>,
}

/// Check the inventory record's declared structure and unique channel IDs.
///
/// "declared_surface_bound" means only that the record names an independent
/// coverage-evidence reference and has no structural omissions. It does not
/// prove the evidence is authentic or that the observed boundary is complete.
pub fn evaluate_channel_inventory(input: &Value) -> InventoryEvaluation {
    let mut reasons = BTreeSet::new();
    let mut refs = BTreeSet::new();

    if input.get("schema").and_then(Value::as_str)
        != Some("luminous.cter.privacy-channel-inventory.v0.1")
    {
        reasons.insert("inventory_schema_missing_or_unsupported".to_owned());
    }
    for field in [
        "inventory_id",
        "deployment_ref",
        "instrumentation_profile_ref",
        "prepared_at",
    ] {
        if input
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            reasons.insert(format!("inventory_{field}_missing"));
        }
    }
    let build = input.get("build_commit").and_then(Value::as_str).unwrap_or("");
    if build.len() != 40
        || !build
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        reasons.insert("inventory_build_commit_invalid".to_owned());
    }

    match input.get("inventory_method").and_then(Value::as_str) {
        Some(
            "static_analysis"
                | "runtime_observation"
                | "host_telemetry"
                | "independent_red_team"
                | "combined",
        ) => {}
        _ => {
            reasons.insert("inventory_method_invalid".to_owned());
        }
    }

    let boundary = input.get("observation_boundary");
    for field in [
        "audiences",
        "time_window_start",
        "time_window_end",
        "external_systems_included",
        "explicit_exclusions",
    ] {
        if boundary
            .and_then(|value| value.get(field))
            .is_none()
        {
            reasons.insert(format!("observation_boundary_{field}_missing"));
        }
    }

    let Some(channels) = input.get("channels").and_then(Value::as_array) else {
        reasons.insert("inventory_channels_missing".to_owned());
        return InventoryEvaluation {
            disposition: "invalid",
            reason_codes: reasons,
            declared_channel_refs: refs,
        };
    };
    if channels.is_empty() {
        reasons.insert("inventory_channels_empty".to_owned());
    }

    for channel in channels {
        let Some(channel_ref) = channel
            .get("channel_ref")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
        else {
            reasons.insert("channel_ref_missing".to_owned());
            continue;
        };
        if !refs.insert(channel_ref.to_owned()) {
            reasons.insert("channel_ref_duplicated".to_owned());
        }
        for field in [
            "channel_kind",
            "audience",
            "purpose_ref",
            "owner_ref",
            "policy_ref",
            "retention",
            "observable_surface",
        ] {
            if channel.get(field).is_none() {
                reasons.insert(format!("channel_{field}_missing"));
            }
        }
        if channel
            .get("declared")
            .and_then(Value::as_bool)
            != Some(true)
        {
            reasons.insert("channel_not_explicitly_declared".to_owned());
        }
        if channel
            .get("data_classes")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            reasons.insert("channel_data_classes_missing".to_owned());
        }
    }

    let coverage = input
        .get("coverage_assessment")
        .and_then(Value::as_str);
    let disposition = match coverage {
        Some("complete_within_declared_boundary")
            if input
                .get("independent_coverage_evidence_ref")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
                && reasons.is_empty() =>
        {
            "declared_surface_bound"
        }
        Some("partial" | "unknown") => {
            reasons.insert("inventory_coverage_not_complete".to_owned());
            "inventory_incomplete"
        }
        Some("complete_within_declared_boundary") => {
            reasons.insert("independent_coverage_evidence_missing_or_structure_invalid".to_owned());
            "inventory_incomplete"
        }
        _ => {
            reasons.insert("inventory_coverage_assessment_invalid".to_owned());
            "invalid"
        }
    };

    if disposition == "declared_surface_bound" {
        InventoryEvaluation {
            disposition,
            reason_codes: reasons,
            declared_channel_refs: refs,
        }
    } else {
        let disposition = if reasons.contains("inventory_schema_missing_or_unsupported")
            || reasons.contains("inventory_channels_missing")
            || reasons.contains("inventory_coverage_assessment_invalid")
        {
            "invalid"
        } else {
            "inventory_incomplete"
        };
        InventoryEvaluation {
            disposition,
            reason_codes: reasons,
            declared_channel_refs: refs,
        }
    }
}
