// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Context-conditioned archaeological prospection representation/model routing.
//!
//! The selector chooses only among exact-context profiles evaluated on
//! independent held-out sites under the same task, metric, reference, sensor
//! and review-budget contract. It uses per-site floors and worst-site primary
//! utility; it never selects on an outer test set and abstains when no profile
//! is qualified. It does not train models or authenticate evaluation evidence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_ROUTING_SCHEMA_VERSION: &str = "luminous.heritage-routing.v0.1";
pub const HERITAGE_ROUTING_REPORT_SCHEMA_VERSION: &str =
    "luminous.heritage-routing-report.v0.1";
pub const MAX_ROUTING_PROFILES: usize = 1_000;
pub const MINIMUM_INDEPENDENT_VALIDATION_SITES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorModality {
    AirborneLidar,
    SatelliteOptical,
    SatelliteSar,
    UavPhotogrammetry,
    ThermalImaging,
    GroundGeophysics,
    Multimodal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationRole {
    Training,
    SelectorValidation,
    OuterTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitProtocol {
    HeldOutSite,
    HeldOutRegion,
    PixelRandomInterpolation,
    RandomTile,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandscapeContext {
    pub biome_or_ecology_id: String,
    pub geomorphology_id: String,
    pub relief_complexity_stratum_id: String,
    pub vegetation_or_canopy_stratum_id: String,
    pub preservation_or_visibility_stratum_id: String,
    pub resolution_class_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitePerformance {
    pub validation_site_id: String,
    /// Preregistered, higher-is-better primary-task utility in [0,1].
    /// Its orientation/normalisation is part of metric_definition_sha256.
    pub primary_metric_utility_lower_bound: f64,
    /// Conservative precision lower bound in [0,1] at the declared operating point.
    pub precision_lower_bound: f64,
    pub false_candidates_per_km2: f64,
    pub coverage_fraction: f64,
    pub review_seconds_per_candidate: f64,
    pub calibration_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationProfile {
    pub profile_id: String,
    pub task_id: String,
    pub feature_class_id: String,
    pub sensor_modality: SensorModality,
    pub channel_semantics_sha256: String,
    pub context: LandscapeContext,
    pub representation_transform_sha256: String,
    pub model_build_sha256: String,
    pub metric_definition_sha256: String,
    pub primary_metric_id: String,
    pub reference_tier_id: String,
    pub review_budget_id: String,
    pub evaluation_role: EvaluationRole,
    pub split_protocol: SplitProtocol,
    pub training_site_ids: Vec<String>,
    pub validation_site_results: Vec<SitePerformance>,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingQualityFloors {
    pub minimum_primary_metric_utility_lower_bound: f64,
    pub minimum_precision_lower_bound: f64,
    pub maximum_false_candidates_per_km2: f64,
    pub minimum_coverage_fraction: f64,
    pub maximum_review_seconds_per_candidate: f64,
    pub minimum_independent_validation_sites: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationSelectionRequest {
    pub schema_version: String,
    pub request_id: String,
    pub task_id: String,
    pub feature_class_id: String,
    pub sensor_modality: SensorModality,
    pub channel_semantics_sha256: String,
    pub context: LandscapeContext,
    pub metric_definition_sha256: String,
    pub primary_metric_id: String,
    pub reference_tier_id: String,
    pub review_budget_id: String,
    pub quality_floors: RoutingQualityFloors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDisposition {
    SelectedQualifiedProfile,
    AbstainNoQualifiedProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileRejectionReason {
    TaskMismatch,
    FeatureClassMismatch,
    SensorModalityMismatch,
    ChannelSemanticsMismatch,
    LandscapeContextMismatch,
    MetricDefinitionMismatch,
    PrimaryMetricMismatch,
    ReferenceTierMismatch,
    ReviewBudgetMismatch,
    NotSelectorValidationRole,
    SplitNotSiteOrRegionHeldOut,
    TrainingValidationSiteOverlap,
    InsufficientIndependentValidationSites,
    RightsNotVerified,
    RightsReceiptMissing,
    PrimaryMetricFloorNotMet,
    PrecisionFloorNotMet,
    FalseCandidateCeilingExceeded,
    CoverageFloorNotMet,
    ReviewCostCeilingExceeded,
    CalibrationFailed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectedProfile {
    pub profile_id: String,
    pub reasons: Vec<ProfileRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepresentationSelectionReport {
    pub schema_version: String,
    pub request_id: String,
    pub disposition: RoutingDisposition,
    pub selected_profile_id: Option<String>,
    pub selected_representation_transform_sha256: Option<String>,
    pub selected_model_build_sha256: Option<String>,
    pub selected_worst_site_primary_metric_utility_lower_bound: Option<f64>,
    pub selected_worst_site_precision_lower_bound: Option<f64>,
    pub selected_worst_site_false_candidates_per_km2: Option<f64>,
    pub candidate_profile_count: usize,
    pub qualified_profile_count: usize,
    pub rejected_profiles: Vec<RejectedProfile>,
    /// The selector never uses an outer test set and never silently falls back.
    pub outer_test_data_used_for_selection: bool,
    pub silent_fallback_performed: bool,
    pub archaeological_identification_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepresentationRoutingError {
    WrongSchemaVersion,
    EmptyRequestId,
    EmptyTaskId,
    EmptyFeatureClassId,
    EmptyPrimaryMetricId,
    EmptyReferenceTierId,
    EmptyReviewBudgetId,
    InvalidChannelSemanticsDigest,
    InvalidMetricDefinitionDigest,
    InvalidContextField,
    InvalidQualityFloor,
    InvalidMinimumValidationSiteCount,
    NoProfiles,
    TooManyProfiles,
    EmptyProfileId,
    DuplicateProfileId(String),
    EmptyTrainingSiteId,
    EmptyValidationSiteId,
    DuplicateTrainingSiteId(String),
    DuplicateValidationSiteId(String),
    InvalidProfileDigest { profile_id: String },
    InvalidSiteMetric { profile_id: String, site_id: String },
    InvalidRightsReceipt { profile_id: String },
}

impl fmt::Display for RepresentationRoutingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage representation routing error: {self:?}")
    }
}

impl std::error::Error for RepresentationRoutingError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_unit_interval(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn validate_context(context: &LandscapeContext) -> Result<(), RepresentationRoutingError> {
    if [
        &context.biome_or_ecology_id,
        &context.geomorphology_id,
        &context.relief_complexity_stratum_id,
        &context.vegetation_or_canopy_stratum_id,
        &context.preservation_or_visibility_stratum_id,
        &context.resolution_class_id,
    ]
    .iter()
    .any(|value| !non_empty(value))
    {
        return Err(RepresentationRoutingError::InvalidContextField);
    }
    Ok(())
}

fn validate_request(
    request: &RepresentationSelectionRequest,
) -> Result<(), RepresentationRoutingError> {
    if request.schema_version != HERITAGE_ROUTING_SCHEMA_VERSION {
        return Err(RepresentationRoutingError::WrongSchemaVersion);
    }
    if !non_empty(&request.request_id) {
        return Err(RepresentationRoutingError::EmptyRequestId);
    }
    if !non_empty(&request.task_id) {
        return Err(RepresentationRoutingError::EmptyTaskId);
    }
    if !non_empty(&request.feature_class_id) {
        return Err(RepresentationRoutingError::EmptyFeatureClassId);
    }
    if !non_empty(&request.primary_metric_id) {
        return Err(RepresentationRoutingError::EmptyPrimaryMetricId);
    }
    if !non_empty(&request.reference_tier_id) {
        return Err(RepresentationRoutingError::EmptyReferenceTierId);
    }
    if !non_empty(&request.review_budget_id) {
        return Err(RepresentationRoutingError::EmptyReviewBudgetId);
    }
    if !valid_sha256(&request.channel_semantics_sha256) {
        return Err(RepresentationRoutingError::InvalidChannelSemanticsDigest);
    }
    if !valid_sha256(&request.metric_definition_sha256) {
        return Err(RepresentationRoutingError::InvalidMetricDefinitionDigest);
    }
    validate_context(&request.context)?;

    let floors = &request.quality_floors;
    if !valid_unit_interval(floors.minimum_primary_metric_utility_lower_bound)
        || !valid_unit_interval(floors.minimum_precision_lower_bound)
        || !valid_unit_interval(floors.minimum_coverage_fraction)
        || !floors.maximum_false_candidates_per_km2.is_finite()
        || floors.maximum_false_candidates_per_km2 < 0.0
        || !floors.maximum_review_seconds_per_candidate.is_finite()
        || floors.maximum_review_seconds_per_candidate <= 0.0
    {
        return Err(RepresentationRoutingError::InvalidQualityFloor);
    }
    if floors.minimum_independent_validation_sites
        < MINIMUM_INDEPENDENT_VALIDATION_SITES
    {
        return Err(RepresentationRoutingError::InvalidMinimumValidationSiteCount);
    }
    Ok(())
}

fn validate_profiles(
    profiles: &[RepresentationProfile],
) -> Result<(), RepresentationRoutingError> {
    if profiles.is_empty() {
        return Err(RepresentationRoutingError::NoProfiles);
    }
    if profiles.len() > MAX_ROUTING_PROFILES {
        return Err(RepresentationRoutingError::TooManyProfiles);
    }

    let mut profile_ids = BTreeSet::new();
    for profile in profiles {
        if !non_empty(&profile.profile_id) {
            return Err(RepresentationRoutingError::EmptyProfileId);
        }
        if !profile_ids.insert(profile.profile_id.clone()) {
            return Err(RepresentationRoutingError::DuplicateProfileId(
                profile.profile_id.clone(),
            ));
        }
        for digest in [
            &profile.channel_semantics_sha256,
            &profile.representation_transform_sha256,
            &profile.model_build_sha256,
            &profile.metric_definition_sha256,
        ] {
            if !valid_sha256(digest) {
                return Err(RepresentationRoutingError::InvalidProfileDigest {
                    profile_id: profile.profile_id.clone(),
                });
            }
        }
        validate_context(&profile.context)?;

        let mut train_sites = BTreeSet::new();
        for site_id in &profile.training_site_ids {
            if !non_empty(site_id) {
                return Err(RepresentationRoutingError::EmptyTrainingSiteId);
            }
            if !train_sites.insert(site_id.clone()) {
                return Err(RepresentationRoutingError::DuplicateTrainingSiteId(
                    site_id.clone(),
                ));
            }
        }

        let mut validation_sites = BTreeSet::new();
        for result in &profile.validation_site_results {
            if !non_empty(&result.validation_site_id) {
                return Err(RepresentationRoutingError::EmptyValidationSiteId);
            }
            if !validation_sites.insert(result.validation_site_id.clone()) {
                return Err(RepresentationRoutingError::DuplicateValidationSiteId(
                    result.validation_site_id.clone(),
                ));
            }
            if !result.primary_metric_utility_lower_bound.is_finite()
                || !(0.0..=1.0).contains(&result.primary_metric_utility_lower_bound)
                || !valid_unit_interval(result.precision_lower_bound)
                || !result.false_candidates_per_km2.is_finite()
                || result.false_candidates_per_km2 < 0.0
                || !valid_unit_interval(result.coverage_fraction)
                || !result.review_seconds_per_candidate.is_finite()
                || result.review_seconds_per_candidate < 0.0
            {
                return Err(RepresentationRoutingError::InvalidSiteMetric {
                    profile_id: profile.profile_id.clone(),
                    site_id: result.validation_site_id.clone(),
                });
            }
        }

        if profile.rights_status == RightsStatus::Verified
            && profile
                .rights_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            return Err(RepresentationRoutingError::InvalidRightsReceipt {
                profile_id: profile.profile_id.clone(),
            });
        }
    }
    Ok(())
}

fn rejection_reasons(
    request: &RepresentationSelectionRequest,
    profile: &RepresentationProfile,
) -> Vec<ProfileRejectionReason> {
    let mut reasons = Vec::new();
    if profile.task_id != request.task_id {
        reasons.push(ProfileRejectionReason::TaskMismatch);
    }
    if profile.feature_class_id != request.feature_class_id {
        reasons.push(ProfileRejectionReason::FeatureClassMismatch);
    }
    if profile.sensor_modality != request.sensor_modality {
        reasons.push(ProfileRejectionReason::SensorModalityMismatch);
    }
    if profile.channel_semantics_sha256 != request.channel_semantics_sha256 {
        reasons.push(ProfileRejectionReason::ChannelSemanticsMismatch);
    }
    if profile.context != request.context {
        reasons.push(ProfileRejectionReason::LandscapeContextMismatch);
    }
    if profile.metric_definition_sha256 != request.metric_definition_sha256 {
        reasons.push(ProfileRejectionReason::MetricDefinitionMismatch);
    }
    if profile.primary_metric_id != request.primary_metric_id {
        reasons.push(ProfileRejectionReason::PrimaryMetricMismatch);
    }
    if profile.reference_tier_id != request.reference_tier_id {
        reasons.push(ProfileRejectionReason::ReferenceTierMismatch);
    }
    if profile.review_budget_id != request.review_budget_id {
        reasons.push(ProfileRejectionReason::ReviewBudgetMismatch);
    }
    if profile.evaluation_role != EvaluationRole::SelectorValidation {
        reasons.push(ProfileRejectionReason::NotSelectorValidationRole);
    }
    if !matches!(
        profile.split_protocol,
        SplitProtocol::HeldOutSite | SplitProtocol::HeldOutRegion
    ) {
        reasons.push(ProfileRejectionReason::SplitNotSiteOrRegionHeldOut);
    }

    let training = profile
        .training_site_ids
        .iter()
        .collect::<BTreeSet<_>>();
    if profile
        .validation_site_results
        .iter()
        .any(|site| training.contains(&site.validation_site_id))
    {
        reasons.push(ProfileRejectionReason::TrainingValidationSiteOverlap);
    }

    if profile.validation_site_results.len()
        < request.quality_floors.minimum_independent_validation_sites
    {
        reasons.push(ProfileRejectionReason::InsufficientIndependentValidationSites);
    }
    if profile.rights_status != RightsStatus::Verified {
        reasons.push(ProfileRejectionReason::RightsNotVerified);
    }
    if profile
        .rights_receipt_ref
        .as_deref()
        .is_none_or(|value| !non_empty(value))
    {
        reasons.push(ProfileRejectionReason::RightsReceiptMissing);
    }

    for site in &profile.validation_site_results {
        let floors = &request.quality_floors;
        if site.primary_metric_utility_lower_bound
            < floors.minimum_primary_metric_utility_lower_bound
        {
            reasons.push(ProfileRejectionReason::PrimaryMetricFloorNotMet);
        }
        if site.precision_lower_bound < floors.minimum_precision_lower_bound {
            reasons.push(ProfileRejectionReason::PrecisionFloorNotMet);
        }
        if site.false_candidates_per_km2 > floors.maximum_false_candidates_per_km2 {
            reasons.push(ProfileRejectionReason::FalseCandidateCeilingExceeded);
        }
        if site.coverage_fraction < floors.minimum_coverage_fraction {
            reasons.push(ProfileRejectionReason::CoverageFloorNotMet);
        }
        if site.review_seconds_per_candidate > floors.maximum_review_seconds_per_candidate {
            reasons.push(ProfileRejectionReason::ReviewCostCeilingExceeded);
        }
        if !site.calibration_passed {
            reasons.push(ProfileRejectionReason::CalibrationFailed);
        }
    }

    reasons.sort_by_key(|reason| *reason as u8);
    reasons.dedup();
    reasons
}

fn worst_site_utility(profile: &RepresentationProfile) -> f64 {
    profile
        .validation_site_results
        .iter()
        .map(|site| site.primary_metric_utility_lower_bound)
        .fold(f64::INFINITY, f64::min)
}

fn worst_site_precision(profile: &RepresentationProfile) -> f64 {
    profile
        .validation_site_results
        .iter()
        .map(|site| site.precision_lower_bound)
        .fold(f64::INFINITY, f64::min)
}

fn worst_site_false_candidates(profile: &RepresentationProfile) -> f64 {
    profile
        .validation_site_results
        .iter()
        .map(|site| site.false_candidates_per_km2)
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Select an exact-context representation/model profile from selector-validation
/// summaries. The function does not inspect any outer-test results. Each
/// profile must be site/region held-out, rights-verified, and meet every
/// per-site quality/cost/calibration floor. No matching profile => abstain.
pub fn select_heritage_representation(
    request: &RepresentationSelectionRequest,
    profiles: &[RepresentationProfile],
) -> Result<RepresentationSelectionReport, RepresentationRoutingError> {
    validate_request(request)?;
    validate_profiles(profiles)?;

    let mut qualified = Vec::new();
    let mut rejected_profiles = Vec::new();

    for profile in profiles {
        let reasons = rejection_reasons(request, profile);
        if reasons.is_empty() {
            qualified.push(profile);
        } else {
            rejected_profiles.push(RejectedProfile {
                profile_id: profile.profile_id.clone(),
                reasons,
            });
        }
    }

    qualified.sort_by(|left, right| {
        worst_site_utility(right)
            .total_cmp(&worst_site_utility(left))
            .then_with(|| {
                worst_site_precision(right).total_cmp(&worst_site_precision(left))
            })
            .then_with(|| {
                worst_site_false_candidates(left)
                    .total_cmp(&worst_site_false_candidates(right))
            })
            .then_with(|| left.profile_id.cmp(&right.profile_id))
    });
    rejected_profiles.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));

    let selected = qualified.first().copied();
    let (disposition, id, transform, model, utility, precision, false_candidates) =
        if let Some(profile) = selected {
            (
                RoutingDisposition::SelectedQualifiedProfile,
                Some(profile.profile_id.clone()),
                Some(profile.representation_transform_sha256.clone()),
                Some(profile.model_build_sha256.clone()),
                Some(worst_site_utility(profile)),
                Some(worst_site_precision(profile)),
                Some(worst_site_false_candidates(profile)),
            )
        } else {
            (
                RoutingDisposition::AbstainNoQualifiedProfile,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        };

    Ok(RepresentationSelectionReport {
        schema_version: HERITAGE_ROUTING_REPORT_SCHEMA_VERSION.to_owned(),
        request_id: request.request_id.clone(),
        disposition,
        selected_profile_id: id,
        selected_representation_transform_sha256: transform,
        selected_model_build_sha256: model,
        selected_worst_site_primary_metric_utility_lower_bound: utility,
        selected_worst_site_precision_lower_bound: precision,
        selected_worst_site_false_candidates_per_km2: false_candidates,
        candidate_profile_count: profiles.len(),
        qualified_profile_count: qualified.len(),
        rejected_profiles,
        outer_test_data_used_for_selection: false,
        silent_fallback_performed: false,
        archaeological_identification_claimed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> LandscapeContext {
        LandscapeContext {
            biome_or_ecology_id: "tropical_montane_forest".into(),
            geomorphology_id: "rugged_mountain".into(),
            relief_complexity_stratum_id: "high_relief".into(),
            vegetation_or_canopy_stratum_id: "dense_canopy".into(),
            preservation_or_visibility_stratum_id: "lidar_observable".into(),
            resolution_class_id: "dtm_0p5m".into(),
        }
    }

    fn request() -> RepresentationSelectionRequest {
        RepresentationSelectionRequest {
            schema_version: HERITAGE_ROUTING_SCHEMA_VERSION.into(),
            request_id: "route-request-001".into(),
            task_id: "stone_wall_detection".into(),
            feature_class_id: "dry_stone_wall".into(),
            sensor_modality: SensorModality::AirborneLidar,
            channel_semantics_sha256: "a".repeat(64),
            context: context(),
            metric_definition_sha256: "b".repeat(64),
            primary_metric_id: "object_map_50_95".into(),
            reference_tier_id: "expert_lidar_interpretation".into(),
            review_budget_id: "100_candidates_per_analyst_hour".into(),
            quality_floors: RoutingQualityFloors {
                minimum_primary_metric_utility_lower_bound: 0.50,
                minimum_precision_lower_bound: 0.75,
                maximum_false_candidates_per_km2: 5.0,
                minimum_coverage_fraction: 0.90,
                maximum_review_seconds_per_candidate: 60.0,
                minimum_independent_validation_sites: 3,
            },
        }
    }

    fn profile(profile_id: &str, scores: &[f64]) -> RepresentationProfile {
        RepresentationProfile {
            profile_id: profile_id.into(),
            task_id: "stone_wall_detection".into(),
            feature_class_id: "dry_stone_wall".into(),
            sensor_modality: SensorModality::AirborneLidar,
            channel_semantics_sha256: "a".repeat(64),
            context: context(),
            representation_transform_sha256: "c".repeat(64),
            model_build_sha256: "d".repeat(64),
            metric_definition_sha256: "b".repeat(64),
            primary_metric_id: "object_map_50_95".into(),
            reference_tier_id: "expert_lidar_interpretation".into(),
            review_budget_id: "100_candidates_per_analyst_hour".into(),
            evaluation_role: EvaluationRole::SelectorValidation,
            split_protocol: SplitProtocol::HeldOutSite,
            training_site_ids: vec!["train-a".into(), "train-b".into()],
            validation_site_results: scores
                .iter()
                .enumerate()
                .map(|(index, score)| SitePerformance {
                    validation_site_id: format!("validation-{index}"),
                    primary_metric_utility_lower_bound: *score,
                    precision_lower_bound: 0.80,
                    false_candidates_per_km2: 3.0,
                    coverage_fraction: 0.95,
                    review_seconds_per_candidate: 40.0,
                    calibration_passed: true,
                })
                .collect(),
            rights_status: RightsStatus::Verified,
            rights_receipt_ref: Some("urn:synthetic:rights:profile".into()),
        }
    }

    #[test]
    fn selects_profile_by_worst_site_not_pooled_mean() {
        let profiles = vec![
            profile("high-mean-fragile", &[0.95, 0.95, 0.51]),
            profile("stable", &[0.70, 0.72, 0.68]),
        ];
        let report = select_heritage_representation(&request(), &profiles).unwrap();
        assert_eq!(
            report.disposition,
            RoutingDisposition::SelectedQualifiedProfile
        );
        assert_eq!(report.selected_profile_id.as_deref(), Some("stable"));
        assert!((report.selected_worst_site_primary_metric_utility_lower_bound.unwrap() - 0.68).abs() < 1e-12);
    }

    #[test]
    fn no_exact_context_match_returns_abstention_not_global_fallback() {
        let mut candidate = profile("other-biome", &[0.80, 0.80, 0.80]);
        candidate.context.biome_or_ecology_id = "open_savanna".into();
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert_eq!(report.disposition, RoutingDisposition::AbstainNoQualifiedProfile);
        assert!(report.selected_profile_id.is_none());
        assert!(!report.silent_fallback_performed);
        assert!(report.rejected_profiles[0].reasons.contains(
            &ProfileRejectionReason::LandscapeContextMismatch
        ));
    }

    #[test]
    fn outer_test_profile_cannot_be_used_for_selection() {
        let mut candidate = profile("test-only", &[0.90, 0.90, 0.90]);
        candidate.evaluation_role = EvaluationRole::OuterTest;
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert_eq!(report.disposition, RoutingDisposition::AbstainNoQualifiedProfile);
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::NotSelectorValidationRole));
        assert!(!report.outer_test_data_used_for_selection);
    }

    #[test]
    fn pixel_random_split_is_not_primary_selector_validation() {
        let mut candidate = profile("pixel-random", &[0.90, 0.90, 0.90]);
        candidate.split_protocol = SplitProtocol::PixelRandomInterpolation;
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::SplitNotSiteOrRegionHeldOut));
    }

    #[test]
    fn training_validation_site_overlap_rejects_profile() {
        let mut candidate = profile("leaky", &[0.90, 0.90, 0.90]);
        candidate.training_site_ids.push("validation-1".into());
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::TrainingValidationSiteOverlap));
    }

    #[test]
    fn duplicate_site_results_fail_closed() {
        let mut candidate = profile("duplicate-site", &[0.90, 0.90, 0.90]);
        candidate.validation_site_results[1].validation_site_id =
            candidate.validation_site_results[0].validation_site_id.clone();
        assert!(matches!(
            select_heritage_representation(&request(), &[candidate]),
            Err(RepresentationRoutingError::DuplicateValidationSiteId(_))
        ));
    }

    #[test]
    fn insufficient_independent_sites_reject_profile() {
        let candidate = profile("too-few", &[0.90, 0.90]);
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::InsufficientIndependentValidationSites));
    }

    #[test]
    fn one_site_below_a_quality_floor_rejects_the_whole_profile() {
        let mut candidate = profile("one-bad-site", &[0.90, 0.90, 0.90]);
        candidate.validation_site_results[2].precision_lower_bound = 0.50;
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert_eq!(report.disposition, RoutingDisposition::AbstainNoQualifiedProfile);
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::PrecisionFloorNotMet));
    }

    #[test]
    fn rights_pending_rejects_profile_even_with_high_scores() {
        let mut candidate = profile("restricted", &[0.99, 0.99, 0.99]);
        candidate.rights_status = RightsStatus::Pending;
        candidate.rights_receipt_ref = None;
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert_eq!(report.disposition, RoutingDisposition::AbstainNoQualifiedProfile);
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::RightsNotVerified));
    }

    #[test]
    fn calibration_failure_rejects_profile() {
        let mut candidate = profile("uncalibrated", &[0.90, 0.90, 0.90]);
        candidate.validation_site_results[1].calibration_passed = false;
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::CalibrationFailed));
    }

    #[test]
    fn mixed_metric_definition_cannot_be_ranked() {
        let mut candidate = profile("wrong-metric", &[0.90, 0.90, 0.90]);
        candidate.primary_metric_id = "pixel_f1".into();
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert_eq!(report.disposition, RoutingDisposition::AbstainNoQualifiedProfile);
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::PrimaryMetricMismatch));
    }

    #[test]
    fn different_channel_semantics_cannot_be_ranked() {
        let mut candidate = profile("nir-mask-mismatch", &[0.90, 0.90, 0.90]);
        candidate.channel_semantics_sha256 = "e".repeat(64);
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::ChannelSemanticsMismatch));
    }

    #[test]
    fn quality_floors_are_applied_to_each_validation_site() {
        let mut candidate = profile("false-alarm-site", &[0.90, 0.90, 0.90]);
        candidate.validation_site_results[0].false_candidates_per_km2 = 20.0;
        let report = select_heritage_representation(&request(), &[candidate]).unwrap();
        assert!(report.rejected_profiles[0]
            .reasons
            .contains(&ProfileRejectionReason::FalseCandidateCeilingExceeded));
    }

    #[test]
    fn equal_profiles_use_deterministic_profile_id_tiebreak() {
        let a = profile("profile-b", &[0.80, 0.81, 0.82]);
        let b = profile("profile-a", &[0.80, 0.81, 0.82]);
        let report = select_heritage_representation(&request(), &[a, b]).unwrap();
        assert_eq!(report.selected_profile_id.as_deref(), Some("profile-a"));
    }

    #[test]
    fn malformed_non_finite_site_metric_fails_closed() {
        let mut candidate = profile("nan", &[0.90, 0.90, 0.90]);
        candidate.validation_site_results[0].primary_metric_utility_lower_bound = f64::NAN;
        assert!(matches!(
            select_heritage_representation(&request(), &[candidate]),
            Err(RepresentationRoutingError::InvalidSiteMetric { .. })
        ));
    }

    #[test]
    fn selected_output_never_claims_archaeological_identification() {
        let report =
            select_heritage_representation(&request(), &[profile("valid", &[0.80, 0.82, 0.81])])
                .unwrap();
        assert_eq!(report.disposition, RoutingDisposition::SelectedQualifiedProfile);
        assert!(!report.archaeological_identification_claimed);
        assert!(!report.outer_test_data_used_for_selection);
        assert!(!report.silent_fallback_performed);
    }
}
