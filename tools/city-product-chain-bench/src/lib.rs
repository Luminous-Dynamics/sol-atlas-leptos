//! Synthetic reference oracle for service-to-product lifecycle evidence chains.
//!
//! The evaluator validates required owner receipt identity and lifecycle
//! completeness. It does not produce engineering, capability, inventory,
//! procurement, commissioning, or local-manufacture authority.

use std::collections::{BTreeMap, BTreeSet};
use serde_json::{Value, json};

const STAGE_ORDER: &[&str] = &[
    "design", "process", "capability", "qualified_material", "available_supply",
    "delivery_receiving", "installation", "commissioning", "maintenance", "recovery",
];

fn array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}
fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}
fn empty_unsupported(case_id: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id, "disposition": "Unsupported",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(), "requirements": []
    })
}
fn stage_rank(stage: &str) -> usize {
    STAGE_ORDER.iter().position(|candidate| *candidate == stage).unwrap_or(usize::MAX)
}
fn stage_state(case_id: &str, scope: &str, requirement: &Value, definition: &Value, receipt: Option<&Value>) -> Value {
    let stage = text(definition, "stage");
    let Some(receipt) = receipt else {
        return json!({"stage": stage, "state": "Unknown", "reason_codes": ["STAGE_RECEIPT_MISSING"]});
    };
    let req_id = text(requirement, "requirement_id");
    let bom = text(requirement, "design_revision");
    let subject = format!("subject:{case_id}:{req_id}:{stage}:{bom}");
    let identity = text(receipt, "requirement_id") == req_id
        && text(receipt, "product_family_id") == text(requirement, "product_family_id")
        && text(receipt, "stage") == stage
        && text(receipt, "owner") == text(definition, "owner")
        && text(receipt, "contract") == text(definition, "contract")
        && text(receipt, "source_revision") == text(definition, "required_source_revision")
        && text(receipt, "scope_digest") == scope
        && text(receipt, "design_revision") == bom
        && text(receipt, "source_subject_digest") == subject
        && !text(receipt, "source_receipt_id").is_empty();
    if !identity {
        return json!({"stage": stage, "state": "Unknown", "reason_codes": ["RECEIPT_IDENTITY_MISMATCH"]});
    }
    if text(receipt, "evidence_state") != "current" {
        return json!({"stage": stage, "state": "Unknown", "reason_codes": ["EVIDENCE_STATE_NOT_CURRENT"]});
    }
    match text(receipt, "result") {
        "satisfied" => json!({"stage": stage, "state": "Satisfied", "reason_codes": []}),
        "blocked" | "rejected" | "unavailable" => json!({"stage": stage, "state": "Blocked", "reason_codes": ["STAGE_RESULT_BLOCKED"]}),
        _ => json!({"stage": stage, "state": "Unknown", "reason_codes": ["STAGE_RESULT_NOT_QUALIFIED"]}),
    }
}

/// Recompute one lifecycle chain from inputs only. The expected output is not read.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let mut errors = BTreeSet::<String>::new();
    let scope = text(input, "scope_digest");
    if scope.is_empty() { errors.insert("EVALUATION_SCOPE_DIGEST_MISSING".to_owned()); }
    let Some(requirements) = array(input, "requirements") else {
        errors.insert("REQUIREMENTS_MISSING".to_owned());
        return empty_unsupported(case_id, errors);
    };
    let Some(receipts) = array(input, "receipts") else {
        errors.insert("RECEIPTS_MISSING".to_owned());
        return empty_unsupported(case_id, errors);
    };
    if requirements.is_empty() { errors.insert("REQUIREMENTS_EMPTY".to_owned()); }

    let mut req_ids = BTreeSet::<String>::new();
    let mut req_by_id = BTreeMap::<String, &Value>::new();
    for req in requirements {
        let id = text(req, "requirement_id");
        if id.is_empty() || !req_ids.insert(id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_REQUIREMENT_ID".to_owned());
            continue;
        }
        if text(req, "service_id").is_empty() || text(req, "district_id").is_empty()
            || text(req, "product_family_id").is_empty() || text(req, "design_revision").is_empty()
            || !matches!(text(req, "sourcing_class"), "local" | "regional" | "imported" | "recovered" | "retained")
        {
            errors.insert("REQUIREMENT_PROFILE_INVALID".to_owned());
        }
        req_by_id.insert(id.to_owned(), req);
        let Some(stages) = array(req, "required_stages") else {
            errors.insert("REQUIRED_STAGES_MISSING".to_owned());
            continue;
        };
        if stages.is_empty() { errors.insert("REQUIRED_STAGE_LIST_EMPTY".to_owned()); }
        let mut seen = BTreeSet::<String>::new();
        for def in stages {
            let stage = text(def, "stage");
            if !STAGE_ORDER.contains(&stage) || !seen.insert(stage.to_owned())
                || text(def, "owner").is_empty() || text(def, "contract").is_empty()
                || text(def, "required_source_revision").is_empty()
            {
                errors.insert("REQUIRED_STAGE_PROFILE_INVALID".to_owned());
            }
        }
    }

    let mut receipt_ids = BTreeSet::<String>::new();
    let mut pairs = BTreeSet::<(String, String)>::new();
    for receipt in receipts {
        let req_id = text(receipt, "requirement_id");
        let stage = text(receipt, "stage");
        let Some(req) = req_by_id.get(req_id) else {
            errors.insert("RECEIPT_REQUIREMENT_UNKNOWN".to_owned());
            continue;
        };
        if !array(req, "required_stages").is_some_and(|defs| defs.iter().any(|d| text(d, "stage") == stage)) {
            errors.insert("RECEIPT_STAGE_UNEXPECTED".to_owned());
        }
        if !pairs.insert((req_id.to_owned(), stage.to_owned())) {
            errors.insert("DUPLICATE_REQUIREMENT_STAGE_RECEIPT".to_owned());
        }
        let receipt_id = text(receipt, "source_receipt_id");
        if receipt_id.is_empty() || !receipt_ids.insert(receipt_id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_SOURCE_RECEIPT_ID".to_owned());
        }
    }
    if !errors.is_empty() { return empty_unsupported(case_id, errors); }

    let mut ordered = requirements.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| text(a, "requirement_id").cmp(text(b, "requirement_id")));
    let mut output = Vec::new();
    let mut ready = Vec::new();
    let mut blocked = Vec::new();
    let mut unknown = Vec::new();

    for req in ordered {
        let req_id = text(req, "requirement_id");
        let mut defs = array(req, "required_stages").unwrap().iter().collect::<Vec<_>>();
        defs.sort_by_key(|d| stage_rank(text(d, "stage")));
        let mut stages = Vec::new();
        for def in defs {
            let stage = text(def, "stage");
            let receipt = receipts.iter().find(|r| text(r, "requirement_id") == req_id && text(r, "stage") == stage);
            stages.push(stage_state(case_id, scope, req, def, receipt));
        }
        let is_blocked = stages.iter().any(|x| text(x, "state") == "Blocked");
        let is_unknown = stages.iter().any(|x| text(x, "state") != "Satisfied");
        let status = if is_blocked {
            blocked.push(req_id.to_owned());
            "Blocked"
        } else if is_unknown {
            unknown.push(req_id.to_owned());
            "Unknown"
        } else {
            ready.push(req_id.to_owned());
            "ReadyForDeclaredLifecycle"
        };
        output.push(json!({
            "requirement_id": req_id,
            "service_id": text(req, "service_id"),
            "district_id": text(req, "district_id"),
            "product_family_id": text(req, "product_family_id"),
            "sourcing_class": text(req, "sourcing_class"),
            "local_productive_capability_claim": "not_assessed_by_coverage_oracle",
            "status": status,
            "stages": stages
        }));
    }
    json!({
        "case_id": case_id, "disposition": "Evaluated", "reason_codes": [],
        "ready_requirement_ids": ready, "blocked_requirement_ids": blocked,
        "unknown_requirement_ids": unknown, "requirements": output
    })
}

fn equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64().zip(y.as_f64()).is_some_and(|(x, y)| x == y),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(a, b)| equivalent(a, b)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && y.iter().all(|(k, v)| x.get(k).is_some_and(|a| equivalent(a, v))),
        _ => a == b,
    }
}
pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    expected.as_object().is_some_and(|fields| fields.iter().all(|(k, v)| actual.get(k).is_some_and(|a| equivalent(a, v))))
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/research/city-product-chain/CITY_PRODUCT_CHAIN_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("product-chain corpus must exist");
        serde_json::from_slice(&bytes).expect("product-chain corpus must parse")
    }
    fn case(id: &str) -> Value {
        corpus()["cases"].as_array().unwrap().iter().find(|x| x["case_id"] == id).unwrap().clone()
    }

    #[test]
    fn corpus_has_24_unique_synthetic_cases() {
        let c = corpus();
        assert_eq!(c["classification"], "synthetic_fixture_only");
        assert_eq!(c["corpus_id"], "CITY-PRODUCT-CHAIN-CORPUS-V0.1");
        let xs = c["cases"].as_array().unwrap();
        assert_eq!(xs.len(), 24);
        let ids = xs.iter().map(|x| x["case_id"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), xs.len());
    }
    #[test]
    fn oracle_matches_all_known_answers() {
        for item in corpus()["cases"].as_array().unwrap() {
            let actual = evaluate_case(item);
            assert!(matches_expected(&actual, &item["expected"]), "{} mismatch\nactual: {}\nexpected: {}", item["case_id"], serde_json::to_string_pretty(&actual).unwrap(), serde_json::to_string_pretty(&item["expected"]).unwrap());
        }
    }
    #[test]
    fn imported_supply_never_implies_local_production() {
        let r = evaluate_case(&case("PC-021"));
        assert_eq!(r["requirements"][0]["status"], "ReadyForDeclaredLifecycle");
        assert_eq!(r["requirements"][0]["sourcing_class"], "imported");
        assert_eq!(r["requirements"][0]["local_productive_capability_claim"], "not_assessed_by_coverage_oracle");
    }
    #[test]
    fn district_gaps_remain_disaggregated() {
        let r = evaluate_case(&case("PC-020"));
        assert_eq!(r["requirements"][0]["status"], "ReadyForDeclaredLifecycle");
        assert_eq!(r["requirements"][1]["status"], "Unknown");
        assert_eq!(r["unknown_requirement_ids"], serde_json::json!(["district-b-wall"]));
    }
    #[test]
    fn blocked_stages_cannot_be_offset() {
        let r = evaluate_case(&case("PC-003"));
        assert_eq!(r["requirements"][0]["status"], "Blocked");
        assert_eq!(r["blocked_requirement_ids"], serde_json::json!(["housing-wall"]));
    }
    #[test]
    fn missing_or_untrusted_receipts_never_produce_readiness() {
        for id in ["PC-002", "PC-005", "PC-008", "PC-009", "PC-010", "PC-012", "PC-013", "PC-014", "PC-015", "PC-016", "PC-017", "PC-018", "PC-022", "PC-024"] {
            assert_ne!(evaluate_case(&case(id))["requirements"][0]["status"], "ReadyForDeclaredLifecycle", "{id}");
        }
    }
    #[test]
    fn duplicate_receipt_replay_is_unsupported() {
        for id in ["PC-019", "PC-023"] {
            assert_eq!(evaluate_case(&case(id))["disposition"], "Unsupported");
        }
    }
    #[test]
    fn input_permutation_does_not_change_normalized_results() {
        let mut item = case("PC-022");
        let before = evaluate_case(&item);
        item["input"]["requirements"].as_array_mut().unwrap().reverse();
        item["input"]["receipts"].as_array_mut().unwrap().reverse();
        assert_eq!(before, evaluate_case(&item));
    }
    #[test]
    fn complete_chain_is_reported_in_lifecycle_order() {
        let r = evaluate_case(&case("PC-001"));
        let stages = r["requirements"][0]["stages"].as_array().unwrap().iter().map(|x| x["stage"].as_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(stages, ["design", "process", "capability", "qualified_material", "available_supply", "delivery_receiving", "installation", "commissioning", "maintenance", "recovery"]);
    }
}
