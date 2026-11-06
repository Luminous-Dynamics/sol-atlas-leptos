// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fail-closed composition audit for HDB-11/HDB-15 evidence before HDB-22.
//!
//! v0.1 supports only the factorized HDB-22 chain. It never maps a joint
//! survey-unit POD into the conditional sensor-detection factor. It also
//! refuses to substitute HDB-15's swept-area fraction for target-location
//! coverage probability. A separately receipted target-location/intersection
//! estimate is required.
//!
//! This module validates structural compatibility and arithmetic only. It
//! does not authenticate source receipts, verify rights issuers, compute GIS
//! intersections, fit probabilities, or calculate an HDB-22 posterior.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::heritage_observability::{
    DetectionCondition, DetectionProbabilityScope, HERITAGE_OBSERVABILITY_SCHEMA_VERSION,
    HeritageObservabilityReport, ObservabilityDisposition, ObservationModality, ObservationOutcome,
    RightsStatus, SurveyAccessTier, VisibilityClass,
};

pub const HERITAGE_COMPOSITION_AUDIT_SCHEMA_VERSION: &str =
    "luminous.heritage-composition-audit.v0.1";
pub const FIELD_SWEEP_REPORT_SCHEMA_VERSION: &str = "luminous.heritage-field-sweep-report.v0.1";
const COVERAGE_RATIO_TOLERANCE: f64 = 1e-9;
const POD_ARITHMETIC_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedCompositionMode {
    FactorizedChain,
    CompositeSurveyUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldSweepMethod {
    FieldWalking,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionAnalyticalRights {
    Allowed,
    Restricted,
    Denied,
    Undetermined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionRightsPurpose {
    ResearchAnalysis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionRightsDecision {
    Allowed,
    Restricted,
    Prohibited,
    Undetermined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRightsClearanceV1 {
    pub rights_decision_id: String,
    pub purpose: CompositionRightsPurpose,
    pub decision: CompositionRightsDecision,
    pub receipt_ref: String,
    pub receipt_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageProbabilityModel {
    UniformAreaWeightedWithinSurveyUnit,
    TargetSpecificSpatialDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageProbabilityIntervalSemantics {
    PointEstimate,
    DeterministicBound,
    FrequentistConfidenceInterval,
    BayesianCredibleInterval,
}

/// Physical observation domain, not the HDB-15 equivalent perfect-detection band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActualObservationDomainBasis {
    RecordedObserverPathAndDeclaredViewBoundary,
    SensorValidDataFootprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbabilityBand {
    pub lower: f64,
    pub upper: f64,
}

impl ProbabilityBand {
    fn validate(self, label: &'static str) -> Result<(), HeritageCompositionError> {
        if !self.lower.is_finite()
            || !self.upper.is_finite()
            || self.lower < 0.0
            || self.upper > 1.0
            || self.lower > self.upper
        {
            return Err(HeritageCompositionError::InvalidProbabilityBand(label));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SweepWidthIntervalReceiptV1 {
    pub confidence_level: f64,
    pub z_value: f64,
    pub standard_error_m: f64,
    pub lower_m: f64,
    pub upper_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSweepReportReceiptV1 {
    pub schema_version: String,
    pub evaluation_id: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub calibration_data_sha256: String,
    pub calibration_split_manifest_sha256: String,
    pub fit_receipt_sha256: String,
    pub survey_unit_id: String,
    pub transect_geometry_sha256: String,
    pub survey_unit_geometry_sha256: String,
    pub spatial_union_receipt_sha256: String,
    pub context_match: bool,
    pub effective_sweep_width_m: f64,
    pub effective_sweep_width_interval_95: SweepWidthIntervalReceiptV1,
    pub buffered_sweep_width_m: f64,
    pub receipt_reported_swept_area_m2: f64,
    pub receipt_reported_coverage_fraction: f64,
    pub validation_brier_score: f64,
    pub fit_transect_count: usize,
    pub validation_transect_count: usize,
    pub fit_seeded_target_count: usize,
    pub validation_seeded_target_count: usize,
    pub rights_status: SourceRightsStatus,
    pub archaeological_absence_claimed: bool,
    pub archaeological_identification_claimed: bool,
    pub geomorphological_surface_survival_estimated: bool,
    pub repeated_search_independence_assumed: bool,
}

/// Context and validation binding for the HDB-15 calibration profile.
/// The crosswalk receipt binds the detailed field-survey context to the
/// HDB-11 detection context; a matching feature-class string alone is not
/// treated as sufficient context equivalence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSweepProfileBindingV1 {
    pub profile_id: String,
    pub profile_sha256: String,
    pub calibration_data_sha256: String,
    pub calibration_split_manifest_sha256: String,
    pub fit_receipt_sha256: String,
    pub maximum_permitted_validation_brier_score: f64,
    pub target_feature_class_id: String,
    pub survey_method: FieldSweepMethod,
    pub method_id: String,
    pub method_build_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_ref: String,
    pub access_tier: SurveyAccessTier,
    pub hdb11_profile_sha256: String,
    pub hdb11_context_sha256: String,
    pub hdb22_context_id: String,
    pub context_alignment_receipt_ref: String,
    pub context_alignment_receipt_sha256: String,
}

/// Geometry-bearing HDB-15 coverage receipt fields. The reported area ratio
/// is preserved as a geometric diagnostic and is never used as a probability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSweepGeometryReceiptV1 {
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    pub survey_unit_area_m2: f64,
    pub buffered_transect_union_area_m2: f64,
    pub buffered_sweep_width_m: f64,
    pub calibration_profile_sha256: String,
    pub transect_geometry_sha256: String,
    pub survey_unit_geometry_sha256: String,
    pub spatial_union_method_id: String,
    pub spatial_union_receipt_sha256: String,
}

/// Candidate-specific coverage probability, computed outside this module
/// from the actual searched geometry and an explicit target-location model.
/// Its receipt must bind to the exact HDB-15 union and survey-unit geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetCoverageProbabilityEvidenceV1 {
    pub target_class_id: String,
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    pub probability_model: CoverageProbabilityModel,
    pub interval_semantics: CoverageProbabilityIntervalSemantics,
    pub confidence_level: Option<f64>,
    pub probability: ProbabilityBand,
    pub target_location_distribution_sha256: String,
    pub intersection_geometry_sha256: String,
    /// Aligns HDB-11's TargetCoveredByActualFootprint event to this exact
    /// HDB-15 geometry/distribution probability receipt.
    pub hdb11_coverage_condition_alignment_receipt_ref: String,
    pub hdb11_coverage_condition_alignment_receipt_sha256: String,
    pub hdb11_coverage_condition_profile_sha256: String,
    /// Independently measured physical observation domain. This is not inferred
    /// from HDB-15's effective-sweep-width buffer.
    pub actual_observation_domain_basis: ActualObservationDomainBasis,
    pub actual_observation_domain_geometry_sha256: String,
    /// Geometry digest used when HDB-11 trials adjudicated the covered condition.
    pub hdb11_conditioned_observation_domain_geometry_sha256: String,
    /// Target-location distribution digest used by HDB-11 coverage conditioning.
    pub hdb11_conditioned_target_location_distribution_sha256: String,
    pub actual_observation_domain_receipt_ref: String,
    pub actual_observation_domain_receipt_sha256: String,
    /// A zero value means no observation could occur outside the declared domain.
    /// Nonzero tails require a continuous spatial detection likelihood instead.
    pub outside_domain_detection_probability: f64,
    pub coverage_probability_receipt_ref: String,
    pub coverage_probability_receipt_sha256: String,
    pub source_field_sweep_report_sha256: String,
    pub source_data_sha256: String,
    pub transect_geometry_sha256: String,
    pub survey_unit_geometry_sha256: String,
    pub spatial_union_receipt_sha256: String,
    /// Required for a uniform-area assumption; it must justify that assumption
    /// for this target class and sampling frame. Not needed for a target-specific
    /// non-uniform distribution with its own evidence digest.
    pub uniform_assumption_justification_ref: Option<String>,
    pub uniform_assumption_justification_sha256: Option<String>,
    pub method_id: String,
    pub method_build_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageCompositionInputV1 {
    pub composition_id: String,
    pub requested_mode: RequestedCompositionMode,
    pub hdb22_context_id: String,
    pub hdb11_analytical_rights: SourceRightsClearanceV1,
    pub hdb15_analytical_rights: SourceRightsClearanceV1,
    pub observability_report_sha256: String,
    pub observability: HeritageObservabilityReport,
    pub field_sweep_report_sha256: String,
    pub field_sweep_report: FieldSweepReportReceiptV1,
    pub field_sweep_profile: FieldSweepProfileBindingV1,
    pub field_sweep_geometry: FieldSweepGeometryReceiptV1,
    pub target_coverage: TargetCoverageProbabilityEvidenceV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactorReceiptReference {
    pub source_schema_version: String,
    pub source_report_sha256: String,
    pub method_id: String,
    pub method_build_sha256: String,
    pub source_data_sha256: String,
    pub analytical_rights: CompositionAnalyticalRights,
    pub rights_purpose: CompositionRightsPurpose,
    pub rights_receipt_sha256: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub calibration_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_ref: String,
    pub access_tier: SurveyAccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalDetectionFactorProposal {
    pub stage: String,
    pub conditioning_semantics: Vec<DetectionCondition>,
    pub probability: ProbabilityBand,
    pub confidence_level: f64,
    pub receipt: FactorReceiptReference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FootprintCoverageFactorProposal {
    pub stage: String,
    pub conditioning_semantics: String,
    pub evidence_basis: String,
    pub method_id: String,
    pub method_build_sha256: String,
    pub source_data_sha256: String,
    pub analytical_rights: CompositionAnalyticalRights,
    pub rights_purpose: CompositionRightsPurpose,
    pub rights_receipt_sha256: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub calibration_data_sha256: String,
    pub probability_model: CoverageProbabilityModel,
    pub interval_semantics: CoverageProbabilityIntervalSemantics,
    pub confidence_level: Option<f64>,
    pub probability: ProbabilityBand,
    pub target_location_distribution_sha256: String,
    pub intersection_geometry_sha256: String,
    pub coverage_geometry_sha256: String,
    pub hdb11_coverage_condition_alignment_receipt_ref: String,
    pub hdb11_coverage_condition_alignment_receipt_sha256: String,
    pub actual_observation_domain_basis: ActualObservationDomainBasis,
    pub actual_observation_domain_geometry_sha256: String,
    pub hdb11_conditioned_observation_domain_geometry_sha256: String,
    pub hdb11_conditioned_target_location_distribution_sha256: String,
    pub actual_observation_domain_receipt_ref: String,
    pub actual_observation_domain_receipt_sha256: String,
    pub outside_domain_detection_probability: f64,
    pub coverage_probability_receipt_ref: String,
    pub coverage_probability_receipt_sha256: String,
    pub field_sweep_report_sha256: String,
    pub transect_geometry_sha256: String,
    pub survey_unit_geometry_sha256: String,
    pub spatial_union_receipt_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_receipt_ref: String,
    pub access_tier: SurveyAccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageCompositionAuditReportV1 {
    pub schema_version: String,
    pub composition_id: String,
    pub hdb22_context_id: String,
    pub target_class_id: String,
    pub survey_unit_id: String,
    pub disposition: String,
    pub factorization: Vec<String>,
    pub conditional_detection: ConditionalDetectionFactorProposal,
    pub footprint_coverage: FootprintCoverageFactorProposal,
    /// Kept only as a geometric diagnostic. It is never copied into a probability field.
    pub hdb15_geometric_coverage_fraction: f64,
    pub joint_detection_factor_used: bool,
    pub area_fraction_used_as_probability: bool,
    /// HDB-22 still requires independent survival, exposure, observability and
    /// false-positive evidence. This audit never claims a complete posterior.
    pub complete_hdb22_stage_set_present: bool,
    pub posterior_computed: bool,
    pub archaeological_absence_claimed: bool,
    pub archaeological_identification_claimed: bool,
    pub action_authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageCompositionError {
    EmptyCompositionId,
    EmptyHdb22ContextId,
    CompositeModeNotSupportedByHdb22V01,
    WrongObservabilitySchemaVersion,
    WrongFieldSweepSchemaVersion,
    ObservabilityReportDigestInvalid,
    FieldSweepReportDigestInvalid,
    ObservabilityNotQualified,
    ObservabilityNotASearchedNonDetection,
    ObservabilityContextMismatch,
    ObservabilityProfileMismatch,
    ObservabilityRightsNotVerified,
    ObservabilitySampleAccountingInvalid,
    ObservabilityProbabilityAccountingInvalid,
    ObservabilityMeasurementInvalid,
    ObservabilityFlagsInvalid,
    DetectionScopeIsComposite,
    DetectionScopeNotSupported,
    DetectionConditioningSetMismatch,
    TargetClassMismatch,
    FieldSweepProfileMismatch,
    FieldSweepContextNotAligned,
    FieldSweepMethodMismatch,
    FieldSweepReportNotQualified,
    FieldSweepRightsNotVerified,
    FieldSweepFlagsInvalid,
    FieldSweepGeometryMismatch,
    FieldSweepCoverageArithmeticMismatch,
    InvalidFieldSweepGeometry,
    TargetCoverageReceiptMismatch,
    TargetLocationDistributionMissing,
    CoverageIntersectionReceiptMissing,
    ActualObservationDomainReceiptMissing,
    OutsideDomainDetectionNotZero,
    UniformAssumptionJustificationMissing,
    InvalidProbabilityBand(&'static str),
    InvalidDigest(&'static str),
    EmptyReceiptReference(&'static str),
    InvalidAccessTier,
    RightsClearanceNotAllowed(&'static str),
    RightsDecisionMismatch(&'static str),
}

impl fmt::Display for HeritageCompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage composition audit error: {self:?}")
    }
}

impl std::error::Error for HeritageCompositionError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_digest(value: &str, label: &'static str) -> Result<(), HeritageCompositionError> {
    if !valid_sha256(value) {
        return Err(HeritageCompositionError::InvalidDigest(label));
    }
    Ok(())
}

fn validate_reference(value: &str, label: &'static str) -> Result<(), HeritageCompositionError> {
    if !non_empty(value) {
        return Err(HeritageCompositionError::EmptyReceiptReference(label));
    }
    Ok(())
}

fn validate_rights_clearance(
    clearance: &SourceRightsClearanceV1,
    expected_decision_id: &str,
    expected_receipt_ref: &str,
    source_label: &'static str,
) -> Result<(), HeritageCompositionError> {
    validate_reference(&clearance.rights_decision_id, "rights_decision_id")?;
    validate_reference(&clearance.receipt_ref, "rights_receipt_ref")?;
    validate_digest(&clearance.receipt_sha256, "rights_receipt_sha256")?;
    if clearance.rights_decision_id != expected_decision_id
        || clearance.receipt_ref != expected_receipt_ref
    {
        return Err(HeritageCompositionError::RightsDecisionMismatch(
            source_label,
        ));
    }
    if clearance.purpose != CompositionRightsPurpose::ResearchAnalysis
        || clearance.decision != CompositionRightsDecision::Allowed
    {
        return Err(HeritageCompositionError::RightsClearanceNotAllowed(
            source_label,
        ));
    }
    Ok(())
}

fn validate_observability(
    input: &HeritageCompositionInputV1,
) -> Result<ConditionalDetectionFactorProposal, HeritageCompositionError> {
    let report = &input.observability;
    if report.schema_version != HERITAGE_OBSERVABILITY_SCHEMA_VERSION {
        return Err(HeritageCompositionError::WrongObservabilitySchemaVersion);
    }
    validate_digest(
        &input.observability_report_sha256,
        "observability_report_sha256",
    )?;
    if report.observation_outcome != ObservationOutcome::NotDetected
        || report
            .observation_id
            .as_deref()
            .is_none_or(|id| !non_empty(id))
    {
        return Err(HeritageCompositionError::ObservabilityNotASearchedNonDetection);
    }
    if report.disposition != ObservabilityDisposition::QualifiedNonDetection
        || !report.reasons.is_empty()
        || report.context_match != true
        || report.visibility != VisibilityClass::Adequate
        || report.observed_coverage_fraction < report.minimum_coverage_fraction
        || report.observed_valid_data_fraction < report.minimum_valid_data_fraction
    {
        return Err(HeritageCompositionError::ObservabilityNotQualified);
    }
    if report.calibration_context != report.observed_context {
        return Err(HeritageCompositionError::ObservabilityContextMismatch);
    }
    if [
        report.minimum_pod_lower_bound,
        report.observed_coverage_fraction,
        report.minimum_coverage_fraction,
        report.observed_valid_data_fraction,
        report.minimum_valid_data_fraction,
    ]
    .iter()
    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        || report.observed_coverage_fraction < report.minimum_coverage_fraction
        || report.observed_valid_data_fraction < report.minimum_valid_data_fraction
        || report.minimum_independent_psus < 20
    {
        return Err(HeritageCompositionError::ObservabilityMeasurementInvalid);
    }
    if report.observation_profile_sha256 != report.profile_sha256 {
        return Err(HeritageCompositionError::ObservabilityProfileMismatch);
    }
    if report.rights_status != RightsStatus::Verified
        || !non_empty(&report.rights_decision_id)
        || !non_empty(&report.rights_receipt_ref)
        || report.access_tier == SurveyAccessTier::Unknown
    {
        return Err(HeritageCompositionError::ObservabilityRightsNotVerified);
    }
    validate_rights_clearance(
        &input.hdb11_analytical_rights,
        &report.rights_decision_id,
        &report.rights_receipt_ref,
        "hdb11_research_analysis",
    )?;
    let accounted_psus = report
        .calibration_eligible_independent_psu_count
        .checked_add(report.calibration_excluded_conditioning_psu_count);
    if report.calibration_total_independent_psu_count > 100_000
        || report.calibration_eligible_independent_psu_count < report.minimum_independent_psus
        || accounted_psus != Some(report.calibration_total_independent_psu_count)
        || report.calibration_detected_count > report.calibration_eligible_independent_psu_count
    {
        return Err(HeritageCompositionError::ObservabilitySampleAccountingInvalid);
    }
    let expected_empirical = report.calibration_detected_count as f64
        / report.calibration_eligible_independent_psu_count as f64;
    let interval = report.probability_of_detection_wilson_95;
    if !report.empirical_probability_of_detection.is_finite()
        || (report.empirical_probability_of_detection - expected_empirical).abs()
            > POD_ARITHMETIC_TOLERANCE
        || !interval.lower.is_finite()
        || !interval.upper.is_finite()
        || interval.confidence_level != 0.95
        || !interval.z_value.is_finite()
        || interval.z_value <= 0.0
        || interval.lower < 0.0
        || interval.upper > 1.0
        || interval.lower > expected_empirical
        || interval.upper < expected_empirical
        || interval.lower < report.minimum_pod_lower_bound
    {
        return Err(HeritageCompositionError::ObservabilityProbabilityAccountingInvalid);
    }
    if report.archaeological_absence_claimed
        || report.archaeological_identification_claimed
        || report.combined_multi_observation_probability_reported
    {
        return Err(HeritageCompositionError::ObservabilityFlagsInvalid);
    }
    if report
        .calibration_context
        .feature_class_id
        .trim()
        .is_empty()
    {
        return Err(HeritageCompositionError::TargetClassMismatch);
    }

    let expected_conditions = vec![
        DetectionCondition::TargetSurvived,
        DetectionCondition::TargetExposed,
        DetectionCondition::TargetObservable,
        DetectionCondition::TargetCoveredByActualFootprint,
    ];
    match report.detection_probability_scope {
        DetectionProbabilityScope::JointPerSurveyUnit => {
            return Err(HeritageCompositionError::DetectionScopeIsComposite);
        }
        DetectionProbabilityScope::ConditionalOnTargetSurvivedExposedObservableAndCovered => {
            if report.conditioning_events != expected_conditions {
                return Err(HeritageCompositionError::DetectionConditioningSetMismatch);
            }
        }
        DetectionProbabilityScope::ConditionalOnTargetPresentedToSensor
        | DetectionProbabilityScope::OtherDeclaredConditioningSet => {
            return Err(HeritageCompositionError::DetectionScopeNotSupported);
        }
    }

    for (value, label) in [
        (
            &report.calibration_context.processing_build_sha256,
            "hdb11_method_build_sha256",
        ),
        (&report.profile_sha256, "hdb11_profile_sha256"),
        (
            &report.calibration_data_sha256,
            "hdb11_calibration_data_sha256",
        ),
        (
            &report.calibration_split_manifest_sha256,
            "hdb11_calibration_split_manifest_sha256",
        ),
        (
            &report.validation_receipt_sha256,
            "hdb11_validation_receipt_sha256",
        ),
    ] {
        validate_digest(value, label)?;
    }
    for (value, label) in [
        (&report.method_id, "hdb11_method_id"),
        (&report.profile_id, "hdb11_profile_id"),
        (
            &report.validation_receipt_ref,
            "hdb11_validation_receipt_ref",
        ),
        (&report.rights_decision_id, "hdb11_rights_decision_id"),
        (&report.rights_receipt_ref, "hdb11_rights_receipt_ref"),
    ] {
        validate_reference(value, label)?;
    }

    Ok(ConditionalDetectionFactorProposal {
        stage: "conditional_detection".to_owned(),
        conditioning_semantics: expected_conditions,
        probability: ProbabilityBand {
            lower: interval.lower,
            upper: interval.upper,
        },
        confidence_level: interval.confidence_level,
        receipt: FactorReceiptReference {
            source_schema_version: report.schema_version.clone(),
            source_report_sha256: input.observability_report_sha256.clone(),
            method_id: report.method_id.clone(),
            method_build_sha256: report.calibration_context.processing_build_sha256.clone(),
            source_data_sha256: report.calibration_data_sha256.clone(),
            analytical_rights: CompositionAnalyticalRights::Allowed,
            rights_purpose: CompositionRightsPurpose::ResearchAnalysis,
            rights_receipt_sha256: input.hdb11_analytical_rights.receipt_sha256.clone(),
            profile_id: report.profile_id.clone(),
            profile_sha256: report.profile_sha256.clone(),
            calibration_data_sha256: report.calibration_data_sha256.clone(),
            validation_split_manifest_sha256: report.calibration_split_manifest_sha256.clone(),
            validation_receipt_ref: report.validation_receipt_ref.clone(),
            validation_receipt_sha256: report.validation_receipt_sha256.clone(),
            rights_decision_id: report.rights_decision_id.clone(),
            rights_receipt_ref: report.rights_receipt_ref.clone(),
            access_tier: report.access_tier,
        },
    })
}

fn validate_field_sweep(
    input: &HeritageCompositionInputV1,
) -> Result<(), HeritageCompositionError> {
    let report = &input.field_sweep_report;
    let profile = &input.field_sweep_profile;
    let geometry = &input.field_sweep_geometry;

    if report.schema_version != FIELD_SWEEP_REPORT_SCHEMA_VERSION {
        return Err(HeritageCompositionError::WrongFieldSweepSchemaVersion);
    }
    validate_digest(
        &input.field_sweep_report_sha256,
        "field_sweep_report_sha256",
    )?;
    if !report.context_match {
        return Err(HeritageCompositionError::FieldSweepReportNotQualified);
    }
    if report.rights_status != SourceRightsStatus::Verified {
        return Err(HeritageCompositionError::FieldSweepRightsNotVerified);
    }
    if report.archaeological_absence_claimed
        || report.archaeological_identification_claimed
        || report.geomorphological_surface_survival_estimated
        || report.repeated_search_independence_assumed
    {
        return Err(HeritageCompositionError::FieldSweepFlagsInvalid);
    }
    if report.profile_id != profile.profile_id
        || report.profile_sha256 != profile.profile_sha256
        || report.profile_sha256 != geometry.calibration_profile_sha256
        || report.calibration_data_sha256 != profile.calibration_data_sha256
        || report.calibration_split_manifest_sha256 != profile.calibration_split_manifest_sha256
        || report.fit_receipt_sha256 != profile.fit_receipt_sha256
    {
        return Err(HeritageCompositionError::FieldSweepProfileMismatch);
    }
    if profile.target_feature_class_id != input.observability.calibration_context.feature_class_id
        || profile.target_feature_class_id != input.target_coverage.target_class_id
    {
        return Err(HeritageCompositionError::TargetClassMismatch);
    }
    if profile.hdb11_profile_sha256 != input.observability.profile_sha256 {
        return Err(HeritageCompositionError::FieldSweepProfileMismatch);
    }
    if profile.hdb22_context_id != input.hdb22_context_id {
        return Err(HeritageCompositionError::FieldSweepContextNotAligned);
    }
    if profile.survey_method != FieldSweepMethod::FieldWalking
        || input.observability.calibration_context.modality != ObservationModality::FieldWalking
    {
        return Err(HeritageCompositionError::FieldSweepMethodMismatch);
    }
    if profile.access_tier == SurveyAccessTier::Unknown {
        return Err(HeritageCompositionError::InvalidAccessTier);
    }
    validate_rights_clearance(
        &input.hdb15_analytical_rights,
        &profile.rights_decision_id,
        &profile.rights_receipt_ref,
        "hdb15_research_analysis",
    )?;
    for (value, label) in [
        (&profile.profile_sha256, "hdb15_profile_sha256"),
        (
            &profile.calibration_data_sha256,
            "hdb15_calibration_data_sha256",
        ),
        (
            &profile.calibration_split_manifest_sha256,
            "hdb15_calibration_split_manifest_sha256",
        ),
        (&profile.fit_receipt_sha256, "hdb15_fit_receipt_sha256"),
        (&profile.method_build_sha256, "hdb15_method_build_sha256"),
        (
            &profile.validation_receipt_sha256,
            "hdb15_validation_receipt_sha256",
        ),
        (
            &profile.context_alignment_receipt_sha256,
            "context_alignment_receipt_sha256",
        ),
        (&profile.hdb11_profile_sha256, "hdb15_hdb11_profile_sha256"),
        (&profile.hdb11_context_sha256, "hdb15_hdb11_context_sha256"),
    ] {
        validate_digest(value, label)?;
    }
    for (value, label) in [
        (&profile.method_id, "hdb15_method_id"),
        (
            &profile.validation_receipt_ref,
            "hdb15_validation_receipt_ref",
        ),
        (&profile.rights_decision_id, "hdb15_rights_decision_id"),
        (&profile.rights_receipt_ref, "hdb15_rights_receipt_ref"),
        (
            &profile.context_alignment_receipt_ref,
            "context_alignment_receipt_ref",
        ),
    ] {
        validate_reference(value, label)?;
    }

    if report.survey_unit_id != geometry.survey_unit_id
        || report.transect_geometry_sha256 != geometry.transect_geometry_sha256
        || report.survey_unit_geometry_sha256 != geometry.survey_unit_geometry_sha256
        || report.spatial_union_receipt_sha256 != geometry.spatial_union_receipt_sha256
    {
        return Err(HeritageCompositionError::FieldSweepGeometryMismatch);
    }
    for (value, label) in [
        (&report.profile_sha256, "report_profile_sha256"),
        (
            &report.calibration_data_sha256,
            "report_calibration_data_sha256",
        ),
        (
            &report.calibration_split_manifest_sha256,
            "report_calibration_split_manifest_sha256",
        ),
        (&report.fit_receipt_sha256, "report_fit_receipt_sha256"),
        (
            &report.transect_geometry_sha256,
            "report_transect_geometry_sha256",
        ),
        (
            &report.survey_unit_geometry_sha256,
            "report_survey_unit_geometry_sha256",
        ),
        (
            &report.spatial_union_receipt_sha256,
            "report_spatial_union_receipt_sha256",
        ),
    ] {
        validate_digest(value, label)?;
    }
    if !non_empty(&report.evaluation_id)
        || !non_empty(&report.profile_id)
        || !non_empty(&report.survey_unit_id)
    {
        return Err(HeritageCompositionError::FieldSweepGeometryMismatch);
    }

    let sweep_interval = report.effective_sweep_width_interval_95;
    if !non_empty(&geometry.survey_unit_crs_id)
        || !non_empty(&geometry.spatial_union_method_id)
        || !report.effective_sweep_width_m.is_finite()
        || report.effective_sweep_width_m <= 0.0
        || !report.buffered_sweep_width_m.is_finite()
        || report.buffered_sweep_width_m <= 0.0
        || (report.buffered_sweep_width_m - geometry.buffered_sweep_width_m).abs()
            > COVERAGE_RATIO_TOLERANCE.max(geometry.buffered_sweep_width_m.abs() * 1e-9)
        || sweep_interval.confidence_level != 0.95
        || !sweep_interval.z_value.is_finite()
        || sweep_interval.z_value <= 0.0
        || !sweep_interval.standard_error_m.is_finite()
        || sweep_interval.standard_error_m < 0.0
        || !sweep_interval.lower_m.is_finite()
        || !sweep_interval.upper_m.is_finite()
        || sweep_interval.lower_m < 0.0
        || sweep_interval.lower_m > sweep_interval.upper_m
        || !report.validation_brier_score.is_finite()
        || !(0.0..=1.0).contains(&report.validation_brier_score)
        || !profile.maximum_permitted_validation_brier_score.is_finite()
        || !(0.0..=1.0).contains(&profile.maximum_permitted_validation_brier_score)
        || report.validation_brier_score > profile.maximum_permitted_validation_brier_score
        || report.fit_transect_count < 3
        || report.validation_transect_count < 2
        || report.fit_seeded_target_count < 20
        || report.validation_seeded_target_count < 10
        || !geometry.survey_unit_area_m2.is_finite()
        || geometry.survey_unit_area_m2 <= 0.0
        || !geometry.buffered_transect_union_area_m2.is_finite()
        || geometry.buffered_transect_union_area_m2 <= 0.0
        || geometry.buffered_transect_union_area_m2
            > geometry.survey_unit_area_m2 * (1.0 + COVERAGE_RATIO_TOLERANCE)
        || !geometry.buffered_sweep_width_m.is_finite()
        || geometry.buffered_sweep_width_m <= 0.0
        || !report.receipt_reported_coverage_fraction.is_finite()
        || report.receipt_reported_coverage_fraction <= 0.0
        || report.receipt_reported_coverage_fraction > 1.0
        || !report.receipt_reported_swept_area_m2.is_finite()
        || report.receipt_reported_swept_area_m2 <= 0.0
    {
        return Err(HeritageCompositionError::InvalidFieldSweepGeometry);
    }
    let geometric_ratio = geometry.buffered_transect_union_area_m2 / geometry.survey_unit_area_m2;
    if (geometric_ratio - report.receipt_reported_coverage_fraction).abs()
        > COVERAGE_RATIO_TOLERANCE
        || (geometry.buffered_transect_union_area_m2 - report.receipt_reported_swept_area_m2).abs()
            > COVERAGE_RATIO_TOLERANCE.max(geometry.buffered_transect_union_area_m2.abs() * 1e-9)
    {
        return Err(HeritageCompositionError::FieldSweepCoverageArithmeticMismatch);
    }
    Ok(())
}

fn validate_target_coverage(
    input: &HeritageCompositionInputV1,
) -> Result<FootprintCoverageFactorProposal, HeritageCompositionError> {
    let report = &input.field_sweep_report;
    let geometry = &input.field_sweep_geometry;
    let evidence = &input.target_coverage;

    evidence
        .probability
        .validate("target_specific_coverage_probability")?;
    if evidence.hdb11_coverage_condition_profile_sha256 != input.observability.profile_sha256 {
        return Err(HeritageCompositionError::TargetCoverageReceiptMismatch);
    }
    if evidence.outside_domain_detection_probability != 0.0 {
        return Err(HeritageCompositionError::OutsideDomainDetectionNotZero);
    }
    if evidence.hdb11_conditioned_observation_domain_geometry_sha256
        != evidence.actual_observation_domain_geometry_sha256
        || evidence.hdb11_conditioned_target_location_distribution_sha256
            != evidence.target_location_distribution_sha256
    {
        return Err(HeritageCompositionError::TargetCoverageReceiptMismatch);
    }
    if evidence.survey_unit_id != report.survey_unit_id
        || evidence.survey_unit_crs_id != geometry.survey_unit_crs_id
        || evidence.transect_geometry_sha256 != report.transect_geometry_sha256
        || evidence.survey_unit_geometry_sha256 != report.survey_unit_geometry_sha256
        || evidence.spatial_union_receipt_sha256 != report.spatial_union_receipt_sha256
        || evidence.source_field_sweep_report_sha256 != input.field_sweep_report_sha256
    {
        return Err(HeritageCompositionError::TargetCoverageReceiptMismatch);
    }
    for (value, label) in [
        (
            &evidence.target_location_distribution_sha256,
            "target_location_distribution_sha256",
        ),
        (
            &evidence.intersection_geometry_sha256,
            "intersection_geometry_sha256",
        ),
        (
            &evidence.hdb11_coverage_condition_alignment_receipt_sha256,
            "hdb11_coverage_condition_alignment_receipt_sha256",
        ),
        (
            &evidence.hdb11_coverage_condition_profile_sha256,
            "hdb11_coverage_condition_profile_sha256",
        ),
        (
            &evidence.actual_observation_domain_geometry_sha256,
            "actual_observation_domain_geometry_sha256",
        ),
        (
            &evidence.hdb11_conditioned_observation_domain_geometry_sha256,
            "hdb11_conditioned_observation_domain_geometry_sha256",
        ),
        (
            &evidence.hdb11_conditioned_target_location_distribution_sha256,
            "hdb11_conditioned_target_location_distribution_sha256",
        ),
        (
            &evidence.actual_observation_domain_receipt_sha256,
            "actual_observation_domain_receipt_sha256",
        ),
        (
            &evidence.coverage_probability_receipt_sha256,
            "coverage_probability_receipt_sha256",
        ),
        (
            &evidence.source_field_sweep_report_sha256,
            "coverage_source_field_sweep_report_sha256",
        ),
        (&evidence.source_data_sha256, "coverage_source_data_sha256"),
        (
            &evidence.transect_geometry_sha256,
            "coverage_transect_geometry_sha256",
        ),
        (
            &evidence.survey_unit_geometry_sha256,
            "coverage_survey_unit_geometry_sha256",
        ),
        (
            &evidence.spatial_union_receipt_sha256,
            "coverage_spatial_union_receipt_sha256",
        ),
        (
            &evidence.method_build_sha256,
            "coverage_method_build_sha256",
        ),
        (
            &evidence.validation_split_manifest_sha256,
            "coverage_validation_split_manifest_sha256",
        ),
        (
            &evidence.validation_receipt_sha256,
            "coverage_validation_receipt_sha256",
        ),
    ] {
        validate_digest(value, label)?;
    }
    for (value, label) in [
        (
            &evidence.coverage_probability_receipt_ref,
            "coverage_probability_receipt_ref",
        ),
        (
            &evidence.hdb11_coverage_condition_alignment_receipt_ref,
            "hdb11_coverage_condition_alignment_receipt_ref",
        ),
        (
            &evidence.actual_observation_domain_receipt_ref,
            "actual_observation_domain_receipt_ref",
        ),
        (&evidence.method_id, "coverage_method_id"),
        (
            &evidence.validation_receipt_ref,
            "coverage_validation_receipt_ref",
        ),
    ] {
        validate_reference(value, label)?;
    }
    match evidence.interval_semantics {
        CoverageProbabilityIntervalSemantics::PointEstimate => {
            if evidence.probability.lower != evidence.probability.upper
                || evidence.confidence_level.is_some()
            {
                return Err(HeritageCompositionError::InvalidProbabilityBand(
                    "point_estimate_semantics",
                ));
            }
        }
        CoverageProbabilityIntervalSemantics::DeterministicBound => {
            if evidence.confidence_level.is_some() {
                return Err(HeritageCompositionError::InvalidProbabilityBand(
                    "deterministic_bound_confidence_level",
                ));
            }
        }
        CoverageProbabilityIntervalSemantics::FrequentistConfidenceInterval
        | CoverageProbabilityIntervalSemantics::BayesianCredibleInterval => {
            let level = evidence.confidence_level.ok_or(
                HeritageCompositionError::InvalidProbabilityBand("interval_confidence_level"),
            )?;
            if !level.is_finite() || !(0.0..1.0).contains(&level) {
                return Err(HeritageCompositionError::InvalidProbabilityBand(
                    "interval_confidence_level",
                ));
            }
        }
    }

    if !non_empty(&evidence.survey_unit_crs_id) {
        return Err(HeritageCompositionError::TargetCoverageReceiptMismatch);
    }

    if evidence.probability_model == CoverageProbabilityModel::UniformAreaWeightedWithinSurveyUnit {
        let reference = evidence
            .uniform_assumption_justification_ref
            .as_deref()
            .ok_or(HeritageCompositionError::UniformAssumptionJustificationMissing)?;
        let digest = evidence
            .uniform_assumption_justification_sha256
            .as_deref()
            .ok_or(HeritageCompositionError::UniformAssumptionJustificationMissing)?;
        validate_reference(reference, "uniform_assumption_justification_ref")?;
        validate_digest(digest, "uniform_assumption_justification_sha256")?;
    } else if evidence.uniform_assumption_justification_ref.is_some()
        || evidence.uniform_assumption_justification_sha256.is_some()
    {
        return Err(HeritageCompositionError::TargetCoverageReceiptMismatch);
    }
    if evidence.probability_model == CoverageProbabilityModel::TargetSpecificSpatialDistribution
        && evidence.target_location_distribution_sha256.is_empty()
    {
        return Err(HeritageCompositionError::TargetLocationDistributionMissing);
    }
    if evidence.intersection_geometry_sha256.is_empty()
        || evidence.coverage_probability_receipt_ref.is_empty()
    {
        return Err(HeritageCompositionError::CoverageIntersectionReceiptMissing);
    }
    if !valid_sha256(&evidence.actual_observation_domain_geometry_sha256)
        || !valid_sha256(&evidence.actual_observation_domain_receipt_sha256)
        || !non_empty(&evidence.actual_observation_domain_receipt_ref)
    {
        return Err(HeritageCompositionError::ActualObservationDomainReceiptMissing);
    }

    Ok(FootprintCoverageFactorProposal {
        stage: "footprint_coverage".to_owned(),
        conditioning_semantics:
            "target_location_inside_independently_receipted_physical_observation_domain; detection_outside_domain_is_zero_by_protocol"
                .to_owned(),
        evidence_basis: "independent_physical_observation_domain_receipt".to_owned(),
        method_id: evidence.method_id.clone(),
        method_build_sha256: evidence.method_build_sha256.clone(),
        source_data_sha256: evidence.source_data_sha256.clone(),
        analytical_rights: CompositionAnalyticalRights::Allowed,
        rights_purpose: CompositionRightsPurpose::ResearchAnalysis,
        rights_receipt_sha256: input.hdb15_analytical_rights.receipt_sha256.clone(),
        profile_id: input.field_sweep_profile.profile_id.clone(),
        profile_sha256: input.field_sweep_profile.profile_sha256.clone(),
        calibration_data_sha256: report.calibration_data_sha256.clone(),
        probability_model: evidence.probability_model,
        interval_semantics: evidence.interval_semantics,
        confidence_level: evidence.confidence_level,
        probability: evidence.probability,
        target_location_distribution_sha256: evidence.target_location_distribution_sha256.clone(),
        intersection_geometry_sha256: evidence.intersection_geometry_sha256.clone(),
        coverage_geometry_sha256: evidence.actual_observation_domain_geometry_sha256.clone(),
        hdb11_coverage_condition_alignment_receipt_ref: evidence
            .hdb11_coverage_condition_alignment_receipt_ref
            .clone(),
        hdb11_coverage_condition_alignment_receipt_sha256: evidence
            .hdb11_coverage_condition_alignment_receipt_sha256
            .clone(),
        actual_observation_domain_basis: evidence.actual_observation_domain_basis,
        actual_observation_domain_geometry_sha256: evidence
            .actual_observation_domain_geometry_sha256
            .clone(),
        hdb11_conditioned_observation_domain_geometry_sha256: evidence
            .hdb11_conditioned_observation_domain_geometry_sha256
            .clone(),
        hdb11_conditioned_target_location_distribution_sha256: evidence
            .hdb11_conditioned_target_location_distribution_sha256
            .clone(),
        actual_observation_domain_receipt_ref: evidence
            .actual_observation_domain_receipt_ref
            .clone(),
        actual_observation_domain_receipt_sha256: evidence
            .actual_observation_domain_receipt_sha256
            .clone(),
        outside_domain_detection_probability: evidence.outside_domain_detection_probability,
        coverage_probability_receipt_ref: evidence.coverage_probability_receipt_ref.clone(),
        coverage_probability_receipt_sha256: evidence.coverage_probability_receipt_sha256.clone(),
        field_sweep_report_sha256: input.field_sweep_report_sha256.clone(),
        transect_geometry_sha256: report.transect_geometry_sha256.clone(),
        survey_unit_geometry_sha256: report.survey_unit_geometry_sha256.clone(),
        spatial_union_receipt_sha256: report.spatial_union_receipt_sha256.clone(),
        validation_split_manifest_sha256: evidence.validation_split_manifest_sha256.clone(),
        validation_receipt_ref: evidence.validation_receipt_ref.clone(),
        validation_receipt_sha256: evidence.validation_receipt_sha256.clone(),
        rights_receipt_ref: input.field_sweep_profile.rights_receipt_ref.clone(),
        access_tier: input.field_sweep_profile.access_tier,
    })
}

/// Audits whether the exact HDB-11 conditional detection estimate and HDB-15
/// geometry can populate the corresponding two factors in HDB-22's factorized
/// chain. Joint POD is rejected because HDB-22 v0.1 has no composite mode.
pub fn audit_heritage_probability_composition(
    input: &HeritageCompositionInputV1,
) -> Result<HeritageCompositionAuditReportV1, HeritageCompositionError> {
    if !non_empty(&input.composition_id) {
        return Err(HeritageCompositionError::EmptyCompositionId);
    }
    if !non_empty(&input.hdb22_context_id) {
        return Err(HeritageCompositionError::EmptyHdb22ContextId);
    }
    if input.requested_mode == RequestedCompositionMode::CompositeSurveyUnit {
        return Err(HeritageCompositionError::CompositeModeNotSupportedByHdb22V01);
    }

    let conditional_detection = validate_observability(input)?;
    validate_field_sweep(input)?;
    let footprint_coverage = validate_target_coverage(input)?;

    if input.field_sweep_profile.target_feature_class_id
        != input.observability.calibration_context.feature_class_id
    {
        return Err(HeritageCompositionError::TargetClassMismatch);
    }

    Ok(HeritageCompositionAuditReportV1 {
        schema_version: HERITAGE_COMPOSITION_AUDIT_SCHEMA_VERSION.to_owned(),
        composition_id: input.composition_id.clone(),
        hdb22_context_id: input.hdb22_context_id.clone(),
        target_class_id: input
            .observability
            .calibration_context
            .feature_class_id
            .clone(),
        survey_unit_id: input.field_sweep_report.survey_unit_id.clone(),
        disposition: "factorized_mapping_eligible_for_two_factors_only".to_owned(),
        factorization: vec![
            "survival".to_owned(),
            "exposure_given_survival".to_owned(),
            "observability_given_survival_and_exposure".to_owned(),
            "target_specific_footprint_coverage_given_survival_exposure_and_observability"
                .to_owned(),
            "conditional_detection_given_survival_exposure_observability_and_coverage".to_owned(),
            "false_positive_given_originally_absent".to_owned(),
        ],
        conditional_detection,
        footprint_coverage,
        hdb15_geometric_coverage_fraction: input
            .field_sweep_report
            .receipt_reported_coverage_fraction,
        joint_detection_factor_used: false,
        area_fraction_used_as_probability: false,
        complete_hdb22_stage_set_present: false,
        posterior_computed: false,
        archaeological_absence_claimed: false,
        archaeological_identification_claimed: false,
        action_authority: "none".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heritage_observability::{
        DetectionCondition, DetectionProbabilityScope, NonDetectionReason, SurveyContext,
        WilsonInterval,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn observability() -> HeritageObservabilityReport {
        let context = SurveyContext {
            feature_class_id: "dry_stone_wall".to_owned(),
            modality: ObservationModality::FieldWalking,
            sensor_configuration_sha256: digest('a'),
            processing_build_sha256: digest('b'),
            environment_stratum_id: "open_grassland_low_relief".to_owned(),
            season_stratum_id: "dry_season".to_owned(),
            resolution_class_id: "field_walking_1m".to_owned(),
        };
        HeritageObservabilityReport {
            schema_version: HERITAGE_OBSERVABILITY_SCHEMA_VERSION.to_owned(),
            candidate_id: "candidate-1".to_owned(),
            profile_id: "pod-profile-1".to_owned(),
            profile_sha256: digest('c'),
            method_id: "field-walk-detection-v1".to_owned(),
            calibration_data_sha256: digest('d'),
            calibration_split_manifest_sha256: digest('e'),
            validation_receipt_ref: "urn:validation:hdb11".to_owned(),
            validation_receipt_sha256: digest('f'),
            rights_decision_id: "rights:hdb11".to_owned(),
            rights_receipt_ref: "urn:rights:hdb11".to_owned(),
            access_tier: SurveyAccessTier::Controlled,
            calibration_context: context.clone(),
            observed_context: context,
            observation_profile_sha256: digest('c'),
            observation_id: Some("observation-1".to_owned()),
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
            calibration_total_independent_psu_count: 24,
            calibration_eligible_independent_psu_count: 20,
            calibration_excluded_conditioning_psu_count: 4,
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
            observed_coverage_fraction: 0.97,
            minimum_coverage_fraction: 0.95,
            observed_valid_data_fraction: 0.98,
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

    fn input() -> HeritageCompositionInputV1 {
        let report_digest = digest('1');
        let sweep_profile_digest = digest('2');
        let transect_digest = digest('3');
        let unit_digest = digest('4');
        let union_digest = digest('5');
        HeritageCompositionInputV1 {
            composition_id: "composition-1".to_owned(),
            requested_mode: RequestedCompositionMode::FactorizedChain,
            hdb22_context_id: "context-1".to_owned(),
            hdb11_analytical_rights: SourceRightsClearanceV1 {
                rights_decision_id: "rights:hdb11".to_owned(),
                purpose: CompositionRightsPurpose::ResearchAnalysis,
                decision: CompositionRightsDecision::Allowed,
                receipt_ref: "urn:rights:hdb11".to_owned(),
                receipt_sha256: digest('0'),
            },
            hdb15_analytical_rights: SourceRightsClearanceV1 {
                rights_decision_id: "rights:hdb15".to_owned(),
                purpose: CompositionRightsPurpose::ResearchAnalysis,
                decision: CompositionRightsDecision::Allowed,
                receipt_ref: "urn:rights:hdb15".to_owned(),
                receipt_sha256: digest('1'),
            },
            observability_report_sha256: digest('6'),
            observability: observability(),
            field_sweep_report_sha256: report_digest.clone(),
            field_sweep_report: FieldSweepReportReceiptV1 {
                schema_version: FIELD_SWEEP_REPORT_SCHEMA_VERSION.to_owned(),
                evaluation_id: "sweep-eval-1".to_owned(),
                profile_id: "sweep-profile-1".to_owned(),
                profile_sha256: sweep_profile_digest.clone(),
                calibration_data_sha256: digest('7'),
                calibration_split_manifest_sha256: digest('8'),
                fit_receipt_sha256: digest('9'),
                survey_unit_id: "survey-unit-1".to_owned(),
                transect_geometry_sha256: transect_digest.clone(),
                survey_unit_geometry_sha256: unit_digest.clone(),
                spatial_union_receipt_sha256: union_digest.clone(),
                context_match: true,
                effective_sweep_width_m: 2.0,
                effective_sweep_width_interval_95: SweepWidthIntervalReceiptV1 {
                    confidence_level: 0.95,
                    z_value: 1.959963984540054,
                    standard_error_m: 0.1,
                    lower_m: 1.8,
                    upper_m: 2.2,
                },
                buffered_sweep_width_m: 2.0,
                receipt_reported_swept_area_m2: 500.0,
                receipt_reported_coverage_fraction: 0.5,
                validation_brier_score: 0.12,
                fit_transect_count: 3,
                validation_transect_count: 2,
                fit_seeded_target_count: 20,
                validation_seeded_target_count: 10,
                rights_status: SourceRightsStatus::Verified,
                archaeological_absence_claimed: false,
                archaeological_identification_claimed: false,
                geomorphological_surface_survival_estimated: false,
                repeated_search_independence_assumed: false,
            },
            field_sweep_profile: FieldSweepProfileBindingV1 {
                profile_id: "sweep-profile-1".to_owned(),
                profile_sha256: sweep_profile_digest.clone(),
                calibration_data_sha256: digest('7'),
                calibration_split_manifest_sha256: digest('8'),
                fit_receipt_sha256: digest('9'),
                maximum_permitted_validation_brier_score: 0.2,
                target_feature_class_id: "dry_stone_wall".to_owned(),
                survey_method: FieldSweepMethod::FieldWalking,
                method_id: "lateral-sweep-model-v1".to_owned(),
                method_build_sha256: digest('a'),
                validation_receipt_ref: "urn:validation:hdb15".to_owned(),
                validation_receipt_sha256: digest('b'),
                rights_decision_id: "rights:hdb15".to_owned(),
                rights_receipt_ref: "urn:rights:hdb15".to_owned(),
                access_tier: SurveyAccessTier::Controlled,
                hdb11_profile_sha256: digest('c'),
                hdb11_context_sha256: digest('d'),
                hdb22_context_id: "context-1".to_owned(),
                context_alignment_receipt_ref: "urn:context-alignment:1".to_owned(),
                context_alignment_receipt_sha256: digest('e'),
            },
            field_sweep_geometry: FieldSweepGeometryReceiptV1 {
                survey_unit_id: "survey-unit-1".to_owned(),
                survey_unit_crs_id: "EPSG:32735".to_owned(),
                survey_unit_area_m2: 1000.0,
                buffered_transect_union_area_m2: 500.0,
                buffered_sweep_width_m: 2.0,
                calibration_profile_sha256: sweep_profile_digest,
                transect_geometry_sha256: transect_digest.clone(),
                survey_unit_geometry_sha256: unit_digest.clone(),
                spatial_union_method_id: "buffer-union-v1".to_owned(),
                spatial_union_receipt_sha256: union_digest.clone(),
            },
            target_coverage: TargetCoverageProbabilityEvidenceV1 {
                target_class_id: "dry_stone_wall".to_owned(),
                survey_unit_id: "survey-unit-1".to_owned(),
                survey_unit_crs_id: "EPSG:32735".to_owned(),
                probability_model: CoverageProbabilityModel::TargetSpecificSpatialDistribution,
                interval_semantics: CoverageProbabilityIntervalSemantics::DeterministicBound,
                confidence_level: None,
                probability: ProbabilityBand {
                    lower: 0.72,
                    upper: 0.81,
                },
                target_location_distribution_sha256: digest('f'),
                intersection_geometry_sha256: digest('0'),
                hdb11_coverage_condition_alignment_receipt_ref:
                    "urn:coverage-condition-alignment:1".to_owned(),
                hdb11_coverage_condition_alignment_receipt_sha256: digest('6'),
                hdb11_coverage_condition_profile_sha256: digest('c'),
                actual_observation_domain_basis:
                    ActualObservationDomainBasis::RecordedObserverPathAndDeclaredViewBoundary,
                actual_observation_domain_geometry_sha256: digest('o'),
                hdb11_conditioned_observation_domain_geometry_sha256: digest('o'),
                hdb11_conditioned_target_location_distribution_sha256: digest('f'),
                actual_observation_domain_receipt_ref: "urn:actual-observation-domain:1".to_owned(),
                actual_observation_domain_receipt_sha256: digest('p'),
                outside_domain_detection_probability: 0.0,
                coverage_probability_receipt_ref: "urn:coverage-probability:1".to_owned(),
                coverage_probability_receipt_sha256: digest('1'),
                source_field_sweep_report_sha256: report_digest,
                source_data_sha256: digest('7'),
                transect_geometry_sha256: transect_digest,
                survey_unit_geometry_sha256: unit_digest,
                spatial_union_receipt_sha256: union_digest,
                uniform_assumption_justification_ref: None,
                uniform_assumption_justification_sha256: None,
                method_id: "target-location-intersection-v1".to_owned(),
                method_build_sha256: digest('2'),
                validation_split_manifest_sha256: digest('3'),
                validation_receipt_ref: "urn:coverage-validation:1".to_owned(),
                validation_receipt_sha256: digest('4'),
            },
        }
    }

    #[test]
    fn factorized_scope_maps_only_the_conditional_detection_and_target_coverage_factors() {
        let report = audit_heritage_probability_composition(&input()).unwrap();
        assert_eq!(
            report.conditional_detection.probability,
            ProbabilityBand {
                lower: 0.699,
                upper: 0.972,
            }
        );
        assert_eq!(
            report.footprint_coverage.probability,
            ProbabilityBand {
                lower: 0.72,
                upper: 0.81,
            }
        );
        assert_eq!(report.hdb15_geometric_coverage_fraction, 0.5);
        assert!(!report.area_fraction_used_as_probability);
        assert!(!report.joint_detection_factor_used);
        assert!(!report.complete_hdb22_stage_set_present);
        assert!(!report.posterior_computed);
        assert!(!report.archaeological_absence_claimed);
        assert_eq!(report.action_authority, "none");
    }

    #[test]
    fn joint_survey_unit_pod_is_never_mapped_to_conditional_detection() {
        let mut value = input();
        value.observability.detection_probability_scope =
            DetectionProbabilityScope::JointPerSurveyUnit;
        value.observability.conditioning_events.clear();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::DetectionScopeIsComposite)
        );
    }

    #[test]
    fn composite_request_is_rejected_until_hdb22_has_a_composite_mode() {
        let mut value = input();
        value.requested_mode = RequestedCompositionMode::CompositeSurveyUnit;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::CompositeModeNotSupportedByHdb22V01)
        );
    }

    #[test]
    fn presented_to_sensor_and_custom_scopes_are_not_silently_coerced() {
        let mut value = input();
        value.observability.detection_probability_scope =
            DetectionProbabilityScope::ConditionalOnTargetPresentedToSensor;
        value
            .observability
            .conditioning_events
            .push(DetectionCondition::TargetPresentedToSensor);
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::DetectionScopeNotSupported)
        );

        value.observability.detection_probability_scope =
            DetectionProbabilityScope::OtherDeclaredConditioningSet;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::DetectionScopeNotSupported)
        );
    }

    #[test]
    fn hdb15_area_fraction_is_not_accepted_without_target_location_probability_receipt() {
        let mut value = input();
        value.target_coverage.probability = ProbabilityBand {
            lower: value.field_sweep_report.receipt_reported_coverage_fraction,
            upper: value.field_sweep_report.receipt_reported_coverage_fraction,
        };
        value
            .target_coverage
            .target_location_distribution_sha256
            .clear();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::InvalidDigest(
                "target_location_distribution_sha256"
            ))
        );
    }

    #[test]
    fn mismatched_source_report_geometry_or_survey_unit_is_rejected() {
        let mut value = input();
        value.target_coverage.survey_unit_id = "other-unit".to_owned();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::TargetCoverageReceiptMismatch)
        );
    }

    #[test]
    fn geometry_fraction_is_recomputed_and_must_match_the_hdb15_report() {
        let mut value = input();
        value.field_sweep_geometry.buffered_transect_union_area_m2 = 700.0;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::FieldSweepCoverageArithmeticMismatch)
        );
    }

    #[test]
    fn unqualified_or_mismatched_hdb11_observation_is_rejected() {
        let mut value = input();
        value.observability.disposition = ObservabilityDisposition::IndeterminateNonDetection;
        value.observability.reasons = vec![NonDetectionReason::CoverageBelowThreshold];
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ObservabilityNotQualified)
        );

        let mut value = input();
        value.observability.observed_context.feature_class_id = "other-feature".to_owned();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ObservabilityContextMismatch)
        );
    }

    #[test]
    fn sample_and_probability_arithmetic_are_checked_before_factor_mapping() {
        let mut value = input();
        value.observability.calibration_total_independent_psu_count = 23;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ObservabilitySampleAccountingInvalid)
        );

        let mut value = input();
        value.observability.empirical_probability_of_detection = 0.91;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ObservabilityProbabilityAccountingInvalid)
        );
    }

    #[test]
    fn context_profile_and_rights_crosswalks_are_required() {
        let mut value = input();
        value.field_sweep_profile.hdb11_profile_sha256 = digest('z');
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::FieldSweepProfileMismatch)
        );

        let mut value = input();
        value
            .field_sweep_profile
            .context_alignment_receipt_ref
            .clear();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::EmptyReceiptReference(
                "context_alignment_receipt_ref"
            ))
        );
    }

    #[test]
    fn uniform_coverage_assumption_requires_its_own_justification_receipt() {
        let mut value = input();
        value.target_coverage.probability_model =
            CoverageProbabilityModel::UniformAreaWeightedWithinSurveyUnit;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::UniformAssumptionJustificationMissing)
        );

        value.target_coverage.uniform_assumption_justification_ref =
            Some("urn:uniform-assumption:justification".to_owned());
        value
            .target_coverage
            .uniform_assumption_justification_sha256 = Some(digest('5'));
        assert!(audit_heritage_probability_composition(&value).is_ok());
    }

    #[test]
    fn hdb22_context_identifier_must_be_bound_by_the_crosswalk_receipt() {
        let mut value = input();
        value.hdb22_context_id = "other-context".to_owned();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::FieldSweepContextNotAligned)
        );
    }

    #[test]
    fn non_finite_observed_coverage_is_rejected_even_if_upstream_report_claims_qualified() {
        let mut value = input();
        value.observability.observed_coverage_fraction = f64::NAN;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ObservabilityMeasurementInvalid)
        );
    }

    #[test]
    fn coverage_interval_semantics_must_be_explicit_and_consistent() {
        let mut value = input();
        value.target_coverage.interval_semantics =
            CoverageProbabilityIntervalSemantics::FrequentistConfidenceInterval;
        value.target_coverage.confidence_level = None;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::InvalidProbabilityBand(
                "interval_confidence_level"
            ))
        );

        value.target_coverage.interval_semantics =
            CoverageProbabilityIntervalSemantics::PointEstimate;
        value.target_coverage.confidence_level = None;
        assert!(audit_heritage_probability_composition(&value).is_err());
    }


    #[test]
    fn hdb11_coverage_conditioning_must_use_the_exact_same_domain_and_distribution() {
        let mut value = input();
        value
            .target_coverage
            .hdb11_conditioned_observation_domain_geometry_sha256 = digest('z');
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::TargetCoverageReceiptMismatch)
        );

        let mut value = input();
        value
            .target_coverage
            .hdb11_conditioned_target_location_distribution_sha256 = digest('z');
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::TargetCoverageReceiptMismatch)
        );
    }

    #[test]
    fn effective_sweep_width_buffer_is_not_accepted_as_the_physical_observation_domain() {
        let mut value = input();
        value.target_coverage.outside_domain_detection_probability = 0.01;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::OutsideDomainDetectionNotZero)
        );

        let mut value = input();
        value
            .target_coverage
            .actual_observation_domain_receipt_ref
            .clear();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ActualObservationDomainReceiptMissing)
        );
    }

    #[test]
    fn target_coverage_must_bind_the_exact_hdb11_coverage_condition_event() {
        let mut value = input();
        value
            .target_coverage
            .hdb11_coverage_condition_profile_sha256 = digest('z');
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::TargetCoverageReceiptMismatch)
        );
    }

    #[test]
    fn hdb15_validation_brier_must_meet_the_bound_in_its_profile_binding() {
        let mut value = input();
        value.field_sweep_report.validation_brier_score = 0.25;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::InvalidFieldSweepGeometry)
        );
    }

    #[test]
    fn hdb15_buffer_width_must_match_the_exact_sweep_width_used_in_geometry() {
        let mut value = input();
        value.field_sweep_report.buffered_sweep_width_m = 2.5;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::InvalidFieldSweepGeometry)
        );
    }

    #[test]
    fn verified_source_rights_receipt_is_not_enough_without_research_analysis_grant() {
        let mut value = input();
        value.hdb11_analytical_rights.decision = CompositionRightsDecision::Undetermined;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::RightsClearanceNotAllowed(
                "hdb11_research_analysis"
            ))
        );

        let mut value = input();
        value.hdb15_analytical_rights.purpose = CompositionRightsPurpose::ResearchAnalysis;
        value.hdb15_analytical_rights.decision = CompositionRightsDecision::Restricted;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::RightsClearanceNotAllowed(
                "hdb15_research_analysis"
            ))
        );
    }

    #[test]
    fn rights_clearance_must_match_the_source_decision_identifier_and_receipt() {
        let mut value = input();
        value.hdb11_analytical_rights.rights_decision_id = "different-rights-decision".to_owned();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::RightsDecisionMismatch(
                "hdb11_research_analysis"
            ))
        );

        let mut value = input();
        value.hdb15_analytical_rights.receipt_ref = "urn:rights:other".to_owned();
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::RightsDecisionMismatch(
                "hdb15_research_analysis"
            ))
        );
    }

    #[test]
    fn detected_observation_does_not_produce_negative_evidence_adapter() {
        let mut value = input();
        value.observability.observation_outcome = ObservationOutcome::Detected;
        assert_eq!(
            audit_heritage_probability_composition(&value),
            Err(HeritageCompositionError::ObservabilityNotASearchedNonDetection)
        );
    }
}
