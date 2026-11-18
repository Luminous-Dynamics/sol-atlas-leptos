// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use sol_atlas_core::territorial::TerritorialRecord;

fn parse_fixture(raw: &str) -> TerritorialRecord {
    serde_json::from_str(raw).expect("territorial fixture must deserialize")
}

fn assert_rejected_for(record: &TerritorialRecord, fragment: &str) {
    let issues = record
        .validate()
        .expect_err("adversarial territorial fixture must be rejected");
    assert!(
        issues.iter().any(|issue| issue.contains(fragment)),
        "expected issue containing {fragment:?}; got {issues:#?}"
    );
}

#[test]
fn asserted_claim_is_preserved_without_implicit_title_promotion() {
    let record = parse_fixture(include_str!(
        "fixtures/territorial/valid_asserted_claim.json"
    ));
    assert_eq!(record.validate(), Ok(()));
    assert_eq!(
        record.assertions[0].bases,
        [sol_atlas_core::territorial::ClaimBasis::Asserted]
    );
}

#[test]
fn registered_record_can_remain_disputed_and_keep_multiple_authorities() {
    let record = parse_fixture(include_str!(
        "fixtures/territorial/valid_disputed_multiauthority_claim.json"
    ));
    assert_eq!(record.validate(), Ok(()));
    assert_eq!(
        record.assertions[0].dispute_state,
        sol_atlas_core::territorial::DisputeState::Challenged
    );
    assert_eq!(record.assertions[0].authority_ids.len(), 2);
}

#[test]
fn administrative_claim_without_registry_authority_is_rejected() {
    let record = parse_fixture(include_str!(
        "fixtures/territorial/invalid_administrative_claim_without_authority.json"
    ));
    assert_rejected_for(&record, "administrative_record without matching registry/regulator authority");
}

#[test]
fn testimony_alone_is_not_a_community_decision_or_authority() {
    let record = parse_fixture(include_str!(
        "fixtures/territorial/invalid_community_testimony_only.json"
    ));
    assert_rejected_for(&record, "community_attested without community-governance/customary authority");
}

#[test]
fn restoration_cannot_close_without_independent_verification() {
    let record = parse_fixture(include_str!(
        "fixtures/territorial/invalid_restoration_completion_without_independent_verifier.json"
    ));
    assert_rejected_for(&record, "verified_complete without independent verifier");
}
