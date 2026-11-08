//! Type-safe routing of CityMaker changes to existing owner impact lanes.
//!
//! This is a registry and binding adapter, not an impact engine. It does not
//! compute dependencies, verify report contents, authenticate owners, or
//! grant approval. A contract-bound result remains explicitly unverified.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const REGISTRY_ID: &str = "citymaker-owner-impact-lanes-2026-11-08";
const CLAIM_CEILING: &str = "synthetic owner-lane routing and declared contract-binding semantics only; no owner authentication, report-content verification, impact correctness, source completeness, engineering applicability or action authority";

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

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| {
            byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
        })
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

fn registry_validation_reasons(registry: &Value) -> BTreeSet<String> {
    let mut reasons = BTreeSet::new();
    if text(registry, "schema_version") != "luminous.city-impact-owner-lane-registry.v0.1"
        || text(registry, "registry_id") != REGISTRY_ID
        || text(registry, "claim_ceiling") != "routing_and_contract_binding_only"
        || text(registry, "action_authority") != "none"
    {
        reasons.insert("OWNER_LANE_REGISTRY_IDENTITY_INVALID".to_owned());
    }

    let mut families = BTreeSet::new();
    for lane in array(registry, "lanes") {
        let family = text(lane, "family");
        if family.is_empty() || !families.insert(family.to_owned()) {
            reasons.insert("OWNER_LANE_REGISTRY_DUPLICATE_OR_EMPTY_FAMILY".to_owned());
        }
        match text(lane, "readiness") {
            "implementation_pinned_open_pr" => {
                if text(lane, "contract_ref").is_empty()
                    || !is_lower_hex(text(lane, "contract_revision"), 40)
                {
                    reasons.insert("OWNER_LANE_CONTRACT_PIN_INCOMPLETE".to_owned());
                }
            }
            "schema_and_runtime_pinned_open_pr" => {
                if text(lane, "contract_ref").is_empty()
                    || !is_lower_hex(text(lane, "contract_revision"), 40)
                    || text(lane, "schema_id").is_empty()
                    || text(lane, "schema_version").is_empty()
                    || !is_lower_hex(text(lane, "runtime_revision"), 40)
                {
                    reasons.insert("OWNER_LANE_CONTRACT_PIN_INCOMPLETE".to_owned());
                }
            }
            "schema_incompatible" | "design_only" | "benchmark_or_design_only" => {}
            _ => {
                reasons.insert("OWNER_LANE_READINESS_UNKNOWN".to_owned());
            }
        }
    }

    let mut change_classes = BTreeSet::new();
    for route in array(registry, "routes") {
        let change_class = text(route, "change_class");
        if change_class.is_empty() || !change_classes.insert(change_class.to_owned()) {
            reasons.insert("OWNER_LANE_REGISTRY_DUPLICATE_OR_EMPTY_ROUTE".to_owned());
        }
        let mut required = BTreeSet::new();
        if array(route, "required_families").is_empty() {
            reasons.insert("OWNER_ROUTE_HAS_NO_REQUIRED_FAMILIES".to_owned());
        }
        for family in array(route, "required_families").iter().filter_map(Value::as_str) {
            if !required.insert(family.to_owned()) {
                reasons.insert("OWNER_ROUTE_DUPLICATES_REQUIRED_FAMILY".to_owned());
            }
            if !families.contains(family) {
                reasons.insert("OWNER_ROUTE_REFERENCES_UNKNOWN_LANE".to_owned());
            }
        }
    }
    reasons
}

fn lane_definition<'a>(registry: &'a Value, family: &str) -> Option<&'a Value> {
    array(registry, "lanes")
        .iter()
        .find(|lane| text(lane, "family") == family)
}

fn route_definition<'a>(registry: &'a Value, change_class: &str) -> Option<&'a Value> {
    array(registry, "routes")
        .iter()
        .find(|route| text(route, "change_class") == change_class)
}

fn required_families(
    registry: &Value,
    event: &Value,
    query: &Value,
) -> Result<Vec<String>, String> {
    let change_class = text(event, "change_class");
    let Some(route) = route_definition(registry, change_class) else {
        return Err("CHANGE_CLASS_HAS_NO_REGISTERED_OWNER_ROUTE".to_owned());
    };
    let mut required = array(route, "required_families")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if bool_value(query, "requires_no_impact_proof") == Some(true) {
        for rule in array(registry, "dynamic_rules") {
            if text(rule, "rule_id") == "NO_IMPACT_REQUIRES_INVENTORY_AND_DEPENDENCY_COVERAGE" {
                for family in array(rule, "additionally_required_families")
                    .iter()
                    .filter_map(Value::as_str)
                {
                    if !required.iter().any(|item| item == family) {
                        required.push(family.to_owned());
                    }
                }
            }
        }
    }
    let mut unique = BTreeSet::new();
    if required.iter().any(|family| !unique.insert(family.clone())) {
        return Err("OWNER_ROUTE_DUPLICATES_REQUIRED_FAMILY".to_owned());
    }
    Ok(required)
}

fn expected_nullable_string_matches(report: &Value, field: &str, lane: &Value) -> bool {
    match lane.get(field) {
        Some(Value::Null) => report.get(field).is_none_or(Value::is_null),
        Some(Value::String(expected)) => text(report, field) == expected,
        _ => false,
    }
}

fn evaluate_lane(
    lane: &Value,
    family: &str,
    report: Option<&Value>,
    event: &Value,
    query: &Value,
) -> Value {
    let readiness = text(lane, "readiness");
    let mut reasons = BTreeSet::<String>::new();

    if readiness == "schema_incompatible" {
        reasons.insert("OWNER_SCHEMA_GRAPH_VERSION_INCOMPATIBLE".to_owned());
        if number(lane, "blocker_issue").is_some() {
            reasons.insert("UPSTREAM_SCHEMA_COMPATIBILITY_WORK_OPEN".to_owned());
        }
        return json!({
            "family": family,
            "state": "insufficient_evidence",
            "readiness": readiness,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "owner_report_content_verified": false,
            "owner_authentication": "not_performed"
        });
    }

    if !matches!(
        readiness,
        "implementation_pinned_open_pr" | "schema_and_runtime_pinned_open_pr"
    ) {
        reasons.insert("OWNER_LANE_NOT_VERSIONED_OR_NOT_INTEGRATED".to_owned());
        return json!({
            "family": family,
            "state": "insufficient_evidence",
            "readiness": readiness,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "owner_report_content_verified": false,
            "owner_authentication": "not_performed"
        });
    }

    let Some(report) = report else {
        reasons.insert("OWNER_REPORT_MISSING".to_owned());
        return json!({
            "family": family,
            "state": "insufficient_evidence",
            "readiness": readiness,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "owner_report_content_verified": false,
            "owner_authentication": "not_performed"
        });
    };

    if text(report, "impact_family") != family {
        reasons.insert("OWNER_REPORT_IMPACT_FAMILY_MISMATCH".to_owned());
    }
    let expected_report_kind = if text(lane, "report_shape").is_empty() {
        text(lane, "contract_kind")
    } else {
        text(lane, "report_shape")
    };
    if text(report, "report_kind") != expected_report_kind {
        reasons.insert("OWNER_REPORT_KIND_MISMATCH".to_owned());
    }
    if text(report, "owner_repo") != text(lane, "owner_repo") {
        reasons.insert("OWNER_REPOSITORY_MISMATCH".to_owned());
    }
    if number(report, "owner_issue") != number(lane, "owner_issue") {
        reasons.insert("OWNER_ISSUE_BINDING_MISMATCH".to_owned());
    }
    if text(report, "contract_ref") != text(lane, "contract_ref")
        || text(report, "contract_revision") != text(lane, "contract_revision")
    {
        reasons.insert("OWNER_CONTRACT_REFERENCE_OR_REVISION_MISMATCH".to_owned());
    }
    if lane.get("schema_id").is_some()
        && !expected_nullable_string_matches(report, "schema_id", lane)
    {
        reasons.insert("OWNER_SCHEMA_ID_MISMATCH".to_owned());
    }
    if lane.get("schema_version").is_some()
        && !expected_nullable_string_matches(report, "schema_version", lane)
    {
        reasons.insert("OWNER_SCHEMA_VERSION_MISMATCH".to_owned());
    }
    if lane.get("runtime_revision").is_some()
        && text(report, "runtime_revision") != text(lane, "runtime_revision")
    {
        reasons.insert("OWNER_RUNTIME_REVISION_MISMATCH".to_owned());
    }
    if text(report, "event_id") != text(event, "event_id")
        || text(report, "event_sha256") != text(event, "event_sha256")
    {
        reasons.insert("OWNER_REPORT_EVENT_BINDING_MISMATCH".to_owned());
    }
    if text(report, "subject_node_id") != text(event, "subject_node_id")
        || text(report, "subject_revision") != text(event, "subject_revision")
    {
        reasons.insert("OWNER_REPORT_SUBJECT_BINDING_MISMATCH".to_owned());
    }
    if report.get("valid_time") != event.get("valid_time")
        || report.get("system_time") != event.get("system_time")
    {
        reasons.insert("OWNER_REPORT_TEMPORAL_BINDING_MISMATCH".to_owned());
    }
    if text(report, "scope_id") != text(query, "scope_id")
        || text(report, "scope_digest") != text(query, "scope_digest")
        || text(report, "functional_unit") != text(query, "functional_unit")
    {
        reasons.insert("OWNER_REPORT_SCOPE_OR_FUNCTIONAL_UNIT_MISMATCH".to_owned());
    }
    if text(report, "evaluated_at") != text(query, "evaluation_at") {
        reasons.insert("OWNER_REPORT_EVALUATION_TIME_MISMATCH".to_owned());
    }
    let digest = text(report, "report_sha256");
    if !is_lower_hex(digest, 64)
        || text(report, "report_reference") != format!("cas:sha256:{digest}")
    {
        reasons.insert("OWNER_REPORT_DIGEST_REFERENCE_INVALID".to_owned());
    }
    if text(report, "action_authority") != "none" {
        reasons.insert("OWNER_REPORT_ACTION_AUTHORITY_ESCALATION".to_owned());
    }

    if family == "dependency_coverage" {
        if text(report, "owner_disposition") != "complete_for_declared_scope"
            || text(report, "claim_ceiling") != "coverage_only"
            || text(report, "independent_review") != "accepted_for_declared_scope"
            || number(report, "unresolved_relationship_count") != Some(0)
            || number(report, "unsupported_relationship_count") != Some(0)
        {
            reasons.insert("DEPENDENCY_COVERAGE_NOT_QUALIFIED_FOR_DECLARED_SCOPE".to_owned());
        }
    }

    let state = if reasons.is_empty() {
        "contract_bound_unverified"
    } else if reasons.iter().any(|reason| {
        reason.contains("MISMATCH")
            || reason.contains("INVALID")
            || reason.contains("ESCALATION")
    }) {
        "invalid_input"
    } else {
        "insufficient_evidence"
    };
    json!({
        "family": family,
        "state": state,
        "readiness": readiness,
        "owner_report_id": text(report, "report_id"),
        "owner_report_sha256": digest,
        "report_reference": text(report, "report_reference"),
        "reason_codes": if reasons.is_empty() {
            vec!["OWNER_REPORT_CONTENT_AND_AUTHORITY_NOT_AUTHENTICATED_HERE"]
        } else {
            reasons.iter().map(String::as_str).collect::<Vec<_>>()
        },
        "owner_report_content_verified": false,
        "owner_authentication": "not_performed",
        "action_authority": "none"
    })
}

/// Evaluates the routing contract from registry and input only. Expected output
/// values are never consulted.
pub fn evaluate_case(case: &Value, registry: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let event = input.get("event").unwrap_or(&Value::Null);
    let query = input.get("query").unwrap_or(&Value::Null);
    let reports = array(input, "owner_reports");
    let mut global_reasons = BTreeSet::<String>::new();

    global_reasons.extend(registry_validation_reasons(registry));
    if text(event, "event_id").is_empty()
        || !is_lower_hex(text(event, "event_sha256"), 64)
        || text(event, "subject_node_id").is_empty()
        || text(event, "subject_revision").is_empty()
        || text(event, "change_class").is_empty()
        || text(event, "time_basis") != "UTC_RFC3339_SECONDS"
    {
        global_reasons.insert("CHANGE_EVENT_BINDING_INCOMPLETE".to_owned());
    }
    let valid_time = event.get("valid_time").unwrap_or(&Value::Null);
    let effective_at = text(valid_time, "effective_at");
    let effective_until = text(valid_time, "effective_until");
    let confidence = text(valid_time, "effective_time_confidence");
    if !valid_utc_timestamp(effective_at)
        || valid_time.get("effective_until").is_none()
        || (!valid_time.get("effective_until").is_some_and(Value::is_null)
            && !valid_utc_timestamp(effective_until))
        || !matches!(
            confidence,
            "verified" | "authority_asserted" | "unknown" | "disputed"
        )
    {
        global_reasons.insert("CHANGE_EVENT_TEMPORAL_BINDING_INCOMPLETE".to_owned());
    } else {
        if confidence != "verified" {
            global_reasons.insert("EVENT_EFFECTIVE_TIME_UNQUALIFIED".to_owned());
        }
        if !effective_until.is_empty() && effective_until < effective_at {
            global_reasons.insert("EVENT_VALID_TIME_WINDOW_INVALID".to_owned());
        }
    }
    let system_time = event.get("system_time").unwrap_or(&Value::Null);
    let recorded_at = text(system_time, "recorded_at");
    let trusted_at_value = system_time.get("trusted_at");
    let trusted_receipt_value = system_time.get("trusted_timestamp_receipt_sha256");
    let trusted_at = trusted_at_value.and_then(Value::as_str).unwrap_or("");
    let trusted_receipt = trusted_receipt_value.and_then(Value::as_str).unwrap_or("");
    let explicitly_untrusted = trusted_at_value.is_some_and(Value::is_null)
        && trusted_receipt_value.is_some_and(Value::is_null);
    let trusted_pair_valid = valid_utc_timestamp(trusted_at)
        && is_lower_hex(trusted_receipt, 64);
    if system_time.get("recorded_at").is_none()
        || trusted_at_value.is_none()
        || trusted_receipt_value.is_none()
        || !valid_utc_timestamp(recorded_at)
        || (!explicitly_untrusted && !trusted_pair_valid)
    {
        global_reasons.insert("CHANGE_EVENT_SYSTEM_TIME_BINDING_INCOMPLETE".to_owned());
    } else if explicitly_untrusted {
        global_reasons.insert("EVENT_SYSTEM_TIME_UNTRUSTED".to_owned());
    } else if trusted_at < recorded_at {
        global_reasons.insert("EVENT_TRUST_TIME_ORDER_INVALID".to_owned());
    }
    if text(query, "scope_id").is_empty()
        || !is_lower_hex(text(query, "scope_digest"), 64)
        || text(query, "functional_unit").is_empty()
        || !valid_utc_timestamp(text(query, "evaluation_at"))
    {
        global_reasons.insert("CITY_QUERY_SCOPE_BINDING_INCOMPLETE".to_owned());
    }

    let required = match required_families(registry, event, query) {
        Ok(value) => value,
        Err(reason) => {
            global_reasons.insert(reason);
            Vec::new()
        }
    };
    let required_set = required.iter().cloned().collect::<BTreeSet<_>>();

    let mut family_counts = BTreeMap::<String, usize>::new();
    for report in reports {
        let family = text(report, "impact_family");
        *family_counts.entry(family.to_owned()).or_default() += 1;
        if !required_set.contains(family) {
            global_reasons.insert("UNEXPECTED_OWNER_REPORT_IMPACT_FAMILY".to_owned());
        }
    }

    let mut lane_results = Vec::with_capacity(required.len());
    for family in &required {
        let Some(lane) = lane_definition(registry, family) else {
            global_reasons.insert("OWNER_LANE_NOT_REGISTERED".to_owned());
            lane_results.push(json!({
                "family": family,
                "state": "insufficient_evidence",
                "reason_codes": ["OWNER_LANE_NOT_REGISTERED"],
                "owner_report_content_verified": false,
                "owner_authentication": "not_performed"
            }));
            continue;
        };
        let count = family_counts.get(family).copied().unwrap_or(0);
        let report = reports
            .iter()
            .find(|item| text(item, "impact_family") == family);
        if count > 1 {
            lane_results.push(json!({
                "family": family,
                "state": "invalid_input",
                "readiness": text(lane, "readiness"),
                "reason_codes": ["DUPLICATE_OWNER_REPORT_FOR_FAMILY"],
                "owner_report_content_verified": false,
                "owner_authentication": "not_performed"
            }));
            continue;
        }
        lane_results.push(evaluate_lane(lane, family, report, event, query));
    }

    if !global_reasons.is_empty() {
        let lane_has_invalid = lane_results
            .iter()
            .any(|lane| text(lane, "state") == "invalid_input");
        let global_has_invalid = global_reasons.iter().any(|reason| {
            reason.contains("INVALID")
                || reason.contains("ESCALATION")
                || reason.contains("UNEXPECTED")
        });
        let route_state = if lane_has_invalid || global_has_invalid {
            "invalid_input"
        } else {
            "insufficient_evidence"
        };
        return json!({
            "case_id": case_id,
            "route_state": route_state,
            "required_families": required,
            "lane_results": lane_results,
            "reason_codes": global_reasons.into_iter().collect::<Vec<_>>(),
            "owner_report_content_verified": false,
            "owner_authentication": "not_performed",
            "action_authority": "none",
            "claim_ceiling": CLAIM_CEILING
        });
    }

    let has_invalid = lane_results
        .iter()
        .any(|lane| text(lane, "state") == "invalid_input");
    let has_insufficient = lane_results
        .iter()
        .any(|lane| text(lane, "state") == "insufficient_evidence");
    let route_state = if has_invalid {
        "invalid_input"
    } else if has_insufficient {
        "insufficient_evidence"
    } else {
        "contract_bound_unverified"
    };

    json!({
        "case_id": case_id,
        "route_state": route_state,
        "required_families": required,
        "lane_results": lane_results,
        "reason_codes": [],
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
            left.get(key)
                .is_some_and(|actual_value| equivalent(actual_value, value))
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
    use serde_json::{Value, json};
    use std::path::PathBuf;

    fn load_json(path: &str) -> Value {
        let full = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
        let bytes = std::fs::read(full).expect("fixture file must exist");
        serde_json::from_slice(&bytes).expect("fixture JSON must parse")
    }

    fn registry() -> Value {
        load_json(
            "../../docs/research/city-impact-router/CITY_IMPACT_OWNER_LANE_REGISTRY_V0_1.json",
        )
    }

    fn corpus() -> Value {
        load_json("../../docs/research/city-impact-router/CITY_IMPACT_LANE_ROUTER_CORPUS_V0_1.json")
    }

    #[test]
    fn registry_contains_unique_families_and_routes() {
        let r = registry();
        let lanes = r["lanes"].as_array().unwrap();
        let families = lanes
            .iter()
            .map(|lane| lane["family"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(families.len(), lanes.len());
        let routes = r["routes"].as_array().unwrap();
        let classes = routes
            .iter()
            .map(|route| route["change_class"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(classes.len(), routes.len());
    }

    #[test]
    fn all_known_answer_route_cases_match() {
        let r = registry();
        for case in corpus()["cases"].as_array().unwrap() {
            let actual = evaluate_case(case, &r);
            let actual_lane_states = actual["lane_results"]
                .as_array()
                .unwrap()
                .iter()
                .map(|lane| {
                    json!({
                        "family": lane["family"],
                        "state": lane["state"]
                    })
                })
                .collect::<Vec<_>>();
            let actual_projection = json!({
                "route_state": actual["route_state"],
                "required_families": actual["required_families"],
                "lane_states": actual_lane_states,
                "reason_codes": actual["reason_codes"]
            });
            assert!(
                matches_expected(&actual_projection, &case["expected"]),
                "{} mismatch\\nactual: {}\\nexpected: {}",
                case["case_id"].as_str().unwrap(),
                serde_json::to_string_pretty(&actual_projection).unwrap(),
                serde_json::to_string_pretty(&case["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn a_structurally_valid_but_wrong_lane_report_is_rejected() {
        let r = registry();
        let mut case = corpus()["cases"][0].clone();
        case["input"]["owner_reports"][0]["impact_family"] =
            Value::String("source_evidence_lifecycle".into());
        let result = evaluate_case(&case, &r);
        assert_eq!(result["route_state"], "invalid_input");
    }

    #[test]
    fn no_impact_adds_source_inventory_and_dependency_coverage() {
        let r = registry();
        let mut case = corpus()["cases"][0].clone();
        case["input"]["query"]["requires_no_impact_proof"] = Value::Bool(true);
        let result = evaluate_case(&case, &r);
        let required = result["required_families"].as_array().unwrap();
        assert!(required.iter().any(|value| value == "source_inventory"));
        assert_eq!(result["route_state"], "insufficient_evidence");
    }

    #[test]
    fn router_corpus_contains_32_unique_cases() {
        let cases = corpus()["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 35);
        let ids = cases
            .iter()
            .map(|case| case["case_id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 35);
    }

    #[test]
    fn bitemporal_event_controls_fail_closed() {
        let r = registry();
        let cases = corpus()["cases"].as_array().unwrap();

        let missing_recorded = evaluate_case(&cases[28], &r);
        assert_eq!(missing_recorded["route_state"], "insufficient_evidence");
        assert!(
            missing_recorded["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "CHANGE_EVENT_SYSTEM_TIME_BINDING_INCOMPLETE")
        );

        let disputed_effective_time = evaluate_case(&cases[29], &r);
        assert_eq!(
            disputed_effective_time["route_state"],
            "insufficient_evidence"
        );
        assert!(
            disputed_effective_time["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "EVENT_EFFECTIVE_TIME_UNQUALIFIED")
        );

        let trusted_time_reversal = evaluate_case(&cases[30], &r);
        assert_eq!(trusted_time_reversal["route_state"], "invalid_input");
        assert!(
            trusted_time_reversal["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "EVENT_TRUST_TIME_ORDER_INVALID")
        );

        let report_time_mismatch = evaluate_case(&cases[31], &r);
        assert_eq!(report_time_mismatch["route_state"], "invalid_input");
        assert_eq!(
            report_time_mismatch["lane_results"][0]["state"],
            "invalid_input"
        );

        let explicitly_untrusted = evaluate_case(&cases[32], &r);
        assert_eq!(explicitly_untrusted["route_state"], "insufficient_evidence");
        assert!(
            explicitly_untrusted["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "EVENT_SYSTEM_TIME_UNTRUSTED")
        );

        let missing_trust_receipt = evaluate_case(&cases[33], &r);
        assert_eq!(missing_trust_receipt["route_state"], "insufficient_evidence");
        assert!(
            missing_trust_receipt["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "CHANGE_EVENT_SYSTEM_TIME_BINDING_INCOMPLETE")
        );

        let invalid_report_with_disputed_time = evaluate_case(&cases[34], &r);
        assert_eq!(invalid_report_with_disputed_time["route_state"], "invalid_input");
        assert!(
            invalid_report_with_disputed_time["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "EVENT_EFFECTIVE_TIME_UNQUALIFIED")
        );
        assert_eq!(
            invalid_report_with_disputed_time["lane_results"][0]["state"],
            "invalid_input"
        );
    }

    #[test]
    fn unsupported_or_schema_incompatible_lanes_never_promote() {
        let r = registry();
        for index in [3usize, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19] {
            let result = evaluate_case(&corpus()["cases"][index], &r);
            assert_ne!(result["route_state"], "contract_bound_unverified");
        }
    }
}
