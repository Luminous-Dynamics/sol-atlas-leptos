use cter_relay_reference::private_coverage::{
    evaluate_private_coverage,
    query_input_snapshot_commitment,
};
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../../egress/private-coverage-cases-v0.1.json"))
        .expect("private coverage corpus must be valid JSON")
}

fn baseline() -> Value {
    corpus()["baseline"].clone()
}

#[test]
fn baseline_partition_is_structurally_valid_but_not_a_cryptographic_proof() {
    let result = evaluate_private_coverage(&baseline());
    assert_eq!(result.disposition, "valid_partition_structure");
    assert!(result.reason_codes.is_empty());
}

#[test]
fn private_coverage_suite_size_and_claim_ceiling_are_frozen() {
    let suite = corpus();
    assert_eq!(suite["cases"].as_array().expect("cases").len(), 44);
    assert_eq!(suite["synthetic_only"].as_bool(), Some(true));
    assert!(
        suite["claim_ceiling"]
            .as_str()
            .expect("claim ceiling")
            .contains("no cryptographic proof")
    );
}

#[test]
fn duplicated_partition_disposition_is_denied() {
    let mut input = baseline();
    let duplicate = input["private_partition"][0].clone();
    input["private_partition"]
        .as_array_mut()
        .expect("private partition")
        .push(duplicate);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("partition_object_disposition_duplicated")
    );
}

#[test]
fn omitted_frontier_item_is_incomplete_not_absent() {
    let mut input = baseline();
    input["private_partition"]
        .as_array_mut()
        .expect("private partition")
        .remove(1);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(
        result
            .reason_codes
            .contains("partition_frontier_object_set_mismatch")
    );
}

#[test]
fn withheld_item_cannot_be_relabelled_absent() {
    let mut input = baseline();
    input["private_partition"][1]["disposition"] = Value::String("absent".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("partition_absence_state_forbidden")
    );
}

#[test]
fn unavailable_item_cannot_be_relabelled_absent() {
    let mut input = baseline();
    input["private_partition"][2]["disposition"] = Value::String("absent".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("partition_absence_state_forbidden")
    );
}

#[test]
fn included_frontier_set_must_equal_package_terminal_source_leaves() {
    let mut input = baseline();
    input["private_partition"][1]["disposition"] = Value::String("included".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("package_included_source_set_mismatch")
    );
}

#[test]
fn package_cannot_claim_a_phantom_source_outside_the_frontier() {
    let mut input = baseline();
    input["package_binding"]["terminal_source_leaf_refs"]
        .as_array_mut()
        .expect("terminal source leaves")
        .push(Value::String("urn:fixture:evidence:phantom".into()));
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("package_contains_phantom_frontier_source")
    );
}

#[test]
fn stale_frontier_receipt_projection_returns_unknown() {
    let mut input = baseline();
    input["frontier"]["version"] = Value::String("v2".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "unknown");
    assert!(
        result
            .reason_codes
            .contains("receipt_frontier_version_or_commitment_mismatch")
    );
}

#[test]
fn query_commitment_substitution_is_denied() {
    let mut input = baseline();
    input["query_binding"]["query_commitment"]["value"] =
        Value::String("9".repeat(64));
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("receipt_query_binding_mismatch:query_commitment")
    );
}

#[test]
fn recipient_substitution_is_denied() {
    let mut input = baseline();
    input["query_binding"]["recipient_ref"] =
        Value::String("urn:fixture:recipient:other".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("receipt_query_binding_mismatch:recipient_ref")
    );
}

#[test]
fn unavailable_receipt_currentness_is_unknown() {
    let mut input = baseline();
    input["receipt_assessment"]["resolution_status"] =
        Value::String("unavailable".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "unknown");
    assert!(result.reason_codes.contains("vds_receipt_unavailable"));
}

#[test]
fn revoked_receipt_is_denied() {
    let mut input = baseline();
    input["receipt_assessment"]["resolution_status"] = Value::String("revoked".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("vds_receipt_revoked"));
}

#[test]
fn public_verifier_view_cannot_expose_item_references_or_counts() {
    let mut input = baseline();
    input["public_verifier_view"]["withheld_count"] = Value::Number(1.into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("public_verifier_view_over_discloses")
    );
}

#[test]
fn global_non_inclusion_claim_is_denied() {
    let mut input = baseline();
    input["public_verifier_view"]["non_inclusion_scope"] = Value::String("global".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("non_inclusion_scope_exceeds_committed_frontier")
    );
}

#[test]
fn public_package_commitment_must_match_custodian_package() {
    let mut input = baseline();
    input["public_verifier_view"]["package_commitment"]["value"] =
        Value::String("8".repeat(64));
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("public_commitment_binding_mismatch:package_commitment")
    );
}

#[test]
fn extra_partition_reference_is_not_part_of_the_frontier() {
    let mut input = baseline();
    input["private_partition"]
        .as_array_mut()
        .expect("partition")
        .push(serde_json::json!({
            "object_ref": "urn:fixture:evidence:extra",
            "scope_state": "in_scope",
            "disposition": "withheld"
        }));
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(
        result
            .reason_codes
            .contains("partition_frontier_object_set_mismatch")
    );
}

#[test]
fn unsupported_operation_is_denied() {
    let mut input = baseline();
    input["query_binding"]["operation"] = Value::String("assert_legal_title".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(
        result
            .reason_codes
            .contains("coverage_operation_outside_supported_scope")
    );
}


fn refresh_snapshot_commitments(input: &mut Value) {
    let digest = query_input_snapshot_commitment(&input["query_input_snapshot"])
        .expect("synthetic snapshot must canonicalize");
    let commitment = serde_json::json!({
        "algorithm": "sha-256",
        "value": digest
    });
    input["query_input_snapshot"]["snapshot_commitment"] = commitment.clone();
    input["query_binding"]["input_snapshot_commitment"] = commitment.clone();
    input["receipt_binding"]["input_snapshot_commitment"] = commitment;
}

fn apply_case_mutation(case_id: &str, input: &mut Value) {
    match case_id {
        "PCOV-BASELINE-001" => {}
        "PCOV-DUPLICATE-DISPOSITION-001" | "PCOV-ALIAS-DUPLICATION-001" => {
            let partition = input["private_partition"]
                .as_array_mut()
                .expect("partition array");
            let duplicate = partition[0].clone();
            partition.push(duplicate);
        }
        "PCOV-OMITTED-FRONTIER-ITEM-001" => {
            input["private_partition"]
                .as_array_mut()
                .expect("partition array")
                .remove(1);
        }
        "PCOV-WITHHELD-AS-ABSENT-001" => {
            input["private_partition"][1]["disposition"] = Value::String("absent".into());
        }
        "PCOV-UNAVAILABLE-AS-ABSENT-001" => {
            input["private_partition"][2]["disposition"] = Value::String("absent".into());
        }
        "PCOV-INCLUDED-SET-MISMATCH-001" => {
            input["private_partition"][1]["disposition"] = Value::String("included".into());
        }
        "PCOV-PHANTOM-PACKAGE-SOURCE-001" => {
            input["package_binding"]["terminal_source_leaf_refs"]
                .as_array_mut()
                .expect("package leaves")
                .push(Value::String("urn:fixture:evidence:phantom".into()));
        }
        "PCOV-FRONTIER-VERSION-DRIFT-001"
        | "PCOV-RECEIPT-FRONTIER-VERSION-DRIFT-001" => {
            input["frontier"]["version"] = Value::String("v2".into());
        }
        "PCOV-QUERY-SUBSTITUTION-001" => {
            input["query_binding"]["query_commitment"]["value"] =
                Value::String("9".repeat(64));
        }
        "PCOV-RECIPIENT-SUBSTITUTION-001" => {
            input["query_binding"]["recipient_ref"] =
                Value::String("urn:fixture:recipient:other".into());
        }
        "PCOV-STALE-VDS-RECEIPT-001" => {
            input["receipt_assessment"]["resolution_status"] = Value::String("stale".into());
        }
        "PCOV-RECEIPT-UNAVAILABLE-001" => {
            input["receipt_assessment"]["resolution_status"] =
                Value::String("unavailable".into());
        }
        "PCOV-RECEIPT-REVOKED-001" => {
            input["receipt_assessment"]["resolution_status"] =
                Value::String("revoked".into());
        }
        "PCOV-NONINCLUSION-AS-GLOBAL-ABSENCE-001" => {
            input["public_verifier_view"]["non_inclusion_scope"] =
                Value::String("global".into());
        }
        "PCOV-PUBLIC-COUNT-LEAK-001" | "PCOV-PUBLIC-WITHHELD-COUNT-LEAK-001" => {
            input["public_verifier_view"]["withheld_count"] = Value::Number(1.into());
        }
        "PCOV-LOW-ENTROPY-HASH-LEAK-001" => {
            input["public_verifier_view"]["low_entropy_hash"] =
                Value::String("0000".into());
        }
        "PCOV-LATE-SOURCE-EVENT-001" => {
            input["receipt_assessment"]["resolution_status"] =
                Value::String("unknown".into());
        }
        "PCOV-RESTORE-SEMANTICS-LOSS-001" => {
            input.as_object_mut().expect("coverage object").remove("query_binding");
        }
        "PCOV-EXTRA-PARTITION-OBJECT-001" => {
            input["private_partition"].as_array_mut().expect("partition").push(
                serde_json::json!({
                    "object_ref": "urn:fixture:evidence:extra",
                    "scope_state": "in_scope",
                    "disposition": "withheld"
                }),
            );
        }
        "PCOV-PUBLIC-INVENTORY-REF-LEAK-001" => {
            input["public_verifier_view"]["object_ref"] =
                Value::String("urn:fixture:evidence:item-b".into());
        }
        "PCOV-OPERATION-SUBSTITUTION-001" => {
            input["query_binding"]["operation"] =
                Value::String("assert_legal_title".into());
        }
        "PCOV-PURPOSE-SUBSTITUTION-001" => {
            input["query_binding"]["purpose_ref"] =
                Value::String("urn:fixture:purpose:other".into());
        }
        "PCOV-JURISDICTION-SUBSTITUTION-001" => {
            input["query_binding"]["jurisdiction_profile_ref"] =
                Value::String("urn:fixture:jurisdiction:other".into());
        }
        "PCOV-PUBLIC-PACKAGE-COMMITMENT-DRIFT-001" => {
            input["public_verifier_view"]["package_commitment"]["value"] =
                Value::String("8".repeat(64));
        }
        "PCOV-FRONTIER-ITEM-DUPLICATE-001" => {
            let first = input["frontier"]["object_refs"][0].clone();
            input["frontier"]["object_refs"].as_array_mut().expect("frontier refs").push(first);
        }
        "PCOV-RECEIPT-PROJECTION-DRIFT-001" => {
            input["receipt_binding"]["recipient_ref"] =
                Value::String("urn:fixture:recipient:other".into());
        }
        "PCOV-PROOF-VIEW-EXTRA-REFERENCE-001" => {
            input["public_verifier_view"]["object_refs"] =
                serde_json::json!(["urn:fixture:evidence:item-b"]);
        }
        "PCOV-RECEIPT-SUBJECT-DRIFT-001" => {
            input["receipt_binding"]["subject_ref"] =
                Value::String("urn:fixture:subject:other".into());
        }
        "PCOV-RECEIPT-DATA-CLASS-DRIFT-001" => {
            input["receipt_binding"]["data_classes"] = serde_json::json!(["restricted"]);
        }
        "PCOV-PREDICATE-SCOPE-MISMATCH-001" => {
            input["private_partition"][3]["scope_state"] = Value::String("in_scope".into());
            input["private_partition"][3]["disposition"] = Value::String("withheld".into());
        }
        "PCOV-UNKNOWN-PREDICATE-AS-IN-SCOPE-001" => {
            input["private_partition"][4]["scope_state"] = Value::String("in_scope".into());
            input["private_partition"][4]["disposition"] = Value::String("withheld".into());
        }
        "PCOV-TRUE-PREDICATE-AS-OUT-OF-SCOPE-001" => {
            input["private_partition"][1]["scope_state"] = Value::String("out_of_scope".into());
            input["private_partition"][1]["disposition"] = Value::String("excluded".into());
        }
        "PCOV-PREDICATE-UNKNOWN-OPERATOR-001" => {
            input["query_binding"]["predicate"]["op"] = Value::String("matches".into());
        }
        "PCOV-PREDICATE-MALFORMED-PATH-001" => {
            input["query_binding"]["predicate"]["args"][0]["path"] =
                Value::String("dataset".into());
        }
        "PCOV-PREDICATE-SUBSTITUTION-001" => {
            input["query_binding"]["predicate"]["args"][1]["value"] =
                Value::String("US".into());
        }
        "PCOV-SNAPSHOT-ITEM-OMISSION-001" => {
            input["query_input_snapshot"]["items"]
                .as_array_mut()
                .expect("snapshot items")
                .remove(4);
            refresh_snapshot_commitments(input);
        }
        "PCOV-SNAPSHOT-SOURCE-OUTSIDE-FRONTIER-001" => {
            input["query_input_snapshot"]["items"][0]["attribute_source_bindings"][0]
                ["source_object_ref"] =
                Value::String("urn:fixture:evidence:outside".into());
            refresh_snapshot_commitments(input);
        }
        "PCOV-SNAPSHOT-ATTRIBUTE-BINDING-MISSING-001" => {
            let bindings = input["query_input_snapshot"]["items"][0]
                ["attribute_source_bindings"]
                .as_array_mut()
                .expect("attribute bindings");
            bindings.retain(|binding| binding["path"] != "/jurisdiction");
            refresh_snapshot_commitments(input);
        }
        "PCOV-SNAPSHOT-ATTRIBUTE-DRIFT-001" => {
            input["query_input_snapshot"]["items"][0]["query_input"]["jurisdiction"] =
                Value::String("US".into());
        }
        "PCOV-SNAPSHOT-RECEIPT-REF-DRIFT-001" => {
            input["receipt_binding"]["input_snapshot_ref"] =
                Value::String("urn:fixture:query-input-snapshot:other".into());
        }
        "PCOV-SNAPSHOT-UNAVAILABLE-AS-TRUE-001" => {
            input["query_input_snapshot"]["items"][0]["availability_state"] =
                Value::String("unavailable".into());
            input["query_input_snapshot"]["items"][0]
                .as_object_mut()
                .expect("snapshot item")
                .remove("query_input");
            input["query_input_snapshot"]["items"][0]["attribute_source_bindings"] =
                serde_json::json!([]);
            refresh_snapshot_commitments(input);
        }
        _ => panic!("unknown private coverage case {case_id}"),
    }
}

#[test]
fn frozen_private_coverage_corpus_matches_every_expected_disposition() {
    let suite = corpus();
    let cases = suite["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 44);
    for case in cases {
        let case_id = case["case_id"].as_str().expect("case id");
        let expected = case["expected"].as_str().expect("expected disposition");
        let mut input = suite["baseline"].clone();
        apply_case_mutation(case_id, &mut input);
        let actual = evaluate_private_coverage(&input);
        assert_eq!(
            actual.disposition,
            expected,
            "case {case_id}; reasons={:?}",
            actual.reason_codes
        );
    }
}

#[test]
fn receipt_projection_must_bind_the_exact_package_commitment() {
    let mut input = baseline();
    input["receipt_binding"]["package_commitment"]["value"] =
        Value::String("7".repeat(64));
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("receipt_package_commitment_mismatch"));
}


#[test]
fn receipt_projection_must_bind_the_exact_subject() {
    let mut input = baseline();
    input["receipt_binding"]["subject_ref"] =
        Value::String("urn:fixture:subject:other".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("receipt_query_binding_mismatch:subject_ref"));
}

#[test]
fn receipt_projection_must_bind_the_exact_data_class_set() {
    let mut input = baseline();
    input["receipt_binding"]["data_classes"] = serde_json::json!(["restricted"]);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("receipt_query_binding_mismatch:data_classes"));
}


#[test]
fn partition_scope_must_match_query_predicate_result() {
    let mut input = baseline();
    input["private_partition"][3]["scope_state"] = Value::String("in_scope".into());
    input["private_partition"][3]["disposition"] = Value::String("withheld".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("partition_scope_does_not_match_query_predicate"));
}

#[test]
fn missing_query_attribute_remains_unknown_not_in_scope() {
    let mut input = baseline();
    input["private_partition"][4]["scope_state"] = Value::String("in_scope".into());
    input["private_partition"][4]["disposition"] = Value::String("withheld".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("partition_scope_does_not_match_query_predicate"));
}

#[test]
fn unsupported_predicate_operator_is_incomplete() {
    let mut input = baseline();
    input["query_binding"]["predicate"]["op"] = Value::String("matches".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(result.reason_codes.contains("query_predicate_invalid:predicate_operator_unsupported"));
}

#[test]
fn malformed_predicate_path_is_incomplete() {
    let mut input = baseline();
    input["query_binding"]["predicate"]["args"][0]["path"] =
        Value::String("dataset".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(result.reason_codes.contains("query_predicate_invalid:predicate_path_must_be_single_member_pointer"));
}


#[test]
fn predicate_substitution_without_commitment_refresh_is_denied() {
    let mut input = baseline();
    input["query_binding"]["predicate"]["args"][1]["value"] =
        Value::String("US".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("query_predicate_commitment_mismatch"));
}

#[test]
fn baseline_query_predicate_commitment_is_recomputed() {
    let result = evaluate_private_coverage(&baseline());
    assert_eq!(result.disposition, "valid_partition_structure");
    assert!(!result.reason_codes.contains("query_predicate_commitment_mismatch"));
}


#[test]
fn query_input_snapshot_covers_the_exact_frontier_set() {
    let mut input = baseline();
    input["query_input_snapshot"]["items"]
        .as_array_mut()
        .expect("snapshot items")
        .remove(4);
    refresh_snapshot_commitments(&mut input);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(result.reason_codes.contains("query_input_snapshot_frontier_object_set_mismatch"));
}

#[test]
fn query_input_snapshot_source_must_belong_to_the_frontier() {
    let mut input = baseline();
    input["query_input_snapshot"]["items"][0]["attribute_source_bindings"][0]
        ["source_object_ref"] = Value::String("urn:fixture:evidence:outside".into());
    refresh_snapshot_commitments(&mut input);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("query_input_attribute_source_outside_frontier"));
}

#[test]
fn every_present_predicate_attribute_requires_a_source_binding() {
    let mut input = baseline();
    let bindings = input["query_input_snapshot"]["items"][0]["attribute_source_bindings"]
        .as_array_mut()
        .expect("attribute bindings");
    bindings.retain(|binding| binding["path"] != "/jurisdiction");
    refresh_snapshot_commitments(&mut input);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "incomplete");
    assert!(result.reason_codes.contains("query_input_attribute_source_binding_missing:/jurisdiction"));
}

#[test]
fn attribute_drift_without_snapshot_refresh_is_denied() {
    let mut input = baseline();
    input["query_input_snapshot"]["items"][0]["query_input"]["jurisdiction"] =
        Value::String("US".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("query_input_snapshot_commitment_mismatch"));
}

#[test]
fn receipt_projection_must_bind_the_exact_input_snapshot_reference() {
    let mut input = baseline();
    input["receipt_binding"]["input_snapshot_ref"] =
        Value::String("urn:fixture:query-input-snapshot:other".into());
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("receipt_query_binding_mismatch:input_snapshot_ref"));
}

#[test]
fn unavailable_query_input_cannot_be_promoted_to_a_true_match() {
    let mut input = baseline();
    input["query_input_snapshot"]["items"][0]["availability_state"] =
        Value::String("unavailable".into());
    input["query_input_snapshot"]["items"][0]
        .as_object_mut()
        .expect("snapshot item")
        .remove("query_input");
    input["query_input_snapshot"]["items"][0]["attribute_source_bindings"] =
        serde_json::json!([]);
    refresh_snapshot_commitments(&mut input);
    let result = evaluate_private_coverage(&input);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("partition_scope_does_not_match_query_predicate"));
}


#[test]
fn query_input_snapshot_commitment_matches_frozen_golden_digest() {
    let input = baseline();
    let snapshot = &input["query_input_snapshot"];
    assert_eq!(
        query_input_snapshot_commitment(snapshot).expect("snapshot canonicalizes"),
        "7eda4c8715c60b17866a5e95675760da6728a04d2066e1de7615fd4a4326cfb8"
    );
}

#[test]
fn query_input_snapshot_commitment_is_invariant_to_item_and_binding_order() {
    let input = baseline();
    let expected = query_input_snapshot_commitment(&input["query_input_snapshot"])
        .expect("snapshot canonicalizes");
    let mut reordered = input["query_input_snapshot"].clone();
    reordered["items"].as_array_mut().expect("items").reverse();
    for item in reordered["items"].as_array_mut().expect("items") {
        item["attribute_source_bindings"]
            .as_array_mut()
            .expect("source bindings")
            .reverse();
    }
    assert_eq!(
        query_input_snapshot_commitment(&reordered).expect("reordered snapshot canonicalizes"),
        expected
    );
}

#[test]
fn query_input_snapshot_commitment_changes_when_source_attribute_changes() {
    let input = baseline();
    let expected = query_input_snapshot_commitment(&input["query_input_snapshot"])
        .expect("snapshot canonicalizes");
    let mut changed = input["query_input_snapshot"].clone();
    changed["items"][0]["query_input"]["jurisdiction"] = Value::String("US".into());
    assert_ne!(
        query_input_snapshot_commitment(&changed).expect("changed snapshot canonicalizes"),
        expected
    );
}

#[test]
fn query_input_snapshot_commitment_rejects_unsupported_profile() {
    let input = baseline();
    let mut changed = input["query_input_snapshot"].clone();
    changed["profile_ref"] = Value::String("urn:luminous:cter:query-input-json-v0.2".into());
    assert_eq!(
        query_input_snapshot_commitment(&changed),
        Err("query_input_snapshot_profile_unsupported")
    );
}
