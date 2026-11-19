use cter_relay_reference::{evaluate, merge_event_sets};
use serde_json::Value;

#[test]
fn frozen_relay_cases_match_expected_dispositions() {
    let corpus: Value = serde_json::from_str(include_str!("../../cases-v0.1.json"))
        .expect("relay corpus must be valid JSON");
    let cases = corpus["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 28, "frozen corpus size is part of the release contract");

    for case in cases {
        let case_id = case["case_id"].as_str().expect("case_id");
        let deliveries = case["deliveries"].as_array().expect("deliveries");
        let actual = evaluate(deliveries).to_json();
        assert_eq!(actual, case["expected"], "case {case_id}");
    }
}

#[test]
fn delivery_set_merge_is_idempotent_commutative_and_associative() {
    let corpus: Value = serde_json::from_str(include_str!("../../cases-v0.1.json"))
        .expect("relay corpus must be valid JSON");
    let cases = corpus["cases"].as_array().expect("cases array");
    let a = cases[0]["deliveries"].as_array().expect("case 0 deliveries");
    let b = cases[1]["deliveries"].as_array().expect("case 1 deliveries");
    let c = cases[2]["deliveries"].as_array().expect("case 2 deliveries");

    let aa = merge_event_sets(a, a);
    let ab = merge_event_sets(a, b);
    let ba = merge_event_sets(b, a);
    let left = merge_event_sets(&ab, c);
    let bc = merge_event_sets(b, c);
    let right = merge_event_sets(a, &bc);

    assert_eq!(aa, merge_event_sets(a, &[]));
    assert_eq!(ab, ba);
    assert_eq!(left, right);
}

#[test]
fn delivery_reordering_does_not_change_the_normalized_result() {
    let corpus: Value = serde_json::from_str(include_str!("../../cases-v0.1.json"))
        .expect("relay corpus must be valid JSON");
    for case in corpus["cases"].as_array().expect("cases array") {
        let deliveries = case["deliveries"].as_array().expect("deliveries");
        let mut reversed = deliveries.clone();
        reversed.reverse();
        assert_eq!(evaluate(deliveries).to_json(), evaluate(&reversed).to_json(), "{}", case["case_id"]);
    }
}
