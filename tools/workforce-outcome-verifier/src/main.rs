use jsonschema::validator_for;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const FIXTURE_DEFAULT: &str = "docs/research/fixtures/workforce_outcome_receipts_v0_1.json";
const SCHEMA_DEFAULT: &str = "docs/research/schemas/workforce_outcome_receipt_v0_1.schema.json";
const RECEIPT_SCHEMA_VERSION: &str = "workforce-outcome-receipt-v0.1";

#[derive(Debug, Serialize, PartialEq)]
struct ReceiptCheck {
    run_id: String,
    policy_count: usize,
    additionality_policy_id: String,
    gross_paid_work_events: u64,
    policy_paid_work_verified_count: u64,
    passed: bool,
    violations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    schema_version: &'static str,
    fixture_sha256: String,
    schema_sha256: String,
    receipt_count: usize,
    receipts_passed: usize,
    receipts_failed: usize,
    receipts: Vec<ReceiptCheck>,
    limitations: Vec<&'static str>,
    claim_ceiling: &'static str,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn validate_schema(schema: &Value, fixture: &Value) -> Result<(), String> {
    let validator =
        validator_for(schema).map_err(|error| format!("schema cannot be compiled: {error}"))?;
    let errors: Vec<String> = validator
        .iter_errors(fixture)
        .map(|error| error.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "JSON Schema validation failed:\n{}",
            errors.join("\n")
        ))
    }
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn count_at(value: &Value, field: &str) -> Option<u64> {
    value.get(field)?.as_u64()
}

fn effect_status_is_not_assessed(additionality: &Value) -> bool {
    [
        "displacement_status",
        "substitution_status",
        "deadweight_status",
        "spillover_status",
    ]
    .iter()
    .all(|field| additionality.get(*field).and_then(Value::as_str) == Some("not_assessed"))
}

fn validate_receipt_contract(receipt: &Value) -> Vec<String> {
    let mut violations = BTreeSet::new();

    let run_id = receipt
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if run_id.is_empty() {
        violations.insert("run_id_missing".to_string());
    }

    let synthetic_only = receipt
        .get("synthetic_only")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if synthetic_only {
        if receipt.get("evidence_class").and_then(Value::as_str) != Some("synthetic")
            || receipt.get("evaluation_design").and_then(Value::as_str)
                != Some("synthetic_contract")
            || receipt.get("claim_ceiling").and_then(Value::as_str)
                != Some("synthetic_contract_only")
        {
            violations.insert("synthetic_claim_ceiling_invalid".to_string());
        }
    }

    if let (Some(start), Some(end)) = (
        string_at(receipt, &["population", "period_start"]),
        string_at(receipt, &["population", "period_end"]),
    ) {
        if start > end {
            violations.insert("population_period_reversed".to_string());
        }
    } else {
        violations.insert("population_period_missing".to_string());
    }

    let policies = receipt
        .get("policy_results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut policy_ids = BTreeSet::new();

    for policy in &policies {
        let policy_id = policy
            .get("policy_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if policy_id.is_empty() {
            violations.insert("policy_id_missing".to_string());
        }
        if !policy_ids.insert(policy_id.to_string()) {
            violations.insert("duplicate_policy_id".to_string());
        }

        let Some(cases) = count_at(policy, "case_count") else {
            violations.insert("case_count_missing".to_string());
            continue;
        };
        let diagnosed = count_at(policy, "diagnosed_count").unwrap_or(u64::MAX);
        let abstained = count_at(policy, "abstained_count").unwrap_or(u64::MAX);
        let correct = count_at(policy, "correct_cause_set_count").unwrap_or(u64::MAX);
        let missing = count_at(policy, "missing_outcome_count").unwrap_or(u64::MAX);
        let completed = count_at(policy, "action_completed_count").unwrap_or(u64::MAX);
        let restored = count_at(policy, "verified_restoration_count").unwrap_or(u64::MAX);
        let paid = count_at(policy, "paid_work_verified_count").unwrap_or(u64::MAX);
        let false_shortage = count_at(policy, "false_shortage_count").unwrap_or(u64::MAX);
        let false_surplus = count_at(policy, "false_surplus_count").unwrap_or(u64::MAX);

        if diagnosed > cases || abstained > cases || diagnosed.saturating_add(abstained) > cases {
            violations.insert(format!("diagnosed_abstained_exceed_cases:{policy_id}"));
        }
        if correct > diagnosed {
            violations.insert(format!("correct_cause_sets_exceed_diagnosed:{policy_id}"));
        }
        if missing > cases {
            violations.insert(format!("missing_outcomes_exceed_cases:{policy_id}"));
        }
        if false_shortage > cases || false_surplus > cases {
            violations.insert(format!("false_diagnoses_exceed_cases:{policy_id}"));
        }
        if restored > completed {
            violations.insert(format!(
                "verified_restoration_exceeds_completed_actions:{policy_id}"
            ));
        }
        if paid > completed {
            violations.insert(format!("paid_work_exceeds_completed_actions:{policy_id}"));
        }
    }

    let additionality = receipt
        .get("additionality_assessment")
        .unwrap_or(&Value::Null);
    let additionality_policy_id = additionality
        .get("policy_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let gross_events = additionality
        .get("gross_paid_work_events")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
    let selected_policy = policies.iter().find(|policy| {
        policy.get("policy_id").and_then(Value::as_str) == Some(additionality_policy_id)
    });

    let mut linked_paid_count = u64::MAX;
    match selected_policy {
        Some(policy) => {
            linked_paid_count = count_at(policy, "paid_work_verified_count").unwrap_or(u64::MAX);
            if linked_paid_count != gross_events {
                violations.insert("gross_paid_work_count_mismatch".to_string());
            }
        }
        None => {
            violations.insert("additionality_policy_not_found".to_string());
        }
    }

    let status = additionality
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let estimate = additionality
        .get("net_additional_paid_hours_estimate")
        .and_then(Value::as_f64);
    let interval = additionality.get("net_additional_interval_hours");
    let design = additionality
        .get("comparison_design")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let counterfactual = additionality
        .get("counterfactual_basis")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let preregistration = additionality
        .get("preregistration_status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let independent_review = additionality
        .get("independent_review")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let additionality_claim = additionality
        .get("claim_ceiling")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let top_claim = receipt
        .get("claim_ceiling")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match status {
        "not_assessed" => {
            if !additionality
                .get("net_additional_paid_hours_estimate")
                .is_some_and(Value::is_null)
                || !interval.is_some_and(Value::is_null)
                || design != "none"
                || !counterfactual.is_empty()
                || preregistration != "not_registered"
                || independent_review
                || !effect_status_is_not_assessed(additionality)
            {
                violations.insert("not_assessed_additionality_has_effect_claim".to_string());
            }
            if additionality_claim == "causal_claim_supported_by_preregistered_design"
                || top_claim == "causal_claim_supported_by_preregistered_design"
            {
                violations.insert("causal_claim_without_additionality_estimate".to_string());
            }
        }
        "descriptive_only" => {
            if additionality_claim == "causal_claim_supported_by_preregistered_design"
                || top_claim == "causal_claim_supported_by_preregistered_design"
            {
                violations.insert("descriptive_additionality_promoted_to_causal".to_string());
            }
        }
        "estimated_with_preregistered_counterfactual" => {
            if synthetic_only
                || receipt.get("evidence_class").and_then(Value::as_str) == Some("synthetic")
            {
                violations.insert("synthetic_additionality_estimate_forbidden".to_string());
            }
            if estimate.is_none() || !interval.is_some_and(Value::is_object) {
                violations.insert("additionality_estimate_or_interval_missing".to_string());
            } else if let Some(bounds) = interval.and_then(|value| {
                Some((value.get("lower")?.as_f64()?, value.get("upper")?.as_f64()?))
            }) {
                let (lower, upper) = bounds;
                if lower > upper || estimate.is_some_and(|value| value < lower || value > upper) {
                    violations.insert("additionality_interval_invalid".to_string());
                }
            } else {
                violations.insert("additionality_interval_invalid".to_string());
            }
            if design == "none"
                || counterfactual.trim().is_empty()
                || preregistration != "verified"
                || !independent_review
            {
                violations.insert("additionality_counterfactual_gate_missing".to_string());
            }
            if !effect_status_is_not_assessed(additionality) {
                // Each effect must be assessed, bounded, or explicitly unresolved;
                // "not_assessed" is the only disallowed state in this branch.
                for field in [
                    "displacement_status",
                    "substitution_status",
                    "deadweight_status",
                    "spillover_status",
                ] {
                    if additionality.get(field).and_then(Value::as_str) == Some("not_assessed") {
                        violations.insert(format!("additionality_effect_not_assessed:{field}"));
                    }
                }
            } else {
                violations.insert("additionality_effects_all_not_assessed".to_string());
            }
            if additionality_claim != "causal_claim_supported_by_preregistered_design" {
                violations.insert("additionality_claim_ceiling_mismatch".to_string());
            }
        }
        _ => {
            violations.insert("additionality_status_unsupported".to_string());
        }
    }

    if top_claim == "causal_claim_supported_by_preregistered_design"
        && status != "estimated_with_preregistered_counterfactual"
    {
        violations.insert("causal_claim_without_net_additionality".to_string());
    }

    violations.into_iter().collect()
}

fn verify_values(
    schema: &Value,
    fixture: &Value,
) -> Result<(Vec<ReceiptCheck>, Vec<String>), String> {
    if !fixture
        .get("fixture_notice")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_uppercase()
        .contains("ALL VALUES BELOW ARE SYNTHETIC CONTRACT TEST DATA")
    {
        return Err("fixture notice does not establish synthetic-only scope".to_string());
    }

    let receipts = fixture
        .get("receipts")
        .and_then(Value::as_array)
        .ok_or_else(|| "receipts array missing".to_string())?;
    let mut summaries = Vec::new();
    let mut errors = Vec::new();
    let mut run_ids = BTreeSet::new();

    for receipt in receipts {
        // The schema describes each receipt, not the outer fixture envelope.
        validate_schema(schema, receipt)?;
        let violations = validate_receipt_contract(receipt);
        let run_id = receipt
            .get("run_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if !run_ids.insert(run_id.clone()) {
            errors.push(format!("{run_id}: duplicate run_id"));
        }
        let additionality = receipt
            .get("additionality_assessment")
            .unwrap_or(&Value::Null);
        let additionality_policy_id = additionality
            .get("policy_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let gross_paid_work_events = additionality
            .get("gross_paid_work_events")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let policy_paid_work_verified_count = receipt
            .get("policy_results")
            .and_then(Value::as_array)
            .and_then(|policies| {
                policies.iter().find(|policy| {
                    policy.get("policy_id").and_then(Value::as_str)
                        == Some(additionality_policy_id.as_str())
                })
            })
            .and_then(|policy| policy.get("paid_work_verified_count"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let policy_count = receipt
            .get("policy_results")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0);
        let passed = violations.is_empty();
        if !passed {
            errors.push(format!("{run_id}: {}", violations.join(",")));
        }
        summaries.push(ReceiptCheck {
            run_id,
            policy_count,
            additionality_policy_id,
            gross_paid_work_events,
            policy_paid_work_verified_count,
            passed,
            violations,
        });
    }

    summaries.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    errors.sort();
    Ok((summaries, errors))
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let fixture_path = args.next().unwrap_or_else(|| FIXTURE_DEFAULT.to_string());
    let schema_path = args.next().unwrap_or_else(|| SCHEMA_DEFAULT.to_string());

    let fixture_bytes =
        fs::read(&fixture_path).map_err(|error| format!("cannot read {fixture_path}: {error}"))?;
    let schema_bytes =
        fs::read(&schema_path).map_err(|error| format!("cannot read {schema_path}: {error}"))?;
    let fixture: Value = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("fixture JSON is invalid: {error}"))?;
    let schema: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;

    let (receipts, errors) = verify_values(&schema, &fixture)?;
    let receipts_passed = receipts.iter().filter(|receipt| receipt.passed).count();
    let receipts_failed = receipts.len() - receipts_passed;
    let report = Report {
        status: if errors.is_empty() && receipts_failed == 0 {
            "SYNTHETIC_OUTCOME_RECEIPT_PASS"
        } else {
            "SYNTHETIC_OUTCOME_RECEIPT_FAIL"
        },
        schema_version: RECEIPT_SCHEMA_VERSION,
        fixture_sha256: sha256(&fixture_bytes),
        schema_sha256: sha256(&schema_bytes),
        receipt_count: receipts.len(),
        receipts_passed,
        receipts_failed,
        receipts,
        limitations: vec![
            "The fixture policy counts and all outcomes are synthetic contract inputs.",
            "Gross paid-work events are reconciled to their selected policy; they are not net additional jobs.",
            "The synthetic-only contract intentionally leaves counterfactual additionality unestimated.",
            "A verifier pass does not validate real sources, establish causality, or demonstrate SOTA superiority.",
        ],
        claim_ceiling: "synthetic_contract_only",
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("receipt serialization failed: {error}"))?
    );
    if report.receipts_failed > 0 || !errors.is_empty() {
        return Err(format!(
            "outcome receipt verification failed: {}",
            errors.join("; ")
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../docs/research/fixtures/workforce_outcome_receipts_v0_1.json");
    const SCHEMA: &str =
        include_str!("../../../docs/research/schemas/workforce_outcome_receipt_v0_1.schema.json");

    fn loaded() -> (Value, Value) {
        (
            serde_json::from_str(FIXTURE).expect("fixture parses"),
            serde_json::from_str(SCHEMA).expect("schema parses"),
        )
    }

    #[test]
    fn schema_and_synthetic_outcome_receipt_pass() {
        let (fixture, schema) = loaded();
        let (receipts, errors) = verify_values(&schema, &fixture).expect("fixture verifies");
        assert!(errors.is_empty());
        assert_eq!(receipts.len(), 1);
        assert!(receipts[0].passed);
        assert_eq!(receipts[0].gross_paid_work_events, 4);
        assert_eq!(receipts[0].policy_paid_work_verified_count, 4);
        assert_eq!(receipts[0].additionality_policy_id, "candidate");
    }

    #[test]
    fn gross_paid_work_must_reconcile_to_the_named_policy() {
        let (mut fixture, schema) = loaded();
        fixture["receipts"][0]["additionality_assessment"]["gross_paid_work_events"] = json!(3);
        let (receipts, _) = verify_values(&schema, &fixture).expect("schema still valid");
        assert!(
            receipts[0]
                .violations
                .contains(&"gross_paid_work_count_mismatch".to_string())
        );
    }

    #[test]
    fn diagnosed_and_abstained_counts_cannot_exceed_cases() {
        let (mut fixture, schema) = loaded();
        fixture["receipts"][0]["policy_results"][2]["diagnosed_count"] = json!(20);
        fixture["receipts"][0]["policy_results"][2]["abstained_count"] = json!(6);
        let (receipts, _) = verify_values(&schema, &fixture).expect("schema still valid");
        assert!(
            receipts[0]
                .violations
                .iter()
                .any(|code| code.starts_with("diagnosed_abstained_exceed_cases:"))
        );
    }

    #[test]
    fn verified_restoration_and_paid_work_cannot_exceed_completed_actions() {
        let (mut fixture, schema) = loaded();
        fixture["receipts"][0]["policy_results"][2]["action_completed_count"] = json!(2);
        let (receipts, _) = verify_values(&schema, &fixture).expect("schema still valid");
        assert!(
            receipts[0]
                .violations
                .iter()
                .any(|code| code.starts_with("verified_restoration_exceeds_completed_actions:"))
        );
        assert!(
            receipts[0]
                .violations
                .iter()
                .any(|code| code.starts_with("paid_work_exceeds_completed_actions:"))
        );
    }

    #[test]
    fn policy_ids_must_be_unique() {
        let (mut fixture, schema) = loaded();
        fixture["receipts"][0]["policy_results"][1]["policy_id"] = json!("candidate");
        let (receipts, _) = verify_values(&schema, &fixture).expect("schema still valid");
        assert!(
            receipts[0]
                .violations
                .contains(&"duplicate_policy_id".to_string())
        );
    }

    #[test]
    fn unassessed_additionality_cannot_emit_estimate_or_causal_claim() {
        let (mut fixture, schema) = loaded();
        fixture["receipts"][0]["additionality_assessment"]["claim_ceiling"] =
            json!("causal_claim_supported_by_preregistered_design");
        assert!(verify_values(&schema, &fixture).is_err());
    }

    #[test]
    fn additionality_assessment_can_be_causal_while_overall_claim_is_lower() {
        let (fixture, _schema) = loaded();
        let mut receipt = fixture["receipts"][0].clone();
        receipt["synthetic_only"] = Value::Bool(false);
        receipt["evidence_class"] = json!("authorized_retrospective");
        receipt["evaluation_design"] = json!("matched_comparison");
        receipt["claim_ceiling"] = json!("descriptive_association_only");
        receipt["additionality_assessment"]["status"] =
            json!("estimated_with_preregistered_counterfactual");
        receipt["additionality_assessment"]["net_additional_paid_hours_estimate"] = json!(100.0);
        receipt["additionality_assessment"]["net_additional_interval_hours"] =
            json!({"lower": 50.0, "upper": 120.0});
        receipt["additionality_assessment"]["displacement_status"] =
            json!("no_material_displacement_evidence_with_limits");
        receipt["additionality_assessment"]["substitution_status"] = json!("possible_substitution");
        receipt["additionality_assessment"]["deadweight_status"] =
            json!("no_material_deadweight_evidence_with_limits");
        receipt["additionality_assessment"]["spillover_status"] = json!("unresolved");
        receipt["additionality_assessment"]["comparison_design"] = json!("matched_comparison");
        receipt["additionality_assessment"]["counterfactual_basis"] =
            json!("Synthetic unit test counterfactual definition.");
        receipt["additionality_assessment"]["preregistration_status"] = json!("verified");
        receipt["additionality_assessment"]["independent_review"] = Value::Bool(true);
        receipt["additionality_assessment"]["claim_ceiling"] =
            json!("causal_claim_supported_by_preregistered_design");
        assert!(validate_receipt_contract(&receipt).is_empty());
    }

    #[test]
    fn input_order_permutation_preserves_semantic_checks() {
        let (fixture, schema) = loaded();
        let (original, _) = verify_values(&schema, &fixture).expect("original verifies");
        let mut permuted = fixture.clone();
        permuted["receipts"][0]["policy_results"]
            .as_array_mut()
            .expect("policies array")
            .reverse();
        let (reordered, _) = verify_values(&schema, &permuted).expect("permuted verifies");
        assert_eq!(original, reordered);
    }
}
