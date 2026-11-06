// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Factor-scope-safe adapter from HDB-11A and HDB-15 into HDB-22.
//!
//! v0.1 accepts only HDB-11A's POD conditional on target survival, exposure,
//! observability and actual footprint coverage. That estimand maps exactly to
//! HDB-22's ConditionalDetection stage. Joint POD, sensor-presentation scope,
//! and arbitrary custom scopes are rejected rather than approximated.
//!
//! HDB-15's swept-area fraction remains geometric. A separate target-location
//! probability receipt is mandatory before it can populate HDB-22 coverage.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::heritage_field_survey::{
    FieldSweepEvaluationReport, RightsStatus as FieldSurveyRightsStatus,
};
use super::heritage_non_detection::{
    AccessTier, AnalyticalRights, Conditioning, CoverageEvidenceBasis, CoverageProbabilityModel,
    EvidenceStage, HeritageNonDetectionError, HeritageNonDetectionInput,
    HeritageNonDetectionReport, ObservationState, ProbabilityBand, StageEvidence,
    evaluate_non_detection,
};
use super::heritage_observability::{
    DetectionCondition, DetectionProbabilityScope, HeritageObservabilityReport,
    ObservationModality, ObservationOutcome, ObservabilityDisposition, RightsStatus,
    NonDetectionReason,
};

pub const HERITAGE_FACTOR_ADAPTER_SCHEMA_VERSION: &str =
    "luminous.heritage-factor-adapter.v0.1";
const PROBABILITY_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterContextBinding {
    pub target_class_id: String,
    pub hdb22_context_id: String,
    pub hdb11_profile_sha256: String,
    pub hdb15_profile_sha256: String,
    /// Digest of the HDB-11 observed context serialized under the declared
    /// canonical context-mapping protocol.
    pub observability_context_sha256: String,
    /// Digest of the HDB-15 field-survey context under the same mapping protocol.
    pub field_survey_context_sha256: String,
    pub mapping_method_id: String,
    pub mapping_receipt_ref: String,
    pub mapping_receipt_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationProvenanceBinding {
    pub method_id: String,
    pub method_build_sha256: String,
    pub calibration_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub source_data_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_decision_id: String,
    pub analytical_rights: AnalyticalRights,
    pub rights_receipt_ref: String,
    pub access_tier: AccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetLocationCoverageReceipt {
    pub survey_unit_id: String,
    pub survey_unit_geometry_sha256: String,
    pub transect_geometry_sha256: String,
    pub coverage_geometry_sha256: String,
    pub spatial_union_receipt_sha256: String,
    pub model: CoverageProbabilityModel,
    pub target_location_probability: ProbabilityBand,
    pub field_sweep_fit_receipt_sha256: String,
    pub method_build_sha256: String,
    pub calibration_data_sha256: String,
    pub target_location_distribution_sha256: String,
    pub calculation_method_id: String,
    pub calculation_receipt_ref: String,
    pub calculation_receipt_sha256: String,
    pub source_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_decision_id: String,
    pub analytical_rights: AnalyticalRights,
    pub rights_receipt_ref: String,
    pub access_tier: AccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageFactorAdapterInput {
    pub schema_version: String,
    pub adapter_id: String,
    pub context_binding: AdapterContextBinding,
    pub hdb11_report: HeritageObservabilityReport,
    pub hdb11_provenance: CalibrationProvenanceBinding,
    pub hdb15_report: FieldSweepEvaluationReport,
    pub hdb15_provenance: CalibrationProvenanceBinding,
    pub target_location_coverage: TargetLocationCoverageReceipt,
    /// HDB-22 prior plus already-qualified Survival, Exposure, Observability
    /// and FalsePositive stages. The adapter supplies only Coverage and POD.
    pub non_detection_template: HeritageNonDetectionInput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageFactorAdapterReport {
    pub schema_version: String,
    pub adapter_id: String,
    pub factorization: String,
    pub hdb11_profile_sha256: String,
    pub hdb15_profile_sha256: String,
    pub target_location_receipt_sha256: String,
    pub target_location_probability: ProbabilityBand,
    pub detection_probability_wilson_95: ProbabilityBand,
    pub hdb15_geometric_coverage_fraction: f64,
    pub conditional_detection_stage_added: bool,
    pub footprint_coverage_stage_added: bool,
    pub joint_pod_used: bool,
    pub double_counting_guard: String,
    pub non_detection_input: HeritageNonDetectionInput,
    pub non_detection_report: HeritageNonDetectionReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageFactorAdapterError {
    WrongSchemaVersion,
    EmptyAdapterId,
    EmptyContextBinding,
    InvalidDigest(String),
    InvalidEvidenceReference(String),
    RightsNotAllowed(String),
    UnsupportedPodScope,
    PodObservationNotQualified,
    PodContextMismatch,
    PodContextNotFieldWalking,
    PodTargetClassMismatch,
    PodConditioningSetMismatch,
    PodMethodBuildMismatch,
    PodInsufficientEligibleCalibration,
    FieldSweepNotQualified,
    FieldSweepRightsNotVerified,
    FieldSweepTargetClassMismatch,
    FieldSweepContextDigestMismatch,
    FieldSweepProvenanceMismatch(String),
    SurveyUnitMismatch,
    SurveyGeometryMismatch,
    TransectGeometryMismatch,
    SpatialUnionReceiptMismatch,
    InvalidTargetLocationProbability,
    UniformLocationModelDoesNotMatchAreaFraction,
    TargetLocationProvenanceMismatch(String),
    TemplateMustBeSearchedNoDetection,
    TemplateTargetClassMismatch,
    TemplateContextMismatch,
    TemplateAlreadyContainsAdaptedStage(EvidenceStage),
    TemplateMissingUpstreamStage(EvidenceStage),
    NonDetectionEvaluation(HeritageNonDetectionError),
}

impl fmt::Display for HeritageFactorAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage factor adapter error: {self:?}")
    }
}

impl std::error::Error for HeritageFactorAdapterError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn require_digest(value: &str, label: &str) -> Result<(), HeritageFactorAdapterError> {
    if !valid_sha256(value) {
        return Err(HeritageFactorAdapterError::InvalidDigest(label.into()));
    }
    Ok(())
}

fn require_ref(value: &str, label: &str) -> Result<(), HeritageFactorAdapterError> {
    if !non_empty(value) {
        return Err(HeritageFactorAdapterError::InvalidEvidenceReference(
            label.into(),
        ));
    }
    Ok(())
}

fn validate_provenance(
    value: &CalibrationProvenanceBinding,
    label: &str,
) -> Result<(), HeritageFactorAdapterError> {
    require_ref(&value.method_id, &format!("{label}.method_id"))?;
    for (digest, field) in [
        (&value.method_build_sha256, "method_build_sha256"),
        (&value.calibration_data_sha256, "calibration_data_sha256"),
        (&value.validation_split_manifest_sha256, "validation_split_manifest_sha256"),
        (&value.source_data_sha256, "source_data_sha256"),
        (&value.validation_receipt_sha256, "validation_receipt_sha256"),
    ] {
        require_digest(digest, &format!("{label}.{field}"))?;
    }
    require_ref(
        &value.validation_receipt_ref,
        &format!("{label}.validation_receipt_ref"),
    )?;
    require_ref(&value.rights_decision_id, &format!("{label}.rights_decision_id"))?;
    if value.analytical_rights != AnalyticalRights::Allowed {
        return Err(HeritageFactorAdapterError::RightsNotAllowed(label.into()));
    }
    require_ref(&value.rights_receipt_ref, &format!("{label}.rights_receipt_ref"))?;
    Ok(())
}

fn validate_context_binding(
    binding: &AdapterContextBinding,
    hdb11: &HeritageObservabilityReport,
    hdb15: &FieldSweepEvaluationReport,
) -> Result<(), HeritageFactorAdapterError> {
    if !non_empty(&binding.target_class_id) || !non_empty(&binding.hdb22_context_id) {
        return Err(HeritageFactorAdapterError::EmptyContextBinding);
    }
    for (digest, label) in [
        (&binding.observability_context_sha256, "observability_context_sha256"),
        (&binding.field_survey_context_sha256, "field_survey_context_sha256"),
        (&binding.mapping_receipt_sha256, "mapping_receipt_sha256"),
    ] {
        require_digest(digest, label)?;
    }
    require_ref(&binding.mapping_method_id, "mapping_method_id")?;
    require_ref(&binding.mapping_receipt_ref, "mapping_receipt_ref")?;
    if binding.hdb11_profile_sha256 != hdb11.profile_sha256
        || binding.hdb15_profile_sha256 != hdb15.profile_sha256
    {
        return Err(HeritageFactorAdapterError::EmptyContextBinding);
    }
    Ok(())
}

fn validate_hdb11(
    report: &HeritageObservabilityReport,
    provenance: &CalibrationProvenanceBinding,
    binding: &AdapterContextBinding,
) -> Result<ProbabilityBand, HeritageFactorAdapterError> {
    if report.detection_probability_scope
        != DetectionProbabilityScope::ConditionalOnTargetSurvivedExposedObservableAndCovered
    {
        return Err(HeritageFactorAdapterError::UnsupportedPodScope);
    }
    let expected_events = vec![
        DetectionCondition::TargetSurvived,
        DetectionCondition::TargetExposed,
        DetectionCondition::TargetObservable,
        DetectionCondition::TargetCoveredByActualFootprint,
    ];
    if report.conditioning_events != expected_events {
        return Err(HeritageFactorAdapterError::PodConditioningSetMismatch);
    }
    if report.observation_outcome != ObservationOutcome::NotDetected
        || report.disposition != ObservabilityDisposition::QualifiedNonDetection
        || !report.reasons.is_empty()
        || !report.context_match
        || report.profile_sha256 != report.observation_profile_sha256
    {
        return Err(HeritageFactorAdapterError::PodObservationNotQualified);
    }
    if report.calibration_eligible_independent_psu_count < report.minimum_independent_psus
        || report.calibration_detected_count > report.calibration_eligible_independent_psu_count
        || report.calibration_total_independent_psu_count
            != report.calibration_eligible_independent_psu_count
                + report.calibration_excluded_conditioning_psu_count
        || report.calibration_eligible_independent_psu_count == 0
    {
        return Err(HeritageFactorAdapterError::PodInsufficientEligibleCalibration);
    }
    let pod_interval = report.probability_of_detection_wilson_95;
    if pod_interval.confidence_level != 0.95
        || !pod_interval.lower.is_finite()
        || !pod_interval.upper.is_finite()
        || pod_interval.lower < 0.0
        || pod_interval.upper > 1.0
        || pod_interval.lower > pod_interval.upper
        || !report.empirical_probability_of_detection.is_finite()
        || !(0.0..=1.0).contains(&report.empirical_probability_of_detection)
    {
        return Err(HeritageFactorAdapterError::PodObservationNotQualified);
    }
    if report.observed_context.modality != ObservationModality::FieldWalking {
        return Err(HeritageFactorAdapterError::PodContextNotFieldWalking);
    }
    if report.observed_context.feature_class_id != binding.target_class_id {
        return Err(HeritageFactorAdapterError::PodTargetClassMismatch);
    }
    if report.observed_context.processing_build_sha256 != provenance.method_build_sha256 {
        return Err(HeritageFactorAdapterError::PodMethodBuildMismatch);
    }
    if report.rights_status != RightsStatus::Verified {
        return Err(HeritageFactorAdapterError::RightsNotAllowed("hdb11".into()));
    }
    if provenance.access_tier == AccessTier::Unknown {
        return Err(HeritageFactorAdapterError::RightsNotAllowed(
            "hdb11_access_tier".into(),
        ));
    }
    require_digest(&report.profile_sha256, "hdb11_profile_sha256")?;
    require_digest(&provenance.method_build_sha256, "hdb11_method_build_sha256")?;
    require_digest(&provenance.calibration_data_sha256, "hdb11_calibration_data_sha256")?;
    require_digest(
        &provenance.validation_split_manifest_sha256,
        "hdb11_validation_split_manifest_sha256",
    )?;

    Ok(ProbabilityBand {
        lower: report.probability_of_detection_wilson_95.lower,
        upper: report.probability_of_detection_wilson_95.upper,
    })
}

fn validate_hdb15(
    report: &FieldSweepEvaluationReport,
    provenance: &CalibrationProvenanceBinding,
    binding: &AdapterContextBinding,
) -> Result<(), HeritageFactorAdapterError> {
    if !report.context_match
        || report.archaeological_absence_claimed
        || report.archaeological_identification_claimed
        || report.geomorphological_surface_survival_estimated
        || report.repeated_search_independence_assumed
    {
        return Err(HeritageFactorAdapterError::FieldSweepNotQualified);
    }
    if report.rights_status != FieldSurveyRightsStatus::Verified
        || provenance.access_tier == AccessTier::Unknown
    {
        return Err(HeritageFactorAdapterError::FieldSweepRightsNotVerified);
    }
    require_digest(&report.profile_sha256, "hdb15_profile_sha256")?;
    if report.profile_sha256 != binding.hdb15_profile_sha256 {
        return Err(HeritageFactorAdapterError::FieldSweepProvenanceMismatch(
            "profile digest does not match context binding".into(),
        ));
    }
    require_digest(&report.calibration_data_sha256, "hdb15_calibration_data_sha256")?;
    require_digest(
        &report.calibration_split_manifest_sha256,
        "hdb15_calibration_split_manifest_sha256",
    )?;
    require_digest(&report.fit_receipt_sha256, "hdb15_fit_receipt_sha256")?;
    require_digest(
        &report.survey_unit_geometry_sha256,
        "hdb15_survey_unit_geometry_sha256",
    )?;
    require_digest(&report.transect_geometry_sha256, "hdb15_transect_geometry_sha256")?;
    require_digest(
        &report.spatial_union_receipt_sha256,
        "hdb15_spatial_union_receipt_sha256",
    )?;
    if report.receipt_reported_coverage_fraction <= 0.0
        || report.receipt_reported_coverage_fraction > 1.0
        || !report.receipt_reported_coverage_fraction.is_finite()
        || !report.effective_sweep_width_m.is_finite()
        || report.effective_sweep_width_m <= 0.0
        || !report.buffered_sweep_width_m.is_finite()
        || (report.effective_sweep_width_m - report.buffered_sweep_width_m).abs()
            > (report.effective_sweep_width_m.abs() * PROBABILITY_TOLERANCE)
                .max(PROBABILITY_TOLERANCE)
    {
        return Err(HeritageFactorAdapterError::FieldSweepNotQualified);
    }
    if provenance.calibration_data_sha256 != report.calibration_data_sha256
        || provenance.validation_split_manifest_sha256
            != report.calibration_split_manifest_sha256
        || provenance.validation_receipt_sha256 != report.fit_receipt_sha256
    {
        return Err(HeritageFactorAdapterError::FieldSweepProvenanceMismatch(
            "profile/fit receipt binding".into(),
        ));
    }
    if !non_empty(&provenance.method_id) || !non_empty(&provenance.validation_receipt_ref) {
        return Err(HeritageFactorAdapterError::FieldSweepProvenanceMismatch(
            "method/fit receipt reference".into(),
        ));
    }
    require_digest(&provenance.method_build_sha256, "hdb15_method_build_sha256")?;
    require_digest(&provenance.source_data_sha256, "hdb15_source_data_sha256")?;
    require_digest(&provenance.validation_receipt_sha256, "hdb15_validation_receipt_sha256")?;

    // The context mapping manifest binds the non-identical HDB-11/HDB-15
    // context representations. The method-level contract requires exact
    // target feature-class identity on both sides.
    if binding.target_class_id.is_empty() {
        return Err(HeritageFactorAdapterError::FieldSweepTargetClassMismatch);
    }
    Ok(())
}

fn validate_target_location_coverage(
    report: &FieldSweepEvaluationReport,
    receipt: &TargetLocationCoverageReceipt,
) -> Result<ProbabilityBand, HeritageFactorAdapterError> {
    if !receipt.target_location_probability.lower.is_finite()
        || !receipt.target_location_probability.upper.is_finite()
        || receipt.target_location_probability.lower < 0.0
        || receipt.target_location_probability.upper > 1.0
        || receipt.target_location_probability.lower > receipt.target_location_probability.upper
    {
        return Err(HeritageFactorAdapterError::InvalidTargetLocationProbability);
    }
    for (digest, label) in [
        (&receipt.survey_unit_geometry_sha256, "target_survey_unit_geometry_sha256"),
        (&receipt.transect_geometry_sha256, "target_transect_geometry_sha256"),
        (&receipt.coverage_geometry_sha256, "coverage_geometry_sha256"),
        (&receipt.spatial_union_receipt_sha256, "target_spatial_union_receipt_sha256"),
        (
            &receipt.target_location_distribution_sha256,
            "target_location_distribution_sha256",
        ),
        (&receipt.field_sweep_fit_receipt_sha256, "field_sweep_fit_receipt_sha256"),
        (&receipt.method_build_sha256, "target_location_method_build_sha256"),
        (&receipt.calibration_data_sha256, "target_location_calibration_data_sha256"),
        (&receipt.calculation_receipt_sha256, "calculation_receipt_sha256"),
        (&receipt.source_data_sha256, "target_location_source_data_sha256"),
        (
            &receipt.validation_split_manifest_sha256,
            "target_location_validation_split_manifest_sha256",
        ),
        (
            &receipt.validation_receipt_sha256,
            "target_location_validation_receipt_sha256",
        ),
    ] {
        require_digest(digest, label)?;
    }
    for (value, label) in [
        (&receipt.survey_unit_id, "survey_unit_id"),
        (&receipt.calculation_method_id, "calculation_method_id"),
        (&receipt.calculation_receipt_ref, "calculation_receipt_ref"),
        (
            &receipt.validation_receipt_ref,
            "target_location_validation_receipt_ref",
        ),
        (
            &receipt.rights_receipt_ref,
            "target_location_rights_receipt_ref",
        ),
    ] {
        require_ref(value, label)?;
    }
    if receipt.survey_unit_id != report.survey_unit_id {
        return Err(HeritageFactorAdapterError::SurveyUnitMismatch);
    }
    if receipt.survey_unit_geometry_sha256 != report.survey_unit_geometry_sha256 {
        return Err(HeritageFactorAdapterError::SurveyGeometryMismatch);
    }
    if receipt.transect_geometry_sha256 != report.transect_geometry_sha256 {
        return Err(HeritageFactorAdapterError::TransectGeometryMismatch);
    }
    if receipt.spatial_union_receipt_sha256 != report.spatial_union_receipt_sha256 {
        return Err(HeritageFactorAdapterError::SpatialUnionReceiptMismatch);
    }
    if receipt.field_sweep_fit_receipt_sha256 != report.fit_receipt_sha256
        || receipt.calibration_data_sha256 != report.calibration_data_sha256
    {
        return Err(
            HeritageFactorAdapterError::TargetLocationProvenanceMismatch(
                "HDB-15 fit/calibration binding".into(),
            ),
        );
    }
    if receipt.access_tier == AccessTier::Unknown
        || receipt.analytical_rights != AnalyticalRights::Allowed
    {
        return Err(HeritageFactorAdapterError::RightsNotAllowed(
            "target_location_access_tier_or_rights".into(),
        ));
    }
    require_ref(
        &receipt.rights_decision_id,
        "target_location_rights_decision_id",
    )?;
    require_ref(
        &receipt.rights_receipt_ref,
        "target_location_rights_receipt_ref",
    )?;
    if receipt.model == CoverageProbabilityModel::UniformAreaWeightedWithinSurveyUnit {
        let observed = report.receipt_reported_coverage_fraction;
        if (receipt.target_location_probability.lower - observed).abs() > PROBABILITY_TOLERANCE
            || (receipt.target_location_probability.upper - observed).abs() > PROBABILITY_TOLERANCE
        {
            return Err(HeritageFactorAdapterError::UniformLocationModelDoesNotMatchAreaFraction);
        }
    }
    Ok(receipt.target_location_probability)
}

fn make_stage(
    stage: EvidenceStage,
    probability: ProbabilityBand,
    target_class_id: &str,
    context_id: &str,
    conditioning: Conditioning,
    provenance: &CalibrationProvenanceBinding,
    profile_sha256: &str,
    calibration_data_sha256: &str,
    validation_split_manifest_sha256: &str,
    coverage: Option<&TargetLocationCoverageReceipt>,
    hdb15_report: Option<&FieldSweepEvaluationReport>,
) -> StageEvidence {
    let coverage_model = coverage.map(|value| value.model);
    StageEvidence {
        stage,
        probability,
        target_class_id: target_class_id.into(),
        context_id: context_id.into(),
        conditioning,
        method_id: coverage
            .map(|value| value.calculation_method_id.clone())
            .unwrap_or_else(|| provenance.method_id.clone()),
        profile_sha256: profile_sha256.into(),
        method_build_sha256: coverage
            .map(|value| value.method_build_sha256.clone())
            .unwrap_or_else(|| provenance.method_build_sha256.clone()),
        calibration_data_sha256: coverage
            .map(|value| value.calibration_data_sha256.clone())
            .unwrap_or_else(|| calibration_data_sha256.into()),
        validation_split_manifest_sha256: coverage
            .map(|value| value.validation_split_manifest_sha256.clone())
            .unwrap_or_else(|| validation_split_manifest_sha256.into()),
        validation_receipt_ref: coverage
            .map(|value| value.calculation_receipt_ref.clone())
            .unwrap_or_else(|| provenance.validation_receipt_ref.clone()),
        validation_receipt_sha256: coverage
            .map(|value| value.calculation_receipt_sha256.clone())
            .unwrap_or_else(|| provenance.validation_receipt_sha256.clone()),
        source_data_sha256: coverage
            .map(|value| value.source_data_sha256.clone())
            .unwrap_or_else(|| provenance.source_data_sha256.clone()),
        analytical_rights: AnalyticalRights::Allowed,
        rights_receipt_ref: coverage
            .map(|value| value.rights_receipt_ref.clone())
            .unwrap_or_else(|| provenance.rights_receipt_ref.clone()),
        access_tier: coverage
            .map(|value| value.access_tier)
            .unwrap_or(provenance.access_tier),
        coverage_evidence_basis: coverage.map(|_| CoverageEvidenceBasis::ActualSpatialUnionReceipt),
        coverage_geometry_sha256: coverage.map(|value| value.coverage_geometry_sha256.clone()),
        spatial_coverage_receipt_sha256: hdb15_report
            .map(|value| value.spatial_union_receipt_sha256.clone()),
        coverage_probability_model: coverage_model,
        target_location_distribution_sha256: coverage
            .map(|value| value.target_location_distribution_sha256.clone()),
    }
}

/// Validates the factorization and creates the exact HDB-22 stage input.
///
/// The caller supplies HDB-22's separately qualified Survival, Exposure,
/// Observability and FalsePositive stages. This adapter supplies only the two
/// factors whose source semantics are bound to HDB-11A and HDB-15.
pub fn adapt_heritage_detection_factors(
    input: &HeritageFactorAdapterInput,
) -> Result<HeritageFactorAdapterReport, HeritageFactorAdapterError> {
    if input.schema_version != HERITAGE_FACTOR_ADAPTER_SCHEMA_VERSION {
        return Err(HeritageFactorAdapterError::WrongSchemaVersion);
    }
    if !non_empty(&input.adapter_id) {
        return Err(HeritageFactorAdapterError::EmptyAdapterId);
    }
    validate_context_binding(
        &input.context_binding,
        &input.hdb11_report,
        &input.hdb15_report,
    )?;
    validate_provenance(&input.hdb11_provenance, "hdb11")?;
    validate_provenance(&input.hdb15_provenance, "hdb15")?;

    let template = &input.non_detection_template;
    if template.observation_state != ObservationState::SearchedNoDetection {
        return Err(HeritageFactorAdapterError::TemplateMustBeSearchedNoDetection);
    }
    if template.target_class_id != input.context_binding.target_class_id
        || input.hdb11_report.observed_context.feature_class_id != template.target_class_id
    {
        return Err(HeritageFactorAdapterError::TemplateTargetClassMismatch);
    }
    if template.context_id != input.context_binding.hdb22_context_id {
        return Err(HeritageFactorAdapterError::TemplateContextMismatch);
    }
    for stage in &template.stage_evidence {
        if matches!(
            stage.stage,
            EvidenceStage::FootprintCoverage | EvidenceStage::ConditionalDetection
        ) {
            return Err(
                HeritageFactorAdapterError::TemplateAlreadyContainsAdaptedStage(stage.stage),
            );
        }
    }
    for expected in [
        EvidenceStage::Survival,
        EvidenceStage::Exposure,
        EvidenceStage::Observability,
        EvidenceStage::FalsePositive,
    ] {
        if !template
            .stage_evidence
            .iter()
            .any(|stage| stage.stage == expected)
        {
            return Err(HeritageFactorAdapterError::TemplateMissingUpstreamStage(
                expected,
            ));
        }
    }

    let detection_probability = validate_hdb11(
        &input.hdb11_report,
        &input.hdb11_provenance,
        &input.context_binding,
    )?;
    validate_hdb15(
        &input.hdb15_report,
        &input.hdb15_provenance,
        &input.context_binding,
    )?;
    let coverage_probability =
        validate_target_location_coverage(&input.hdb15_report, &input.target_location_coverage)?;

    let context_id = &template.context_id;
    let target_class_id = &template.target_class_id;

    let hdb15_coverage_stage = make_stage(
        EvidenceStage::FootprintCoverage,
        coverage_probability,
        target_class_id,
        context_id,
        Conditioning::CoveredGivenSurvivalExposureAndObservability,
        &input.hdb15_provenance,
        &input.hdb15_report.profile_sha256,
        &input.hdb15_report.calibration_data_sha256,
        &input.hdb15_report.calibration_split_manifest_sha256,
        Some(&input.target_location_coverage),
        Some(&input.hdb15_report),
    );

    let hdb11_detection_stage = make_stage(
        EvidenceStage::ConditionalDetection,
        detection_probability,
        target_class_id,
        context_id,
        Conditioning::DetectedGivenSurvivalExposureObservabilityAndCoverage,
        &input.hdb11_provenance,
        &input.hdb11_report.profile_sha256,
        &input.hdb11_provenance.calibration_data_sha256,
        &input.hdb11_provenance.validation_split_manifest_sha256,
        None,
        None,
    );

    let mut adapted = template.clone();
    adapted.stage_evidence.push(hdb15_coverage_stage);
    adapted.stage_evidence.push(hdb11_detection_stage);

    let evaluated = evaluate_non_detection(&adapted)
        .map_err(HeritageFactorAdapterError::NonDetectionEvaluation)?;

    Ok(HeritageFactorAdapterReport {
        schema_version: HERITAGE_FACTOR_ADAPTER_SCHEMA_VERSION.into(),
        adapter_id: input.adapter_id.clone(),
        factorization: "factorized_survival_exposure_observability_coverage_then_conditional_sensor_detection"
            .into(),
        hdb11_profile_sha256: input.hdb11_report.profile_sha256.clone(),
        hdb15_profile_sha256: input.hdb15_report.profile_sha256.clone(),
        target_location_receipt_sha256: input.target_location_coverage.calculation_receipt_sha256.clone(),
        target_location_probability: coverage_probability,
        detection_probability_wilson_95: detection_probability,
        hdb15_geometric_coverage_fraction: input.hdb15_report.receipt_reported_coverage_fraction,
        conditional_detection_stage_added: true,
        footprint_coverage_stage_added: true,
        joint_pod_used: false,
        double_counting_guard:
            "hdb11_pod_is_conditioned_on_survival_exposure_observability_and_actual_coverage; hdb15_area_fraction_is_not_used_as_probability_without_target_location_receipt"
                .into(),
        non_detection_input: adapted,
        non_detection_report: evaluated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heritage_field_survey::SweepWidthInterval;
    use crate::heritage_non_detection::{PriorEvidence, StageEvidence};
    use crate::heritage_observability::{
        NonDetectionReason, ObservationModality, RightsStatus, SurveyContext, VisibilityClass,
        WilsonInterval,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn hdb11_context() -> crate::heritage_observability::SurveyContext {
        SurveyContext {
            feature_class_id: "dry_stone_wall".into(),
            modality: ObservationModality::FieldWalking,
            sensor_configuration_sha256: digest('a'),
            processing_build_sha256: digest('b'),
            environment_stratum_id: "open_grassland".into(),
            season_stratum_id: "dry_season".into(),
            resolution_class_id: "fieldwalking_visual".into(),
        }
    }

    fn hdb11_report() -> HeritageObservabilityReport {
        HeritageObservabilityReport {
            schema_version: "luminous.heritage-observability.v0.2".into(),
            candidate_id: "candidate-1".into(),
            profile_id: "pod-profile".into(),
            profile_sha256: digest('c'),
            calibration_context: hdb11_context(),
            observed_context: hdb11_context(),
            observation_profile_sha256: digest('c'),
            observation_id: Some("obs-1".into()),
            observation_outcome: ObservationOutcome::NotDetected,
            disposition: ObservabilityDisposition::QualifiedNonDetection,
            detection_probability_scope:
                DetectionProbabilityScope::ConditionalOnTargetSurvivedExposedObservableAndCovered,
            conditioning_events: vec![
                DetectionCondition::TargetSurvived,
                DetectionCondition::TargetExposed,
                DetectionCondition::TargetObservable,
                DetectionCondition::TargetCoveredByActualFootprint,
            ],
            calibration_total_independent_psu_count: 25,
            calibration_eligible_independent_psu_count: 20,
            calibration_excluded_conditioning_psu_count: 5,
            calibration_detected_count: 20,
            empirical_probability_of_detection: 1.0,
            probability_of_detection_wilson_95: WilsonInterval {
                confidence_level: 0.95,
                z_value: 1.959_963_984_540_054,
                lower: 0.838_874_841_947_311_1,
                upper: 1.0,
            },
            minimum_independent_psus: 20,
            minimum_pod_lower_bound: 0.80,
            observed_coverage_fraction: 0.97,
            minimum_coverage_fraction: 0.95,
            observed_valid_data_fraction: 0.99,
            minimum_valid_data_fraction: 0.95,
            visibility: VisibilityClass::Adequate,
            context_match: true,
            rights_status: RightsStatus::Verified,
            reasons: Vec::<NonDetectionReason>::new(),
            archaeological_absence_claimed: false,
            archaeological_identification_claimed: false,
            combined_multi_observation_probability_reported: false,
        }
    }

    fn hdb15_report() -> FieldSweepEvaluationReport {
        FieldSweepEvaluationReport {
            schema_version: "luminous.heritage-field-sweep-report.v0.1".into(),
            evaluation_id: "sweep-eval-1".into(),
            profile_id: "field-profile".into(),
            profile_sha256: digest('d'),
            calibration_data_sha256: digest('e'),
            calibration_split_manifest_sha256: digest('f'),
            fit_receipt_sha256: digest('1'),
            survey_unit_id: "survey-unit-1".into(),
            transect_geometry_sha256: digest('2'),
            survey_unit_geometry_sha256: digest('3'),
            spatial_union_receipt_sha256: digest('4'),
            context_match: true,
            effective_sweep_width_m: 1.5,
            effective_sweep_width_interval_95: SweepWidthInterval {
                confidence_level: 0.95,
                z_value: 1.959_963_984_540_054,
                standard_error_m: 0.1,
                lower_m: 1.304,
                upper_m: 1.696,
            },
            buffered_sweep_width_m: 1.5,
            receipt_reported_swept_area_m2: 9700.0,
            receipt_reported_coverage_fraction: 0.97,
            validation_brier_score: 0.08,
            fit_transect_count: 3,
            validation_transect_count: 2,
            fit_seeded_target_count: 20,
            validation_seeded_target_count: 10,
            rights_status: FieldSurveyRightsStatus::Verified,
            archaeological_absence_claimed: false,
            archaeological_identification_claimed: false,
            geomorphological_surface_survival_estimated: false,
            repeated_search_independence_assumed: false,
        }
    }

    fn provenance(
        method_id: &str,
        build: char,
        data: char,
        validation: char,
    ) -> CalibrationProvenanceBinding {
        CalibrationProvenanceBinding {
            method_id: method_id.into(),
            method_build_sha256: digest(build),
            calibration_data_sha256: digest('9'),
            validation_split_manifest_sha256: digest('8'),
            source_data_sha256: digest(data),
            validation_receipt_ref: format!("receipt:{method_id}"),
            validation_receipt_sha256: digest(validation),
            rights_decision_id: format!("rights-decision:{method_id}"),
            analytical_rights: AnalyticalRights::Allowed,
            rights_receipt_ref: format!("rights:{method_id}"),
            access_tier: AccessTier::Controlled,
        }
    }

    fn hdb22_stage(stage: EvidenceStage, probability: f64) -> StageEvidence {
        let conditioning = match stage {
            EvidenceStage::Survival => Conditioning::OriginallyPresent,
            EvidenceStage::Exposure => Conditioning::SurvivedGivenOriginallyPresent,
            EvidenceStage::Observability => Conditioning::ObservableGivenSurvivalAndExposure,
            EvidenceStage::FalsePositive => Conditioning::FalsePositiveGivenOriginallyAbsent,
            _ => unreachable!(),
        };
        StageEvidence {
            stage,
            probability: ProbabilityBand {
                lower: probability,
                upper: probability,
            },
            target_class_id: "dry_stone_wall".into(),
            context_id: "context-1".into(),
            conditioning,
            method_id: format!("method:{stage:?}"),
            profile_sha256: digest('5'),
            method_build_sha256: digest('6'),
            calibration_data_sha256: digest('7'),
            validation_split_manifest_sha256: digest('8'),
            validation_receipt_ref: format!("stage-receipt:{stage:?}"),
            validation_receipt_sha256: digest('9'),
            source_data_sha256: digest('a'),
            analytical_rights: AnalyticalRights::Allowed,
            rights_receipt_ref: format!("stage-rights:{stage:?}"),
            access_tier: AccessTier::Controlled,
            coverage_evidence_basis: None,
            coverage_geometry_sha256: None,
            spatial_coverage_receipt_sha256: None,
            coverage_probability_model: None,
            target_location_distribution_sha256: None,
        }
    }

    fn input() -> HeritageFactorAdapterInput {
        let hdb15 = hdb15_report();
        HeritageFactorAdapterInput {
            schema_version: HERITAGE_FACTOR_ADAPTER_SCHEMA_VERSION.into(),
            adapter_id: "adapter-test-1".into(),
            context_binding: AdapterContextBinding {
                target_class_id: "dry_stone_wall".into(),
                hdb22_context_id: "context-1".into(),
                hdb11_profile_sha256: digest('c'),
                hdb15_profile_sha256: digest('d'),
                observability_context_sha256: digest('b'),
                field_survey_context_sha256: digest('c'),
                mapping_method_id: "explicit-context-map-v1".into(),
                mapping_receipt_ref: "receipt:context-map".into(),
                mapping_receipt_sha256: digest('d'),
            },
            hdb11_report: hdb11_report(),
            hdb11_provenance: provenance("hdb11-fieldwalking-pod", 'b', 'e', 'f'),
            hdb15_report: hdb15.clone(),
            hdb15_provenance: CalibrationProvenanceBinding {
                method_id: "hdb15-sweep-width".into(),
                method_build_sha256: digest('a'),
                calibration_data_sha256: hdb15.calibration_data_sha256.clone(),
                validation_split_manifest_sha256: hdb15.calibration_split_manifest_sha256.clone(),
                source_data_sha256: digest('b'),
                validation_receipt_ref: "receipt:hdb15-fit".into(),
                validation_receipt_sha256: hdb15.fit_receipt_sha256.clone(),
                rights_decision_id: "rights-decision:hdb15".into(),
                analytical_rights: AnalyticalRights::Allowed,
                rights_receipt_ref: "rights:hdb15".into(),
                access_tier: AccessTier::Controlled,
            },
            target_location_coverage: TargetLocationCoverageReceipt {
                survey_unit_id: hdb15.survey_unit_id.clone(),
                survey_unit_geometry_sha256: hdb15.survey_unit_geometry_sha256.clone(),
                transect_geometry_sha256: hdb15.transect_geometry_sha256.clone(),
                coverage_geometry_sha256: digest('a'),
                spatial_union_receipt_sha256: hdb15.spatial_union_receipt_sha256.clone(),
                model: CoverageProbabilityModel::UniformAreaWeightedWithinSurveyUnit,
                target_location_probability: ProbabilityBand {
                    lower: 0.97,
                    upper: 0.97,
                },
                field_sweep_fit_receipt_sha256: hdb15.fit_receipt_sha256.clone(),
                method_build_sha256: digest('a'),
                calibration_data_sha256: hdb15.calibration_data_sha256.clone(),
                target_location_distribution_sha256: digest('b'),
                calculation_method_id: "uniform-area-location-model".into(),
                calculation_receipt_ref: "receipt:location-probability".into(),
                calculation_receipt_sha256: digest('c'),
                source_data_sha256: digest('d'),
                validation_split_manifest_sha256: digest('e'),
                validation_receipt_ref: "receipt:location-validation".into(),
                validation_receipt_sha256: digest('f'),
                rights_decision_id: "rights-decision:location".into(),
                analytical_rights: AnalyticalRights::Allowed,
                rights_receipt_ref: "rights:location".into(),
                access_tier: AccessTier::Controlled,
            },
            non_detection_template: HeritageNonDetectionInput {
                schema_version: "luminous.heritage-nondetection.v0.1".into(),
                event_id: "event-1".into(),
                target_class_id: "dry_stone_wall".into(),
                context_id: "context-1".into(),
                observation_state: ObservationState::SearchedNoDetection,
                prior: PriorEvidence {
                    target_class_id: "dry_stone_wall".into(),
                    context_id: "context-1".into(),
                    probability: ProbabilityBand {
                        lower: 0.6,
                        upper: 0.6,
                    },
                    prior_profile_sha256: digest('1'),
                    source_data_sha256: digest('2'),
                    validation_split_manifest_sha256: digest('3'),
                    validation_receipt_ref: "receipt:prior".into(),
                    validation_receipt_sha256: digest('4'),
                    rights_decision_id: "rights:prior".into(),
                    analytical_rights: AnalyticalRights::Allowed,
                    rights_receipt_ref: "rights:prior-receipt".into(),
                    access_tier: AccessTier::Controlled,
                },
                stage_evidence: vec![
                    hdb22_stage(EvidenceStage::Survival, 0.9),
                    hdb22_stage(EvidenceStage::Exposure, 0.8),
                    hdb22_stage(EvidenceStage::Observability, 0.7),
                    hdb22_stage(EvidenceStage::FalsePositive, 0.1),
                ],
            },
        }
    }

    #[test]
    fn conditional_hdb11_pod_and_explicit_hdb15_location_receipt_adapt_into_hdb22() {
        let report = adapt_heritage_detection_factors(&input()).unwrap();
        assert_eq!(
            report.schema_version,
            HERITAGE_FACTOR_ADAPTER_SCHEMA_VERSION
        );
        assert!(report.conditional_detection_stage_added);
        assert!(report.footprint_coverage_stage_added);
        assert!(!report.joint_pod_used);
        assert_eq!(
            report.detection_probability_wilson_95,
            ProbabilityBand {
                lower: 0.838_874_841_947_311_1,
                upper: 1.0,
            }
        );
        assert_eq!(report.target_location_probability.lower, 0.97);
        assert_eq!(report.non_detection_input.stage_evidence.len(), 6);
        assert!(
            report
                .non_detection_report
                .posterior_probability_originally_present
                .is_some()
        );
    }

    #[test]
    fn joint_pod_is_rejected_instead_of_double_counted() {
        let mut value = input();
        value.hdb11_report.detection_probability_scope =
            DetectionProbabilityScope::JointPerSurveyUnit;
        value.hdb11_report.conditioning_events.clear();
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::UnsupportedPodScope)
        );
    }

    #[test]
    fn target_presented_to_sensor_scope_is_rejected_until_hdb22_models_presentation() {
        let mut value = input();
        value.hdb11_report.detection_probability_scope =
            DetectionProbabilityScope::ConditionalOnTargetPresentedToSensor;
        value.hdb11_report.conditioning_events = vec![
            DetectionCondition::TargetSurvived,
            DetectionCondition::TargetExposed,
            DetectionCondition::TargetObservable,
            DetectionCondition::TargetCoveredByActualFootprint,
            DetectionCondition::TargetPresentedToSensor,
        ];
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::UnsupportedPodScope)
        );
    }

    #[test]
    fn arbitrary_joint_custom_scope_is_not_implicitly_mapped() {
        let mut value = input();
        value.hdb11_report.detection_probability_scope =
            DetectionProbabilityScope::OtherDeclaredConditioningSet;
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::UnsupportedPodScope)
        );
    }

    #[test]
    fn geometric_coverage_fraction_without_matching_uniform_model_is_rejected() {
        let mut value = input();
        value.target_location_coverage.target_location_probability = ProbabilityBand {
            lower: 0.8,
            upper: 0.9,
        };
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::UniformLocationModelDoesNotMatchAreaFraction)
        );
    }

    #[test]
    fn target_specific_model_is_allowed_only_with_its_own_probability_receipt() {
        let mut value = input();
        value.target_location_coverage.model =
            CoverageProbabilityModel::TargetSpecificSpatialDistribution;
        value.target_location_coverage.target_location_probability = ProbabilityBand {
            lower: 0.61,
            upper: 0.72,
        };
        let report = adapt_heritage_detection_factors(&value).unwrap();
        assert_eq!(report.target_location_probability.lower, 0.61);
        assert_eq!(report.target_location_probability.upper, 0.72);
    }

    #[test]
    fn wrong_survey_unit_or_spatial_union_receipt_is_rejected() {
        let mut value = input();
        value.target_location_coverage.survey_unit_id = "other-unit".into();
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::SurveyUnitMismatch)
        );

        let mut value = input();
        value.target_location_coverage.spatial_union_receipt_sha256 = digest('a');
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::SpatialUnionReceiptMismatch)
        );
    }

    #[test]
    fn mismatched_build_rights_or_unqualified_pod_is_rejected() {
        let mut value = input();
        value.hdb11_provenance.method_build_sha256 = digest('0');
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::PodMethodBuildMismatch)
        );

        let mut value = input();
        value.hdb11_report.context_match = false;
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::PodObservationNotQualified)
        );

        let mut value = input();
        value.hdb15_report.rights_status = FieldSurveyRightsStatus::Pending;
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::FieldSweepRightsNotVerified)
        );
    }

    #[test]
    fn preexisting_coverage_or_conditional_detection_cannot_be_overwritten() {
        let mut value = input();
        let mut preexisting_coverage = hdb22_stage(EvidenceStage::Survival, 0.5);
        preexisting_coverage.stage = EvidenceStage::FootprintCoverage;
        value
            .non_detection_template
            .stage_evidence
            .push(preexisting_coverage);
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(
                HeritageFactorAdapterError::TemplateAlreadyContainsAdaptedStage(
                    EvidenceStage::FootprintCoverage
                )
            )
        );
    }

    #[test]
    fn inaccessible_or_non_detection_observation_states_are_not_adapted() {
        let mut value = input();
        value.non_detection_template.observation_state =
            ObservationState::NotObservableOrInaccessible;
        assert_eq!(
            adapt_heritage_detection_factors(&value),
            Err(HeritageFactorAdapterError::TemplateMustBeSearchedNoDetection)
        );
    }
}
