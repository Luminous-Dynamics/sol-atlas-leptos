use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{env, fs, process};

const CONTRACT_ID: &str = "highveld-cross-tool-household-resilience-benchmark-v0.1";
const RECEIPT_FORMAT: &str = "highveld-b2-composition-receipt-v0.1";
const RECEIPT_FIELDS: &[&str] = &[
    "event_trace_sha256",
    "geometry_sha256",
    "resident_requirements_sha256",
    "initial_state_sha256",
    "source_snapshot_sha256",
    "engine_input_sha256",
    "engine_output_sha256",
    "source_rights_revision",
];
const MATCHED_TEXT_FIELDS: &[&str] = &[
    "design_revision_id",
    "resource_budget_id",
    "information_budget_id",
    "functional_unit_id",
    "time_window_id",
];
const MATCHED_DIGEST_FIELDS: &[&str] = &[
    "geometry_sha256",
    "resident_requirements_sha256",
    "event_trace_sha256",
    "initial_state_sha256",
    "source_snapshot_sha256",
];

fn push_error(errors: &mut Vec<Value>, pair_id: &str, code: &str, detail: &str) {
    errors.push(json!({ "pair_id": pair_id, "code": code, "detail": detail }));
}

fn text_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn valid_sha256(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .map(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
        .unwrap_or(false)
}

fn calculate_receipt_sha256(receipt: &Value) -> Option<String> {
    if text_field(receipt, "receipt_format") != Some(RECEIPT_FORMAT) {
        return None;
    }

    // Versioned, length-prefixed field encoding avoids JSON key-order and
    // whitespace ambiguity. Length is measured in UTF-8 bytes.
    let mut canonical = Vec::new();
    canonical.extend_from_slice(b"HIGHVELD_B2_COMPOSITION_RECEIPT_V0.1\n");
    for field in RECEIPT_FIELDS {
        let value = text_field(receipt, field)?;
        canonical.extend_from_slice(field.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(value.len().to_string().as_bytes());
        canonical.push(b':');
        canonical.extend_from_slice(value.as_bytes());
        canonical.push(b'\n');
    }
    Some(format!("{:x}", Sha256::digest(&canonical)))
}

fn check_sha256(value: Option<&Value>, pair_id: &str, field: &str, errors: &mut Vec<Value>) {
    if !valid_sha256(value) {
        push_error(
            errors,
            pair_id,
            "INVALID_SHA256",
            &format!("{field} must be 64 lowercase hexadecimal characters"),
        );
    }
}

fn audit_arm_decisions(
    arm: &Value,
    arm_name: &str,
    pair_id: &str,
    decision_as_of: u64,
    expectations: &[Value],
    errors: &mut Vec<Value>,
    future_leaks: &mut u64,
    critical_false_passes: &mut u64,
) -> u64 {
    let Some(decisions) = arm.get("decisions").and_then(Value::as_array) else {
        push_error(
            errors,
            pair_id,
            "MISSING_DECISIONS",
            &format!("{arm_name} must contain a decisions array"),
        );
        return 0;
    };

    let mut seen: Vec<String> = Vec::new();
    let mut mismatches = 0_u64;

    for decision in decisions {
        let Some(service_id) = text_field(decision, "service_id") else {
            push_error(
                errors,
                pair_id,
                "MISSING_SERVICE_ID",
                &format!("{arm_name} decision has no service_id"),
            );
            continue;
        };

        if seen.iter().any(|seen_id| seen_id == service_id) {
            push_error(
                errors,
                pair_id,
                "DUPLICATE_SERVICE_DECISION",
                &format!("{arm_name} duplicates {service_id}"),
            );
        }
        seen.push(service_id.to_owned());

        let actual_status = text_field(decision, "status");
        let actual_floor = text_field(decision, "hard_floor_state");
        if !matches!(actual_status, Some("PASS" | "FAIL" | "BLOCKED" | "UNKNOWN")) {
            push_error(
                errors,
                pair_id,
                "INVALID_DECISION_STATUS",
                &format!("{arm_name}/{service_id} has an invalid status"),
            );
        }
        if !matches!(
            actual_floor,
            Some("SATISFIED" | "VIOLATED" | "UNKNOWN" | "NOT_APPLICABLE")
        ) {
            push_error(
                errors,
                pair_id,
                "INVALID_HARD_FLOOR_STATE",
                &format!("{arm_name}/{service_id} has an invalid hard_floor_state"),
            );
        }

        let expected = expectations
            .iter()
            .find(|item| text_field(item, "service_id") == Some(service_id));
        let Some(expected) = expected else {
            push_error(
                errors,
                pair_id,
                "UNEXPECTED_SERVICE_DECISION",
                &format!("{arm_name} returned unregistered service {service_id}"),
            );
            continue;
        };

        let expected_status = text_field(expected, "expected_status");
        let expected_floor = text_field(expected, "expected_hard_floor_state");
        if actual_status != expected_status || actual_floor != expected_floor {
            mismatches += 1;
            if arm_name == "B2" {
                push_error(
                    errors,
                    pair_id,
                    "B2_ORACLE_MISMATCH",
                    &format!(
                        "{service_id}: B2 returned {:?}/{:?}, oracle requires {:?}/{:?}",
                        actual_status, actual_floor, expected_status, expected_floor
                    ),
                );
            }
        }

        let oracle_floor_unsafe = matches!(expected_floor, Some("VIOLATED" | "UNKNOWN"));
        let actual_floor_unsafe = matches!(actual_floor, Some("VIOLATED" | "UNKNOWN"));
        let critical_false_pass = actual_status == Some("PASS")
            && (expected_status != Some("PASS") || oracle_floor_unsafe || actual_floor_unsafe);

        if critical_false_pass {
            *critical_false_passes += 1;
            if arm_name == "B2" {
                push_error(
                    errors,
                    pair_id,
                    "B2_CRITICAL_FALSE_PASS",
                    &format!("{service_id}: PASS despite an unsafe hard floor or non-PASS oracle"),
                );
            }
        }

        if arm_name == "B2" {
            let validation = decision.get("evidence_validation");
            let rights_state = validation.and_then(|v| text_field(v, "rights_state"));
            let scope_state = validation.and_then(|v| text_field(v, "scope_join_state"));
            if rights_state.is_none() || scope_state.is_none() {
                push_error(
                    errors,
                    pair_id,
                    "B2_VALIDATION_MISSING",
                    &format!("{service_id}: B2 must record rights_state and scope_join_state"),
                );
            } else if actual_status == Some("PASS")
                && (rights_state != Some("VALID") || scope_state != Some("APPROVED"))
            {
                push_error(
                    errors,
                    pair_id,
                    "B2_UNQUALIFIED_EVIDENCE_PASS",
                    &format!("{service_id}: PASS requires VALID rights and APPROVED scope"),
                );
            }
        }

        let Some(evidence) = decision.get("evidence").and_then(Value::as_array) else {
            push_error(
                errors,
                pair_id,
                "EVIDENCE_LIST_MISSING",
                &format!("{arm_name}/{service_id} must include an evidence array"),
            );
            continue;
        };

        for item in evidence {
            if text_field(item, "evidence_id").is_none() {
                push_error(
                    errors,
                    pair_id,
                    "EVIDENCE_ID_MISSING",
                    &format!("{arm_name}/{service_id} evidence has no evidence_id"),
                );
            }
            let available_at = item.get("available_at_offset_s").and_then(Value::as_u64);
            let used = item.get("used_for_decision").and_then(Value::as_bool);
            if available_at.is_none() || used.is_none() {
                push_error(
                    errors,
                    pair_id,
                    "EVIDENCE_AVAILABILITY_MISSING",
                    &format!(
                        "{arm_name}/{service_id} evidence needs availability time and used_for_decision"
                    ),
                );
            } else if used == Some(true) && available_at.unwrap_or(0) > decision_as_of {
                *future_leaks += 1;
                push_error(
                    errors,
                    pair_id,
                    "FUTURE_INFORMATION_LEAK",
                    &format!(
                        "{arm_name}/{service_id} used evidence available at {} after decision time {decision_as_of}",
                        available_at.unwrap_or(0)
                    ),
                );
            }
        }
    }

    for expected in expectations {
        if let Some(service_id) = text_field(expected, "service_id") {
            if !seen.iter().any(|seen_id| seen_id == service_id) {
                push_error(
                    errors,
                    pair_id,
                    "MISSING_SERVICE_DECISION",
                    &format!("{arm_name} omitted oracle service {service_id}"),
                );
            }
        }
    }
    mismatches
}

fn audit_manifest(root: &Value) -> Value {
    let mut errors: Vec<Value> = Vec::new();
    let mut b1_oracle_mismatches = 0_u64;
    let mut b2_oracle_mismatches = 0_u64;
    let mut b1_critical_false_passes = 0_u64;
    let mut b2_critical_false_passes = 0_u64;
    let mut future_information_leaks = 0_u64;
    let mut pairs_checked = 0_u64;
    let mut seen_pair_ids: Vec<String> = Vec::new();

    if text_field(root, "schema_version") != Some("0.1.0") {
        push_error(
            &mut errors,
            "",
            "SCHEMA_VERSION_MISMATCH",
            "schema_version must be 0.1.0",
        );
    }
    if text_field(root, "benchmark_contract_id") != Some(CONTRACT_ID) {
        push_error(
            &mut errors,
            "",
            "CONTRACT_ID_MISMATCH",
            "benchmark_contract_id does not match Highveld v0.1",
        );
    }
    if text_field(root, "experiment_id").is_none() {
        push_error(
            &mut errors,
            "",
            "EXPERIMENT_ID_MISSING",
            "experiment_id must be non-empty",
        );
    }
    if text_field(root, "claim_ceiling").is_none() {
        push_error(
            &mut errors,
            "",
            "CLAIM_CEILING_MISSING",
            "claim_ceiling must be non-empty",
        );
    }

    let Some(pairs) = root.get("pairs").and_then(Value::as_array) else {
        push_error(
            &mut errors,
            "",
            "PAIRS_MISSING",
            "pairs must be a non-empty array",
        );
        return json!({
            "status": "FAIL",
            "protocol": "highveld-b1-b2-paired-audit-v0.1",
            "pairs_checked": 0,
            "b1_oracle_mismatch_count": 0,
            "b2_oracle_mismatch_count": 0,
            "b1_critical_false_pass_count": 0,
            "b2_critical_false_pass_count": 0,
            "future_information_leak_count": 0,
            "errors": errors
        });
    };

    if pairs.is_empty() {
        push_error(
            &mut errors,
            "",
            "PAIRS_EMPTY",
            "at least one matched pair is required",
        );
    }

    for pair in pairs {
        pairs_checked += 1;
        let pair_id = text_field(pair, "pair_id").unwrap_or("<missing-pair-id>");
        if seen_pair_ids.iter().any(|seen| seen == pair_id) {
            push_error(
                &mut errors,
                pair_id,
                "DUPLICATE_PAIR_ID",
                "pair_id must be unique within an experiment",
            );
        }
        seen_pair_ids.push(pair_id.to_owned());

        if text_field(pair, "candidate_id").is_none() || text_field(pair, "scenario_id").is_none() {
            push_error(
                &mut errors,
                pair_id,
                "PAIR_IDENTITY_MISSING",
                "candidate_id and scenario_id must be non-empty",
            );
        }
        let decision_as_of = pair.get("decision_as_of_offset_s").and_then(Value::as_u64);
        if decision_as_of.is_none() {
            push_error(
                &mut errors,
                pair_id,
                "DECISION_TIME_MISSING",
                "decision_as_of_offset_s must be a non-negative integer",
            );
        }
        let decision_as_of = decision_as_of.unwrap_or(0);

        let Some(matched) = pair.get("matched_inputs") else {
            push_error(
                &mut errors,
                pair_id,
                "MATCHED_INPUTS_MISSING",
                "matched_inputs object is required",
            );
            continue;
        };
        for field in MATCHED_TEXT_FIELDS {
            if text_field(matched, field).is_none() {
                push_error(
                    &mut errors,
                    pair_id,
                    "MATCHED_INPUT_FIELD_MISSING",
                    &format!("matched_inputs.{field} must be non-empty"),
                );
            }
        }
        for field in MATCHED_DIGEST_FIELDS {
            check_sha256(matched.get(field), pair_id, field, &mut errors);
        }

        let Some(b1) = pair.get("b1") else {
            push_error(&mut errors, pair_id, "B1_MISSING", "b1 arm is required");
            continue;
        };
        let Some(b2) = pair.get("b2") else {
            push_error(&mut errors, pair_id, "B2_MISSING", "b2 arm is required");
            continue;
        };

        let b1_versions = b1.get("engine_tool_versions");
        let b2_versions = b2.get("engine_tool_versions");
        if b1_versions
            .and_then(Value::as_array)
            .map_or(true, |v| v.is_empty())
            || b1_versions != b2_versions
        {
            push_error(
                &mut errors,
                pair_id,
                "ENGINE_VERSION_MISMATCH",
                "B1 and B2 need the same non-empty engine_tool_versions array",
            );
        }

        let b1_input = b1.get("engine_input_sha256");
        let b2_input = b2.get("engine_input_sha256");
        let b1_output = b1.get("engine_output_sha256");
        let b2_output = b2.get("engine_output_sha256");
        check_sha256(b1_input, pair_id, "b1.engine_input_sha256", &mut errors);
        check_sha256(b2_input, pair_id, "b2.engine_input_sha256", &mut errors);
        check_sha256(b1_output, pair_id, "b1.engine_output_sha256", &mut errors);
        check_sha256(b2_output, pair_id, "b2.engine_output_sha256", &mut errors);
        if b1_input != b2_input || b1_output != b2_output {
            push_error(
                &mut errors,
                pair_id,
                "ENGINE_IO_MISMATCH",
                "B1 and B2 must have identical raw engine input and output digests",
            );
        }

        if b1
            .get("composition_receipt")
            .map_or(false, |r| !r.is_null())
            || b1.get("composition_layer").map_or(false, |r| !r.is_null())
        {
            push_error(
                &mut errors,
                pair_id,
                "B1_HAS_COMPOSITION_LAYER",
                "B1 must remain the specialist-tool baseline without Luminous composition",
            );
        }

        let layer_ok = b2
            .get("composition_layer")
            .map(|layer| {
                text_field(layer, "name").is_some() && text_field(layer, "version").is_some()
            })
            .unwrap_or(false);
        if !layer_ok {
            push_error(
                &mut errors,
                pair_id,
                "B2_COMPOSITION_LAYER_MISSING",
                "B2 must identify a named and versioned composition layer",
            );
        }

        let Some(receipt) = b2.get("composition_receipt") else {
            push_error(
                &mut errors,
                pair_id,
                "B2_RECEIPT_MISSING",
                "B2 must carry a composition receipt",
            );
            continue;
        };

        for (receipt_field, matched_field) in [
            ("event_trace_sha256", "event_trace_sha256"),
            ("geometry_sha256", "geometry_sha256"),
            (
                "resident_requirements_sha256",
                "resident_requirements_sha256",
            ),
            ("initial_state_sha256", "initial_state_sha256"),
            ("source_snapshot_sha256", "source_snapshot_sha256"),
        ] {
            check_sha256(
                receipt.get(receipt_field),
                pair_id,
                receipt_field,
                &mut errors,
            );
            if receipt.get(receipt_field) != matched.get(matched_field) {
                push_error(
                    &mut errors,
                    pair_id,
                    "RECEIPT_INPUT_BINDING_MISMATCH",
                    &format!("B2 receipt does not bind matched input {matched_field}"),
                );
            }
        }

        check_sha256(
            receipt.get("engine_input_sha256"),
            pair_id,
            "receipt.engine_input_sha256",
            &mut errors,
        );
        check_sha256(
            receipt.get("engine_output_sha256"),
            pair_id,
            "receipt.engine_output_sha256",
            &mut errors,
        );
        check_sha256(
            receipt.get("receipt_sha256"),
            pair_id,
            "receipt.receipt_sha256",
            &mut errors,
        );
        if receipt.get("engine_input_sha256") != b2_input
            || receipt.get("engine_output_sha256") != b2_output
        {
            push_error(
                &mut errors,
                pair_id,
                "RECEIPT_ENGINE_BINDING_MISMATCH",
                "B2 receipt must bind exact raw engine input and output digests",
            );
        }
        if text_field(receipt, "source_rights_revision").is_none() {
            push_error(
                &mut errors,
                pair_id,
                "RIGHTS_REVISION_MISSING",
                "B2 receipt must pin the source-rights revision",
            );
        }
        if text_field(receipt, "receipt_format") != Some(RECEIPT_FORMAT) {
            push_error(
                &mut errors,
                pair_id,
                "RECEIPT_FORMAT_MISMATCH",
                "B2 receipt must identify the supported versioned canonical encoding",
            );
        } else if calculate_receipt_sha256(receipt).as_deref()
            != text_field(receipt, "receipt_sha256")
        {
            push_error(
                &mut errors,
                pair_id,
                "RECEIPT_DIGEST_MISMATCH",
                "B2 receipt SHA-256 does not match its canonical payload",
            );
        }

        let Some(expectations) = pair.get("oracle_expectations").and_then(Value::as_array) else {
            push_error(
                &mut errors,
                pair_id,
                "ORACLE_EXPECTATIONS_MISSING",
                "oracle_expectations array is required",
            );
            continue;
        };
        if expectations.is_empty() {
            push_error(
                &mut errors,
                pair_id,
                "ORACLE_EXPECTATIONS_EMPTY",
                "at least one independent service expectation is required",
            );
        }

        b1_oracle_mismatches += audit_arm_decisions(
            b1,
            "B1",
            pair_id,
            decision_as_of,
            expectations,
            &mut errors,
            &mut future_information_leaks,
            &mut b1_critical_false_passes,
        );
        b2_oracle_mismatches += audit_arm_decisions(
            b2,
            "B2",
            pair_id,
            decision_as_of,
            expectations,
            &mut errors,
            &mut future_information_leaks,
            &mut b2_critical_false_passes,
        );
    }

    json!({
        "status": if errors.is_empty() { "PASS" } else { "FAIL" },
        "protocol": "highveld-b1-b2-paired-audit-v0.1",
        "pairs_checked": pairs_checked,
        "b1_oracle_mismatch_count": b1_oracle_mismatches,
        "b2_oracle_mismatch_count": b2_oracle_mismatches,
        "b1_critical_false_pass_count": b1_critical_false_passes,
        "b2_critical_false_pass_count": b2_critical_false_passes,
        "future_information_leak_count": future_information_leaks,
        "errors": errors
    })
}

fn repeated_digest(ch: char) -> String {
    ch.to_string().repeat(64)
}

fn synthetic_decision(status: &str, floor: &str, rights: Option<&str>, used: bool) -> Value {
    let mut decision = json!({
        "service_id": "heatbank_hot_water",
        "status": status,
        "hard_floor_state": floor,
        "evidence": [{
            "evidence_id": "synthetic_observation_001",
            "available_at_offset_s": 20,
            "used_for_decision": used
        }]
    });
    if let Some(rights_state) = rights {
        decision["evidence_validation"] = json!({
            "rights_state": rights_state,
            "scope_join_state": "APPROVED"
        });
    }
    decision
}

fn synthetic_pair(
    pair_id: &str,
    rights: &str,
    b1_status: &str,
    b1_floor: &str,
    b2_status: &str,
    b2_floor: &str,
    expected_status: &str,
    expected_floor: &str,
) -> Value {
    let geometry = repeated_digest('b');
    let requirements = repeated_digest('c');
    let trace = repeated_digest('a');
    let initial = repeated_digest('d');
    let source = repeated_digest('e');
    let engine_input = repeated_digest('f');
    let engine_output = repeated_digest('1');
    let versions = json!([
        { "tool": "EnergyPlus", "version": "26.2.0" },
        { "tool": "BOPTEST", "version": "0.9.0" }
    ]);
    let mut composition_receipt = json!({
        "receipt_format": RECEIPT_FORMAT,
        "event_trace_sha256": trace,
        "geometry_sha256": geometry,
        "resident_requirements_sha256": requirements,
        "initial_state_sha256": initial,
        "source_snapshot_sha256": source,
        "engine_input_sha256": engine_input,
        "engine_output_sha256": engine_output,
        "source_rights_revision": "synthetic-rights-r1",
        "receipt_sha256": ""
    });
    composition_receipt["receipt_sha256"] =
        json!(calculate_receipt_sha256(&composition_receipt).unwrap_or_default());

    json!({
        "pair_id": pair_id,
        "candidate_id": "INDEPENDENT_SERVICES",
        "scenario_id": "synthetic_control_only",
        "decision_as_of_offset_s": 50,
        "matched_inputs": {
            "design_revision_id": "synthetic-design-r1",
            "geometry_sha256": geometry,
            "resident_requirements_sha256": requirements,
            "event_trace_sha256": trace,
            "initial_state_sha256": initial,
            "source_snapshot_sha256": source,
            "resource_budget_id": "synthetic-resource-budget-r1",
            "information_budget_id": "synthetic-information-budget-r1",
            "functional_unit_id": "resident-task-hour-r1",
            "time_window_id": "scenario-t0-plus-3600s"
        },
        "b1": {
            "engine_tool_versions": versions,
            "engine_input_sha256": engine_input,
            "engine_output_sha256": engine_output,
            "operator_minutes": 20,
            "decisions": [synthetic_decision(b1_status, b1_floor, None, true)]
        },
        "b2": {
            "engine_tool_versions": versions,
            "engine_input_sha256": engine_input,
            "engine_output_sha256": engine_output,
            "operator_minutes": 22,
            "composition_layer": { "name": "Luminous evidence composition", "version": "0.1.0" },
            "composition_receipt": composition_receipt,
            "decisions": [synthetic_decision(b2_status, b2_floor, Some(rights), rights == "VALID")]
        },
        "oracle_expectations": [{
            "service_id": "heatbank_hot_water",
            "expected_status": expected_status,
            "expected_hard_floor_state": expected_floor
        }]
    })
}

fn synthetic_self_test_manifest() -> Value {
    json!({
        "schema_version": "0.1.0",
        "benchmark_contract_id": CONTRACT_ID,
        "experiment_id": "synthetic-self-test-not-an-industry-result",
        "claim_ceiling": "Synthetic harness self-test only; no incumbent tool or physical dwelling was evaluated.",
        "pairs": [
            synthetic_pair("clean-source-known-answer", "VALID", "PASS", "SATISFIED", "PASS", "SATISFIED", "PASS", "SATISFIED"),
            synthetic_pair("revoked-source-known-answer", "REVOKED", "PASS", "UNKNOWN", "BLOCKED", "UNKNOWN", "BLOCKED", "UNKNOWN")
        ]
    })
}

fn run_self_test() -> Value {
    let report = audit_manifest(&synthetic_self_test_manifest());
    let metrics_ok = report["status"].as_str() == Some("PASS")
        && report["pairs_checked"].as_u64() == Some(2)
        && report["b1_oracle_mismatch_count"].as_u64() == Some(1)
        && report["b2_oracle_mismatch_count"].as_u64() == Some(0)
        && report["b1_critical_false_pass_count"].as_u64() == Some(1)
        && report["b2_critical_false_pass_count"].as_u64() == Some(0)
        && report["future_information_leak_count"].as_u64() == Some(0);
    json!({
        "self_test_status": if metrics_ok { "PASS" } else { "FAIL" },
        "claim_ceiling": "Synthetic harness self-test only; not a B1/B2 industry result.",
        "audit_report": report
    })
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && args[0] == "--self-test" {
        let result = run_self_test();
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_owned())
        );
        if result["self_test_status"].as_str() != Some("PASS") {
            process::exit(1);
        }
        return;
    }

    if args.len() == 2 && args[0] == "--input" {
        let content = match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(error) => {
                eprintln!("failed to read {}: {error}", args[1]);
                process::exit(2);
            }
        };
        let manifest: Value = match serde_json::from_str(&content) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("invalid JSON in {}: {error}", args[1]);
                process::exit(2);
            }
        };
        let report = audit_manifest(&manifest);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_owned())
        );
        if report["status"].as_str() != Some("PASS") {
            process::exit(1);
        }
        return;
    }

    eprintln!(
        "Usage:\n  cargo run --bin highveld-paired-audit -- --self-test\n  cargo run --bin highveld-paired-audit -- --input <paired-run.json>"
    );
    process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_error(report: &Value, code: &str) -> bool {
        report["errors"]
            .as_array()
            .map(|errors| {
                errors
                    .iter()
                    .any(|error| error.get("code").and_then(Value::as_str) == Some(code))
            })
            .unwrap_or(false)
    }

    #[test]
    fn synthetic_self_test_demonstrates_baseline_miss_and_composition_match() {
        let result = run_self_test();
        assert_eq!(result["self_test_status"].as_str(), Some("PASS"));
        assert_eq!(
            result["audit_report"]["b1_critical_false_pass_count"].as_u64(),
            Some(1)
        );
        assert_eq!(
            result["audit_report"]["b2_critical_false_pass_count"].as_u64(),
            Some(0)
        );
    }

    #[test]
    fn rejects_engine_input_mismatch() {
        let mut manifest = synthetic_self_test_manifest();
        manifest["pairs"][0]["b2"]["engine_input_sha256"] = json!(repeated_digest('7'));
        let report = audit_manifest(&manifest);
        assert_eq!(report["status"].as_str(), Some("FAIL"));
        assert!(has_error(&report, "ENGINE_IO_MISMATCH"));
    }

    #[test]
    fn rejects_future_information() {
        let mut manifest = synthetic_self_test_manifest();
        manifest["pairs"][0]["b2"]["decisions"][0]["evidence"][0]["available_at_offset_s"] =
            json!(99);
        let report = audit_manifest(&manifest);
        assert!(has_error(&report, "FUTURE_INFORMATION_LEAK"));
        assert_eq!(report["future_information_leak_count"].as_u64(), Some(1));
    }

    #[test]
    fn rejects_b2_pass_on_unknown_hard_floor() {
        let mut manifest = synthetic_self_test_manifest();
        manifest["pairs"][1]["b2"]["decisions"][0]["status"] = json!("PASS");
        let report = audit_manifest(&manifest);
        assert!(has_error(&report, "B2_CRITICAL_FALSE_PASS"));
        assert!(has_error(&report, "B2_ORACLE_MISMATCH"));
    }

    #[test]
    fn rejects_b2_receipt_that_does_not_bind_geometry() {
        let mut manifest = synthetic_self_test_manifest();
        manifest["pairs"][0]["b2"]["composition_receipt"]["geometry_sha256"] =
            json!(repeated_digest('8'));
        let report = audit_manifest(&manifest);
        assert!(has_error(&report, "RECEIPT_INPUT_BINDING_MISMATCH"));
    }

    #[test]
    fn rejects_tampered_receipt_payload_even_when_digest_shape_is_valid() {
        let mut manifest = synthetic_self_test_manifest();
        manifest["pairs"][0]["b2"]["composition_receipt"]["source_rights_revision"] =
            json!("tampered-rights-revision");
        let report = audit_manifest(&manifest);
        assert!(has_error(&report, "RECEIPT_DIGEST_MISMATCH"));
    }

    #[test]
    fn rejects_a_luminous_layer_in_b1() {
        let mut manifest = synthetic_self_test_manifest();
        manifest["pairs"][0]["b1"]["composition_layer"] = json!({
            "name": "Luminous evidence composition",
            "version": "0.1.0"
        });
        let report = audit_manifest(&manifest);
        assert!(has_error(&report, "B1_HAS_COMPOSITION_LAYER"));
    }
}
