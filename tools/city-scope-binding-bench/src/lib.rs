//! Synthetic reference oracle for CityMaker's common evaluation-scope binder.
//!
//! This crate validates only the thin metadata envelope around owner-native
//! artifacts. It never interprets owner payloads, changes owner dispositions,
//! verifies payload bytes, or promotes an owner claim. Canonical scope
//! serialization is schema-aware: declared set-like arrays are sorted by
//! stable identity, while other arrays retain their order.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

const SCOPE_VERSION: &str = "citymaker-evaluation-scope/0.1";

fn array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonical_scope(scope: &Value) -> Value {
    let mut value = scope.clone();

    if let Some(groups) = value
        .get_mut("population_frame")
        .and_then(|frame| frame.get_mut("group_ids"))
        .and_then(Value::as_array_mut)
    {
        groups.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    }

    if let Some(scenarios) = value.get_mut("scenarios").and_then(Value::as_array_mut) {
        for scenario in scenarios.iter_mut() {
            if let Some(events) = scenario.get_mut("event_ids").and_then(Value::as_array_mut) {
                events.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
            }
        }
        scenarios.sort_by(|left, right| text(left, "scenario_id").cmp(text(right, "scenario_id")));
    }

    if let Some(requirements) = value
        .get_mut("service_requirements")
        .and_then(Value::as_array_mut)
    {
        requirements.sort_by(|left, right| {
            text(left, "requirement_id").cmp(text(right, "requirement_id"))
        });
    }

    if let Some(units) = value.get_mut("functional_units").and_then(Value::as_array_mut) {
        units.sort_by(|left, right| {
            text(left, "functional_unit_id").cmp(text(right, "functional_unit_id"))
        });
    }

    if let Some(revisions) = value
        .get_mut("design_revisions")
        .and_then(Value::as_array_mut)
    {
        revisions.sort_by(|left, right| text(left, "subject_id").cmp(text(right, "subject_id")));
    }

    canonicalize_value(value)
}

fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_value).collect()),
        Value::Object(object) => {
            let mut entries = object.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut canonical = Map::new();
            for (key, value) in entries {
                canonical.insert(key, canonicalize_value(value));
            }
            Value::Object(canonical)
        }
        other => other,
    }
}

pub fn scope_digest(scope: &Value) -> String {
    let canonical = canonical_scope(scope);
    let bytes = serde_json::to_vec(&canonical).expect("canonical scope serializes");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn binding_entry(artifact_id: &str, state: &str, raw_owner_disposition: &str) -> Value {
    json!({
        "artifact_id": artifact_id,
        "binding_state": state,
        "raw_owner_disposition": if raw_owner_disposition.is_empty() {
            "Missing"
        } else {
            raw_owner_disposition
        },
        "payload_bytes_verified": false
    })
}

fn empty_binding(case_id: &str, disposition: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id,
        "binding_disposition": disposition,
        "scope_digest": null,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "required_artifact_ids": [],
        "artifact_bindings": [],
        "owner_result_promotion": "none"
    })
}

/// Evaluate a case from input only. The expected known-answer object is never read.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let mut errors = BTreeSet::<String>::new();
    let mut not_comparable = BTreeSet::<String>::new();
    let mut unknown = BTreeSet::<String>::new();

    let Some(scope) = input.get("scope") else {
        errors.insert("SCOPE_MISSING".to_owned());
        return empty_binding(case_id, "Unsupported", errors);
    };

    if text(scope, "schema_version") != SCOPE_VERSION {
        errors.insert("SCOPE_SCHEMA_UNSUPPORTED".to_owned());
    }

    let classification = text(scope, "classification");
    let spatial_scope = scope.get("spatial_scope").unwrap_or(&Value::Null);
    if classification == "synthetic_fixture_only"
        && spatial_scope
            .get("real_world_coordinates_present")
            .and_then(Value::as_bool)
            == Some(true)
    {
        errors.insert("SYNTHETIC_SCOPE_HAS_REAL_WORLD_COORDINATES".to_owned());
    }

    let actual_scope_digest = scope_digest(scope);
    if text(input, "scope_digest") != actual_scope_digest {
        errors.insert("SCOPE_DIGEST_DECLARATION_MISMATCH".to_owned());
    }

    let Some(required_ids) = array(input, "required_artifact_ids") else {
        errors.insert("REQUIRED_ARTIFACT_IDS_MISSING".to_owned());
        return empty_binding(case_id, "Unsupported", errors);
    };

    let mut required = Vec::<String>::new();
    let mut required_set = BTreeSet::<String>::new();
    for item in required_ids {
        let Some(id) = item.as_str() else {
            errors.insert("REQUIRED_ARTIFACT_ID_INVALID".to_owned());
            continue;
        };
        if id.is_empty() || !required_set.insert(id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_REQUIRED_ARTIFACT_ID".to_owned());
            continue;
        }
        required.push(id.to_owned());
    }

    let Some(artifacts) = array(input, "artifacts") else {
        errors.insert("ARTIFACTS_MISSING".to_owned());
        return empty_binding(case_id, "Unsupported", errors);
    };
    let registry = array(input, "owner_schema_registry").unwrap_or(&[]);

    let mut seen_artifact_ids = BTreeSet::<String>::new();
    let mut seen_receipt_ids = BTreeSet::<String>::new();
    let mut artifact_bindings = BTreeMap::<String, Value>::new();

    for artifact in artifacts {
        let artifact_id = text(artifact, "artifact_id");
        if artifact_id.is_empty() || !seen_artifact_ids.insert(artifact_id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_ARTIFACT_ID".to_owned());
            continue;
        }

        let receipt_id = text(artifact, "receipt_id");
        if receipt_id.is_empty() || !seen_receipt_ids.insert(receipt_id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_RECEIPT_ID".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "Unsupported", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        let owner = text(artifact, "owner_repo");
        let contract = text(artifact, "contract_id");
        let version = text(artifact, "contract_version");
        let owner_contract = registry
            .iter()
            .filter(|entry| {
                text(entry, "owner_repo") == owner && text(entry, "contract_id") == contract
            })
            .collect::<Vec<_>>();
        let registration = owner_contract
            .iter()
            .find(|entry| text(entry, "contract_version") == version);

        let Some(registration) = registration else {
            if owner_contract.is_empty() {
                unknown.insert("OWNER_SCHEMA_REGISTRY_ENTRY_MISSING".to_owned());
                artifact_bindings.insert(
                    artifact_id.to_owned(),
                    binding_entry(artifact_id, "Unknown", text(artifact, "raw_owner_disposition")),
                );
            } else {
                errors.insert("OWNER_CONTRACT_VERSION_UNSUPPORTED".to_owned());
                artifact_bindings.insert(
                    artifact_id.to_owned(),
                    binding_entry(artifact_id, "Unsupported", text(artifact, "raw_owner_disposition")),
                );
            }
            continue;
        };

        let schema_digest = text(artifact, "schema_artifact_digest");
        let schema_algorithm = text(artifact, "schema_digest_algorithm");
        let source_revision = text(artifact, "source_revision");
        let content_digest = text(artifact, "content_digest");
        let subject_digest = text(artifact, "subject_digest");
        if schema_digest.is_empty()
            || source_revision.is_empty()
            || content_digest.is_empty()
            || subject_digest.is_empty()
        {
            unknown.insert("OWNER_ARTIFACT_IDENTITY_INCOMPLETE".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "Unknown", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        let schema_digest_length = match schema_algorithm {
            "git_blob_sha1" => 40,
            "sha256" => 64,
            _ => 0,
        };
        if schema_digest_length == 0
            || !valid_hex(schema_digest, schema_digest_length)
            || !valid_hex(source_revision, 40)
            || !valid_hex(content_digest, 64)
            || !valid_hex(subject_digest, 64)
        {
            errors.insert("OWNER_ARTIFACT_DIGEST_OR_REVISION_MALFORMED".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "Unsupported", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        if schema_digest != text(registration, "schema_artifact_digest")
            || schema_algorithm != text(registration, "schema_digest_algorithm")
            || source_revision != text(registration, "source_revision")
        {
            not_comparable.insert("OWNER_SCHEMA_OR_SOURCE_REVISION_MISMATCH".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "NotComparable", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        let population = scope.get("population_frame").unwrap_or(&Value::Null);
        let horizon = scope.get("time_horizon").unwrap_or(&Value::Null);
        if text(artifact, "scope_digest") != actual_scope_digest
            || text(artifact, "boundary_id") != text(spatial_scope, "boundary_id")
            || text(artifact, "boundary_revision") != text(spatial_scope, "boundary_revision")
            || text(artifact, "population_frame_id") != text(population, "frame_id")
            || text(artifact, "horizon_id") != text(horizon, "horizon_id")
            || artifact.get("window_start_tick").and_then(Value::as_i64)
                != horizon.get("start_tick").and_then(Value::as_i64)
            || artifact.get("window_end_tick").and_then(Value::as_i64)
                != horizon.get("end_tick").and_then(Value::as_i64)
        {
            not_comparable.insert("OWNER_ARTIFACT_SCOPE_MISMATCH".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "NotComparable", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        if !array(scope, "scenarios").unwrap_or(&[]).iter().any(|scenario| {
            text(scenario, "scenario_id") == text(artifact, "scenario_id")
        }) {
            not_comparable.insert("SCENARIO_NOT_IN_SCOPE".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "NotComparable", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        if !array(scope, "functional_units").unwrap_or(&[]).iter().any(|unit| {
            text(unit, "functional_unit_id") == text(artifact, "functional_unit_ref")
        }) {
            not_comparable.insert("FUNCTIONAL_UNIT_NOT_IN_SCOPE".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "NotComparable", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        if let Some(weather) = scope.get("weather_profile") {
            if text(artifact, "weather_profile_id") != text(weather, "profile_id") {
                not_comparable.insert("WEATHER_PROFILE_MISMATCH".to_owned());
                artifact_bindings.insert(
                    artifact_id.to_owned(),
                    binding_entry(artifact_id, "NotComparable", text(artifact, "raw_owner_disposition")),
                );
                continue;
            }
        }

        let design_subject = text(artifact, "design_subject_id");
        if !design_subject.is_empty() {
            let design = array(scope, "design_revisions")
                .unwrap_or(&[])
                .iter()
                .find(|revision| text(revision, "subject_id") == design_subject);
            if design.is_none_or(|revision| {
                text(revision, "design_revision") != text(artifact, "design_revision")
                    || text(revision, "bom_revision") != text(artifact, "bom_revision")
            }) {
                not_comparable.insert("DESIGN_OR_BOM_REVISION_MISMATCH".to_owned());
                artifact_bindings.insert(
                    artifact_id.to_owned(),
                    binding_entry(artifact_id, "NotComparable", text(artifact, "raw_owner_disposition")),
                );
                continue;
            }
        }

        if text(artifact, "evidence_state") != "current" {
            unknown.insert("OWNER_ARTIFACT_EVIDENCE_NOT_CURRENT".to_owned());
            artifact_bindings.insert(
                artifact_id.to_owned(),
                binding_entry(artifact_id, "Unknown", text(artifact, "raw_owner_disposition")),
            );
            continue;
        }

        artifact_bindings.insert(
            artifact_id.to_owned(),
            binding_entry(artifact_id, "Bound", text(artifact, "raw_owner_disposition")),
        );
    }

    for required_id in &required {
        if !seen_artifact_ids.contains(required_id) {
            unknown.insert("REQUIRED_ARTIFACT_MISSING".to_owned());
        }
    }

    let disposition = if !errors.is_empty() {
        "Unsupported"
    } else if !not_comparable.is_empty() {
        "NotComparable"
    } else if !unknown.is_empty() {
        "Unknown"
    } else {
        "Bound"
    };
    let mut reason_codes = errors
        .into_iter()
        .chain(not_comparable)
        .chain(unknown)
        .collect::<Vec<_>>();
    reason_codes.sort();
    required.sort();

    json!({
        "case_id": case_id,
        "binding_disposition": disposition,
        "scope_digest": actual_scope_digest,
        "reason_codes": reason_codes,
        "required_artifact_ids": required,
        "artifact_bindings": artifact_bindings.into_values().collect::<Vec<_>>(),
        "owner_result_promotion": "none"
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
                    .zip(right)
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
    use super::{evaluate_case, matches_expected, scope_digest};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-scope-binding/CITY_SCOPE_BINDING_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("scope-binding corpus must exist");
        serde_json::from_slice(&bytes).expect("scope-binding corpus must parse")
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
    fn corpus_has_25_unique_synthetic_cases() {
        let value = corpus();
        assert_eq!(value["classification"], "synthetic_fixture_only");
        assert_eq!(value["corpus_id"], "CITY-SCOPE-BINDING-CORPUS-V0.1");
        let cases = value["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 25);
        let ids = cases
            .iter()
            .map(|item| item["case_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), cases.len());
    }

    #[test]
    fn independent_oracle_matches_all_scope_binding_answers() {
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
    fn baseline_is_bound_without_claiming_payload_digest_verified() {
        let result = evaluate_case(&case("SB-001"));
        assert_eq!(result["binding_disposition"], "Bound");
        assert!(!result["artifact_bindings"][0]["payload_bytes_verified"].as_bool().unwrap());
    }

    #[test]
    fn mismatched_scope_is_not_comparable() {
        for id in ["SB-002", "SB-003", "SB-004", "SB-006"] {
            assert_eq!(evaluate_case(&case(id))["binding_disposition"], "NotComparable");
        }
    }

    #[test]
    fn missing_stale_and_revoked_evidence_is_unknown() {
        for id in ["SB-008", "SB-010", "SB-011", "SB-012", "SB-016", "SB-022"] {
            assert_eq!(evaluate_case(&case(id))["binding_disposition"], "Unknown");
        }
    }

    #[test]
    fn duplicate_and_malformed_identities_are_unsupported() {
        for id in ["SB-013", "SB-014", "SB-020", "SB-021"] {
            assert_eq!(evaluate_case(&case(id))["binding_disposition"], "Unsupported");
        }
    }

    #[test]
    fn raw_blocked_or_import_dependent_results_are_preserved() {
        let blocked = evaluate_case(&case("SB-015"));
        assert_eq!(blocked["binding_disposition"], "Bound");
        assert_eq!(blocked["artifact_bindings"][0]["raw_owner_disposition"], "Blocked");
        let imported = evaluate_case(&case("SB-024"));
        assert_eq!(imported["binding_disposition"], "Bound");
        assert_eq!(imported["artifact_bindings"][0]["raw_owner_disposition"], "ImportDependent");
        assert_eq!(imported["owner_result_promotion"], "none");
    }

    #[test]
    fn owner_schema_and_source_revision_mismatches_are_not_comparable() {
        for id in ["SB-009", "SB-023"] {
            assert_eq!(evaluate_case(&case(id))["binding_disposition"], "NotComparable");
        }
    }

    #[test]
    fn canonical_scope_digest_is_invariant_to_declared_set_order() {
        let mut item = case("SB-001");
        let before = scope_digest(&item["input"]["scope"]);
        item["input"]["scope"]["scenarios"].as_array_mut().unwrap().reverse();
        item["input"]["scope"]["service_requirements"].as_array_mut().unwrap().reverse();
        item["input"]["scope"]["functional_units"].as_array_mut().unwrap().reverse();
        item["input"]["scope"]["population_frame"]["group_ids"].as_array_mut().unwrap().reverse();
        item["input"]["scope"]["design_revisions"].as_array_mut().unwrap().reverse();
        assert_eq!(before, scope_digest(&item["input"]["scope"]));
    }

    #[test]
    fn ordered_arrays_remain_order_sensitive() {
        let mut item = case("SB-001");
        item["input"]["scope"]["custom_ordered_timeline"] =
            serde_json::json!([{"tick":1},{"tick":2}]);
        let first = scope_digest(&item["input"]["scope"]);
        item["input"]["scope"]["custom_ordered_timeline"].as_array_mut().unwrap().reverse();
        assert_ne!(first, scope_digest(&item["input"]["scope"]));
    }
}
