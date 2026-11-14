// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Per-activity, per-metric truth and acquisition admissibility.
//!
//! One physical activity can be eligible for signal/event scoring while being
//! ineligible for localization, geometry or topology. Missing spatial support,
//! weak positioning or incomplete trench truth is represented explicitly;
//! it is never silently promoted into negative labels or precise coordinates.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityTruthTier {
    StructuredIndependentSurvey,
    TrenchImageInterpreted,
    OwnerRecordOnly,
    Conflicted,
    Insufficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityTargetTruth {
    Present,
    AbsentWithinInspectedSupport,
    Unknown,
    Conflicted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositioningSource {
    GnssRtk,
    WheelEncoderOnly,
    Mixed,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WheelContactStatus {
    Verified,
    Compromised,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialReferenceStatus {
    Qualified,
    Partial,
    Missing,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldActivityAdmission {
    pub dataset_id: String,
    pub project_id: String,
    pub activity_id: String,
    pub raw_radargram_sha256: Option<String>,
    pub truth_tier: ActivityTruthTier,
    pub target_truth: ActivityTargetTruth,
    pub truth_evidence_ids: Vec<String>,
    pub target_linkage_reviewed: bool,
    pub target_reference_id: Option<String>,
    pub spatial_reference_status: SpatialReferenceStatus,
    pub spatial_support_id: Option<String>,
    pub positioning_source: PositioningSource,
    pub positions_trace_aligned: bool,
    pub positioning_calibration_receipt_id: Option<String>,
    pub horizontal_position_sigma_m: Option<f64>,
    pub wheel_contact_status: WheelContactStatus,
    pub geometry_reference_id: Option<String>,
    /// Required when a trench-image interpretation is used for metric geometry.
    pub trench_geometry_registration_receipt_id: Option<String>,
    pub has_3d_geometry: bool,
    pub connected_topology_reference_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldEvaluationTrack {
    SignalEventDetection,
    EventToTargetAssociation,
    Localization,
    Geometry2d,
    NetworkTopology3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackIneligibilityReason {
    RawRadargramMissing,
    RawRadargramDigestInvalid,
    TruthNotIndependent,
    TruthStateUnknownOrConflicted,
    TargetTruthLinkNotReviewed,
    TargetReferenceMissing,
    InspectedSupportReferenceMissing,
    SpatialSupportMissing,
    SpatialReferenceNotQualified,
    PositioningNotTraceAligned,
    PositioningSourceMissing,
    WheelContactNotVerified,
    PositioningCalibrationMissing,
    PositionUncertaintyMissingOrInvalid,
    GeometryReferenceMissing,
    TrenchGeometryRegistrationMissing,
    Structured3dGeometryMissing,
    ConnectedTopologyReferenceMissing,
    StructuredIndependentTopologyTruthRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "reason", rename_all = "snake_case")]
pub enum TrackEligibility {
    Eligible,
    Ineligible(TrackIneligibilityReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivityTrackAdmissibility {
    pub activity_id: String,
    pub signal_event_detection: TrackEligibility,
    pub event_to_target_association: TrackEligibility,
    pub localization: TrackEligibility,
    pub geometry_2d: TrackEligibility,
    pub network_topology_3d: TrackEligibility,
    pub claim_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldActivityAdmissionError {
    EmptyIdentifier(&'static str),
    InvalidRawDigest,
    DuplicateTruthEvidence(String),
    EmptyTruthEvidence(String),
    InvalidTargetReference,
    InvalidSpatialSupport,
    InvalidPositioningCalibration,
    InvalidGeometryReference,
    InvalidGeometryRegistrationReceipt,
    InvalidTopologyReference,
    InvalidPositionSigma,
    InconsistentWheelContactDeclaration,
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn independent_truth(tier: ActivityTruthTier) -> bool {
    matches!(
        tier,
        ActivityTruthTier::StructuredIndependentSurvey
            | ActivityTruthTier::TrenchImageInterpreted
    )
}

fn truth_is_scoreable(state: ActivityTargetTruth) -> bool {
    matches!(
        state,
        ActivityTargetTruth::Present | ActivityTargetTruth::AbsentWithinInspectedSupport
    )
}

pub fn validate_field_activity_admission(
    activity: &FieldActivityAdmission,
) -> Vec<FieldActivityAdmissionError> {
    let mut errors = Vec::new();
    for (field, value) in [
        ("dataset_id", activity.dataset_id.as_str()),
        ("project_id", activity.project_id.as_str()),
        ("activity_id", activity.activity_id.as_str()),
    ] {
        if !non_empty(value) {
            errors.push(FieldActivityAdmissionError::EmptyIdentifier(field));
        }
    }

    if activity
        .raw_radargram_sha256
        .as_deref()
        .is_some_and(|digest| !valid_sha256(digest))
    {
        errors.push(FieldActivityAdmissionError::InvalidRawDigest);
    }

    let mut evidence = BTreeSet::new();
    for evidence_id in &activity.truth_evidence_ids {
        if !non_empty(evidence_id) {
            errors.push(FieldActivityAdmissionError::EmptyTruthEvidence(
                activity.activity_id.clone(),
            ));
        } else if !evidence.insert(evidence_id.as_str()) {
            errors.push(FieldActivityAdmissionError::DuplicateTruthEvidence(
                evidence_id.clone(),
            ));
        }
    }

    if activity
        .target_reference_id
        .as_deref()
        .is_some_and(|value| !non_empty(value))
    {
        errors.push(FieldActivityAdmissionError::InvalidTargetReference);
    }
    if activity
        .spatial_support_id
        .as_deref()
        .is_some_and(|value| !non_empty(value))
    {
        errors.push(FieldActivityAdmissionError::InvalidSpatialSupport);
    }
    if activity
        .positioning_calibration_receipt_id
        .as_deref()
        .is_some_and(|value| !non_empty(value))
    {
        errors.push(FieldActivityAdmissionError::InvalidPositioningCalibration);
    }
    if activity.horizontal_position_sigma_m.is_some_and(|sigma| {
        !sigma.is_finite() || sigma <= 0.0
    }) {
        errors.push(FieldActivityAdmissionError::InvalidPositionSigma);
    }
    if activity
        .geometry_reference_id
        .as_deref()
        .is_some_and(|value| !non_empty(value))
    {
        errors.push(FieldActivityAdmissionError::InvalidGeometryReference);
    }
    if activity
        .trench_geometry_registration_receipt_id
        .as_deref()
        .is_some_and(|value| !non_empty(value))
    {
        errors.push(FieldActivityAdmissionError::InvalidGeometryRegistrationReceipt);
    }
    if activity
        .connected_topology_reference_id
        .as_deref()
        .is_some_and(|value| !non_empty(value))
    {
        errors.push(FieldActivityAdmissionError::InvalidTopologyReference);
    }
    if activity.positioning_source == PositioningSource::WheelEncoderOnly
        && activity.wheel_contact_status == WheelContactStatus::NotApplicable
    {
        errors.push(FieldActivityAdmissionError::InconsistentWheelContactDeclaration);
    }

    errors
}

fn event_track_reason(activity: &FieldActivityAdmission) -> Option<TrackIneligibilityReason> {
    if activity.raw_radargram_sha256.is_none() {
        return Some(TrackIneligibilityReason::RawRadargramMissing);
    }
    if activity
        .raw_radargram_sha256
        .as_deref()
        .is_some_and(|digest| !valid_sha256(digest))
    {
        return Some(TrackIneligibilityReason::RawRadargramDigestInvalid);
    }
    if !independent_truth(activity.truth_tier) {
        return Some(TrackIneligibilityReason::TruthNotIndependent);
    }
    if !truth_is_scoreable(activity.target_truth) {
        return Some(TrackIneligibilityReason::TruthStateUnknownOrConflicted);
    }
    if !activity.target_linkage_reviewed || activity.truth_evidence_ids.is_empty() {
        return Some(TrackIneligibilityReason::TargetTruthLinkNotReviewed);
    }
    if activity.target_truth == ActivityTargetTruth::AbsentWithinInspectedSupport
        && activity.target_reference_id.as_deref().map(non_empty) != Some(true)
    {
        return Some(TrackIneligibilityReason::InspectedSupportReferenceMissing);
    }
    None
}

fn localization_reason(activity: &FieldActivityAdmission) -> Option<TrackIneligibilityReason> {
    if event_track_reason(activity).is_some() {
        return event_track_reason(activity);
    }
    if activity.spatial_support_id.as_deref().map(non_empty) != Some(true) {
        return Some(TrackIneligibilityReason::SpatialSupportMissing);
    }
    if activity.spatial_reference_status != SpatialReferenceStatus::Qualified {
        return Some(TrackIneligibilityReason::SpatialReferenceNotQualified);
    }
    if !activity.positions_trace_aligned {
        return Some(TrackIneligibilityReason::PositioningNotTraceAligned);
    }
    match activity.positioning_source {
        PositioningSource::Missing => {
            return Some(TrackIneligibilityReason::PositioningSourceMissing);
        }
        PositioningSource::WheelEncoderOnly | PositioningSource::Mixed
            if activity.wheel_contact_status != WheelContactStatus::Verified =>
        {
            return Some(TrackIneligibilityReason::WheelContactNotVerified);
        }
        _ => {}
    }
    if activity
        .positioning_calibration_receipt_id
        .as_deref()
        .map(non_empty)
        != Some(true)
    {
        return Some(TrackIneligibilityReason::PositioningCalibrationMissing);
    }
    if !activity
        .horizontal_position_sigma_m
        .is_some_and(|sigma| sigma.is_finite() && sigma > 0.0)
    {
        return Some(TrackIneligibilityReason::PositionUncertaintyMissingOrInvalid);
    }
    None
}

fn eligibility(
    reason: Option<TrackIneligibilityReason>,
) -> TrackEligibility {
    reason.map_or(TrackEligibility::Eligible, TrackEligibility::Ineligible)
}

pub fn assess_field_activity(
    activity: &FieldActivityAdmission,
) -> Result<ActivityTrackAdmissibility, Vec<FieldActivityAdmissionError>> {
    let errors = validate_field_activity_admission(activity);
    if !errors.is_empty() {
        return Err(errors);
    }

    let event_reason = event_track_reason(activity);
    let association_reason = if event_reason.is_some() {
        event_reason
    } else if activity.target_reference_id.as_deref().map(non_empty) != Some(true) {
        Some(TrackIneligibilityReason::TargetReferenceMissing)
    } else {
        None
    };

    let localization = localization_reason(activity);
    let geometry_reason = if localization.is_some() {
        localization
    } else if activity.geometry_reference_id.as_deref().map(non_empty) != Some(true) {
        Some(TrackIneligibilityReason::GeometryReferenceMissing)
    } else if activity.truth_tier == ActivityTruthTier::TrenchImageInterpreted
        && activity
            .trench_geometry_registration_receipt_id
            .as_deref()
            .map(non_empty)
            != Some(true)
    {
        Some(TrackIneligibilityReason::TrenchGeometryRegistrationMissing)
    } else {
        None
    };

    let topology_reason = if localization.is_some() {
        localization
    } else if activity.truth_tier != ActivityTruthTier::StructuredIndependentSurvey {
        Some(TrackIneligibilityReason::StructuredIndependentTopologyTruthRequired)
    } else if !activity.has_3d_geometry {
        Some(TrackIneligibilityReason::Structured3dGeometryMissing)
    } else if activity
        .connected_topology_reference_id
        .as_deref()
        .map(non_empty)
        != Some(true)
    {
        Some(TrackIneligibilityReason::ConnectedTopologyReferenceMissing)
    } else {
        None
    };

    Ok(ActivityTrackAdmissibility {
        activity_id: activity.activity_id.clone(),
        signal_event_detection: eligibility(event_reason),
        event_to_target_association: eligibility(association_reason),
        localization: eligibility(localization),
        geometry_2d: eligibility(geometry_reason),
        network_topology_3d: eligibility(topology_reason),
        claim_boundary: "Per-activity metric eligibility only. Eligibility does not establish detection performance, geometry accuracy, complete truth, sensor qualification or engineering acceptance.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/field-activity-admissibility-synthetic-v0.1.json"
    );

    fn fixture() -> Vec<FieldActivityAdmission> {
        serde_json::from_str(FIXTURE).expect("field activity admission fixture parses")
    }

    #[test]
    fn field_activity_fixture_parses_and_assesses() {
        let activities = fixture();
        assert_eq!(activities.len(), 5);
        assert!(activities.iter().all(|activity| validate_field_activity_admission(activity).is_empty()));
    }

    #[test]
    fn wheel_encoder_with_unverified_contact_cannot_enter_localization() {
        let activity = &fixture()[1];
        let result = assess_field_activity(activity).unwrap();
        assert_eq!(result.signal_event_detection, TrackEligibility::Eligible);
        assert_eq!(result.localization, TrackEligibility::Ineligible(
            TrackIneligibilityReason::WheelContactNotVerified
        ));
        assert_eq!(result.geometry_2d, result.localization);
        assert_eq!(result.network_topology_3d, result.localization);
    }

    #[test]
    fn missing_georeferencing_does_not_block_signal_detection() {
        let activity = &fixture()[2];
        let result = assess_field_activity(activity).unwrap();
        assert_eq!(result.signal_event_detection, TrackEligibility::Eligible);
        assert_eq!(result.event_to_target_association, TrackEligibility::Eligible);
        assert_eq!(result.localization, TrackEligibility::Ineligible(
            TrackIneligibilityReason::SpatialSupportMissing
        ));
    }

    #[test]
    fn mixed_positioning_with_unverified_wheel_contact_is_not_localization_eligible() {
        let mut activity = fixture()[4].clone();
        activity.positioning_source = PositioningSource::Mixed;
        activity.wheel_contact_status = WheelContactStatus::Compromised;
        let result = assess_field_activity(&activity).unwrap();
        assert_eq!(
            result.localization,
            TrackEligibility::Ineligible(TrackIneligibilityReason::WheelContactNotVerified)
        );
    }

    #[test]
    fn record_only_truth_is_not_a_negative_example() {
        let result = assess_field_activity(&fixture()[3]).unwrap();
        assert_eq!(result.signal_event_detection, TrackEligibility::Ineligible(
            TrackIneligibilityReason::TruthNotIndependent
        ));
    }

    #[test]
    fn trench_image_truth_needs_registered_geometry_and_cannot_establish_3d_topology() {
        let activity = &fixture()[0];
        let result = assess_field_activity(activity).unwrap();
        assert_eq!(result.geometry_2d, TrackEligibility::Ineligible(
            TrackIneligibilityReason::TrenchGeometryRegistrationMissing
        ));
        assert_eq!(result.network_topology_3d, TrackEligibility::Ineligible(
            TrackIneligibilityReason::StructuredIndependentTopologyTruthRequired
        ));
    }

    #[test]
    fn topology_requires_structured_3d_truth_and_connected_reference() {
        let mut activity = fixture()[4].clone();
        activity.connected_topology_reference_id = None;
        let result = assess_field_activity(&activity).unwrap();
        assert_eq!(result.localization, TrackEligibility::Eligible);
        assert_eq!(result.geometry_2d, TrackEligibility::Eligible);
        assert_eq!(result.network_topology_3d, TrackEligibility::Ineligible(
            TrackIneligibilityReason::ConnectedTopologyReferenceMissing
        ));
    }

    #[test]
    fn conflicted_or_unknown_truth_is_not_scored_as_absence() {
        let mut activity = fixture()[4].clone();
        activity.target_truth = ActivityTargetTruth::Conflicted;
        let result = assess_field_activity(&activity).unwrap();
        assert_eq!(result.signal_event_detection, TrackEligibility::Ineligible(
            TrackIneligibilityReason::TruthStateUnknownOrConflicted
        ));
    }

    #[test]
    fn absence_truth_requires_a_named_inspected_support() {
        let mut activity = fixture()[5].clone();
        activity.target_reference_id = None;
        let result = assess_field_activity(&activity).unwrap();
        assert_eq!(
            result.signal_event_detection,
            TrackEligibility::Ineligible(
                TrackIneligibilityReason::InspectedSupportReferenceMissing
            )
        );
    }

    #[test]
    fn reviewed_absence_is_scoped_and_does_not_claim_universal_empty_ground() {
        let activity = fixture()[5].clone();
        let result = assess_field_activity(&activity).unwrap();
        assert_eq!(result.signal_event_detection, TrackEligibility::Eligible);
        assert_eq!(result.event_to_target_association, TrackEligibility::Eligible);
        assert_eq!(result.localization, TrackEligibility::Eligible);
        assert_eq!(result.network_topology_3d, TrackEligibility::Ineligible(
            TrackIneligibilityReason::Structured3dGeometryMissing
        ));
    }

    #[test]
    fn invalid_position_uncertainty_is_rejected() {
        let mut activity = fixture()[4].clone();
        activity.horizontal_position_sigma_m = Some(f64::NAN);
        assert!(validate_field_activity_admission(&activity).contains(
            &FieldActivityAdmissionError::InvalidPositionSigma
        ));
    }
}
