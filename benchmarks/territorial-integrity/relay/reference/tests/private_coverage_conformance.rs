use cter_relay_reference::private_coverage::evaluate_private_coverage;
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../egress/private-coverage-cases-v0.1.json"))
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
    assert_eq!(suite["cases"].as_array().expect("cases").len(), 30);
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


fn apply_case_mutation(case_id: &str, input: &mut Value) {
    let partition = input["private_partition"]
        .as_array_mut()
        .expect("partition array");
    match case_id {
        "PCOV-BASELINE-001" => {}
        "PCOV-DUPLICATE-DISPOSITION-001" | "PCOV-ALIAS-DUPLICATION-001" => {
            partition.push(partition[0].clone());
        }
        "PCOV-OMITTED-FRONTIER-ITEM-001" => {
            partition.remove(1);
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
        _ => panic!("unknown private coverage case {case_id}"),
    }
}

#[test]
fn frozen_private_coverage_corpus_matches_every_expected_disposition() {
    let suite = corpus();
    let cases = suite["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 30);
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
