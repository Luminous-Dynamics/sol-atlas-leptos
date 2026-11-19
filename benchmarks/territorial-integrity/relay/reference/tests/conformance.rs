use cter_relay_reference::{evaluate, merge_event_sets};
use serde_json::Value;

#[test]
fn frozen_relay_cases_match_expected_dispositions() {
    let corpus: Value = serde_json::from_str(include_str!("../../cases-v0.1.json"))
        .expect("relay corpus must be valid JSON");
    let cases = corpus["cases"].as_array().expect("cases array");
    assert!(!cases.is_empty(), "the frozen corpus must not be empty");

    for case in cases {
        let case_id = case["case_id"].as_str().expect("case_id");
        let events = case["events"].as_array().expect("events");
        let actual = evaluate(events).to_json();
        assert_eq!(actual, case["expected"], "case {case_id}");
    }
}

#[test]
fn event_set_merge_is_idempotent_commutative_and_associative() {
    let corpus: Value = serde_json::from_str(include_str!("../../cases-v0.1.json"))
        .expect("relay corpus must be valid JSON");
    let cases = corpus["cases"].as_array().expect("cases array");
    let a = cases[0]["events"].as_array().expect("case 0 events");
    let b = cases[1]["events"].as_array().expect("case 1 events");
    let c = cases[2]["events"].as_array().expect("case 2 events");

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
fn replay_order_does_not_change_the_normalized_result() {
    let corpus: Value = serde_json::from_str(include_str!("../../cases-v0.1.json"))
        .expect("relay corpus must be valid JSON");
    let case = &corpus["cases"][3];
    let events = case["events"].as_array().expect("events");
    let mut reversed = events.clone();
    reversed.reverse();
    assert_eq!(evaluate(events).to_json(), evaluate(&reversed).to_json());
}
