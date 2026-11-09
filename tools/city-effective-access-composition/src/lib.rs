//! Read-only composition reference for effective service restoration and access.
//!
//! The evaluator does not route, calculate service capacity, ingest measurements,
//! authorize work, or infer individual service receipt. It composes independently
//! evaluated owner reports under one explicit beneficiary and temporal scope.

use std::collections::BTreeSet;

use serde_json::{Value, json};

pub const CORPUS_ID: &str = "CITY-EFFECTIVE-ACCESS-COMPOSITION-CORPUS-V0.1";
pub const CLAIM_CEILING: &str = "synthetic_effective_access_and_delivery_composition_only";

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn object<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn tick(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn add(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn source_authorized(source: &Value) -> bool {
    !source.is_null()
        && !text(source, "owner").is_empty()
        && !text(source, "contract_id").is_empty()
        && !text(source, "contract_revision").is_empty()
        && !text(source, "receipt_id").is_empty()
        && valid_sha256(text(source, "content_sha256"))
        && text(source, "evidence_state") == "current"
        && text(source, "rights_state") == "permitted"
        && text(source, "issuer_state") == "authorized"
}

fn source_matches_scope(source: &Value, scope: &Value) -> bool {
    let window = object(scope, "observation_window");
    [
        ("service_id", "service_id"),
        ("service_revision", "service_revision"),
        ("subject_id", "subject_id"),
        ("subject_digest", "subject_digest"),
        ("functional_unit", "functional_unit"),
        ("group_id", "group_id"),
        ("population_frame_id", "population_frame_id"),
        ("scenario_id", "scenario_id"),
        ("common_scope_digest", "common_scope_digest"),
        ("operating_window_id", "operating_window_id"),
    ]
    .iter()
    .all(|(source_key, scope_key)| text(source, source_key) == text(scope, scope_key))
        && text(source, "observation_window_id") == text(window, "window_id")
        && tick(source, "start_tick") == tick(window, "start_tick")
        && tick(source, "end_tick") == tick(window, "end_tick")
        && text(source, "tick_unit") == text(window, "tick_unit")
}

fn validate_source(
    source: &Value,
    scope: &Value,
    lane: &str,
    reasons: &mut BTreeSet<String>,
) -> (bool, bool) {
    if !source_authorized(source) {
        add(reasons, &format!("{lane}_OWNER_EVIDENCE_INVALID_OR_NOT_CURRENT"));
        (false, false)
    } else if !source_matches_scope(source, scope) {
        add(reasons, &format!("{lane}_SOURCE_SCOPE_OR_TIME_NOT_COMPARABLE"));
        (true, false)
    } else {
        (true, true)
    }
}

fn scope_demand(scope: &Value) -> (&Value, f64, &str) {
    let demand = object(scope, "demand");
    (demand, number(demand, "value").unwrap_or(-1.0), text(demand, "unit"))
}

fn evaluate_restoration(input: &Value, scope: &Value, reasons: &mut BTreeSet<String>) -> String {
    let restoration = object(input, "restoration");
    let (authorized, comparable) = validate_source(
        object(restoration, "source_ref"),
        scope,
        "RESTORATION",
        reasons,
    );
    if !authorized {
        return "Invalid".to_owned();
    }
    if !comparable {
        return "NotComparable".to_owned();
    }
    if text(restoration, "service_id") != text(scope, "service_id")
        || text(restoration, "service_revision") != text(scope, "service_revision")
    {
        add(reasons, "RESTORATION_SERVICE_IDENTITY_MISMATCH");
        return "NotComparable".to_owned();
    }

    match text(restoration, "bridge_disposition") {
        "NotRestored" => "NotRestored".to_owned(),
        "Invalid" => "Invalid".to_owned(),
        "InsufficientEvidence" => "Unknown".to_owned(),
        "ProvisionalRecovery" => "ProvisionalRecovery".to_owned(),
        "RestoredForDeclaredScope" => {
            let stable = object(restoration, "stable_adequacy");
            let minimum = tick(stable, "minimum_ticks").unwrap_or(i64::MAX);
            let observed = tick(stable, "observed_ticks").unwrap_or(0);
            if restoration.get("qualified").and_then(Value::as_bool) == Some(true)
                && stable.get("qualified").and_then(Value::as_bool) == Some(true)
                && observed >= minimum
                && minimum > 0
            {
                "RestoredForDeclaredScope".to_owned()
            } else {
                "ProvisionalRecovery".to_owned()
            }
        }
        _ => {
            add(reasons, "RESTORATION_DISPOSITION_UNSUPPORTED");
            "Unknown".to_owned()
        }
    }
}

fn evaluate_access(input: &Value, scope: &Value, reasons: &mut BTreeSet<String>) -> String {
    let access = object(input, "access");
    let (authorized, comparable) =
        validate_source(object(access, "source_ref"), scope, "ACCESS", reasons);
    if !authorized {
        return "Unknown".to_owned();
    }
    if !comparable {
        return "NotComparable".to_owned();
    }
    let group = object(access, "group");
    let opportunity = object(access, "opportunity");
    let (demand, demand_value, demand_unit) = scope_demand(scope);
    let frame_matches = text(group, "group_id") == text(scope, "group_id")
        && text(group, "population_frame_id") == text(scope, "population_frame_id")
        && group.get("population").and_then(Value::as_u64)
            == scope.get("population").and_then(Value::as_u64);
    if !frame_matches {
        add(reasons, "ACCESS_BENEFICIARY_FRAME_MISMATCH");
        return "NotComparable".to_owned();
    }
    if number(group, "demand") != Some(demand_value)
        || text(group, "demand_unit") != demand_unit
        || demand.is_null()
    {
        add(reasons, "ACCESS_DEMAND_DENOMINATOR_MISMATCH");
        return "NotComparable".to_owned();
    }
    if text(opportunity, "service_id") != text(scope, "service_id") {
        add(reasons, "ACCESS_SERVICE_IDENTITY_MISMATCH");
        return "NotComparable".to_owned();
    }
    if text(opportunity, "operating_window_id") != text(scope, "operating_window_id") {
        add(reasons, "ACCESS_OPERATING_WINDOW_MISMATCH");
        return "NotComparable".to_owned();
    }
    if text(opportunity, "capacity_unit") != demand_unit {
        add(reasons, "ACCESS_CAPACITY_DEMAND_UNIT_MISMATCH");
        return "NotComparable".to_owned();
    }

    let capacity = number(opportunity, "reachable_capacity_potential");
    if let Some(owner_codes) = access.get("owner_reason_codes").and_then(Value::as_array) {
        for owner_code in owner_codes.iter().filter_map(Value::as_str) {
            if matches!(
                owner_code,
                "NETWORK_EDGE_UNKNOWN"
                    | "NETWORK_EVIDENCE_STALE"
                    | "SERVICE_AVAILABILITY_UNKNOWN"
                    | "SERVICE_PRICE_UNKNOWN"
                    | "SERVICE_ELIGIBILITY_UNKNOWN"
                    | "SERVICE_STEP_FREE_UNKNOWN"
                    | "SERVICE_CAPACITY_EVIDENCE_UNKNOWN"
            ) {
                add(reasons, owner_code);
            } else {
                add(reasons, "ACCESS_OWNER_REASON_CODE_UNSUPPORTED");
            }
        }
    }

    match text(opportunity, "status") {
        "Inaccessible" => "Inaccessible".to_owned(),
        "Unknown" => "Unknown".to_owned(),
        "AccessibleButCapacityConstrained" => {
            if capacity.is_some_and(|value| value < demand_value) {
                "AccessibleButCapacityConstrained".to_owned()
            } else {
                add(reasons, "ACCESS_CAPACITY_STATUS_INCONSISTENT");
                "Unknown".to_owned()
            }
        }
        "AccessibleAndAdequatePotential" => {
            if capacity.is_some_and(|value| value >= demand_value) && demand_value > 0.0 {
                "AccessibleAndAdequatePotential".to_owned()
            } else {
                add(reasons, "ACCESS_CAPACITY_STATUS_INCONSISTENT");
                "Unknown".to_owned()
            }
        }
        _ => {
            add(reasons, "ACCESS_STATUS_UNSUPPORTED");
            "Unknown".to_owned()
        }
    }
}

fn evaluate_in_use(input: &Value, scope: &Value, reasons: &mut BTreeSet<String>) -> String {
    let in_use = object(input, "in_use");
    let (authorized, comparable) =
        validate_source(object(in_use, "source_ref"), scope, "IN_USE", reasons);
    if !authorized {
        return "Invalid".to_owned();
    }
    if !comparable {
        return "NotComparable".to_owned();
    }
    if text(in_use, "service_id") != text(scope, "service_id") {
        add(reasons, "IN_USE_SERVICE_IDENTITY_MISMATCH");
        return "NotComparable".to_owned();
    }
    let metric = object(in_use, "metric");
    let floor = object(scope, "service_floor");
    if text(metric, "metric_id") != text(floor, "metric_id")
        || text(metric, "unit") != text(floor, "unit")
        || text(metric, "direction") != text(floor, "direction")
        || number(metric, "threshold") != number(floor, "threshold")
    {
        add(reasons, "IN_USE_SERVICE_FLOOR_DEFINITION_MISMATCH");
        return "NotComparable".to_owned();
    }
    if text(in_use, "observation_window_id")
        != text(object(scope, "observation_window"), "window_id")
    {
        add(reasons, "IN_USE_OBSERVATION_WINDOW_MISMATCH");
        return "NotComparable".to_owned();
    }
    if text(in_use, "evaluation_state") != "Evaluated" {
        return "Unknown".to_owned();
    }
    match text(metric, "threshold_state") {
        "Pass" if text(metric, "observation_state") == "Current" => "Pass".to_owned(),
        "Fail" if text(metric, "observation_state") == "Current" => "Fail".to_owned(),
        "Unknown" | "NotComparable" => "Unknown".to_owned(),
        _ => {
            add(reasons, "IN_USE_OBSERVATION_STATE_UNSUPPORTED");
            "Invalid".to_owned()
        }
    }
}

fn evaluate_delivery(
    input: &Value,
    scope: &Value,
    reasons: &mut BTreeSet<String>,
) -> (String, Option<f64>, Option<f64>, String) {
    let delivery = object(input, "beneficiary_delivery");
    if delivery.is_null() {
        return ("NotProvided".to_owned(), None, None, "NotProvided".to_owned());
    }
    let (authorized, comparable) = validate_source(
        object(delivery, "source_ref"),
        scope,
        "DELIVERY",
        reasons,
    );
    if !authorized {
        return ("Invalid".to_owned(), None, None, "Unknown".to_owned());
    }
    if !comparable {
        return ("Unknown".to_owned(), None, None, "Unknown".to_owned());
    }
    let (demand, demand_value, demand_unit) = scope_demand(scope);
    if text(delivery, "service_id") != text(scope, "service_id")
        || text(delivery, "group_id") != text(scope, "group_id")
        || text(delivery, "population_frame_id") != text(scope, "population_frame_id")
    {
        add(reasons, "DELIVERY_BENEFICIARY_FRAME_MISMATCH");
        return ("Unknown".to_owned(), None, None, "Unknown".to_owned());
    }
    if text(delivery, "demand_unit") != demand_unit
        || text(delivery, "delivered_unit") != demand_unit
        || number(delivery, "demand") != Some(demand_value)
        || demand.is_null()
    {
        add(reasons, "DELIVERY_DEMAND_DENOMINATOR_OR_UNIT_MISMATCH");
        return ("Unknown".to_owned(), None, None, "Unknown".to_owned());
    }
    if text(delivery, "window_id")
        != text(object(scope, "observation_window"), "window_id")
    {
        add(reasons, "DELIVERY_OBSERVATION_WINDOW_MISMATCH");
        return ("Unknown".to_owned(), None, None, "Unknown".to_owned());
    }
    if text(delivery, "evidence_state") != "current" {
        return ("Unknown".to_owned(), None, None, "Unknown".to_owned());
    }
    let delivered = number(delivery, "delivered");
    if delivered.is_none_or(|value| value < 0.0) || demand_value <= 0.0 {
        add(reasons, "DELIVERY_QUANTITY_INVALID");
        return ("Invalid".to_owned(), None, None, "Unknown".to_owned());
    }
    let group_coverage = delivered.map(|value| value / demand_value);

    let aggregate_demand = number(delivery, "aggregate_demand");
    let aggregate_delivered = number(delivery, "aggregate_delivered");
    let aggregate_state = match (aggregate_demand, aggregate_delivered) {
        (Some(demand), Some(received)) if demand > 0.0 && received >= 0.0 => {
            if received >= demand { "AggregateMeetsDemand" } else { "AggregateBelowDemand" }
        }
        (None, None) => "NotProvided",
        _ => {
            add(reasons, "AGGREGATE_DELIVERY_QUANTITY_INVALID");
            "Unknown"
        }
    };
    let group_state = if delivered.unwrap_or(0.0) >= demand_value {
        "Observed"
    } else {
        "BelowDemand"
    };
    (
        group_state.to_owned(),
        group_coverage,
        match (aggregate_demand, aggregate_delivered) {
            (Some(demand), Some(received)) if demand > 0.0 => Some(received / demand),
            _ => None,
        },
        aggregate_state.to_owned(),
    )
}

pub fn evaluate_case(case: &Value) -> Value {
    let input = object(case, "input");
    let scope = object(input, "scope");
    let mut reasons = BTreeSet::<String>::new();

    let restoration_state = evaluate_restoration(input, scope, &mut reasons);
    let access_state = evaluate_access(input, scope, &mut reasons);
    let operational_service_state = evaluate_in_use(input, scope, &mut reasons);
    let (delivery_state, group_coverage, aggregate_coverage, aggregate_delivery_state) =
        evaluate_delivery(input, scope, &mut reasons);

    let experience = object(input, "experience");
    let experience_disclosure_state = if experience.is_null()
        || experience.get("requested").and_then(Value::as_bool) != Some(true)
    {
        "NotRequested"
    } else if text(experience, "consent_state") == "authorized"
        && text(experience, "purpose_state") == "authorized"
        && text(experience, "disclosure_state") == "permitted"
    {
        "Permitted"
    } else if matches!(
        text(experience, "consent_state"),
        "denied" | "not_granted"
    ) || text(experience, "disclosure_state") == "denied"
    {
        "Restricted"
    } else {
        "Unknown"
    };

    let invalid = reasons.iter().any(|reason| reason.ends_with("_OWNER_EVIDENCE_INVALID_OR_NOT_CURRENT")
        || reason == "ACCESS_CAPACITY_STATUS_INCONSISTENT"
        || reason == "IN_USE_OBSERVATION_STATE_UNSUPPORTED"
        || reason == "DELIVERY_QUANTITY_INVALID");
    let composition_disposition = if invalid || restoration_state == "Invalid" {
        "Invalid"
    } else if restoration_state == "NotRestored" {
        "NotRestored"
    } else if restoration_state == "ProvisionalRecovery" {
        "ProvisionalRecovery"
    } else if restoration_state != "RestoredForDeclaredScope"
        || access_state == "Unknown"
        || access_state == "NotComparable"
        || operational_service_state == "Unknown"
        || operational_service_state == "NotComparable"
        || operational_service_state == "Invalid"
    {
        "InsufficientEvidence"
    } else if access_state == "Inaccessible" {
        "Inaccessible"
    } else if access_state == "AccessibleButCapacityConstrained" {
        "CapacityConstrained"
    } else if operational_service_state == "Fail" {
        "OperationalServiceBelowFloor"
    } else if operational_service_state != "Pass" {
        "InsufficientEvidence"
    } else if !object(input, "beneficiary_delivery").is_null() && delivery_state == "Unknown" {
        "InsufficientEvidence"
    } else if delivery_state == "BelowDemand" {
        "BeneficiaryDeliveryBelowDemand"
    } else if delivery_state == "Observed" {
        "RestoredAndPotentiallyAccessibleWithDeliveryEvidence"
    } else {
        "RestoredAndPotentiallyAccessibleOnly"
    };

    json!({
        "case_id": text(case, "case_id"),
        "restoration_state": restoration_state,
        "potential_access_state": access_state,
        "operational_service_state": operational_service_state,
        "beneficiary_delivery_state": delivery_state,
        "group_delivery_coverage_fraction": group_coverage,
        "aggregate_delivery_state": aggregate_delivery_state,
        "aggregate_delivery_coverage_fraction": aggregate_coverage,
        "experience_disclosure_state": experience_disclosure_state,
        "composition_disposition": composition_disposition,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING
    })
}

fn check_keys(value: &Value, allowed: &[&str], path: &str, errors: &mut Vec<String>) {
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

pub fn validate_corpus(corpus: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check_keys(
        corpus,
        &["schema_id", "schema_version", "corpus_id", "classification", "policy_note", "claim_ceiling", "cases"],
        "corpus",
        &mut errors,
    );
    if text(corpus, "schema_id")
        != "org.luminousdynamics.city-effective-access-composition.synthetic-corpus"
        || text(corpus, "schema_version") != "0.1.0"
        || text(corpus, "corpus_id") != CORPUS_ID
        || text(corpus, "classification") != "synthetic_fixture_only"
        || text(corpus, "claim_ceiling") != CLAIM_CEILING
        || text(corpus, "policy_note").is_empty()
    {
        errors.push("CORPUS_IDENTITY_OR_CLAIM_CEILING_INVALID".to_owned());
    }
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        errors.push("CORPUS_CASES_MISSING".to_owned());
        return errors;
    };
    if cases.len() != 28 {
        errors.push("CORPUS_CASE_COUNT_NOT_28".to_owned());
    }
    let mut ids = BTreeSet::new();
    for (index, case) in cases.iter().enumerate() {
        check_keys(case, &["case_id", "input", "expected"], &format!("cases[{index}]"), &mut errors);
        let id = text(case, "case_id");
        if id.is_empty() || !ids.insert(id.to_owned()) {
            errors.push(format!("cases[{index}]:EMPTY_OR_DUPLICATE_CASE_ID"));
        }
        check_keys(
            object(case, "input"),
            &["scope", "restoration", "access", "in_use", "beneficiary_delivery", "experience"],
            &format!("cases[{index}].input"),
            &mut errors,
        );
        if !case.get("expected").is_some_and(Value::is_object) {
            errors.push(format!("cases[{index}]:EXPECTED_OBJECT_MISSING"));
        }
    }
    errors.sort();
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS: &str = include_str!(
        "../../../docs/research/city-effective-access-composition/CITY_EFFECTIVE_ACCESS_COMPOSITION_CORPUS_V0_1.json"
    );

    fn corpus() -> Value {
        serde_json::from_str(CORPUS).expect("frozen composition corpus parses")
    }

    #[test]
    fn all_28_frozen_scenarios_match_exactly() {
        let corpus = corpus();
        let errors = validate_corpus(&corpus);
        assert!(errors.is_empty(), "{errors:?}");
        for case in corpus["cases"].as_array().unwrap() {
            let actual = evaluate_case(case);
            assert_eq!(
                actual,
                case["expected"],
                "{} actual={} expected={}",
                text(case, "case_id"),
                actual,
                case["expected"]
            );
        }
    }

    #[test]
    fn reachable_capacity_is_never_promoted_to_beneficiary_delivery() {
        let case = corpus()["cases"][0].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["potential_access_state"], "AccessibleAndAdequatePotential");
        assert_eq!(result["beneficiary_delivery_state"], "NotProvided");
        assert_eq!(result["group_delivery_coverage_fraction"], Value::Null);
        assert_eq!(result["composition_disposition"], "RestoredAndPotentiallyAccessibleOnly");
    }

    #[test]
    fn subgroup_shortfall_is_not_hidden_by_aggregate_delivery() {
        let case = corpus()["cases"][22].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["aggregate_delivery_state"], "AggregateMeetsDemand");
        assert_eq!(result["beneficiary_delivery_state"], "BelowDemand");
        assert_eq!(result["composition_disposition"], "BeneficiaryDeliveryBelowDemand");
    }

    #[test]
    fn experience_disclosure_is_independent_of_technical_service() {
        let case = corpus()["cases"][23].clone();
        let result = evaluate_case(&case);
        assert_eq!(result["experience_disclosure_state"], "Restricted");
        assert_eq!(result["composition_disposition"], "RestoredAndPotentiallyAccessibleOnly");
    }

    #[test]
    fn unknown_input_fields_fail_closed() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["unreviewed_extension"] = json!(true);
        assert!(!validate_corpus(&corpus_with_case(case)).is_empty());
    }

    fn corpus_with_case(case: Value) -> Value {
        let mut corpus = corpus();
        corpus["cases"][0] = case;
        corpus
    }
}
