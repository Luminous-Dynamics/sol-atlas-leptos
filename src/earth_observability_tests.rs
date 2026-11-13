// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Native contract tests for the Earth Observability research corpus.
//! These fixtures are abstract and synthetic; they do not qualify field data.

use sol_atlas_core::earth_observability::{
    ChangeDisposition, ChangeEvidence, ChallengeManifest, DetectionQualification,
    EvaluationSplit, NonDetectionDisposition, SpatialFrame, SpatialValidationError,
    SurveyActionEligibility, TransformReceipt, TransformValidationError, classify_change,
    classify_non_detection, survey_action_is_eligible, validate_challenge_manifest,
    validate_evaluation_split, validate_spatial_observation, validate_transform_receipt,
};

const CASE_MANIFEST: &str =
    include_str!("../research/earth-observability/earth-observability-case-manifest-v0.1.json");

fn local_frame() -> SpatialFrame {
    SpatialFrame {
        frame_id: "synthetic-local-frame-a".to_string(),
        horizontal_crs: Some("LOCAL:METRIC".to_string()),
        axis_order: Some(vec!["east".to_string(), "north".to_string()]),
        horizontal_unit: Some("metre".to_string()),
        vertical_datum: Some("local-vertical-datum-v1".to_string()),
        height_type: Some("orthometric_height".to_string()),
        vertical_unit: Some("metre".to_string()),
        coordinate_epoch: Some(2026.0),
    }
}

#[test]
fn earth_observability_manifest_parses_and_passes_contract() {
    let manifest: ChallengeManifest =
        serde_json::from_str(CASE_MANIFEST).expect("challenge manifest must parse");
    assert_eq!(manifest.case_count, 18);
    validate_challenge_manifest(&manifest).expect("manifest contract must pass");
}

#[test]
fn challenge_manifest_rejects_duplicate_case_ids() {
    let mut manifest: ChallengeManifest =
        serde_json::from_str(CASE_MANIFEST).expect("challenge manifest must parse");
    manifest.cases[1].case_id = manifest.cases[0].case_id.clone();
    let errors = validate_challenge_manifest(&manifest).expect_err("duplicate must fail");
    assert!(errors.iter().any(|error| error.contains("duplicate case_id")));
}

#[test]
fn challenge_manifest_cannot_be_promoted_to_generated_measurements() {
    let mut manifest: ChallengeManifest =
        serde_json::from_str(CASE_MANIFEST).expect("challenge manifest must parse");
    manifest.status = "synthetic_measurements_generated".to_string();
    assert!(validate_challenge_manifest(&manifest).is_err());
}

#[test]
fn coordinate_axis_order_must_match_declared_frame() {
    let frame = local_frame();
    let swapped = vec!["north".to_string(), "east".to_string()];
    let errors = validate_spatial_observation(
        &frame,
        &frame.frame_id,
        &swapped,
        false,
    );
    assert!(errors.contains(&SpatialValidationError::AxisOrderMismatch));
}

#[test]
fn three_dimensional_observation_requires_vertical_datum_and_height_type() {
    let mut frame = local_frame();
    frame.vertical_datum = None;
    frame.height_type = None;
    frame.vertical_unit = None;
    let encoded = vec!["east".to_string(), "north".to_string()];
    let errors = validate_spatial_observation(&frame, &frame.frame_id, &encoded, true);
    assert!(errors.contains(&SpatialValidationError::MissingVerticalDatum));
    assert!(errors.contains(&SpatialValidationError::MissingHeightType));
    assert!(errors.contains(&SpatialValidationError::MissingVerticalUnit));
}

#[test]
fn coordinate_transform_must_bind_exact_frames_and_digests() {
    let receipt = TransformReceipt {
        source_frame_id: "frame-a".to_string(),
        target_frame_id: "frame-b".to_string(),
        operation_id: "synthetic-transform-v1".to_string(),
        operation_sha256: "a".repeat(64),
        grid_resource_sha256: vec!["b".repeat(64)],
    };
    assert!(validate_transform_receipt(&receipt, "frame-a", "frame-b").is_empty());

    let mut bad_digest = receipt.clone();
    bad_digest.operation_sha256 = "not-a-digest".to_string();
    assert!(validate_transform_receipt(&bad_digest, "frame-a", "frame-b")
        .contains(&TransformValidationError::InvalidOperationDigest));
    assert!(validate_transform_receipt(&receipt, "frame-x", "frame-b")
        .contains(&TransformValidationError::FrameIdentityMismatch));
}

#[test]
fn independent_checkpoints_cannot_leak_into_fit_partition() {
    let split = EvaluationSplit {
        fit_ids: vec!["fit-1".to_string(), "fit-2".to_string()],
        calibration_ids: vec!["cal-1".to_string()],
        checkpoint_ids: vec!["fit-2".to_string(), "check-1".to_string()],
    };
    assert!(!validate_evaluation_split(&split).is_empty());
}

#[test]
fn non_detection_without_target_coverage_is_unobserved() {
    let disposition = classify_non_detection(DetectionQualification {
        footprint_covers_target: false,
        calibration_valid: true,
        target_in_validated_domain: true,
        probability_of_detection: Some(0.99),
        minimum_required_probability: 0.90,
    });
    assert_eq!(disposition, NonDetectionDisposition::Unobserved);
}

#[test]
fn below_detection_limit_is_not_absence_evidence() {
    let disposition = classify_non_detection(DetectionQualification {
        footprint_covers_target: true,
        calibration_valid: true,
        target_in_validated_domain: true,
        probability_of_detection: Some(0.35),
        minimum_required_probability: 0.90,
    });
    assert_eq!(disposition, NonDetectionDisposition::BelowDetectionLimit);
}

#[test]
fn no_detection_outside_validated_domain_is_out_of_domain() {
    let disposition = classify_non_detection(DetectionQualification {
        footprint_covers_target: true,
        calibration_valid: true,
        target_in_validated_domain: false,
        probability_of_detection: Some(0.99),
        minimum_required_probability: 0.90,
    });
    assert_eq!(disposition, NonDetectionDisposition::OutOfDomain);
}

#[test]
fn survey_authority_is_a_hard_gate_not_a_weighted_preference() {
    let eligibility = SurveyActionEligibility {
        land_access_authorized: true,
        method_authorized: true,
        required_permits_satisfied: false,
        custodial_review_required: false,
        custodial_review_completed: false,
    };
    assert!(!survey_action_is_eligible(eligibility));
}

#[test]
fn change_inside_uncertainty_detection_limit_is_not_confirmed() {
    let disposition = classify_change(ChangeEvidence {
        estimated_change: 0.018,
        combined_standard_uncertainty: 0.012,
        detection_threshold_sigma: 2.0,
    });
    assert_eq!(disposition, ChangeDisposition::BelowDetectionLimit);
}

#[test]
fn invalid_change_uncertainty_is_rejected() {
    let disposition = classify_change(ChangeEvidence {
        estimated_change: 0.05,
        combined_standard_uncertainty: f64::NAN,
        detection_threshold_sigma: 2.0,
    });
    assert_eq!(disposition, ChangeDisposition::InvalidEvidence);
}