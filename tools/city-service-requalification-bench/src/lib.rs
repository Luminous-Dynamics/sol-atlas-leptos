//! Synthetic-first CityMaker service requalification compositor.
//!
//! This crate composes declared service-owner reports, requalification
//! obligations and physical restoration receipts. It does not calculate
//! engineering thresholds, authenticate report bytes or authorize operation.

use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const CORPUS_ID: &str = "CITY-SERVICE-REQUALIFICATION-CORPUS-V0.1";
const CLAIM_CEILING: &str = "synthetic owner-report envelope and state-composition semantics only; no report-content authentication, engineering threshold validation, physical safety determination, source completeness or action authority";

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn bool_value(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }
    let year = value[0..4].parse::<u32>().ok();
    let month = value[5..7].parse::<u32>().ok();
    let day = value[8..10].parse::<u32>().ok();
    let hour = value[11..13].parse::<u32>().ok();
    let minute = value[14..16].parse::<u32>().ok();
    let second = value[17..19].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) =
        (year, month, day, hour, minute, second)
    else {
        return false;
    };
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= max_day && hour <= 23 && minute <= 59 && second <= 59
}

fn string_set(values: &[Value]) -> BTreeSet<String> {
    values
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn has_all(actual: &[Value], required: &[Value]) -> bool {
    let actual = string_set(actual);
    required
        .iter()
        .filter_map(Value::as_str)
        .all(|value| actual.contains(value))
}

fn same_temporal_envelope(report: &Value, event: &Value, query: &Value) -> bool {
    let query_time = query.get("temporal_cutoff").unwrap_or(&Value::Null);
    report.get("valid_time") == event.get("valid_time")
        && report.get("system_time") == event.get("system_time")
        && report.get("temporal_cutoff") == Some(query_time)
        && text(query_time, "time_basis") == "UTC_RFC3339_SECONDS"
        && valid_utc_timestamp(text(query_time, "world_time_as_of"))
        && valid_utc_timestamp(text(query_time, "knowledge_time_as_of"))
        && valid_utc_timestamp(text(query_time, "trust_time_as_of"))
        && valid_utc_timestamp(text(query_time, "evaluated_at"))
        && matches!(
            text(query_time, "temporal_perspective"),
            "historically_known_then" | "currently_reconstructed_history"
        )
}

fn event_and_query_reasons(event: &Value, query: &Value) -> BTreeSet<String> {
    let mut reasons = BTreeSet::new();
    if text(event, "event_id").is_empty()
        || !lower_hex(text(event, "event_sha256"), 64)
        || text(event, "subject_node_id").is_empty()
        || text(event, "subject_revision").is_empty()
        || text(event, "time_basis") != "UTC_RFC3339_SECONDS"
    {
        reasons.insert("CHANGE_EVENT_BINDING_INCOMPLETE".to_owned());
    }

    let valid_time = event.get("valid_time").unwrap_or(&Value::Null);
    let effective_at = text(valid_time, "effective_at");
    let effective_until = valid_time.get("effective_until");
    let confidence = text(valid_time, "effective_time_confidence");
    if !valid_utc_timestamp(effective_at)
        || effective_until.is_none()
        || (effective_until.is_some_and(|value| !value.is_null())
            && !valid_utc_timestamp(effective_until.and_then(Value::as_str).unwrap_or("")))
        || !matches!(confidence, "verified" | "authority_asserted" | "unknown" | "disputed")
    {
        reasons.insert("CHANGE_EVENT_VALID_TIME_INCOMPLETE".to_owned());
    } else {
        if confidence != "verified" {
            reasons.insert("CHANGE_EVENT_EFFECTIVE_TIME_UNQUALIFIED".to_owned());
        }
        if let Some(until) = effective_until.and_then(Value::as_str) {
            if until < effective_at {
                reasons.insert("CHANGE_EVENT_VALID_INTERVAL_INVALID".to_owned());
            }
        }
    }

    let system_time = event.get("system_time").unwrap_or(&Value::Null);
    let recorded_at = text(system_time, "recorded_at");
    let trusted_at = system_time.get("trusted_at");
    let trusted_receipt = system_time.get("trusted_timestamp_receipt_sha256");
    let explicitly_untrusted = trusted_at.is_some_and(Value::is_null)
        && trusted_receipt.is_some_and(Value::is_null);
    let trust_pair_valid = valid_utc_timestamp(
        trusted_at.and_then(Value::as_str).unwrap_or(""),
    ) && lower_hex(
        trusted_receipt.and_then(Value::as_str).unwrap_or(""),
        64,
    );
    if system_time.get("recorded_at").is_none()
        || trusted_at.is_none()
        || trusted_receipt.is_none()
        || !valid_utc_timestamp(recorded_at)
        || (!explicitly_untrusted && !trust_pair_valid)
    {
        reasons.insert("CHANGE_EVENT_SYSTEM_TIME_INCOMPLETE".to_owned());
    } else if explicitly_untrusted {
        reasons.insert("CHANGE_EVENT_SYSTEM_TIME_UNTRUSTED".to_owned());
    } else if trusted_at.and_then(Value::as_str).unwrap_or("") < recorded_at {
        reasons.insert("CHANGE_EVENT_TRUST_TIME_ORDER_INVALID".to_owned());
    }

    if text(query, "scope_id").is_empty()
        || !lower_hex(text(query, "scope_digest"), 64)
        || text(query, "functional_unit").is_empty()
    {
        reasons.insert("CITY_SERVICE_QUERY_SCOPE_INCOMPLETE".to_owned());
    }
    let temporal_cutoff = query.get("temporal_cutoff").unwrap_or(&Value::Null);
    let world_time = text(temporal_cutoff, "world_time_as_of");
    let knowledge_time = text(temporal_cutoff, "knowledge_time_as_of");
    let trust_time = text(temporal_cutoff, "trust_time_as_of");
    if text(temporal_cutoff, "time_basis") != "UTC_RFC3339_SECONDS"
        || !valid_utc_timestamp(world_time)
        || !valid_utc_timestamp(knowledge_time)
        || !valid_utc_timestamp(trust_time)
        || !valid_utc_timestamp(text(temporal_cutoff, "evaluated_at"))
        || !matches!(
            text(temporal_cutoff, "temporal_perspective"),
            "historically_known_then" | "currently_reconstructed_history"
        )
    {
        reasons.insert("CITY_SERVICE_TEMPORAL_CUTOFF_INCOMPLETE".to_owned());
    } else {
        if valid_utc_timestamp(effective_at)
            && (effective_at > world_time
                || effective_until
                    .and_then(Value::as_str)
                    .is_some_and(|until| world_time >= until))
        {
            reasons.insert("EVENT_NOT_VALID_AT_REQUESTED_WORLD_TIME".to_owned());
        }
        let system_time = event.get("system_time").unwrap_or(&Value::Null);
        let recorded_at = text(system_time, "recorded_at");
        if valid_utc_timestamp(recorded_at) && recorded_at > knowledge_time {
            reasons.insert("EVENT_RECORDED_AFTER_KNOWLEDGE_CUTOFF".to_owned());
        }
        if let Some(trusted_at) = system_time.get("trusted_at").and_then(Value::as_str) {
            if valid_utc_timestamp(trusted_at) && trusted_at > trust_time {
                reasons.insert("EVENT_TRUSTED_AFTER_TRUST_CUTOFF".to_owned());
            }
        }
    }
    reasons
}

fn report_binding_reasons(
    report: &Value,
    requirement: &Value,
    event: &Value,
    query: &Value,
) -> BTreeSet<String> {
    let mut reasons = BTreeSet::new();
    for (field, code) in [
        ("service_id", "SERVICE_REPORT_ID_MISMATCH"),
        ("owner_id", "SERVICE_REPORT_OWNER_MISMATCH"),
        ("owner_repo", "SERVICE_REPORT_REPOSITORY_MISMATCH"),
        ("contract_ref", "SERVICE_REPORT_CONTRACT_MISMATCH"),
        ("contract_revision", "SERVICE_REPORT_CONTRACT_REVISION_MISMATCH"),
        ("schema_id", "SERVICE_REPORT_SCHEMA_MISMATCH"),
        ("schema_version", "SERVICE_REPORT_SCHEMA_VERSION_MISMATCH"),
        ("runtime_revision", "SERVICE_REPORT_RUNTIME_MISMATCH"),
        ("report_kind", "SERVICE_REPORT_KIND_MISMATCH"),
    ] {
        if text(report, field) != text(requirement, field) {
            reasons.insert(code.to_owned());
        }
    }
    if text(report, "event_id") != text(event, "event_id")
        || text(report, "event_sha256") != text(event, "event_sha256")
    {
        reasons.insert("SERVICE_REPORT_EVENT_BINDING_MISMATCH".to_owned());
    }
    if text(report, "subject_node_id") != text(event, "subject_node_id")
        || text(report, "subject_revision") != text(event, "subject_revision")
    {
        reasons.insert("SERVICE_REPORT_SUBJECT_BINDING_MISMATCH".to_owned());
    }
    if text(report, "scope_id") != text(query, "scope_id")
        || text(report, "scope_digest") != text(query, "scope_digest")
        || text(report, "functional_unit") != text(query, "functional_unit")
    {
        reasons.insert("SERVICE_REPORT_SCOPE_BINDING_MISMATCH".to_owned());
    }
    let cutoff = query.get("temporal_cutoff").unwrap_or(&Value::Null);
    if !same_temporal_envelope(report, event, query)
        || text(report, "evaluated_at") != text(cutoff, "evaluated_at")
    {
        reasons.insert("SERVICE_REPORT_TEMPORAL_BINDING_MISMATCH".to_owned());
    }
    let digest = text(report, "report_sha256");
    if !lower_hex(digest, 64)
        || text(report, "report_reference") != format!("cas:sha256:{digest}")
    {
        reasons.insert("SERVICE_REPORT_DIGEST_REFERENCE_INVALID".to_owned());
    }
    if text(report, "action_authority") != "none" {
        reasons.insert("SERVICE_REPORT_ACTION_AUTHORITY_ESCALATION".to_owned());
    }
    if !lower_hex(text(report, "contract_revision"), 40)
        || !lower_hex(text(report, "runtime_revision"), 40)
    {
        reasons.insert("SERVICE_REPORT_CONTRACT_PIN_INVALID".to_owned());
    }
    reasons
}

fn evaluate_obligations(report: &Value, event: &Value, query: &Value) -> (String, Vec<String>) {
    if !bool_value(report, "affected").unwrap_or(false) {
        return ("not_required_for_declared_scope".to_owned(), Vec::new());
    }
    let obligations = report.get("required_obligations").and_then(Value::as_array);
    let Some(obligations) = obligations else {
        return (
            "blocked_insufficient_evidence".to_owned(),
            vec!["REQUIRED_OBLIGATION_SET_MISSING".to_owned()],
        );
    };
    if obligations.is_empty() {
        return (
            "blocked_insufficient_evidence".to_owned(),
            vec!["AFFECTED_SERVICE_HAS_NO_DECLARED_REQUALIFICATION_OBLIGATIONS".to_owned()],
        );
    }

    let mut ids = BTreeSet::new();
    let mut reasons = BTreeSet::new();
    let mut has_open = false;
    let mut has_blocked = false;
    let mut has_invalid = false;

    for obligation in obligations {
        let id = text(obligation, "obligation_id");
        if id.is_empty() || !ids.insert(id.to_owned()) {
            has_invalid = true;
            reasons.insert("REQUALIFICATION_OBLIGATION_ID_EMPTY_OR_DUPLICATED".to_owned());
            continue;
        }
        match text(obligation, "state") {
            "required_open" => has_open = true,
            "blocked_insufficient_evidence" => has_blocked = true,
            "invalid_evidence" => has_invalid = true,
            "discharged_by_exact_receipt" => {
                let receipt = obligation.get("discharge_receipt").unwrap_or(&Value::Null);
                let cutoff = query.get("temporal_cutoff").unwrap_or(&Value::Null);
                let digest = text(receipt, "receipt_sha256");
                let valid = text(receipt, "obligation_id") == id
                    && text(receipt, "owner_id") == text(obligation, "owner_id")
                    && text(receipt, "event_id") == text(event, "event_id")
                    && text(receipt, "event_sha256") == text(event, "event_sha256")
                    && text(receipt, "asset_revision") == text(event, "subject_revision")
                    && text(receipt, "scope_id") == text(query, "scope_id")
                    && text(receipt, "scope_digest") == text(query, "scope_digest")
                    && text(receipt, "functional_unit") == text(query, "functional_unit")
                    && text(receipt, "evaluated_at") == text(cutoff, "evaluated_at")
                    && lower_hex(digest, 64)
                    && text(receipt, "action_authority") == "none";
                if !valid {
                    has_invalid = true;
                    reasons.insert("REQUALIFICATION_DISCHARGE_RECEIPT_BINDING_INVALID".to_owned());
                }
            }
            _ => {
                has_invalid = true;
                reasons.insert("REQUALIFICATION_OBLIGATION_STATE_INVALID".to_owned());
            }
        }
    }

    let state = if has_invalid {
        "invalid_evidence"
    } else if has_blocked {
        "blocked_insufficient_evidence"
    } else if has_open {
        "required_open"
    } else {
        "discharged_by_exact_receipt"
    };
    (state.to_owned(), reasons.into_iter().collect())
}

fn physical_receipt_state(
    receipt: Option<&Value>,
    expected_kind: &str,
    service_id: &str,
    event: &Value,
    query: &Value,
) -> String {
    let Some(receipt) = receipt else {
        return "pending".to_owned();
    };
    let status = text(receipt, "status");
    if status == "pending" {
        return "pending".to_owned();
    }
    if status == "failed" {
        return "failed".to_owned();
    }
    if status != "passed" {
        return "invalid".to_owned();
    }
    let digest = text(receipt, "receipt_sha256");
    let cutoff = query.get("temporal_cutoff").unwrap_or(&Value::Null);
    let valid = text(receipt, "receipt_kind") == expected_kind
        && text(receipt, "service_id") == service_id
        && text(receipt, "event_id") == text(event, "event_id")
        && text(receipt, "event_sha256") == text(event, "event_sha256")
        && text(receipt, "asset_revision") == text(event, "subject_revision")
        && text(receipt, "scope_id") == text(query, "scope_id")
        && text(receipt, "scope_digest") == text(query, "scope_digest")
        && text(receipt, "functional_unit") == text(query, "functional_unit")
        && text(receipt, "evaluated_at") == text(cutoff, "evaluated_at")
        && has_all(array(receipt, "population_groups"), array(query, "required_population_groups"))
        && lower_hex(digest, 64)
        && text(receipt, "action_authority") == "none";
    if valid {
        "passed".to_owned()
    } else {
        "invalid".to_owned()
    }
}

fn measurement_state(
    measurement: Option<&Value>,
    service_id: &str,
    event: &Value,
    query: &Value,
) -> String {
    let Some(measurement) = measurement else {
        return "pending".to_owned();
    };
    if text(measurement, "state") == "pending" {
        return "pending".to_owned();
    }
    let cutoff = query.get("temporal_cutoff").unwrap_or(&Value::Null);
    let digest = text(measurement, "measurement_sha256");
    let sample_count = number(measurement, "sample_count");
    let required_samples = number(measurement, "required_sample_count");
    let valid_from = text(measurement, "valid_from");
    let valid_until = text(measurement, "valid_until");
    let evaluated_at = text(cutoff, "evaluated_at");
    let valid = text(measurement, "service_id") == service_id
        && text(measurement, "event_id") == text(event, "event_id")
        && text(measurement, "event_sha256") == text(event, "event_sha256")
        && text(measurement, "asset_revision") == text(event, "subject_revision")
        && text(measurement, "scope_id") == text(query, "scope_id")
        && text(measurement, "scope_digest") == text(query, "scope_digest")
        && text(measurement, "functional_unit") == text(query, "functional_unit")
        && text(measurement, "evaluated_at") == evaluated_at
        && valid_utc_timestamp(valid_from)
        && valid_utc_timestamp(valid_until)
        && valid_from <= evaluated_at
        && evaluated_at < valid_until
        && has_all(array(measurement, "population_groups"), array(query, "required_population_groups"))
        && sample_count.is_some()
        && required_samples.is_some()
        && sample_count.unwrap_or(0) >= required_samples.unwrap_or(u64::MAX)
        && lower_hex(digest, 64)
        && lower_hex(text(measurement, "calibration_receipt_sha256"), 64)
        && text(measurement, "independent_review") == "accepted_for_declared_scope"
        && text(measurement, "action_authority") == "none";
    if !valid {
        "invalid".to_owned()
    } else if text(measurement, "service_state") == "adequate" {
        "adequate".to_owned()
    } else if matches!(
        text(measurement, "service_state"),
        "constrained" | "interrupted" | "inaccessible"
    ) {
        "not_adequate".to_owned()
    } else {
        "invalid".to_owned()
    }
}

fn evaluate_restoration(report: &Value, event: &Value, query: &Value) -> (String, Vec<String>) {
    if !bool_value(report, "affected").unwrap_or(false) {
        return ("not_planned".to_owned(), Vec::new());
    }
    let plan = report.get("restoration").unwrap_or(&Value::Null);
    if plan.is_null() {
        return (
            "plan_unknown".to_owned(),
            vec!["RESTORATION_PLAN_MISSING".to_owned()],
        );
    }
    let plan_state = text(plan, "plan_state");
    let execution_state = text(plan, "execution_state");
    match plan_state {
        "infeasible" => return ("plan_infeasible".to_owned(), Vec::new()),
        "unknown" => return ("plan_unknown".to_owned(), Vec::new()),
        "feasible" => {}
        _ => {
            return (
                "plan_unknown".to_owned(),
                vec!["RESTORATION_PLAN_STATE_INVALID".to_owned()],
            );
        }
    }
    match execution_state {
        "not_started" => return ("plan_feasible".to_owned(), Vec::new()),
        "in_progress" => return ("work_in_progress".to_owned(), Vec::new()),
        "completed" => {}
        _ => {
            return (
                "restoration_unknown".to_owned(),
                vec!["RESTORATION_EXECUTION_STATE_INVALID".to_owned()],
            );
        }
    }

    let service_id = text(report, "service_id");
    let inspection = physical_receipt_state(
        plan.get("inspection"),
        "inspection",
        service_id,
        event,
        query,
    );
    let commissioning = physical_receipt_state(
        plan.get("commissioning"),
        "commissioning",
        service_id,
        event,
        query,
    );
    if inspection == "invalid" || commissioning == "invalid" {
        return (
            "restoration_unknown".to_owned(),
            vec!["INSPECTION_OR_COMMISSIONING_RECEIPT_INVALID".to_owned()],
        );
    }
    if inspection == "failed" || commissioning == "failed" {
        return ("restoration_partial_or_constrained".to_owned(), Vec::new());
    }
    if inspection != "passed" || commissioning != "passed" {
        return (
            "awaiting_inspection_or_commissioning".to_owned(),
            Vec::new(),
        );
    }

    match measurement_state(
        plan.get("outcome_measurement"),
        service_id,
        event,
        query,
    )
    .as_str()
    {
        "pending" => ("awaiting_outcome_measurement".to_owned(), Vec::new()),
        "invalid" => (
            "restoration_unknown".to_owned(),
            vec!["OUTCOME_MEASUREMENT_BINDING_OR_COVERAGE_INVALID".to_owned()],
        ),
        "not_adequate" => ("restoration_partial_or_constrained".to_owned(), Vec::new()),
        "adequate" => (
            "restoration_demonstrated_for_declared_scope".to_owned(),
            Vec::new(),
        ),
        _ => (
            "restoration_unknown".to_owned(),
            vec!["OUTCOME_MEASUREMENT_STATE_INVALID".to_owned()],
        ),
    }
}

fn evaluate_service(
    requirement: &Value,
    report: Option<&Value>,
    event: &Value,
    query: &Value,
) -> Value {
    let service_id = text(requirement, "service_id");
    let Some(report) = report else {
        return json!({
            "service_id": service_id,
            "report_state": "insufficient_evidence",
            "service_state": "unknown",
            "requalification_state": "blocked_insufficient_evidence",
            "restoration_state": "restoration_unknown",
            "reason_codes": ["SERVICE_OWNER_REPORT_MISSING"],
            "owner_report_content_verified": false,
            "action_authority": "none"
        });
    };

    let mut reasons = report_binding_reasons(report, requirement, event, query);
    let affected = bool_value(report, "affected");
    if affected.is_none() {
        reasons.insert("SERVICE_REPORT_AFFECTED_FIELD_MISSING".to_owned());
        return json!({
            "service_id": service_id,
            "report_state": "insufficient_evidence",
            "service_state": "unknown",
            "requalification_state": "blocked_insufficient_evidence",
            "restoration_state": "restoration_unknown",
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "owner_report_content_verified": false,
            "action_authority": "none"
        });
    }
    if !reasons.is_empty() {
        return json!({
            "service_id": service_id,
            "report_state": "invalid_input",
            "service_state": "unknown",
            "requalification_state": "invalid_evidence",
            "restoration_state": "restoration_unknown",
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "owner_report_content_verified": false,
            "action_authority": "none"
        });
    }

    let required_groups = array(requirement, "required_population_groups");
    let reported_groups = array(report, "population_groups");
    let group_coverage_valid = !required_groups.is_empty() && has_all(reported_groups, required_groups);
    let report_state = if group_coverage_valid {
        "contract_bound_unverified"
    } else {
        "insufficient_evidence"
    };
    let declared_service_state = text(report, "service_state");
    let service_state = if group_coverage_valid
        && matches!(
            declared_service_state,
            "adequate" | "constrained" | "interrupted" | "inaccessible" | "unknown"
        )
    {
        declared_service_state
    } else {
        "unknown"
    };
    let mut report_reasons = Vec::new();
    if !group_coverage_valid {
        report_reasons.push("SERVICE_POPULATION_COVERAGE_INCOMPLETE");
    }
    if service_state == "unknown" && declared_service_state != "unknown" {
        report_reasons.push("SERVICE_STATE_ENUM_INVALID");
    }

    let (requalification_state, obligation_reasons) =
        evaluate_obligations(report, event, query);
    let (restoration_state, restoration_reasons) =
        evaluate_restoration(report, event, query);
    report_reasons.extend(obligation_reasons);
    report_reasons.extend(restoration_reasons);

    json!({
        "service_id": service_id,
        "report_state": report_state,
        "service_state": service_state,
        "requalification_state": requalification_state,
        "restoration_state": restoration_state,
        "reason_codes": report_reasons,
        "owner_report_content_verified": false,
        "action_authority": "none"
    })
}

/// Evaluates the declared service-owner contracts and their composed outputs.
/// The function never treats a plan as completed work or a work order as
/// proof of restored service.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let event = input.get("event").unwrap_or(&Value::Null);
    let query = input.get("query").unwrap_or(&Value::Null);
    let requirements = array(query, "service_requirements");
    let reports = array(input, "service_reports");
    let mut global_reasons = event_and_query_reasons(event, query);

    if requirements.is_empty() {
        global_reasons.insert("SERVICE_REQUIREMENT_SET_EMPTY".to_owned());
    }
    let mut required_ids = BTreeSet::new();
    for requirement in requirements {
        let service_id = text(requirement, "service_id");
        if service_id.is_empty() || !required_ids.insert(service_id.to_owned()) {
            global_reasons.insert("SERVICE_REQUIREMENT_ID_EMPTY_OR_DUPLICATED".to_owned());
        }
    }

    for report in reports {
        let service_id = text(report, "service_id");
        if !required_ids.contains(service_id) {
            global_reasons.insert("UNEXPECTED_SERVICE_OWNER_REPORT".to_owned());
        }
    }

    let mut service_results = Vec::with_capacity(requirements.len());
    for requirement in requirements {
        let service_id = text(requirement, "service_id");
        let matching = reports
            .iter()
            .filter(|report| text(report, "service_id") == service_id)
            .collect::<Vec<_>>();
        if matching.len() > 1 {
            service_results.push(json!({
                "service_id": service_id,
                "report_state": "invalid_input",
                "service_state": "unknown",
                "requalification_state": "invalid_evidence",
                "restoration_state": "restoration_unknown",
                "reason_codes": ["DUPLICATE_SERVICE_OWNER_REPORT"],
                "owner_report_content_verified": false,
                "action_authority": "none"
            }));
            continue;
        }
        service_results.push(evaluate_service(
            requirement,
            matching.first().copied(),
            event,
            query,
        ));
    }

    let has_invalid_report = service_results
        .iter()
        .any(|service| text(service, "report_state") == "invalid_input");
    let has_insufficient_report = service_results
        .iter()
        .any(|service| text(service, "report_state") == "insufficient_evidence");
    let has_invalid_composition = service_results.iter().any(|service| {
        text(service, "requalification_state") == "invalid_evidence"
            || array(service, "reason_codes").iter().filter_map(Value::as_str).any(|reason| {
                reason.contains("INVALID")
                    || reason.contains("MISMATCH")
                    || reason.contains("ESCALATION")
            })
    });
    let has_invalid_global = global_reasons
        .iter()
        .any(|reason| reason.contains("INVALID") || reason.contains("DUPLICATED") || reason.contains("UNEXPECTED"));
    let route_state = if has_invalid_report || has_invalid_composition || has_invalid_global {
        "invalid_input"
    } else if !global_reasons.is_empty() || has_insufficient_report {
        "insufficient_evidence"
    } else {
        "contract_bound_unverified"
    };

    json!({
        "case_id": case_id,
        "route_state": route_state,
        "service_results": service_results,
        "reason_codes": global_reasons.into_iter().collect::<Vec<_>>(),
        "service_outcome_verified": false,
        "owner_report_content_verified": false,
        "owner_authentication": "not_performed",
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING
    })
}

fn equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => left
            .as_u64()
            .zip(right.as_u64())
            .is_some_and(|(a, b)| a == b),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(left), Value::Object(right)) => right.iter().all(|(key, value)| {
            left.get(key).is_some_and(|actual_value| equivalent(actual_value, value))
        }),
        _ => actual == expected,
    }
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    equivalent(actual, expected)
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected};
    use serde_json::Value;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-service-requalification/CITY_SERVICE_REQUALIFICATION_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("service requalification corpus must exist");
        serde_json::from_slice(&bytes).expect("service requalification corpus must parse")
    }

    #[test]
    fn all_frozen_service_requalification_cases_match() {
        let data = corpus();
        let cases = data["cases"].as_array().unwrap();
        for case in cases {
            let actual = evaluate_case(case);
            assert!(
                matches_expected(&actual, &case["expected"]),
                "{} mismatch\nactual: {}\nexpected: {}",
                case["case_id"].as_str().unwrap(),
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&case["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn a_feasible_plan_never_proves_restoration() {
        let cases = corpus()["cases"].as_array().unwrap().clone();
        let result = evaluate_case(&cases[1]);
        assert_eq!(result["service_results"][0]["restoration_state"], "plan_feasible");
        assert_eq!(result["service_results"][0]["service_state"], "interrupted");
    }

    #[test]
    fn completed_work_without_inspection_does_not_restore_service() {
        let cases = corpus()["cases"].as_array().unwrap().clone();
        let result = evaluate_case(&cases[3]);
        assert_eq!(
            result["service_results"][0]["restoration_state"],
            "awaiting_inspection_or_commissioning"
        );
    }

    #[test]
    fn restored_service_requires_measurement_coverage_and_owner_receipts() {
        let cases = corpus()["cases"].as_array().unwrap().clone();
        let result = evaluate_case(&cases[5]);
        assert_eq!(
            result["service_results"][0]["restoration_state"],
            "restoration_demonstrated_for_declared_scope"
        );
        assert_eq!(result["service_outcome_verified"], false);
        assert_eq!(result["action_authority"], "none");
    }
}
