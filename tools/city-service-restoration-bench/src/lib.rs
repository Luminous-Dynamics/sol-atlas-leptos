//! Synthetic-first, read-only restoration-plan feasibility oracle.
//!
//! The evaluator composes owner-provided facts. It does not schedule crews,
//! allocate stock, dispatch deliveries, calculate continuity timelines,
//! authorize isolation/work, or commission equipment.

use std::collections::BTreeSet;

use serde_json::{Value, json};

const CLAIM_CEILING: &str = "synthetic restoration-feasibility and lifecycle-separation semantics only; no real resource, safety, engineering or service claim is qualified";

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn u64_field(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn records<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn add_unknown(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn add_conflict(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn interval_overlaps(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> bool {
    a_start < b_end && b_start < a_end
}

fn lifecycle_axis(raw: &str, evidence: &str, allowed: &[&str], default: &str) -> String {
    if raw.is_empty() {
        return default.to_owned();
    }
    if raw == "unknown" {
        return "unknown".to_owned();
    }
    if allowed.contains(&raw)
        && (raw == "not_started" || raw == "not_performed" || evidence == "current")
    {
        raw.to_owned()
    } else {
        "unknown".to_owned()
    }
}

fn evaluate_plan(input: &Value, reasons: &mut BTreeSet<String>, conflicts: &mut BTreeSet<String>) {
    if str_field(input, "classification") != "synthetic_fixture_only" {
        add_unknown(reasons, "INPUT_CLASSIFICATION_UNBOUND");
    }
    let as_of = input.get("as_of").unwrap_or(&Value::Null);
    if u64_field(as_of, "valid_at").is_none()
        || u64_field(as_of, "recorded_at").is_none()
        || u64_field(as_of, "ingested_at").is_none()
        || str_field(as_of, "time_basis") != "UTC_UNIX_SECONDS"
        || str_field(as_of, "source_timezone").is_empty()
    {
        add_unknown(reasons, "AS_OF_TIME_BINDING_INCOMPLETE");
    }

    let scope = input.get("scope").unwrap_or(&Value::Null);
    if str_field(scope, "binding_state") != "bound"
        || str_field(scope, "scope_digest").len() != 64
        || str_field(scope, "design_revision").is_empty()
        || str_field(scope, "asset_revision").is_empty()
        || str_field(scope, "functional_unit").is_empty()
    {
        add_unknown(reasons, "COMMON_SCOPE_BINDING_INCOMPLETE");
    }

    let window = input.get("planning_window").unwrap_or(&Value::Null);
    let planning_window = match (u64_field(window, "start_at"), u64_field(window, "end_at")) {
        (Some(start), Some(end)) if start < end
            && str_field(window, "time_basis") == "UTC_UNIX_SECONDS" =>
        {
            Some((start, end))
        }
        (Some(_), Some(_)) => {
            add_unknown(reasons, "PLANNING_WINDOW_INVALID");
            None
        }
        _ => {
            add_unknown(reasons, "PLANNING_WINDOW_MISSING");
            None
        }
    };

    for (field, reason) in [
        ("part_requirements", "PART_REQUIREMENTS_MISSING"),
        ("reservations", "RESOURCE_RESERVATIONS_MISSING"),
        ("deliveries", "DELIVERY_EVIDENCE_MISSING"),
        ("crew_requirements", "CREW_REQUIREMENTS_MISSING"),
        ("tool_requirements", "TOOL_REQUIREMENTS_MISSING"),
        (
            "calibration_requirements",
            "CALIBRATION_REQUIREMENTS_MISSING",
        ),
        ("isolation_requirements", "ISOLATION_REQUIREMENTS_MISSING"),
        ("access_requirements", "ACCESS_REQUIREMENTS_MISSING"),
        ("tolerance_checks", "TOLERANCE_CHECKS_MISSING"),
        ("repair_envelope_checks", "REPAIR_ENVELOPE_CHECKS_MISSING"),
        ("payload_checks", "PAYLOAD_CHECKS_MISSING"),
    ] {
        if records(input, field).is_empty() {
            add_unknown(reasons, reason);
        }
    }

    for part in records(input, "part_requirements") {
        let required = u64_field(part, "required_quantity");
        let available = u64_field(part, "available_quantity");
        match str_field(part, "stock_state") {
            "out_of_stock" => add_conflict(conflicts, "PART_OUT_OF_STOCK"),
            "confirmed" => match (required, available) {
                (Some(required), Some(available)) if available < required => {
                    add_conflict(conflicts, "PART_OUT_OF_STOCK");
                }
                (Some(_), Some(_)) => {}
                _ => add_unknown(reasons, "PART_QUANTITY_UNKNOWN"),
            },
            _ => add_unknown(reasons, "PART_STOCK_EVIDENCE_MISSING"),
        }

        if str_field(part, "required_product_revision").is_empty()
            || str_field(part, "available_product_revision").is_empty()
        {
            add_unknown(reasons, "PART_REVISION_EVIDENCE_MISSING");
        } else if str_field(part, "required_product_revision")
            != str_field(part, "available_product_revision")
        {
            add_conflict(conflicts, "PART_REVISION_MISMATCH");
        }

        match str_field(part, "lot_state") {
            "revoked" => add_conflict(conflicts, "LOT_REVOKED"),
            "qualified" => {}
            _ => add_unknown(reasons, "LOT_QUALIFICATION_UNKNOWN"),
        }
        if str_field(part, "custody_state") != "accepted" {
            add_unknown(reasons, "LOT_CUSTODY_UNVERIFIED");
        }
        if str_field(part, "reservation_state") != "confirmed" {
            add_unknown(reasons, "PART_RESERVATION_MISSING");
        }
        let use_window = match (
            u64_field(part, "use_start_at"),
            u64_field(part, "use_end_at"),
        ) {
            (Some(start), Some(end)) if start < end => Some((start, end)),
            (Some(_), Some(_)) => {
                add_conflict(conflicts, "PART_USE_WINDOW_INVALID");
                None
            }
            _ => {
                add_unknown(reasons, "PART_USE_WINDOW_UNKNOWN");
                None
            }
        };
        if let Some((use_start, use_end)) = use_window {
            if let Some((window_start, window_end)) = planning_window {
                if use_start < window_start || use_end > window_end {
                    add_conflict(conflicts, "PART_USE_OUTSIDE_PLANNING_WINDOW");
                }
            }
            if let (Some(from), Some(until)) = (
                u64_field(part, "available_from"),
                u64_field(part, "available_until"),
            ) {
                if from >= until {
                    add_conflict(conflicts, "PART_AVAILABILITY_WINDOW_INVALID");
                } else if from > use_start || until < use_end {
                    add_conflict(conflicts, "PART_AVAILABILITY_WINDOW_MISMATCH");
                }
            } else {
                add_unknown(reasons, "PART_AVAILABILITY_WINDOW_UNKNOWN");
            }
        }
        if str_field(part, "evidence_state") != "current" {
            add_unknown(reasons, "PART_EVIDENCE_NOT_CURRENT");
        }
    }

    for substitution in records(input, "substitutions") {
        if bool_field(substitution, "substitute_used") == Some(true)
            && (bool_field(substitution, "qualified") != Some(true)
                || bool_field(substitution, "interface_compatible") != Some(true)
                || str_field(substitution, "approved_revision").is_empty()
                || str_field(substitution, "evidence_state") != "current")
        {
            add_unknown(reasons, "SUBSTITUTE_QUALIFICATION_MISSING");
        }
    }

    let reservations = records(input, "reservations");
    for (index, left) in reservations.iter().enumerate() {
        if str_field(left, "evidence_state") != "current"
            || str_field(left, "resource_id").is_empty()
            || str_field(left, "work_package_id").is_empty()
        {
            add_unknown(reasons, "RESOURCE_RESERVATION_EVIDENCE_UNKNOWN");
        }
        if let (Some(start), Some(end)) = (u64_field(left, "start_at"), u64_field(left, "end_at")) {
            if start >= end {
                add_conflict(conflicts, "RESOURCE_RESERVATION_INTERVAL_INVALID");
            }
        } else {
            add_unknown(reasons, "RESOURCE_RESERVATION_INTERVAL_UNKNOWN");
        }
        for right in reservations.iter().skip(index + 1) {
            let same_resource = !str_field(left, "resource_id").is_empty()
                && str_field(left, "resource_id") == str_field(right, "resource_id")
                && str_field(left, "resource_kind") == str_field(right, "resource_kind");
            let distinct_packages =
                str_field(left, "work_package_id") != str_field(right, "work_package_id");
            let exclusive = bool_field(left, "exclusive") == Some(true)
                && bool_field(right, "exclusive") == Some(true);
            let interval = (
                u64_field(left, "start_at"),
                u64_field(left, "end_at"),
                u64_field(right, "start_at"),
                u64_field(right, "end_at"),
            );
            if let (true, true, true, (Some(a), Some(b), Some(c), Some(d))) =
                (same_resource, distinct_packages, exclusive, interval)
                && interval_overlaps(a, b, c, d)
            {
                add_conflict(conflicts, "EXCLUSIVE_RESOURCE_DOUBLE_BOOKED");
            }
        }
    }

    for delivery in records(input, "deliveries") {
        match str_field(delivery, "route_state") {
            "closed" => add_conflict(conflicts, "DELIVERY_ROUTE_CLOSED"),
            "open" => {}
            _ => add_unknown(reasons, "DELIVERY_ROUTE_UNKNOWN"),
        }
        if str_field(delivery, "capacity_state") != "confirmed" {
            add_unknown(reasons, "SUPPLIER_CAPACITY_UNKNOWN");
        }
        if let (Some(arrival), Some(latest_start)) = (
            u64_field(delivery, "earliest_arrival"),
            u64_field(delivery, "latest_start"),
        ) {
            if arrival > latest_start {
                add_conflict(conflicts, "DELIVERY_AFTER_WORK_WINDOW");
            }
        } else {
            add_unknown(reasons, "DELIVERY_WINDOW_UNKNOWN");
        }
        match str_field(delivery, "unloading_state") {
            "denied" => add_conflict(conflicts, "SITE_UNLOADING_DENIED"),
            "allowed" => {}
            _ => add_unknown(reasons, "SITE_UNLOADING_EVIDENCE_UNKNOWN"),
        }
        if str_field(delivery, "evidence_state") != "current" {
            add_unknown(reasons, "DELIVERY_EVIDENCE_NOT_CURRENT");
        }
    }

    for crew in records(input, "crew_requirements") {
        if str_field(crew, "skill_state") != "qualified"
            || str_field(crew, "evidence_state") != "current"
        {
            add_unknown(reasons, "CREW_SKILL_EVIDENCE_MISSING");
        }
        if let (Some(valid_from), Some(valid_until), Some(work_start), Some(work_end)) = (
            u64_field(crew, "qualification_valid_from"),
            u64_field(crew, "qualification_valid_until"),
            u64_field(crew, "work_start"),
            u64_field(crew, "work_end"),
        ) {
            if valid_from > work_start || valid_until < work_end {
                add_conflict(conflicts, "CREW_QUALIFICATION_EXPIRED");
            }
        } else {
            add_unknown(reasons, "CREW_QUALIFICATION_INTERVAL_UNKNOWN");
        }
        if let (Some(required), Some(available)) = (
            u64_field(crew, "required_hours"),
            u64_field(crew, "available_hours"),
        ) {
            if required > available {
                add_conflict(conflicts, "CREW_CAPACITY_INSUFFICIENT");
            }
        } else {
            add_unknown(reasons, "CREW_CAPACITY_UNKNOWN");
        }
    }

    for tool in records(input, "tool_requirements") {
        match str_field(tool, "availability_state") {
            "unavailable" => add_conflict(conflicts, "TOOL_UNAVAILABLE"),
            "available" => {}
            _ => add_unknown(reasons, "TOOL_AVAILABILITY_UNKNOWN"),
        }
        if let (Some(from), Some(until), Some(start), Some(end)) = (
            u64_field(tool, "available_from"),
            u64_field(tool, "available_until"),
            u64_field(tool, "work_start"),
            u64_field(tool, "work_end"),
        ) {
            if from > start || until < end {
                add_conflict(conflicts, "TOOL_WORK_WINDOW_MISMATCH");
            }
        } else {
            add_unknown(reasons, "TOOL_WORK_WINDOW_UNKNOWN");
        }
    }

    for calibration in records(input, "calibration_requirements") {
        if str_field(calibration, "calibration_state") == "expired" {
            add_conflict(conflicts, "CALIBRATION_EXPIRED");
        } else if str_field(calibration, "calibration_state") != "current"
            || str_field(calibration, "evidence_state") != "current"
        {
            add_unknown(reasons, "CALIBRATION_EVIDENCE_UNKNOWN");
        }
        if let (Some(at), Some(from), Some(until)) = (
            u64_field(calibration, "measurement_at"),
            u64_field(calibration, "valid_from"),
            u64_field(calibration, "valid_until"),
        ) {
            if at < from || at >= until {
                add_conflict(conflicts, "CALIBRATION_EXPIRED");
            }
        } else {
            add_unknown(reasons, "CALIBRATION_INTERVAL_UNKNOWN");
        }
    }

    for precedence in records(input, "precedence_constraints") {
        if str_field(precedence, "evidence_state") != "current" {
            add_unknown(reasons, "WORK_PRECEDENCE_EVIDENCE_UNKNOWN");
        }
        if let (Some(pred_end), Some(successor_start)) = (
            u64_field(precedence, "predecessor_end"),
            u64_field(precedence, "successor_start"),
        ) {
            if successor_start < pred_end {
                add_conflict(conflicts, "WORK_PRECEDENCE_VIOLATION");
            }
        } else {
            add_unknown(reasons, "WORK_PRECEDENCE_WINDOW_UNKNOWN");
        }
    }

    for isolation in records(input, "isolation_requirements") {
        if bool_field(isolation, "required") == Some(true) {
            if str_field(isolation, "evidence_state") != "current" {
                add_unknown(reasons, "ISOLATION_EVIDENCE_MISSING");
            }
            if bool_field(isolation, "protected_service_affected") == Some(true)
                && bool_field(isolation, "mitigation_accepted") != Some(true)
            {
                add_conflict(conflicts, "PROTECTED_SERVICE_MITIGATION_MISSING");
            }
        } else if bool_field(isolation, "required").is_none() {
            add_unknown(reasons, "ISOLATION_REQUIREMENT_UNKNOWN");
        }
    }

    for access in records(input, "access_requirements") {
        match str_field(access, "access_state") {
            "denied" => add_conflict(conflicts, "ACCESS_DENIED"),
            "allowed" => {}
            _ => add_unknown(reasons, "ACCESS_EVIDENCE_UNKNOWN"),
        }
        if str_field(access, "evidence_state") != "current" {
            add_unknown(reasons, "ACCESS_EVIDENCE_NOT_CURRENT");
        }
    }

    for tolerance in records(input, "tolerance_checks") {
        match (
            tolerance.get("observed_offset").and_then(Value::as_f64),
            tolerance
                .get("qualified_adjustment_limit")
                .and_then(Value::as_f64),
        ) {
            (Some(observed), Some(limit)) if observed.abs() > limit => {
                add_conflict(conflicts, "ADJUSTMENT_ENVELOPE_EXCEEDED");
            }
            (Some(_), Some(_)) => {}
            _ => add_unknown(reasons, "ADJUSTMENT_EVIDENCE_UNKNOWN"),
        }
        if str_field(tolerance, "evidence_state") != "current" {
            add_unknown(reasons, "ADJUSTMENT_EVIDENCE_NOT_CURRENT");
        }
    }

    for check in records(input, "repair_envelope_checks") {
        match bool_field(check, "condition_inside_approved_envelope") {
            Some(false) => add_conflict(conflicts, "REPAIR_OUTSIDE_APPROVED_ENVELOPE"),
            Some(true) => {}
            None => add_unknown(reasons, "REPAIR_ENVELOPE_EVIDENCE_UNKNOWN"),
        }
        if str_field(check, "evidence_state") != "current" {
            add_unknown(reasons, "REPAIR_ENVELOPE_EVIDENCE_NOT_CURRENT");
        }
    }

    let rework = input.get("rework_accounting").unwrap_or(&Value::Null);
    if str_field(rework, "state") != "complete"
        || str_field(rework, "evidence_state") != "current"
        || u64_field(rework, "failed_attempts").is_none()
        || u64_field(rework, "rework_count").is_none()
    {
        add_unknown(reasons, "REWORK_ACCOUNTING_MISSING");
    }

    for payload in records(input, "payload_checks") {
        if bool_field(payload, "bytes_verified") != Some(true) {
            add_unknown(reasons, "PAYLOAD_BYTES_UNVERIFIED");
        } else if str_field(payload, "issuer_state") != "authorized"
            || str_field(payload, "rights_state") != "permitted"
            || str_field(payload, "technical_state") != "qualified"
            || str_field(payload, "evidence_state") != "current"
        {
            add_unknown(reasons, "ISSUER_RIGHTS_OR_TECHNICAL_AUTHORITY_UNKNOWN");
        }
    }
}

fn evaluate_lifecycle(
    input: &Value,
    reasons: &mut BTreeSet<String>,
) -> (String, String, String, String) {
    let lifecycle = input.get("lifecycle").unwrap_or(&Value::Null);
    let work = lifecycle_axis(
        str_field(lifecycle, "work_execution"),
        str_field(lifecycle, "execution_evidence_state"),
        &[
            "not_started",
            "in_progress",
            "failed",
            "completed",
            "unknown",
        ],
        "unknown",
    );
    if str_field(lifecycle, "work_execution") == "completed" && work != "completed" {
        reasons.insert("WORK_EXECUTION_EVIDENCE_UNKNOWN".to_owned());
    }

    let inspection_raw = str_field(lifecycle, "inspection_state");
    let inspection = if inspection_raw == "pass" || inspection_raw == "fail" {
        if work == "completed" && str_field(lifecycle, "inspection_evidence_state") == "current" {
            inspection_raw.to_owned()
        } else {
            reasons.insert("INSPECTION_EVIDENCE_UNKNOWN".to_owned());
            "unknown".to_owned()
        }
    } else {
        lifecycle_axis(
            inspection_raw,
            str_field(lifecycle, "inspection_evidence_state"),
            &["not_performed", "unknown", "pass", "fail"],
            "unknown",
        )
    };

    let commissioning_raw = str_field(lifecycle, "commissioning_state");
    let commissioning = if commissioning_raw == "pass" || commissioning_raw == "fail" {
        if inspection == "pass" && str_field(lifecycle, "commissioning_evidence_state") == "current"
        {
            commissioning_raw.to_owned()
        } else {
            reasons.insert("COMMISSIONING_EVIDENCE_UNKNOWN".to_owned());
            "unknown".to_owned()
        }
    } else {
        lifecycle_axis(
            commissioning_raw,
            str_field(lifecycle, "commissioning_evidence_state"),
            &["not_performed", "unknown", "pass", "fail"],
            "unknown",
        )
    };

    let service_state = str_field(lifecycle, "service_state");
    let service = if commissioning != "pass" {
        "unknown".to_owned()
    } else if !matches!(
        service_state,
        "adequate" | "constrained" | "inaccessible" | "unknown"
    ) {
        reasons.insert("CONTINUITY_SERVICE_STATE_UNSUPPORTED".to_owned());
        "unknown".to_owned()
    } else if service_state == "unknown"
        || str_field(lifecycle, "continuity_receipt_state") != "current"
        || str_field(lifecycle, "service_scope_digest")
            != text_field(input.get("scope").unwrap_or(&Value::Null), "scope_digest")
    {
        reasons.insert("CONTINUITY_SERVICE_EVIDENCE_UNKNOWN".to_owned());
        "unknown".to_owned()
    } else if bool_field(lifecycle, "stable_adequacy_met") != Some(true) {
        reasons.insert("STABLE_ADEQUACY_INTERVAL_NOT_MET".to_owned());
        "unknown".to_owned()
    } else {
        service_state.to_owned()
    };

    (work, inspection, commissioning, service)
}

fn text_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Evaluate a restoration case from its input only. The expected oracle is
/// deliberately never consulted by this function.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = str_field(case, "id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let mut plan_reasons = BTreeSet::new();
    let mut lifecycle_reasons = BTreeSet::new();
    let mut conflicts = BTreeSet::new();

    evaluate_plan(input, &mut plan_reasons, &mut conflicts);
    let (work, inspection, commissioning, service) =
        evaluate_lifecycle(input, &mut lifecycle_reasons);

    let plan_readiness = if !conflicts.is_empty() {
        "infeasible"
    } else if !plan_reasons.is_empty() {
        "unknown"
    } else {
        "feasible"
    };
    let reason_codes = plan_reasons
        .into_iter()
        .chain(conflicts)
        .chain(lifecycle_reasons)
        .collect::<BTreeSet<_>>();

    json!({
        "id": case_id,
        "plan_readiness": plan_readiness,
        "work_execution": work,
        "inspection_state": inspection,
        "commissioning_state": commissioning,
        "service_state": service,
        "reason_codes": reason_codes.into_iter().collect::<Vec<_>>(),
        "action_authority": "none",
        "causal_effect_established": false,
        "claim_ceiling": CLAIM_CEILING
    })
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
            "../../../docs/research/city-service-restoration/CITY_SERVICE_RESTORATION_EVAL_CORPUS_V0_1.json"
        );
        serde_json::from_str(source).expect("synthetic corpus parses")
    }

    #[test]
    fn every_frozen_restoration_case_matches_the_known_answer() {
        let corpus = corpus();
        let cases = corpus["cases"].as_array().expect("cases array");
        assert_eq!(cases.len(), 32);
        for case in cases {
            let actual = evaluate_case(case);
            assert!(
                matches_expected(&actual, &case["expected"]),
                "{}: actual={} expected={}",
                str_field(case, "id"),
                actual,
                case["expected"]
            );
        }
    }

    #[test]
    fn missing_planning_window_does_not_hide_a_proven_hard_conflict() {
        let mut case = corpus()["cases"][0].clone();
        case["input"].as_object_mut().unwrap().remove("planning_window");
        case["input"]["part_requirements"][0]["stock_state"] = json!("out_of_stock");
        let result = evaluate_case(&case);
        assert_eq!(result["plan_readiness"], "infeasible");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PART_OUT_OF_STOCK")
        );
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PLANNING_WINDOW_MISSING")
        );
    }

    #[test]
    fn hard_conflict_takes_precedence_over_missing_evidence() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["part_requirements"][0]["stock_state"] = json!("out_of_stock");
        case["input"]["part_requirements"][0]["custody_state"] = json!("unknown");
        let result = evaluate_case(&case);
        assert_eq!(result["plan_readiness"], "infeasible");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PART_OUT_OF_STOCK")
        );
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "LOT_CUSTODY_UNVERIFIED")
        );
    }

    #[test]
    fn adjacent_half_open_reservations_do_not_conflict() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["reservations"][0]["end_at"] = json!(2500);
        case["input"]["reservations"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "reservation_id":"RES-2","resource_id":"LOT-1","resource_kind":"part_lot",
                "work_package_id":"WP-2","exclusive":true,"start_at":2500,"end_at":3000,
                "evidence_state":"current"
            }));
        let result = evaluate_case(&case);
        assert_ne!(result["plan_readiness"], "infeasible");
        assert!(
            !result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "EXCLUSIVE_RESOURCE_DOUBLE_BOOKED")
        );
    }

    #[test]
    fn missing_resource_domain_never_defaults_to_feasible() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]
            .as_object_mut()
            .unwrap()
            .remove("crew_requirements");
        let result = evaluate_case(&case);
        assert_eq!(result["plan_readiness"], "unknown");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "CREW_REQUIREMENTS_MISSING")
        );
    }

    #[test]
    fn commissioning_without_continuity_evidence_does_not_restore_service() {
        let result = evaluate_case(&corpus()["cases"][28]);
        assert_eq!(result["plan_readiness"], "feasible");
        assert_eq!(result["commissioning_state"], "pass");
        assert_eq!(result["service_state"], "unknown");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "CONTINUITY_SERVICE_EVIDENCE_UNKNOWN")
        );
    }

    #[test]
    fn stable_adequacy_requires_the_owner_declared_interval() {
        let result = evaluate_case(&corpus()["cases"][29]);
        assert_eq!(result["commissioning_state"], "pass");
        assert_eq!(result["service_state"], "unknown");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "STABLE_ADEQUACY_INTERVAL_NOT_MET")
        );
    }

    #[test]
    fn byte_verification_does_not_promote_issuer_rights_or_technical_authority() {
        let result = evaluate_case(&corpus()["cases"][31]);
        assert_eq!(result["plan_readiness"], "unknown");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "ISSUER_RIGHTS_OR_TECHNICAL_AUTHORITY_UNKNOWN")
        );
    }

    #[test]
    fn proposed_feasibility_never_promotes_lifecycle_or_authority() {
        let result = evaluate_case(&corpus()["cases"][0]);
        assert_eq!(result["plan_readiness"], "feasible");
        assert_eq!(result["work_execution"], "not_started");
        assert_eq!(result["inspection_state"], "not_performed");
        assert_eq!(result["commissioning_state"], "not_performed");
        assert_eq!(result["service_state"], "unknown");
        assert_eq!(result["action_authority"], "none");
        assert_eq!(result["causal_effect_established"], false);
    }
    #[test]
    fn part_availability_only_needs_to_cover_its_actual_use_interval() {
        let mut case = corpus()["cases"][0].clone();
        let part = &mut case["input"]["part_requirements"][0];
        part["available_from"] = json!(1900);
        part["available_until"] = json!(3100);
        part["use_start_at"] = json!(2000);
        part["use_end_at"] = json!(3000);
        let result = evaluate_case(&case);
        assert_eq!(result["plan_readiness"], "feasible");
        assert!(
            !result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PART_AVAILABILITY_WINDOW_MISMATCH")
        );
    }

    #[test]
    fn part_must_be_available_for_the_entire_actual_use_interval() {
        let mut late = corpus()["cases"][0].clone();
        let part = &mut late["input"]["part_requirements"][0];
        part["available_from"] = json!(2100);
        part["available_until"] = json!(3100);
        let result = evaluate_case(&late);
        assert_eq!(result["plan_readiness"], "infeasible");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PART_AVAILABILITY_WINDOW_MISMATCH")
        );

        let mut early_expiry = corpus()["cases"][0].clone();
        let part = &mut early_expiry["input"]["part_requirements"][0];
        part["available_from"] = json!(1900);
        part["available_until"] = json!(2900);
        let result = evaluate_case(&early_expiry);
        assert_eq!(result["plan_readiness"], "infeasible");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PART_AVAILABILITY_WINDOW_MISMATCH")
        );
    }

    #[test]
    fn missing_part_use_interval_is_unknown_not_infeasible() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["part_requirements"][0]
            .as_object_mut()
            .unwrap()
            .remove("use_start_at");
        let result = evaluate_case(&case);
        assert_eq!(result["plan_readiness"], "unknown");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "PART_USE_WINDOW_UNKNOWN")
        );
    }

    #[test]
    fn part_use_interval_uses_half_open_end_boundary() {
        let mut case = corpus()["cases"][0].clone();
        let part = &mut case["input"]["part_requirements"][0];
        part["available_from"] = json!(2000);
        part["available_until"] = json!(3000);
        part["use_start_at"] = json!(2000);
        part["use_end_at"] = json!(3000);
        let result = evaluate_case(&case);
        assert_eq!(result["plan_readiness"], "feasible");
    }
    #[test]
    fn service_state_requires_an_owner_native_enum_value() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["lifecycle"]["work_execution"] = json!("completed");
        case["input"]["lifecycle"]["inspection_state"] = json!("pass");
        case["input"]["lifecycle"]["commissioning_state"] = json!("pass");
        case["input"]["lifecycle"]["service_state"] = json!("fully_operational");
        case["input"]["lifecycle"]["continuity_receipt_state"] = json!("current");
        case["input"]["lifecycle"]["stable_adequacy_met"] = json!(true);
        case["input"]["lifecycle"]["execution_evidence_state"] = json!("current");
        case["input"]["lifecycle"]["inspection_evidence_state"] = json!("current");
        case["input"]["lifecycle"]["commissioning_evidence_state"] = json!("current");
        let result = evaluate_case(&case);
        assert_eq!(result["service_state"], "unknown");
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "CONTINUITY_SERVICE_STATE_UNSUPPORTED")
        );
    }

    #[test]
    fn current_continuity_receipt_can_project_adequate_service_after_stable_confirmation() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["lifecycle"]["work_execution"] = json!("completed");
        case["input"]["lifecycle"]["inspection_state"] = json!("pass");
        case["input"]["lifecycle"]["commissioning_state"] = json!("pass");
        case["input"]["lifecycle"]["service_state"] = json!("adequate");
        case["input"]["lifecycle"]["continuity_receipt_state"] = json!("current");
        case["input"]["lifecycle"]["stable_adequacy_met"] = json!(true);
        case["input"]["lifecycle"]["execution_evidence_state"] = json!("current");
        case["input"]["lifecycle"]["inspection_evidence_state"] = json!("current");
        case["input"]["lifecycle"]["commissioning_evidence_state"] = json!("current");
        let result = evaluate_case(&case);
        assert_eq!(result["work_execution"], "completed");
        assert_eq!(result["inspection_state"], "pass");
        assert_eq!(result["commissioning_state"], "pass");
        assert_eq!(result["service_state"], "adequate");
    }
}
