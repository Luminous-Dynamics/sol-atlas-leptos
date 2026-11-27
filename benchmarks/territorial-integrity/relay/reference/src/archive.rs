//! Structural reference evaluator for CTER community evidence archives.
//!
//! This evaluator checks manifest consistency and conservative disposition
//! rules over synthetic records. It does not read or restore payload bytes,
//! resolve authorization/consent, verify signatures or timestamps, decrypt
//! archives, or determine legal effect.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEvaluation {
    pub disposition: &'static str,
    pub reasons: Vec<&'static str>,
}

fn nonempty<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)?
        .as_str()
        .filter(|text| !text.trim().is_empty())
}

fn valid_digest(value: &Value) -> bool {
    let Some(algorithm) = nonempty(value, "algorithm") else {
        return false;
    };
    if !matches!(algorithm, "SHA-256" | "BLAKE3") {
        return false;
    }
    let Some(hex) = nonempty(value, "value") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_access_class(class: &str) -> bool {
    matches!(
        class,
        "public" | "community" | "restricted" | "culturally_protected"
    )
}

fn evaluation(disposition: &'static str, reason: &'static str) -> ArchiveEvaluation {
    ArchiveEvaluation {
        disposition,
        reasons: vec![reason],
    }
}

/// Classify a manifest without claiming that referenced evidence or
/// authorization is authentic. The evaluator is a structural policy reference,
/// not an exporter, cryptographic verifier or restore implementation.
pub fn evaluate_archive(manifest: &Value) -> ArchiveEvaluation {
    if nonempty(manifest, "schema") != Some("luminous.cter.community-archive.v0")
        || nonempty(manifest, "claim_ceiling")
            != Some("archive-structure-and-restoration-only-no-legal-effect")
        || nonempty(manifest, "archive_id").is_none()
        || nonempty(manifest, "profile_version") != Some("0.1")
    {
        return evaluation("invalid", "archive_identity_or_claim_ceiling_invalid");
    }

    let Some(export_context) = manifest.get("export_context") else {
        return evaluation("invalid", "export_context_missing");
    };
    if nonempty(export_context, "destination_ref").is_none()
        || nonempty(export_context, "authorization_ref").is_none()
        || nonempty(export_context, "purpose_ref").is_none()
    {
        return evaluation(
            "invalid",
            "export_destination_authorization_or_purpose_reference_missing",
        );
    }
    let Some(raw_authorized_classes) = export_context
        .get("authorized_access_classes")
        .and_then(Value::as_array)
    else {
        return evaluation("invalid", "authorized_access_classes_missing");
    };
    if raw_authorized_classes.is_empty() {
        return evaluation("invalid", "authorized_access_classes_empty");
    }
    let mut authorized_classes = BTreeSet::new();
    for value in raw_authorized_classes {
        let Some(class) = value.as_str() else {
            return evaluation("invalid", "authorized_access_class_not_a_string");
        };
        if !valid_access_class(class) || !authorized_classes.insert(class.to_owned()) {
            return evaluation("invalid", "authorized_access_class_unknown_or_duplicated");
        }
    }

    let Some(objects) = manifest.get("objects").and_then(Value::as_array) else {
        return evaluation("invalid", "objects_not_an_array");
    };
    let Some(frontiers) = manifest.get("source_frontiers").and_then(Value::as_array) else {
        return evaluation("invalid", "source_frontiers_not_an_array");
    };
    if frontiers.is_empty() {
        return evaluation("incomplete", "no_declared_source_frontier");
    }

    let mut object_refs = BTreeSet::new();
    let mut object_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut has_withheld = false;
    let mut license_review = false;
    let mut disclosure_violation = false;

    for object in objects {
        let Some(object_ref) = nonempty(object, "object_ref") else {
            return evaluation("invalid", "object_reference_missing");
        };
        if !object_refs.insert(object_ref.to_owned()) {
            return evaluation("invalid", "duplicate_object_reference");
        }
        let Some(owner) = nonempty(object, "owner_system") else {
            return evaluation("invalid", "object_owner_missing");
        };
        *object_counts.entry(owner.to_owned()).or_default() += 1;

        let Some(access_class) = nonempty(object, "access_class") else {
            return evaluation("invalid", "object_access_class_missing");
        };
        if !valid_access_class(access_class) {
            return evaluation("invalid", "object_access_class_unknown");
        }
        let Some(disposition) = nonempty(object, "disposition") else {
            return evaluation("invalid", "object_disposition_missing");
        };

        match disposition {
            "included" => {
                // Access classes are not a universal hierarchy. The exact
                // class must be named by the destination's explicit authorization.
                if !authorized_classes.contains(access_class) {
                    disclosure_violation = true;
                }
                if !valid_digest(&object["digest"]) {
                    return evaluation("invalid", "included_object_digest_missing_or_invalid");
                }
                if nonempty(object, "payload_ref").is_none() {
                    return evaluation("invalid", "included_object_payload_reference_missing");
                }
                match nonempty(object, "license_state") {
                    Some("redistributable" | "reference_only") => {}
                    Some("not_cleared" | "unknown") | None => license_review = true,
                    _ => return evaluation("invalid", "object_license_state_unknown"),
                }
            }
            "withheld" | "unavailable" => {
                has_withheld = true;
                if nonempty(object, "reason_code").is_none() {
                    return evaluation("invalid", "withheld_object_reason_missing");
                }
                // A non-disclosing commitment may be exported, but a payload
                // locator for a withheld object would expose its retrieval path.
                if object.get("payload_ref").is_some() {
                    disclosure_violation = true;
                }
            }
            _ => return evaluation("invalid", "object_disposition_unknown"),
        }
    }

    let mut frontier_owners = BTreeSet::new();
    let mut incomplete = false;
    for frontier in frontiers {
        let Some(owner) = nonempty(frontier, "owner_system") else {
            return evaluation("invalid", "frontier_owner_missing");
        };
        if !frontier_owners.insert(owner.to_owned()) {
            return evaluation("invalid", "duplicate_source_frontier_owner");
        }
        if nonempty(frontier, "snapshot_ref").is_none()
            || nonempty(frontier, "query_or_scope_ref").is_none()
            || !valid_digest(&frontier["snapshot_digest"])
        {
            return evaluation("invalid", "frontier_snapshot_identity_invalid");
        }
        let Some(observed) = frontier.get("observed_count").and_then(Value::as_u64) else {
            return evaluation("invalid", "frontier_observed_count_missing");
        };
        let Some(expected) = frontier.get("expected_count").and_then(Value::as_u64) else {
            incomplete = true;
            continue;
        };
        if expected != observed || object_counts.get(owner).copied().unwrap_or(0) != observed {
            incomplete = true;
        }
        let complete =
            nonempty(frontier, "completeness") == Some("complete_within_declared_frontier");
        let gaps_empty = frontier
            .get("gap_refs")
            .and_then(Value::as_array)
            .is_some_and(|gaps| gaps.is_empty());
        if !complete || !gaps_empty {
            incomplete = true;
        }
    }
    if object_counts
        .keys()
        .any(|owner| !frontier_owners.contains(owner))
    {
        incomplete = true;
    }

    if let Some(transport) = manifest.get("transport_observation") {
        let carrier = nonempty(transport, "carrier_bundle_state");
        let durable = nonempty(transport, "durable_evidence_state");
        if carrier == Some("expired") && durable != Some("retained") {
            incomplete = true;
        }
        if durable == Some("retained") && nonempty(transport, "retained_event_ref").is_none() {
            return evaluation("invalid", "retained_transport_event_reference_missing");
        }
    }

    if disclosure_violation {
        return evaluation(
            "disclosure_violation",
            "export_exceeds_declared_destination_clearance",
        );
    }
    if incomplete {
        return evaluation(
            "incomplete",
            "declared_frontier_or_object_accounting_is_incomplete",
        );
    }

    let Some(restore) = manifest.get("restore") else {
        return evaluation("invalid", "restore_section_missing");
    };
    let custody = nonempty(restore, "key_custody");
    let provider_independent = restore
        .get("provider_independent_restore")
        .and_then(Value::as_bool)
        == Some(true);
    let restore_passed = nonempty(restore, "restore_test_status") == Some("pass")
        && nonempty(restore, "restore_instructions_ref").is_some()
        && nonempty(restore, "restore_run_ref").is_some();
    if !provider_independent
        || !matches!(
            custody,
            Some("community_threshold" | "participant_controlled" | "independent_trustee")
        )
        || !restore_passed
    {
        return evaluation(
            "not_independently_restorable",
            "provider_independent_custody_or_restore_test_not_qualified",
        );
    }

    let Some(preservation) = manifest.get("preservation").and_then(Value::as_array) else {
        return evaluation("invalid", "preservation_not_an_array");
    };
    if preservation.is_empty() {
        return evaluation("review_required", "no_preservation_record_declared");
    }
    let mut renewed_chain = false;
    let mut historical_indeterminate = false;
    for record in preservation {
        let algorithm_state = nonempty(record, "algorithm_state");
        let validation_state = nonempty(record, "validation_state");
        if validation_state == Some("failed") {
            historical_indeterminate = true;
            continue;
        }
        if matches!(
            algorithm_state,
            Some("compromised" | "deprecated" | "unknown")
        ) {
            let renewal_ref = nonempty(record, "renewal_event_ref");
            let predecessor = nonempty(record, "predecessor_ref");
            let timestamp = nonempty(record, "timestamp_evidence_ref");
            let renewal_time = record
                .get("renewal_time_unix_seconds")
                .and_then(Value::as_i64);
            let weakening_time = record
                .get("algorithm_weakening_time_unix_seconds")
                .and_then(Value::as_i64);
            let renewal_is_timely = match (renewal_time, weakening_time) {
                (Some(renewal), Some(weakening)) => renewal <= weakening,
                _ => false,
            };
            if validation_state == Some("valid")
                && renewal_ref.is_some()
                && predecessor.is_some()
                && timestamp.is_some()
                && renewal_is_timely
            {
                renewed_chain = true;
            } else {
                historical_indeterminate = true;
            }
        } else if validation_state != Some("valid")
            || nonempty(record, "timestamp_evidence_ref").is_none()
        {
            historical_indeterminate = true;
        } else if nonempty(record, "renewal_event_ref").is_some() {
            renewed_chain = true;
        }
    }
    if historical_indeterminate {
        return evaluation(
            "indeterminate_historical_validation",
            "preservation_chain_cannot_establish_historical_validation",
        );
    }
    if license_review {
        return evaluation(
            "review_required",
            "license_or_redistribution_rights_unresolved",
        );
    }
    if has_withheld {
        return evaluation(
            "eligible_with_declared_exclusions",
            "withheld_objects_accounted_for_without_payload_disclosure",
        );
    }
    if renewed_chain {
        return evaluation(
            "eligible_with_preservation_chain",
            "predecessor_bound_preservation_renewal_declared",
        );
    }
    evaluation(
        "eligible_for_independent_restore",
        "frontiers_and_restore_controls_structurally_consistent",
    )
}
