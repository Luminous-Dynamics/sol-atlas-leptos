//! Synthetic reference oracle for CityMaker production-geography adapters.
//!
//! This is not a site-selection optimizer or a production, resource, routing,
//! capability-closure, zoning or consent authority. It verifies that each
//! comparison dimension is backed by an exact, current owner receipt and
//! aggregates non-compensatory hard-gate outcomes without recomputing them.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

const CURRENT_FIXTURE_EVIDENCE: &str = "synthetic_current";

#[derive(Clone, Debug)]
struct Dimension {
    id: String,
    owner: String,
    contract: String,
    revision: String,
    hard_gate: bool,
}

fn array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn empty_unsupported(case_id: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id,
        "disposition": "Unsupported",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "alternatives": []
    })
}

/// Evaluate one scenario from owner-receipt inputs only. The expected output
/// is never read and cannot influence comparison eligibility or gate status.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = string(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let mut errors = BTreeSet::<String>::new();

    let scope_digest = string(input, "scope_digest");
    if scope_digest.is_empty() {
        errors.insert("COMPARISON_SCOPE_DIGEST_MISSING".to_owned());
    }

    let Some(dimension_values) = array(input, "dimensions") else {
        errors.insert("REQUIRED_DIMENSIONS_MISSING".to_owned());
        return empty_unsupported(case_id, errors);
    };
    if dimension_values.is_empty() {
        errors.insert("REQUIRED_DIMENSIONS_EMPTY".to_owned());
    }

    let Some(alternative_values) = array(input, "alternatives") else {
        errors.insert("ALTERNATIVES_MISSING".to_owned());
        return empty_unsupported(case_id, errors);
    };
    if alternative_values.is_empty() {
        errors.insert("ALTERNATIVES_EMPTY".to_owned());
    }

    let mut dimensions = Vec::<Dimension>::new();
    let mut dimension_ids = BTreeSet::<String>::new();

    for dimension in dimension_values {
        let id = string(dimension, "id");
        let owner = string(dimension, "owner");
        let contract = string(dimension, "contract");
        let revision = string(dimension, "revision");
        let Some(hard_gate) = dimension.get("hard_gate").and_then(Value::as_bool) else {
            errors.insert("DIMENSION_HARD_GATE_FLAG_INVALID".to_owned());
            continue;
        };

        if id.is_empty() || owner.is_empty() || contract.is_empty() || revision.is_empty() {
            errors.insert("DIMENSION_DEFINITION_INVALID".to_owned());
            continue;
        }
        if !dimension_ids.insert(id.to_owned()) {
            errors.insert("DUPLICATE_DIMENSION_ID".to_owned());
            continue;
        }

        dimensions.push(Dimension {
            id: id.to_owned(),
            owner: owner.to_owned(),
            contract: contract.to_owned(),
            revision: revision.to_owned(),
            hard_gate,
        });
    }

    let mut alternative_ids = BTreeSet::<String>::new();
    let mut source_receipt_ids = BTreeSet::<String>::new();

    for alternative in alternative_values {
        let alternative_id = string(alternative, "id");
        if alternative_id.is_empty() || !alternative_ids.insert(alternative_id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_ALTERNATIVE_ID".to_owned());
        }

        let Some(receipts) = array(alternative, "receipts") else {
            errors.insert("ALTERNATIVE_RECEIPTS_MISSING".to_owned());
            continue;
        };

        let mut seen_dimensions = BTreeSet::<String>::new();
        for receipt in receipts {
            let dimension_id = string(receipt, "dimension_id");
            let receipt_id = string(receipt, "receipt_id");

            if dimension_id.is_empty() || receipt_id.is_empty() {
                errors.insert("RECEIPT_IDENTITY_MISSING".to_owned());
            }
            if !seen_dimensions.insert(dimension_id.to_owned()) {
                errors.insert("DUPLICATE_DIMENSION_RECEIPT".to_owned());
            }
            if !source_receipt_ids.insert(receipt_id.to_owned()) {
                errors.insert("DUPLICATE_SOURCE_RECEIPT_ID".to_owned());
            }
            if !dimension_ids.contains(dimension_id) {
                errors.insert("UNDECLARED_RECEIPT_DIMENSION".to_owned());
            }
            if !matches!(string(receipt, "result"), "Pass" | "Fail" | "Unknown") {
                errors.insert("RECEIPT_RESULT_UNSUPPORTED".to_owned());
            }
        }
    }

    if !errors.is_empty() {
        return empty_unsupported(case_id, errors);
    }

    dimensions.sort_by(|left, right| left.id.cmp(&right.id));
    let mut alternatives = alternative_values.iter().collect::<Vec<_>>();
    alternatives.sort_by(|left, right| string(left, "id").cmp(string(right, "id")));

    let mut alternative_outputs = Vec::<Value>::new();

    for alternative in alternatives {
        let alternative_id = string(alternative, "id");
        if string(alternative, "scope_digest") != scope_digest {
            alternative_outputs.push(json!({
                "alternative_id": alternative_id,
                "status": "NotComparable",
                "reason_codes": ["ALTERNATIVE_SCOPE_MISMATCH"],
                "dimension_results": []
            }));
            continue;
        }

        let receipts = array(alternative, "receipts").expect("receipt arrays validated");
        let receipt_by_dimension = receipts
            .iter()
            .map(|receipt| (string(receipt, "dimension_id"), receipt))
            .collect::<BTreeMap<_, _>>();

        let mut dimension_results = Vec::<Value>::new();
        let mut reasons = BTreeSet::<String>::new();

        for dimension in &dimensions {
            let receipt = receipt_by_dimension.get(dimension.id.as_str()).copied();
            let (status, reason_code) = match receipt {
                None => ("Unknown", Some("REQUIRED_OWNER_RECEIPT_MISSING")),
                Some(receipt) => {
                    let expected_subject = format!(
                        "synthetic-subject:{case_id}:{alternative_id}:{}",
                        dimension.id
                    );
                    let identity_matches = string(receipt, "owner") == dimension.owner
                        && string(receipt, "contract") == dimension.contract
                        && string(receipt, "source_revision") == dimension.revision
                        && string(receipt, "source_scope_digest") == scope_digest
                        && string(receipt, "source_subject_digest") == expected_subject
                        && string(receipt, "evidence_state") == CURRENT_FIXTURE_EVIDENCE;

                    if !identity_matches {
                        (
                            "Unknown",
                            Some("OWNER_RECEIPT_IDENTITY_OR_EVIDENCE_MISMATCH"),
                        )
                    } else {
                        match string(receipt, "result") {
                            "Pass" => ("Pass", None),
                            "Fail" => ("Fail", Some("OWNER_HARD_GATE_FAILED")),
                            _ => ("Unknown", Some("OWNER_EVIDENCE_UNKNOWN")),
                        }
                    }
                }
            };

            if let Some(reason) = reason_code {
                reasons.insert(format!("{reason}:{}", dimension.id));
            }

            dimension_results.push(json!({
                "dimension_id": dimension.id,
                "hard_gate": dimension.hard_gate,
                "status": status,
                "reason_code": reason_code
            }));
        }

        let has_hard_gate_failure = dimension_results
            .iter()
            .any(|result| result["hard_gate"] == true && result["status"] == "Fail");
        let has_unknown = dimension_results
            .iter()
            .any(|result| result["status"] == "Unknown");

        let status = if has_hard_gate_failure {
            "Infeasible"
        } else if has_unknown {
            "Unknown"
        } else {
            "EligibleForComparison"
        };

        alternative_outputs.push(json!({
            "alternative_id": alternative_id,
            "status": status,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "dimension_results": dimension_results
        }));
    }

    json!({
        "case_id": case_id,
        "disposition": "Evaluated",
        "reason_codes": [],
        "alternatives": alternative_outputs
    })
}

fn json_equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .is_some_and(|(left, right)| left == right),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| json_equivalent(left, right))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && right.iter().all(|(key, expected_value)| {
                    left.get(key)
                        .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
                })
        }
        _ => actual == expected,
    }
}

/// Compare declared known-answer fields. JSON integer/floating encodings
/// compare by exact numeric value; strings, arrays and object keys stay exact.
pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    expected.as_object().is_some_and(|fields| {
        fields.iter().all(|(key, expected_value)| {
            actual
                .get(key)
                .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-maker/CITY_PRODUCTION_GEOGRAPHY_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("production geography corpus must exist");
        serde_json::from_slice(&bytes).expect("production geography corpus must parse")
    }

    fn case(id: &str) -> Value {
        corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["case_id"] == id)
            .unwrap()
            .clone()
    }

    #[test]
    fn corpus_has_24_unique_synthetic_cases() {
        let corpus = corpus();
        assert_eq!(corpus["classification"], "synthetic_fixture_only");
        assert_eq!(corpus["corpus_id"], "CITY-PRODUCTION-GEOGRAPHY-CORPUS-V0.1");
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        let ids = cases
            .iter()
            .map(|item| item["case_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), cases.len());
    }

    #[test]
    fn independent_reference_oracle_matches_every_known_answer() {
        for item in corpus()["cases"].as_array().unwrap() {
            let actual = evaluate_case(item);
            assert!(
                matches_expected(&actual, &item["expected"]),
                "{} mismatch\nactual: {}\nexpected: {}",
                item["case_id"],
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&item["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn explicit_hard_gate_failure_is_infeasible() {
        let result = evaluate_case(&case("PG-002"));
        assert_eq!(result["alternatives"][0]["status"], "Infeasible");
    }

    #[test]
    fn known_failure_takes_precedence_over_unknown_evidence() {
        let result = evaluate_case(&case("PG-015"));
        assert_eq!(result["alternatives"][0]["status"], "Infeasible");
        assert!(
            result["alternatives"][0]["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason
                    == "OWNER_RECEIPT_IDENTITY_OR_EVIDENCE_MISMATCH:worker_access")
        );
    }

    #[test]
    fn stale_or_mismatched_owner_receipts_never_pass() {
        for id in [
            "PG-003", "PG-006", "PG-008", "PG-009", "PG-010", "PG-011", "PG-012", "PG-013",
            "PG-023", "PG-024",
        ] {
            let result = evaluate_case(&case(id));
            assert_eq!(result["alternatives"][0]["status"], "Unknown", "{id}");
        }
    }

    #[test]
    fn missing_required_owner_receipt_is_unknown() {
        for id in ["PG-004", "PG-016"] {
            let result = evaluate_case(&case(id));
            assert_eq!(result["alternatives"][0]["status"], "Unknown", "{id}");
        }
    }

    #[test]
    fn candidate_scope_mismatch_is_not_comparable() {
        let result = evaluate_case(&case("PG-014"));
        assert_eq!(result["alternatives"][1]["status"], "NotComparable");
    }

    #[test]
    fn duplicate_and_unregistered_receipts_fail_closed() {
        for id in ["PG-017", "PG-018", "PG-019", "PG-020"] {
            assert_eq!(
                evaluate_case(&case(id))["disposition"],
                "Unsupported",
                "{id}"
            );
        }
    }

    #[test]
    fn all_current_pass_receipts_are_eligible_for_owner_frontier_comparison() {
        let result = evaluate_case(&case("PG-001"));
        assert_eq!(result["alternatives"][0]["status"], "EligibleForComparison");
        assert_eq!(
            result["alternatives"][0]["dimension_results"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
    }

    #[test]
    fn input_permutation_does_not_change_normalized_owner_gating() {
        let mut item = case("PG-021");
        let baseline = evaluate_case(&item);
        item["input"]["dimensions"]
            .as_array_mut()
            .unwrap()
            .reverse();
        item["input"]["alternatives"]
            .as_array_mut()
            .unwrap()
            .reverse();
        for alternative in item["input"]["alternatives"].as_array_mut().unwrap() {
            alternative["receipts"].as_array_mut().unwrap().reverse();
        }
        let permuted = evaluate_case(&item);
        assert_eq!(baseline, permuted);
    }
}
