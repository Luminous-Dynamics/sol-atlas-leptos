//! Synthetic cross-owner composition auditor for one CityMaker reference scenario.
//!
//! This package checks that independently produced owner projections refer to
//! the same synthetic scope, asset, design/BOM revision and scenario. It does
//! not recalculate temporal state, canonical impact graphs, restoration
//! feasibility, continuity, commissioning, supply facts or engineering
//! correctness.

use std::collections::BTreeSet;

use serde_json::{Value, json};

const CLAIM_CEILING: &str =
    "synthetic cross-owner receipt-composition semantics only; no real district, asset, service, engineering or professional claim is qualified";

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn records<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn add(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn reason_vec(reasons: BTreeSet<String>) -> Vec<String> {
    reasons.into_iter().collect()
}

fn output(
    case_id: &str,
    scope_state: &str,
    projection_state: &str,
    change_state: &str,
    plan_readiness: &str,
    work_execution: &str,
    inspection_state: &str,
    commissioning_state: &str,
    service_state: &str,
    disposition: &str,
    reasons: BTreeSet<String>,
) -> Value {
    json!({
        "case_id": case_id,
        "scope_state": scope_state,
        "projection_state": projection_state,
        "change_state": change_state,
        "plan_readiness": plan_readiness,
        "work_execution": work_execution,
        "inspection_state": inspection_state,
        "commissioning_state": commissioning_state,
        "service_state": service_state,
        "composition_disposition": disposition,
        "reason_codes": reason_vec(reasons),
        "action_authority": "none",
        "causal_effect_established": false,
        "claim_ceiling": CLAIM_CEILING
    })
}

fn receipt_valid(projection: &Value) -> bool {
    text(projection, "receipt_state") == "qualified"
        && text(projection, "evidence_class") == "synthetic_projection_fixture"
        && !text(projection, "owner").is_empty()
        && !text(projection, "receipt_id").is_empty()
        && is_hex(text(projection, "source_commit"), 40)
        && is_hex(text(projection, "corpus_sha256"), 64)
        && !text(projection, "source_case_id").is_empty()
}

fn revalidation_valid(
    finding: &Value,
    scope: &Value,
    event_id: &str,
    report_id: &str,
) -> bool {
    let receipt = finding.get("revalidation").unwrap_or(&Value::Null);
    let trusted_at = number(receipt, "trusted_at");
    let evaluation_at = number(scope, "evaluation_time");

    text(receipt, "state") == "qualified"
        && !text(receipt, "receipt_id").is_empty()
        && text(receipt, "claim_id") == text(finding, "claim_id")
        && text(receipt, "scope_digest") == text(scope, "scope_digest")
        && text(receipt, "asset_id") == text(scope, "asset_id")
        && text(receipt, "design_revision") == text(scope, "design_revision")
        && text(receipt, "event_id") == event_id
        && text(receipt, "report_id") == report_id
        && trusted_at.is_some()
        && evaluation_at.is_some()
        && trusted_at.is_some_and(|trusted| evaluation_at.is_some_and(|at| trusted <= at))
}

fn evaluate(case: &Value) -> Value {
    let case_id = text(case, "id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let scope = input.get("scope").unwrap_or(&Value::Null);
    let projections = input.get("projections").unwrap_or(&Value::Null);
    let state = projections.get("state_sync").unwrap_or(&Value::Null);
    let impact = projections.get("change_impact").unwrap_or(&Value::Null);
    let restoration = projections.get("restoration").unwrap_or(&Value::Null);
    let continuity = projections.get("continuity").unwrap_or(&Value::Null);

    let mut reasons = BTreeSet::<String>::new();
    let mut scope_state = "bound";
    let mut projection_state = "qualified";
    let mut change_state = "not_required";
    let mut plan_readiness = text(restoration, "plan_readiness").to_owned();
    let mut work_execution = text(restoration, "work_execution").to_owned();
    let mut inspection_state = text(restoration, "inspection_state").to_owned();
    let mut commissioning_state = text(restoration, "commissioning_state").to_owned();
    let mut service_state = text(continuity, "service_state").to_owned();

    if text(input, "classification") != "synthetic_fixture_only"
        || text(scope, "scope_id").is_empty()
        || !is_hex(text(scope, "scope_digest"), 64)
        || text(scope, "scenario_id").is_empty()
        || text(scope, "time_basis") != "UTC_UNIX_SECONDS"
        || number(scope, "evaluation_time").is_none()
        || text(scope, "asset_id").is_empty()
        || !is_hex(text(scope, "subject_digest"), 64)
        || text(scope, "design_revision").is_empty()
        || text(scope, "bom_revision").is_empty()
        || text(scope, "service_id").is_empty()
    {
        add(&mut reasons, "COMMON_SCOPE_BINDING_INCOMPLETE");
        return output(
            case_id, "unknown", "insufficient_evidence", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "blocked", reasons,
        );
    }

    let scope_projections = [state, impact, restoration, continuity];
    for projection in scope_projections {
        if text(projection, "scope_id") != text(scope, "scope_id")
            || text(projection, "scope_digest") != text(scope, "scope_digest")
            || text(projection, "scenario_id") != text(scope, "scenario_id")
        {
            add(&mut reasons, "SCOPE_DIGEST_OR_SCENARIO_MISMATCH");
            scope_state = "not_comparable";
        }
    }
    if scope_state == "not_comparable" {
        return output(
            case_id, scope_state, "not_comparable", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "not_comparable", reasons,
        );
    }

    let source_projections = [state, impact, restoration, continuity];
    let mut receipt_ids = BTreeSet::new();
    for projection in source_projections {
        if !receipt_valid(projection) {
            add(&mut reasons, "UPSTREAM_RECEIPT_NOT_QUALIFIED");
            projection_state = "insufficient_evidence";
        }
        let receipt_id = text(projection, "receipt_id");
        if !receipt_id.is_empty() && !receipt_ids.insert(receipt_id.to_owned()) {
            add(&mut reasons, "DUPLICATE_OWNER_PROJECTION_RECEIPT");
            projection_state = "invalid_evidence";
        }
    }
    if projection_state == "invalid_evidence" {
        return output(
            case_id, scope_state, projection_state, "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "invalid_evidence", reasons,
        );
    }
    if projection_state != "qualified" {
        return output(
            case_id, scope_state, projection_state, "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "blocked", reasons,
        );
    }

    if text(state, "asset_id") != text(scope, "asset_id")
        || text(state, "subject_digest") != text(scope, "subject_digest")
        || text(restoration, "asset_id") != text(scope, "asset_id")
        || text(continuity, "service_id") != text(scope, "service_id")
    {
        add(&mut reasons, "OWNER_ASSET_OR_SERVICE_IDENTITY_MISMATCH");
        return output(
            case_id, "not_comparable", "qualified", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "not_comparable", reasons,
        );
    }

    if text(restoration, "design_revision") != text(scope, "design_revision")
        || text(restoration, "bom_revision") != text(scope, "bom_revision")
        || text(state, "source_revision") != text(scope, "design_revision")
    {
        add(&mut reasons, "DESIGN_OR_BOM_REVISION_MISMATCH");
        return output(
            case_id, "not_comparable", "qualified", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "not_comparable", reasons,
        );
    }

    let evaluation_time = number(scope, "evaluation_time").unwrap_or(0);
    let (valid_at, recorded_at, ingested_at) = (
        number(state, "valid_at"),
        number(state, "recorded_at"),
        number(state, "ingested_at"),
    );
    if !matches!((valid_at, recorded_at, ingested_at), (Some(valid), Some(recorded), Some(ingested)) if valid <= recorded && recorded <= ingested && ingested <= evaluation_time)
    {
        add(&mut reasons, "STATE_PROJECTION_TIME_CUT_INVALID");
        return output(
            case_id, scope_state, "invalid_evidence", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "invalid_evidence", reasons,
        );
    }
    if text(state, "state") != "Current" {
        add(
            &mut reasons,
            match text(state, "state") {
                "Revoked" => "OWNER_ASSET_STATE_REVOKED",
                "Disputed" => "OWNER_ASSET_STATE_DISPUTED",
                "Stale" => "OWNER_ASSET_STATE_STALE",
                _ => "OWNER_ASSET_STATE_NOT_CURRENT",
            },
        );
        return output(
            case_id, scope_state, "qualified", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "blocked", reasons,
        );
    }

    if text(impact, "coverage_state") != "complete"
        || text(impact, "report_state") != "qualified"
        || !is_hex(text(impact, "graph_snapshot_digest"), 64)
        || text(impact, "report_id").is_empty()
    {
        add(&mut reasons, "CHANGE_IMPACT_COVERAGE_INCOMPLETE");
        return output(
            case_id, scope_state, "insufficient_evidence", "unknown", "unknown",
            "unknown", "unknown", "unknown", "unknown", "blocked", reasons,
        );
    }

    let required_claims = [
        "claim_panel_fit",
        "claim_panel_manufacture",
        "claim_repair_plan",
    ];
    let findings = records(impact, "claim_findings");
    let mut seen_claims = BTreeSet::new();
    let mut any_revalidated = false;

    for claim_id in required_claims {
        let Some(finding) = findings.iter().find(|finding| text(finding, "claim_id") == claim_id) else {
            add(&mut reasons, "REQUIRED_DEPENDENCY_CLAIM_MISSING");
            change_state = "insufficient_evidence";
            break;
        };
        if !seen_claims.insert(claim_id) {
            add(&mut reasons, "DUPLICATE_DEPENDENCY_CLAIM");
            change_state = "invalid_evidence";
            break;
        }

        match text(finding, "disposition") {
            "not_affected" => {
                if text(finding, "coverage_proof_id").is_empty() {
                    add(&mut reasons, "NO_IMPACT_COVERAGE_PROOF_MISSING");
                    change_state = "insufficient_evidence";
                    break;
                }
            }
            "revalidation_required" => {
                let path = records(finding, "dependency_path");
                let path_ok = path.len() >= 2
                    && path.first().and_then(Value::as_str) == Some(text(scope, "asset_id"))
                    && path.last().and_then(Value::as_str) == Some(claim_id);
                if !path_ok {
                    add(&mut reasons, "DEPENDENCY_PATH_MISMATCH");
                    change_state = "invalid_evidence";
                    break;
                }
                if !revalidation_valid(
                    finding,
                    scope,
                    text(impact, "event_id"),
                    text(impact, "report_id"),
                ) {
                    add(&mut reasons, "CLAIM_REVALIDATION_REQUIRED");
                    change_state = "revalidation_required";
                } else {
                    any_revalidated = true;
                }
            }
            "invalid_evidence" => {
                add(&mut reasons, "DEPENDENT_CLAIM_INVALIDATED");
                change_state = "invalid_evidence";
                break;
            }
            "insufficient_evidence" => {
                add(&mut reasons, "DEPENDENT_CLAIM_INSUFFICIENT");
                change_state = "insufficient_evidence";
                break;
            }
            "applicable_current" => {}
            _ => {
                add(&mut reasons, "IMPACT_DISPOSITION_UNSUPPORTED");
                change_state = "invalid_evidence";
                break;
            }
        }
    }

    if change_state == "invalid_evidence" || change_state == "insufficient_evidence" {
        return output(
            case_id, scope_state, change_state, change_state, "unknown",
            "unknown", "unknown", "unknown", "unknown", "blocked", reasons,
        );
    }
    if change_state == "revalidation_required" {
        return output(
            case_id, scope_state, "qualified", change_state, "unknown",
            "unknown", "unknown", "unknown", "unknown", "blocked", reasons,
        );
    }
    if any_revalidated {
        change_state = "revalidated";
    }

    plan_readiness = text(restoration, "plan_readiness").to_owned();
    work_execution = text(restoration, "work_execution").to_owned();
    inspection_state = text(restoration, "inspection_state").to_owned();
    commissioning_state = text(restoration, "commissioning_state").to_owned();

    if text(restoration, "action_authority") != "none" {
        add(&mut reasons, "RESTORATION_PROJECTION_CLAIMS_ACTION_AUTHORITY");
        return output(
            case_id, scope_state, "invalid_evidence", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "invalid_evidence", reasons,
        );
    }
    if text(restoration, "payload_bytes_verified") != "true" {
        add(&mut reasons, "PAYLOAD_BYTES_UNVERIFIED");
        projection_state = "insufficient_evidence";
        return output(
            case_id, scope_state, projection_state, change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "blocked", reasons,
        );
    }

    match text(restoration, "lot_state") {
        "revoked" => {
            add(&mut reasons, "MATERIAL_LOT_REVOKED");
            plan_readiness = "infeasible".to_owned();
        }
        "qualified" => {}
        _ => {
            add(&mut reasons, "MATERIAL_LOT_QUALIFICATION_UNKNOWN");
            plan_readiness = "unknown".to_owned();
        }
    }
    if text(restoration, "custody_state") != "accepted" {
        add(&mut reasons, "MATERIAL_CUSTODY_NOT_ACCEPTED");
        if plan_readiness == "feasible" {
            plan_readiness = "unknown".to_owned();
        }
    }

    if plan_readiness == "infeasible" {
        return output(
            case_id, scope_state, "qualified", change_state, "infeasible",
            work_execution, inspection_state, commissioning_state, "unknown",
            "infeasible", reasons,
        );
    }
    if plan_readiness != "feasible" {
        add(&mut reasons, "RESTORATION_PLAN_NOT_FEASIBLE_EVIDENCED");
        return output(
            case_id, scope_state, "insufficient_evidence", change_state, "unknown",
            work_execution, inspection_state, commissioning_state, "unknown",
            "blocked", reasons,
        );
    }

    if !matches!(
        work_execution,
        "not_started" | "in_progress" | "failed" | "completed" | "unknown"
    ) || !matches!(
        inspection_state,
        "not_performed" | "pass" | "fail" | "unknown"
    ) || !matches!(
        commissioning_state,
        "not_performed" | "pass" | "fail" | "unknown"
    ) {
        add(&mut reasons, "LIFECYCLE_AXIS_VALUE_UNSUPPORTED");
        return output(
            case_id, scope_state, "invalid_evidence", change_state, plan_readiness,
            "unknown", "unknown", "unknown", "unknown", "invalid_evidence", reasons,
        );
    }

    if work_execution == "failed" {
        add(&mut reasons, "WORK_EXECUTION_FAILED");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "blocked", reasons,
        );
    }
    if work_execution == "in_progress" {
        add(&mut reasons, "WORK_EXECUTION_IN_PROGRESS");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "work_in_progress", reasons,
        );
    }
    if work_execution != "completed" {
        service_state = text(continuity, "service_state").to_owned();
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, service_state,
            "review_candidate", reasons,
        );
    }

    if inspection_state == "fail" {
        add(&mut reasons, "INDEPENDENT_INSPECTION_FAILED");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "blocked", reasons,
        );
    }
    if inspection_state != "pass" {
        add(&mut reasons, "INDEPENDENT_INSPECTION_PENDING");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "awaiting_inspection", reasons,
        );
    }

    if commissioning_state == "fail" {
        add(&mut reasons, "COMMISSIONING_FAILED");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "blocked", reasons,
        );
    }
    if commissioning_state != "pass" {
        add(&mut reasons, "COMMISSIONING_PENDING");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "unknown",
            "awaiting_commissioning", reasons,
        );
    }

    service_state = text(continuity, "service_state").to_owned();
    if text(continuity, "continuity_receipt_state") != "current"
        || service_state == "unknown"
    {
        add(&mut reasons, "CONTINUITY_SERVICE_EVIDENCE_UNKNOWN");
        service_state = "unknown".to_owned();
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, service_state,
            "awaiting_service_confirmation", reasons,
        );
    }
    if service_state == "adequate" {
        if text(scope, "service_restoration_required") == "true"
            && bool_field(continuity, "stable_adequacy_met") != Some(true)
        {
            add(&mut reasons, "STABLE_ADEQUACY_INTERVAL_NOT_MET");
            service_state = "unknown".to_owned();
            return output(
                case_id, scope_state, "qualified", change_state, plan_readiness,
                work_execution, inspection_state, commissioning_state, service_state,
                "awaiting_service_confirmation", reasons,
            );
        }
        if commissioning_state != "pass" {
            add(&mut reasons, "SERVICE_ADEQUACY_WITHOUT_COMMISSIONING");
            return output(
                case_id, scope_state, "invalid_evidence", change_state, plan_readiness,
                work_execution, inspection_state, commissioning_state, "unknown",
                "invalid_evidence", reasons,
            );
        }
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, "adequate",
            "service_owner_confirmed_adequate", reasons,
        );
    }

    if service_state == "constrained" || service_state == "inaccessible" {
        add(&mut reasons, "CONTINUITY_OWNER_REPORTS_SERVICE_GAP");
        return output(
            case_id, scope_state, "qualified", change_state, plan_readiness,
            work_execution, inspection_state, commissioning_state, service_state,
            "service_owner_reports_constrained", reasons,
        );
    }

    add(&mut reasons, "CONTINUITY_SERVICE_STATE_UNSUPPORTED");
    output(
        case_id, scope_state, "invalid_evidence", change_state, plan_readiness,
        work_execution, inspection_state, commissioning_state, "unknown",
        "invalid_evidence", reasons,
    )
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

/// Evaluate a composed scenario using owner projection fixtures only.
pub fn evaluate_case(case: &Value) -> Value {
    evaluate(case)
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            actual.len() == expected.len()
                && actual.iter().all(|(key, value)| {
                    expected
                        .get(key)
                        .is_some_and(|expected_value| matches_expected(value, expected_value))
                })
        }
        (Value::Array(actual), Value::Array(expected)) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected)
                    .all(|(left, right)| matches_expected(left, right))
        }
        _ => actual == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn corpus() -> Value {
        let source = include_str!(
            "../../../docs/research/city-composed-reference-scenario/CITY_COMPOSED_REFERENCE_SCENARIO_CORPUS_V0_1.json"
        );
        serde_json::from_str(source).expect("composed scenario corpus parses")
    }

    #[test]
    fn every_composed_reference_scenario_matches_known_answers() {
        let corpus = corpus();
        let cases = corpus["cases"].as_array().expect("cases array");
        assert_eq!(cases.len(), 28);
        for case in cases {
            let actual = evaluate_case(case);
            assert!(
                matches_expected(&actual, &case["expected"]),
                "{}: actual={} expected={}",
                text(case, "id"),
                actual,
                case["expected"]
            );
        }
    }

    #[test]
    fn hard_material_conflict_cannot_be_compensated_by_positive_receipts() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["projections"]["restoration"]["lot_state"] = json!("revoked");
        case["input"]["projections"]["restoration"]["payload_bytes_verified"] = json!(false);
        let actual = evaluate_case(&case);
        assert_eq!(actual["plan_readiness"], "infeasible");
        assert_eq!(actual["composition_disposition"], "infeasible");
        assert!(actual["reason_codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "MATERIAL_LOT_REVOKED"));
    }

    #[test]
    fn plan_feasibility_never_promotes_work_or_restoration() {
        let actual = evaluate_case(&corpus()["cases"][0]);
        assert_eq!(actual["plan_readiness"], "feasible");
        assert_eq!(actual["work_execution"], "not_started");
        assert_eq!(actual["inspection_state"], "not_performed");
        assert_eq!(actual["commissioning_state"], "not_performed");
        assert_eq!(actual["service_state"], "unknown");
        assert_eq!(actual["composition_disposition"], "review_candidate");
        assert_eq!(actual["action_authority"], "none");
    }

    #[test]
    fn a_revalidation_receipt_for_one_claim_does_not_discharge_other_claims() {
        let mut case = corpus()["cases"][9].clone();
        let missing = &mut case["input"]["projections"]["change_impact"]["claim_findings"][1];
        missing["disposition"] = json!("revalidation_required");
        missing["revalidation"] = Value::Null;
        let actual = evaluate_case(&case);
        assert_eq!(actual["change_state"], "revalidation_required");
        assert_eq!(actual["composition_disposition"], "blocked");
    }

    #[test]
    fn continuity_owner_alone_can_confirm_stable_service() {
        let actual = evaluate_case(&corpus()["cases"][20]);
        assert_eq!(actual["commissioning_state"], "pass");
        assert_eq!(actual["service_state"], "adequate");
        assert_eq!(
            actual["composition_disposition"],
            "service_owner_confirmed_adequate"
        );
        assert_eq!(actual["causal_effect_established"], false);
    }
}
