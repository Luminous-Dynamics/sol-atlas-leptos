use cter_relay_reference::privacy::{classify_paired_world_case, evaluate_channel_inventory};
use serde_json::Value;
use std::collections::BTreeSet;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../../privacy/cases-v0.1.json"))
        .expect("privacy fixture corpus must be valid JSON")
}

fn declared_channels(corpus: &Value) -> BTreeSet<String> {
    corpus["declared_channels"]
        .as_array()
        .expect("declared channel array")
        .iter()
        .map(|value| value.as_str().expect("channel id").to_owned())
        .collect()
}

#[test]
fn paired_world_corpus_size_and_synthetic_ceiling_are_frozen() {
    let corpus = corpus();
    assert_eq!(corpus["cases"].as_array().expect("cases").len(), 24);
    assert!(
        corpus["synthetic_only"]
            .as_bool()
            .expect("synthetic-only marker")
    );
    assert_eq!(
        corpus["claim_ceiling"],
        "fixture-level observable transcript equality and canary detection only; no production privacy guarantee or universal non-inference proof"
    );
}

#[test]
fn every_paired_world_fixture_matches_its_expected_disposition() {
    let corpus = corpus();
    let declared = declared_channels(&corpus);
    for case in corpus["cases"].as_array().expect("cases") {
        let id = case["case_id"].as_str().expect("case id");
        let expected = case["expected"].as_str().expect("expected disposition");
        let actual = classify_paired_world_case(case, &declared);
        assert_eq!(actual.disposition, expected, "case {id}; {actual:?}");
    }
}

#[test]
fn undeclared_observable_channel_makes_inventory_incomplete() {
    let corpus = corpus();
    let declared = declared_channels(&corpus);
    let case = corpus["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["case_id"] == "PRIVACY-UNDECLARED-CHANNEL-001")
        .expect("undeclared channel case");
    let result = classify_paired_world_case(case, &declared);
    assert_eq!(result.disposition, "inventory_incomplete");
    assert!(
        result
            .undeclared_channels
            .contains("untracked_plugin_export")
    );
}

#[test]
fn protected_world_pairs_are_exactly_equal_for_non_disclosure_controls() {
    let corpus = corpus();
    let declared = declared_channels(&corpus);
    for case in corpus["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .filter(|case| case["expected"] == "non_disclosure")
    {
        let result = classify_paired_world_case(case, &declared);
        assert_eq!(result.disposition, "non_disclosure", "{}", case["case_id"]);
        assert!(result.different_channels.is_empty());
        assert_eq!(result.canary_hit_count, 0);
    }
}

#[test]
fn explicit_canary_in_any_observable_channel_is_a_leak() {
    let corpus = corpus();
    let declared = declared_channels(&corpus);
    let case = corpus["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["case_id"] == "PRIVACY-AI-PROMPT-LEAK-001")
        .expect("AI prompt canary case");
    let result = classify_paired_world_case(case, &declared);
    assert_eq!(result.disposition, "leak_detected");
    assert!(result.canary_hit_count > 0);
}

#[test]
fn observable_difference_is_a_leak_even_without_canary_hit() {
    let corpus = corpus();
    let declared = declared_channels(&corpus);
    let case = corpus["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["case_id"] == "PRIVACY-LOW-ENTROPY-HASH-001")
        .expect("digest difference case");
    let result = classify_paired_world_case(case, &declared);
    assert_eq!(result.disposition, "leak_detected");
    assert_eq!(result.canary_hit_count, 0);
    assert!(result.different_channels.contains("object_metadata"));
}

fn inventory_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../privacy/channel-inventory-synthetic-v0.1.json"
    ))
    .expect("synthetic inventory fixture must be valid JSON")
}

#[test]
fn partial_channel_inventory_never_claims_complete_coverage() {
    let inventory = inventory_fixture();
    let result = evaluate_channel_inventory(&inventory);
    assert_eq!(result.disposition, "inventory_incomplete");
    assert!(
        result
            .reason_codes
            .contains("inventory_coverage_not_complete")
    );
    assert_eq!(result.declared_channel_refs.len(), 3);
}

#[test]
fn complete_claim_requires_an_independent_coverage_evidence_reference() {
    let mut inventory = inventory_fixture();
    inventory["coverage_assessment"] = Value::String("complete_within_declared_boundary".into());
    let without_evidence = evaluate_channel_inventory(&inventory);
    assert_eq!(without_evidence.disposition, "inventory_incomplete");

    inventory["independent_coverage_evidence_ref"] =
        Value::String("urn:fixture:coverage-evidence:independent-001".into());
    let with_evidence = evaluate_channel_inventory(&inventory);
    assert_eq!(with_evidence.disposition, "declared_surface_bound");
    assert!(with_evidence.reason_codes.is_empty());
}

#[test]
fn duplicate_channel_reference_is_not_a_valid_inventory() {
    let mut inventory = inventory_fixture();
    let duplicate = inventory["channels"][0].clone();
    inventory["channels"]
        .as_array_mut()
        .expect("channels array")
        .push(duplicate);
    let result = evaluate_channel_inventory(&inventory);
    assert_eq!(result.disposition, "inventory_incomplete");
    assert!(result.reason_codes.contains("channel_ref_duplicated"));
}
