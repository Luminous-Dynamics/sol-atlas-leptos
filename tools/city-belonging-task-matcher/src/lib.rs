use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

const REPORT_VERSION: &str = "city-belonging-task-report/0.1";
const POLICY_VERSION: &str = "city-belonging-constraint-policy/0.1";
const CLAIM_CEILING: &str = "synthetic task benchmark only; real-world search quality, service truth, safety, human connection and loneliness outcomes are not assessed";
const TASK_SCHEMA: &str = include_str!("../../../docs/research/fixtures/city-belonging-task-benchmark-v0.1.schema.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchState {
    Satisfied,
    Mismatch,
    Unknown,
}

impl MatchState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Mismatch => "mismatch",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
struct CaseDecision {
    eligible_candidate_ids: Vec<String>,
    preferred_candidate_ids: Vec<String>,
    candidate_receipts: Vec<Value>,
}

pub fn verify_files(fixture_path: &str, manifest_path: &str) -> Result<Value, String> {
    let fixture = std::fs::read(fixture_path)
        .map_err(|e| format!("cannot read fixture {}: {}", fixture_path, e))?;
    let manifest = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("cannot read manifest {}: {}", manifest_path, e))?;
    verify_bytes(&fixture, &manifest)
}

pub fn verify_bytes(fixture_bytes: &[u8], manifest_text: &str) -> Result<Value, String> {
    let manifest: Value = serde_json::from_str(manifest_text)
        .map_err(|e| format!("manifest is not valid JSON: {}", e))?;
    if required_str(&manifest, "hash_algorithm", "manifest")? != "SHA-256" {
        return Err("manifest.hash_algorithm must be SHA-256".to_owned());
    }

    let declared_schema_hash = required_str(&manifest, "case_schema_sha256", "manifest")?;
    let actual_schema_hash = sha256_hex(TASK_SCHEMA.as_bytes());
    if declared_schema_hash != actual_schema_hash {
        return Err(format!(
            "task schema SHA-256 mismatch: manifest={}, actual={}",
            declared_schema_hash, actual_schema_hash
        ));
    }

    let declared_fixture_hash = required_str(&manifest, "fixture_sha256", "manifest")?;
    let actual_fixture_hash = sha256_hex(fixture_bytes);
    if declared_fixture_hash != actual_fixture_hash {
        return Err(format!(
            "task fixture SHA-256 mismatch: manifest={}, actual={}",
            declared_fixture_hash, actual_fixture_hash
        ));
    }
    if !fixture_bytes.ends_with(b"\n") {
        return Err("task fixture must be newline-terminated".to_owned());
    }
    let schema: Value = serde_json::from_str(TASK_SCHEMA)
        .map_err(|e| format!("embedded task schema is malformed: {}", e))?;
    if required_str(&schema, "$id", "task schema")? != "city-belonging-task-benchmark-v0.1.schema.json" {
        return Err("unexpected embedded task schema id".to_owned());
    }
    if required_str(&manifest, "policy_version", "manifest")? != POLICY_VERSION {
        return Err("unsupported constraint policy version".to_owned());
    }

    let declared_count = manifest.get("record_count").and_then(Value::as_u64)
        .ok_or_else(|| "manifest.record_count must be an unsigned integer".to_owned())? as usize;
    let lines: Vec<&[u8]> = fixture_bytes.split(|byte| *byte == b'\n').collect();
    let mut seen_cases = BTreeSet::new();
    let mut receipts = Vec::new();
    let mut candidate_count = 0usize;
    let mut hard_constraint_checks = 0usize;
    let mut unknown_constraint_checks = 0usize;

    for (index, raw) in lines.iter().enumerate() {
        if raw.is_empty() {
            if index == lines.len() - 1 {
                continue;
            }
            return Err(format!("blank JSONL line {}", index + 1));
        }
        let case: Value = serde_json::from_slice(raw)
            .map_err(|e| format!("line {} is not valid JSON: {}", index + 1, e))?;
        let where_ = format!("line {}", index + 1);
        validate_case_shape(&case, &where_)?;
        let case_id = required_str(&case, "case_id", &where_)?;
        if !valid_case_id(case_id) {
            return Err(format!("invalid case_id {}", case_id));
        }
        if !seen_cases.insert(case_id.to_owned()) {
            return Err(format!("duplicate case_id {}", case_id));
        }

        let decision = evaluate_case(&case)?;
        let expected = case.get("expected")
            .ok_or_else(|| format!("{} missing expected contract", case_id))?;
        let expected_eligible = string_set(expected.get("eligible_candidate_ids"), &format!("{}.expected.eligible_candidate_ids", case_id))?;
        let expected_preferred = string_set(expected.get("preferred_candidate_ids"), &format!("{}.expected.preferred_candidate_ids", case_id))?;
        let actual_eligible: BTreeSet<String> = decision.eligible_candidate_ids.iter().cloned().collect();
        let actual_preferred: BTreeSet<String> = decision.preferred_candidate_ids.iter().cloned().collect();

        if expected_eligible != actual_eligible {
            return Err(format!(
                "{} eligible-set mismatch: expected={:?}, actual={:?}",
                case_id, expected_eligible, actual_eligible
            ));
        }
        if expected_preferred != actual_preferred {
            return Err(format!(
                "{} preferred-set mismatch: expected={:?}, actual={:?}",
                case_id, expected_preferred, actual_preferred
            ));
        }

        let all_candidate_ids = candidate_id_set(&case, case_id)?;
        let expected_reasons = expected.get("ineligible_reasons").and_then(Value::as_object)
            .ok_or_else(|| format!("{}.expected.ineligible_reasons must be an object", case_id))?;
        let reason_ids: BTreeSet<String> = expected_reasons.keys().cloned().collect();
        let rejected_ids: BTreeSet<String> = all_candidate_ids.difference(&actual_eligible).cloned().collect();
        if reason_ids != rejected_ids {
            return Err(format!(
                "{} exclusion-reason keys must cover every and only ineligible candidate: rejected={:?}, reasons={:?}",
                case_id, rejected_ids, reason_ids
            ));
        }
        for (candidate_id, reason) in expected_reasons {
            if reason.as_str().map(str::trim).unwrap_or_default().is_empty() {
                return Err(format!("{}.expected.ineligible_reasons.{} must be non-empty", case_id, candidate_id));
            }
        }

        for receipt in &decision.candidate_receipts {
            candidate_count += 1;
            if let Some(results) = receipt.get("hard_constraint_results").and_then(Value::as_array) {
                hard_constraint_checks += results.len();
                unknown_constraint_checks += results.iter()
                    .filter(|r| r.get("result").and_then(Value::as_str) == Some("unknown"))
                    .count();
            }
        }
        let no_fit = decision.eligible_candidate_ids.is_empty();
        let no_fit_contract = if no_fit {
            Some(required_str(expected, "no_fit_response", case_id)?.to_owned())
        } else {
            None
        };
        if no_fit && no_fit_contract.as_ref().map(|text| text.trim().is_empty()).unwrap_or(true) {
            return Err(format!("{} no-fit result needs an explicit expected response contract", case_id));
        }

        receipts.push(json!({
            "case_id": case_id,
            "family": required_str(&case, "family", case_id)?,
            "eligible_candidate_ids": decision.eligible_candidate_ids,
            "preferred_candidate_ids": decision.preferred_candidate_ids,
            "candidate_receipts": decision.candidate_receipts,
            "no_fit": no_fit,
            "no_fit_response_contract": no_fit_contract,
            "personalization_contract": expected.get("personalization"),
            "sensitive_intake_required": expected.get("sensitive_intake_required"),
            "search_history_retention": expected.get("search_history_retention"),
            "claim_ceiling": CLAIM_CEILING
        }));
    }

    if receipts.len() != declared_count {
        return Err(format!(
            "manifest record_count mismatch: declared={}, parsed={}",
            declared_count, receipts.len()
        ));
    }
    if receipts.is_empty() {
        return Err("task corpus contains no cases".to_owned());
    }

    Ok(json!({
        "report_version": REPORT_VERSION,
        "status": "SYNTHETIC_TASK_FIXTURE_MATCH",
        "claim_ceiling": CLAIM_CEILING,
        "benchmark_id": manifest.get("benchmark_id").and_then(Value::as_str).unwrap_or("unknown"),
        "policy_version": POLICY_VERSION,
        "fixture_sha256": actual_fixture_hash,
        "case_schema_sha256": actual_schema_hash,
        "record_count": receipts.len(),
        "candidate_count": candidate_count,
        "hard_constraint_checks": hard_constraint_checks,
        "unknown_hard_constraint_checks": unknown_constraint_checks,
        "hard_constraint_violations": 0,
        "unknowns_promoted_to_positive": 0,
        "soft_preferences_removed_eligible_candidates": 0,
        "all_cases_match_expected": true,
        "real_world_search_quality": "NOT_ASSESSED",
        "service_truth_and_safety": "NOT_ASSESSED",
        "human_task_success": "NOT_ASSESSED",
        "social_connection_outcomes": "NOT_ASSESSED",
        "case_receipts": receipts
    }))
}

fn evaluate_case(case: &Value) -> Result<CaseDecision, String> {
    let case_id = required_str(case, "case_id", "case")?;
    let candidates = case.get("candidates").and_then(Value::as_array)
        .ok_or_else(|| format!("{}.candidates must be an array", case_id))?;
    let constraints = case.get("hard_constraints").and_then(Value::as_array)
        .ok_or_else(|| format!("{}.hard_constraints must be an array", case_id))?;
    let preferences = case.get("soft_preferences").and_then(Value::as_array)
        .ok_or_else(|| format!("{}.soft_preferences must be an array", case_id))?;

    let mut seen_candidates = BTreeSet::new();
    let mut eligible = Vec::new();
    let mut preferred = Vec::new();
    let mut receipts = Vec::new();

    for candidate in candidates {
        let candidate_id = required_str(candidate, "candidate_id", case_id)?;
        if !seen_candidates.insert(candidate_id.to_owned()) {
            return Err(format!("{} duplicate candidate_id {}", case_id, candidate_id));
        }
        let facts = candidate.get("facts").and_then(Value::as_object)
            .ok_or_else(|| format!("{}.{} facts must be an object", case_id, candidate_id))?;

        let mut constraint_results = Vec::new();
        let mut has_mismatch = false;
        let mut has_unknown = false;
        for constraint in constraints {
            let field = required_str(constraint, "field", case_id)?;
            let operator = required_str(constraint, "operator", case_id)?;
            let expected_value = constraint.get("value")
                .ok_or_else(|| format!("{}. constraint {} missing value", case_id, field))?;
            let result = evaluate_constraint(facts, field, operator, expected_value, constraint)?;
            has_mismatch |= result == MatchState::Mismatch;
            has_unknown |= result == MatchState::Unknown;
            constraint_results.push(json!({
                "field": field,
                "operator": operator,
                "expected_value": expected_value,
                "result": result.as_str()
            }));
        }

        // Fail closed: every hard constraint must be affirmatively satisfied.
        let is_eligible = !has_mismatch && !has_unknown;
        let is_preferred = is_eligible && !preferences.is_empty()
            && preferences.iter().all(|preference| preference_matches(facts, preference));

        let eligibility = if is_eligible {
            eligible.push(candidate_id.to_owned());
            if is_preferred { preferred.push(candidate_id.to_owned()); }
            "eligible"
        } else if has_mismatch {
            "ineligible_constraint_mismatch"
        } else {
            "unknown_hard_constraint"
        };
        let candidate_reason_codes = reason_codes(&constraint_results);
        receipts.push(json!({
            "candidate_id": candidate_id,
            "eligibility": eligibility,
            "hard_constraint_results": constraint_results,
            "soft_preference_match": is_preferred,
            "reason_codes": candidate_reason_codes
        }));
    }

    eligible.sort();
    preferred.sort();
    receipts.sort_by(|a, b| {
        a.get("candidate_id").and_then(Value::as_str).unwrap_or("")
            .cmp(b.get("candidate_id").and_then(Value::as_str).unwrap_or(""))
    });
    Ok(CaseDecision {
        eligible_candidate_ids: eligible,
        preferred_candidate_ids: preferred,
        candidate_receipts: receipts,
    })
}

fn evaluate_constraint(
    facts: &Map<String, Value>,
    field: &str,
    operator: &str,
    expected: &Value,
    constraint: &Value,
) -> Result<MatchState, String> {
    if let Some(required_fact) = constraint.get("requires").and_then(Value::as_str) {
        match facts.get(required_fact) {
            Some(value) if value.as_str() == Some("verified") => {}
            _ => return Ok(MatchState::Unknown),
        }
    }

    let observed = facts.get(field);
    match operator {
        "equals" => {
            if is_unknown(observed) { return Ok(MatchState::Unknown); }
            Ok(if observed == Some(expected) { MatchState::Satisfied } else { MatchState::Mismatch })
        }
        "contains_verified" => {
            let values = match observed.and_then(Value::as_array) {
                Some(values) if !is_unknown(observed) => values,
                _ => return Ok(MatchState::Unknown),
            };
            let target = expected.as_str().ok_or_else(|| "contains_verified target must be a string".to_owned())?;
            let verified_target = format!("verified:{}", target);
            if values.iter().any(|value| value.as_str() == Some(verified_target.as_str())) {
                Ok(MatchState::Satisfied)
            } else if values.iter().any(|value| is_unknown(Some(value))) {
                Ok(MatchState::Unknown)
            } else {
                Ok(MatchState::Mismatch)
            }
        }
        "after_local_time" => {
            let value = match observed.and_then(Value::as_str) {
                Some(value) if !is_unknown(observed) => value,
                _ => return Ok(MatchState::Unknown),
            };
            let target = expected.as_str().ok_or_else(|| "after_local_time target must be HH:MM".to_owned())?;
            let target_minutes = parse_minutes(target).ok_or_else(|| format!("invalid HH:MM target {}", target))?;
            if value.starts_with("verified_before_") {
                return Ok(MatchState::Mismatch);
            }
            let encoded = value.strip_prefix("verified_after_")
                .ok_or_else(|| format!("unsupported verified schedule fact {}", value))?;
            let hour: u32 = encoded.parse().map_err(|_| format!("invalid verified schedule hour {}", encoded))?;
            if hour > 23 { return Err(format!("verified schedule hour out of range {}", hour)); }
            let observed_minutes = hour * 60;
            Ok(if observed_minutes >= target_minutes { MatchState::Satisfied } else { MatchState::Mismatch })
        }
        "less_than_or_equal" => {
            let limit = expected.as_f64().ok_or_else(|| "less_than_or_equal target must be numeric".to_owned())?;
            let observed = match observed.and_then(Value::as_f64) {
                Some(value) => value,
                None => return Ok(MatchState::Unknown),
            };
            Ok(if observed <= limit { MatchState::Satisfied } else { MatchState::Mismatch })
        }
        _ => Err(format!("unknown hard constraint operator {}", operator)),
    }
}

fn preference_matches(facts: &Map<String, Value>, preference: &Value) -> bool {
    let field = match preference.get("field").and_then(Value::as_str) {
        Some(field) => field,
        None => return false,
    };
    let target = match preference.get("value") {
        Some(value) => value,
        None => return false,
    };
    let observed = match facts.get(field) {
        Some(value) if !is_unknown(Some(value)) => value,
        _ => return false,
    };
    if observed == target { return true; }
    match (observed.as_str(), target.as_str()) {
        (Some(actual), Some(target)) => actual.starts_with(&format!("{}_", target))
            || actual == format!("verified:{}", target),
        _ => false,
    }
}

fn reason_codes(results: &[Value]) -> Vec<String> {
    let mut codes = BTreeSet::new();
    for result in results {
        let field = result.get("field").and_then(Value::as_str).unwrap_or("unknown-field");
        let state = result.get("result").and_then(Value::as_str).unwrap_or("unknown");
        if state != "satisfied" {
            codes.insert(format!("{}:{}", field, state));
        }
    }
    codes.into_iter().collect()
}

fn validate_case_shape(case: &Value, where_: &str) -> Result<(), String> {
    let object = case.as_object().ok_or_else(|| format!("{} must be an object", where_))?;
    let allowed: BTreeSet<&str> = [
        "case_id", "family", "user_request", "hard_constraints", "soft_preferences",
        "candidates", "expected", "forbidden_behaviors", "rationale"
    ].into_iter().collect();
    for key in object.keys() {
        if !allowed.contains(key.as_str()) {
            return Err(format!("{} has undeclared top-level field {}", where_, key));
        }
    }
    for key in [
        "case_id", "family", "user_request", "hard_constraints", "soft_preferences",
        "candidates", "expected", "forbidden_behaviors", "rationale"
    ] {
        if !object.contains_key(key) {
            return Err(format!("{} missing required field {}", where_, key));
        }
    }
    let _ = required_str(case, "family", where_)?;
    let _ = required_str(case, "user_request", where_)?;
    let _ = required_str(case, "rationale", where_)?;
    if !case.get("hard_constraints").and_then(Value::as_array).is_some() {
        return Err(format!("{}.hard_constraints must be an array", where_));
    }
    if !case.get("soft_preferences").and_then(Value::as_array).is_some() {
        return Err(format!("{}.soft_preferences must be an array", where_));
    }
    let candidates = case.get("candidates").and_then(Value::as_array)
        .ok_or_else(|| format!("{}.candidates must be an array", where_))?;
    if candidates.is_empty() {
        return Err(format!("{}.candidates must not be empty", where_));
    }
    for candidate in candidates {
        let id = required_str(candidate, "candidate_id", where_)?;
        if !candidate.get("facts").and_then(Value::as_object).is_some() {
            return Err(format!("{}.{} facts must be an object", where_, id));
        }
    }
    let expected = case.get("expected").and_then(Value::as_object)
        .ok_or_else(|| format!("{}.expected must be an object", where_))?;
    for key in ["eligible_candidate_ids", "preferred_candidate_ids", "ineligible_reasons", "must_preserve_unknowns"] {
        if !expected.contains_key(key) {
            return Err(format!("{}.expected missing {}", where_, key));
        }
    }
    if expected.get("must_preserve_unknowns").and_then(Value::as_bool) != Some(true) {
        return Err(format!("{}.expected.must_preserve_unknowns must be true", where_));
    }
    if expected.get("eligible_candidate_ids").and_then(Value::as_array).is_none()
        || expected.get("preferred_candidate_ids").and_then(Value::as_array).is_none()
        || expected.get("ineligible_reasons").and_then(Value::as_object).is_none() {
        return Err(format!("{}.expected eligible/preferred/reason fields have invalid shapes", where_));
    }
    if !case.get("forbidden_behaviors").and_then(Value::as_array).map(|list| !list.is_empty()).unwrap_or(false) {
        return Err(format!("{}.forbidden_behaviors must be a non-empty array", where_));
    }
    Ok(())
}

fn candidate_id_set(case: &Value, where_: &str) -> Result<BTreeSet<String>, String> {
    let candidates = case.get("candidates").and_then(Value::as_array)
        .ok_or_else(|| format!("{}.candidates must be an array", where_))?;
    let values = candidates.iter()
        .map(|candidate| required_str(candidate, "candidate_id", where_).map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    let set: BTreeSet<String> = values.iter().cloned().collect();
    if set.len() != values.len() {
        return Err(format!("{} duplicate candidate IDs", where_));
    }
    Ok(set)
}

fn is_unknown(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(text)) => {
            text.trim().is_empty()
                || matches!(text.as_str(), "unknown" | "not_available" | "not_checked" | "unverified")
        }
        Some(Value::Array(values)) => values.is_empty() || values.iter().all(|value| is_unknown(Some(value))),
        _ => false,
    }
}

fn string_set(value: Option<&Value>, where_: &str) -> Result<BTreeSet<String>, String> {
    let array = value.and_then(Value::as_array)
        .ok_or_else(|| format!("{} must be a string array", where_))?;
    let values = array.iter().map(|item| {
        item.as_str().map(str::to_owned)
            .ok_or_else(|| format!("{} entries must be strings", where_))
    }).collect::<Result<Vec<_>, _>>()?;
    let set: BTreeSet<String> = values.iter().cloned().collect();
    if set.len() != values.len() {
        return Err(format!("{} contains duplicate values", where_));
    }
    Ok(set)
}

fn required_str<'a>(value: &'a Value, key: &str, where_: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("{}.{} must be a non-empty string", where_, key))
}

fn valid_case_id(value: &str) -> bool {
    value.len() == 7 && value.starts_with("CBT-") && value[4..].bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_minutes(value: &str) -> Option<u32> {
    let bytes = value.as_bytes();
    if bytes.len() != 5 || bytes[2] != b':' || !bytes.iter().enumerate().all(|(i, b)| i == 2 || b.is_ascii_digit()) {
        return None;
    }
    let hour: u32 = value[..2].parse().ok()?;
    let minute: u32 = value[3..].parse().ok()?;
    if hour > 23 || minute > 59 { return None; }
    Some(hour * 60 + minute)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<_>>().join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../docs/research/fixtures/city-belonging-task-benchmark-v0.1.jsonl");
    const MANIFEST: &str = include_str!("../../../docs/research/fixtures/city-belonging-task-benchmark-v0.1.manifest.json");

    fn case(id: &str) -> Value {
        FIXTURE.lines().find_map(|line| {
            let value: Value = serde_json::from_str(line).ok()?;
            (value.get("case_id").and_then(Value::as_str) == Some(id)).then_some(value)
        }).expect("case ID exists in frozen fixture")
    }

    #[test]
    fn golden_corpus_matches_expected_eligible_and_preferred_sets() {
        let report = verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap();
        assert_eq!(report["status"], "SYNTHETIC_TASK_FIXTURE_MATCH");
        assert_eq!(report["record_count"], 14);
        assert_eq!(report["hard_constraint_violations"], 0);
        assert_eq!(report["unknowns_promoted_to_positive"], 0);
        assert_eq!(report["soft_preferences_removed_eligible_candidates"], 0);
        assert_eq!(report["all_cases_match_expected"], true);
    }

    #[test]
    fn changed_fixture_bytes_fail_before_evaluation() {
        let mut bytes = FIXTURE.as_bytes().to_vec();
        bytes[0] ^= 1;
        assert!(verify_bytes(&bytes, MANIFEST).unwrap_err().contains("task fixture SHA-256 mismatch"));
    }

    #[test]
    fn schema_digest_mismatch_fails_closed() {
        let mut manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        manifest["case_schema_sha256"] = json!("deadbeef");
        let manifest = serde_json::to_string(&manifest).unwrap();
        assert!(verify_bytes(FIXTURE.as_bytes(), &manifest).unwrap_err().contains("task schema SHA-256 mismatch"));
    }

    #[test]
    fn hard_unknown_cost_does_not_match_free_only_request() {
        let result = evaluate_case(&case("CBT-001")).unwrap();
        assert_eq!(result.eligible_candidate_ids, vec!["A"]);
        let b = result.candidate_receipts.iter().find(|r| r["candidate_id"] == "B").unwrap();
        assert_eq!(b["eligibility"], "unknown_hard_constraint");
    }

    #[test]
    fn unknown_accessibility_does_not_satisfy_mandatory_access_need() {
        assert_eq!(evaluate_case(&case("CBT-002")).unwrap().eligible_candidate_ids, vec!["A"]);
    }

    #[test]
    fn language_match_requires_explicit_verified_value() {
        assert_eq!(evaluate_case(&case("CBT-003")).unwrap().eligible_candidate_ids, vec!["A"]);
    }

    #[test]
    fn expired_event_is_not_eligible_for_current_events() {
        assert_eq!(evaluate_case(&case("CBT-004")).unwrap().eligible_candidate_ids, vec!["A"]);
    }

    #[test]
    fn soft_preference_does_not_remove_eligible_alternative() {
        let result = evaluate_case(&case("CBT-010")).unwrap();
        assert_eq!(result.eligible_candidate_ids, vec!["A", "B"]);
        assert_eq!(result.preferred_candidate_ids, vec!["A"]);
    }

    #[test]
    fn no_fit_is_an_explicit_result_not_a_synthetic_match() {
        assert!(evaluate_case(&case("CBT-009")).unwrap().eligible_candidate_ids.is_empty());
    }

    #[test]
    fn virtual_and_service_area_candidates_do_not_need_fake_pins() {
        let result = evaluate_case(&case("CBT-007")).unwrap();
        assert_eq!(result.eligible_candidate_ids, vec!["A", "B", "C"]);
        assert_eq!(result.preferred_candidate_ids, vec!["A"]);
    }

    #[test]
    fn explicit_worldview_filter_does_not_infer_user_identity() {
        assert_eq!(evaluate_case(&case("CBT-008")).unwrap().eligible_candidate_ids, vec!["A"]);
    }

    #[test]
    fn basic_discovery_does_not_require_sensitive_intake() {
        let result = verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap();
        let privacy = result["case_receipts"].as_array().unwrap().iter()
            .find(|entry| entry["case_id"] == "CBT-014").unwrap();
        assert_eq!(privacy["sensitive_intake_required"], false);
        assert_eq!(privacy["personalization_contract"], "off");
    }

    #[test]
    fn duplicate_candidate_ids_fail_closed() {
        let mut value = case("CBT-001");
        value["candidates"][1]["candidate_id"] = json!("A");
        assert!(evaluate_case(&value).unwrap_err().contains("duplicate candidate_id"));
    }

    #[test]
    fn unknown_operator_fails_closed() {
        let mut value = case("CBT-001");
        value["hard_constraints"][0]["operator"] = json!("opaque_score");
        assert!(evaluate_case(&value).unwrap_err().contains("unknown hard constraint operator"));
    }

    #[test]
    fn an_unverified_route_cannot_satisfy_a_travel_time_limit() {
        let mut value = case("CBT-013");
        value["candidates"][0]["facts"]["route_estimate"] = json!("unknown");
        value["candidates"][1]["facts"]["route_estimate"] = json!("not_available");
        let result = evaluate_case(&value).unwrap();
        assert!(result.eligible_candidate_ids.is_empty());
        let a = result.candidate_receipts.iter().find(|r| r["candidate_id"] == "A").unwrap();
        assert_eq!(a["eligibility"], "unknown_hard_constraint");
    }

    #[test]
    fn unexpected_task_fields_fail_closed() {
        let mut value = case("CBT-001");
        value["opaque_match_score"] = json!(0.99);
        assert!(validate_case_shape(&value, "test CBT-001").unwrap_err().contains("undeclared top-level field"));
    }

    #[test]
    fn exclusion_reasons_must_cover_every_rejected_candidate() {
        let mut value: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        value["expected"]["ineligible_reasons"].as_object_mut().unwrap().remove("C");
        let corpus = format!("{}\n", serde_json::to_string(&value).unwrap());
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let mut manifest = manifest;
        manifest["fixture_sha256"] = json!(sha256_hex(corpus.as_bytes()));
        manifest["record_count"] = json!(1);
        let manifest = serde_json::to_string(&manifest).unwrap();
        assert!(verify_bytes(corpus.as_bytes(), &manifest).unwrap_err().contains("exclusion-reason keys"));
    }


    #[test]
    fn candidate_order_is_a_metamorphic_invariant() {
        let original = case("CBT-010");
        let original_result = evaluate_case(&original).unwrap();
        let mut permuted = original;
        permuted["candidates"].as_array_mut().unwrap().reverse();
        let permuted_result = evaluate_case(&permuted).unwrap();
        assert_eq!(original_result.eligible_candidate_ids, permuted_result.eligible_candidate_ids);
        assert_eq!(original_result.preferred_candidate_ids, permuted_result.preferred_candidate_ids);
        assert_eq!(original_result.candidate_receipts, permuted_result.candidate_receipts);
    }

    #[test]
    fn hard_constraint_order_is_a_metamorphic_invariant() {
        let mut original = case("CBT-009");
        original["expected"]["no_fit_response"] = json!("synthetic expectation");
        let original_result = evaluate_case(&original).unwrap();
        original["hard_constraints"].as_array_mut().unwrap().reverse();
        let reversed_result = evaluate_case(&original).unwrap();
        assert_eq!(original_result.eligible_candidate_ids, reversed_result.eligible_candidate_ids);
        assert_eq!(original_result.preferred_candidate_ids, reversed_result.preferred_candidate_ids);
    }

    #[test]
    fn changing_soft_preferences_cannot_change_hard_eligibility() {
        let mut value = case("CBT-010");
        let before = evaluate_case(&value).unwrap();
        assert_eq!(before.eligible_candidate_ids, vec!["A", "B"]);
        value["soft_preferences"][0]["value"] = json!("large");
        let after = evaluate_case(&value).unwrap();
        assert_eq!(after.eligible_candidate_ids, before.eligible_candidate_ids);
        assert_eq!(after.preferred_candidate_ids, vec!["B"]);
    }

    #[test]
    fn adding_a_hard_constraint_cannot_add_eligible_candidates() {
        let mut value = case("CBT-010");
        let before = evaluate_case(&value).unwrap();
        value["hard_constraints"].as_array_mut().unwrap().push(json!({
            "field": "group_size",
            "operator": "equals",
            "value": "small_verified"
        }));
        let after = evaluate_case(&value).unwrap();
        assert!(after.eligible_candidate_ids.iter().all(|id| before.eligible_candidate_ids.contains(id)));
        assert_eq!(after.eligible_candidate_ids, vec!["A"]);
    }

    #[test]
    fn removing_a_hard_constraint_cannot_remove_eligible_candidates() {
        let mut value = case("CBT-002");
        let before = evaluate_case(&value).unwrap();
        assert_eq!(before.eligible_candidate_ids, vec!["A"]);
        value["hard_constraints"].as_array_mut().unwrap().pop();
        let after = evaluate_case(&value).unwrap();
        assert!(before.eligible_candidate_ids.iter().all(|id| after.eligible_candidate_ids.contains(id)));
        assert_eq!(after.eligible_candidate_ids, vec!["A", "B", "C"]);
    }

    #[test]
    fn irrelevant_candidate_facts_do_not_change_constraint_decisions() {
        let mut value = case("CBT-001");
        let before = evaluate_case(&value).unwrap();
        value["candidates"][1]["facts"]["popularity_score"] = json!(999999);
        value["candidates"][1]["facts"]["click_count"] = json!(1000000);
        let after = evaluate_case(&value).unwrap();
        assert_eq!(before.eligible_candidate_ids, after.eligible_candidate_ids);
        assert_eq!(before.preferred_candidate_ids, after.preferred_candidate_ids);
        assert_eq!(before.candidate_receipts, after.candidate_receipts);
    }

    #[test]
    fn verified_match_still_does_not_override_other_hard_constraints() {
        let value = case("CBT-009");
        let decision = evaluate_case(&value).unwrap();
        assert!(decision.eligible_candidate_ids.is_empty());
        let candidate_a = decision.candidate_receipts.iter().find(|r| r["candidate_id"] == "A").unwrap();
        assert_eq!(candidate_a["hard_constraint_results"][0]["result"], "satisfied");
        assert_eq!(candidate_a["hard_constraint_results"][2]["result"], "unknown");
        let candidate_b = decision.candidate_receipts.iter().find(|r| r["candidate_id"] == "B").unwrap();
        assert_eq!(candidate_b["hard_constraint_results"][0]["result"], "unknown");
        assert_eq!(candidate_b["hard_constraint_results"][2]["result"], "satisfied");
    }

    #[test]
    fn identical_inputs_produce_identical_report_bytes() {
        let first = serde_json::to_vec(&verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap()).unwrap();
        let second = serde_json::to_vec(&verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap()).unwrap();
        assert_eq!(first, second);
    }
}
