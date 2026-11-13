// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-contract primitives for the Earth Observability benchmark.
//!
//! This module validates declarations and qualification preconditions. It does
//! not transform coordinates, estimate geological truth, run geophysical
//! inversions, or certify a physical survey.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRole {
    RawObservation,
    ProcessedObservation,
    Interpretation,
    ModelOutput,
    IndependentValidation,
    DecisionOrReview,
}

/// Availability and currentness are not represented by Option alone.
/// There is deliberately no Absent variant: absence is a qualified conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Present,
    Partial,
    Unknown,
    Unobserved,
    NotAcquired,
    Withheld,
    Restricted,
    Stale,
    Conflicted,
    Invalid,
    Superseded,
    OutOfDomain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialFrame {
    pub frame_id: String,
    pub horizontal_crs: Option<String>,
    pub axis_order: Option<Vec<String>>,
    pub horizontal_unit: Option<String>,
    pub vertical_datum: Option<String>,
    pub height_type: Option<String>,
    pub vertical_unit: Option<String>,
    pub coordinate_epoch: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialValidationError {
    EmptyFrameId,
    MissingHorizontalCrs,
    MissingAxisOrder,
    InvalidAxisOrder,
    FrameIdentityMismatch,
    AxisOrderMismatch,
    MissingHorizontalUnit,
    MissingVerticalDatum,
    MissingHeightType,
    MissingVerticalUnit,
    InvalidCoordinateEpoch,
}

/// Validate the declared frame against how the observation was encoded.
/// Axis order is compared exactly; it is never guessed from coordinate values.
pub fn validate_spatial_observation(
    frame: &SpatialFrame,
    observation_frame_id: &str,
    encoded_axis_order: &[String],
    has_vertical_coordinate: bool,
) -> Vec<SpatialValidationError> {
    let mut errors = Vec::new();

    if frame.frame_id.trim().is_empty() {
        errors.push(SpatialValidationError::EmptyFrameId);
    }
    if observation_frame_id != frame.frame_id {
        errors.push(SpatialValidationError::FrameIdentityMismatch);
    }
    if frame
        .horizontal_crs
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        errors.push(SpatialValidationError::MissingHorizontalCrs);
    }
    if frame
        .horizontal_unit
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        errors.push(SpatialValidationError::MissingHorizontalUnit);
    }

    match frame.axis_order.as_deref() {
        None => errors.push(SpatialValidationError::MissingAxisOrder),
        Some(order) => {
            let normalized: Vec<String> = order
                .iter()
                .map(|axis| axis.trim().to_ascii_lowercase())
                .collect();
            let unique: BTreeSet<&str> = normalized.iter().map(String::as_str).collect();
            if order.len() < 2
                || order.iter().any(|axis| axis.trim().is_empty())
                || unique.len() != order.len()
            {
                errors.push(SpatialValidationError::InvalidAxisOrder);
            } else if order != encoded_axis_order {
                errors.push(SpatialValidationError::AxisOrderMismatch);
            }
        }
    }

    if has_vertical_coordinate {
        if frame
            .vertical_datum
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            errors.push(SpatialValidationError::MissingVerticalDatum);
        }
        if frame
            .height_type
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            errors.push(SpatialValidationError::MissingHeightType);
        }
        if frame
            .vertical_unit
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            errors.push(SpatialValidationError::MissingVerticalUnit);
        }
    }

    if frame
        .coordinate_epoch
        .is_some_and(|epoch| !epoch.is_finite() || epoch <= 0.0)
    {
        errors.push(SpatialValidationError::InvalidCoordinateEpoch);
    }

    errors
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformReceipt {
    pub source_frame_id: String,
    pub target_frame_id: String,
    pub operation_id: String,
    pub operation_sha256: String,
    #[serde(default)]
    pub grid_resource_sha256: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformValidationError {
    EmptyOperationId,
    FrameIdentityMismatch,
    InvalidOperationDigest,
    InvalidGridDigest,
    DuplicateGridDigest,
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Check transform identity and content-addressing. This does not prove the
/// mathematical correctness of the operation or referenced grid resources.
pub fn validate_transform_receipt(
    receipt: &TransformReceipt,
    expected_source_frame_id: &str,
    expected_target_frame_id: &str,
) -> Vec<TransformValidationError> {
    let mut errors = Vec::new();

    if receipt.operation_id.trim().is_empty() {
        errors.push(TransformValidationError::EmptyOperationId);
    }
    if receipt.source_frame_id != expected_source_frame_id
        || receipt.target_frame_id != expected_target_frame_id
    {
        errors.push(TransformValidationError::FrameIdentityMismatch);
    }
    if !is_sha256_hex(&receipt.operation_sha256) {
        errors.push(TransformValidationError::InvalidOperationDigest);
    }

    let mut seen = BTreeSet::new();
    for digest in &receipt.grid_resource_sha256 {
        if !is_sha256_hex(digest) {
            errors.push(TransformValidationError::InvalidGridDigest);
        } else if !seen.insert(digest) {
            errors.push(TransformValidationError::DuplicateGridDigest);
        }
    }

    errors
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectionQualification {
    pub footprint_covers_target: bool,
    pub calibration_valid: bool,
    pub target_in_validated_domain: bool,
    pub probability_of_detection: Option<f64>,
    pub minimum_required_probability: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonDetectionDisposition {
    OutOfDomain,
    Unobserved,
    Unqualified,
    BelowDetectionLimit,
    /// Eligible to support an absence inference; not itself proof of absence.
    AbsenceEvidenceEligible,
}

/// Classify a non-detection without silently converting it into physical absence.
pub fn classify_non_detection(qualification: DetectionQualification) -> NonDetectionDisposition {
    if !qualification.target_in_validated_domain {
        return NonDetectionDisposition::OutOfDomain;
    }
    if !qualification.footprint_covers_target {
        return NonDetectionDisposition::Unobserved;
    }
    if !qualification.calibration_valid {
        return NonDetectionDisposition::Unqualified;
    }

    let Some(pod) = qualification.probability_of_detection else {
        return NonDetectionDisposition::Unqualified;
    };

    let required = qualification.minimum_required_probability;
    if !pod.is_finite()
        || !required.is_finite()
        || !(0.0..=1.0).contains(&pod)
        || !(0.0..=1.0).contains(&required)
    {
        return NonDetectionDisposition::Unqualified;
    }
    if pod < required {
        return NonDetectionDisposition::BelowDetectionLimit;
    }

    NonDetectionDisposition::AbsenceEvidenceEligible
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationSplit {
    pub fit_ids: Vec<String>,
    pub calibration_ids: Vec<String>,
    pub checkpoint_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitValidationError {
    EmptyIdentifier,
    DuplicateWithinPartition,
    CrossPartitionLeakage,
}

/// Check fitting, calibration and independent checkpoint IDs are disjoint.
/// This does not replace spatial-overlap or near-duplicate artifact audits.
pub fn validate_evaluation_split(split: &EvaluationSplit) -> Vec<SplitValidationError> {
    let mut errors = Vec::new();
    let mut all_ids = BTreeSet::new();

    for partition in [
        &split.fit_ids,
        &split.calibration_ids,
        &split.checkpoint_ids,
    ] {
        let mut local_ids = BTreeSet::new();
        for id in partition {
            if id.trim().is_empty() {
                errors.push(SplitValidationError::EmptyIdentifier);
            }
            if !local_ids.insert(id) {
                errors.push(SplitValidationError::DuplicateWithinPartition);
            }
            if !all_ids.insert(id) {
                errors.push(SplitValidationError::CrossPartitionLeakage);
            }
        }
    }

    errors
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurveyActionEligibility {
    pub land_access_authorized: bool,
    pub method_authorized: bool,
    pub required_permits_satisfied: bool,
    pub custodial_review_required: bool,
    pub custodial_review_completed: bool,
}

/// A high information-gain estimate never overrides missing authority.
pub fn survey_action_is_eligible(eligibility: SurveyActionEligibility) -> bool {
    eligibility.land_access_authorized
        && eligibility.method_authorized
        && eligibility.required_permits_satisfied
        && (!eligibility.custodial_review_required || eligibility.custodial_review_completed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDisposition {
    DetectedChange,
    BelowDetectionLimit,
    InvalidEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChangeEvidence {
    pub estimated_change: f64,
    pub combined_standard_uncertainty: f64,
    pub detection_threshold_sigma: f64,
}

/// Conservative scalar screen for a synthetic change-detection fixture.
/// Real surveys require spatial covariance and method-specific validation.
pub fn classify_change(evidence: ChangeEvidence) -> ChangeDisposition {
    if !evidence.estimated_change.is_finite()
        || !evidence.combined_standard_uncertainty.is_finite()
        || !evidence.detection_threshold_sigma.is_finite()
        || evidence.combined_standard_uncertainty < 0.0
        || evidence.detection_threshold_sigma <= 0.0
    {
        return ChangeDisposition::InvalidEvidence;
    }

    let detection_limit =
        evidence.combined_standard_uncertainty * evidence.detection_threshold_sigma;
    if evidence.estimated_change.abs() <= detection_limit {
        ChangeDisposition::BelowDetectionLimit
    } else {
        ChangeDisposition::DetectedChange
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeManifest {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub benchmark_id: String,
    pub manifest_version: String,
    pub status: String,
    pub created: String,
    pub geography: String,
    pub claim_boundary: String,
    pub case_count: usize,
    pub cases: Vec<ChallengeCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeCase {
    pub case_id: String,
    pub track: String,
    pub kind: String,
    pub modalities: Vec<String>,
    pub scenario: String,
    pub conditions: Vec<String>,
    pub expected_disposition: String,
    pub assertions: Vec<String>,
    pub reference: String,
    pub split_role: String,
    pub data_status: String,
}

/// Validate authored challenge-manifest invariants. Full JSON Schema validation
/// remains a separate gate; this is an offline native Rust guard.
pub fn validate_challenge_manifest(manifest: &ChallengeManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if manifest.benchmark_id != "EARTH-OBSERVABILITY-QUAL-001" {
        errors.push("unexpected benchmark_id".to_string());
    }
    if manifest.manifest_version.trim().is_empty() {
        errors.push("manifest_version is empty".to_string());
    }
    if manifest.status != "planned_synthetic_cases_not_yet_generated" {
        errors.push("manifest status must not imply generated measurements".to_string());
    }
    if manifest.geography != "abstract_non_geographic" {
        errors.push("synthetic challenge cases must be non-geographic".to_string());
    }
    if manifest.claim_boundary.trim().len() < 20 {
        errors.push("claim_boundary is missing or too short".to_string());
    }
    if manifest.case_count != manifest.cases.len() {
        errors.push("case_count does not match cases length".to_string());
    }
    if manifest.cases.is_empty() {
        errors.push("challenge manifest has no cases".to_string());
    }

    let valid_tracks = [
        "spatial_conformance",
        "survey_geometry",
        "observability",
        "subsurface_reconstruction",
        "evidence_semantics",
        "uncertainty_and_transfer",
        "adaptive_investigation",
        "temporal_revalidation",
    ];
    let valid_kinds = ["positive_control", "negative_control", "adversarial"];
    let mut case_ids = BTreeSet::new();

    for case in &manifest.cases {
        if case.case_id.trim().is_empty()
            || !case
                .case_id
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
        {
            errors.push(format!("invalid case_id: {}", case.case_id));
        }
        if !case_ids.insert(&case.case_id) {
            errors.push(format!("duplicate case_id: {}", case.case_id));
        }
        if !valid_tracks.contains(&case.track.as_str()) {
            errors.push(format!("unsupported track for {}", case.case_id));
        }
        if !valid_kinds.contains(&case.kind.as_str()) {
            errors.push(format!("unsupported case kind for {}", case.case_id));
        }
        if case.modalities.is_empty()
            || case.conditions.is_empty()
            || case.assertions.is_empty()
            || case.scenario.trim().len() < 20
            || case.expected_disposition.trim().is_empty()
            || case.reference.trim().is_empty()
        {
            errors.push(format!("incomplete case definition: {}", case.case_id));
        }
        if case.split_role != "synthetic_held_out" {
            errors.push(format!("invalid split_role for {}", case.case_id));
        }
        if case.data_status != "definition_only" {
            errors.push(format!(
                "case must remain definition_only: {}",
                case.case_id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Rights are supplied by an authoritative source/custody workflow. This
/// module checks that a declared grant covers a requested operation; it does
/// not parse licences, determine legal status, or authenticate the issuer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataUseAction {
    Read,
    Process,
    CreateDerivative,
    Redistribute,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataUseGrant {
    pub grant_id: String,
    pub source_id: String,
    pub issuer_id: String,
    pub grant_sha256: String,
    pub valid_from_unix_s: u64,
    pub expires_at_unix_s: Option<u64>,
    pub may_read: bool,
    pub may_process: bool,
    pub may_create_derivative: bool,
    pub may_redistribute: bool,
    pub conditions_satisfied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataUseDenial {
    MissingGrant,
    InvalidGrantReceipt,
    SourceIdentityMismatch,
    GrantNotYetValid,
    GrantExpired,
    ConditionsNotSatisfied,
    ActionNotGranted,
}

/// Fail closed unless an exact-source grant receipt explicitly authorizes the
/// requested action at the supplied clock time. The digest is syntax-checked
/// here; cryptographic receipt verification and issuer trust remain upstream.
pub fn authorize_data_use(
    grant: Option<&DataUseGrant>,
    source_id: &str,
    action: DataUseAction,
    now_unix_s: u64,
) -> Result<(), DataUseDenial> {
    let Some(grant) = grant else {
        return Err(DataUseDenial::MissingGrant);
    };

    if grant.grant_id.trim().is_empty()
        || grant.source_id.trim().is_empty()
        || grant.issuer_id.trim().is_empty()
        || !is_sha256_hex(&grant.grant_sha256)
    {
        return Err(DataUseDenial::InvalidGrantReceipt);
    }
    if grant.source_id != source_id {
        return Err(DataUseDenial::SourceIdentityMismatch);
    }
    if now_unix_s < grant.valid_from_unix_s {
        return Err(DataUseDenial::GrantNotYetValid);
    }
    if grant
        .expires_at_unix_s
        .is_some_and(|expires_at| now_unix_s >= expires_at)
    {
        return Err(DataUseDenial::GrantExpired);
    }
    if !grant.conditions_satisfied {
        return Err(DataUseDenial::ConditionsNotSatisfied);
    }

    let allowed = match action {
        DataUseAction::Read => grant.may_read,
        DataUseAction::Process => grant.may_read && grant.may_process,
        DataUseAction::CreateDerivative => {
            grant.may_read && grant.may_process && grant.may_create_derivative
        }
        DataUseAction::Redistribute => {
            grant.may_read
                && grant.may_process
                && grant.may_create_derivative
                && grant.may_redistribute
        }
    };

    if allowed {
        Ok(())
    } else {
        Err(DataUseDenial::ActionNotGranted)
    }
}

/// Independent interpretations can only be compared when they describe the
/// same explicitly declared spatial support. This is not a model-selection rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationClaim {
    pub claim_id: String,
    pub source_id: String,
    pub support_id: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretationComparison {
    InsufficientEvidence,
    IncomparableSupport,
    Consistent,
    Conflicted,
}

/// Preserve disagreement; never select a winning interpretation implicitly.
pub fn compare_interpretations(claims: &[InterpretationClaim]) -> InterpretationComparison {
    if claims.len() < 2
        || claims.iter().any(|claim| {
            claim.claim_id.trim().is_empty()
                || claim.source_id.trim().is_empty()
                || claim.support_id.trim().is_empty()
                || claim.value.trim().is_empty()
        })
    {
        return InterpretationComparison::InsufficientEvidence;
    }

    let first_support = &claims[0].support_id;
    if claims
        .iter()
        .any(|claim| &claim.support_id != first_support)
    {
        return InterpretationComparison::IncomparableSupport;
    }

    let distinct_values: BTreeSet<&str> = claims.iter().map(|claim| claim.value.as_str()).collect();
    if distinct_values.len() == 1 {
        InterpretationComparison::Consistent
    } else {
        InterpretationComparison::Conflicted
    }
}

/// A DAS event has fibre-relative chainage. This calibration record is required
/// before a later adapter may project chainage into geographic space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FibreRouteCalibration {
    pub calibration_id: String,
    pub route_geometry_sha256: String,
    pub valid_chainage_min_m: f64,
    pub valid_chainage_max_m: f64,
    pub horizontal_position_sigma_m: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FibreGeolocationDisposition {
    FibreChainageOnly,
    Unqualified,
    OutsideCalibratedRoute,
    GeographicProjectionEligible,
}

/// Qualifies only whether a route calibration is admissible for this chainage.
/// It deliberately returns no geographic coordinate and does not solve route
/// geometry. A separate projection adapter must consume the qualified receipt.
pub fn qualify_fibre_geolocation(
    chainage_m: f64,
    calibration: Option<&FibreRouteCalibration>,
) -> FibreGeolocationDisposition {
    if !chainage_m.is_finite() || chainage_m < 0.0 {
        return FibreGeolocationDisposition::Unqualified;
    }

    let Some(calibration) = calibration else {
        return FibreGeolocationDisposition::FibreChainageOnly;
    };

    if calibration.calibration_id.trim().is_empty()
        || !is_sha256_hex(&calibration.route_geometry_sha256)
        || !calibration.valid_chainage_min_m.is_finite()
        || !calibration.valid_chainage_max_m.is_finite()
        || calibration.valid_chainage_min_m < 0.0
        || calibration.valid_chainage_max_m <= calibration.valid_chainage_min_m
        || !calibration.horizontal_position_sigma_m.is_finite()
        || calibration.horizontal_position_sigma_m <= 0.0
    {
        return FibreGeolocationDisposition::Unqualified;
    }

    if chainage_m < calibration.valid_chainage_min_m
        || chainage_m > calibration.valid_chainage_max_m
    {
        FibreGeolocationDisposition::OutsideCalibratedRoute
    } else {
        FibreGeolocationDisposition::GeographicProjectionEligible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_frame() -> SpatialFrame {
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
    fn valid_spatial_observation_passes() {
        let frame = valid_frame();
        let encoded = vec!["east".to_string(), "north".to_string()];
        assert!(
            validate_spatial_observation(&frame, "synthetic-local-frame-a", &encoded, true)
                .is_empty()
        );
    }

    #[test]
    fn plausible_but_swapped_axis_order_is_rejected() {
        let frame = valid_frame();
        let encoded = vec!["north".to_string(), "east".to_string()];
        assert!(
            validate_spatial_observation(&frame, "synthetic-local-frame-a", &encoded, false)
                .contains(&SpatialValidationError::AxisOrderMismatch)
        );
    }

    #[test]
    fn vertical_coordinate_requires_vertical_semantics() {
        let mut frame = valid_frame();
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
    fn frame_identity_mismatch_is_rejected() {
        let frame = valid_frame();
        let encoded = vec!["east".to_string(), "north".to_string()];
        assert!(
            validate_spatial_observation(&frame, "different-frame", &encoded, false)
                .contains(&SpatialValidationError::FrameIdentityMismatch)
        );
    }

    #[test]
    fn transform_digest_and_frame_binding_are_checked() {
        let receipt = TransformReceipt {
            source_frame_id: "frame-a".to_string(),
            target_frame_id: "frame-b".to_string(),
            operation_id: "synthetic-transform-v1".to_string(),
            operation_sha256: "a".repeat(64),
            grid_resource_sha256: vec!["b".repeat(64)],
        };
        assert!(validate_transform_receipt(&receipt, "frame-a", "frame-b").is_empty());

        let mut bad = receipt.clone();
        bad.operation_sha256 = "not-a-digest".to_string();
        assert!(
            validate_transform_receipt(&bad, "frame-a", "frame-b")
                .contains(&TransformValidationError::InvalidOperationDigest)
        );
        assert!(
            validate_transform_receipt(&receipt, "frame-x", "frame-b")
                .contains(&TransformValidationError::FrameIdentityMismatch)
        );
    }

    #[test]
    fn duplicate_or_cross_partition_checkpoint_ids_are_rejected() {
        let split = EvaluationSplit {
            fit_ids: vec!["fit-1".to_string(), "fit-2".to_string()],
            calibration_ids: vec!["cal-1".to_string()],
            checkpoint_ids: vec!["fit-2".to_string(), "check-1".to_string()],
        };
        assert!(
            validate_evaluation_split(&split)
                .contains(&SplitValidationError::CrossPartitionLeakage)
        );
    }

    #[test]
    fn disjoint_evaluation_partitions_pass_identity_guard() {
        let split = EvaluationSplit {
            fit_ids: vec!["fit-1".to_string(), "fit-2".to_string()],
            calibration_ids: vec!["cal-1".to_string()],
            checkpoint_ids: vec!["check-1".to_string()],
        };
        assert!(validate_evaluation_split(&split).is_empty());
    }

    #[test]
    fn non_detection_without_coverage_is_unobserved_not_absent() {
        let result = classify_non_detection(DetectionQualification {
            footprint_covers_target: false,
            calibration_valid: true,
            target_in_validated_domain: true,
            probability_of_detection: Some(0.99),
            minimum_required_probability: 0.90,
        });
        assert_eq!(result, NonDetectionDisposition::Unobserved);
    }

    #[test]
    fn out_of_domain_precedes_high_nominal_detection_probability() {
        let result = classify_non_detection(DetectionQualification {
            footprint_covers_target: true,
            calibration_valid: true,
            target_in_validated_domain: false,
            probability_of_detection: Some(0.99),
            minimum_required_probability: 0.90,
        });
        assert_eq!(result, NonDetectionDisposition::OutOfDomain);
    }

    #[test]
    fn low_detection_probability_is_not_absence_evidence() {
        let result = classify_non_detection(DetectionQualification {
            footprint_covers_target: true,
            calibration_valid: true,
            target_in_validated_domain: true,
            probability_of_detection: Some(0.35),
            minimum_required_probability: 0.90,
        });
        assert_eq!(result, NonDetectionDisposition::BelowDetectionLimit);
    }

    #[test]
    fn calibrated_detection_domain_can_make_nondetection_evidence_eligible() {
        let result = classify_non_detection(DetectionQualification {
            footprint_covers_target: true,
            calibration_valid: true,
            target_in_validated_domain: true,
            probability_of_detection: Some(0.97),
            minimum_required_probability: 0.90,
        });
        assert_eq!(result, NonDetectionDisposition::AbsenceEvidenceEligible);
    }

    #[test]
    fn missing_authority_blocks_high_information_survey() {
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
    fn required_custodial_review_must_be_complete() {
        let eligibility = SurveyActionEligibility {
            land_access_authorized: true,
            method_authorized: true,
            required_permits_satisfied: true,
            custodial_review_required: true,
            custodial_review_completed: false,
        };
        assert!(!survey_action_is_eligible(eligibility));
    }

    #[test]
    fn change_inside_uncertainty_envelope_is_not_confirmed() {
        let disposition = classify_change(ChangeEvidence {
            estimated_change: 0.018,
            combined_standard_uncertainty: 0.012,
            detection_threshold_sigma: 2.0,
        });
        assert_eq!(disposition, ChangeDisposition::BelowDetectionLimit);
    }

    #[test]
    fn change_outside_declared_detection_limit_is_detected_in_fixture() {
        let disposition = classify_change(ChangeEvidence {
            estimated_change: 0.050,
            combined_standard_uncertainty: 0.010,
            detection_threshold_sigma: 2.0,
        });
        assert_eq!(disposition, ChangeDisposition::DetectedChange);
    }

    #[test]
    fn invalid_change_uncertainty_is_rejected() {
        let disposition = classify_change(ChangeEvidence {
            estimated_change: 0.050,
            combined_standard_uncertainty: f64::NAN,
            detection_threshold_sigma: 2.0,
        });
        assert_eq!(disposition, ChangeDisposition::InvalidEvidence);
    }
}