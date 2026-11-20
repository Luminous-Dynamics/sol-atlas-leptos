use cter_relay_reference::egress::evaluate_disclosure_manifest;
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../egress/cases-v0.1.json"))
        .expect("egress corpus must be valid JSON")
}

fn baseline() -> Value {
    corpus()["baseline_manifest"].clone()
}

#[test]
fn baseline_manifest_has_structurally_closed_request_destination_and_lineage() {
    let result = evaluate_disclosure_manifest(&baseline());
    assert_eq!(result.disposition, "valid_structure");
    assert!(result.reason_codes.is_empty());
}

#[test]
fn adversarial_suite_is_frozen_and_explicitly_unexecuted() {
    let suite = corpus();
    assert_eq!(suite["cases"].as_array().expect("case array").len(), 31);
    assert_eq!(
        suite["execution_status"],
        "reference_validator_and_independent_oracle_implemented_not_executed"
    );
}

#[test]
fn recipient_cannot_drift_between_authorization_and_destination() {
    let mut input = baseline();
    input["destination_binding"]["recipient_ref"] =
        Value::String("urn:fixture:recipient:other".into());
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("recipient_binding_mismatch"));
}

#[test]
fn output_data_classes_cannot_exceed_the_authorized_set() {
    let mut input = baseline();
    input["package"]["outputs"][0]["data_classes"] =
        serde_json::json!(["community", "restricted"]);
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("output_data_class_outside_authorized_scope")
    );
}

#[test]
fn malformed_payload_digest_is_incomplete_not_verified() {
    let mut input = baseline();
    input["package"]["outputs"][0]["content_digest"]["algorithm"] =
        Value::String("blake3".into());
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(result.reason_codes.contains("output_content_digest_malformed"));
}

#[test]
fn missing_source_reference_breaks_lineage_closure() {
    let mut input = baseline();
    input["package"]["outputs"][0]["source_refs"] = serde_json::json!([]);
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(result.reason_codes.contains("output_source_lineage_empty"));
}

#[test]
fn missing_transformation_reference_breaks_lineage_closure() {
    let mut input = baseline();
    input["package"]["outputs"][0]["transformation_refs"] =
        serde_json::json!(["urn:fixture:transform:missing"]);
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(
        result
            .reason_codes
            .contains("transformation_lineage_incomplete")
    );
}

#[test]
fn transformation_dependency_cycles_are_denied() {
    let mut input = baseline();
    let output_ref = input["package"]["outputs"][0]["object_ref"]
        .as_str()
        .expect("output ref")
        .to_owned();
    input["package"]["transformations"][0]["input_object_refs"] =
        serde_json::json!([output_ref]);
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("transformation_graph_cycle"));
}

#[test]
fn duplicate_output_identity_is_denied() {
    let mut input = baseline();
    let output = input["package"]["outputs"][0].clone();
    input["package"]["outputs"]
        .as_array_mut()
        .expect("outputs array")
        .push(output);
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("output_object_identity_duplicated")
    );
}


#[test]
fn classification_change_requires_external_authority_review() {
    let mut input = baseline();
    input["authorization_binding"]["data_classes"] =
        serde_json::json!(["community", "public"]);
    input["package"]["outputs"][0]["data_classes"] = serde_json::json!(["public"]);
    input["package"]["transformations"][0]["transform_kind"] =
        Value::String("generalization".into());
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "review");
    assert!(
        result
            .reason_codes
            .contains("classification_change_authority_reference_missing")
    );
}

#[test]
fn a_classification_authority_reference_is_not_self_authenticating() {
    let mut input = baseline();
    input["authorization_binding"]["data_classes"] =
        serde_json::json!(["community", "public"]);
    input["package"]["outputs"][0]["data_classes"] = serde_json::json!(["public"]);
    input["package"]["transformations"][0]["transform_kind"] =
        Value::String("generalization".into());
    input["package"]["transformations"][0]["classification_change_authority_ref"] =
        Value::String("urn:fixture:authority:classification-change".into());
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "unknown");
    assert!(
        result
            .reason_codes
            .contains("classification_change_authority_resolution_unavailable")
    );
}

#[test]
fn identity_copy_requires_source_and_output_digest_commitments_to_match() {
    let mut input = baseline();
    input["package"]["outputs"][0]["content_digest"]["value"] =
        Value::String("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into());
    let result = evaluate_disclosure_manifest(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("identity_copy_digest_mismatch"));
}
