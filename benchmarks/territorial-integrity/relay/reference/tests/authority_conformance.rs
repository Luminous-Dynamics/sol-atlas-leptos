use cter_relay_reference::authority::evaluate_authority_use;
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../authority-cases-v0.1.json"))
        .expect("authority corpus must be valid JSON")
}

#[test]
fn authority_consumer_corpus_matches_every_expected_disposition() {
    let cases = corpus();
    let cases = cases["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 35, "authority corpus size is part of the release contract");
    for case in cases {
        let case_id = case["case_id"].as_str().expect("case id");
        let expected = case["expected"].as_str().expect("expected disposition");
        let actual = evaluate_authority_use(&case["input"]);
        assert_eq!(actual.disposition, expected, "case {case_id}; reasons={:?}", actual.reason_codes);
    }
}

#[test]
fn cter_cannot_assert_or_create_legal_title() {
    let cases = corpus();
    let case = cases["cases"].as_array().expect("cases")
        .iter().find(|case| case["case_id"] == "AUTH-CTER-CANNOT-ASSERT-LEGAL-TITLE-001")
        .expect("legal-effect negative control");
    let result = evaluate_authority_use(&case["input"]);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("legal_effect_outside_cter"));
}

#[test]
fn recorded_dissent_is_retained_without_becoming_an_automatic_veto() {
    let cases = corpus();
    let case = cases["cases"].as_array().expect("cases")
        .iter().find(|case| case["case_id"] == "AUTH-RECORDED-DISSENT-NOT-AUTO-VETO-001")
        .expect("recorded dissent case");
    let result = evaluate_authority_use(&case["input"]);
    assert_eq!(result.disposition, "eligible");
    assert!(result.reason_codes.is_empty());
}

#[test]
fn unresolved_material_dissent_and_open_challenge_require_review() {
    let cases = corpus();
    for id in ["AUTH-MATERIAL-DISSENT-UNRESOLVED-001", "AUTH-OPEN-CHALLENGE-001"] {
        let case = cases["cases"].as_array().expect("cases")
            .iter().find(|case| case["case_id"] == id).expect("review case");
        assert_eq!(evaluate_authority_use(&case["input"]).disposition, "review", "case {id}");
    }
}

#[test]
fn collection_consent_does_not_authorize_publication() {
    let cases = corpus();
    let case = cases["cases"].as_array().expect("cases")
        .iter().find(|case| case["case_id"] == "AUTH-MEASUREMENT-CONSENT-NOT-PUBLICATION-001")
        .expect("publication separation case");
    assert_ne!(evaluate_authority_use(&case["input"]).disposition, "eligible");
}


#[test]
fn authority_scope_and_status_mutations_never_remain_eligible() {
    let cases = corpus();
    let base = cases["cases"].as_array().expect("cases")
        .iter().find(|case| case["case_id"] == "AUTH-ALLOW-COMMUNITY-COLLECT-001")
        .expect("positive baseline")["input"].clone();

    let mutations: Vec<(&str, fn(&mut Value))> = vec![
        ("receipt type", |v| v["decisions"][0]["receipt_type"] = Value::String("TransportReceipt".into())),
        ("principal", |v| v["decisions"][0]["principal_ref"] = Value::String("did:example:other".into())),
        ("subject", |v| v["decisions"][0]["subject_ref"] = Value::String("participant:other".into())),
        ("operation", |v| v["decisions"][0]["operation"] = Value::String("publish_observation".into())),
        ("resource", |v| v["decisions"][0]["resource_ref"] = Value::String("urn:site:other".into())),
        ("snapshot", |v| v["decisions"][0]["project_snapshot_sha256"] = Value::String("b".repeat(64))),
        ("purpose", |v| v["decisions"][0]["purpose_ref"] = Value::String("urn:purpose:other".into())),
        ("recipient", |v| v["decisions"][0]["recipient_ref"] = Value::String("urn:recipient:other".into())),
        ("jurisdiction", |v| v["decisions"][0]["jurisdiction_profile_ref"] = Value::String("urn:jurisdiction:other".into())),
        ("data class", |v| v["decisions"][0]["authorized_data_classes"] = serde_json::json!(["community", "culturally_protected"])),
        ("signature", |v| v["decisions"][0]["signature_status"] = Value::String("invalid".into())),
        ("issuer authority", |v| v["decisions"][0]["authority_status"] = Value::String("unresolved".into())),
        ("revocation", |v| v["decisions"][0]["revocation_status"] = Value::String("unknown".into())),
        ("currentness", |v| v["decisions"][0]["currentness_status"] = Value::String("stale".into())),
        ("offline lease", |v| v["decisions"][0]["offline_lease_expires_at"] = Value::String("2026-11-20T02:59:59Z".into())),
        ("explicit denial", |v| v["decisions"][0]["disposition"] = Value::String("deny".into())),
        ("remove site access", |v| v["decisions"].as_array_mut().expect("decisions").retain(|r| r["decision_domain"] != "site_access")),
        ("remove community mandate", |v| v["decisions"].as_array_mut().expect("decisions").retain(|r| r["decision_domain"] != "community_mandate")),
        ("contested representation", |v| v["governance_context"]["representation_status"] = Value::String("contested".into())),
        ("representation gaps", |v| v["governance_context"]["coverage_status"] = Value::String("known_gaps".into())),
        ("unresolved dissent", |v| v["governance_context"]["dissent_status"] = Value::String("material_unresolved".into())),
        ("open challenge", |v| v["governance_context"]["challenge_status"] = Value::String("open".into())),
    ];

    for (name, mutate) in mutations {
        let mut input = base.clone();
        mutate(&mut input);
        let result = evaluate_authority_use(&input);
        assert_ne!(result.disposition, "eligible", "mutation {name} bypassed the gate");
    }
}

#[test]
fn exact_duplicate_receipt_replay_is_idempotent_but_identity_divergence_is_denied() {
    let cases = corpus();
    let base = cases["cases"].as_array().expect("cases")
        .iter().find(|case| case["case_id"] == "AUTH-ALLOW-COMMUNITY-COLLECT-001")
        .expect("positive baseline")["input"].clone();

    let mut exact_replay = base.clone();
    let duplicate = exact_replay["decisions"][0].clone();
    exact_replay["decisions"].as_array_mut().expect("decisions").push(duplicate);
    assert_eq!(evaluate_authority_use(&exact_replay).disposition, "eligible");

    let mut divergent = base;
    let mut changed = divergent["decisions"][0].clone();
    changed["disposition"] = Value::String("deny".into());
    divergent["decisions"].as_array_mut().expect("decisions").push(changed);
    let result = evaluate_authority_use(&divergent);
    assert_eq!(result.disposition, "deny");
    assert!(result.reason_codes.contains("authorization_receipt_identity_conflict"));
}
