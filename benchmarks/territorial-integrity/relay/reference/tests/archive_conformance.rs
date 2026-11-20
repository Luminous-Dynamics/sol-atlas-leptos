use cter_relay_reference::archive::evaluate_archive;
use serde_json::Value;

#[test]
fn frozen_archive_cases_match_expected_dispositions() {
    let corpus: Value = serde_json::from_str(include_str!("../../archive-cases-v0.1.json"))
        .expect("archive corpus must be valid JSON");
    let cases = corpus["cases"].as_array().expect("archive cases");
    assert_eq!(cases.len(), 13, "archive corpus size is part of the release contract");

    for case in cases {
        let case_id = case["case_id"].as_str().expect("case id");
        let actual = evaluate_archive(&case["manifest"]);
        assert_eq!(actual.disposition, case["expected"].as_str().expect("expected"), "case {case_id}");
    }
}

#[test]
fn destination_clearance_is_not_inferred_from_archive_ownership() {
    let corpus: Value = serde_json::from_str(include_str!("../../archive-cases-v0.1.json"))
        .expect("archive corpus must be valid JSON");
    let mut manifest = corpus["cases"][0]["manifest"].clone();
    manifest["export_context"]["authorized_access_classes"] = serde_json::json!(["public"]);
    assert_eq!(evaluate_archive(&manifest).disposition, "disclosure_violation");
}

#[test]
fn an_expired_carrier_is_not_an_expired_evidence_record() {
    let corpus: Value = serde_json::from_str(include_str!("../../archive-cases-v0.1.json"))
        .expect("archive corpus must be valid JSON");
    let manifest = &corpus["cases"][6]["manifest"];
    assert_eq!(evaluate_archive(manifest).disposition, "eligible_for_independent_restore");
}


#[test]
fn access_classes_are_not_a_linear_clearance_hierarchy() {
    let corpus: Value = serde_json::from_str(include_str!("../../archive-cases-v0.1.json"))
        .expect("archive corpus");
    let case = corpus["cases"].as_array().expect("cases")
        .iter().find(|case| case["case_id"] == "ARCHIVE-NO-IMPLICIT-CLASS-HIERARCHY-001")
        .expect("explicit class grant fixture");
    assert_eq!(evaluate_archive(&case["manifest"]).disposition, "disclosure_violation");
}
