//! Reference composition evaluator for the city restoration outcome bridge.
//!
//! This crate consumes owner-evaluated synthetic reports. It never recomputes
//! feasibility, continuity, in-use outcomes, physical acceptance, or service
//! delivery. Its claim ceiling is synthetic composition mechanics only.

use std::collections::{BTreeSet, HashSet};

use serde_json::{Value, json};

pub const CORPUS_ID: &str = "CITY-RESTORATION-OUTCOME-BRIDGE-CORPUS-V0.1";
pub const CLAIM_CEILING: &str = "synthetic_service_restoration_composition_only";

const ROOT_KEYS: &[&str] = &[
    "corpus_id",
    "schema_version",
    "classification",
    "policy",
    "claim_ceiling",
    "source_refs",
    "cases",
];
const CASE_KEYS: &[&str] = &["case_id", "input", "expected"];
const INPUT_KEYS: &[&str] = &[
    "case_id",
    "classification",
    "service_contract",
    "requested_scope",
    "scope_binding",
    "interruption",
    "intervention",
    "restoration",
    "continuity",
    "measurement",
    "physical_output",
    "mandatory_services",
    "mandatory_hard_floors",
    "resident_experience",
    "temporal_cuts",
];
const REF_KEYS: &[&str] = &[
    "owner",
    "contract_id",
    "contract_revision",
    "receipt_id",
    "source_commit",
    "source_case_id",
    "content_sha256",
    "subject_id",
    "subject_digest",
    "source_scope_digest",
    "functional_unit",
    "group_id",
    "scenario_id",
    "valid_from_tick",
    "valid_until_tick",
    "recorded_tick",
    "registered_tick",
    "ingested_tick",
    "evidence_state",
    "rights_state",
    "issuer_state",
];
const EXPECTED_KEYS: &[&str] = &[
    "case_id",
    "bridge_disposition",
    "historical_disposition",
    "plan_readiness",
    "work_execution",
    "inspection_state",
    "commissioning_state",
    "continuity_state",
    "measured_outcome_state",
    "disclosure_disposition",
    "restoration_qualified_for_scope",
    "recovery_trajectory",
    "reason_codes",
    "claim_ceiling",
    "action_authority",
    "causal_effect_established",
];

fn skip_json_whitespace(bytes: &[u8], cursor: &mut usize) {
    while bytes
        .get(*cursor)
        .is_some_and(|byte| matches!(*byte, 32 | 10 | 13 | 9))
    {
        *cursor += 1;
    }
}

fn scan_json_string_token<'a>(bytes: &'a [u8], cursor: &mut usize) -> Result<&'a str, String> {
    let start = *cursor;
    if bytes.get(*cursor).copied() != Some(34) {
        return Err("JSON_STRING_EXPECTED".to_owned());
    }
    *cursor += 1;
    while let Some(byte) = bytes.get(*cursor).copied() {
        match byte {
            34 => {
                *cursor += 1;
                return std::str::from_utf8(&bytes[start..*cursor])
                    .map_err(|_| "JSON_NOT_UTF8".to_owned());
            }
            92 => {
                *cursor += 1;
                if *cursor >= bytes.len() {
                    return Err("JSON_STRING_ESCAPE_TRUNCATED".to_owned());
                }
                *cursor += 1;
            }
            _ => *cursor += 1,
        }
    }
    Err("JSON_STRING_UNTERMINATED".to_owned())
}

/// Scan original bytes before serde_json creates maps; compare decoded key spellings.
fn scan_json_value(bytes: &[u8], cursor: &mut usize) -> Result<(), String> {
    skip_json_whitespace(bytes, cursor);
    match bytes.get(*cursor).copied() {
        Some(123) => {
            *cursor += 1;
            skip_json_whitespace(bytes, cursor);
            if bytes.get(*cursor).copied() == Some(125) {
                *cursor += 1;
                return Ok(());
            }
            let mut keys = HashSet::new();
            loop {
                skip_json_whitespace(bytes, cursor);
                let token = scan_json_string_token(bytes, cursor)?;
                let key: String = serde_json::from_str(token)
                    .map_err(|_| "JSON_OBJECT_KEY_INVALID".to_owned())?;
                if !keys.insert(key.clone()) {
                    return Err(format!("JSON_DUPLICATE_MEMBER_KEY:{key}"));
                }
                skip_json_whitespace(bytes, cursor);
                if bytes.get(*cursor).copied() != Some(58) {
                    return Err("JSON_OBJECT_COLON_EXPECTED".to_owned());
                }
                *cursor += 1;
                scan_json_value(bytes, cursor)?;
                skip_json_whitespace(bytes, cursor);
                match bytes.get(*cursor).copied() {
                    Some(44) => *cursor += 1,
                    Some(125) => {
                        *cursor += 1;
                        return Ok(());
                    }
                    _ => return Err("JSON_OBJECT_SEPARATOR_EXPECTED".to_owned()),
                }
            }
        }
        Some(91) => {
            *cursor += 1;
            skip_json_whitespace(bytes, cursor);
            if bytes.get(*cursor).copied() == Some(93) {
                *cursor += 1;
                return Ok(());
            }
            loop {
                scan_json_value(bytes, cursor)?;
                skip_json_whitespace(bytes, cursor);
                match bytes.get(*cursor).copied() {
                    Some(44) => *cursor += 1,
                    Some(93) => {
                        *cursor += 1;
                        return Ok(());
                    }
                    _ => return Err("JSON_ARRAY_SEPARATOR_EXPECTED".to_owned()),
                }
            }
        }
        Some(34) => {
            let token = scan_json_string_token(bytes, cursor)?;
            serde_json::from_str::<String>(token)
                .map(|_| ())
                .map_err(|_| "JSON_STRING_INVALID".to_owned())
        }
        Some(_) => {
            let start = *cursor;
            while bytes
                .get(*cursor)
                .is_some_and(|byte| !matches!(*byte, 32 | 10 | 13 | 9 | 44 | 93 | 125))
            {
                *cursor += 1;
            }
            if *cursor == start {
                Err("JSON_VALUE_EXPECTED".to_owned())
            } else {
                Ok(())
            }
        }
        None => Err("JSON_VALUE_EXPECTED".to_owned()),
    }
}

/// Reject duplicate decoded keys, malformed UTF-8, invalid JSON and trailing bytes.
pub fn parse_unique_json_bytes(bytes: &[u8]) -> Result<Value, String> {
    let input = std::str::from_utf8(bytes).map_err(|_| "JSON_NOT_UTF8".to_owned())?;
    let mut cursor = 0;
    scan_json_value(bytes, &mut cursor)?;
    skip_json_whitespace(bytes, &mut cursor);
    if cursor != bytes.len() {
        return Err("JSON_TRAILING_CONTENT".to_owned());
    }
    serde_json::from_str(input).map_err(|error| format!("JSON_PARSE_FAILED:{error}"))
}

fn profile_for<'a>(profiles: &'a Value, record: &Value) -> Option<&'a Value> {
    array(profiles, "profiles").iter().find(|profile| {
        string(profile, "owner") == string(record, "owner")
            && string(profile, "schema_id") == string(record, "schema_id")
    })
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn object<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}

fn list<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn tick(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn add(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn check_keys(value: &Value, path: &str, allowed: &[&str], errors: &mut Vec<String>) {
    let Some(map) = value.as_object() else {
        errors.push(format!("{path}:OBJECT_REQUIRED"));
        return;
    };
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            errors.push(format!("{path}:UNKNOWN_FIELD:{key}"));
        }
    }
}

fn ref_is_visible(source: &Value, cut: &Value) -> bool {
    let Some(valid_at) = tick(cut, "valid_tick") else {
        return false;
    };
    let Some(recorded_at) = tick(cut, "recorded_tick") else {
        return false;
    };
    let Some(registered_at) = tick(cut, "registered_tick") else {
        return false;
    };
    let Some(ingested_at) = tick(cut, "ingested_tick") else {
        return false;
    };
    let Some(valid_from) = tick(source, "valid_from_tick") else {
        return false;
    };
    let Some(recorded) = tick(source, "recorded_tick") else {
        return false;
    };
    let Some(registered) = tick(source, "registered_tick") else {
        return false;
    };
    let Some(ingested) = tick(source, "ingested_tick") else {
        return false;
    };
    let valid_until = tick(source, "valid_until_tick");
    valid_from <= valid_at
        && valid_until.is_none_or(|until| valid_at < until)
        && recorded <= recorded_at
        && registered <= registered_at
        && ingested <= ingested_at
}

fn source_binding_exists(binding: &Value, source: &Value, scope: &Value) -> bool {
    if source.is_null() {
        return false;
    }
    let source_matches_scope = text(source, "subject_id") == text(scope, "subject_id")
        && text(source, "subject_digest") == text(scope, "subject_digest")
        && text(source, "functional_unit") == text(scope, "functional_unit")
        && text(source, "group_id") == text(scope, "group_id")
        && text(source, "scenario_id") == text(scope, "scenario_id");
    if !source_matches_scope {
        return false;
    }
    list(binding, "source_bindings").iter().any(|candidate| {
        text(candidate, "owner") == text(source, "owner")
            && text(candidate, "contract_id") == text(source, "contract_id")
            && text(candidate, "contract_revision") == text(source, "contract_revision")
            && text(candidate, "receipt_id") == text(source, "receipt_id")
            && text(candidate, "source_scope_digest") == text(source, "source_scope_digest")
            && text(candidate, "subject_id") == text(scope, "subject_id")
            && text(candidate, "subject_digest") == text(scope, "subject_digest")
            && text(candidate, "functional_unit") == text(scope, "functional_unit")
            && text(candidate, "group_id") == text(scope, "group_id")
            && text(candidate, "scenario_id") == text(scope, "scenario_id")
            && text(candidate, "binding_state") == "Bound"
            && text(candidate, "common_scope_digest") == text(binding, "common_scope_digest")
    })
}

fn owner_receipts_bound(input: &Value, reasons: &mut BTreeSet<String>) {
    let binding = object(input, "scope_binding");
    let scope = object(input, "requested_scope");
    match text(binding, "binding_state") {
        "NotComparable" => {
            add(reasons, "COMMON_SCOPE_NOT_COMPARABLE");
            add(reasons, "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE");
        }
        "Unknown" => {
            add(reasons, "COMMON_SCOPE_BINDING_UNKNOWN");
            add(reasons, "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE");
        }
        "Bound" => {
            let contract = object(input, "service_contract");
            let restoration = object(input, "restoration");
            let interruption = object(input, "interruption");
            let continuity = object(input, "continuity");
            let measurement = object(input, "measurement");
            let mut refs = vec![
                object(contract, "source_ref"),
                object(restoration, "source_ref"),
                object(interruption, "source_ref"),
                object(continuity, "source_ref"),
                object(measurement, "source_ref"),
            ];
            let observation = object(measurement, "observation");
            if !observation.is_null() {
                refs.push(object(observation, "source_ref"));
            }
            let physical = object(input, "physical_output");
            if physical.get("required").and_then(Value::as_bool) == Some(true) {
                refs.push(object(physical, "source_ref"));
            }
            if refs
                .iter()
                .any(|source| !source_binding_exists(binding, source, scope))
            {
                add(reasons, "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE");
            }
        }
        _ => {
            add(reasons, "COMMON_SCOPE_BINDING_UNKNOWN");
            add(reasons, "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE");
        }
    }

    // Check the measurement's underlying observation separately from the
    // aggregate evaluator receipt. This also applies when common-scope
    // binding is Unknown, where we must report both failures.
    let measurement = object(input, "measurement");
    let observation = object(measurement, "observation");
    let observation_ref = object(observation, "source_ref");
    if !observation.is_null()
        && !observation_ref.is_null()
        && !source_binding_exists(binding, observation_ref, scope)
    {
        add(reasons, "MEASUREMENT_OWNER_RECEIPT_BINDING_INVALID");
    }
}

fn source_authority_valid(source: &Value) -> bool {
    !source.is_null()
        && text(source, "evidence_state") == "current"
        && text(source, "rights_state") == "permitted"
        && text(source, "issuer_state") == "authorized"
}

fn measured_outcome(input: &Value) -> String {
    let contract = object(input, "service_contract");
    if contract
        .get("measurement_required")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return "NotRequired".to_owned();
    }
    let measurement = object(input, "measurement");
    if text(measurement, "evaluation_state") != "Evaluated" {
        return "Unknown".to_owned();
    }
    if list(measurement, "metric_results").iter().any(|metric| {
        text(metric, "threshold_state") != "Pass" || text(metric, "observation_state") != "Current"
    }) {
        return "Unknown".to_owned();
    }
    "Pass".to_owned()
}

fn recovery_trajectory(input: &Value) -> Value {
    let interruption_start = tick(object(input, "interruption"), "start_tick");
    let continuity = object(input, "continuity");
    let trajectory = object(continuity, "trajectory");
    let stability = object(continuity, "stable_adequacy");
    let first_adequate = tick(trajectory, "first_adequate_tick");
    let stable_start = tick(stability, "start_tick");
    let stable_confirmed = tick(stability, "confirmed_tick");
    let delta = |end: Option<u64>, start: Option<u64>| {
        end.zip(start)
            .and_then(|(end, start)| end.checked_sub(start))
    };
    json!({
        "interruption_start_tick": interruption_start,
        "first_adequate_tick": first_adequate,
        "time_to_first_adequate_ticks": delta(first_adequate, interruption_start),
        "stable_adequacy_start_tick": stable_start,
        "stable_adequacy_confirmed_tick": stable_confirmed,
        "time_to_sustained_adequate_ticks": delta(stable_confirmed, interruption_start),
        "adequate_ticks": tick(trajectory, "adequate_ticks"),
        "constrained_ticks": tick(trajectory, "constrained_ticks"),
        "inaccessible_ticks": tick(trajectory, "inaccessible_ticks"),
        "unknown_ticks": tick(trajectory, "unknown_ticks"),
        "recurrence_count": tick(trajectory, "recurrence_count"),
        "demand_units_unmet": null,
        "denominator_state": "not_established"
    })
}

fn evaluate_current(input: &Value) -> (String, BTreeSet<String>) {
    let mut reasons = BTreeSet::new();
    owner_receipts_bound(input, &mut reasons);

    let temporal_cuts = object(input, "temporal_cuts");
    let current_cut = object(temporal_cuts, "current");
    let contract_ref = object(object(input, "service_contract"), "source_ref");
    let restoration_ref = object(object(input, "restoration"), "source_ref");
    let interruption_ref = object(object(input, "interruption"), "source_ref");
    let continuity_ref = object(object(input, "continuity"), "source_ref");
    let measurement = object(input, "measurement");
    let measurement_ref = object(measurement, "source_ref");
    let observation_ref = object(object(measurement, "observation"), "source_ref");
    let physical_ref = object(object(input, "physical_output"), "source_ref");
    let all_receipts = [
        contract_ref,
        restoration_ref,
        interruption_ref,
        continuity_ref,
        measurement_ref,
        observation_ref,
        physical_ref,
    ];
    if all_receipts
        .iter()
        .filter(|source| !source.is_null())
        .any(|source| !ref_is_visible(source, current_cut))
    {
        add(&mut reasons, "OWNER_RECEIPT_NOT_VISIBLE_AT_CURRENT_CUT");
    }

    let scope = object(input, "requested_scope");
    let contract = object(input, "service_contract");
    let restoration = object(input, "restoration");
    let continuity = object(input, "continuity");
    let measurement = object(input, "measurement");
    let intervention = object(input, "intervention");
    let physical = object(input, "physical_output");

    if text(contract, "functional_unit") != text(scope, "functional_unit") {
        add(&mut reasons, "FUNCTIONAL_UNIT_NOT_COMPARABLE");
        add(&mut reasons, "COMMON_SCOPE_NOT_COMPARABLE");
    }

    let mut authority_refs = vec![
        object(contract, "source_ref"),
        object(restoration, "source_ref"),
        object(interruption(input), "source_ref"),
        object(continuity, "source_ref"),
        object(measurement, "source_ref"),
        object(object(measurement, "observation"), "source_ref"),
    ];
    if physical.get("required").and_then(Value::as_bool) == Some(true) {
        authority_refs.push(object(physical, "source_ref"));
    }
    if authority_refs
        .iter()
        .any(|source| !source.is_null() && !source_authority_valid(source))
    {
        add(&mut reasons, "OWNER_EVIDENCE_NOT_CURRENT_AUTHORIZED");
    }

    if !matches!(
        text(continuity, "evidence_state"),
        "current" | "revoked" | "disputed"
    ) || text(continuity, "evidence_state") == "stale"
    {
        add(&mut reasons, "CONTINUITY_EVIDENCE_NOT_CURRENT");
        add(&mut reasons, "OWNER_EVIDENCE_NOT_CURRENT_AUTHORIZED");
    }
    if matches!(text(continuity, "evidence_state"), "revoked" | "disputed") {
        add(&mut reasons, "CONTINUITY_EVIDENCE_REVOKED_OR_DISPUTED");
        add(&mut reasons, "OWNER_EVIDENCE_REVOKED_OR_DISPUTED");
    }

    if text(restoration, "work_execution") != "completed" {
        add(&mut reasons, "RESTORATION_WORK_NOT_COMPLETE");
    }
    if text(restoration, "inspection_state") != "pass"
        || text(restoration, "inspection_evidence_state") != "current"
    {
        add(&mut reasons, "INSPECTION_EVIDENCE_INCOMPLETE");
    }
    if text(restoration, "commissioning_state") != "pass"
        || text(restoration, "commissioning_evidence_state") != "current"
    {
        add(&mut reasons, "COMMISSIONING_EVIDENCE_INCOMPLETE");
    }
    let physical_output_required = contract
        .get("physical_output_required")
        .and_then(Value::as_bool)
        == Some(true)
        || physical.get("required").and_then(Value::as_bool) == Some(true)
        || intervention
            .get("accepted_output_required")
            .and_then(Value::as_bool)
            == Some(true);
    if physical_output_required
        && (text(physical, "state") != "accepted"
            || !source_authority_valid(object(physical, "source_ref")))
    {
        add(&mut reasons, "ACCEPTED_PHYSICAL_OUTPUT_MISSING");
    }

    if text(continuity, "state") != "Adequate" {
        add(&mut reasons, "SERVICE_NOT_ADEQUATE");
    }
    let stable = object(continuity, "stable_adequacy");
    let contract_minimum =
        tick(object(contract, "stability_policy"), "minimum_contiguous_adequate_ticks")
            .unwrap_or(u64::MAX);
    let receipt_minimum = tick(stable, "minimum_ticks").unwrap_or(0);
    if stable.get("qualified").and_then(Value::as_bool) != Some(true)
        || tick(stable, "observed_ticks").unwrap_or(0) < contract_minimum
        || receipt_minimum != contract_minimum
    {
        add(&mut reasons, "STABLE_ADEQUACY_INTERVAL_NOT_ESTABLISHED");
    }
    if tick(object(continuity, "trajectory"), "recurrence_count").unwrap_or(0) > 0 {
        add(&mut reasons, "RECURRENT_SERVICE_DEGRADATION");
        add(&mut reasons, "STABLE_ADEQUACY_INTERVAL_NOT_ESTABLISHED");
    }

    if list(input, "mandatory_services").iter().any(|service| {
        service.get("required").and_then(Value::as_bool) == Some(true)
            && text(service, "state") != "adequate"
    }) {
        add(&mut reasons, "DEPENDENT_SERVICE_NOT_RESTORED");
    }
    if list(input, "mandatory_hard_floors").iter().any(|floor| {
        floor.get("required").and_then(Value::as_bool) == Some(true)
            && text(floor, "state") != "pass"
    }) {
        add(&mut reasons, "MANDATORY_SERVICE_FLOOR_FAILED");
    }

    if contract
        .get("measurement_required")
        .and_then(Value::as_bool)
        == Some(true)
    {
        let observation = object(measurement, "observation");
        let observation_ref = object(observation, "source_ref");
        let current_cut = object(object(input, "temporal_cuts"), "current");
        if observation.is_null()
            || observation_ref.is_null()
            || !ref_is_visible(observation_ref, current_cut)
        {
            add(&mut reasons, "MEASUREMENT_NOT_VISIBLE_AT_CURRENT_CUT");
        }
        if tick(observation, "observed_tick").unwrap_or(0)
            < tick(intervention, "completed_tick").unwrap_or(u64::MAX)
        {
            add(&mut reasons, "MEASUREMENT_PRE_INTERVENTION");
        }
        if text(observation, "calibration_state") == "stale" {
            add(&mut reasons, "MEASUREMENT_CALIBRATION_STALE");
        }
        if text(observation, "applicability_state") != "within_domain" {
            add(&mut reasons, "MEASUREMENT_OUTSIDE_APPLICABILITY");
        }
        let observation_ref = object(observation, "source_ref");
        if !observation_ref.is_null()
            && (!source_authority_valid(observation_ref)
                || !source_binding_exists(
                object(input, "scope_binding"),
                observation_ref,
                object(input, "requested_scope"),
            ))
        {
            add(&mut reasons, "MEASUREMENT_OWNER_RECEIPT_BINDING_INVALID");
            add(&mut reasons, "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE");
        }
    }

    let invalid = reasons.iter().any(|reason| {
        matches!(
            reason.as_str(),
            "CONTINUITY_EVIDENCE_REVOKED_OR_DISPUTED" | "OWNER_EVIDENCE_REVOKED_OR_DISPUTED"
        )
    });
    let not_restored = reasons.iter().any(|reason| {
        matches!(
            reason.as_str(),
            "RESTORATION_WORK_NOT_COMPLETE"
                | "SERVICE_NOT_ADEQUATE"
                | "RECURRENT_SERVICE_DEGRADATION"
                | "DEPENDENT_SERVICE_NOT_RESTORED"
                | "MANDATORY_SERVICE_FLOOR_FAILED"
        )
    });
    let stable_only =
        reasons.len() == 1 && reasons.contains("STABLE_ADEQUACY_INTERVAL_NOT_ESTABLISHED");

    let disposition = if invalid {
        "Invalid"
    } else if not_restored {
        "NotRestored"
    } else if stable_only && text(continuity, "state") == "Adequate" {
        "ProvisionalRecovery"
    } else if reasons.is_empty() {
        "RestoredForDeclaredScope"
    } else {
        "InsufficientEvidence"
    };
    (disposition.to_owned(), reasons)
}

fn interruption(input: &Value) -> &Value {
    object(input, "interruption")
}

fn historical_disposition(input: &Value, current: &str, reasons: &BTreeSet<String>) -> String {
    if reasons.contains("MANDATORY_SERVICE_FLOOR_FAILED") {
        return "NotRestored".to_owned();
    }
    let contract = object(input, "service_contract");
    let measurement = object(input, "measurement");
    let observation = object(measurement, "observation");
    let cuts = object(input, "temporal_cuts");
    let historical = object(cuts, "historical");
    if contract.get("measurement_required").and_then(Value::as_bool) == Some(true) {
        let observation_ref = object(observation, "source_ref");
        if observation.is_null()
            || observation_ref.is_null()
            || !ref_is_visible(observation_ref, historical)
            || tick(observation, "observed_tick").unwrap_or(u64::MAX)
                > tick(historical, "valid_tick").unwrap_or(0)
            || tick(observation, "recorded_tick").unwrap_or(u64::MAX)
                > tick(historical, "recorded_tick").unwrap_or(0)
            || tick(observation, "registered_tick").unwrap_or(u64::MAX)
                > tick(historical, "registered_tick").unwrap_or(0)
            || tick(observation, "ingested_tick").unwrap_or(u64::MAX)
                > tick(historical, "ingested_tick").unwrap_or(0)
        {
            return "InsufficientEvidence".to_owned();
        }
    }
    if reasons.contains("OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE")
        || reasons.contains("COMMON_SCOPE_NOT_COMPARABLE")
        || reasons.contains("COMMON_SCOPE_BINDING_UNKNOWN")
    {
        return "InsufficientEvidence".to_owned();
    }
    current.to_owned()
}

pub fn evaluate_case(case: &Value) -> Value {
    let input = object(case, "input");
    let (current_disposition, reasons) = evaluate_current(input);
    let historical = historical_disposition(input, &current_disposition, &reasons);
    let restoration = object(input, "restoration");
    let continuity = object(input, "continuity");
    let resident = object(input, "resident_experience");
    let disposition = current_disposition == "RestoredForDeclaredScope";

    json!({
        "case_id": text(case, "case_id"),
        "bridge_disposition": current_disposition,
        "historical_disposition": historical,
        "plan_readiness": text(restoration, "plan_readiness"),
        "work_execution": text(restoration, "work_execution"),
        "inspection_state": text(restoration, "inspection_state"),
        "commissioning_state": text(restoration, "commissioning_state"),
        "continuity_state": text(continuity, "state"),
        "measured_outcome_state": measured_outcome(input),
        "disclosure_disposition": text(resident, "disclosure_state"),
        "restoration_qualified_for_scope": disposition,
        "recovery_trajectory": recovery_trajectory(input),
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "claim_ceiling": CLAIM_CEILING,
        "action_authority": "none",
        "causal_effect_established": false
    })
}

pub fn validate_case(case: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check_keys(case, "case", CASE_KEYS, &mut errors);
    let input = object(case, "input");
    check_keys(input, "input", INPUT_KEYS, &mut errors);
    for (key, allowed) in [
        (
            "service_contract",
            &[
                "service_id",
                "service_category",
                "owner",
                "contract_id",
                "revision",
                "state",
                "functional_unit",
                "beneficiary_group",
                "beneficiary_denominator",
                "minimum_service_floor",
                "measurement_required",
                "stability_policy",
                "physical_output_required",
                "source_ref",
            ][..],
        ),
        (
            "requested_scope",
            &[
                "subject_id",
                "subject_digest",
                "functional_unit",
                "group_id",
                "population_frame",
                "scenario_id",
                "common_scope_digest",
            ][..],
        ),
        (
            "scope_binding",
            &[
                "binding_state",
                "common_scope_digest",
                "binder_contract_id",
                "binder_schema_version",
                "crosswalk_id",
                "binder_source_commit",
                "source_bindings",
            ][..],
        ),
        (
            "interruption",
            &["event_id", "state", "start_tick", "end_tick", "source_ref"][..],
        ),
        (
            "intervention",
            &["completed_tick", "accepted_output_required"][..],
        ),
        (
            "restoration",
            &[
                "plan_readiness",
                "work_execution",
                "execution_evidence_state",
                "inspection_state",
                "inspection_evidence_state",
                "commissioning_state",
                "commissioning_evidence_state",
                "accepted_output_state",
                "source_ref",
            ][..],
        ),
        (
            "continuity",
            &[
                "state",
                "recovery_state",
                "evidence_state",
                "rights_state",
                "source_ref",
                "stable_adequacy",
                "trajectory",
            ][..],
        ),
        (
            "measurement",
            &[
                "required",
                "evaluation_state",
                "source_ref",
                "metric_results",
                "observation",
            ][..],
        ),
        ("physical_output", &["required", "state", "source_ref"][..]),
        (
            "resident_experience",
            &[
                "requested",
                "present",
                "consent_state",
                "purpose",
                "authorized_purpose",
                "disclosure_state",
            ][..],
        ),
        ("temporal_cuts", &["historical", "current"][..]),
    ] {
        check_keys(
            object(input, key),
            &format!("input.{key}"),
            allowed,
            &mut errors,
        );
    }

    for source in [
        object(object(input, "service_contract"), "source_ref"),
        object(object(input, "restoration"), "source_ref"),
        object(object(input, "interruption"), "source_ref"),
        object(object(input, "continuity"), "source_ref"),
        object(object(input, "measurement"), "source_ref"),
        object(
            object(object(input, "measurement"), "observation"),
            "source_ref",
        ),
        object(object(input, "physical_output"), "source_ref"),
    ] {
        if !source.is_null() {
            check_keys(source, "source_ref", REF_KEYS, &mut errors);
        }
    }

    for cut_name in ["historical", "current"] {
        check_keys(
            object(object(input, "temporal_cuts"), cut_name),
            &format!("input.temporal_cuts.{cut_name}"),
            &[
                "valid_tick",
                "recorded_tick",
                "registered_tick",
                "ingested_tick",
            ],
            &mut errors,
        );
    }
    check_keys(
        object(object(input, "service_contract"), "stability_policy"),
        "input.service_contract.stability_policy",
        &["minimum_contiguous_adequate_ticks"],
        &mut errors,
    );
    check_keys(
        object(object(input, "continuity"), "stable_adequacy"),
        "input.continuity.stable_adequacy",
        &[
            "confirmed_tick",
            "minimum_ticks",
            "observed_ticks",
            "qualified",
            "start_tick",
        ],
        &mut errors,
    );
    check_keys(
        object(object(input, "continuity"), "trajectory"),
        "input.continuity.trajectory",
        &[
            "adequate_ticks",
            "constrained_ticks",
            "first_adequate_tick",
            "inaccessible_ticks",
            "last_degradation_end_tick",
            "recurrence_count",
            "unknown_ticks",
        ],
        &mut errors,
    );
    let measurement = object(input, "measurement");
    for (index, metric) in list(measurement, "metric_results").iter().enumerate() {
        check_keys(
            metric,
            &format!("input.measurement.metric_results[{index}]"),
            &[
                "metric_id",
                "observation_state",
                "observed_value",
                "reason_codes",
                "threshold_state",
                "unit",
            ],
            &mut errors,
        );
    }
    let observation = object(measurement, "observation");
    if !observation.is_null() {
        check_keys(
            observation,
            "input.measurement.observation",
            &[
                "applicability_state",
                "calibration_state",
                "direction",
                "evidence_kind",
                "evidence_state",
                "ingested_tick",
                "metric_id",
                "observation_window",
                "observed_tick",
                "recorded_tick",
                "registered_tick",
                "source_ref",
                "threshold",
                "unit",
                "value",
            ],
            &mut errors,
        );
        check_keys(
            object(observation, "observation_window"),
            "input.measurement.observation.observation_window",
            &["start_tick", "end_tick"],
            &mut errors,
        );
    }
    check_keys(
        object(input, "requested_scope"),
        "input.requested_scope",
        &[
            "subject_id",
            "subject_digest",
            "functional_unit",
            "group_id",
            "population_frame",
            "scenario_id",
            "common_scope_digest",
        ],
        &mut errors,
    );
    check_keys(
        object(input, "intervention"),
        "input.intervention",
        &["completed_tick", "accepted_output_required"],
        &mut errors,
    );

    for (index, binding) in list(object(input, "scope_binding"), "source_bindings")
        .iter()
        .enumerate()
    {
        check_keys(
            binding,
            &format!("input.scope_binding.source_bindings[{index}]"),
            &[
                "owner",
                "contract_id",
                "contract_revision",
                "receipt_id",
                "source_scope_digest",
                "subject_id",
                "subject_digest",
                "functional_unit",
                "group_id",
                "scenario_id",
                "common_scope_digest",
                "binding_state",
            ],
            &mut errors,
        );
    }
    for (index, service) in list(input, "mandatory_services").iter().enumerate() {
        check_keys(
            service,
            &format!("input.mandatory_services[{index}]"),
            &["service_id", "service_category", "state", "required"],
            &mut errors,
        );
    }
    for (index, floor) in list(input, "mandatory_hard_floors").iter().enumerate() {
        check_keys(
            floor,
            &format!("input.mandatory_hard_floors[{index}]"),
            &["floor_id", "state", "required"],
            &mut errors,
        );
    }
    check_keys(
        object(case, "expected"),
        "expected",
        EXPECTED_KEYS,
        &mut errors,
    );
    errors.sort();
    errors
}

pub fn validate_corpus(corpus: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check_keys(corpus, "corpus", ROOT_KEYS, &mut errors);
    if text(corpus, "corpus_id") != CORPUS_ID
        || text(corpus, "schema_version") != "0.1.0"
        || text(corpus, "classification") != "synthetic_fixture_only"
        || text(corpus, "claim_ceiling") != CLAIM_CEILING
    {
        errors.push("corpus:IDENTITY_OR_CLAIM_CEILING_UNSUPPORTED".to_owned());
    }
    if list(corpus, "cases").len() != 26 {
        errors.push("corpus:EXPECTED_26_CASES".to_owned());
    }
    let mut seen = BTreeSet::new();
    for (index, case) in list(corpus, "cases").iter().enumerate() {
        let id = text(case, "case_id");
        if id.is_empty() || !seen.insert(id.to_owned()) {
            errors.push(format!("corpus.cases[{index}]:EMPTY_OR_DUPLICATE_ID"));
        }
        if text(object(case, "input"), "case_id") != id {
            errors.push(format!("corpus.cases[{index}]:CASE_ID_MISMATCH"));
        }
        errors.extend(validate_case(case));
    }
    errors.sort();
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const CORPUS: &str = include_str!(
        "../../../docs/research/city-restoration-outcome-bridge/CITY_RESTORATION_OUTCOME_BRIDGE_CORPUS_V0_1.json"
    );

    fn corpus() -> Value {
        serde_json::from_str(CORPUS).expect("frozen corpus parses")
    }

    #[test]
    fn all_twenty_six_frozen_scenarios_match_exactly() {
        let corpus = corpus();
        let errors = validate_corpus(&corpus);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(list(&corpus, "cases").len(), 26);
        for case in list(&corpus, "cases") {
            let actual = evaluate_case(case);
            assert_eq!(
                actual,
                case["expected"],
                "{}: actual={} expected={}",
                text(case, "case_id"),
                actual,
                case["expected"]
            );
        }
    }

    #[test]
    fn duplicate_decoded_json_keys_are_rejected_before_map_creation() {
        assert!(parse_unique_json_bytes(br#"{"claim":1,"claim":1}"#).is_err());
        assert!(parse_unique_json_bytes(br#"{"claim":1,"\u0063laim":2}"#).is_err());
        assert!(parse_unique_json_bytes(br#"{"ok":true} trailing"#).is_err());
    }

    #[test]
    fn unknown_root_and_nested_fields_fail_closed() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["unreviewed_extension"] = json!(true);
        assert!(!validate_case(&case).is_empty());

        let mut nested = corpus()["cases"][0].clone();
        nested["input"]["service_contract"]["silent_fallback"] = json!(true);
        assert!(!validate_case(&nested).is_empty());
    }

    #[test]
    fn panel_acceptance_does_not_substitute_for_building_commissioning() {
        let case = corpus()["cases"][20].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "InsufficientEvidence");
        assert_eq!(result["commissioning_state"], "not_performed");
    }

    #[test]
    fn no_consent_is_not_a_negative_service_measurement() {
        let case = corpus()["cases"][12].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "RestoredForDeclaredScope");
        assert_eq!(result["measured_outcome_state"], "NotRequired");
        assert_eq!(result["disclosure_disposition"], "denied");
    }

    #[test]
    fn one_failed_mandatory_floor_cannot_be_averaged_away() {
        let case = corpus()["cases"][15].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "NotRestored");
        assert_eq!(
            result["reason_codes"],
            json!(["MANDATORY_SERVICE_FLOOR_FAILED"])
        );
    }

    #[test]
    fn historical_replay_excludes_later_ingested_measurement() {
        let case = corpus()["cases"][0].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["historical_disposition"], "InsufficientEvidence");
        assert_eq!(result["bridge_disposition"], "RestoredForDeclaredScope");
    }

    #[test]
    fn scope_binding_must_match_source_scope_digest_and_population_identity() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["scope_binding"]["source_bindings"][0]["source_scope_digest"] =
            json!("9999999999999999999999999999999999999999999999999999999999999999");
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "InsufficientEvidence");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE")
        );

        let mut case = corpus()["cases"][0].clone();
        case["input"]["scope_binding"]["source_bindings"][0]["subject_digest"] =
            json!("9999999999999999999999999999999999999999999999999999999999999999");
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "InsufficientEvidence");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE")
        );
    }

    #[test]
    fn scope_binding_rejects_a_receipt_bound_to_another_functional_unit() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["scope_binding"]["source_bindings"][0]["functional_unit"] =
            json!("FU-BLDG-01");
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "InsufficientEvidence");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "OWNER_RECEIPT_NOT_BOUND_TO_COMMON_SCOPE")
        );
    }

    #[test]
    fn current_replay_rejects_a_late_ingested_owner_receipt() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["measurement"]["observation"]["source_ref"]["ingested_tick"] = json!(16);
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "InsufficientEvidence");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "OWNER_RECEIPT_NOT_VISIBLE_AT_CURRENT_CUT")
        );
    }

    #[test]
    fn stable_period_must_match_the_service_contract_floor() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["continuity"]["stable_adequacy"]["minimum_ticks"] = json!(1);
        let result = evaluate_case(&case);
        assert_eq!(result["bridge_disposition"], "ProvisionalRecovery");
        assert_eq!(
            result["reason_codes"],
            json!(["STABLE_ADEQUACY_INTERVAL_NOT_ESTABLISHED"])
        );
    }

    #[test]
    fn recovery_trajectory_uses_owner_ticks_without_synthesizing_unmet_demand() {
        let case = corpus()["cases"][0].clone();
        let result = evaluate_case(&case);
        assert_eq!(
            result["recovery_trajectory"]["time_to_first_adequate_ticks"],
            2
        );
        assert_eq!(
            result["recovery_trajectory"]["time_to_sustained_adequate_ticks"],
            4
        );
        assert!(result["recovery_trajectory"]["demand_units_unmet"].is_null());
        assert_eq!(
            result["recovery_trajectory"]["denominator_state"],
            "not_established"
        );
    }
}
