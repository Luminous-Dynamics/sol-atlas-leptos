// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Context-qualified archaeological survey observability.
//!
//! This evaluator asks whether a non-detection is informative under a
//! context-matched, independently calibrated survey regime. It never turns a
//! non-detection into proof of absence or an observation into an archaeological
//! identification. v0.2 binds the detection-probability estimand to an explicit
//! conditioning set and computes PoD only over trials satisfying that set.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_OBSERVABILITY_SCHEMA_VERSION: &str = "luminous.heritage-observability.v0.2";
pub const MINIMUM_CALIBRATION_PSUS_FLOOR: usize = 20;
pub const MAX_CALIBRATION_TRIALS: usize = 100_000;
pub const WILSON_95_Z: f64 = 1.959_963_984_540_054;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationModality {
    AirborneLidar,
    SatelliteOptical,
    SatelliteSar,
    UavPhotogrammetry,
    GroundGeophysics,
    FieldWalking,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationSamplingDesign {
    ProbabilitySampledIndependentSites,
    UnsupportedComplexOrUnequalProbability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceEvidenceTier {
    FieldAssessed,
    IndependentExpertReview,
    InstrumentObservation,
    CatalogueOnly,
    ModelOnly,
}

impl ReferenceEvidenceTier {
    fn supports_presence_reference(self) -> bool {
        matches!(
            self,
            Self::FieldAssessed | Self::IndependentExpertReview | Self::InstrumentObservation
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityClass {
    Adequate,
    Partial,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOutcome {
    NotSurveyed,
    Detected,
    NotDetected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityDisposition {
    Detected,
    QualifiedNonDetection,
    IndeterminateNonDetection,
    NotSurveyed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionProbabilityScope {
    /// Probability of detection across the complete survey-unit process.
    /// No upstream conditioning events are removed from the denominator.
    JointPerSurveyUnit,
    /// Probability of sensor detection conditional on target survival, exposure,
    /// observability and actual footprint coverage.
    ConditionalOnTargetSurvivedExposedObservableAndCovered,
    /// Probability of sensor detection conditional on the target being presented
    /// to the sensor, with the upstream stages explicitly evidenced as well.
    ConditionalOnTargetPresentedToSensor,
    /// A non-standard, explicitly declared conditioning set.
    OtherDeclaredConditioningSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionCondition {
    TargetSurvived,
    TargetExposed,
    TargetObservable,
    TargetCoveredByActualFootprint,
    TargetPresentedToSensor,
    TargetObservationDataValid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditioningEvidence {
    pub condition: DetectionCondition,
    pub satisfied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyContext {
    pub feature_class_id: String,
    pub modality: ObservationModality,
    pub sensor_configuration_sha256: String,
    pub processing_build_sha256: String,
    pub environment_stratum_id: String,
    pub season_stratum_id: String,
    pub resolution_class_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationProfile {
    pub profile_id: String,
    pub profile_sha256: String,
    pub context: SurveyContext,
    pub sampling_design: CalibrationSamplingDesign,
    pub detection_probability_scope: DetectionProbabilityScope,
    /// Canonically ordered conditioning events. The declared set defines the
    /// calibration denominator; only trials satisfying every event are eligible.
    pub conditioning_events: Vec<DetectionCondition>,
    /// Preregistered minimum eligible independent positive-site PSUs. Values below 20
    /// are rejected as a policy floor for this v0.2 evaluator.
    pub minimum_independent_psus: usize,
    /// Required lower endpoint of the two-sided 95% Wilson interval.
    pub minimum_pod_lower_bound: f64,
    /// Minimum fraction of the target survey unit that was actually searched.
    pub minimum_coverage_fraction: f64,
    /// Minimum fraction of the target survey unit with valid usable observations.
    pub minimum_valid_data_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationTrial {
    pub psu_id: String,
    pub site_id: String,
    pub profile_sha256: String,
    pub context: SurveyContext,
    /// Exactly one value per declared profile conditioning event, in canonical order.
    pub conditioning_evidence: Vec<ConditioningEvidence>,
    /// True when this independent, known-positive PSU's target feature was detected.
    pub detected: bool,
    pub reference_evidence_tier: ReferenceEvidenceTier,
    pub reference_evidence_ref: String,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyObservation {
    pub observation_id: Option<String>,
    pub outcome: ObservationOutcome,
    pub context: SurveyContext,
    pub calibration_profile_sha256: String,
    pub coverage_fraction: f64,
    pub valid_data_fraction: f64,
    pub visibility: VisibilityClass,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeritageObservabilityInput {
    pub schema_version: String,
    pub candidate_id: String,
    pub calibration_profile: CalibrationProfile,
    pub calibration_trials: Vec<CalibrationTrial>,
    pub observation: SurveyObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WilsonInterval {
    pub confidence_level: f64,
    pub z_value: f64,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonDetectionReason {
    CalibrationSampleBelowMinimum,
    DetectionProbabilityLowerBoundBelowThreshold,
    CalibrationProfileDigestMismatch,
    FeatureClassMismatch,
    ModalityMismatch,
    SensorConfigurationMismatch,
    ProcessingBuildMismatch,
    EnvironmentStratumMismatch,
    SeasonStratumMismatch,
    ResolutionClassMismatch,
    CoverageBelowThreshold,
    ValidDataBelowThreshold,
    VisibilityNotAdequate,
    RightsNotVerified,
    RightsReceiptMissing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeritageObservabilityReport {
    pub schema_version: String,
    pub candidate_id: String,
    pub profile_id: String,
    pub profile_sha256: String,
    pub calibration_context: SurveyContext,
    pub observed_context: SurveyContext,
    pub observation_profile_sha256: String,
    pub observation_id: Option<String>,
    pub observation_outcome: ObservationOutcome,
    pub disposition: ObservabilityDisposition,
    pub detection_probability_scope: DetectionProbabilityScope,
    pub conditioning_events: Vec<DetectionCondition>,
    pub calibration_total_independent_psu_count: usize,
    pub calibration_eligible_independent_psu_count: usize,
    pub calibration_excluded_conditioning_psu_count: usize,
    pub calibration_detected_count: usize,
    pub empirical_probability_of_detection: f64,
    pub probability_of_detection_wilson_95: WilsonInterval,
    pub minimum_independent_psus: usize,
    pub minimum_pod_lower_bound: f64,
    pub observed_coverage_fraction: f64,
    pub minimum_coverage_fraction: f64,
    pub observed_valid_data_fraction: f64,
    pub minimum_valid_data_fraction: f64,
    pub visibility: VisibilityClass,
    pub context_match: bool,
    pub rights_status: RightsStatus,
    pub reasons: Vec<NonDetectionReason>,
    /// Structural evaluation never declares an archaeological absence.
    pub archaeological_absence_claimed: bool,
    /// A detection observation is not automatically an archaeological identification.
    pub archaeological_identification_claimed: bool,
    /// v0.1 does not multiply repeated observations or assume conditional independence.
    pub combined_multi_observation_probability_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageObservabilityError {
    WrongSchemaVersion,
    EmptyCandidateId,
    EmptyProfileId,
    InvalidProfileDigest,
    EmptyFeatureClassId,
    InvalidSensorConfigurationDigest,
    InvalidProcessingBuildDigest,
    EmptyEnvironmentStratumId,
    EmptySeasonStratumId,
    EmptyResolutionClassId,
    UnsupportedSamplingDesign,
    InvalidConditioningEventSet,
    DuplicateConditioningEvent,
    TrialConditioningEvidenceMismatch { psu_id: String },
    DuplicateTrialConditioningEvidence { psu_id: String, condition: DetectionCondition },
    ConditioningEvidenceContradictsDetection { psu_id: String },
    NoEligibleCalibrationTrials,
    MinimumCalibrationPsuCountBelowFloor,
    InvalidMinimumPodLowerBound,
    InvalidMinimumCoverageFraction,
    InvalidMinimumValidDataFraction,
    NoCalibrationTrials,
    TooManyCalibrationTrials,
    EmptyCalibrationPsuId,
    EmptyCalibrationSiteId,
    DuplicateCalibrationPsuId(String),
    DuplicateCalibrationSiteId(String),
    CalibrationProfileDigestMismatch { psu_id: String },
    CalibrationContextMismatch { psu_id: String },
    CalibrationReferenceEvidenceInsufficient { psu_id: String },
    CalibrationRightsNotVerified { psu_id: String },
    CalibrationReferenceEvidenceMissing { psu_id: String },
    CalibrationRightsReceiptMissing { psu_id: String },
    InvalidObservationId,
    ObservationIdRequired,
    ObservationIdForbiddenWhenNotSurveyed,
    InvalidObservationProfileDigest,
    InvalidCoverageFraction,
    InvalidValidDataFraction,
    InvalidWilsonInterval,
}

impl fmt::Display for HeritageObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage observability error: {self:?}")
    }
}

impl std::error::Error for HeritageObservabilityError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_fraction(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn validate_conditioning_event_set(
    profile: &CalibrationProfile,
) -> Result<(), HeritageObservabilityError> {
    let mut unique = BTreeSet::new();
    for condition in &profile.conditioning_events {
        if !unique.insert(*condition) {
            return Err(HeritageObservabilityError::DuplicateConditioningEvent);
        }
    }
    if profile
        .conditioning_events
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err(HeritageObservabilityError::InvalidConditioningEventSet);
    }

    let expected: BTreeSet<DetectionCondition> = match profile.detection_probability_scope {
        DetectionProbabilityScope::JointPerSurveyUnit => BTreeSet::new(),
        DetectionProbabilityScope::ConditionalOnTargetSurvivedExposedObservableAndCovered => [
            DetectionCondition::TargetSurvived,
            DetectionCondition::TargetExposed,
            DetectionCondition::TargetObservable,
            DetectionCondition::TargetCoveredByActualFootprint,
        ]
        .into_iter()
        .collect(),
        DetectionProbabilityScope::ConditionalOnTargetPresentedToSensor => [
            DetectionCondition::TargetSurvived,
            DetectionCondition::TargetExposed,
            DetectionCondition::TargetObservable,
            DetectionCondition::TargetCoveredByActualFootprint,
            DetectionCondition::TargetPresentedToSensor,
        ]
        .into_iter()
        .collect(),
        DetectionProbabilityScope::OtherDeclaredConditioningSet => {
            if profile.conditioning_events.is_empty() {
                return Err(HeritageObservabilityError::InvalidConditioningEventSet);
            }
            unique.clone()
        }
    };
    if unique != expected {
        return Err(HeritageObservabilityError::InvalidConditioningEventSet);
    }
    Ok(())
}

fn trial_is_conditioning_eligible(trial: &CalibrationTrial) -> bool {
    trial.conditioning_evidence.iter().all(|evidence| evidence.satisfied)
}

fn validate_profile(profile: &CalibrationProfile) -> Result<(), HeritageObservabilityError> {
    if !non_empty(&profile.profile_id) {
        return Err(HeritageObservabilityError::EmptyProfileId);
    }
    if !valid_sha256(&profile.profile_sha256) {
        return Err(HeritageObservabilityError::InvalidProfileDigest);
    }
    if !non_empty(&profile.context.feature_class_id) {
        return Err(HeritageObservabilityError::EmptyFeatureClassId);
    }
    if !valid_sha256(&profile.context.sensor_configuration_sha256) {
        return Err(HeritageObservabilityError::InvalidSensorConfigurationDigest);
    }
    if !valid_sha256(&profile.context.processing_build_sha256) {
        return Err(HeritageObservabilityError::InvalidProcessingBuildDigest);
    }
    if !non_empty(&profile.context.environment_stratum_id) {
        return Err(HeritageObservabilityError::EmptyEnvironmentStratumId);
    }
    if !non_empty(&profile.context.season_stratum_id) {
        return Err(HeritageObservabilityError::EmptySeasonStratumId);
    }
    if !non_empty(&profile.context.resolution_class_id) {
        return Err(HeritageObservabilityError::EmptyResolutionClassId);
    }
    if profile.sampling_design != CalibrationSamplingDesign::ProbabilitySampledIndependentSites {
        return Err(HeritageObservabilityError::UnsupportedSamplingDesign);
    }
    validate_conditioning_event_set(profile)?;
    if profile.minimum_independent_psus < MINIMUM_CALIBRATION_PSUS_FLOOR {
        return Err(HeritageObservabilityError::MinimumCalibrationPsuCountBelowFloor);
    }
    if !profile.minimum_pod_lower_bound.is_finite()
        || !(0.0..=1.0).contains(&profile.minimum_pod_lower_bound)
        || profile.minimum_pod_lower_bound == 0.0
    {
        return Err(HeritageObservabilityError::InvalidMinimumPodLowerBound);
    }
    if !valid_fraction(profile.minimum_coverage_fraction)
        || profile.minimum_coverage_fraction == 0.0
    {
        return Err(HeritageObservabilityError::InvalidMinimumCoverageFraction);
    }
    if !valid_fraction(profile.minimum_valid_data_fraction)
        || profile.minimum_valid_data_fraction == 0.0
    {
        return Err(HeritageObservabilityError::InvalidMinimumValidDataFraction);
    }
    Ok(())
}

fn validate_trials(
    profile: &CalibrationProfile,
    trials: &[CalibrationTrial],
) -> Result<(), HeritageObservabilityError> {
    if trials.is_empty() {
        return Err(HeritageObservabilityError::NoCalibrationTrials);
    }
    if trials.len() > MAX_CALIBRATION_TRIALS {
        return Err(HeritageObservabilityError::TooManyCalibrationTrials);
    }

    let mut psu_ids = BTreeSet::new();
    let mut site_ids = BTreeSet::new();
    for trial in trials {
        if !non_empty(&trial.psu_id) {
            return Err(HeritageObservabilityError::EmptyCalibrationPsuId);
        }
        if !non_empty(&trial.site_id) {
            return Err(HeritageObservabilityError::EmptyCalibrationSiteId);
        }
        if !psu_ids.insert(trial.psu_id.clone()) {
            return Err(HeritageObservabilityError::DuplicateCalibrationPsuId(
                trial.psu_id.clone(),
            ));
        }
        if !site_ids.insert(trial.site_id.clone()) {
            return Err(HeritageObservabilityError::DuplicateCalibrationSiteId(
                trial.site_id.clone(),
            ));
        }
        if trial.profile_sha256 != profile.profile_sha256 {
            return Err(
                HeritageObservabilityError::CalibrationProfileDigestMismatch {
                    psu_id: trial.psu_id.clone(),
                },
            );
        }
        if trial.context != profile.context {
            return Err(HeritageObservabilityError::CalibrationContextMismatch {
                psu_id: trial.psu_id.clone(),
            });
        }
        let mut evidence_conditions = BTreeSet::new();
        for evidence in &trial.conditioning_evidence {
            if !evidence_conditions.insert(evidence.condition) {
                return Err(
                    HeritageObservabilityError::DuplicateTrialConditioningEvidence {
                        psu_id: trial.psu_id.clone(),
                        condition: evidence.condition,
                    },
                );
            }
        }
        let declared_conditions: Vec<DetectionCondition> = trial
            .conditioning_evidence
            .iter()
            .map(|evidence| evidence.condition)
            .collect();
        if declared_conditions != profile.conditioning_events {
            return Err(HeritageObservabilityError::TrialConditioningEvidenceMismatch {
                psu_id: trial.psu_id.clone(),
            });
        }
        if trial.detected && !trial_is_conditioning_eligible(trial) {
            return Err(
                HeritageObservabilityError::ConditioningEvidenceContradictsDetection {
                    psu_id: trial.psu_id.clone(),
                },
            );
        }
        if !trial.reference_evidence_tier.supports_presence_reference() {
            return Err(
                HeritageObservabilityError::CalibrationReferenceEvidenceInsufficient {
                    psu_id: trial.psu_id.clone(),
                },
            );
        }
        if !non_empty(&trial.reference_evidence_ref) {
            return Err(
                HeritageObservabilityError::CalibrationReferenceEvidenceMissing {
                    psu_id: trial.psu_id.clone(),
                },
            );
        }
        if trial.rights_status != RightsStatus::Verified {
            return Err(HeritageObservabilityError::CalibrationRightsNotVerified {
                psu_id: trial.psu_id.clone(),
            });
        }
        if !non_empty(&trial.rights_receipt_ref) {
            return Err(
                HeritageObservabilityError::CalibrationRightsReceiptMissing {
                    psu_id: trial.psu_id.clone(),
                },
            );
        }
    }
    Ok(())
}

fn wilson_interval(
    successes: usize,
    trials: usize,
) -> Result<WilsonInterval, HeritageObservabilityError> {
    if trials == 0 || successes > trials {
        return Err(HeritageObservabilityError::InvalidWilsonInterval);
    }

    let n = trials as f64;
    let p = successes as f64 / n;
    let z = WILSON_95_Z;
    let z2 = z * z;
    let denominator = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator;
    let half_width = z * ((p * (1.0 - p) / n) + (z2 / (4.0 * n * n))).sqrt() / denominator;
    let lower = (center - half_width).max(0.0);
    let upper = (center + half_width).min(1.0);

    if !lower.is_finite() || !upper.is_finite() || lower < 0.0 || upper > 1.0 || lower > upper {
        return Err(HeritageObservabilityError::InvalidWilsonInterval);
    }

    Ok(WilsonInterval {
        confidence_level: 0.95,
        z_value: z,
        lower,
        upper,
    })
}

fn context_mismatch_reasons(
    expected: &SurveyContext,
    observed: &SurveyContext,
) -> Vec<NonDetectionReason> {
    let mut reasons = Vec::new();
    if expected.feature_class_id != observed.feature_class_id {
        reasons.push(NonDetectionReason::FeatureClassMismatch);
    }
    if expected.modality != observed.modality {
        reasons.push(NonDetectionReason::ModalityMismatch);
    }
    if expected.sensor_configuration_sha256 != observed.sensor_configuration_sha256 {
        reasons.push(NonDetectionReason::SensorConfigurationMismatch);
    }
    if expected.processing_build_sha256 != observed.processing_build_sha256 {
        reasons.push(NonDetectionReason::ProcessingBuildMismatch);
    }
    if expected.environment_stratum_id != observed.environment_stratum_id {
        reasons.push(NonDetectionReason::EnvironmentStratumMismatch);
    }
    if expected.season_stratum_id != observed.season_stratum_id {
        reasons.push(NonDetectionReason::SeasonStratumMismatch);
    }
    if expected.resolution_class_id != observed.resolution_class_id {
        reasons.push(NonDetectionReason::ResolutionClassMismatch);
    }
    reasons
}

fn non_detection_reasons(
    input: &HeritageObservabilityInput,
    interval: WilsonInterval,
) -> Vec<NonDetectionReason> {
    let profile = &input.calibration_profile;
    let observation = &input.observation;
    let mut reasons = context_mismatch_reasons(&profile.context, &observation.context);

    if observation.calibration_profile_sha256 != profile.profile_sha256 {
        reasons.push(NonDetectionReason::CalibrationProfileDigestMismatch);
    }
    let eligible_count = input.calibration_trials.iter().filter(|trial| trial_is_conditioning_eligible(trial)).count();
    if eligible_count < profile.minimum_independent_psus {
        reasons.push(NonDetectionReason::CalibrationSampleBelowMinimum);
    }
    if interval.lower < profile.minimum_pod_lower_bound {
        reasons.push(NonDetectionReason::DetectionProbabilityLowerBoundBelowThreshold);
    }
    if observation.coverage_fraction < profile.minimum_coverage_fraction {
        reasons.push(NonDetectionReason::CoverageBelowThreshold);
    }
    if observation.valid_data_fraction < profile.minimum_valid_data_fraction {
        reasons.push(NonDetectionReason::ValidDataBelowThreshold);
    }
    if observation.visibility != VisibilityClass::Adequate {
        reasons.push(NonDetectionReason::VisibilityNotAdequate);
    }
    if observation.rights_status != RightsStatus::Verified {
        reasons.push(NonDetectionReason::RightsNotVerified);
    }
    if observation
        .rights_receipt_ref
        .as_deref()
        .is_none_or(|value| !non_empty(value))
    {
        reasons.push(NonDetectionReason::RightsReceiptMissing);
    }
    reasons
}

/// Evaluate whether the supplied observation supports an informative
/// non-detection under an exact, context-matched calibration profile.
///
/// This function performs structural and numerical checks only. It does not
/// authenticate evidence, rights receipts, the calibration sampling frame,
/// sensor configuration, or the source of a profile digest.
pub fn evaluate_heritage_observability(
    input: &HeritageObservabilityInput,
) -> Result<HeritageObservabilityReport, HeritageObservabilityError> {
    if input.schema_version != HERITAGE_OBSERVABILITY_SCHEMA_VERSION {
        return Err(HeritageObservabilityError::WrongSchemaVersion);
    }
    if !non_empty(&input.candidate_id) {
        return Err(HeritageObservabilityError::EmptyCandidateId);
    }

    validate_profile(&input.calibration_profile)?;
    validate_trials(&input.calibration_profile, &input.calibration_trials)?;
    let eligible_trials: Vec<&CalibrationTrial> = input
        .calibration_trials
        .iter()
        .filter(|trial| trial_is_conditioning_eligible(trial))
        .collect();
    if eligible_trials.is_empty() {
        return Err(HeritageObservabilityError::NoEligibleCalibrationTrials);
    }

    let observation = &input.observation;
    match observation.outcome {
        ObservationOutcome::NotSurveyed if observation.observation_id.is_some() => {
            return Err(HeritageObservabilityError::ObservationIdForbiddenWhenNotSurveyed);
        }
        ObservationOutcome::NotSurveyed => {}
        _ => {
            let Some(observation_id) = observation.observation_id.as_deref() else {
                return Err(HeritageObservabilityError::ObservationIdRequired);
            };
            if !non_empty(observation_id) {
                return Err(HeritageObservabilityError::InvalidObservationId);
            }
        }
    }

    if !valid_sha256(&observation.calibration_profile_sha256) {
        return Err(HeritageObservabilityError::InvalidObservationProfileDigest);
    }
    if !valid_fraction(observation.coverage_fraction) {
        return Err(HeritageObservabilityError::InvalidCoverageFraction);
    }
    if !valid_fraction(observation.valid_data_fraction) {
        return Err(HeritageObservabilityError::InvalidValidDataFraction);
    }

    let detected_count = eligible_trials.iter().filter(|trial| trial.detected).count();
    let total_trial_count = input.calibration_trials.len();
    let eligible_trial_count = eligible_trials.len();
    let excluded_conditioning_count = total_trial_count - eligible_trial_count;
    let empirical_pod = detected_count as f64 / eligible_trial_count as f64;
    let interval = wilson_interval(detected_count, eligible_trial_count)?;
    let reasons = if observation.outcome == ObservationOutcome::NotDetected {
        non_detection_reasons(input, interval)
    } else {
        Vec::new()
    };
    let context_match =
        context_mismatch_reasons(&input.calibration_profile.context, &observation.context)
            .is_empty()
            && observation.calibration_profile_sha256 == input.calibration_profile.profile_sha256;

    let disposition = match observation.outcome {
        ObservationOutcome::NotSurveyed => ObservabilityDisposition::NotSurveyed,
        ObservationOutcome::Detected => ObservabilityDisposition::Detected,
        ObservationOutcome::NotDetected if reasons.is_empty() => {
            ObservabilityDisposition::QualifiedNonDetection
        }
        ObservationOutcome::NotDetected => ObservabilityDisposition::IndeterminateNonDetection,
    };

    Ok(HeritageObservabilityReport {
        schema_version: HERITAGE_OBSERVABILITY_SCHEMA_VERSION.to_owned(),
        candidate_id: input.candidate_id.clone(),
        profile_id: input.calibration_profile.profile_id.clone(),
        profile_sha256: input.calibration_profile.profile_sha256.clone(),
        calibration_context: input.calibration_profile.context.clone(),
        observed_context: observation.context.clone(),
        observation_profile_sha256: observation.calibration_profile_sha256.clone(),
        observation_id: observation.observation_id.clone(),
        observation_outcome: observation.outcome,
        disposition,
        detection_probability_scope: input.calibration_profile.detection_probability_scope,
        conditioning_events: input.calibration_profile.conditioning_events.clone(),
        calibration_total_independent_psu_count: total_trial_count,
        calibration_eligible_independent_psu_count: eligible_trial_count,
        calibration_excluded_conditioning_psu_count: excluded_conditioning_count,
        calibration_detected_count: detected_count,
        empirical_probability_of_detection: empirical_pod,
        probability_of_detection_wilson_95: interval,
        minimum_independent_psus: input.calibration_profile.minimum_independent_psus,
        minimum_pod_lower_bound: input.calibration_profile.minimum_pod_lower_bound,
        observed_coverage_fraction: observation.coverage_fraction,
        minimum_coverage_fraction: input.calibration_profile.minimum_coverage_fraction,
        observed_valid_data_fraction: observation.valid_data_fraction,
        minimum_valid_data_fraction: input.calibration_profile.minimum_valid_data_fraction,
        visibility: observation.visibility,
        context_match,
        rights_status: observation.rights_status,
        reasons,
        archaeological_absence_claimed: false,
        archaeological_identification_claimed: false,
        combined_multi_observation_probability_reported: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> SurveyContext {
        SurveyContext {
            feature_class_id: "stone_enclosure".into(),
            modality: ObservationModality::AirborneLidar,
            sensor_configuration_sha256: "a".repeat(64),
            processing_build_sha256: "b".repeat(64),
            environment_stratum_id: "open_grassland_low_relief".into(),
            season_stratum_id: "leaf_off_dry_season".into(),
            resolution_class_id: "dem_0p5m".into(),
        }
    }

    fn profile() -> CalibrationProfile {
        CalibrationProfile {
            profile_id: "lidar-stone-enclosure-grassland-v1".into(),
            profile_sha256: "c".repeat(64),
            context: context(),
            sampling_design: CalibrationSamplingDesign::ProbabilitySampledIndependentSites,
            detection_probability_scope: DetectionProbabilityScope::JointPerSurveyUnit,
            conditioning_events: Vec::new(),
            minimum_independent_psus: 20,
            minimum_pod_lower_bound: 0.80,
            minimum_coverage_fraction: 0.95,
            minimum_valid_data_fraction: 0.95,
        }
    }

    fn trials(detections: usize) -> Vec<CalibrationTrial> {
        (0..20)
            .map(|index| CalibrationTrial {
                psu_id: format!("psu-{index}"),
                site_id: format!("site-{index}"),
                profile_sha256: "c".repeat(64),
                context: context(),
                conditioning_evidence: Vec::new(),
                detected: index < detections,
                reference_evidence_tier: ReferenceEvidenceTier::FieldAssessed,
                reference_evidence_ref: format!("urn:synthetic:field-reference:{index}"),
                rights_status: RightsStatus::Verified,
                rights_receipt_ref: format!("urn:synthetic:rights:{index}"),
            })
            .collect()
    }

    fn input(outcome: ObservationOutcome) -> HeritageObservabilityInput {
        HeritageObservabilityInput {
            schema_version: HERITAGE_OBSERVABILITY_SCHEMA_VERSION.into(),
            candidate_id: "candidate-001".into(),
            calibration_profile: profile(),
            calibration_trials: trials(20),
            observation: SurveyObservation {
                observation_id: (outcome != ObservationOutcome::NotSurveyed)
                    .then(|| "observation-001".into()),
                outcome,
                context: context(),
                calibration_profile_sha256: "c".repeat(64),
                coverage_fraction: 0.98,
                valid_data_fraction: 0.99,
                visibility: VisibilityClass::Adequate,
                rights_status: RightsStatus::Verified,
                rights_receipt_ref: Some("urn:synthetic:target-rights".into()),
            },
        }
    }

    #[test]
    fn well_calibrated_context_matched_non_detection_is_qualified_but_not_absence() {
        let report =
            evaluate_heritage_observability(&input(ObservationOutcome::NotDetected)).unwrap();
        assert_eq!(
            report.disposition,
            ObservabilityDisposition::QualifiedNonDetection
        );
        assert!((report.empirical_probability_of_detection - 1.0).abs() < 1e-12);
        assert!(report.probability_of_detection_wilson_95.lower > 0.80);
        assert!(!report.archaeological_absence_claimed);
        assert!(!report.archaeological_identification_claimed);
        assert!(!report.combined_multi_observation_probability_reported);
        assert!(report.reasons.is_empty());
        assert_eq!(report.calibration_context, context());
        assert_eq!(report.observed_context, context());
        assert_eq!(report.observation_profile_sha256, "c".repeat(64));
    }

    #[test]
    fn weak_detection_rate_cannot_qualify_a_non_detection() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials = trials(10);
        let report = evaluate_heritage_observability(&value).unwrap();
        assert_eq!(
            report.disposition,
            ObservabilityDisposition::IndeterminateNonDetection
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::DetectionProbabilityLowerBoundBelowThreshold)
        );
    }

    #[test]
    fn insufficient_independent_psus_withholds_non_detection() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials = trials(20);
        value.calibration_trials.truncate(19);
        let report = evaluate_heritage_observability(&value).unwrap();
        assert_eq!(
            report.disposition,
            ObservabilityDisposition::IndeterminateNonDetection
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::CalibrationSampleBelowMinimum)
        );
    }

    #[test]
    fn duplicate_psu_or_site_does_not_inflate_calibration_sample_size() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials[1].psu_id = value.calibration_trials[0].psu_id.clone();
        assert!(matches!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::DuplicateCalibrationPsuId(_))
        ));

        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials[1].site_id = value.calibration_trials[0].site_id.clone();
        assert!(matches!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::DuplicateCalibrationSiteId(_))
        ));
    }

    #[test]
    fn feature_sensor_configuration_processing_and_stratum_shifts_are_rejected() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.context.modality = ObservationModality::SatelliteOptical;
        value.observation.context.environment_stratum_id = "dense_forest".into();
        value.observation.context.sensor_configuration_sha256 = "d".repeat(64);
        let report = evaluate_heritage_observability(&value).unwrap();
        assert_eq!(
            report.disposition,
            ObservabilityDisposition::IndeterminateNonDetection
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::ModalityMismatch)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::EnvironmentStratumMismatch)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::SensorConfigurationMismatch)
        );
        assert!(!report.context_match);
    }

    #[test]
    fn processing_season_feature_and_resolution_shifts_are_rejected() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.context.feature_class_id = "cropmark".into();
        value.observation.context.processing_build_sha256 = "e".repeat(64);
        value.observation.context.season_stratum_id = "leaf_on_wet_season".into();
        value.observation.context.resolution_class_id = "sentinel2_10m".into();
        let report = evaluate_heritage_observability(&value).unwrap();
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::FeatureClassMismatch)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::ProcessingBuildMismatch)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::SeasonStratumMismatch)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::ResolutionClassMismatch)
        );
    }

    #[test]
    fn partial_coverage_invalid_data_and_blocked_visibility_are_indeterminate() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.coverage_fraction = 0.70;
        value.observation.valid_data_fraction = 0.80;
        value.observation.visibility = VisibilityClass::Blocked;
        let report = evaluate_heritage_observability(&value).unwrap();
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::CoverageBelowThreshold)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::ValidDataBelowThreshold)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::VisibilityNotAdequate)
        );
        assert_eq!(
            report.disposition,
            ObservabilityDisposition::IndeterminateNonDetection
        );
    }

    #[test]
    fn unverified_rights_and_missing_receipt_withhold_qualification() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.rights_status = RightsStatus::Pending;
        value.observation.rights_receipt_ref = None;
        let report = evaluate_heritage_observability(&value).unwrap();
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::RightsNotVerified)
        );
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::RightsReceiptMissing)
        );
    }

    #[test]
    fn catalogue_only_or_model_only_reference_cannot_calibrate_presence() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials[0].reference_evidence_tier = ReferenceEvidenceTier::ModelOnly;
        assert!(matches!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::CalibrationReferenceEvidenceInsufficient { .. })
        ));
    }

    #[test]
    fn calibration_trial_context_mismatch_is_rejected_even_when_digest_matches() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials[0].context.environment_stratum_id = "dense_forest".into();
        assert!(matches!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::CalibrationContextMismatch { .. })
        ));
    }

    #[test]
    fn unverified_calibration_rights_fail_closed() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_trials[0].rights_status = RightsStatus::Pending;
        assert!(matches!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::CalibrationRightsNotVerified { .. })
        ));
    }

    #[test]
    fn profile_digest_mismatch_in_target_observation_is_not_silently_accepted() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.calibration_profile_sha256 = "d".repeat(64);
        let report = evaluate_heritage_observability(&value).unwrap();
        assert!(
            report
                .reasons
                .contains(&NonDetectionReason::CalibrationProfileDigestMismatch)
        );
        assert!(!report.context_match);
        assert_eq!(
            report.disposition,
            ObservabilityDisposition::IndeterminateNonDetection
        );
    }

    #[test]
    fn detected_observation_is_not_promoted_to_archaeological_identification() {
        let report = evaluate_heritage_observability(&input(ObservationOutcome::Detected)).unwrap();
        assert_eq!(report.disposition, ObservabilityDisposition::Detected);
        assert!(!report.archaeological_identification_claimed);
        assert!(!report.archaeological_absence_claimed);
    }

    #[test]
    fn not_surveyed_is_not_a_non_detection() {
        let report =
            evaluate_heritage_observability(&input(ObservationOutcome::NotSurveyed)).unwrap();
        assert_eq!(report.disposition, ObservabilityDisposition::NotSurveyed);
        assert!(report.reasons.is_empty());
    }

    #[test]
    fn unsupported_complex_sampling_design_is_withheld() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.calibration_profile.sampling_design =
            CalibrationSamplingDesign::UnsupportedComplexOrUnequalProbability;
        assert_eq!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::UnsupportedSamplingDesign)
        );
    }

    #[test]
    fn wilson_interval_has_correct_known_answer_for_twenty_of_twenty() {
        let interval = wilson_interval(20, 20).unwrap();
        assert!((interval.lower - 0.838_874_841_947_311_1).abs() < 1e-12);
        assert_eq!(interval.upper, 1.0);
        assert_eq!(interval.confidence_level, 0.95);
    }

    #[test]
    fn malformed_coverage_and_non_finite_values_fail_closed() {
        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.coverage_fraction = f64::NAN;
        assert_eq!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::InvalidCoverageFraction)
        );

        let mut value = input(ObservationOutcome::NotDetected);
        value.observation.valid_data_fraction = f64::INFINITY;
        assert_eq!(
            evaluate_heritage_observability(&value),
            Err(HeritageObservabilityError::InvalidValidDataFraction)
        );
    }
}
