use cter_relay_reference::private_coverage::query_input_snapshot_commitment;
use cter_relay_reference::release_binding::{
    evaluate_composed_release_binding, evaluate_composed_release_binding_json,
};
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../egress/composed-release-cases-v0.1.json"
    ))
    .expect("composed release corpus must be valid JSON")
}

fn baseline() -> (Value, Value) {
    let fixture = corpus();
    (
        fixture["baseline"]["disclosure_manifest"].clone(),
        fixture["baseline"]["private_coverage"].clone(),
    )
}

fn apply_mutation(case_id: &str, manifest: &mut Value, coverage: &mut Value) {
    match case_id {
        "COMPOSED-BASELINE-001" => {}
        "COMPOSED-COVERAGE-REFERENCE-DRIFT-001" => {
            coverage["coverage_ref"] = Value::String("urn:fixture:coverage:other".into());
        }
        "COMPOSED-PACKAGE-DIGEST-DRIFT-001" => {
            coverage["package_binding"]["package_commitment"]["value"] =
                Value::String("9".repeat(64));
        }
        "COMPOSED-REQUEST-DRIFT-001" => {
            coverage["query_binding"]["request_commitment"]["value"] =
                Value::String("9".repeat(64));
        }
        "COMPOSED-PRINCIPAL-DRIFT-001" => {
            coverage["query_binding"]["principal_ref"] =
                Value::String("urn:fixture:principal:other".into());
        }
        "COMPOSED-SUBJECT-DRIFT-001" => {
            coverage["query_binding"]["subject_ref"] =
                Value::String("urn:fixture:subject:other".into());
        }
        "COMPOSED-OPERATION-DRIFT-001" => {
            coverage["query_binding"]["operation"] = Value::String("assert_legal_title".into());
        }
        "COMPOSED-PURPOSE-DRIFT-001" => {
            coverage["query_binding"]["purpose_ref"] =
                Value::String("urn:fixture:purpose:other".into());
        }
        "COMPOSED-RECIPIENT-DRIFT-001" => {
            coverage["query_binding"]["recipient_ref"] =
                Value::String("urn:fixture:recipient:other".into());
        }
        "COMPOSED-RESOURCE-DRIFT-001" => {
            coverage["query_binding"]["resource_ref"] =
                Value::String("urn:fixture:resource:other".into());
        }
        "COMPOSED-JURISDICTION-DRIFT-001" => {
            coverage["query_binding"]["jurisdiction_profile_ref"] =
                Value::String("urn:fixture:jurisdiction:other".into());
        }
        "COMPOSED-SNAPSHOT-DRIFT-001" => {
            coverage["query_binding"]["project_snapshot_sha256"] = Value::String("9".repeat(64));
        }
        "COMPOSED-FRONTIER-DRIFT-001" => {
            manifest["authorization_binding"]["evidence_frontier_ref"] =
                Value::String("urn:fixture:frontier:other".into());
        }
        "COMPOSED-DATA-CLASS-DRIFT-001" => {
            coverage["query_binding"]["data_classes"] = serde_json::json!(["restricted"]);
        }
        "COMPOSED-COVERAGE-QUERY-REF-DRIFT-001" => {
            manifest["authorization_binding"]["coverage_query_ref"] =
                Value::String("urn:fixture:query:other".into());
        }
        "COMPOSED-COVERAGE-QUERY-COMMITMENT-DRIFT-001" => {
            manifest["authorization_binding"]["coverage_query_commitment"]["value"] =
                Value::String("9".repeat(64));
        }
        "COMPOSED-COVERAGE-QUERY-PROFILE-DRIFT-001" => {
            coverage["query_binding"]["query_profile_ref"] =
                Value::String("urn:luminous:cter:predicate-json-v0.2".into());
        }
        "COMPOSED-QUERY-PREDICATE-DRIFT-001" => {
            coverage["query_binding"]["predicate"]["args"][1]["value"] = Value::String("US".into());
        }
        "COMPOSED-INPUT-SNAPSHOT-REF-DRIFT-001" => {
            manifest["authorization_binding"]["coverage_input_snapshot_ref"] =
                Value::String("urn:fixture:query-input-snapshot:other".into());
        }
        "COMPOSED-INPUT-SNAPSHOT-COMMITMENT-DRIFT-001" => {
            manifest["authorization_binding"]["coverage_input_snapshot_commitment"]["value"] =
                Value::String("9".repeat(64));
        }
        "COMPOSED-INPUT-SNAPSHOT-ATTRIBUTE-DRIFT-001" => {
            coverage["query_input_snapshot"]["items"][0]["query_input"]["jurisdiction"] =
                Value::String("US".into());
        }
        "COMPOSED-ASSERTED-LEAF-DRIFT-001" => {
            coverage["package_binding"]["terminal_source_leaf_refs"] =
                serde_json::json!(["urn:fixture:source:other"]);
        }
        "COMPOSED-DERIVED-LEAF-DRIFT-001" => {
            let replacement = "urn:fixture:source:other";
            coverage["frontier"]["object_refs"][0] = Value::String(replacement.into());
            coverage["private_partition"][0]["object_ref"] = Value::String(replacement.into());
            coverage["package_binding"]["terminal_source_leaf_refs"] =
                serde_json::json!([replacement]);
        }
        "COMPOSED-MULTI-STAGE-PROVENANCE-001" => {
            let first = manifest["package"]["outputs"][0].clone();
            let first_ref = first["object_ref"]
                .as_str()
                .expect("first output ref")
                .to_owned();
            let first_digest = first["content_digest"].clone();
            let second_ref = "urn:fixture:output:derived-002";

            manifest["package"]["outputs"]
                .as_array_mut()
                .expect("package outputs")
                .push(serde_json::json!({
                    "object_ref": second_ref,
                    "content_digest": {
                        "algorithm": "sha-256",
                        "value": "2222222222222222222222222222222222222222222222222222222222222222"
                    },
                    "byte_length": 96,
                    "media_type": "application/octet-stream",
                    "data_classes": ["community"],
                    "source_refs": [{
                        "object_ref": first_ref,
                        "content_digest": first_digest,
                        "data_classes": ["community"]
                    }],
                    "transformation_refs": ["urn:fixture:transform:derived-002"]
                }));
            manifest["package"]["transformations"]
                .as_array_mut()
                .expect("package transformations")
                .push(serde_json::json!({
                    "transform_ref": "urn:fixture:transform:derived-002",
                    "transform_kind": "format_conversion",
                    "profile_ref": "urn:fixture:transform-profile:format-conversion-v0",
                    "version_commitment": {
                        "algorithm": "sha-256",
                        "value": "3333333333333333333333333333333333333333333333333333333333333333"
                    },
                    "input_object_refs": [first_ref],
                    "output_object_refs": [second_ref]
                }));
        }
        _ => panic!("unknown composed release case {case_id}"),
    }
}

#[test]
fn baseline_composed_release_is_structurally_bound() {
    let (manifest, coverage) = baseline();
    let result = evaluate_composed_release_binding(&manifest, &coverage);
    assert_eq!(
        result.disposition, "valid_composed_structure",
        "{:?}",
        result.reason_codes
    );
    assert!(result.reason_codes.is_empty());
    assert_eq!(
        result.derived_terminal_source_leaf_refs,
        std::collections::BTreeSet::from(["urn:fixture:source:synthetic-001".to_owned()])
    );
}

#[test]
fn frozen_composed_release_corpus_matches_every_expected_disposition() {
    let suite = corpus();
    let cases = suite["cases"].as_array().expect("case array");
    assert_eq!(cases.len(), 24);
    for case in cases {
        let case_id = case["case_id"].as_str().expect("case id");
        let expected = case["expected"].as_str().expect("expected disposition");
        let (mut manifest, mut coverage) = baseline();
        apply_mutation(case_id, &mut manifest, &mut coverage);
        let actual = evaluate_composed_release_binding(&manifest, &coverage);
        assert_eq!(
            actual.disposition, expected,
            "case {case_id}; reasons={:?}",
            actual.reason_codes
        );
    }
}

#[test]
fn composed_release_derives_leaves_through_intermediate_outputs() {
    let (mut manifest, mut coverage) = baseline();
    apply_mutation(
        "COMPOSED-MULTI-STAGE-PROVENANCE-001",
        &mut manifest,
        &mut coverage,
    );
    let result = evaluate_composed_release_binding(&manifest, &coverage);
    assert_eq!(
        result.disposition, "valid_composed_structure",
        "{:?}",
        result.reason_codes
    );
    assert_eq!(
        result.derived_terminal_source_leaf_refs,
        std::collections::BTreeSet::from(["urn:fixture:source:synthetic-001".to_owned()])
    );
}

#[test]
fn composed_raw_json_entrypoint_uses_strict_duplicate_member_parser() {
    let (manifest, coverage) = baseline();
    let manifest_json = serde_json::to_string(&manifest).expect("manifest serializes");
    let coverage_json = serde_json::to_string(&coverage).expect("coverage serializes");
    let result = evaluate_composed_release_binding_json(&manifest_json, &coverage_json)
        .expect("strict parser accepts unique JSON");
    assert_eq!(result.disposition, "valid_composed_structure");

    let duplicate_manifest = manifest_json.replacen(
        "\"manifest_id\":",
        "\"manifest_id\":\"urn:fixture:manifest:duplicate\", \"manifest_id\":",
        1,
    );
    assert!(evaluate_composed_release_binding_json(&duplicate_manifest, &coverage_json).is_err());

    let duplicate_coverage = coverage_json.replacen(
        "\"query_ref\":",
        "\"query_ref\":\"urn:fixture:query:duplicate\", \"query_ref\":",
        1,
    );
    assert!(evaluate_composed_release_binding_json(&manifest_json, &duplicate_coverage).is_err());
}

#[test]
fn composed_query_input_snapshot_commitment_matches_frozen_golden_digest() {
    let (_, coverage) = baseline();
    assert_eq!(
        query_input_snapshot_commitment(&coverage["query_input_snapshot"])
            .expect("composed snapshot canonicalizes"),
        "a8f000ebab94a5ad4479faa5fb12cf2980cc094376d0cc3eb438698804c3d788"
    );
}
