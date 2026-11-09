//! Read-only composition of three independently owned habitat-service evidence lanes.
//!
//! This crate does not recompute restoration continuity, spatial access, service capacity,
//! metrology, or user outcomes. It checks that owner result envelopes remain bound to one
//! canonical scope, are temporally visible and authorized, and agree with non-compensatory
//! service gates. Reachable capacity is not realized use.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const CORPUS_ID: &str = "CITY-HABITAT-SERVICE-COMPOSITION-CORPUS-V0.1";
pub const CLAIM_CEILING: &str = "synthetic cross-owner service evidence composition only";
pub const EXPECTED_CASES: usize = 24;

const REQUIRED_ROLES: &[&str] = &["restoration", "access", "in_use"];

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn object<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn integer(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn add(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_commit(value: &str) -> bool {
    valid_hex(value, 40)
}

/// Hash the complete declared scope as serde_json's deterministic, lexicographically ordered
/// object representation. The hash is separate from the object and is never self-referential.
pub fn canonical_scope_sha256(scope: &Value) -> Result<String, String> {
    if !scope.is_object() {
        return Err("EVALUATION_SCOPE_OBJECT_REQUIRED".to_owned());
    }
    let bytes = serde_json::to_vec(scope)
        .map_err(|_| "EVALUATION_SCOPE_CANONICALIZATION_FAILED".to_owned())?;
    Ok(hex_digest(&bytes))
}

fn expected_contract(role: &str) -> Option<&'static str> {
    match role {
        "restoration" => Some("CITY-SERVICE-RESTORATION-001"),
        "access" => Some("CITY-MORPH-ACCESS-001"),
        "in_use" => Some("CITY-IN-USE-OUTCOMES-001"),
        _ => None,
    }
}

fn scope_identity_matches(record: &Value, scope: &Value) -> bool {
    let window = object(scope, "time_window");
    let record_window = object(record, "time_window");
    text(record, "service_id") == text(scope, "service_id")
        && text(record, "subject_id") == text(scope, "subject_id")
        && text(record, "subject_digest") == text(scope, "subject_digest")
        && text(record, "functional_unit") == text(scope, "functional_unit")
        && text(record, "population_frame_id") == text(scope, "population_frame_id")
        && text(record, "group_id") == text(scope, "group_id")
        && text(record, "scenario_id") == text(scope, "scenario_id")
        && text(record_window, "window_id") == text(window, "window_id")
        && integer(record_window, "start_tick") == integer(window, "start_tick")
        && integer(record_window, "end_tick") == integer(window, "end_tick")
        && text(record_window, "tick_unit") == text(window, "tick_unit")
}

fn time_visible(time: &Value, cut: &Value) -> bool {
    let valid_from = integer(time, "valid_from_tick");
    let valid_until = integer(time, "valid_until_tick");
    let recorded = integer(time, "recorded_tick");
    let registered = integer(time, "registered_tick");
    let ingested = integer(time, "ingested_tick");
    let valid_at = integer(cut, "valid_tick");
    let recorded_at = integer(cut, "recorded_tick");
    let registered_at = integer(cut, "registered_tick");
    let ingested_at = integer(cut, "ingested_tick");
    valid_from.zip(valid_until).zip(valid_at).is_some_and(|((from, until), at)| from <= at && at < until)
        && recorded.zip(recorded_at).is_some_and(|(source, at)| source <= at)
        && registered.zip(registered_at).is_some_and(|(source, at)| source <= at)
        && ingested.zip(ingested_at).is_some_and(|(source, at)| source <= at)
}

fn validate_scope_binding(
    binding: &Value,
    role: &str,
    scope: &Value,
    scope_sha256: &str,
    cut: &Value,
    reasons: &mut BTreeSet<String>,
) -> bool {
    let expected_digest = text(object(scope, "owner_scope_digests"), role);
    let mut valid = true;
    if text(binding, "owner") != "sol-atlas"
        || text(binding, "contract_id") != "CITY-SCOPE-BINDING-001"
        || text(binding, "contract_revision").is_empty()
        || text(binding, "receipt_id").is_empty()
        || !valid_commit(text(binding, "source_commit"))
        || !valid_hex(text(binding, "content_sha256"), 64)
    {
        add(reasons, "SCOPE_BINDING_RECEIPT_IDENTITY_OR_DIGEST_INVALID");
        valid = false;
    }
    if text(binding, "binding_state") != "Bound"
        || text(binding, "common_scope_sha256") != scope_sha256
        || text(binding, "source_scope_digest") != expected_digest
    {
        add(reasons, "OWNER_SCOPE_BINDING_MISMATCH");
        valid = false;
    }
    if text(binding, "evidence_state") != "current"
        || text(binding, "rights_state") != "permitted"
        || text(binding, "issuer_state") != "authorized"
    {
        add(reasons, "SCOPE_BINDING_AUTHORITY_OR_RIGHTS_INVALID");
        valid = false;
    }
    if !time_visible(object(binding, "time"), cut) {
        add(reasons, "SCOPE_BINDING_NOT_VISIBLE_AT_EVALUATION_CUT");
        valid = false;
    }
    valid
}

fn validate_owner_receipt(
    role: &str,
    owner_input: &Value,
    scope: &Value,
    scope_sha256: &str,
    cut: &Value,
    reasons: &mut BTreeSet<String>,
) -> bool {
    let Some(expected_contract) = expected_contract(role) else {
        add(reasons, "OWNER_ROLE_UNSUPPORTED");
        return false;
    };
    let receipt = object(owner_input, "receipt");
    let mut valid = true;
    if receipt.is_null() {
        add(reasons, "OWNER_RECEIPT_MISSING");
        return false;
    }
    if text(receipt, "owner") != "sol-atlas"
        || text(receipt, "contract_id") != expected_contract
        || text(receipt, "contract_revision").is_empty()
        || text(receipt, "receipt_id").is_empty()
        || text(receipt, "source_case_id").is_empty()
    {
        add(reasons, "OWNER_RECEIPT_IDENTITY_MISMATCH");
        valid = false;
    }
    if !valid_commit(text(receipt, "source_commit"))
        || !valid_hex(text(receipt, "content_sha256"), 64)
    {
        add(reasons, "OWNER_RECEIPT_DIGEST_INVALID");
        valid = false;
    }
    if !scope_identity_matches(receipt, scope)
        || text(receipt, "source_scope_digest") != text(object(scope, "owner_scope_digests"), role)
    {
        add(reasons, "OWNER_SCOPE_IDENTITY_MISMATCH");
        valid = false;
    }
    if text(receipt, "evidence_state") == "revoked" || text(receipt, "evidence_state") == "disputed" {
        add(reasons, "OWNER_EVIDENCE_REVOKED_OR_DISPUTED");
        valid = false;
    } else if text(receipt, "evidence_state") != "current" {
        add(reasons, "OWNER_EVIDENCE_NOT_CURRENT");
        valid = false;
    }
    if text(receipt, "rights_state") != "permitted" {
        add(reasons, "OWNER_RIGHTS_NOT_CLEARED");
        valid = false;
    }
    if text(receipt, "issuer_state") != "authorized" {
        add(reasons, "OWNER_ISSUER_NOT_AUTHORIZED");
        valid = false;
    }
    if !time_visible(object(receipt, "time"), cut) {
        add(reasons, "OWNER_RECEIPT_NOT_VISIBLE_AT_EVALUATION_CUT");
        valid = false;
    }
    if !validate_scope_binding(
        object(owner_input, "scope_binding"),
        role,
        scope,
        scope_sha256,
        cut,
        reasons,
    ) {
        valid = false;
    }
    valid
}

fn validate_service_observation(
    observation: &Value,
    scope: &Value,
    scope_sha256: &str,
    cut: &Value,
    reasons: &mut BTreeSet<String>,
) -> bool {
    let mut valid = true;
    let expected_digest = text(object(scope, "owner_scope_digests"), "service_operation");
    if observation.is_null() {
        add(reasons, "SERVICE_OPERATION_SOURCE_RECEIPT_MISSING");
        return false;
    }
    if text(observation, "owner") != "symthaea"
        || text(observation, "contract_id") != "CIV-SERVICE-001"
        || text(observation, "contract_revision") != "service-v1"
        || text(observation, "receipt_id").is_empty()
        || text(observation, "source_scope_digest") != expected_digest
        || text(observation, "service_id") != text(scope, "service_id")
        || text(observation, "subject_id") != text(scope, "subject_id")
        || text(observation, "subject_digest") != text(scope, "subject_digest")
        || text(observation, "functional_unit") != text(scope, "functional_unit")
        || text(observation, "population_frame_id") != text(scope, "population_frame_id")
        || text(observation, "group_id") != text(scope, "group_id")
        || text(observation, "scenario_id") != text(scope, "scenario_id")
    {
        add(reasons, "SERVICE_OPERATION_SOURCE_SCOPE_MISMATCH");
        valid = false;
    }
    if !valid_commit(text(observation, "source_commit"))
        || !valid_hex(text(observation, "content_sha256"), 64)
    {
        add(reasons, "SERVICE_OPERATION_SOURCE_DIGEST_INVALID");
        valid = false;
    }
    if text(observation, "evidence_state") == "revoked" || text(observation, "evidence_state") == "disputed" {
        add(reasons, "SERVICE_OPERATION_SOURCE_REVOKED_OR_DISPUTED");
        valid = false;
    } else if text(observation, "evidence_state") != "current" {
        add(reasons, "SERVICE_OPERATION_SOURCE_NOT_CURRENT");
        valid = false;
    }
    if text(observation, "rights_state") != "permitted" || text(observation, "issuer_state") != "authorized" {
        add(reasons, "SERVICE_OPERATION_SOURCE_AUTHORITY_OR_RIGHTS_INVALID");
        valid = false;
    }
    if !time_visible(object(observation, "time"), cut) {
        add(reasons, "SERVICE_OPERATION_SOURCE_NOT_VISIBLE_AT_EVALUATION_CUT");
        valid = false;
    }
    let binding = object(observation, "scope_binding");
    if text(binding, "binding_state") != "Bound"
        || text(binding, "common_scope_sha256") != scope_sha256
        || text(binding, "source_scope_digest") != expected_digest
        || text(binding, "contract_id") != "CITY-SCOPE-BINDING-001"
        || !valid_hex(text(binding, "content_sha256"), 64)
        || !time_visible(object(binding, "time"), cut)
    {
        add(reasons, "SERVICE_OPERATION_SCOPE_BINDING_INVALID");
        valid = false;
    }
    valid
}

fn evaluate_restoration(owner: &Value, receipt_valid: bool, scope: &Value, reasons: &mut BTreeSet<String>) -> &'static str {
    if !receipt_valid {
        return "Unknown";
    }
    let result = object(owner, "result");
    let stable = object(result, "stable_adequacy");
    let minimum = integer(scope, "minimum_stable_ticks").unwrap_or(i64::MAX);
    let stable_ok = stable.get("qualified").and_then(Value::as_bool) == Some(true)
        && integer(stable, "observed_ticks").is_some_and(|ticks| ticks >= minimum)
        && integer(stable, "minimum_ticks") == Some(minimum);
    if text(result, "bridge_disposition") == "RestoredForDeclaredScope"
        && result.get("restoration_qualified_for_scope").and_then(Value::as_bool) == Some(true)
        && text(result, "continuity_state") == "Adequate"
        && stable_ok
    {
        "Qualified"
    } else {
        add(reasons, "RESTORATION_NOT_QUALIFIED");
        "NotQualified"
    }
}

fn evaluate_access(
    owner: &Value,
    receipt_valid: bool,
    scope: &Value,
    reasons: &mut BTreeSet<String>,
) -> &'static str {
    if !receipt_valid {
        return "Unknown";
    }
    let result = object(owner, "result");
    let groups = array(result, "groups");
    let mut matching = groups.iter().filter(|group| text(group, "group_id") == text(scope, "group_id"));
    let Some(group) = matching.next() else {
        add(reasons, "ACCESS_GROUP_MISSING");
        return "Unknown";
    };
    if matching.next().is_some() {
        add(reasons, "ACCESS_GROUP_DUPLICATE");
        return "Invalid";
    }
    let demand = object(scope, "demand");
    let capacity = number(group, "reachable_capacity_potential");
    if number(group, "demand") != number(demand, "value")
        || text(group, "demand_unit") != text(demand, "unit")
    {
        add(reasons, "ACCESS_GROUP_SCOPE_MISMATCH");
        return "Invalid";
    }
    let service_ids = array(group, "accessible_service_ids");
    let includes_service = service_ids.iter().any(|id| id.as_str() == Some(text(scope, "service_id")));
    let unknown_services = array(group, "unknown_service_ids");
    let status = text(group, "status");
    match status {
        "AccessibleAndAdequatePotential" => {
            if !includes_service || !unknown_services.is_empty() {
                add(reasons, "ACCESS_RESULT_INTERNALLY_INCONSISTENT");
                return "Invalid";
            }
            if capacity.is_none_or(|value| value < number(demand, "value").unwrap_or(f64::INFINITY)) {
                add(reasons, "ACCESS_RESULT_INTERNALLY_INCONSISTENT");
                return "Invalid";
            }
            if text(result, "disposition") != "FeasibleForSyntheticCase" {
                add(reasons, "ACCESS_RESULT_DISPOSITION_INCONSISTENT");
                return "Invalid";
            }
            "AccessibleAndAdequatePotential"
        }
        "AccessibleButCapacityConstrained" => {
            add(reasons, "REACHABLE_CAPACITY_BELOW_GROUP_DEMAND");
            "CapacityConstrained"
        }
        "Inaccessible" => {
            if includes_service {
                add(reasons, "ACCESS_RESULT_INTERNALLY_INCONSISTENT");
                return "Invalid";
            }
            add(reasons, "NO_USABLE_SERVICE_WITHIN_ACCESS_ENVELOPE");
            "Inaccessible"
        }
        "Unknown" => {
            add(reasons, "ACCESS_POTENTIAL_UNKNOWN");
            "Unknown"
        }
        _ => {
            add(reasons, "ACCESS_GROUP_STATUS_UNSUPPORTED");
            "Invalid"
        }
    }
}

fn evaluate_in_use(
    owner: &Value,
    receipt_valid: bool,
    scope: &Value,
    scope_sha256: &str,
    cut: &Value,
    reasons: &mut BTreeSet<String>,
) -> &'static str {
    if !receipt_valid {
        return "Unknown";
    }
    let result = object(owner, "result");
    if text(result, "disposition") != "Evaluated" {
        add(reasons, "IN_USE_RESULT_NOT_EVALUATED");
        return "Unknown";
    }
    let requirement = object(scope, "operational_requirement");
    let metric_id = text(requirement, "metric_id");
    let metric_results = array(result, "metric_results");
    let mut matching = metric_results.iter().filter(|metric| text(metric, "metric_id") == metric_id);
    let Some(metric) = matching.next() else {
        add(reasons, "SERVICE_OPERATION_METRIC_MISSING");
        return "Unknown";
    };
    if matching.next().is_some() {
        add(reasons, "SERVICE_OPERATION_METRIC_DUPLICATE");
        return "Invalid";
    }
    if text(metric, "service_id") != text(scope, "service_id")
        || text(metric, "unit") != text(requirement, "unit")
        || text(metric, "window_id") != text(object(scope, "time_window"), "window_id")
    {
        add(reasons, "SERVICE_OPERATION_METRIC_SCOPE_MISMATCH");
        return "Invalid";
    }
    let source_valid = validate_service_observation(
        object(result, "source_observation"),
        scope,
        scope_sha256,
        cut,
        reasons,
    );
    if !source_valid || text(metric, "observation_state") != "Current" {
        add(reasons, "SERVICE_OPERATION_EVIDENCE_NOT_CURRENT");
        return "Unknown";
    }
    let minimum = number(requirement, "minimum").unwrap_or(f64::INFINITY);
    let observed = number(metric, "observed_value");
    let coverage = number(metric, "coverage_fraction");
    let minimum_coverage = number(requirement, "minimum_coverage_fraction").unwrap_or(1.0);
    if coverage.is_none_or(|fraction| fraction < minimum_coverage) {
        add(reasons, "SERVICE_OPERATION_COVERAGE_BELOW_MINIMUM");
        return "Unknown";
    }
    match text(metric, "threshold_state") {
        "Fail" => {
            add(reasons, "SERVICE_OPERATIONAL_FLOOR_FAILED");
            "Failed"
        }
        "Pass" => {
            if observed.is_none_or(|value| value < minimum) {
                add(reasons, "IN_USE_RESULT_INTERNALLY_INCONSISTENT");
                "Invalid"
            } else {
                "Pass"
            }
        }
        _ => {
            add(reasons, "SERVICE_OPERATION_THRESHOLD_UNKNOWN");
            "Unknown"
        }
    }
}

fn is_invalid(reasons: &BTreeSet<String>) -> bool {
    reasons.iter().any(|reason| matches!(
        reason.as_str(),
        "SCOPE_DIGEST_MISMATCH"
            | "SCOPE_DIGEST_INVALID"
            | "OWNER_ROLE_UNSUPPORTED"
            | "DUPLICATE_OWNER_ROLE"
            | "OWNER_RECEIPT_IDENTITY_MISMATCH"
            | "OWNER_RECEIPT_DIGEST_INVALID"
            | "OWNER_SCOPE_IDENTITY_MISMATCH"
            | "OWNER_SCOPE_BINDING_MISMATCH"
            | "OWNER_SCOPE_BINDING_NOT_ESTABLISHED"
            | "OWNER_EVIDENCE_REVOKED_OR_DISPUTED"
            | "OWNER_RIGHTS_NOT_CLEARED"
            | "OWNER_ISSUER_NOT_AUTHORIZED"
            | "SCOPE_BINDING_RECEIPT_IDENTITY_OR_DIGEST_INVALID"
            | "SCOPE_BINDING_AUTHORITY_OR_RIGHTS_INVALID"
            | "ACCESS_GROUP_DUPLICATE"
            | "ACCESS_GROUP_SCOPE_MISMATCH"
            | "ACCESS_RESULT_INTERNALLY_INCONSISTENT"
            | "ACCESS_RESULT_DISPOSITION_INCONSISTENT"
            | "ACCESS_GROUP_STATUS_UNSUPPORTED"
            | "SERVICE_OPERATION_SOURCE_SCOPE_MISMATCH"
            | "SERVICE_OPERATION_SOURCE_DIGEST_INVALID"
            | "SERVICE_OPERATION_SOURCE_REVOKED_OR_DISPUTED"
            | "SERVICE_OPERATION_SOURCE_AUTHORITY_OR_RIGHTS_INVALID"
            | "SERVICE_OPERATION_SCOPE_BINDING_INVALID"
            | "SERVICE_OPERATION_METRIC_DUPLICATE"
            | "SERVICE_OPERATION_METRIC_SCOPE_MISMATCH"
            | "IN_USE_RESULT_INTERNALLY_INCONSISTENT"
    ))
}

/// Evaluate one fully materialized composition case. This function never grants action authority.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let scope = object(case, "evaluation_scope");
    let cut = object(case, "evaluation_cut");
    let mut reasons = BTreeSet::<String>::new();

    let scope_hash = match canonical_scope_sha256(scope) {
        Ok(hash) => hash,
        Err(error) => {
            add(&mut reasons, &error);
            String::new()
        }
    };
    if !valid_hex(text(case, "scope_sha256"), 64) {
        add(&mut reasons, "SCOPE_DIGEST_INVALID");
    } else if text(case, "scope_sha256") != scope_hash {
        add(&mut reasons, "SCOPE_DIGEST_MISMATCH");
    }
    if text(scope, "service_id").is_empty()
        || text(scope, "subject_id").is_empty()
        || !valid_hex(text(scope, "subject_digest"), 64)
        || text(scope, "functional_unit").is_empty()
        || text(scope, "population_frame_id").is_empty()
        || text(scope, "group_id").is_empty()
        || text(scope, "scenario_id").is_empty()
        || text(object(scope, "time_window"), "window_id").is_empty()
        || integer(object(scope, "time_window"), "end_tick").is_none_or(|end| {
            integer(object(scope, "time_window"), "start_tick").is_none_or(|start| end <= start)
        })
        || number(object(scope, "demand"), "value").is_none_or(|value| value <= 0.0)
        || text(object(scope, "demand"), "unit").is_empty()
    {
        add(&mut reasons, "EVALUATION_SCOPE_INVALID");
    }

    let mut owners = BTreeMap::<String, &Value>::new();
    for owner in array(case, "owners") {
        let role = text(owner, "role");
        if expected_contract(role).is_none() {
            add(&mut reasons, "OWNER_ROLE_UNSUPPORTED");
            continue;
        }
        if owners.insert(role.to_owned(), owner).is_some() {
            add(&mut reasons, "DUPLICATE_OWNER_ROLE");
        }
    }
    for role in REQUIRED_ROLES {
        if !owners.contains_key(*role) {
            add(&mut reasons, "OWNER_RECEIPT_MISSING");
        }
    }

    let mut receipt_valid = BTreeMap::<String, bool>::new();
    for role in REQUIRED_ROLES {
        if let Some(owner) = owners.get(*role) {
            receipt_valid.insert(
                (*role).to_owned(),
                validate_owner_receipt(role, owner, scope, &scope_hash, cut, &mut reasons),
            );
        }
    }

    let restoration_state = owners.get("restoration").map_or("Unknown", |owner| {
        evaluate_restoration(
            owner,
            receipt_valid.get("restoration").copied().unwrap_or(false),
            scope,
            &mut reasons,
        )
    });
    let access_state = owners.get("access").map_or("Unknown", |owner| {
        evaluate_access(
            owner,
            receipt_valid.get("access").copied().unwrap_or(false),
            scope,
            &mut reasons,
        )
    });
    let operational_state = owners.get("in_use").map_or("Unknown", |owner| {
        evaluate_in_use(
            owner,
            receipt_valid.get("in_use").copied().unwrap_or(false),
            scope,
            &scope_hash,
            cut,
            &mut reasons,
        )
    });

    let disposition = if is_invalid(&reasons) || restoration_state == "Invalid"
        || access_state == "Invalid"
        || operational_state == "Invalid"
    {
        "Invalid"
    } else if restoration_state == "NotQualified" {
        "RestorationNotQualified"
    } else if access_state == "CapacityConstrained" || access_state == "Inaccessible" {
        "AccessConstrained"
    } else if operational_state == "Failed" {
        "OperationalFloorFailed"
    } else if restoration_state != "Qualified"
        || access_state != "AccessibleAndAdequatePotential"
        || operational_state != "Pass"
    {
        "InsufficientEvidence"
    } else {
        "RestoredAccessPotentialAndOperationSupported"
    };

    json!({
        "case_id": case_id,
        "disposition": disposition,
        "restoration_state": restoration_state,
        "access_state": access_state,
        "operational_delivery_state": operational_state,
        "realized_use_state": "NotMeasured",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING
    })
}

pub fn validate_corpus(corpus: &Value) -> Vec<String> {
    let mut errors = BTreeSet::<String>::new();
    if text(corpus, "schema_id")
        != "org.luminousdynamics.city-habitat-service-composition.corpus"
        || text(corpus, "schema_version") != "0.1.0"
        || text(corpus, "corpus_id") != CORPUS_ID
        || text(corpus, "classification") != "synthetic_fixture_only"
        || text(corpus, "claim_ceiling") != CLAIM_CEILING
        || text(corpus, "policy_note").is_empty()
    {
        add(&mut errors, "CORPUS_IDENTITY_OR_CLAIM_CEILING_INVALID");
    }
    if array(corpus, "cases").len() != EXPECTED_CASES {
        add(&mut errors, "CORPUS_CASE_COUNT_INVALID");
    }
    let mut ids = BTreeSet::new();
    for (index, case) in array(corpus, "cases").iter().enumerate() {
        let id = text(case, "case_id");
        if id.is_empty() || !ids.insert(id.to_owned()) {
            add(&mut errors, &format!("CASE_ID_EMPTY_OR_DUPLICATE:{index}"));
        }
        if !case.get("expected").is_some_and(Value::is_object) {
            add(&mut errors, &format!("EXPECTED_OUTPUT_INVALID:{id}"));
        }
        if !case.get("evaluation_scope").is_some_and(Value::is_object)
            || !case.get("evaluation_cut").is_some_and(Value::is_object)
            || !case.get("owners").is_some_and(Value::is_array)
        {
            add(&mut errors, &format!("CASE_INPUT_INVALID:{id}"));
        }
    }
    errors.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS: &str = include_str!(
        "../../../docs/research/city-habitat-service-composition/CITY_HABITAT_SERVICE_COMPOSITION_CORPUS_V0_1.json"
    );

    fn corpus() -> Value {
        serde_json::from_str(CORPUS).expect("frozen corpus parses")
    }

    #[test]
    fn all_twenty_four_frozen_cases_match_exactly() {
        let corpus = corpus();
        assert!(validate_corpus(&corpus).is_empty(), "{:?}", validate_corpus(&corpus));
        for case in array(&corpus, "cases") {
            let actual = evaluate_case(case);
            assert_eq!(
                actual,
                case["expected"],
                "{} mismatch\nactual: {}\nexpected: {}",
                text(case, "case_id"),
                actual,
                case["expected"]
            );
        }
    }

    #[test]
    fn scope_hash_is_canonical_and_bound_to_all_declared_dimensions() {
        let case = &corpus()["cases"][0];
        assert_eq!(
            canonical_scope_sha256(&case["evaluation_scope"]).unwrap(),
            case["scope_sha256"]
        );
        let mut altered = case["evaluation_scope"].clone();
        altered["population_frame_id"] = json!("another-frame");
        assert_ne!(
            canonical_scope_sha256(&altered).unwrap(),
            case["scope_sha256"].as_str().unwrap()
        );
    }

    #[test]
    fn owner_record_order_does_not_change_composition() {
        let mut case = corpus()["cases"][0].clone();
        let before = evaluate_case(&case);
        case["owners"].as_array_mut().unwrap().reverse();
        assert_eq!(before, evaluate_case(&case));
    }

    #[test]
    fn reachable_capacity_is_not_promoted_when_the_group_is_inaccessible() {
        let case = corpus()["cases"].as_array().unwrap()
            .iter().find(|item| text(item, "case_id") == "HSC-004").unwrap();
        let result = evaluate_case(case);
        let access = array(case, "owners").iter()
            .find(|owner| text(owner, "role") == "access")
            .expect("access owner");
        assert_eq!(
            access["result"]["category_totals"][0]["status"],
            "KnownAggregateCapacityAdequate"
        );
        assert_eq!(result["access_state"], "Inaccessible");
        assert_eq!(result["disposition"], "AccessConstrained");
        assert_eq!(result["realized_use_state"], "NotMeasured");
    }

    #[test]
    fn operational_availability_never_claims_realized_service_use() {
        let case = corpus()["cases"].as_array().unwrap()
            .iter().find(|item| text(item, "case_id") == "HSC-001").unwrap();
        let result = evaluate_case(case);
        assert_eq!(result["operational_delivery_state"], "Pass");
        assert_eq!(result["realized_use_state"], "NotMeasured");
        assert_eq!(result["action_authority"], "none");
    }

    #[test]
    fn late_ingested_owner_source_is_not_visible_at_the_evaluation_cut() {
        let case = corpus()["cases"].as_array().unwrap()
            .iter().find(|item| text(item, "case_id") == "HSC-012").unwrap();
        let result = evaluate_case(case);
        assert_eq!(result["operational_delivery_state"], "Unknown");
        assert_eq!(result["disposition"], "InsufficientEvidence");
    }

    #[test]
    fn duplicate_owner_roles_are_rejected() {
        let case = corpus()["cases"].as_array().unwrap()
            .iter().find(|item| text(item, "case_id") == "HSC-019").unwrap();
        assert_eq!(evaluate_case(case)["disposition"], "Invalid");
    }
}
