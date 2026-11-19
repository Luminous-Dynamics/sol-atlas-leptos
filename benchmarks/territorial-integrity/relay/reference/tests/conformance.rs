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


fn synthetic_delivery(
    id: usize,
    stream_id: &str,
    sequence: u64,
    previous_hash: Option<String>,
    projection_key: &str,
) -> (Value, String) {
    let event_hash = format!("{:064x}", id + 1);
    let payload_hash = format!("{:064x}", id + 10_001);
    let delivery = serde_json::json!({
        "event": {
            "event_id": format!("event-{id}"),
            "stream_id": stream_id,
            "sequence": sequence,
            "previous_hash": previous_hash,
            "event_hash": event_hash,
            "actor": format!("actor-{stream_id}")
        },
        "assessment": {
            "assessment_id": format!("assessment-{id}"),
            "verifier_id": "synthetic-verifier",
            "policy_id": "synthetic-key-policy-v0",
            "assessed_at": "2026-11-19T00:00:00Z",
            "key_status_snapshot_ref": "urn:synthetic:key-status:snapshot:1",
            "event_time_basis": "trusted_append_receipt",
            "status": "verified"
        },
        "context": {
            "projection_key": projection_key,
            "access_class": "restricted"
        },
        "payload": {
            "blob_ref": format!("cas://synthetic/{id}"),
            "media_type": "application/vnd.mycelix.signed-scientific-event+json",
            "blob_sha256": payload_hash
        }
    });
    (delivery, event_hash)
}

#[test]
fn reversed_ten_thousand_event_chain_uses_predecessor_propagation() {
    let mut events = Vec::with_capacity(10_000);
    let mut previous = None;
    for id in 0..10_000 {
        let (event, hash) = synthetic_delivery(id, "one-stream", id as u64, previous, "claim:one");
        previous = Some(hash);
        events.push(event);
    }
    events.reverse();
    let actual = evaluate(&events);
    assert_eq!(actual.accepted_events.len(), 10_000);
    assert!(actual.missing_parent_events.is_empty());
    assert!(actual.blocked_parent_events.is_empty());
    assert!(actual.conflict_groups.is_empty());
}

#[test]
fn high_contention_projection_is_one_compact_group() {
    let mut events = Vec::with_capacity(1_000);
    for id in 0..1_000 {
        let stream_id = format!("stream-{id}");
        let (event, _) = synthetic_delivery(id, &stream_id, 0, None, "shared-claim");
        events.push(event);
    }
    let actual = evaluate(&events);
    assert_eq!(actual.accepted_events.len(), 1_000);
    assert_eq!(actual.conflict_groups.len(), 1);
    let group = actual.conflict_groups.get("shared-claim").expect("one group");
    assert_eq!(group.len(), 1_000);
}
