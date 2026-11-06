// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! HDB-24A: fail-closed projections from HDB-11A and HDB-15 evidence into
//! the factorized HDB-22 v0.1 stage contract.
//!
//! This module deliberately does not treat a HDB-15 area fraction as a
//! target-location probability. It requires a separate target-location
//! intersection receipt and preserves the source report/profile/split/rights
//! provenance. Composite survey-unit POD is rejected until HDB-22 has a
//! separately qualified composite update mode.
//!
//! The projections mirror HDB-22's StageEvidence fields but are kept separate
//! while the HDB-11A/HDB-15/HDB-22 PR stack is still under review. They are
//! intended for a later direct conversion once those source branches land.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::heritage_observability::{
    DetectionCondition, DetectionProbabilityScope, HeritageObservabilityReport,
    ObservationOutcome, ObservabilityDisposition, RightsStatus, SurveyAccessTier, SurveyContext,
};

pub const HERITAGE_COMPOSITION_SCHEMA_VERSION: &str =
    "luminous.heritage-composition-hdb24a.v0.1";
pub const HDB15_FIELD_SWEEP_REPORT_SCHEMA_VERSION: &str =
    "luminous.heritage-field-sweep-report.v0.1";
pub const MAX_ABSOLUTE_ARITHMETIC_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hdb24CompositionMode {
    FactorizedConditional,
    CompositeSurveyUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hdb24CoverageProbabilityModel {
    UniformAreaWeightedWithinSurveyUnit,
    TargetSpecificSpatialDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb24ProbabilityBand {
    pub lower: f64,
    pub upper: f64,
}

impl Hdb24ProbabilityBand {
    fn validate(self, label: &str) -> Result<(), Hdb24CompositionError> {
        if !self.lower.is_finite()
            || !self.upper.is_finite()
            || self.lower < 0.0
            || self.upper > 1.0
            || self.lower > self.upper
        {
            return Err(Hdb24CompositionError::InvalidProbabilityBand(label.into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hdb24RightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb24Hdb15SourceBinding {
    /// Digest of the complete, canonical HDB-15 report bytes.
    pub report_sha256: String,
    pub schema_version: String,
    pub evaluation_id: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub calibration_data_sha256: String,
    pub calibration_split_manifest_sha256: String,
    pub fit_receipt_sha256: String,
    pub target_class_id: String,
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    pub survey_unit_area_m2: f64,
    pub transect_geometry_sha256: String,
    pub survey_unit_geometry_sha256: String,
    /// Digest of the actual union geometry, supplied by the GIS receipt.
    pub coverage_union_geometry_sha256: String,
    pub spatial_union_receipt_sha256: String,
    pub spatial_union_method_id: String,
    pub spatial_union_method_build_sha256: String,
    pub effective_sweep_width_m: f64,
    pub buffered_sweep_width_m: f64,
    pub receipt_reported_swept_area_m2: f64,
    /// Descriptive GIS union area / survey-unit area. Never a POD factor.
    pub receipt_reported_coverage_fraction: f64,
    pub context_match: bool,
    pub rights_status: Hdb24RightsStatus,
    pub rights_decision_id: String,
    pub rights_receipt_ref: String,
    pub access_tier: SurveyAccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb24ContextBinding {
    /// Existing HDB-22 event identity. This must match the HDB-11 observation.
    pub event_id: String,
    pub candidate_id: String,
    pub target_class_id: String,
    pub context_id: String,
    pub context_manifest_sha256: String,
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    /// Context descriptor shared with HDB-11. HDB-15 has its own survey-method
    /// context and is instead bound through target class, survey unit and CRS.
    pub survey_context: SurveyContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb24TargetCoverageReceipt {
    pub event_id: String,
    pub target_class_id: String,
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    pub coverage_probability: Option<Hdb24ProbabilityBand>,
    pub model: Option<Hdb24CoverageProbabilityModel>,
    /// Digest of the target-location distribution (or explicit uniform-area
    /// assumption model) used for the intersection computation.
    pub target_location_distribution_sha256: Option<String>,
    pub coverage_union_geometry_sha256: String,
    pub spatial_union_receipt_sha256: String,
    pub intersection_receipt_ref: String,
    pub intersection_receipt_sha256: String,
    pub source_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub method_build_sha256: String,
    pub rights_status: Hdb24RightsStatus,
    pub rights_decision_id: String,
    pub rights_receipt_ref: String,
    pub access_tier: SurveyAccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb24AdapterInput {
    pub schema_version: String,
    pub composition_mode: Hdb24CompositionMode,
    pub context: Hdb24ContextBinding,
    /// Digest of the complete, canonical HDB-11A report bytes.
    pub hdb11_report_sha256: String,
    pub hdb11_report: HeritageObservabilityReport,
    pub hdb15: Hdb24Hdb15SourceBinding,
    pub target_coverage: Hdb24TargetCoverageReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hdb22ProjectedStage {
    FootprintCoverage,
    ConditionalDetection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hdb22ProjectedConditioning {
    CoveredGivenSurvivalExposureAndObservability,
    DetectedGivenSurvivalExposureObservabilityAndCoverage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb22StageProjection {
    pub stage: Hdb22ProjectedStage,
    pub probability: Hdb24ProbabilityBand,
    pub target_class_id: String,
    pub context_id: String,
    pub conditioning: Hdb22ProjectedConditioning,
    pub method_id: String,
    pub profile_sha256: String,
    pub method_build_sha256: String,
    pub calibration_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub source_data_sha256: String,
    pub rights_decision_id: String,
    pub analytical_rights_allowed: bool,
    pub rights_receipt_ref: String,
    pub access_tier: SurveyAccessTier,
    pub coverage_probability_model: Option<Hdb24CoverageProbabilityModel>,
    pub coverage_geometry_sha256: Option<String>,
    pub spatial_coverage_receipt_sha256: Option<String>,
    pub target_location_distribution_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hdb24CompositionReport {
    pub schema_version: String,
    pub composition_mode: Hdb24CompositionMode,
    pub event_id: String,
    pub candidate_id: String,
    pub target_class_id: String,
    pub context_id: String,
    pub context_manifest_sha256: String,
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    pub hdb11_report_sha256: String,
    pub hdb11_profile_sha256: String,
    pub hdb11_calibration_data_sha256: String,
    pub hdb11_calibration_split_manifest_sha256: String,
    pub hdb11_validation_receipt_sha256: String,
    pub hdb15_report_sha256: String,
    pub hdb15_profile_sha256: String,
    pub hdb15_calibration_split_manifest_sha256: String,
    pub hdb15_spatial_union_receipt_sha256: String,
    /// Kept for descriptive GIS QA only. It is never used as the target
    /// coverage probability in either stage projection.
    pub hdb15_area_fraction_descriptive_only: f64,
    pub target_coverage_probability: Hdb24ProbabilityBand,
    pub coverage_stage: Hdb22StageProjection,
    pub conditional_detection_stage: Hdb22StageProjection,
    pub area_fraction_used_as_probability: bool,
    pub repeated_visit_independence_assumed: bool,
    pub archaeological_absence_claimed: bool,
    pub archaeological_identification_claimed: bool,
    pub action_authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hdb24CompositionError {
    WrongSchemaVersion,
    CompositeModeUnsupportedByHdb22V01,
    EmptyBindingField(&'static str),
    InvalidDigest(&'static str),
    WrongHdb11SchemaVersion,
    WrongHdb11Disposition,
    WrongHdb11ObservationOutcome,
    Hdb11ContextNotQualified,
    Hdb11TargetClassMismatch,
    Hdb11CandidateMismatch,
    Hdb11EventMismatch,
    Hdb11ProfileScopeNotFactorized,
    Hdb11ConditioningSetMismatch,
    Hdb11RightsNotVerified,
    Hdb11CalibrationSampleInsufficient,
    WrongHdb15SchemaVersion,
    Hdb15ContextNotQualified,
    Hdb15RightsNotVerified,
    Hdb15TargetClassMismatch,
    Hdb15SurveyUnitMismatch,
    Hdb15CrsMismatch,
    Hdb15CoverageArithmeticMismatch,
    Hdb15SweepWidthMismatch,
    CoverageProbabilityMissing,
    CoverageModelMissing,
    TargetLocationDistributionMissing,
    TargetCoverageReceiptMismatch,
    TargetCoverageRightsNotVerified,
    InvalidProbabilityBand(String),
}

impl fmt::Display for Hdb24CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HDB-24 composition error: {self:?}")
    }
}

impl std::error::Error for Hdb24CompositionError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn require_digest(value: &str, label: &'static str) -> Result<(), Hdb24CompositionError> {
    if !valid_sha256(value) {
        return Err(Hdb24CompositionError::InvalidDigest(label));
    }
    Ok(())
}

fn require_text(value: &str, label: &'static str) -> Result<(), Hdb24CompositionError> {
    if !non_empty(value) {
        return Err(Hdb24CompositionError::EmptyBindingField(label));
    }
    Ok(())
}

fn approximately_equal(left: f64, right: f64) -> bool {
    let tolerance = (left.abs().max(right.abs()) * MAX_ABSOLUTE_ARITHMETIC_TOLERANCE)
        .max(MAX_ABSOLUTE_ARITHMETIC_TOLERANCE);
    (left - right).abs() <= tolerance
}

fn validate_context_binding(input: &Hdb24AdapterInput) -> Result<(), Hdb24CompositionError> {
    let context = &input.context;
    for (value, label) in [
        (context.event_id.as_str(), "event_id"),
        (context.candidate_id.as_str(), "candidate_id"),
        (context.target_class_id.as_str(), "target_class_id"),
        (context.context_id.as_str(), "context_id"),
        (context.survey_unit_id.as_str(), "survey_unit_id"),
        (context.survey_unit_crs_id.as_str(), "survey_unit_crs_id"),
    ] {
        require_text(value, label)?;
    }
    require_digest(&context.context_manifest_sha256, "context_manifest_sha256")?;
    if input.hdb11_report.candidate_id != context.candidate_id {
        return Err(Hdb24CompositionError::Hdb11CandidateMismatch);
    }
    if input.hdb11_report.observation_id.as_deref() != Some(context.event_id.as_str()) {
        return Err(Hdb24CompositionError::Hdb11EventMismatch);
    }
    if input.hdb11_report.calibration_context != context.survey_context
        || input.hdb11_report.observed_context != context.survey_context
    {
        return Err(Hdb24CompositionError::Hdb11ContextNotQualified);
    }
    if input.hdb11_report.calibration_context.feature_class_id != context.target_class_id {
        return Err(Hdb24CompositionError::Hdb11TargetClassMismatch);
    }
    Ok(())
}

fn validate_hdb11(input: &Hdb24AdapterInput) -> Result<(), Hdb24CompositionError> {
    let report = &input.hdb11_report;
    if report.schema_version != "luminous.heritage-observability.v0.2" {
        return Err(Hdb24CompositionError::WrongHdb11SchemaVersion);
    }
    require_digest(&input.hdb11_report_sha256, "hdb11_report_sha256")?;
    require_digest(&report.profile_sha256, "hdb11_profile_sha256")?;
    require_digest(&report.calibration_data_sha256, "hdb11_calibration_data_sha256")?;
    require_digest(
        &report.calibration_split_manifest_sha256,
        "hdb11_calibration_split_manifest_sha256",
    )?;
    require_digest(
        &report.validation_receipt_sha256,
        "hdb11_validation_receipt_sha256",
    )?;
    require_text(&report.profile_id, "hdb11_profile_id")?;
    require_text(&report.method_id, "hdb11_method_id")?;
    require_text(&report.validation_receipt_ref, "hdb11_validation_receipt_ref")?;
    require_text(&report.rights_decision_id, "hdb11_rights_decision_id")?;
    require_text(&report.rights_receipt_ref, "hdb11_rights_receipt_ref")?;

    if report.observation_outcome != ObservationOutcome::NotDetected {
        return Err(Hdb24CompositionError::WrongHdb11ObservationOutcome);
    }
    if report.disposition != ObservabilityDisposition::QualifiedNonDetection {
        return Err(Hdb24CompositionError::WrongHdb11Disposition);
    }
    if !report.context_match || !report.reasons.is_empty() {
        return Err(Hdb24CompositionError::Hdb11ContextNotQualified);
    }
    if report.archaeological_absence_claimed || report.archaeological_identification_claimed {
        return Err(Hdb24CompositionError::WrongHdb11Disposition);
    }
    if report.rights_status != RightsStatus::Verified || report.access_tier == SurveyAccessTier::Unknown {
        return Err(Hdb24CompositionError::Hdb11RightsNotVerified);
    }
    if report.calibration_eligible_independent_psu_count < report.minimum_independent_psus {
        return Err(Hdb24CompositionError::Hdb11CalibrationSampleInsufficient);
    }

    let expected = [
        DetectionCondition::TargetSurvived,
        DetectionCondition::TargetExposed,
        DetectionCondition::TargetObservable,
        DetectionCondition::TargetCoveredByActualFootprint,
    ];
    if report.detection_probability_scope
        != DetectionProbabilityScope::ConditionalOnTargetSurvivedExposedObservableAndCovered
    {
        return Err(Hdb24CompositionError::Hdb11ProfileScopeNotFactorized);
    }
    if report.conditioning_events != expected.to_vec() {
        return Err(Hdb24CompositionError::Hdb11ConditioningSetMismatch);
    }
    report
        .probability_of_detection_wilson_95
        .lower
        .is_finite()
        .then_some(())
        .ok_or(Hdb24CompositionError::InvalidProbabilityBand(
            "hdb11_wilson_interval".into(),
        ))?;
    Hdb24ProbabilityBand {
        lower: report.probability_of_detection_wilson_95.lower,
        upper: report.probability_of_detection_wilson_95.upper,
    }
    .validate("hdb11_wilson_interval")?;
    Ok(())
}

fn validate_hdb15(input: &Hdb24AdapterInput) -> Result<(), Hdb24CompositionError> {
    let source = &input.hdb15;
    if source.schema_version != HDB15_FIELD_SWEEP_REPORT_SCHEMA_VERSION {
        return Err(Hdb24CompositionError::WrongHdb15SchemaVersion);
    }
    for (value, label) in [
        (&source.report_sha256, "hdb15_report_sha256"),
        (&source.profile_sha256, "hdb15_profile_sha256"),
        (&source.calibration_data_sha256, "hdb15_calibration_data_sha256"),
        (
            &source.calibration_split_manifest_sha256,
            "hdb15_calibration_split_manifest_sha256",
        ),
        (&source.fit_receipt_sha256, "hdb15_fit_receipt_sha256"),
        (&source.transect_geometry_sha256, "hdb15_transect_geometry_sha256"),
        (&source.survey_unit_geometry_sha256, "hdb15_survey_unit_geometry_sha256"),
        (&source.coverage_union_geometry_sha256, "hdb15_coverage_union_geometry_sha256"),
        (&source.spatial_union_receipt_sha256, "hdb15_spatial_union_receipt_sha256"),
        (&source.spatial_union_method_build_sha256, "hdb15_spatial_union_method_build_sha256"),
    ] {
        require_digest(value, label)?;
    }
    for (value, label) in [
        (source.evaluation_id.as_str(), "hdb15_evaluation_id"),
        (source.profile_id.as_str(), "hdb15_profile_id"),
        (source.target_class_id.as_str(), "hdb15_target_class_id"),
        (source.survey_unit_id.as_str(), "hdb15_survey_unit_id"),
        (source.survey_unit_crs_id.as_str(), "hdb15_survey_unit_crs_id"),
        (source.spatial_union_method_id.as_str(), "hdb15_spatial_union_method_id"),
        (source.rights_decision_id.as_str(), "hdb15_rights_decision_id"),
        (source.rights_receipt_ref.as_str(), "hdb15_rights_receipt_ref"),
    ] {
        require_text(value, label)?;
    }
    if !source.context_match {
        return Err(Hdb24CompositionError::Hdb15ContextNotQualified);
    }
    if source.rights_status != Hdb24RightsStatus::Verified
        || source.access_tier == SurveyAccessTier::Unknown
    {
        return Err(Hdb24CompositionError::Hdb15RightsNotVerified);
    }
    if source.target_class_id != input.context.target_class_id {
        return Err(Hdb24CompositionError::Hdb15TargetClassMismatch);
    }
    if source.survey_unit_id != input.context.survey_unit_id {
        return Err(Hdb24CompositionError::Hdb15SurveyUnitMismatch);
    }
    if source.survey_unit_crs_id != input.context.survey_unit_crs_id {
        return Err(Hdb24CompositionError::Hdb15CrsMismatch);
    }
    if !source.survey_unit_area_m2.is_finite()
        || source.survey_unit_area_m2 <= 0.0
        || !source.receipt_reported_swept_area_m2.is_finite()
        || source.receipt_reported_swept_area_m2 <= 0.0
        || source.receipt_reported_swept_area_m2 > source.survey_unit_area_m2
        || !source.receipt_reported_coverage_fraction.is_finite()
        || !(0.0..=1.0).contains(&source.receipt_reported_coverage_fraction)
    {
        return Err(Hdb24CompositionError::Hdb15CoverageArithmeticMismatch);
    }
    let derived_fraction =
        source.receipt_reported_swept_area_m2 / source.survey_unit_area_m2;
    if !approximately_equal(derived_fraction, source.receipt_reported_coverage_fraction) {
        return Err(Hdb24CompositionError::Hdb15CoverageArithmeticMismatch);
    }
    if !source.effective_sweep_width_m.is_finite()
        || source.effective_sweep_width_m <= 0.0
        || !source.buffered_sweep_width_m.is_finite()
        || source.buffered_sweep_width_m <= 0.0
        || !approximately_equal(source.effective_sweep_width_m, source.buffered_sweep_width_m)
    {
        return Err(Hdb24CompositionError::Hdb15SweepWidthMismatch);
    }
    Ok(())
}

fn validate_target_coverage(input: &Hdb24AdapterInput) -> Result<(), Hdb24CompositionError> {
    let receipt = &input.target_coverage;
    require_text(&receipt.event_id, "target_coverage_event_id")?;
    require_text(&receipt.target_class_id, "target_coverage_target_class_id")?;
    require_text(&receipt.survey_unit_id, "target_coverage_survey_unit_id")?;
    require_text(&receipt.survey_unit_crs_id, "target_coverage_survey_unit_crs_id")?;
    require_text(&receipt.intersection_receipt_ref, "target_coverage_intersection_receipt_ref")?;
    require_text(&receipt.rights_decision_id, "target_coverage_rights_decision_id")?;
    require_text(&receipt.rights_receipt_ref, "target_coverage_rights_receipt_ref")?;
    for (value, label) in [
        (&receipt.coverage_union_geometry_sha256, "coverage_union_geometry_sha256"),
        (&receipt.spatial_union_receipt_sha256, "target_spatial_union_receipt_sha256"),
        (&receipt.intersection_receipt_sha256, "intersection_receipt_sha256"),
        (&receipt.source_data_sha256, "target_coverage_source_data_sha256"),
        (
            &receipt.validation_split_manifest_sha256,
            "target_coverage_validation_split_manifest_sha256",
        ),
        (&receipt.method_build_sha256, "target_coverage_method_build_sha256"),
    ] {
        require_digest(value, label)?;
    }
    if receipt.event_id != input.context.event_id
        || receipt.target_class_id != input.context.target_class_id
        || receipt.survey_unit_id != input.context.survey_unit_id
        || receipt.survey_unit_crs_id != input.context.survey_unit_crs_id
        || receipt.coverage_union_geometry_sha256
            != input.hdb15.coverage_union_geometry_sha256
        || receipt.spatial_union_receipt_sha256 != input.hdb15.spatial_union_receipt_sha256
    {
        return Err(Hdb24CompositionError::TargetCoverageReceiptMismatch);
    }
    if receipt.rights_status != Hdb24RightsStatus::Verified
        || receipt.access_tier == SurveyAccessTier::Unknown
    {
        return Err(Hdb24CompositionError::TargetCoverageRightsNotVerified);
    }
    let probability = receipt
        .coverage_probability
        .ok_or(Hdb24CompositionError::CoverageProbabilityMissing)?;
    probability.validate("target_coverage_probability")?;
    if receipt.model.is_none() {
        return Err(Hdb24CompositionError::CoverageModelMissing);
    }
    if receipt
        .target_location_distribution_sha256
        .as_deref()
        .is_none_or(|digest| !valid_sha256(digest))
    {
        return Err(Hdb24CompositionError::TargetLocationDistributionMissing);
    }
    Ok(())
}

fn project_coverage_stage(input: &Hdb24AdapterInput) -> Hdb22StageProjection {
    let receipt = &input.target_coverage;
    let source = &input.hdb15;
    Hdb22StageProjection {
        stage: Hdb22ProjectedStage::FootprintCoverage,
        probability: receipt.coverage_probability.expect("validated target coverage"),
        target_class_id: input.context.target_class_id.clone(),
        context_id: input.context.context_id.clone(),
        conditioning: Hdb22ProjectedConditioning::CoveredGivenSurvivalExposureAndObservability,
        method_id: source.spatial_union_method_id.clone(),
        profile_sha256: source.profile_sha256.clone(),
        method_build_sha256: receipt.method_build_sha256.clone(),
        calibration_data_sha256: source.calibration_data_sha256.clone(),
        validation_split_manifest_sha256: receipt.validation_split_manifest_sha256.clone(),
        validation_receipt_ref: receipt.intersection_receipt_ref.clone(),
        validation_receipt_sha256: receipt.intersection_receipt_sha256.clone(),
        source_data_sha256: receipt.source_data_sha256.clone(),
        rights_decision_id: receipt.rights_decision_id.clone(),
        analytical_rights_allowed: true,
        rights_receipt_ref: receipt.rights_receipt_ref.clone(),
        access_tier: receipt.access_tier,
        coverage_probability_model: receipt.model,
        coverage_geometry_sha256: Some(receipt.coverage_union_geometry_sha256.clone()),
        spatial_coverage_receipt_sha256: Some(receipt.spatial_union_receipt_sha256.clone()),
        target_location_distribution_sha256: receipt.target_location_distribution_sha256.clone(),
    }
}

fn project_detection_stage(input: &Hdb24AdapterInput) -> Hdb22StageProjection {
    let report = &input.hdb11_report;
    Hdb22StageProjection {
        stage: Hdb22ProjectedStage::ConditionalDetection,
        probability: Hdb24ProbabilityBand {
            lower: report.probability_of_detection_wilson_95.lower,
            upper: report.probability_of_detection_wilson_95.upper,
        },
        target_class_id: input.context.target_class_id.clone(),
        context_id: input.context.context_id.clone(),
        conditioning: Hdb22ProjectedConditioning::DetectedGivenSurvivalExposureObservabilityAndCoverage,
        method_id: report.method_id.clone(),
        profile_sha256: report.profile_sha256.clone(),
        method_build_sha256: report.calibration_context.processing_build_sha256.clone(),
        calibration_data_sha256: report.calibration_data_sha256.clone(),
        validation_split_manifest_sha256: report.calibration_split_manifest_sha256.clone(),
        validation_receipt_ref: report.validation_receipt_ref.clone(),
        validation_receipt_sha256: report.validation_receipt_sha256.clone(),
        source_data_sha256: report.calibration_data_sha256.clone(),
        rights_decision_id: report.rights_decision_id.clone(),
        analytical_rights_allowed: true,
        rights_receipt_ref: report.rights_receipt_ref.clone(),
        access_tier: report.access_tier,
        coverage_probability_model: None,
        coverage_geometry_sha256: None,
        spatial_coverage_receipt_sha256: None,
        target_location_distribution_sha256: None,
    }
}

/// Adapt HDB-11A conditional sensor detection and HDB-15 geometry plus a
/// separate target-location intersection receipt into the two HDB-22 v0.1
/// factorized stage projections. HDB-22 still requires independently supplied
/// survival, exposure, observability and false-positive stages.
pub fn adapt_hdb11_hdb15_to_hdb22(
    input: &Hdb24AdapterInput,
) -> Result<Hdb24CompositionReport, Hdb24CompositionError> {
    if input.schema_version != HERITAGE_COMPOSITION_SCHEMA_VERSION {
        return Err(Hdb24CompositionError::WrongSchemaVersion);
    }
    if input.composition_mode == Hdb24CompositionMode::CompositeSurveyUnit {
        return Err(Hdb24CompositionError::CompositeModeUnsupportedByHdb22V01);
    }

    validate_context_binding(input)?;
    validate_hdb11(input)?;
    validate_hdb15(input)?;
    validate_target_coverage(input)?;

    let coverage_stage = project_coverage_stage(input);
    let conditional_detection_stage = project_detection_stage(input);
    let target_coverage_probability = input
        .target_coverage
        .coverage_probability
        .expect("validated target coverage");

    Ok(Hdb24CompositionReport {
        schema_version: HERITAGE_COMPOSITION_SCHEMA_VERSION.to_owned(),
        composition_mode: Hdb24CompositionMode::FactorizedConditional,
        event_id: input.context.event_id.clone(),
        candidate_id: input.context.candidate_id.clone(),
        target_class_id: input.context.target_class_id.clone(),
        context_id: input.context.context_id.clone(),
        context_manifest_sha256: input.context.context_manifest_sha256.clone(),
        survey_unit_id: input.context.survey_unit_id.clone(),
        survey_unit_crs_id: input.context.survey_unit_crs_id.clone(),
        hdb11_report_sha256: input.hdb11_report_sha256.clone(),
        hdb11_profile_sha256: input.hdb11_report.profile_sha256.clone(),
        hdb11_calibration_data_sha256: input.hdb11_report.calibration_data_sha256.clone(),
        hdb11_calibration_split_manifest_sha256: input
            .hdb11_report
            .calibration_split_manifest_sha256
            .clone(),
        hdb11_validation_receipt_sha256: input.hdb11_report.validation_receipt_sha256.clone(),
        hdb15_report_sha256: input.hdb15.report_sha256.clone(),
        hdb15_profile_sha256: input.hdb15.profile_sha256.clone(),
        hdb15_calibration_split_manifest_sha256: input
            .hdb15
            .calibration_split_manifest_sha256
            .clone(),
        hdb15_spatial_union_receipt_sha256: input.hdb15.spatial_union_receipt_sha256.clone(),
        hdb15_area_fraction_descriptive_only: input.hdb15.receipt_reported_coverage_fraction,
        target_coverage_probability,
        coverage_stage,
        conditional_detection_stage,
        area_fraction_used_as_probability: false,
        repeated_visit_independence_assumed: false,
        archaeological_absence_claimed: false,
        archaeological_identification_claimed: false,
        action_authority: "none".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heritage_observability::{
        DetectionCondition, DetectionProbabilityScope, NonDetectionReason, SurveyAccessTier,
        SurveyContext, WilsonInterval,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn survey_context() -> SurveyContext {
        SurveyContext {
            feature_class_id: "dry_stone_wall".into(),
            modality: crate::heritage_observability::ObservationModality::AirborneLidar,
            sensor_configuration_sha256: digest('a'),
            processing_build_sha256: digest('b'),
            environment_stratum_id: "open_grassland".into(),
            season_stratum_id: "dry_season".into(),
            resolution_class_id: "dem_0p5m".into(),
        }
    }

    fn hdb11_report() -> HeritageObservabilityReport {
        HeritageObservabilityReport {
            schema_version: "luminous.heritage-observability.v0.2".into(),
            candidate_id: "candidate-001".into(),
            profile_id: "pod-profile-001".into(),
            profile_sha256: digest('c'),
            method_id: "lidar-dry-stonewall-classifier".into(),
            calibration_data_sha256: digest('d'),
            calibration_split_manifest_sha256: digest('e'),
            validation_receipt_ref: "urn:validation:pod-001".into(),
            validation_receipt_sha256: digest('f'),
            rights_decision_id: "rights:pod-001".into(),
            rights_receipt_ref: "urn:rights:pod-001".into(),
            access_tier: SurveyAccessTier::Controlled,
            calibration_context: survey_context(),
            observed_context: survey_context(),
            observation_profile_sha256: digest('c'),
            observation_id: Some("event-001".into()),
            observation_outcome: ObservationOutcome::NotDetected,
            disposition: ObservabilityDisposition::QualifiedNonDetection,
            detection_probability_scope: DetectionProbabilityScope::ConditionalOnTargetSurvivedExposedObservableAndCovered,
            conditioning_events: vec![
                DetectionCondition::TargetSurvived,
                DetectionCondition::TargetExposed,
                DetectionCondition::TargetObservable,
                DetectionCondition::TargetCoveredByActualFootprint,
            ],
            calibration_total_independent_psu_count: 25,
            calibration_eligible_independent_psu_count: 20,
            calibration_excluded_conditioning_psu_count: 5,
            calibration_detected_count: 18,
            empirical_probability_of_detection: 0.9,
            probability_of_detection_wilson_95: WilsonInterval {
                confidence_level: 0.95,
                z_value: 1.959963984540054,
                lower: 0.699,
                upper: 0.972,
            },
            minimum_independent_psus: 20,
            minimum_pod_lower_bound: 0.65,
            observed_coverage_fraction: 0.98,
            minimum_coverage_fraction: 0.95,
            observed_valid_data_fraction: 0.99,
            minimum_valid_data_fraction: 0.95,
            visibility: crate::heritage_observability::VisibilityClass::Adequate,
            context_match: true,
            rights_status: RightsStatus::Verified,
            reasons: Vec::<NonDetectionReason>::new(),
            archaeological_absence_claimed: false,
            archaeological_identification_claimed: false,
            combined_multi_observation_probability_reported: false,
        }
    }

    fn hdb15_source() -> Hdb24Hdb15SourceBinding {
        Hdb24Hdb15SourceBinding {
            report_sha256: digest('1'),
            schema_version: HDB15_FIELD_SWEEP_REPORT_SCHEMA_VERSION.into(),
            evaluation_id: "sweep-eval-001".into(),
            profile_id: "sweep-profile-001".into(),
            profile_sha256: digest('2'),
            calibration_data_sha256: digest('3'),
            calibration_split_manifest_sha256: digest('4'),
            fit_receipt_sha256: digest('5'),
            target_class_id: "dry_stone_wall".into(),
            survey_unit_id: "survey-unit-001".into(),
            survey_unit_crs_id: "EPSG:32735".into(),
            survey_unit_area_m2: 10_000.0,
            transect_geometry_sha256: digest('6'),
            survey_unit_geometry_sha256: digest('7'),
            coverage_union_geometry_sha256: digest('8'),
            spatial_union_receipt_sha256: digest('9'),
            spatial_union_method_id: "buffer-union-v1".into(),
            spatial_union_method_build_sha256: digest('a'),
            effective_sweep_width_m: 4.0,
            buffered_sweep_width_m: 4.0,
            receipt_reported_swept_area_m2: 6_000.0,
            receipt_reported_coverage_fraction: 0.6,
            context_match: true,
            rights_status: Hdb24RightsStatus::Verified,
            rights_decision_id: "rights:sweep-001".into(),
            rights_receipt_ref: "urn:rights:sweep-001".into(),
            access_tier: SurveyAccessTier::Controlled,
        }
    }

    fn target_coverage() -> Hdb24TargetCoverageReceipt {
        Hdb24TargetCoverageReceipt {
            event_id: "event-001".into(),
            target_class_id: "dry_stone_wall".into(),
            survey_unit_id: "survey-unit-001".into(),
            survey_unit_crs_id: "EPSG:32735".into(),
            coverage_probability: Some(Hdb24ProbabilityBand {
                lower: 0.72,
                upper: 0.84,
            }),
            model: Some(Hdb24CoverageProbabilityModel::TargetSpecificSpatialDistribution),
            target_location_distribution_sha256: Some(digest('b')),
            coverage_union_geometry_sha256: digest('8'),
            spatial_union_receipt_sha256: digest('9'),
            intersection_receipt_ref: "urn:target-intersection:001".into(),
            intersection_receipt_sha256: digest('c'),
            source_data_sha256: digest('b'),
            validation_split_manifest_sha256: digest('d'),
            method_build_sha256: digest('e'),
            rights_status: Hdb24RightsStatus::Verified,
            rights_decision_id: "rights:target-coverage-001".into(),
            rights_receipt_ref: "urn:rights:target-coverage-001".into(),
            access_tier: SurveyAccessTier::Controlled,
        }
    }

    fn input() -> Hdb24AdapterInput {
        Hdb24AdapterInput {
            schema_version: HERITAGE_COMPOSITION_SCHEMA_VERSION.into(),
            composition_mode: Hdb24CompositionMode::FactorizedConditional,
            context: Hdb24ContextBinding {
                event_id: "event-001".into(),
                candidate_id: "candidate-001".into(),
                target_class_id: "dry_stone_wall".into(),
                context_id: "site-context-001".into(),
                context_manifest_sha256: digest('f'),
                survey_unit_id: "survey-unit-001".into(),
                survey_unit_crs_id: "EPSG:32735".into(),
                survey_context: survey_context(),
            },
            hdb11_report_sha256: digest('0'),
            hdb11_report: hdb11_report(),
            hdb15: hdb15_source(),
            target_coverage: target_coverage(),
        }
    }

    #[test]
    fn factorized_conditional_inputs_produce_two_provenance_bound_hdb22_stage_projections() {
        let report = adapt_hdb11_hdb15_to_hdb22(&input()).unwrap();
        assert_eq!(report.composition_mode, Hdb24CompositionMode::FactorizedConditional);
        assert_eq!(report.coverage_stage.stage, Hdb22ProjectedStage::FootprintCoverage);
        assert_eq!(report.conditional_detection_stage.stage, Hdb22ProjectedStage::ConditionalDetection);
        assert_eq!(report.coverage_stage.probability.lower, 0.72);
        assert_eq!(report.conditional_detection_stage.probability.lower, 0.699);
        assert_eq!(report.hdb15_area_fraction_descriptive_only, 0.6);
        assert!(!report.area_fraction_used_as_probability);
        assert!(!report.repeated_visit_independence_assumed);
        assert!(!report.archaeological_absence_claimed);
        assert_eq!(report.action_authority, "none");
    }

    #[test]
    fn joint_per_survey_unit_pod_is_rejected_until_hdb22_has_composite_mode() {
        let mut input = input();
        input.composition_mode = Hdb24CompositionMode::CompositeSurveyUnit;
        input.hdb11_report.detection_probability_scope = DetectionProbabilityScope::JointPerSurveyUnit;
        input.hdb11_report.conditioning_events.clear();
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::CompositeModeUnsupportedByHdb22V01)
        );
    }

    #[test]
    fn hdb15_area_fraction_is_never_promoted_to_target_coverage_probability() {
        let mut input = input();
        input.target_coverage.coverage_probability = None;
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::CoverageProbabilityMissing)
        );
    }

    #[test]
    fn target_survey_unit_and_crs_must_match_exactly() {
        let mut input = input();
        input.target_coverage.survey_unit_id = "another-unit".into();
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::TargetCoverageReceiptMismatch)
        );

        let mut input = input();
        input.hdb15.survey_unit_crs_id = "EPSG:4326".into();
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::Hdb15CrsMismatch)
        );
    }

    #[test]
    fn invalid_coverage_fraction_arithmetic_does_not_become_probability() {
        let mut input = input();
        input.hdb15.receipt_reported_coverage_fraction = 0.72;
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::Hdb15CoverageArithmeticMismatch)
        );
    }

    #[test]
    fn unknown_rights_or_missing_target_location_model_is_rejected() {
        let mut input = input();
        input.target_coverage.rights_status = Hdb24RightsStatus::Unknown;
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::TargetCoverageRightsNotVerified)
        );

        let mut input = input();
        input.target_coverage.target_location_distribution_sha256 = None;
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::TargetLocationDistributionMissing)
        );
    }

    #[test]
    fn mismatched_hdb11_scope_and_indeterminate_non_detection_are_rejected() {
        let mut input = input();
        input.hdb11_report.detection_probability_scope = DetectionProbabilityScope::JointPerSurveyUnit;
        input.hdb11_report.conditioning_events.clear();
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::Hdb11ProfileScopeNotFactorized)
        );

        let mut input = input();
        input.hdb11_report.disposition = ObservabilityDisposition::IndeterminateNonDetection;
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::WrongHdb11Disposition)
        );
    }

    #[test]
    fn invalid_receipt_digests_fail_closed() {
        let mut input = input();
        input.hdb15.spatial_union_receipt_sha256 = "not-a-digest".into();
        assert_eq!(
            adapt_hdb11_hdb15_to_hdb22(&input),
            Err(Hdb24CompositionError::InvalidDigest("hdb15_spatial_union_receipt_sha256"))
        );
    }
}
