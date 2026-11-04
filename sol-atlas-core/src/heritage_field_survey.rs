// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Calibrated pedestrian field-survey detectability and effective sweep width.
//!
//! v0.1 consumes a frozen, externally fitted exponential lateral detection
//! function p(r) = b * exp(-k * r^2), validates its calibration provenance and
//! exact context, computes W = integral p(r) dr = b * sqrt(pi / k), and emits
//! a delta-method uncertainty interval plus a nominal coverage estimate.
//!
//! It does NOT fit b/k from raw calibration transects, compute spatial unions,
//! authenticate receipts, estimate archaeological presence, or declare absence.
//! The coverage estimate is meaningful only when the supplied GIS-side receipt
//! has already corrected transect geometry for overlaps and clipping.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const FIELD_SWEEP_SCHEMA_VERSION: &str = "luminous.heritage-field-sweep.v0.1";
pub const FIELD_SWEEP_REPORT_SCHEMA_VERSION: &str =
    "luminous.heritage-field-sweep-report.v0.1";
pub const MIN_FIT_TRANSECTS: usize = 3;
pub const MIN_VALIDATION_TRANSECTS: usize = 2;
pub const MIN_FIT_TARGETS: usize = 20;
pub const MIN_VALIDATION_TARGETS: usize = 10;
pub const NORMAL_95_Z: f64 = 1.959_963_984_540_054;
const COVARIANCE_TOLERANCE: f64 = 1e-12;
const COVERAGE_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyMethod {
    FieldWalking,
    GroundMagnetometry,
    GroundPenetratingRadar,
    Augering,
    TestPitting,
    UavSurfaceSurvey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSurveyContext {
    pub target_material_or_feature_class_id: String,
    pub obtrusiveness_class_id: String,
    pub survey_method: SurveyMethod,
    pub crew_cohort_id: String,
    pub search_speed_m_per_minute: f64,
    pub visibility_stratum_id: String,
    pub ground_cover_stratum_id: String,
    pub soil_moisture_stratum_id: String,
    pub illumination_stratum_id: String,
    pub season_stratum_id: String,
    pub terrain_stratum_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SweepWidthCalibrationProfile {
    pub profile_id: String,
    pub profile_sha256: String,
    pub context: FieldSurveyContext,
    /// Fit of p(r) = b * exp(-k * r^2); b is dimensionless.
    pub detection_probability_at_transect: f64,
    /// k has units 1/m^2 when range is measured in metres.
    pub lateral_decay_per_metre_squared: f64,
    pub detection_probability_standard_error: f64,
    pub lateral_decay_standard_error: f64,
    /// Covariance between b and k; units 1/m^2.
    pub b_k_covariance: f64,
    pub fit_method_id: String,
    pub fit_software_version: String,
    pub fit_receipt_sha256: String,
    pub calibration_data_sha256: String,
    pub calibration_split_manifest_sha256: String,
    pub fit_transect_ids: Vec<String>,
    pub validation_transect_ids: Vec<String>,
    pub fit_seeded_target_count: usize,
    pub validation_seeded_target_count: usize,
    /// Brier score on held-out seeded-target trials. Does not authenticate them.
    pub validation_brier_score: f64,
    pub maximum_permitted_validation_brier_score: f64,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurveyCoverageReceipt {
    pub survey_unit_id: String,
    pub survey_unit_crs_id: String,
    pub survey_unit_area_m2: f64,
    /// Total path length after GIS-side clipping and the declared overlap audit.
    /// This is not raw GPS length.
    pub overlap_adjusted_path_length_m: f64,
    pub transect_geometry_sha256: String,
    pub survey_unit_geometry_sha256: String,
    pub spatial_union_method_id: String,
    /// Digest of the GIS-side spatial-union/overlap audit receipt.
    pub spatial_union_receipt_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSweepEvaluationInput {
    pub schema_version: String,
    pub evaluation_id: String,
    pub calibration_profile: SweepWidthCalibrationProfile,
    pub observed_context: FieldSurveyContext,
    pub coverage: SurveyCoverageReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SweepWidthInterval {
    pub confidence_level: f64,
    pub z_value: f64,
    pub standard_error_m: f64,
    pub lower_m: f64,
    pub upper_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSweepEvaluationReport {
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
    pub effective_sweep_width_interval_95: SweepWidthInterval,
    pub overlap_adjusted_path_length_m: f64,
    pub nominal_swept_area_m2: f64,
    /// Area ratio based on an externally audited, overlap-adjusted geometry.
    /// It is not archaeological occupancy probability or proof of absence.
    pub nominal_coverage_fraction: f64,
    pub validation_brier_score: f64,
    pub fit_transect_count: usize,
    pub validation_transect_count: usize,
    pub fit_seeded_target_count: usize,
    pub validation_seeded_target_count: usize,
    pub rights_status: RightsStatus,
    /// The report is a survey-effectiveness estimate, not an absence claim.
    pub archaeological_absence_claimed: bool,
    pub archaeological_identification_claimed: bool,
    pub geomorphological_surface_survival_estimated: bool,
    pub repeated_search_independence_assumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSweepError {
    WrongSchemaVersion,
    EmptyEvaluationId,
    EmptyProfileId,
    InvalidProfileDigest,
    InvalidFitReceiptDigest,
    InvalidCalibrationDataDigest,
    InvalidCalibrationSplitDigest,
    EmptyFitMethod,
    EmptyFitSoftwareVersion,
    EmptyContextField,
    UnsupportedSurveyMethod,
    InvalidSearchSpeed,
    InvalidDetectionIntercept,
    InvalidLateralDecay,
    InvalidStandardError,
    InvalidParameterCovariance,
    InvalidFitSampleCounts,
    DuplicateFitTransectId(String),
    DuplicateValidationTransectId(String),
    FitValidationTransectOverlap(String),
    InvalidValidationBrierScore,
    RightsNotVerified,
    RightsReceiptMissing,
    ContextMismatch,
    EmptySurveyUnitId,
    EmptyCrsId,
    InvalidSurveyUnitArea,
    InvalidPathLength,
    InvalidTransectGeometryDigest,
    InvalidSurveyUnitGeometryDigest,
    EmptySpatialUnionMethod,
    InvalidSpatialUnionReceiptDigest,
    InvalidCoverageRatio,
    InvalidSweepWidthArithmetic,
}

impl fmt::Display for FieldSweepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage field sweep error: {self:?}")
    }
}

impl std::error::Error for FieldSweepError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn validate_context(context: &FieldSurveyContext) -> Result<(), FieldSweepError> {
    if [
        &context.target_material_or_feature_class_id,
        &context.obtrusiveness_class_id,
        &context.crew_cohort_id,
        &context.visibility_stratum_id,
        &context.ground_cover_stratum_id,
        &context.soil_moisture_stratum_id,
        &context.illumination_stratum_id,
        &context.season_stratum_id,
        &context.terrain_stratum_id,
    ]
    .iter()
    .any(|value| !non_empty(value))
    {
        return Err(FieldSweepError::EmptyContextField);
    }
    if !finite_positive(context.search_speed_m_per_minute) {
        return Err(FieldSweepError::InvalidSearchSpeed);
    }
    Ok(())
}

fn validate_profile(
    profile: &SweepWidthCalibrationProfile,
) -> Result<(), FieldSweepError> {
    if !non_empty(&profile.profile_id) {
        return Err(FieldSweepError::EmptyProfileId);
    }
    if !valid_sha256(&profile.profile_sha256) {
        return Err(FieldSweepError::InvalidProfileDigest);
    }
    if !valid_sha256(&profile.fit_receipt_sha256) {
        return Err(FieldSweepError::InvalidFitReceiptDigest);
    }
    if !valid_sha256(&profile.calibration_data_sha256) {
        return Err(FieldSweepError::InvalidCalibrationDataDigest);
    }
    if !valid_sha256(&profile.calibration_split_manifest_sha256) {
        return Err(FieldSweepError::InvalidCalibrationSplitDigest);
    }
    if !non_empty(&profile.fit_method_id) || !non_empty(&profile.fit_software_version) {
        return Err(FieldSweepError::EmptyFitMethod);
    }
    validate_context(&profile.context)?;
    if profile.context.survey_method != SurveyMethod::FieldWalking {
        return Err(FieldSweepError::UnsupportedSurveyMethod);
    }

    let b = profile.detection_probability_at_transect;
    if !b.is_finite() || b <= 0.0 || b > 1.0 {
        return Err(FieldSweepError::InvalidDetectionIntercept);
    }
    if !finite_positive(profile.lateral_decay_per_metre_squared) {
        return Err(FieldSweepError::InvalidLateralDecay);
    }
    if !profile.detection_probability_standard_error.is_finite()
        || profile.detection_probability_standard_error < 0.0
        || !profile.lateral_decay_standard_error.is_finite()
        || profile.lateral_decay_standard_error < 0.0
    {
        return Err(FieldSweepError::InvalidStandardError);
    }
    let covariance_limit = profile.detection_probability_standard_error
        * profile.lateral_decay_standard_error;
    if !profile.b_k_covariance.is_finite()
        || profile.b_k_covariance.abs() > covariance_limit + COVARIANCE_TOLERANCE
    {
        return Err(FieldSweepError::InvalidParameterCovariance);
    }

    if profile.fit_transect_ids.len() < MIN_FIT_TRANSECTS
        || profile.validation_transect_ids.len() < MIN_VALIDATION_TRANSECTS
        || profile.fit_seeded_target_count < MIN_FIT_TARGETS
        || profile.validation_seeded_target_count < MIN_VALIDATION_TARGETS
    {
        return Err(FieldSweepError::InvalidFitSampleCounts);
    }

    let mut fit_ids = BTreeSet::new();
    for id in &profile.fit_transect_ids {
        if !non_empty(id) || !fit_ids.insert(id.clone()) {
            return Err(FieldSweepError::DuplicateFitTransectId(id.clone()));
        }
    }
    let mut validation_ids = BTreeSet::new();
    for id in &profile.validation_transect_ids {
        if !non_empty(id) || !validation_ids.insert(id.clone()) {
            return Err(FieldSweepError::DuplicateValidationTransectId(id.clone()));
        }
        if fit_ids.contains(id) {
            return Err(FieldSweepError::FitValidationTransectOverlap(id.clone()));
        }
    }

    if !profile.validation_brier_score.is_finite()
        || !(0.0..=1.0).contains(&profile.validation_brier_score)
        || !profile.maximum_permitted_validation_brier_score.is_finite()
        || !(0.0..=1.0).contains(&profile.maximum_permitted_validation_brier_score)
        || profile.validation_brier_score > profile.maximum_permitted_validation_brier_score
    {
        return Err(FieldSweepError::InvalidValidationBrierScore);
    }
    if profile.rights_status != RightsStatus::Verified {
        return Err(FieldSweepError::RightsNotVerified);
    }
    if profile
        .rights_receipt_ref
        .as_deref()
        .is_none_or(|value| !non_empty(value))
    {
        return Err(FieldSweepError::RightsReceiptMissing);
    }
    Ok(())
}

fn validate_coverage(receipt: &SurveyCoverageReceipt) -> Result<(), FieldSweepError> {
    if !non_empty(&receipt.survey_unit_id) {
        return Err(FieldSweepError::EmptySurveyUnitId);
    }
    if !non_empty(&receipt.survey_unit_crs_id) {
        return Err(FieldSweepError::EmptyCrsId);
    }
    if !finite_positive(receipt.survey_unit_area_m2) {
        return Err(FieldSweepError::InvalidSurveyUnitArea);
    }
    if !finite_positive(receipt.overlap_adjusted_path_length_m) {
        return Err(FieldSweepError::InvalidPathLength);
    }
    if !valid_sha256(&receipt.transect_geometry_sha256) {
        return Err(FieldSweepError::InvalidTransectGeometryDigest);
    }
    if !valid_sha256(&receipt.survey_unit_geometry_sha256) {
        return Err(FieldSweepError::InvalidSurveyUnitGeometryDigest);
    }
    if !non_empty(&receipt.spatial_union_method_id) {
        return Err(FieldSweepError::EmptySpatialUnionMethod);
    }
    if !valid_sha256(&receipt.spatial_union_receipt_sha256) {
        return Err(FieldSweepError::InvalidSpatialUnionReceiptDigest);
    }
    Ok(())
}

/// Computes W = integral[-infinity,+infinity] b exp(-k*r^2) dr.
///
/// A first-order delta-method standard error is propagated from the supplied
/// fitted-parameter covariance matrix. This normal approximation is not a
/// substitute for bootstrap/profile-likelihood intervals when b is near 0/1
/// or the calibration sample is small.
pub fn effective_sweep_width(
    profile: &SweepWidthCalibrationProfile,
) -> Result<SweepWidthInterval, FieldSweepError> {
    let b = profile.detection_probability_at_transect;
    let k = profile.lateral_decay_per_metre_squared;
    if !b.is_finite() || b <= 0.0 || b > 1.0 {
        return Err(FieldSweepError::InvalidDetectionIntercept);
    }
    if !finite_positive(k) {
        return Err(FieldSweepError::InvalidLateralDecay);
    }

    let sqrt_pi = std::f64::consts::PI.sqrt();
    let width = b * (std::f64::consts::PI / k).sqrt();
    let d_w_db = (std::f64::consts::PI / k).sqrt();
    let d_w_dk = -0.5 * b * sqrt_pi / k.powf(1.5);
    let se_b = profile.detection_probability_standard_error;
    let se_k = profile.lateral_decay_standard_error;
    let variance = d_w_db.powi(2) * se_b.powi(2)
        + d_w_dk.powi(2) * se_k.powi(2)
        + 2.0 * d_w_db * d_w_dk * profile.b_k_covariance;

    if !width.is_finite() || width <= 0.0 || !variance.is_finite() || variance < -COVARIANCE_TOLERANCE {
        return Err(FieldSweepError::InvalidSweepWidthArithmetic);
    }
    let standard_error = variance.max(0.0).sqrt();
    let half = NORMAL_95_Z * standard_error;
    let lower = (width - half).max(0.0);
    let upper = width + half;
    if !standard_error.is_finite() || !upper.is_finite() || lower > upper {
        return Err(FieldSweepError::InvalidSweepWidthArithmetic);
    }

    Ok(SweepWidthInterval {
        confidence_level: 0.95,
        z_value: NORMAL_95_Z,
        standard_error_m: standard_error,
        lower_m: lower,
        upper_m: upper,
    })
}

pub fn probability_of_detection_at_range(
    b: f64,
    k_per_m2: f64,
    absolute_range_m: f64,
) -> Result<f64, FieldSweepError> {
    if !b.is_finite() || b <= 0.0 || b > 1.0 {
        return Err(FieldSweepError::InvalidDetectionIntercept);
    }
    if !finite_positive(k_per_m2) {
        return Err(FieldSweepError::InvalidLateralDecay);
    }
    if !absolute_range_m.is_finite() || absolute_range_m < 0.0 {
        return Err(FieldSweepError::InvalidPathLength);
    }
    let p = b * (-k_per_m2 * absolute_range_m.powi(2)).exp();
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(FieldSweepError::InvalidSweepWidthArithmetic);
    }
    Ok(p)
}

pub fn evaluate_field_sweep(
    input: &FieldSweepEvaluationInput,
) -> Result<FieldSweepEvaluationReport, FieldSweepError> {
    if input.schema_version != FIELD_SWEEP_SCHEMA_VERSION {
        return Err(FieldSweepError::WrongSchemaVersion);
    }
    if !non_empty(&input.evaluation_id) {
        return Err(FieldSweepError::EmptyEvaluationId);
    }
    validate_profile(&input.calibration_profile)?;
    validate_coverage(&input.coverage)?;

    if input.observed_context != input.calibration_profile.context {
        return Err(FieldSweepError::ContextMismatch);
    }

    let interval = effective_sweep_width(&input.calibration_profile)?;
    let width = input.calibration_profile.detection_probability_at_transect
        * (std::f64::consts::PI
            / input.calibration_profile.lateral_decay_per_metre_squared)
            .sqrt();
    let swept_area = width * input.coverage.overlap_adjusted_path_length_m;
    let nominal_coverage = swept_area / input.coverage.survey_unit_area_m2;

    // This ratio uses a declared GIS-side overlap/clipping-adjusted path length.
    // If it exceeds one, the input's "adjusted" geometry receipt is inconsistent.
    if !swept_area.is_finite()
        || !nominal_coverage.is_finite()
        || nominal_coverage <= 0.0
        || nominal_coverage > 1.0 + COVERAGE_TOLERANCE
    {
        return Err(FieldSweepError::InvalidCoverageRatio);
    }

    Ok(FieldSweepEvaluationReport {
        schema_version: FIELD_SWEEP_REPORT_SCHEMA_VERSION.to_owned(),
        evaluation_id: input.evaluation_id.clone(),
        profile_id: input.calibration_profile.profile_id.clone(),
        profile_sha256: input.calibration_profile.profile_sha256.clone(),
        calibration_data_sha256: input.calibration_profile.calibration_data_sha256.clone(),
        calibration_split_manifest_sha256: input
            .calibration_profile
            .calibration_split_manifest_sha256
            .clone(),
        fit_receipt_sha256: input.calibration_profile.fit_receipt_sha256.clone(),
        survey_unit_id: input.coverage.survey_unit_id.clone(),
        transect_geometry_sha256: input.coverage.transect_geometry_sha256.clone(),
        survey_unit_geometry_sha256: input.coverage.survey_unit_geometry_sha256.clone(),
        spatial_union_receipt_sha256: input.coverage.spatial_union_receipt_sha256.clone(),
        context_match: true,
        effective_sweep_width_m: width,
        effective_sweep_width_interval_95: interval,
        overlap_adjusted_path_length_m: input.coverage.overlap_adjusted_path_length_m,
        nominal_swept_area_m2: swept_area,
        nominal_coverage_fraction: nominal_coverage.min(1.0),
        validation_brier_score: input.calibration_profile.validation_brier_score,
        fit_transect_count: input.calibration_profile.fit_transect_ids.len(),
        validation_transect_count: input.calibration_profile.validation_transect_ids.len(),
        fit_seeded_target_count: input.calibration_profile.fit_seeded_target_count,
        validation_seeded_target_count: input.calibration_profile.validation_seeded_target_count,
        rights_status: input.calibration_profile.rights_status,
        archaeological_absence_claimed: false,
        archaeological_identification_claimed: false,
        geomorphological_surface_survival_estimated: false,
        repeated_search_independence_assumed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> FieldSurveyContext {
        FieldSurveyContext {
            target_material_or_feature_class_id: "ceramic_sherd".into(),
            obtrusiveness_class_id: "small_red_shard".into(),
            survey_method: SurveyMethod::FieldWalking,
            crew_cohort_id: "crew-alpha".into(),
            search_speed_m_per_minute: 45.0,
            visibility_stratum_id: "bare_dry_ground".into(),
            ground_cover_stratum_id: "sparse_stubble".into(),
            soil_moisture_stratum_id: "dry".into(),
            illumination_stratum_id: "overcast_diffuse".into(),
            season_stratum_id: "post_harvest_autumn".into(),
            terrain_stratum_id: "low_relief_terrace".into(),
        }
    }

    fn profile() -> SweepWidthCalibrationProfile {
        SweepWidthCalibrationProfile {
            profile_id: "crew-alpha-ceramic-bare-ground-v1".into(),
            profile_sha256: "a".repeat(64),
            context: context(),
            detection_probability_at_transect: 0.82,
            lateral_decay_per_metre_squared: 0.05,
            detection_probability_standard_error: 0.02,
            lateral_decay_standard_error: 0.004,
            b_k_covariance: -0.00002,
            fit_method_id: "nonlinear_least_squares_bounded".into(),
            fit_software_version: "external-fit-tool-1.0".into(),
            fit_receipt_sha256: "b".repeat(64),
            calibration_data_sha256: "c".repeat(64),
            calibration_split_manifest_sha256: "d".repeat(64),
            fit_transect_ids: vec!["fit-1".into(), "fit-2".into(), "fit-3".into()],
            validation_transect_ids: vec!["valid-1".into(), "valid-2".into()],
            fit_seeded_target_count: 60,
            validation_seeded_target_count: 30,
            validation_brier_score: 0.12,
            maximum_permitted_validation_brier_score: 0.20,
            rights_status: RightsStatus::Verified,
            rights_receipt_ref: Some("urn:synthetic:rights:calibration".into()),
        }
    }

    fn input() -> FieldSweepEvaluationInput {
        FieldSweepEvaluationInput {
            schema_version: FIELD_SWEEP_SCHEMA_VERSION.into(),
            evaluation_id: "sweep-eval-001".into(),
            calibration_profile: profile(),
            observed_context: context(),
            coverage: SurveyCoverageReceipt {
                survey_unit_id: "unit-01".into(),
                survey_unit_crs_id: "EPSG:32636".into(),
                survey_unit_area_m2: 100_000.0,
                overlap_adjusted_path_length_m: 1_000.0,
                transect_geometry_sha256: "e".repeat(64),
                survey_unit_geometry_sha256: "f".repeat(64),
                spatial_union_method_id: "buffered_transect_union_v1".into(),
                spatial_union_receipt_sha256: "1".repeat(64),
            },
        }
    }

    #[test]
    fn analytical_sweep_width_matches_known_value() {
        let interval = effective_sweep_width(&profile()).unwrap();
        let expected = 0.82 * (std::f64::consts::PI / 0.05_f64).sqrt();
        assert!((interval.lower_m - (expected - NORMAL_95_Z * interval.standard_error_m).max(0.0)).abs() < 1e-12);
        assert!((interval.upper_m - (expected + NORMAL_95_Z * interval.standard_error_m)).abs() < 1e-12);
        assert!(interval.lower_m <= expected && expected <= interval.upper_m);
    }

    #[test]
    fn detection_probability_decreases_with_lateral_range() {
        let near = probability_of_detection_at_range(0.82, 0.05, 1.0).unwrap();
        let far = probability_of_detection_at_range(0.82, 0.05, 5.0).unwrap();
        assert!(near < 0.82);
        assert!(far < near);
    }

    #[test]
    fn evaluation_preserves_calibration_and_spatial_receipts() {
        let report = evaluate_field_sweep(&input()).unwrap();
        assert!(report.context_match);
        assert!(!report.archaeological_absence_claimed);
        assert!(!report.archaeological_identification_claimed);
        assert!(!report.geomorphological_surface_survival_estimated);
        assert!(!report.repeated_search_independence_assumed);
        assert_eq!(report.profile_sha256, "a".repeat(64));
        assert_eq!(report.spatial_union_receipt_sha256, "1".repeat(64));
        assert!(report.nominal_coverage_fraction > 0.0 && report.nominal_coverage_fraction <= 1.0);
    }

    #[test]
    fn exact_context_mismatch_is_rejected() {
        let mut value = input();
        value.observed_context.visibility_stratum_id = "dense_vegetation".into();
        assert_eq!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::ContextMismatch)
        );
    }

    #[test]
    fn non_fieldwalking_method_cannot_reuse_profile() {
        let mut value = input();
        value.calibration_profile.context.survey_method = SurveyMethod::GroundMagnetometry;
        value.observed_context.survey_method = SurveyMethod::GroundMagnetometry;
        assert_eq!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::UnsupportedSurveyMethod)
        );
    }

    #[test]
    fn invalid_b_or_k_fails_closed() {
        let mut value = profile();
        value.detection_probability_at_transect = 1.2;
        assert_eq!(
            effective_sweep_width(&value),
            Err(FieldSweepError::InvalidDetectionIntercept)
        );
        value = profile();
        value.lateral_decay_per_metre_squared = 0.0;
        assert_eq!(
            effective_sweep_width(&value),
            Err(FieldSweepError::InvalidLateralDecay)
        );
    }

    #[test]
    fn non_positive_semidefinite_covariance_is_rejected() {
        let mut value = profile();
        value.b_k_covariance = 1.0;
        assert_eq!(
            validate_profile(&value),
            Err(FieldSweepError::InvalidParameterCovariance)
        );
    }

    #[test]
    fn fit_and_validation_transects_must_be_disjoint() {
        let mut value = input();
        value.calibration_profile.validation_transect_ids[0] = "fit-1".into();
        assert!(matches!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::FitValidationTransectOverlap(_))
        ));
    }

    #[test]
    fn insufficient_fit_or_validation_sample_is_rejected() {
        let mut value = input();
        value.calibration_profile.fit_transect_ids.truncate(2);
        assert_eq!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::InvalidFitSampleCounts)
        );
    }

    #[test]
    fn poor_validation_fit_is_rejected() {
        let mut value = input();
        value.calibration_profile.validation_brier_score = 0.35;
        assert_eq!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::InvalidValidationBrierScore)
        );
    }

    #[test]
    fn non_verified_rights_are_rejected() {
        let mut value = input();
        value.calibration_profile.rights_status = RightsStatus::Pending;
        value.calibration_profile.rights_receipt_ref = None;
        assert_eq!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::RightsNotVerified)
        );
    }

    #[test]
    fn inconsistent_nominal_coverage_is_rejected() {
        let mut value = input();
        value.coverage.overlap_adjusted_path_length_m = 100_000.0;
        assert_eq!(
            evaluate_field_sweep(&value),
            Err(FieldSweepError::InvalidCoverageRatio)
        );
    }
}
