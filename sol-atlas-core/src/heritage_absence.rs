// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Survival-, exposure-, observability-, coverage- and detection-aware
//! Bayesian interpretation of a single archaeological non-detection.
//!
//! Every probability is a calibrated conditional band. Their product is a
//! chain rule over explicitly ordered stages, not a claim that the latent
//! events are mutually independent. The interval posterior is a conservative
//! rectangular-bound calculation, not a credible interval.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const HERITAGE_ABSENCE_SCHEMA_VERSION: &str = "luminous.heritage-absence.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyObservationOutcome {
    Detected,
    SearchedNoDetection,
    NotSearched,
    NotObservable,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoUpdateReason {
    Detected,
    NotSearched,
    NotObservable,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyObservation {
    pub observation_id: String,
    pub survey_unit_id: String,
    pub target_class_id: String,
    pub context_id: String,
    pub method_id: String,
    pub effort_profile_sha256: String,
    pub outcome: SurveyObservationOutcome,
    pub coverage_receipt_ref: Option<String>,
    pub coverage_receipt_sha256: Option<String>,
    pub effort_receipt_ref: Option<String>,
    pub effort_receipt_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibratedProbabilityBand {
    pub lower: f64,
    pub upper: f64,
    pub context_id: String,
    pub target_class_id: String,
    /// Method identity is required for sensor/search-conditioned stages.
    pub method_id: Option<String>,
    /// Required for stages conditioned on the actual survey effort profile.
    pub effort_profile_sha256: Option<String>,
    pub calibration_profile_sha256: String,
    pub calibration_data_sha256: String,
    pub validation_split_sha256: String,
    pub conditional_semantics_sha256: String,
    pub receipt_ref: String,
    pub receipt_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonDetectionModel {
    pub model_id: String,
    pub context_id: String,
    pub target_class_id: String,
    pub method_id: String,
    pub effort_profile_sha256: String,
    /// P(target originally present before the current survey event).
    pub prior_original_presence: CalibratedProbabilityBand,
    /// P(evidence survives to present | it was originally present).
    pub survival: CalibratedProbabilityBand,
    /// P(evidence is exposed/accessibly preserved | it survived).
    pub exposure_given_survival: CalibratedProbabilityBand,
    /// P(target is observable under this context/method | survived and exposed).
    pub observability_given_survival_and_exposure: CalibratedProbabilityBand,
    /// P(target is inside the actually searched/acquired footprint | observable).
    pub footprint_coverage_given_observability: CalibratedProbabilityBand,
    /// P(detected | survived, exposed, observable and covered).
    pub conditional_detection_given_covered: CalibratedProbabilityBand,
    /// P(false positive | target was originally absent), for this method/effort.
    pub false_positive_given_absence: CalibratedProbabilityBand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbabilityInterval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonDetectionPosteriorReport {
    pub observation_id: String,
    pub model_id: String,
    pub target_class_id: String,
    pub context_id: String,
    pub method_id: String,
    pub prior_original_presence: ProbabilityInterval,
    pub survival: ProbabilityInterval,
    pub exposure_given_survival: ProbabilityInterval,
    pub observability_given_survival_and_exposure: ProbabilityInterval,
    pub footprint_coverage_given_observability: ProbabilityInterval,
    pub conditional_detection_given_covered: ProbabilityInterval,
    pub false_positive_given_absence: ProbabilityInterval,
    /// Chain-rule product across the stated conditional stages.
    pub probability_detect_if_originally_present: ProbabilityInterval,
    pub probability_no_detection_if_originally_present: ProbabilityInterval,
    pub probability_no_detection_if_originally_absent: ProbabilityInterval,
    pub posterior_original_presence_given_no_detection: ProbabilityInterval,
    pub coverage_receipt_sha256: String,
    pub effort_receipt_sha256: String,
    pub calibration_profile_digests: Vec<String>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "assessment", content = "record", rename_all = "snake_case")]
pub enum AbsenceAssessment {
    PosteriorUpdate(NonDetectionPosteriorReport),
    NoUpdate {
        observation_id: String,
        reason: NoUpdateReason,
        claim_ceiling: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageAbsenceError {
    EmptyIdentity(String),
    InvalidDigest(String),
    MissingModelForNegativeObservation,
    ModelObservationContextMismatch,
    MissingCoverageReceipt,
    MissingEffortReceipt,
    InvalidProbabilityBand(String),
    InvalidProbabilityBandContext(String),
    InvalidMethodBinding(String),
    InvalidEffortBinding(String),
    InvalidCoverageReceiptBinding,
    UndefinedBayesianUpdate,
    SerializationFailure,
}

impl fmt::Display for HeritageAbsenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HeritageAbsenceError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_probability_band(
    name: &str,
    band: &CalibratedProbabilityBand,
    observation: &SurveyObservation,
    method_required: bool,
    effort_required: bool,
) -> Result<(), HeritageAbsenceError> {
    if !band.lower.is_finite()
        || !band.upper.is_finite()
        || band.lower < 0.0
        || band.upper > 1.0
        || band.lower > band.upper
        || !valid_sha256(&band.calibration_profile_sha256)
        || !valid_sha256(&band.calibration_data_sha256)
        || !valid_sha256(&band.validation_split_sha256)
        || !valid_sha256(&band.conditional_semantics_sha256)
        || !non_empty(&band.receipt_ref)
        || !valid_sha256(&band.receipt_sha256)
    {
        return Err(HeritageAbsenceError::InvalidProbabilityBand(
            name.to_owned(),
        ));
    }
    if band.context_id != observation.context_id
        || band.target_class_id != observation.target_class_id
    {
        return Err(HeritageAbsenceError::InvalidProbabilityBandContext(
            name.to_owned(),
        ));
    }
    if method_required {
        if band.method_id.as_deref() != Some(observation.method_id.as_str()) {
            return Err(HeritageAbsenceError::InvalidMethodBinding(
                name.to_owned(),
            ));
        }
    } else if band.method_id.is_some() {
        return Err(HeritageAbsenceError::InvalidMethodBinding(
            name.to_owned(),
        ));
    }
    if effort_required {
        if band.effort_profile_sha256.as_deref()
            != Some(observation.effort_profile_sha256.as_str())
        {
            return Err(HeritageAbsenceError::InvalidEffortBinding(
                name.to_owned(),
            ));
        }
    } else if band.effort_profile_sha256.is_some() {
        return Err(HeritageAbsenceError::InvalidEffortBinding(
            name.to_owned(),
        ));
    }
    Ok(())
}

fn validate_non_detection_model(
    observation: &SurveyObservation,
    model: &NonDetectionModel,
) -> Result<(), HeritageAbsenceError> {
    if !non_empty(&model.model_id)
        || model.context_id != observation.context_id
        || model.target_class_id != observation.target_class_id
        || model.method_id != observation.method_id
        || model.effort_profile_sha256 != observation.effort_profile_sha256
    {
        return Err(HeritageAbsenceError::ModelObservationContextMismatch);
    }

    let bands = [
        ("prior_original_presence", &model.prior_original_presence, false, false),
        ("survival", &model.survival, false, false),
        ("exposure_given_survival", &model.exposure_given_survival, false, false),
        (
            "observability_given_survival_and_exposure",
            &model.observability_given_survival_and_exposure,
            true,
            false,
        ),
        (
            "footprint_coverage_given_observability",
            &model.footprint_coverage_given_observability,
            true,
            true,
        ),
        (
            "conditional_detection_given_covered",
            &model.conditional_detection_given_covered,
            true,
            true,
        ),
        (
            "false_positive_given_absence",
            &model.false_positive_given_absence,
            true,
            true,
        ),
    ];
    for (name, band, method_required, effort_required) in bands {
        validate_probability_band(
            name,
            band,
            observation,
            method_required,
            effort_required,
        )?;
    }

    let coverage = &model.footprint_coverage_given_observability;
    if observation.coverage_receipt_ref.as_deref() != Some(coverage.receipt_ref.as_str())
        || observation.coverage_receipt_sha256.as_deref()
            != Some(coverage.receipt_sha256.as_str())
    {
        return Err(HeritageAbsenceError::InvalidCoverageReceiptBinding);
    }
    Ok(())
}

fn interval(band: &CalibratedProbabilityBand) -> ProbabilityInterval {
    ProbabilityInterval {
        lower: band.lower,
        upper: band.upper,
    }
}

fn bayes_presence_given_non_detection(
    prior_presence: f64,
    no_detection_if_present: f64,
    no_detection_if_absent: f64,
) -> Result<f64, HeritageAbsenceError> {
    let numerator = prior_presence * no_detection_if_present;
    let denominator =
        numerator + (1.0 - prior_presence) * no_detection_if_absent;
    if !denominator.is_finite() || denominator <= f64::EPSILON {
        return Err(HeritageAbsenceError::UndefinedBayesianUpdate);
    }
    Ok(numerator / denominator)
}

/// Interprets one observation only. Repeated visits must not be multiplied as
/// independent negatives without a validated joint dependence model.
pub fn assess_absence_evidence(
    observation: &SurveyObservation,
    model: Option<&NonDetectionModel>,
) -> Result<AbsenceAssessment, HeritageAbsenceError> {
    for (field, value) in [
        ("observation_id", observation.observation_id.as_str()),
        ("survey_unit_id", observation.survey_unit_id.as_str()),
        ("target_class_id", observation.target_class_id.as_str()),
        ("context_id", observation.context_id.as_str()),
        ("method_id", observation.method_id.as_str()),
    ] {
        if !non_empty(value) {
            return Err(HeritageAbsenceError::EmptyIdentity(field.to_owned()));
        }
    }

    let no_update_reason = match observation.outcome {
        SurveyObservationOutcome::Detected => Some(NoUpdateReason::Detected),
        SurveyObservationOutcome::NotSearched => Some(NoUpdateReason::NotSearched),
        SurveyObservationOutcome::NotObservable => Some(NoUpdateReason::NotObservable),
        SurveyObservationOutcome::Inconclusive => Some(NoUpdateReason::Inconclusive),
        SurveyObservationOutcome::SearchedNoDetection => None,
    };
    if let Some(reason) = no_update_reason {
        return Ok(AbsenceAssessment::NoUpdate {
            observation_id: observation.observation_id.clone(),
            reason,
            claim_ceiling: "no_negative_evidence_update_for_this_observation_state".to_owned(),
        });
    }

    let model =
        model.ok_or(HeritageAbsenceError::MissingModelForNegativeObservation)?;

    if !valid_sha256(&observation.effort_profile_sha256) {
        return Err(HeritageAbsenceError::InvalidDigest(
            "effort_profile_sha256".to_owned(),
        ));
    }
    let coverage_ref = observation
        .coverage_receipt_ref
        .as_deref()
        .filter(|value| non_empty(value))
        .ok_or(HeritageAbsenceError::MissingCoverageReceipt)?;
    let coverage_sha = observation
        .coverage_receipt_sha256
        .as_deref()
        .filter(|value| valid_sha256(value))
        .ok_or(HeritageAbsenceError::MissingCoverageReceipt)?;
    let effort_ref = observation
        .effort_receipt_ref
        .as_deref()
        .filter(|value| non_empty(value))
        .ok_or(HeritageAbsenceError::MissingEffortReceipt)?;
    let effort_sha = observation
        .effort_receipt_sha256
        .as_deref()
        .filter(|value| valid_sha256(value))
        .ok_or(HeritageAbsenceError::MissingEffortReceipt)?;
    if !non_empty(coverage_ref) || !non_empty(effort_ref) {
        return Err(HeritageAbsenceError::MissingCoverageReceipt);
    }

    validate_non_detection_model(observation, model)?;

    let detection_bands = [
        &model.survival,
        &model.exposure_given_survival,
        &model.observability_given_survival_and_exposure,
        &model.footprint_coverage_given_observability,
        &model.conditional_detection_given_covered,
    ];
    let p_detect_lower = detection_bands.iter().map(|band| band.lower).product::<f64>();
    let p_detect_upper = detection_bands.iter().map(|band| band.upper).product::<f64>();
    let p_miss_present_lower = 1.0 - p_detect_upper;
    let p_miss_present_upper = 1.0 - p_detect_lower;
    let p_miss_absent_lower = 1.0 - model.false_positive_given_absence.upper;
    let p_miss_absent_upper = 1.0 - model.false_positive_given_absence.lower;

    // Posterior is increasing in prior and P(no-detection | present), and
    // decreasing in P(no-detection | absent). These corners give a conservative
    // outer interval over the declared rectangular parameter bands.
    let posterior_lower = bayes_presence_given_non_detection(
        model.prior_original_presence.lower,
        p_miss_present_lower,
        p_miss_absent_upper,
    )?;
    let posterior_upper = bayes_presence_given_non_detection(
        model.prior_original_presence.upper,
        p_miss_present_upper,
        p_miss_absent_lower,
    )?;

    let calibration_profile_digests = vec![
        model.prior_original_presence.calibration_profile_sha256.clone(),
        model.survival.calibration_profile_sha256.clone(),
        model.exposure_given_survival.calibration_profile_sha256.clone(),
        model
            .observability_given_survival_and_exposure
            .calibration_profile_sha256
            .clone(),
        model
            .footprint_coverage_given_observability
            .calibration_profile_sha256
            .clone(),
        model
            .conditional_detection_given_covered
            .calibration_profile_sha256
            .clone(),
        model.false_positive_given_absence.calibration_profile_sha256.clone(),
    ];

    Ok(AbsenceAssessment::PosteriorUpdate(
        NonDetectionPosteriorReport {
            observation_id: observation.observation_id.clone(),
            model_id: model.model_id.clone(),
            target_class_id: observation.target_class_id.clone(),
            context_id: observation.context_id.clone(),
            method_id: observation.method_id.clone(),
            prior_original_presence: interval(&model.prior_original_presence),
            survival: interval(&model.survival),
            exposure_given_survival: interval(&model.exposure_given_survival),
            observability_given_survival_and_exposure: interval(
                &model.observability_given_survival_and_exposure,
            ),
            footprint_coverage_given_observability: interval(
                &model.footprint_coverage_given_observability,
            ),
            conditional_detection_given_covered: interval(
                &model.conditional_detection_given_covered,
            ),
            false_positive_given_absence: interval(&model.false_positive_given_absence),
            probability_detect_if_originally_present: ProbabilityInterval {
                lower: p_detect_lower,
                upper: p_detect_upper,
            },
            probability_no_detection_if_originally_present: ProbabilityInterval {
                lower: p_miss_present_lower,
                upper: p_miss_present_upper,
            },
            probability_no_detection_if_originally_absent: ProbabilityInterval {
                lower: p_miss_absent_lower,
                upper: p_miss_absent_upper,
            },
            posterior_original_presence_given_no_detection: ProbabilityInterval {
                lower: posterior_lower,
                upper: posterior_upper,
            },
            coverage_receipt_sha256: coverage_sha.to_owned(),
            effort_receipt_sha256: effort_sha.to_owned(),
            calibration_profile_digests,
            claim_ceiling:
                "single_observation_bayesian_update_conditional_on_declared_stage_models_only"
                    .to_owned(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(label: &str) -> String {
        let mut value = label.as_bytes().to_vec();
        value.resize(64, b'x');
        value.truncate(64);
        value.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn band(
        lower: f64,
        upper: f64,
        label: &str,
        method_id: Option<&str>,
        effort: Option<&str>,
    ) -> CalibratedProbabilityBand {
        CalibratedProbabilityBand {
            lower,
            upper,
            context_id: "dry-stone-geomorph-context-1".to_owned(),
            target_class_id: "dry-stone-wall".to_owned(),
            method_id: method_id.map(str::to_owned),
            effort_profile_sha256: effort.map(str::to_owned),
            calibration_profile_sha256: digest(&format!("{label}-profile")),
            calibration_data_sha256: digest(&format!("{label}-data")),
            validation_split_sha256: digest(&format!("{label}-split")),
            conditional_semantics_sha256: digest(&format!("{label}-conditional-contract")),
            receipt_ref: if label == "coverage" {
                "coverage://receipt/1".to_owned()
            } else {
                format!("calibration://receipt/{label}")
            },
            receipt_sha256: if label == "coverage" {
                digest("coverage-receipt")
            } else {
                digest(&format!("{label}-receipt"))
            },
        }
    }

    fn observation(outcome: SurveyObservationOutcome) -> SurveyObservation {
        SurveyObservation {
            observation_id: "negative-survey-1".to_owned(),
            survey_unit_id: "unit-1".to_owned(),
            target_class_id: "dry-stone-wall".to_owned(),
            context_id: "dry-stone-geomorph-context-1".to_owned(),
            method_id: "pedestrian-fieldwalking-v1".to_owned(),
            effort_profile_sha256: digest("effort-profile"),
            outcome,
            coverage_receipt_ref: Some("coverage://receipt/1".to_owned()),
            coverage_receipt_sha256: Some(digest("coverage-receipt")),
            effort_receipt_ref: Some("effort://receipt/1".to_owned()),
            effort_receipt_sha256: Some(digest("effort-receipt")),
        }
    }

    fn model() -> NonDetectionModel {
        let method = "pedestrian-fieldwalking-v1";
        let effort = digest("effort-profile");
        NonDetectionModel {
            model_id: "presence-posterior-model-1".to_owned(),
            context_id: "dry-stone-geomorph-context-1".to_owned(),
            target_class_id: "dry-stone-wall".to_owned(),
            method_id: method.to_owned(),
            effort_profile_sha256: effort.clone(),
            prior_original_presence: band(0.5, 0.5, "prior", None, None),
            survival: band(0.8, 0.8, "survival", None, None),
            exposure_given_survival: band(0.5, 0.5, "exposure", None, None),
            observability_given_survival_and_exposure: band(
                0.5, 0.5, "observability", Some(method), None,
            ),
            footprint_coverage_given_observability: band(
                0.5, 0.5, "coverage", Some(method), Some(&effort),
            ),
            conditional_detection_given_covered: band(
                0.8, 0.8, "detection", Some(method), Some(&effort),
            ),
            false_positive_given_absence: band(
                0.02, 0.02, "false-positive", Some(method), Some(&effort),
            ),
        }
    }

    #[test]
    fn complete_non_detection_chain_updates_prior_with_separate_stages() {
        let result = assess_absence_evidence(
            &observation(SurveyObservationOutcome::SearchedNoDetection),
            Some(&model()),
        )
        .expect("valid one-event update");
        let AbsenceAssessment::PosteriorUpdate(report) = result else {
            panic!("searched no-detection should update");
        };
        assert!((report.probability_detect_if_originally_present.lower - 0.08).abs() < 1e-12);
        assert!((report.probability_no_detection_if_originally_present.lower - 0.92).abs() < 1e-12);
        assert!((report.probability_no_detection_if_originally_absent.lower - 0.98).abs() < 1e-12);
        let expected = 0.46 / (0.46 + 0.49);
        assert!((report.posterior_original_presence_given_no_detection.lower - expected).abs() < 1e-12);
        assert_eq!(report.claim_ceiling, "single_observation_bayesian_update_conditional_on_declared_stage_models_only");
    }

    #[test]
    fn parameter_bands_produce_an_outer_posterior_interval() {
        let mut input = model();
        input.prior_original_presence = band(0.4, 0.6, "prior", None, None);
        input.survival = band(0.7, 0.9, "survival", None, None);
        input.exposure_given_survival = band(0.4, 0.7, "exposure", None, None);
        input.observability_given_survival_and_exposure = band(
            0.3, 0.8, "observability", Some(&input.method_id), None,
        );
        let effort = input.effort_profile_sha256.clone();
        input.footprint_coverage_given_observability = band(
            0.4, 0.9, "coverage", Some(&input.method_id), Some(&effort),
        );
        input.conditional_detection_given_covered = band(
            0.6, 0.95, "detection", Some(&input.method_id), Some(&effort),
        );
        input.false_positive_given_absence = band(
            0.01, 0.05, "false-positive", Some(&input.method_id), Some(&effort),
        );
        let result = assess_absence_evidence(
            &observation(SurveyObservationOutcome::SearchedNoDetection),
            Some(&input),
        )
        .expect("interval update");
        let AbsenceAssessment::PosteriorUpdate(report) = result else {
            panic!("searched no-detection should update");
        };
        let posterior = report.posterior_original_presence_given_no_detection;
        assert!(posterior.lower <= posterior.upper);
        assert!(posterior.lower >= 0.0 && posterior.upper <= 1.0);
        assert!(report.probability_detect_if_originally_present.lower
            < report.probability_detect_if_originally_present.upper);
    }

    #[test]
    fn not_searched_not_observable_and_inconclusive_do_not_update() {
        for (outcome, reason) in [
            (SurveyObservationOutcome::NotSearched, NoUpdateReason::NotSearched),
            (SurveyObservationOutcome::NotObservable, NoUpdateReason::NotObservable),
            (SurveyObservationOutcome::Inconclusive, NoUpdateReason::Inconclusive),
            (SurveyObservationOutcome::Detected, NoUpdateReason::Detected),
        ] {
            let result = assess_absence_evidence(&observation(outcome), None)
                .expect("non-negative state remains a record");
            assert!(matches!(
                result,
                AbsenceAssessment::NoUpdate { reason: actual, .. } if actual == reason
            ));
        }
    }

    #[test]
    fn negative_observation_requires_calibration_model_and_actual_coverage_effort() {
        assert_eq!(
            assess_absence_evidence(
                &observation(SurveyObservationOutcome::SearchedNoDetection),
                None
            ),
            Err(HeritageAbsenceError::MissingModelForNegativeObservation)
        );
        let mut input = observation(SurveyObservationOutcome::SearchedNoDetection);
        input.coverage_receipt_ref = None;
        assert_eq!(
            assess_absence_evidence(&input, Some(&model())),
            Err(HeritageAbsenceError::MissingCoverageReceipt)
        );
    }

    #[test]
    fn model_and_observation_context_method_and_effort_must_match() {
        let mut input = model();
        input.exposure_given_survival.context_id = "other-context".to_owned();
        assert!(matches!(
            assess_absence_evidence(
                &observation(SurveyObservationOutcome::SearchedNoDetection),
                Some(&input)
            ),
            Err(HeritageAbsenceError::InvalidProbabilityBandContext(_))
        ));

        let mut input = model();
        input.conditional_detection_given_covered.method_id =
            Some("different-method".to_owned());
        assert!(matches!(
            assess_absence_evidence(
                &observation(SurveyObservationOutcome::SearchedNoDetection),
                Some(&input)
            ),
            Err(HeritageAbsenceError::InvalidMethodBinding(_))
        ));
    }

    #[test]
    fn coverage_receipt_must_be_the_one_used_by_the_coverage_band() {
        let mut input = observation(SurveyObservationOutcome::SearchedNoDetection);
        input.coverage_receipt_sha256 = Some(digest("other-coverage"));
        assert!(matches!(
            assess_absence_evidence(&input, Some(&model())),
            Err(HeritageAbsenceError::InvalidCoverageReceiptBinding)
        ));
    }

    #[test]
    fn invalid_probability_or_digest_bounds_fail_closed() {
        let mut input = model();
        input.survival.upper = f64::NAN;
        assert!(matches!(
            assess_absence_evidence(
                &observation(SurveyObservationOutcome::SearchedNoDetection),
                Some(&input)
            ),
            Err(HeritageAbsenceError::InvalidProbabilityBand(_))
        ));

        let mut input = model();
        input.false_positive_given_absence.calibration_profile_sha256 = "not-a-digest".to_owned();
        assert!(matches!(
            assess_absence_evidence(
                &observation(SurveyObservationOutcome::SearchedNoDetection),
                Some(&input)
            ),
            Err(HeritageAbsenceError::InvalidProbabilityBand(_))
        ));
    }
}
