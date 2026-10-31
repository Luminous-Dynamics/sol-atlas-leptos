use industry_assurance_validator::validate_profile;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const EVALUATOR_VERSION: &str = "ieac-runtime-admission-v0.1.0";
pub const CHECKER_VERSION: &str = "ieac-runtime-independent-checker.v0.1.0";

fn sha256_value(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("JSON values serialize");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonical_utc_seconds(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T'
        || bytes[13] != b':' || bytes[16] != b':' || bytes[19] != b'Z'
        || !bytes.iter().enumerate().all(|(index, byte)| {
            [4, 7, 10, 13, 16, 19].contains(&index) || byte.is_ascii_digit()
        })
    {
        return None;
    }
    let part = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end]).ok()?.parse::<i64>().ok()
    };
    let (year, month, day, hour, minute, second) = (
        part(0, 4)?, part(5, 7)?, part(8, 10)?,
        part(11, 13)?, part(14, 16)?, part(17, 19)?,
    );
    if year < 1 || !(1..=12).contains(&month) || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute) || !(0..=59).contains(&second)
    {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days).contains(&day) {
        return None;
    }

    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;
    Some(days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn text_at<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
}

fn number_at(value: &Value, pointer: &str) -> Option<f64> {
    value.pointer(pointer).and_then(Value::as_f64).filter(|v| v.is_finite())
}

fn unique_push(set: &mut BTreeSet<String>, code: impl Into<String>) {
    set.insert(code.into());
}

fn artifact_binding<'a>(profile: &'a Value, id: &str) -> Option<&'a Value> {
    profile.get("artifact_bindings")?.as_array()?.iter().find(|item| {
        item.get("artifact_id").and_then(Value::as_str) == Some(id)
    })
}

fn check_artifact(
    profile: &Value,
    artifact_id: &str,
    expected_role: &str,
    expected_sha: &str,
    reasons: &mut BTreeSet<String>,
) -> bool {
    let Some(binding) = artifact_binding(profile, artifact_id) else {
        unique_push(reasons, "ARTIFACT_BINDING_MISSING");
        return false;
    };
    let mut valid = true;
    if binding.get("role").and_then(Value::as_str) != Some(expected_role) {
        unique_push(reasons, "ARTIFACT_ROLE_MISMATCH");
        valid = false;
    }
    if binding.get("content_sha256").and_then(Value::as_str) != Some(expected_sha) {
        unique_push(reasons, "ARTIFACT_DIGEST_MISMATCH");
        valid = false;
    }
    if binding.get("rights_state").and_then(Value::as_str) != Some("verified") {
        unique_push(reasons, "ARTIFACT_RIGHTS_NOT_VERIFIED");
        valid = false;
    }
    valid
}

fn make_condition_result(variable: &str, result: &str, reasons: &[String]) -> Value {
    json!({
        "variable": variable,
        "result": result,
        "reason_codes": reasons
    })
}

fn evaluate_conditions(
    profile: &Value,
    request: &Value,
    as_of_seconds: Option<i64>,
    reasons: &mut BTreeSet<String>,
    reject: &mut bool,
    abstain: &mut bool,
) -> Vec<Value> {
    let mut results = Vec::new();
    let declared = profile.pointer("/operating_envelope/conditions")
        .and_then(Value::as_array).cloned().unwrap_or_default();
    let supplied = request.pointer("/snapshot/conditions")
        .and_then(Value::as_array).cloned().unwrap_or_default();

    if declared.is_empty() {
        unique_push(reasons, "OPERATING_ENVELOPE_EMPTY");
        *abstain = true;
        return results;
    }

    let mut supplied_by_variable: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for measurement in supplied {
        if let Some(variable) = measurement.get("variable").and_then(Value::as_str) {
            supplied_by_variable.entry(variable.to_owned()).or_default().push(measurement);
        }
    }

    for condition in declared {
        let variable = condition.get("variable").and_then(Value::as_str).unwrap_or("unknown");
        let mut local = BTreeSet::new();
        let mut result = "in_domain";

        if !["measured", "validated_range"].contains(
            &condition.get("evidence_state").and_then(Value::as_str).unwrap_or("")
        ) {
            unique_push(&mut local, "CONDITION_EVIDENCE_NOT_ADMITTED");
            result = "indeterminate";
            *abstain = true;
        }

        let max_age = condition.get("max_age_seconds").and_then(Value::as_u64);
        if max_age.is_none() {
            unique_push(&mut local, "FRESHNESS_LIMIT_NOT_DECLARED");
            result = "indeterminate";
            *abstain = true;
        }

        let matches = supplied_by_variable.get(variable).cloned().unwrap_or_default();
        let measurement = if matches.len() == 1 {
            matches.first()
        } else {
            if matches.is_empty() {
                unique_push(&mut local, "CONDITION_MEASUREMENT_MISSING");
                result = "indeterminate";
                *abstain = true;
            } else {
                unique_push(&mut local, "DUPLICATE_CONDITION_MEASUREMENT");
                result = "out_of_domain";
                *reject = true;
            }
            None
        };

        if let Some(measurement) = measurement {
            if measurement.get("unit").and_then(Value::as_str)
                != condition.get("unit").and_then(Value::as_str)
            {
                unique_push(&mut local, "UNIT_MISMATCH");
                result = "out_of_domain";
                *reject = true;
            }

            let value = measurement.get("value").and_then(Value::as_f64).filter(|v| v.is_finite());
            let lower = measurement.get("lower").and_then(Value::as_f64).filter(|v| v.is_finite());
            let upper = measurement.get("upper").and_then(Value::as_f64).filter(|v| v.is_finite());
            match (value, lower, upper) {
                (Some(value), Some(lower), Some(upper)) if lower <= value && value <= upper => {
                    let minimum = condition.get("minimum").and_then(Value::as_f64);
                    let maximum = condition.get("maximum").and_then(Value::as_f64);
                    if minimum.is_some_and(|minimum| value < minimum)
                        || maximum.is_some_and(|maximum| value > maximum)
                    {
                        unique_push(&mut local, "VALUE_OUTSIDE_OPERATING_ENVELOPE");
                        result = "out_of_domain";
                        *reject = true;
                    } else if minimum.is_some_and(|minimum| lower < minimum)
                        || maximum.is_some_and(|maximum| upper > maximum)
                    {
                        unique_push(&mut local, "MEASUREMENT_INTERVAL_CROSSES_ENVELOPE");
                        result = "marginal";
                        *abstain = true;
                    }
                }
                _ => {
                    unique_push(&mut local, "INVALID_MEASUREMENT_INTERVAL");
                    result = "out_of_domain";
                    *reject = true;
                }
            }

            let observed = measurement.get("observed_at").and_then(Value::as_str)
                .and_then(canonical_utc_seconds);
            match (observed, as_of_seconds, max_age) {
                (Some(observed), Some(as_of), Some(max_age)) if observed > as_of => {
                    unique_push(&mut local, "FUTURE_DATED_MEASUREMENT");
                    result = "out_of_domain";
                    *reject = true;
                }
                (Some(observed), Some(as_of), Some(max_age)) if (as_of - observed) as u64 > max_age => {
                    unique_push(&mut local, "MEASUREMENT_STALE");
                    result = "indeterminate";
                    *abstain = true;
                }
                (Some(_), Some(_), Some(_)) => {}
                _ => {
                    unique_push(&mut local, "MEASUREMENT_TIME_INDETERMINATE");
                    result = "indeterminate";
                    *abstain = true;
                }
            }

            let evidence_id = measurement.get("evidence_artifact_id").and_then(Value::as_str);
            let source_ids = condition.get("source_artifact_ids").and_then(Value::as_array);
            let declared_source = evidence_id.is_some_and(|id| {
                source_ids.is_some_and(|values| values.iter().any(|item| item.as_str() == Some(id)))
            });
            if !declared_source {
                unique_push(&mut local, "MEASUREMENT_SOURCE_NOT_BOUND_TO_CONDITION");
                result = "out_of_domain";
                *reject = true;
            } else if let Some(evidence_id) = evidence_id {
                let evidence_digest = measurement.get("evidence_sha256").and_then(Value::as_str).unwrap_or("");
                if !check_artifact(profile, evidence_id, "measurement_source", evidence_digest, reasons) {
                    unique_push(&mut local, "MEASUREMENT_SOURCE_BINDING_INVALID");
                    result = "out_of_domain";
                    *reject = true;
                }
                if !is_sha256(evidence_digest) {
                    unique_push(&mut local, "MEASUREMENT_SOURCE_DIGEST_INVALID");
                    result = "out_of_domain";
                    *reject = true;
                }
            }
        }

        for code in &local {
            unique_push(reasons, code.clone());
        }
        results.push(make_condition_result(variable, result, &local.into_iter().collect::<Vec<_>>()));
    }

    results
}



fn validate_schema_subset(instance: &Value, schema: &Value, path: &str, out: &mut BTreeSet<String>) {
    if let Some(expected) = schema.get("type") {
        let matches_type = |kind: &str| match kind {
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "integer" => instance.as_i64().is_some() || instance.as_u64().is_some(),
            "number" => instance.as_f64().is_some_and(f64::is_finite),
            "boolean" => instance.is_boolean(),
            "null" => instance.is_null(),
            _ => false,
        };
        let matches = if let Some(kind) = expected.as_str() {
            matches_type(kind)
        } else {
            expected.as_array().is_some_and(|kinds| {
                kinds.iter().filter_map(Value::as_str).any(matches_type)
            })
        };
        if !matches {
            out.insert(format!("WIRE_TYPE_MISMATCH:{path}"));
            return;
        }
    }

    if let Some(constant) = schema.get("const") {
        if instance != constant {
            out.insert(format!("WIRE_CONST_MISMATCH:{path}"));
        }
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        if !values.contains(instance) {
            out.insert(format!("WIRE_ENUM_MISMATCH:{path}"));
        }
    }

    if let Some(text) = instance.as_str() {
        if let Some(minimum) = schema.get("minLength").and_then(Value::as_u64) {
            if text.chars().count() < minimum as usize {
                out.insert(format!("WIRE_STRING_TOO_SHORT:{path}"));
            }
        }
        if let Some(maximum) = schema.get("maxLength").and_then(Value::as_u64) {
            if text.chars().count() > maximum as usize {
                out.insert(format!("WIRE_STRING_TOO_LONG:{path}"));
            }
        }
        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
            let valid = if pattern.contains("[0-9a-fA-F]{64}") {
                is_sha256(text)
            } else if pattern.contains("T[0-9]{2}") {
                canonical_utc_seconds(text).is_some()
            } else {
                true
            };
            if !valid {
                out.insert(format!("WIRE_PATTERN_MISMATCH:{path}"));
            }
        }
        if schema.get("format").and_then(Value::as_str) == Some("date-time")
            && canonical_utc_seconds(text).is_none()
        {
            out.insert(format!("WIRE_DATE_TIME_INVALID:{path}"));
        }
    }

    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64) {
        if instance.as_f64().is_some_and(|value| value < minimum) {
            out.insert(format!("WIRE_NUMBER_BELOW_MINIMUM:{path}"));
        }
    }
    if let Some(minimum) = schema.get("minItems").and_then(Value::as_u64) {
        if instance.as_array().is_some_and(|items| items.len() < minimum as usize) {
            out.insert(format!("WIRE_ARRAY_TOO_SHORT:{path}"));
        }
    }

    if let Some(object) = instance.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    out.insert(format!("WIRE_REQUIRED_FIELD_MISSING:{path}/{key}"));
                }
            }
        }
        if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                for key in object.keys() {
                    if !properties.contains_key(key) {
                        out.insert(format!("WIRE_UNKNOWN_PROPERTY:{path}/{key}"));
                    }
                }
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, child_schema) in properties {
                if let Some(child) = object.get(key) {
                    validate_schema_subset(
                        child,
                        child_schema,
                        &format!("{path}/{key}"),
                        out,
                    );
                }
            }
        }
    }

    if let (Some(items_schema), Some(items)) = (schema.get("items"), instance.as_array()) {
        for (index, item) in items.iter().enumerate() {
            validate_schema_subset(
                item,
                items_schema,
                &format!("{path}/{index}"),
                out,
            );
        }
    }
}

fn runtime_request_schema() -> Value {
    serde_json::from_str(include_str!("../../../schemas/industry-runtime-admission-request-v0.1.schema.json"))
        .expect("embedded runtime request schema must be valid JSON")
}

fn runtime_report_schema() -> Value {
    serde_json::from_str(include_str!("../../../schemas/industry-runtime-admission-report-v0.1.schema.json"))
        .expect("embedded runtime report schema must be valid JSON")
}

pub fn validate_runtime_request(request: &Value) -> Vec<String> {
    let mut issues = BTreeSet::new();
    validate_schema_subset(request, &runtime_request_schema(), "$", &mut issues);
    issues.into_iter().collect()
}

pub fn validate_runtime_report(report: &Value) -> Vec<String> {
    let mut issues = BTreeSet::new();
    validate_schema_subset(report, &runtime_report_schema(), "$", &mut issues);
    issues.into_iter().collect()
}

pub fn evaluate_admission(profile: &Value, request: &Value) -> Value {
    let mut reasons = BTreeSet::new();
    let mut reject = false;
    let mut abstain = false;

    let request_issues = validate_runtime_request(request);
    if !request_issues.is_empty() {
        unique_push(&mut reasons, "REQUEST_CONTRACT_INVALID");
        for code in request_issues { unique_push(&mut reasons, code); }
        reject = true;
    }

    if !validate_profile(profile).is_empty() {
        unique_push(&mut reasons, "PROFILE_CONTRACT_INVALID");
        reject = true;
    }

    let profile_digest = sha256_value(profile);
    if text_at(request, "/profile_sha256") != Some(profile_digest.as_str()) {
        unique_push(&mut reasons, "PROFILE_DIGEST_MISMATCH");
        reject = true;
    }
    let snapshot_digest = request.get("snapshot").map(sha256_value).unwrap_or_default();
    if text_at(request, "/snapshot_sha256") != Some(snapshot_digest.as_str()) {
        unique_push(&mut reasons, "SNAPSHOT_DIGEST_MISMATCH");
        reject = true;
    }
    let revocation_digest = request.get("revocation_snapshot").map(sha256_value).unwrap_or_default();
    if text_at(request, "/revocation_snapshot_sha256") != Some(revocation_digest.as_str()) {
        unique_push(&mut reasons, "REVOCATION_SNAPSHOT_DIGEST_MISMATCH");
        reject = true;
    }

    for (profile_field, request_field, code) in [
        ("/profile_id", "/profile_id", "PROFILE_ID_MISMATCH"),
        ("/claim/capability_id", "/capability_id", "CAPABILITY_ID_MISMATCH"),
        ("/claim/instance_id", "/instance_id", "INSTANCE_ID_MISMATCH"),
        ("/claim/decision_id", "/decision_id", "DECISION_ID_MISMATCH"),
    ] {
        if text_at(profile, profile_field) != text_at(request, request_field) {
            unique_push(&mut reasons, code);
            reject = true;
        }
    }

    let tier = text_at(request, "/decision_tier").unwrap_or("");
    if !["D0_descriptive", "D1_advisory"].contains(&tier)
        || text_at(profile, "/decision_tier") != Some(tier)
    {
        unique_push(&mut reasons, "DECISION_TIER_NOT_ADMITTED");
        reject = true;
    }
    let disposition = text_at(profile, "/disposition").unwrap_or("");
    if disposition == "revoked" {
        unique_push(&mut reasons, "PROFILE_REVOKED");
        reject = true;
    } else if tier == "D1_advisory" && disposition != "qualified_for_profile" {
        unique_push(&mut reasons, "PROFILE_NOT_QUALIFIED_FOR_ADVISORY");
        reject = true;
    }

    let action = text_at(request, "/action").unwrap_or("");
    let profile_actions = profile.pointer("/authority/permitted_actions")
        .and_then(Value::as_array).cloned().unwrap_or_default();
    if !profile_actions.iter().any(|item| item.as_str() == Some(action)) {
        unique_push(&mut reasons, "ACTION_NOT_PERMITTED_BY_PROFILE");
        reject = true;
    }
    let allowed_by_tier = if tier == "D0_descriptive" {
        ["view", "annotate"].contains(&action)
    } else {
        ["view", "annotate", "recommend", "rank_options", "prepare_plan"].contains(&action)
    };
    if !allowed_by_tier {
        unique_push(&mut reasons, "ACTION_EXCEEDS_V0_1_AUTHORITY");
        reject = true;
    }
    if tier == "D1_advisory"
        && (profile.pointer("/authority/human_approval_required").and_then(Value::as_bool) != Some(true)
            || profile.pointer("/authority/actuation_enabled").and_then(Value::as_bool) != Some(false))
    {
        unique_push(&mut reasons, "ADVISORY_AUTHORITY_CONFIGURATION_INVALID");
        reject = true;
    }

    let as_of_text = text_at(request, "/as_of").unwrap_or("");
    let as_of_seconds = canonical_utc_seconds(as_of_text);
    let valid_from = text_at(profile, "/validity/valid_from").and_then(canonical_utc_seconds);
    let review_by = text_at(profile, "/validity/review_by").and_then(canonical_utc_seconds);
    match (as_of_seconds, valid_from, review_by) {
        (Some(now), Some(start), Some(end)) if now >= start && now < end => {}
        (Some(now), Some(start), Some(end)) if now < start || now >= end => {
            unique_push(&mut reasons, "PROFILE_OUTSIDE_VALIDITY_WINDOW");
            reject = true;
        }
        _ => {
            unique_push(&mut reasons, "PROFILE_VALIDITY_TIME_INDETERMINATE");
            abstain = true;
        }
    }

    if text_at(request, "/asset_id") != text_at(profile, "/claim/instance_id") {
        unique_push(&mut reasons, "ASSET_INSTANCE_MISMATCH");
        reject = true;
    }
    let model_id = text_at(request, "/model_id").unwrap_or("");
    let model_ids = profile.pointer("/claim/model_ids").and_then(Value::as_array);
    if !model_ids.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(model_id))) {
        unique_push(&mut reasons, "MODEL_ID_NOT_BOUND_TO_CLAIM");
        reject = true;
    }
    for (id_field, artifact_field, code) in [
        ("/evaluator_id", "/evaluator_artifact_id", "EVALUATOR_ID_MISMATCH"),
        ("/configuration_id", "/configuration_artifact_id", "CONFIGURATION_ID_MISMATCH"),
        ("/topology_id", "/topology_artifact_id", "TOPOLOGY_ID_MISMATCH"),
    ] {
        if text_at(request, id_field) != text_at(request, artifact_field) {
            unique_push(&mut reasons, code);
            reject = true;
        }
    }

    for (id_field, role, digest_field) in [
        ("/model_artifact_id", "model", "/model_sha256"),
        ("/evaluator_artifact_id", "evaluator", "/evaluator_sha256"),
        ("/configuration_artifact_id", "configuration", "/configuration_sha256"),
        ("/topology_artifact_id", "topology", "/topology_sha256"),
    ] {
        let id = text_at(request, id_field).unwrap_or("");
        let digest = text_at(request, digest_field).unwrap_or("");
        if !is_sha256(digest) || !check_artifact(profile, id, role, digest, &mut reasons) {
            unique_push(&mut reasons, "RUNTIME_IDENTITY_BINDING_INVALID");
            reject = true;
        }
    }

    let snapshot = request.get("snapshot").unwrap_or(&Value::Null);
    if text_at(snapshot, "/snapshot_id") != text_at(request, "/snapshot_id")
        || text_at(snapshot, "/as_of") != Some(as_of_text)
        || text_at(snapshot, "/asset_id") != text_at(request, "/asset_id")
        || text_at(snapshot, "/topology_sha256") != text_at(request, "/topology_sha256")
    {
        unique_push(&mut reasons, "SNAPSHOT_CONTEXT_MISMATCH");
        reject = true;
    }

    let revocation = request.get("revocation_snapshot").unwrap_or(&Value::Null);
    if text_at(revocation, "/as_of") != Some(as_of_text) {
        unique_push(&mut reasons, "REVOCATION_SNAPSHOT_TIME_MISMATCH");
        abstain = true;
    }
    match text_at(revocation, "/status") {
        Some("current") => {}
        Some("unknown" | "unavailable") | None => {
            unique_push(&mut reasons, "REVOCATION_STATE_UNAVAILABLE");
            abstain = true;
        }
        _ => {
            unique_push(&mut reasons, "REVOCATION_STATE_INVALID");
            reject = true;
        }
    }
    match text_at(revocation, "/authorization_status") {
        Some("authorized") => {}
        Some("denied") => {
            unique_push(&mut reasons, "AUTHORIZATION_DENIED");
            reject = true;
        }
        _ => {
            unique_push(&mut reasons, "AUTHORIZATION_STATE_UNKNOWN");
            abstain = true;
        }
    }

    let profile_id = text_at(profile, "/profile_id").unwrap_or("");
    let revoked_profiles = revocation.pointer("/revoked_profile_ids").and_then(Value::as_array);
    if revoked_profiles.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(profile_id))) {
        unique_push(&mut reasons, "PROFILE_REVOKED_BY_AUTHORITY");
        reject = true;
    }
    let revoked_actions = revocation.pointer("/allowed_actions").and_then(Value::as_array);
    if !revoked_actions.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(action))) {
        unique_push(&mut reasons, "ACTION_NOT_AUTHORIZED_BY_CURRENT_SOURCE");
        reject = true;
    }

    let bound_ids: Vec<&str> = [
        "/model_artifact_id", "/evaluator_artifact_id", "/configuration_artifact_id",
        "/topology_artifact_id",
    ].iter().filter_map(|p| text_at(request, p)).collect();
    let measurement_ids: Vec<&str> = request.pointer("/snapshot/conditions").and_then(Value::as_array)
        .into_iter().flatten().filter_map(|item| item.get("evidence_artifact_id").and_then(Value::as_str)).collect();
    let all_ids: BTreeSet<&str> = bound_ids.into_iter().chain(measurement_ids).collect();
    let revoked_ids = revocation.pointer("/revoked_artifact_ids").and_then(Value::as_array);
    if revoked_ids.is_some_and(|items| items.iter().any(|item| item.as_str().is_some_and(|id| all_ids.contains(id)))) {
        unique_push(&mut reasons, "ARTIFACT_REVOKED");
        reject = true;
    }
    let identity_digests: Vec<&str> = [
        "/model_sha256", "/evaluator_sha256", "/configuration_sha256", "/topology_sha256",
    ].iter().filter_map(|p| text_at(request, p)).collect();
    let revoked_sha = revocation.pointer("/revoked_sha256").and_then(Value::as_array);
    if revoked_sha.is_some_and(|items| items.iter().any(|item| item.as_str().is_some_and(|sha| identity_digests.contains(&sha)))) {
        unique_push(&mut reasons, "IDENTITY_DIGEST_REVOKED");
        reject = true;
    }

    let condition_results = evaluate_conditions(
        profile, request, as_of_seconds, &mut reasons, &mut reject, &mut abstain,
    );

    let admission = if reject {
        "reject"
    } else if abstain {
        "abstain"
    } else if tier == "D0_descriptive" {
        "admit_descriptive"
    } else {
        "admit_advisory"
    };
    let highest_action = if admission.starts_with("admit_") { json!(action) } else { Value::Null };
    json!({
        "schema_version": "luminous.ieac-runtime-admission-report.v0.1",
        "request_id": text_at(request, "/request_id").unwrap_or("unknown"),
        "profile_id": profile_id,
        "asset_id": text_at(request, "/asset_id").unwrap_or("unknown"),
        "snapshot_id": text_at(request, "/snapshot_id").unwrap_or("unknown"),
        "as_of": as_of_text,
        "action": action,
        "admission": admission,
        "highest_permitted_action": highest_action,
        "condition_results": condition_results,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "profile_sha256": profile_digest,
        "request_sha256": sha256_value(request),
        "snapshot_sha256": snapshot_digest,
        "revocation_snapshot_sha256": revocation_digest,
        "model_sha256": text_at(request, "/model_sha256").unwrap_or(""),
        "evaluator_sha256": text_at(request, "/evaluator_sha256").unwrap_or(""),
        "configuration_sha256": text_at(request, "/configuration_sha256").unwrap_or(""),
        "topology_sha256": text_at(request, "/topology_sha256").unwrap_or(""),
        "evaluator_version": EVALUATOR_VERSION
    })
}

fn independent_expected(profile: &Value, request: &Value) -> Value {
    let mut reasons = BTreeSet::new();
    let mut fatal = false;
    let mut uncertain = false;

    let request_issues = validate_runtime_request(request);
    if !request_issues.is_empty() {
        reasons.insert("REQUEST_CONTRACT_INVALID".to_owned());
        for code in request_issues { reasons.insert(code); }
        fatal = true;
    }

    if !validate_profile(profile).is_empty() {
        reasons.insert("PROFILE_CONTRACT_INVALID".to_owned());
        fatal = true;
    }
    let p_hash = sha256_value(profile);
    let s_hash = request.get("snapshot").map(sha256_value).unwrap_or_default();
    let r_hash = request.get("revocation_snapshot").map(sha256_value).unwrap_or_default();
    if request["profile_sha256"].as_str() != Some(p_hash.as_str()) {
        reasons.insert("PROFILE_DIGEST_MISMATCH".to_owned());
        fatal = true;
    }
    if request["snapshot_sha256"].as_str() != Some(s_hash.as_str()) {
        reasons.insert("SNAPSHOT_DIGEST_MISMATCH".to_owned());
        fatal = true;
    }
    if request["revocation_snapshot_sha256"].as_str() != Some(r_hash.as_str()) {
        reasons.insert("REVOCATION_SNAPSHOT_DIGEST_MISMATCH".to_owned());
        fatal = true;
    }
    let equalities = [
        (profile["profile_id"].as_str(), request["profile_id"].as_str(), "PROFILE_ID_MISMATCH"),
        (profile["claim"]["capability_id"].as_str(), request["capability_id"].as_str(), "CAPABILITY_ID_MISMATCH"),
        (profile["claim"]["instance_id"].as_str(), request["instance_id"].as_str(), "INSTANCE_ID_MISMATCH"),
        (profile["claim"]["decision_id"].as_str(), request["decision_id"].as_str(), "DECISION_ID_MISMATCH"),
        (profile["claim"]["instance_id"].as_str(), request["asset_id"].as_str(), "ASSET_INSTANCE_MISMATCH"),
    ];
    for (left, right, reason) in equalities {
        if left != right {
            reasons.insert(reason.to_owned());
            fatal = true;
        }
    }

    let tier = request["decision_tier"].as_str().unwrap_or("");
    let disposition = profile["disposition"].as_str().unwrap_or("");
    if profile["decision_tier"].as_str() != Some(tier)
        || !matches!(tier, "D0_descriptive" | "D1_advisory")
    {
        reasons.insert("DECISION_TIER_NOT_ADMITTED".to_owned());
        fatal = true;
    }
    if disposition == "revoked" {
        reasons.insert("PROFILE_REVOKED".to_owned());
        fatal = true;
    }
    if tier == "D1_advisory" && disposition != "qualified_for_profile" {
        reasons.insert("PROFILE_NOT_QUALIFIED_FOR_ADVISORY".to_owned());
        fatal = true;
    }

    let action = request["action"].as_str().unwrap_or("");
    let allowed: &[&str] = if tier == "D0_descriptive" {
        &["view", "annotate"]
    } else {
        &["view", "annotate", "recommend", "rank_options", "prepare_plan"]
    };
    if !allowed.contains(&action)
        || !profile["authority"]["permitted_actions"].as_array().is_some_and(|items| {
            items.iter().any(|item| item.as_str() == Some(action))
        })
    {
        reasons.insert("ACTION_NOT_PERMITTED_BY_PROFILE".to_owned());
        fatal = true;
    }
    if tier == "D1_advisory"
        && (profile["authority"]["actuation_enabled"] != false
            || profile["authority"]["human_approval_required"] != true)
    {
        reasons.insert("ADVISORY_AUTHORITY_CONFIGURATION_INVALID".to_owned());
        fatal = true;
    }

    let instant = request["as_of"].as_str().and_then(canonical_utc_seconds);
    let starts = profile["validity"]["valid_from"].as_str().and_then(canonical_utc_seconds);
    let expires = profile["validity"]["review_by"].as_str().and_then(canonical_utc_seconds);
    if let (Some(now), Some(start), Some(end)) = (instant, starts, expires) {
        if now < start || now >= end {
            reasons.insert("PROFILE_OUTSIDE_VALIDITY_WINDOW".to_owned());
            fatal = true;
        }
    } else {
        reasons.insert("PROFILE_VALIDITY_TIME_INDETERMINATE".to_owned());
        uncertain = true;
    }

    let models = profile["claim"]["model_ids"].as_array();
    let model_id = request["model_id"].as_str().unwrap_or("");
    if !models.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(model_id))) {
        reasons.insert("MODEL_ID_NOT_BOUND_TO_CLAIM".to_owned());
        fatal = true;
    }
    for (id_key, artifact_key, code) in [
        ("evaluator_id", "evaluator_artifact_id", "EVALUATOR_ID_MISMATCH"),
        ("configuration_id", "configuration_artifact_id", "CONFIGURATION_ID_MISMATCH"),
        ("topology_id", "topology_artifact_id", "TOPOLOGY_ID_MISMATCH"),
    ] {
        if request[id_key].as_str() != request[artifact_key].as_str() {
            reasons.insert(code.to_owned());
            fatal = true;
        }
    }

    let artifact_expectations = [
        ("model_artifact_id", "model_sha256", "model"),
        ("evaluator_artifact_id", "evaluator_sha256", "evaluator"),
        ("configuration_artifact_id", "configuration_sha256", "configuration"),
        ("topology_artifact_id", "topology_sha256", "topology"),
    ];
    for (id_key, digest_key, role) in artifact_expectations {
        let id = request[id_key].as_str().unwrap_or("");
        let expected_digest = request[digest_key].as_str().unwrap_or("");
        let found = profile["artifact_bindings"].as_array().and_then(|items| items.iter().find(|item| {
            item["artifact_id"].as_str() == Some(id)
        }));
        if let Some(item) = found {
            if item["role"].as_str() != Some(role) {
                reasons.insert("ARTIFACT_ROLE_MISMATCH".to_owned());
            }
            if item["content_sha256"].as_str() != Some(expected_digest) {
                reasons.insert("ARTIFACT_DIGEST_MISMATCH".to_owned());
            }
            if item["rights_state"].as_str() != Some("verified") {
                reasons.insert("ARTIFACT_RIGHTS_NOT_VERIFIED".to_owned());
            }
        } else {
            reasons.insert("ARTIFACT_BINDING_MISSING".to_owned());
        }
        let good = found.is_some_and(|item| {
            item["role"].as_str() == Some(role)
                && item["content_sha256"].as_str() == Some(expected_digest)
                && item["rights_state"].as_str() == Some("verified")
        });
        if !is_sha256(expected_digest) || !good {
            reasons.insert("RUNTIME_IDENTITY_BINDING_INVALID".to_owned());
            fatal = true;
        }
    }

    let snap = &request["snapshot"];
    if snap["snapshot_id"].as_str() != request["snapshot_id"].as_str()
        || snap["as_of"].as_str() != request["as_of"].as_str()
        || snap["asset_id"].as_str() != request["asset_id"].as_str()
        || snap["topology_sha256"].as_str() != request["topology_sha256"].as_str()
    {
        reasons.insert("SNAPSHOT_CONTEXT_MISMATCH".to_owned());
        fatal = true;
    }

    let revocation = &request["revocation_snapshot"];
    if revocation["as_of"].as_str() != request["as_of"].as_str() {
        reasons.insert("REVOCATION_SNAPSHOT_TIME_MISMATCH".to_owned());
        uncertain = true;
    }
    match revocation["status"].as_str() {
        Some("current") => {}
        Some("unknown") | Some("unavailable") | None => {
            reasons.insert("REVOCATION_STATE_UNAVAILABLE".to_owned());
            uncertain = true;
        }
        _ => {
            reasons.insert("REVOCATION_STATE_INVALID".to_owned());
            fatal = true;
        }
    }
    match revocation["authorization_status"].as_str() {
        Some("authorized") => {}
        Some("denied") => {
            reasons.insert("AUTHORIZATION_DENIED".to_owned());
            fatal = true;
        }
        _ => {
            reasons.insert("AUTHORIZATION_STATE_UNKNOWN".to_owned());
            uncertain = true;
        }
    }
    if revocation["allowed_actions"].as_array().is_none_or(|items| {
        !items.iter().any(|item| item.as_str() == Some(action))
    }) {
        reasons.insert("ACTION_NOT_AUTHORIZED_BY_CURRENT_SOURCE".to_owned());
        fatal = true;
    }
    if revocation["revoked_profile_ids"].as_array().is_some_and(|items| {
        items.iter().any(|item| item.as_str() == profile["profile_id"].as_str())
    }) {
        reasons.insert("PROFILE_REVOKED_BY_AUTHORITY".to_owned());
        fatal = true;
    }

    let mut all_artifact_ids = BTreeSet::new();
    for key in ["model_artifact_id", "evaluator_artifact_id", "configuration_artifact_id", "topology_artifact_id"] {
        if let Some(id) = request[key].as_str() { all_artifact_ids.insert(id); }
    }
    if let Some(measurements) = request["snapshot"]["conditions"].as_array() {
        for measurement in measurements {
            if let Some(id) = measurement["evidence_artifact_id"].as_str() {
                all_artifact_ids.insert(id);
            }
        }
    }
    if revocation["revoked_artifact_ids"].as_array().is_some_and(|items| {
        items.iter().any(|item| item.as_str().is_some_and(|id| all_artifact_ids.contains(id)))
    }) {
        reasons.insert("ARTIFACT_REVOKED".to_owned());
        fatal = true;
    }
    let identity_sha = ["model_sha256", "evaluator_sha256", "configuration_sha256", "topology_sha256"];
    if revocation["revoked_sha256"].as_array().is_some_and(|items| {
        items.iter().any(|item| identity_sha.iter().any(|key| item.as_str() == request[*key].as_str()))
    }) {
        reasons.insert("IDENTITY_DIGEST_REVOKED".to_owned());
        fatal = true;
    }

    let mut condition_results = Vec::new();
    let declared = profile["operating_envelope"]["conditions"].as_array().cloned().unwrap_or_default();
    let supplied = request["snapshot"]["conditions"].as_array().cloned().unwrap_or_default();
    if declared.is_empty() {
        reasons.insert("OPERATING_ENVELOPE_EMPTY".to_owned());
        uncertain = true;
    }
    for condition in declared {
        let variable = condition["variable"].as_str().unwrap_or("unknown");
        let found: Vec<&Value> = supplied.iter().filter(|m| m["variable"].as_str() == Some(variable)).collect();
        let mut local = BTreeSet::new();
        let mut condition_state = "in_domain";
        let Some(measurement) = (if found.len() == 1 { found.first().copied() } else { None }) else {
            if found.len() > 1 {
                local.insert("DUPLICATE_CONDITION_MEASUREMENT".to_owned());
                condition_state = "out_of_domain";
                fatal = true;
            } else {
                local.insert("CONDITION_MEASUREMENT_MISSING".to_owned());
                condition_state = "indeterminate";
                uncertain = true;
            }
            for code in &local { reasons.insert(code.clone()); }
            condition_results.push(make_condition_result(variable, condition_state, &local.into_iter().collect::<Vec<_>>()));
            continue;
        };

        if !matches!(condition["evidence_state"].as_str(), Some("measured" | "validated_range")) {
            local.insert("CONDITION_EVIDENCE_NOT_ADMITTED".to_owned());
            condition_state = "indeterminate";
            uncertain = true;
        }
        let max_age = condition["max_age_seconds"].as_u64();
        if max_age.is_none() {
            local.insert("FRESHNESS_LIMIT_NOT_DECLARED".to_owned());
            condition_state = "indeterminate";
            uncertain = true;
        }
        if measurement["unit"].as_str() != condition["unit"].as_str() {
            local.insert("UNIT_MISMATCH".to_owned());
            condition_state = "out_of_domain";
            fatal = true;
        }

        let v = measurement["value"].as_f64().filter(|x| x.is_finite());
        let lo = measurement["lower"].as_f64().filter(|x| x.is_finite());
        let hi = measurement["upper"].as_f64().filter(|x| x.is_finite());
        if let (Some(v), Some(lo), Some(hi)) = (v, lo, hi) {
            if lo > v || v > hi {
                local.insert("INVALID_MEASUREMENT_INTERVAL".to_owned());
                condition_state = "out_of_domain";
                fatal = true;
            } else {
                let min = condition["minimum"].as_f64();
                let max = condition["maximum"].as_f64();
                if min.is_some_and(|bound| v < bound) || max.is_some_and(|bound| v > bound) {
                    local.insert("VALUE_OUTSIDE_OPERATING_ENVELOPE".to_owned());
                    condition_state = "out_of_domain";
                    fatal = true;
                } else if min.is_some_and(|bound| lo < bound) || max.is_some_and(|bound| hi > bound) {
                    local.insert("MEASUREMENT_INTERVAL_CROSSES_ENVELOPE".to_owned());
                    condition_state = "marginal";
                    uncertain = true;
                }
            }
        } else {
            local.insert("INVALID_MEASUREMENT_INTERVAL".to_owned());
            condition_state = "out_of_domain";
            fatal = true;
        }

        let observed = measurement["observed_at"].as_str().and_then(canonical_utc_seconds);
        if let (Some(observed), Some(now), Some(limit)) = (observed, instant, max_age) {
            if observed > now {
                local.insert("FUTURE_DATED_MEASUREMENT".to_owned());
                condition_state = "out_of_domain";
                fatal = true;
            } else if (now - observed) as u64 > limit {
                local.insert("MEASUREMENT_STALE".to_owned());
                condition_state = "indeterminate";
                uncertain = true;
            }
        } else {
            local.insert("MEASUREMENT_TIME_INDETERMINATE".to_owned());
            condition_state = "indeterminate";
            uncertain = true;
        }

        let source_id = measurement["evidence_artifact_id"].as_str().unwrap_or("");
        let permitted_sources = condition["source_artifact_ids"].as_array();
        let source_declared = permitted_sources.is_some_and(|items| {
            items.iter().any(|item| item.as_str() == Some(source_id))
        });
        let source = profile["artifact_bindings"].as_array().and_then(|items| items.iter().find(|item| {
            item["artifact_id"].as_str() == Some(source_id)
        }));
        let source_hash = measurement["evidence_sha256"].as_str().unwrap_or("");
        if let Some(item) = source {
            if item["role"].as_str() != Some("measurement_source") {
                reasons.insert("ARTIFACT_ROLE_MISMATCH".to_owned());
            }
            if item["content_sha256"].as_str() != Some(source_hash) {
                reasons.insert("ARTIFACT_DIGEST_MISMATCH".to_owned());
            }
            if item["rights_state"].as_str() != Some("verified") {
                reasons.insert("ARTIFACT_RIGHTS_NOT_VERIFIED".to_owned());
            }
        } else {
            reasons.insert("ARTIFACT_BINDING_MISSING".to_owned());
        }
        if !source_declared || !source.is_some_and(|item| {
            item["role"].as_str() == Some("measurement_source")
                && item["content_sha256"].as_str() == Some(source_hash)
                && item["rights_state"].as_str() == Some("verified")
        }) || !is_sha256(source_hash) {
            local.insert("MEASUREMENT_SOURCE_BINDING_INVALID".to_owned());
            condition_state = "out_of_domain";
            fatal = true;
        }
        for code in &local { reasons.insert(code.clone()); }
        condition_results.push(make_condition_result(variable, condition_state, &local.into_iter().collect::<Vec<_>>()));
    }

    let admission = if fatal { "reject" } else if uncertain { "abstain" }
        else if tier == "D0_descriptive" { "admit_descriptive" } else { "admit_advisory" };
    json!({
        "admission": admission,
        "highest_permitted_action": if admission.starts_with("admit_") { json!(action) } else { Value::Null },
        "condition_results": condition_results,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "profile_sha256": p_hash,
        "snapshot_sha256": s_hash,
        "revocation_snapshot_sha256": r_hash,
        "request_sha256": sha256_value(request),
        "asset_id": request["asset_id"],
        "snapshot_id": request["snapshot_id"],
        "as_of": request["as_of"],
        "action": request["action"],
        "model_sha256": request["model_sha256"],
        "evaluator_sha256": request["evaluator_sha256"],
        "configuration_sha256": request["configuration_sha256"],
        "topology_sha256": request["topology_sha256"]
    })
}

pub fn independent_check(profile: &Value, request: &Value, report: &Value) -> Vec<String> {
    let expected = independent_expected(profile, request);
    let mut failures = Vec::new();
    for key in [
        "admission", "highest_permitted_action",
        "profile_sha256", "snapshot_sha256", "revocation_snapshot_sha256", "request_sha256",
        "asset_id", "snapshot_id", "as_of", "action", "model_sha256", "evaluator_sha256",
        "configuration_sha256", "topology_sha256",
    ] {
        if report.get(key) != expected.get(key) {
            failures.push(format!("REPORT_FIELD_MISMATCH:{key}"));
        }
    }
    let actual_conditions = report.get("condition_results").and_then(Value::as_array);
    let expected_conditions = expected.get("condition_results").and_then(Value::as_array);
    let condition_projection = |items: Option<&Vec<Value>>| {
        items.map(|values| values.iter().map(|item| {
            json!({"variable": item.get("variable"), "result": item.get("result")})
        }).collect::<Vec<_>>())
    };
    if condition_projection(actual_conditions) != condition_projection(expected_conditions) {
        failures.push("REPORT_FIELD_MISMATCH:condition_results".to_owned());
    }
    if report.get("report_sha256").is_some() && !report_digest_is_valid(report) {
        failures.push("REPORT_DIGEST_MISMATCH".to_owned());
    }
    if report.get("schema_version").and_then(Value::as_str)
        != Some("luminous.ieac-runtime-admission-report.v0.1")
    {
        failures.push("REPORT_SCHEMA_VERSION_MISMATCH".to_owned());
    }
    failures.sort();
    failures
}

pub fn seal_report(report: &mut Value, checker_status: &str, checker_errors: &[String]) {
    if let Some(object) = report.as_object_mut() {
        object.remove("report_sha256");
    }
    report["independent_checker"] = json!({
        "checker_version": CHECKER_VERSION,
        "status": checker_status,
        "errors": checker_errors
    });
    let digest = sha256_value(report);
    report["report_sha256"] = json!(digest);
}

pub fn report_digest_is_valid(report: &Value) -> bool {
    let Some(expected) = report.get("report_sha256").and_then(Value::as_str) else {
        return false;
    };
    let mut unsigned = report.clone();
    let Some(object) = unsigned.as_object_mut() else { return false; };
    object.remove("report_sha256");
    is_sha256(expected) && sha256_value(&unsigned) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_files() -> (Value, Value) {
        let profile: Value = serde_json::from_str(include_str!("../../../schemas/examples/industry-runtime-admission-profile-synthetic-v0.1.json")).unwrap();
        let request: Value = serde_json::from_str(include_str!("../../../schemas/examples/industry-runtime-admission-request-synthetic-v0.1.json")).unwrap();
        (profile, request)
    }

    #[test]
    fn synthetic_fixture_admits_only_descriptive_annotation() {
        let (profile, request) = fixture_files();
        let report = evaluate_admission(&profile, &request);
        assert_eq!(report["admission"], "admit_descriptive", "{report:#}");
        assert_eq!(report["highest_permitted_action"], "annotate");
        assert!(independent_check(&profile, &request, &report).is_empty());
    }

    #[test]
    fn independent_checker_detects_tampered_decision() {
        let (profile, request) = fixture_files();
        let mut report = evaluate_admission(&profile, &request);
        report["admission"] = json!("admit_advisory");
        assert!(!independent_check(&profile, &request, &report).is_empty());
    }

    #[test]
    fn stale_measurement_abstains() {
        let (profile, mut request) = fixture_files();
        request["snapshot"]["conditions"][0]["observed_at"] = json!("2026-10-30T00:00:00Z");
        request["snapshot_sha256"] = json!(sha256_value(&request["snapshot"]));
        request["revocation_snapshot"]["as_of"] = request["as_of"].clone();
        request["revocation_snapshot_sha256"] = json!(sha256_value(&request["revocation_snapshot"]));
        request["profile_sha256"] = json!(sha256_value(&profile));
        let report = evaluate_admission(&profile, &request);
        assert_eq!(report["admission"], "abstain", "{report:#}");
        assert!(report["reason_codes"].as_array().unwrap().iter().any(|v| v == "MEASUREMENT_STALE"));
    }

    #[test]
    fn revoked_profile_rejects() {
        let (profile, mut request) = fixture_files();
        request["revocation_snapshot"]["revoked_profile_ids"] = json!([profile["profile_id"]]);
        request["revocation_snapshot_sha256"] = json!(sha256_value(&request["revocation_snapshot"]));
        let report = evaluate_admission(&profile, &request);
        assert_eq!(report["admission"], "reject");
    }

    #[test]
    fn out_of_range_measurement_rejects() {
        let (profile, mut request) = fixture_files();
        request["snapshot"]["conditions"][0]["value"] = json!(25);
        request["snapshot"]["conditions"][0]["lower"] = json!(24);
        request["snapshot"]["conditions"][0]["upper"] = json!(26);
        request["snapshot_sha256"] = json!(sha256_value(&request["snapshot"]));
        let report = evaluate_admission(&profile, &request);
        assert_eq!(report["admission"], "reject");
    }

    #[test]
    fn D2_and_D3_requests_are_rejected_by_v0_1() {
        let (profile, mut request) = fixture_files();
        request["decision_tier"] = json!("D2_delegated_planning");
        let report = evaluate_admission(&profile, &request);
        assert_eq!(report["admission"], "reject");
        request["decision_tier"] = json!("D3_autonomous_control");
        let report = evaluate_admission(&profile, &request);
        assert_eq!(report["admission"], "reject");
    }

    #[test]
    fn sealed_report_digest_replays() {
        let (profile, request) = fixture_files();
        let mut report = evaluate_admission(&profile, &request);
        let errors = independent_check(&profile, &request, &report);
        assert!(errors.is_empty());
        seal_report(&mut report, "passed", &errors);
        assert!(report_digest_is_valid(&report));
    }
}
