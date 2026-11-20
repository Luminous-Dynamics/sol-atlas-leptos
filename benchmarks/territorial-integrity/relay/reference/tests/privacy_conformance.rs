use cter_relay_reference::privacy::classify_paired_world_case;
use serde_json::Value;
use std::collections::BTreeSet;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../privacy/cases-v0.1.json"))
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
    assert_eq!(corpus["synthetic_only"], true);
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
            .contains("support_bundle")
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
    assert!(result.canary_hit_count == 0);
    assert!(
        result
            .different_channels
            .contains("object_metadata")
    );
}
