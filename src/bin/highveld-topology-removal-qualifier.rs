use serde_json::{Value, json};
use std::{env, process};

const MATRIX: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/TOPOLOGY_REMOVAL_MATRIX_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/TOPOLOGY_REMOVAL_MATRIX_V0.1.schema.json"
);

fn string_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn strings_are_unique(values: &[Value]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    values.iter().all(|value| {
        value
            .as_str()
            .is_some_and(|text| !text.trim().is_empty() && seen.insert(text))
    })
}

fn build_plan() -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let matrix: Value = match serde_json::from_str(MATRIX) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("matrix JSON parse failed: {error}")]),
    };
    let schema: Value = match serde_json::from_str(SCHEMA) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("schema JSON parse failed: {error}")]),
    };

    if string_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if string_at(&matrix, "format_version") != Some("0.1") {
        errors.push("format_version must be 0.1".to_owned());
    }
    if string_at(&matrix, "study_id") != Some("HIGHVELD-TOPOLOGY-REMOVAL-001") {
        errors.push("unexpected study_id".to_owned());
    }
    if string_at(&matrix, "parent_candidate") != Some("NEIGHBORHOOD_RESILIENCE_COMMONS") {
        errors.push("unexpected parent_candidate".to_owned());
    }
    if string_at(&matrix, "maturity") != Some("PROTOCOL_ONLY_NOT_EXECUTED") {
        errors.push("maturity escalation: protocol must remain not executed".to_owned());
    }
    if string_at(&matrix, "execution_status") != Some("NOT_EXECUTED") {
        errors.push(
            "execution_status escalation: this tool does not execute the experiment".to_owned(),
        );
    }
    if matrix.get("pr_736_matrix_change").and_then(Value::as_bool) != Some(false) {
        errors.push("the frozen PR #736 matrix must remain unchanged".to_owned());
    }
    if string_at(&matrix, "event_time_and_ids") != Some("MUST_NOT_BE_SYNTHESIZED_BY_THIS_MANIFEST")
    {
        errors.push(
            "event time and IDs must remain owned by the native Symtropy interface".to_owned(),
        );
    }

    let expected_arms = [
        "CENTRAL_HUB",
        "DISTRIBUTED_NODES",
        "FEDERATED_PEER_MESH",
        "MOBILE_SUPPORT",
        "LAYERED_HYBRID",
    ];
    let arms = array_at(&matrix, "topology_arms")
        .cloned()
        .unwrap_or_default();
    if arms.len() != expected_arms.len() || !strings_are_unique(&arms) {
        errors.push("topology_arms must contain five unique, non-empty IDs".to_owned());
    }
    for expected in expected_arms {
        if !arms.iter().any(|arm| arm.as_str() == Some(expected)) {
            errors.push(format!("missing topology arm {expected}"));
        }
    }

    let frozen = array_at(&matrix, "frozen_inputs_required")
        .cloned()
        .unwrap_or_default();
    if frozen.len() < 8 || !strings_are_unique(&frozen) {
        errors.push(
            "frozen_inputs_required must contain at least eight unique input bindings".to_owned(),
        );
    }
    for required in [
        "requirement_set_digest",
        "geometry_access_digest",
        "exogenous_trace_digest",
        "initial_resource_state_digest",
        "independent_service_oracle_id",
        "independent_dependency_oracle_id",
    ] {
        if !frozen.iter().any(|item| item.as_str() == Some(required)) {
            errors.push(format!("missing frozen input binding {required}"));
        }
    }

    let comparisons = array_at(&matrix, "comparison_modes")
        .cloned()
        .unwrap_or_default();
    for required in ["CAPACITY_MATCHED", "BUDGET_MATCHED"] {
        let found = comparisons.iter().any(|item| {
            string_at(item, "id") == Some(required)
                && item.get("required").and_then(Value::as_bool) == Some(true)
        });
        if !found {
            errors.push(format!("required matched comparison {required} is missing"));
        }
    }

    let removals = array_at(&matrix, "removals").cloned().unwrap_or_default();
    if removals.len() < 9 {
        errors.push("at least nine removal families are required".to_owned());
    }
    let removal_ids: Vec<Value> = removals
        .iter()
        .map(|item| item.get("id").cloned().unwrap_or(Value::Null))
        .collect();
    if !strings_are_unique(&removal_ids) {
        errors.push("removal IDs must be unique and non-empty".to_owned());
    }
    for removal in &removals {
        let removal_id = string_at(removal, "id").unwrap_or("<missing>");
        let applies_to = array_at(removal, "applies_to")
            .cloned()
            .unwrap_or_default();
        if applies_to.is_empty() || !strings_are_unique(&applies_to) {
            errors.push(format!(
                "{removal_id}: applies_to must be a non-empty unique list"
            ));
        }
        for arm in &applies_to {
            if !arms.iter().any(|known| known == arm) {
                errors.push(format!(
                    "{removal_id}: unknown topology arm in applies_to: {}",
                    arm.as_str().unwrap_or("<non-string>")
                ));
            }
        }
        if string_at(removal, "target").is_none() || string_at(removal, "kind").is_none() {
            errors.push(format!("{removal_id}: target and kind are required"));
        }
    }

    if string_at(&matrix, "not_applicable_semantics") != Some("EXPLICIT_NOT_APPLICABLE_WITH_REASON")
    {
        errors.push("not-applicable cases must remain explicit and carry a reason".to_owned());
    }
    if string_at(&matrix["adaptation_policy"], "first_pass")
        != Some("STRICT_ABLATION_NO_HIDDEN_RECONFIGURATION")
    {
        errors.push("first pass must be strict ablation without hidden reconfiguration".to_owned());
    }
    if string_at(&matrix["adaptation_policy"], "future_information") != Some("PROHIBITED") {
        errors.push("future-information use must be prohibited".to_owned());
    }

    let gates = &matrix["hard_gates"];
    for gate in [
        "household_floor_false_passes",
        "missing_household_service_outcomes",
        "availability_as_delivery_false_claims",
        "missed_common_causes_against_finite_oracle",
        "duplicate_resource_credits_or_double_commitments",
        "silent_unit_conversions",
        "unauthorized_shared_data_accesses",
        "unknown_or_out_of_domain_as_pass",
        "positive_control_false_blocks",
    ] {
        if gates.get(gate).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("hard gate {gate} must be exactly zero"));
        }
    }
    if gates
        .get("exact_finite_known_answer_agreement_percent")
        .and_then(Value::as_u64)
        != Some(100)
    {
        errors.push("exact finite known-answer agreement must be 100 percent".to_owned());
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut cases = Vec::new();
    let mut eligible = 0_u64;
    let mut not_applicable = 0_u64;
    for arm in &arms {
        let arm_id = arm.as_str().unwrap_or_default();
        for removal in &removals {
            let removal_id = string_at(removal, "id").unwrap_or_default();
            let target = string_at(removal, "target").unwrap_or_default();
            let applies = array_at(removal, "applies_to")
                .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(arm_id)));
            let (state, reason) = if applies {
                eligible += 1;
                (
                    "ELIGIBLE_FOR_TARGET_BINDING",
                    "bind exact component and failure-domain IDs from the frozen topology package before any execution",
                )
            } else {
                not_applicable += 1;
                (
                    "NOT_APPLICABLE",
                    "this removal family is not registered for this topology class; no failure injection is implied",
                )
            };
            cases.push(json!({
                "case_id": format!("{arm_id}__{removal_id}"),
                "topology_arm": arm_id,
                "removal_id": removal_id,
                "target_class": target,
                "case_state": state,
                "reason": reason,
                "execution_status": "NOT_EXECUTED"
            }));
        }
    }

    Ok(json!({
        "audit_state": "STRUCTURAL_CONTRACT_PASS",
        "study_id": string_at(&matrix, "study_id").unwrap_or_default(),
        "maturity": "PROTOCOL_ONLY_NOT_EXECUTED",
        "topology_arm_count": arms.len(),
        "removal_family_count": removals.len(),
        "case_pair_count": cases.len(),
        "eligible_for_target_binding_count": eligible,
        "not_applicable_count": not_applicable,
        "execution_status": "NOT_EXECUTED",
        "schema_engine": "NOT_RUN; schema declaration and manifest semantics are checked by this Rust qualifier",
        "cases": cases
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("--help");
    match mode {
        "--self-test" | "--plan" => match build_plan() {
            Ok(report) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                );
            }
            Err(errors) => {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "audit_state": "STRUCTURAL_CONTRACT_FAIL",
                        "errors": errors,
                        "execution_status": "NOT_EXECUTED"
                    }))
                    .unwrap_or_default()
                );
                process::exit(1);
            }
        },
        "--help" | "-h" => {
            println!("highveld-topology-removal-qualifier");
            println!(
                "  --self-test  validate the preregistered removal manifest and emit case inventory"
            );
            println!(
                "  --plan       emit the same deterministic inventory without implying execution"
            );
        }
        _ => {
            eprintln!("unknown argument; use --help");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_expands_to_complete_five_by_nine_inventory() {
        let report = build_plan().expect("the committed protocol manifest must qualify");
        assert_eq!(report["topology_arm_count"].as_u64(), Some(5));
        assert_eq!(report["removal_family_count"].as_u64(), Some(9));
        assert_eq!(report["case_pair_count"].as_u64(), Some(45));
        assert_eq!(
            report["eligible_for_target_binding_count"].as_u64(),
            Some(33)
        );
        assert_eq!(report["not_applicable_count"].as_u64(), Some(12));
    }

    #[test]
    fn inventory_never_claims_experiment_execution() {
        let report = build_plan().expect("the protocol manifest must qualify");
        assert_eq!(
            report["maturity"].as_str(),
            Some("PROTOCOL_ONLY_NOT_EXECUTED")
        );
        assert_eq!(report["execution_status"].as_str(), Some("NOT_EXECUTED"));
        assert!(
            report["cases"]
                .as_array()
                .expect("cases array")
                .iter()
                .all(|case| case["execution_status"].as_str() == Some("NOT_EXECUTED"))
        );
    }

    #[test]
    fn not_applicable_cases_are_explicit_and_reasoned() {
        let report = build_plan().expect("the protocol manifest must qualify");
        let cases = report["cases"].as_array().expect("cases array");
        assert!(
            cases
                .iter()
                .filter(|case| case["case_state"] == "NOT_APPLICABLE")
                .all(|case| case["reason"]
                    .as_str()
                    .is_some_and(|reason| !reason.trim().is_empty()))
        );
    }

    #[test]
    fn event_clock_and_ids_are_not_synthesized() {
        let matrix: Value = serde_json::from_str(MATRIX).expect("valid matrix JSON");
        assert_eq!(
            string_at(&matrix, "event_time_and_ids"),
            Some("MUST_NOT_BE_SYNTHESIZED_BY_THIS_MANIFEST")
        );
    }
}
