//! CityMaker's bitemporal query/report binding adapter.
//!
//! The adapter composes owner-native valid-time and system-time assertions with
//! four explicit query cutoffs. It does not authenticate timestamp receipts,
//! resolve legal retroactivity, establish source completeness, or authorize work.

use serde_json::{Value, json};
use std::collections::BTreeSet;

const QUERY_SCHEMA: &str = "luminous.citymaker-temporal-query.v0.1";
const CLAIM_CEILING: &str = "temporal query and declared owner-report binding semantics only; no timestamp authentication, signer authority, legal retroactivity, source completeness, engineering correctness or action authority";

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Accepts a fixed-width UTC RFC 3339 whole-second timestamp only. Fixed-width
/// canonical UTC strings can then be compared lexicographically without offset
/// or fractional-second normalization ambiguity.
pub fn valid_utc_timestamp(value: &str) -> bool {
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
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= max_day && hour <= 23 && minute <= 59 && second <= 59
}

fn add_missing(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn is_invalid_reason(reason: &str) -> bool {
    reason.contains("MISMATCH")
        || reason.contains("INVALID")
        || reason.contains("ESCALATION")
        || reason == "EVENT_TRUST_TIME_ORDER_INVALID"
        || reason == "CHANGE_EVENT_SYSTEM_TIME_BINDING_INVALID"
        || reason == "CHANGE_EVENT_TEMPORAL_BINDING_INVALID"
}

fn compare_query_binding(query: &Value, report: &Value) -> (String, BTreeSet<String>) {
    const FIELDS: [&str; 6] = [
        "time_basis",
        "world_time_as_of",
        "knowledge_time_as_of",
        "trust_time_as_of",
        "evaluated_at",
        "temporal_perspective",
    ];
    let mut reasons = BTreeSet::new();
    let binding = report.get("query_binding").unwrap_or(&Value::Null);
    let mut missing = false;
    let mut mismatch = false;
    for field in FIELDS {
        let expected = query.get(field);
        let actual = binding.get(field);
        match (expected, actual) {
            (Some(Value::String(left)), Some(Value::String(right))) if left == right => {}
            (Some(Value::String(_)), Some(Value::String(_))) => mismatch = true,
            _ => missing = true,
        }
    }
    if missing {
        reasons.insert("OWNER_REPORT_TEMPORAL_QUERY_BINDING_INCOMPLETE".to_owned());
    }
    if mismatch {
        reasons.insert("OWNER_REPORT_TEMPORAL_QUERY_BINDING_MISMATCH".to_owned());
    }
    let state = if mismatch {
        "mismatch"
    } else if missing {
        "incomplete"
    } else {
        "bound"
    };
    (state.to_owned(), reasons)
}

fn event_dimensions(
    query: &Value,
    event: &Value,
    reasons: &mut BTreeSet<String>,
) -> (String, String, String, bool) {
    let world_as_of = string(query, "world_time_as_of");
    let knowledge_as_of = string(query, "knowledge_time_as_of");
    let trust_as_of = string(query, "trust_time_as_of");
    let valid_time = event.get("valid_time").unwrap_or(&Value::Null);
    let effective_at = string(valid_time, "effective_at");
    let effective_until_value = valid_time.get("effective_until");
    let effective_until = effective_until_value.and_then(Value::as_str);
    let confidence = string(valid_time, "effective_time_confidence");

    let valid_time_complete = valid_time.get("effective_at").is_some()
        && effective_until_value.is_some()
        && valid_time.get("effective_time_confidence").is_some();
    let valid_time_valid = valid_utc_timestamp(effective_at)
        && (effective_until.is_none_or(valid_utc_timestamp))
        && matches!(
            confidence,
            "verified" | "authority_asserted" | "unknown" | "disputed"
        );
    let mut world_state = "unknown";
    if !valid_time_complete {
        add_missing(reasons, "CHANGE_EVENT_TEMPORAL_BINDING_INCOMPLETE");
    } else if !valid_time_valid {
        reasons.insert("CHANGE_EVENT_TEMPORAL_BINDING_INVALID".to_owned());
        world_state = "invalid";
    } else if effective_until.is_some_and(|until| until < effective_at) {
        reasons.insert("EVENT_VALID_TIME_WINDOW_INVALID".to_owned());
        world_state = "invalid_interval";
    } else if confidence != "verified" {
        reasons.insert("EVENT_EFFECTIVE_TIME_UNQUALIFIED".to_owned());
        world_state = "unqualified";
    } else if effective_at > world_as_of {
        reasons.insert("EVENT_NOT_YET_EFFECTIVE".to_owned());
        world_state = "not_yet_effective";
    } else if effective_until.is_some_and(|until| until <= world_as_of) {
        reasons.insert("EVENT_VALID_INTERVAL_EXPIRED".to_owned());
        world_state = "expired";
    } else if valid_utc_timestamp(world_as_of) {
        world_state = "valid_at_world_time";
    }

    let system_time = event.get("system_time").unwrap_or(&Value::Null);
    let recorded_at = string(system_time, "recorded_at");
    let trusted_at_value = system_time.get("trusted_at");
    let receipt_value = system_time.get("trusted_timestamp_receipt_sha256");
    let trusted_at = trusted_at_value.and_then(Value::as_str).unwrap_or("");
    let receipt = receipt_value.and_then(Value::as_str).unwrap_or("");
    let system_complete = system_time.get("recorded_at").is_some()
        && trusted_at_value.is_some()
        && receipt_value.is_some();
    let explicitly_untrusted = trusted_at_value.is_some_and(Value::is_null)
        && receipt_value.is_some_and(Value::is_null);
    let trust_pair_valid = valid_utc_timestamp(trusted_at) && lower_hex(receipt, 64);
    let system_valid = valid_utc_timestamp(recorded_at)
        && (explicitly_untrusted || trust_pair_valid)
        && (trusted_at_value.is_some_and(Value::is_null)
            == receipt_value.is_some_and(Value::is_null));

    let mut knowledge_state = "unknown";
    let mut trust_state = "unknown";
    let mut late_arrival = false;
    if !system_complete {
        add_missing(reasons, "CHANGE_EVENT_SYSTEM_TIME_BINDING_INCOMPLETE");
    } else if !system_valid {
        reasons.insert("CHANGE_EVENT_SYSTEM_TIME_BINDING_INVALID".to_owned());
        knowledge_state = "invalid";
        trust_state = "invalid";
    } else {
        late_arrival = valid_utc_timestamp(effective_at) && recorded_at > effective_at;
        if recorded_at > knowledge_as_of {
            reasons.insert("EVENT_RECORDED_AFTER_KNOWLEDGE_CUTOFF".to_owned());
            knowledge_state = "recorded_after_cutoff";
        } else {
            knowledge_state = "known_by_cutoff";
        }

        if explicitly_untrusted {
            reasons.insert("EVENT_SYSTEM_TIME_UNTRUSTED".to_owned());
            trust_state = "untrusted";
        } else if trusted_at < recorded_at {
            reasons.insert("EVENT_TRUST_TIME_ORDER_INVALID".to_owned());
            trust_state = "invalid";
        } else if trusted_at > trust_as_of {
            reasons.insert("EVENT_TRUSTED_AFTER_TRUST_CUTOFF".to_owned());
            trust_state = "trusted_after_cutoff";
        } else {
            trust_state = "trusted_by_cutoff";
        }
    }

    (world_state.to_owned(), knowledge_state.to_owned(), trust_state.to_owned(), late_arrival)
}

/// Evaluates temporal applicability and exact owner-report binding.
/// The function never authenticates owner timestamps or trust receipts.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = string(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let query = input.get("query").unwrap_or(&Value::Null);
    let event = input.get("event").unwrap_or(&Value::Null);
    let report = input.get("owner_report").unwrap_or(&Value::Null);
    let mut reasons = BTreeSet::<String>::new();

    if string(query, "schema_version") != QUERY_SCHEMA
        || string(query, "time_basis") != "UTC_RFC3339_SECONDS"
    {
        reasons.insert("TEMPORAL_QUERY_SCHEMA_INVALID".to_owned());
    }

    let query_fields = [
        "world_time_as_of",
        "knowledge_time_as_of",
        "trust_time_as_of",
        "evaluated_at",
        "temporal_perspective",
    ];
    let mut query_missing = false;
    for field in query_fields {
        if query.get(field).and_then(Value::as_str).is_none() {
            query_missing = true;
        }
    }
    if query_missing {
        reasons.insert("TEMPORAL_QUERY_BINDING_INCOMPLETE".to_owned());
    }

    let mut query_time_invalid = false;
    for field in [
        "world_time_as_of",
        "knowledge_time_as_of",
        "trust_time_as_of",
        "evaluated_at",
    ] {
        let value = string(query, field);
        if !value.is_empty() && !valid_utc_timestamp(value) {
            query_time_invalid = true;
        }
    }
    if query_time_invalid {
        reasons.insert("TEMPORAL_QUERY_TIME_INVALID".to_owned());
    }

    let perspective = string(query, "temporal_perspective");
    if !matches!(
        perspective,
        "historically_known_then" | "currently_reconstructed_history"
    ) {
        reasons.insert("TEMPORAL_QUERY_PERSPECTIVE_INVALID".to_owned());
    }

    if string(event, "time_basis") != "UTC_RFC3339_SECONDS"
        || string(event, "event_id").is_empty()
        || string(event, "subject_node_id").is_empty()
        || string(event, "subject_revision").is_empty()
    {
        reasons.insert("CHANGE_EVENT_BINDING_INCOMPLETE".to_owned());
    }
    if !lower_hex(string(event, "event_sha256"), 64) {
        reasons.insert("CHANGE_EVENT_IDENTITY_DIGEST_INVALID".to_owned());
    }

    let (world_state, knowledge_state, trust_state, late_arrival) =
        event_dimensions(query, event, &mut reasons);

    let (mut report_binding_state, report_query_reasons) =
        compare_query_binding(query, report);
    reasons.extend(report_query_reasons);

    let report_required = [
        "report_id",
        "event_id",
        "event_sha256",
        "subject_node_id",
        "subject_revision",
        "report_evaluated_at",
        "report_sha256",
        "report_reference",
    ];
    if report_required
        .iter()
        .any(|field| report.get(*field).and_then(Value::as_str).is_none())
    {
        reasons.insert("OWNER_REPORT_TEMPORAL_BINDING_INCOMPLETE".to_owned());
        if report_binding_state == "bound" {
            report_binding_state = "incomplete".to_owned();
        }
    }

    if string(report, "event_id") != string(event, "event_id")
        || string(report, "event_sha256") != string(event, "event_sha256")
    {
        reasons.insert("OWNER_REPORT_EVENT_BINDING_MISMATCH".to_owned());
        report_binding_state = "mismatch".to_owned();
    }
    if string(report, "subject_node_id") != string(event, "subject_node_id")
        || string(report, "subject_revision") != string(event, "subject_revision")
    {
        reasons.insert("OWNER_REPORT_SUBJECT_BINDING_MISMATCH".to_owned());
        report_binding_state = "mismatch".to_owned();
    }
    if report.get("valid_time") != event.get("valid_time")
        || report.get("system_time") != event.get("system_time")
    {
        reasons.insert("OWNER_REPORT_EVENT_TIME_BINDING_MISMATCH".to_owned());
        report_binding_state = "mismatch".to_owned();
    }
    if string(report, "report_evaluated_at") != string(query, "evaluated_at") {
        reasons.insert("OWNER_REPORT_EVALUATION_TIME_MISMATCH".to_owned());
        report_binding_state = "mismatch".to_owned();
    }

    let report_sha = string(report, "report_sha256");
    if !report_sha.is_empty()
        && (!lower_hex(report_sha, 64)
            || string(report, "report_reference") != format!("cas:sha256:{report_sha}"))
    {
        reasons.insert("OWNER_REPORT_DIGEST_REFERENCE_INVALID".to_owned());
        report_binding_state = "mismatch".to_owned();
    }
    if string(report, "action_authority") != "none" {
        reasons.insert("OWNER_REPORT_ACTION_AUTHORITY_ESCALATION".to_owned());
        report_binding_state = "mismatch".to_owned();
    }

    let invalid = reasons.iter().any(|reason| is_invalid_reason(reason));
    let insufficient = reasons.iter().any(|reason| {
        reason.contains("INCOMPLETE")
            || reason.contains("UNQUALIFIED")
            || reason.contains("UNTRUSTED")
    });
    let excluded = matches!(
        world_state.as_str(),
        "not_yet_effective" | "expired"
    ) || knowledge_state == "recorded_after_cutoff"
        || trust_state == "trusted_after_cutoff";

    let disposition = if invalid {
        "invalid_input"
    } else if insufficient {
        "insufficient_evidence"
    } else if excluded {
        "excluded_from_temporal_projection"
    } else {
        "eligible_for_declared_temporal_projection"
    };

    json!({
        "case_id": case_id,
        "disposition": disposition,
        "temporal_perspective": perspective,
        "world_time_state": world_state,
        "knowledge_time_state": knowledge_state,
        "trust_time_state": trust_state,
        "report_binding_state": report_binding_state,
        "late_arrival": late_arrival,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "owner_report_content_verified": false,
        "timestamp_authentication": "not_performed",
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

    fn load_json(path: &str) -> Value {
        let full = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
        let bytes = std::fs::read(full).expect("fixture file must exist");
        serde_json::from_slice(&bytes).expect("fixture JSON must parse")
    }

    fn corpus() -> Value {
        load_json("../../docs/research/city-temporal-replay/CITY_TEMPORAL_QUERY_CORPUS_V0_1.json")
    }

    #[test]
    fn all_temporal_projection_cases_match_independent_gold() {
        for case in corpus()["cases"].as_array().unwrap() {
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
    fn corpus_has_24_unique_cases() {
        let cases = corpus()["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        let ids = cases
            .iter()
            .map(|case| case["case_id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 24);
    }

    #[test]
    fn historical_and_current_reconstruction_are_distinct_temporal_projections() {
        let cases = corpus()["cases"].as_array().unwrap();
        let historical = evaluate_case(&cases[1]);
        let reconstructed = evaluate_case(&cases[2]);
        assert_eq!(historical["disposition"], "excluded_from_temporal_projection");
        assert_eq!(historical["knowledge_time_state"], "recorded_after_cutoff");
        assert_eq!(reconstructed["disposition"], "eligible_for_declared_temporal_projection");
        assert_eq!(reconstructed["late_arrival"], true);
        assert_eq!(reconstructed["temporal_perspective"], "currently_reconstructed_history");
    }

    #[test]
    fn owner_report_must_bind_all_four_times_and_perspective() {
        let cases = corpus()["cases"].as_array().unwrap();
        for index in [13usize, 14, 15, 16, 17] {
            assert_eq!(evaluate_case(&cases[index])["disposition"], "invalid_input");
            assert_eq!(evaluate_case(&cases[index])["report_binding_state"], "mismatch");
        }
    }
}
