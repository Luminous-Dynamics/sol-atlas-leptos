// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::BTreeSet;

use serde_json::Value;
use tib_state_transition_harness::{evaluate_case, Outcome};

const CORPUS: &str = include_str!(
    "../../../benchmarks/territorial-integrity/tib-property-state-transitions-v0.1.json"
);

fn corpus() -> Value {
    serde_json::from_str(CORPUS).expect("TIB property state transition corpus must parse")
}

#[test]
fn frozen_transition_corpus_has_nine_unique_synthetic_scenarios() {
    let root = corpus();
    assert_eq!(root["suite_id"], "TIB-PROPERTY-STATE-TRANSITIONS-001");
    assert_eq!(root["data_classification"], "synthetic_non_personal");
    assert_eq!(root["real_world_case_data_included"], false);
    assert_eq!(root["claim_ceiling"], "reference-only-no-legal-effect");

    let scenarios = root["scenarios"]
        .as_array()
        .expect("scenarios must be an array");
    assert_eq!(scenarios.len(), 9);

    let mut ids = BTreeSet::new();
    for scenario in scenarios {
        let id = scenario["scenario_id"]
            .as_str()
            .expect("scenario ID must be a string");
        assert!(ids.insert(id), "duplicate transition scenario {id}");
        assert_eq!(scenario["expected_authoritative_mutation"], false);
    }
}

#[test]
fn every_frozen_scenario_matches_expected_host_disposition_without_mutation() {
    let root = corpus();
    let scenarios = root["scenarios"].as_array().unwrap();

    for scenario in scenarios {
        let id = scenario["scenario_id"].as_str().unwrap();
        let result = evaluate_case(scenario);

        let expected_outcome = scenario["expected_outcome"].as_str().unwrap();
        let expected_disposition = scenario["expected_host_disposition"].as_str().unwrap();

        let actual_outcome = match result.outcome {
            Outcome::ReviewRequired => "review_required",
            Outcome::NotEvaluable => "not_evaluable",
            Outcome::StructuralReject => "structural_reject",
        };

        assert_eq!(actual_outcome, expected_outcome, "{id}");
        assert_eq!(result.host_disposition, expected_disposition, "{id}");
        assert!(!result.authoritative_mutation, "{id} must never mutate an authority");
    }
}

#[test]
fn timed_out_operation_retry_is_rejected_until_first_outcome_is_reconciled() {
    let root = corpus();
    let retry = root["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scenario| scenario["scenario_id"] == "TIB-PT-009")
        .unwrap();
    let result = evaluate_case(retry);
    assert_eq!(result.outcome, Outcome::StructuralReject);
    assert_eq!(result.host_disposition, "reject_retry_and_hold");
    assert!(!result.authoritative_mutation);
}

#[test]
fn internal_completion_and_external_registration_remain_distinct() {
    let root = corpus();
    let transfer = root["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scenario| scenario["scenario_id"] == "TIB-PT-001")
        .unwrap();
    assert_eq!(transfer["initial_internal_state"]["transfer"], "completed");
    assert_eq!(
        transfer["external_operation"]["request_state"],
        "not_submitted"
    );

    let result = evaluate_case(transfer);
    assert_eq!(result.outcome, Outcome::ReviewRequired);
    assert_eq!(result.host_disposition, "hold_for_reconciliation");
    assert!(!result.authoritative_mutation);
}

#[test]
fn digest_without_issuer_competence_and_valid_receipt_with_active_conflict_are_held() {
    let root = corpus();
    let scenarios = root["scenarios"].as_array().unwrap();

    for id in ["TIB-PT-002", "TIB-PT-006", "TIB-PT-008"] {
        let scenario = scenarios
            .iter()
            .find(|scenario| scenario["scenario_id"] == id)
            .unwrap();
        let result = evaluate_case(scenario);
        assert_eq!(result.outcome, Outcome::ReviewRequired, "{id}");
        assert!(!result.authoritative_mutation, "{id}");
    }

#[test]
fn replay_of_identical_evidence_frontier_is_deterministic() {
    let root = corpus();
    for scenario in root["scenarios"].as_array().unwrap() {
        let first = evaluate_case(scenario);
        let second = evaluate_case(scenario);
        assert_eq!(first, second, "{}", scenario["scenario_id"]);
    }
}

#[test]
fn surface_parcel_cannot_substitute_for_a_subsurface_volume() {
    let root = corpus();
    let scenario = root["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scenario| scenario["scenario_id"] == "TIB-PT-007")
        .unwrap();

    assert_eq!(
        scenario["conflicts"][0]["kind"],
        "spatial_representation_mismatch"
    );
    assert!(scenario["conflicts"][0]["subsurface_volume"].is_null());
    let result = evaluate_case(scenario);
    assert_eq!(result.outcome, Outcome::ReviewRequired);
    assert!(!result.authoritative_mutation);
}

#[test]
fn authenticated_issuer_outside_decision_scope_is_not_promoted() {
    let root = corpus();
    let scenario = root["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scenario| scenario["scenario_id"] == "TIB-PT-008")
        .unwrap();

    assert_eq!(
        scenario["authority_assessment"]["issuer"],
        "authenticated"
    );
    let result = evaluate_case(scenario);
    assert_eq!(result.outcome, Outcome::ReviewRequired);
    assert_eq!(result.host_disposition, "hold_for_reconciliation");
    assert!(!result.authoritative_mutation);
}

}
