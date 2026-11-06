// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Exact event-graph oracle for HDB-24 probability composition.
//!
//! This is a deterministic synthetic benchmark, not a field calibration model.
//! The chain is a product of sequential conditional probabilities, so the
//! factorized and joint representations must agree by the probability chain
//! rule without any mutual-independence assumption.
//!
//! The deliberately incorrect double-count result is emitted only as a
//! negative-control comparator. The production factor-scope validator rejects
//! any factor list that mixes a joint POD with its component factors.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const HERITAGE_PROBABILITY_BENCHMARK_SCHEMA_VERSION: &str =
    "luminous.heritage-probability-benchmark.v0.1";
const PROBABILITY_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionMode {
    FactorizedChain,
    CompositeSurveyUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionFactor {
    Survival,
    ExposureGivenSurvival,
    ObservabilityGivenSurvivalAndExposure,
    FootprintCoverageGivenPriorStages,
    ConditionalDetectionGivenPriorStages,
    JointPerSurveyUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventProbabilities {
    pub survival: f64,
    pub exposure_given_survival: f64,
    pub observability_given_survival_and_exposure: f64,
    pub footprint_coverage_given_prior_stages: f64,
    pub conditional_detection_given_prior_stages: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageProbabilityBenchmarkInputV1 {
    pub scenario_id: String,
    pub scenario_manifest_sha256: String,
    pub evaluation_protocol_sha256: String,
    pub evaluator_build_sha256: String,
    pub prior_probability_originally_present: f64,
    pub events: EventProbabilities,
    /// Independently supplied synthetic joint POD. It must equal the chain
    /// product within tolerance; it is not derived inside this benchmark.
    pub joint_probability_of_detection_per_survey_unit: f64,
    pub false_positive_probability_if_originally_absent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodScoreV1 {
    pub probability_of_detection_if_present: f64,
    pub posterior_present_given_detection: Option<f64>,
    pub posterior_present_given_no_detection: Option<f64>,
    pub posterior_bias_given_detection: Option<f64>,
    pub posterior_bias_given_no_detection: Option<f64>,
    /// Exact expected squared-error loss for posterior presence probability.
    pub expected_brier_score: f64,
    /// Exact expected logarithmic loss for posterior presence probability.
    pub expected_log_loss: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageProbabilityBenchmarkReportV1 {
    pub schema_version: String,
    pub scenario_id: String,
    pub scenario_manifest_sha256: String,
    pub evaluation_protocol_sha256: String,
    pub evaluator_build_sha256: String,
    pub upstream_event_chain_probability: f64,
    pub conditional_detection_probability: f64,
    pub exact_factorized_probability_of_detection: f64,
    pub declared_joint_probability_of_detection: f64,
    pub exact_factorization_residual: f64,
    pub false_positive_probability_if_originally_absent: f64,
    pub exact_factorized: MethodScoreV1,
    pub exact_composite: MethodScoreV1,
    /// Deliberately incorrect: joint POD multiplied by all upstream factors
    /// again. This must never be accepted by the production factor validator.
    pub naive_double_count_negative_control: MethodScoreV1,
    pub exact_factorized_and_composite_posterior_agree: bool,
    pub naive_double_count_has_nonzero_bias: bool,
    pub mutual_independence_assumed: bool,
    pub archaeological_absence_claimed: bool,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageProbabilityBenchmarkError {
    EmptyScenarioId,
    InvalidDigest(&'static str),
    InvalidProbability(&'static str),
    JointProbabilityDoesNotMatchConditionalChain,
    InvalidFactorScope(CompositionMode),
    DuplicateFactor(CompositionFactor),
}

impl fmt::Display for HeritageProbabilityBenchmarkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage probability benchmark error: {self:?}")
    }
}

impl std::error::Error for HeritageProbabilityBenchmarkError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_probability(value: f64, label: &'static str) -> Result<(), HeritageProbabilityBenchmarkError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(HeritageProbabilityBenchmarkError::InvalidProbability(label));
    }
    Ok(())
}

/// Validates the only two supported, non-overlapping factor sets.
///
/// Factor order is canonical and part of the deterministic contract. A joint
/// probability is exclusive of all five conditional components.
pub fn validate_composition_factor_scope(
    mode: CompositionMode,
    factors: &[CompositionFactor],
) -> Result<(), HeritageProbabilityBenchmarkError> {
    let mut seen = Vec::with_capacity(factors.len());
    for factor in factors {
        if seen.contains(factor) {
            return Err(HeritageProbabilityBenchmarkError::DuplicateFactor(*factor));
        }
        seen.push(*factor);
    }

    let expected: &[CompositionFactor] = match mode {
        CompositionMode::FactorizedChain => &[
            CompositionFactor::Survival,
            CompositionFactor::ExposureGivenSurvival,
            CompositionFactor::ObservabilityGivenSurvivalAndExposure,
            CompositionFactor::FootprintCoverageGivenPriorStages,
            CompositionFactor::ConditionalDetectionGivenPriorStages,
        ],
        CompositionMode::CompositeSurveyUnit => &[CompositionFactor::JointPerSurveyUnit],
    };
    if factors != expected {
        return Err(HeritageProbabilityBenchmarkError::InvalidFactorScope(mode));
    }
    Ok(())
}

fn posterior(
    prior: f64,
    likelihood_if_present: f64,
    likelihood_if_absent: f64,
) -> Option<f64> {
    let present = prior * likelihood_if_present;
    let absent = (1.0 - prior) * likelihood_if_absent;
    let denominator = present + absent;
    if !denominator.is_finite() || denominator <= 0.0 {
        None
    } else {
        Some((present / denominator).clamp(0.0, 1.0))
    }
}

fn exact_scores(
    prior: f64,
    pod: f64,
    false_positive: f64,
) -> (Option<f64>, Option<f64>, f64, Option<f64>) {
    let present_detected = prior * pod;
    let absent_detected = (1.0 - prior) * false_positive;
    let present_missed = prior * (1.0 - pod);
    let absent_missed = (1.0 - prior) * (1.0 - false_positive);

    let detected_mass = present_detected + absent_detected;
    let missed_mass = present_missed + absent_missed;

    let posterior_detected = posterior(prior, pod, false_positive);
    let posterior_missed = posterior(prior, 1.0 - pod, 1.0 - false_positive);

    let mut brier = 0.0;
    if detected_mass > 0.0 {
        let p = posterior_detected.expect("positive detected mass implies a defined posterior");
        brier += present_detected * (1.0 - p).powi(2) + absent_detected * p.powi(2);
    }
    if missed_mass > 0.0 {
        let p = posterior_missed.expect("positive missed mass implies a defined posterior");
        brier += present_missed * (1.0 - p).powi(2) + absent_missed * p.powi(2);
    }

    // Exact expected log loss can be infinite when an observation with
    // positive probability is assigned zero posterior mass to its true state.
    let mut log_loss = 0.0;
    let mut log_defined = true;
    for (present_mass, absent_mass, estimate) in [
        (present_detected, absent_detected, posterior_detected),
        (present_missed, absent_missed, posterior_missed),
    ] {
        if present_mass + absent_mass == 0.0 {
            continue;
        }
        let Some(p) = estimate else {
            log_defined = false;
            continue;
        };
        if (present_mass > 0.0 && p <= 0.0) || (absent_mass > 0.0 && p >= 1.0) {
            log_defined = false;
            continue;
        }
        if present_mass > 0.0 {
            log_loss -= present_mass * p.ln();
        }
        if absent_mass > 0.0 {
            log_loss -= absent_mass * (1.0 - p).ln();
        }
    }

    let log_score = if log_defined { Some(log_loss) } else { None };
    (posterior_detected, posterior_missed, brier, log_score)
}

fn method_score(
    prior: f64,
    pod: f64,
    false_positive: f64,
    reference_detected: Option<f64>,
    reference_missed: Option<f64>,
) -> MethodScoreV1 {
    let (posterior_detected, posterior_missed, brier, log_loss) =
        exact_scores(prior, pod, false_positive);
    MethodScoreV1 {
        probability_of_detection_if_present: pod,
        posterior_present_given_detection: posterior_detected,
        posterior_present_given_no_detection: posterior_missed,
        posterior_bias_given_detection: posterior_detected
            .zip(reference_detected)
            .map(|(value, reference)| value - reference),
        posterior_bias_given_no_detection: posterior_missed
            .zip(reference_missed)
            .map(|(value, reference)| value - reference),
        expected_brier_score: brier,
        expected_log_loss: log_loss,
    }
}

pub fn evaluate_probability_composition_benchmark(
    input: &HeritageProbabilityBenchmarkInputV1,
) -> Result<HeritageProbabilityBenchmarkReportV1, HeritageProbabilityBenchmarkError> {
    if !non_empty(&input.scenario_id) {
        return Err(HeritageProbabilityBenchmarkError::EmptyScenarioId);
    }
    for (value, label) in [
        (&input.scenario_manifest_sha256, "scenario_manifest_sha256"),
        (&input.evaluation_protocol_sha256, "evaluation_protocol_sha256"),
        (&input.evaluator_build_sha256, "evaluator_build_sha256"),
    ] {
        if !valid_sha256(value) {
            return Err(HeritageProbabilityBenchmarkError::InvalidDigest(label));
        }
    }
    validate_probability(input.prior_probability_originally_present, "prior_probability")?;
    validate_probability(
        input.joint_probability_of_detection_per_survey_unit,
        "joint_probability_of_detection_per_survey_unit",
    )?;
    validate_probability(
        input.false_positive_probability_if_originally_absent,
        "false_positive_probability_if_originally_absent",
    )?;
    for (value, label) in [
        (input.events.survival, "survival"),
        (input.events.exposure_given_survival, "exposure_given_survival"),
        (
            input.events.observability_given_survival_and_exposure,
            "observability_given_survival_and_exposure",
        ),
        (
            input.events.footprint_coverage_given_prior_stages,
            "footprint_coverage_given_prior_stages",
        ),
        (
            input.events.conditional_detection_given_prior_stages,
            "conditional_detection_given_prior_stages",
        ),
    ] {
        validate_probability(value, label)?;
    }

    let upstream = input.events.survival
        * input.events.exposure_given_survival
        * input.events.observability_given_survival_and_exposure
        * input.events.footprint_coverage_given_prior_stages;
    let exact_factorized = upstream * input.events.conditional_detection_given_prior_stages;
    let residual =
        input.joint_probability_of_detection_per_survey_unit - exact_factorized;
    if residual.abs() > PROBABILITY_TOLERANCE {
        return Err(
            HeritageProbabilityBenchmarkError::JointProbabilityDoesNotMatchConditionalChain,
        );
    }

    let reference = method_score(
        input.prior_probability_originally_present,
        exact_factorized,
        input.false_positive_probability_if_originally_absent,
        None,
        None,
    );
    let exact_factorized_score = reference;
    let exact_composite_score = method_score(
        input.prior_probability_originally_present,
        input.joint_probability_of_detection_per_survey_unit,
        input.false_positive_probability_if_originally_absent,
        reference.posterior_present_given_detection,
        reference.posterior_present_given_no_detection,
    );

    // Negative control only: the joint POD already includes upstream events,
    // so multiplying by the upstream chain again is deliberately incorrect.
    let naive_pod =
        input.joint_probability_of_detection_per_survey_unit * upstream;
    let naive_score = method_score(
        input.prior_probability_originally_present,
        naive_pod,
        input.false_positive_probability_if_originally_absent,
        reference.posterior_present_given_detection,
        reference.posterior_present_given_no_detection,
    );

    let posterior_agrees = match (
        reference.posterior_present_given_detection,
        exact_composite_score.posterior_present_given_detection,
        reference.posterior_present_given_no_detection,
        exact_composite_score.posterior_present_given_no_detection,
    ) {
        (Some(left_d), Some(right_d), Some(left_m), Some(right_m)) => {
            (left_d - right_d).abs() <= PROBABILITY_TOLERANCE
                && (left_m - right_m).abs() <= PROBABILITY_TOLERANCE
        }
        (None, None, None, None) => true,
        _ => false,
    };
    let nonzero_bias = naive_score
        .posterior_bias_given_detection
        .is_some_and(|bias| bias.abs() > PROBABILITY_TOLERANCE)
        || naive_score
            .posterior_bias_given_no_detection
            .is_some_and(|bias| bias.abs() > PROBABILITY_TOLERANCE);

    Ok(HeritageProbabilityBenchmarkReportV1 {
        schema_version: HERITAGE_PROBABILITY_BENCHMARK_SCHEMA_VERSION.to_owned(),
        scenario_id: input.scenario_id.clone(),
        scenario_manifest_sha256: input.scenario_manifest_sha256.clone(),
        evaluation_protocol_sha256: input.evaluation_protocol_sha256.clone(),
        evaluator_build_sha256: input.evaluator_build_sha256.clone(),
        upstream_event_chain_probability: upstream,
        conditional_detection_probability: input.events.conditional_detection_given_prior_stages,
        exact_factorized_probability_of_detection: exact_factorized,
        declared_joint_probability_of_detection: input.joint_probability_of_detection_per_survey_unit,
        exact_factorization_residual: residual,
        false_positive_probability_if_originally_absent: input
            .false_positive_probability_if_originally_absent,
        exact_factorized: exact_factorized_score,
        exact_composite: exact_composite_score,
        naive_double_count_negative_control: naive_score,
        exact_factorized_and_composite_posterior_agree: posterior_agrees,
        naive_double_count_has_nonzero_bias: nonzero_bias,
        mutual_independence_assumed: false,
        archaeological_absence_claimed: false,
        claim_ceiling: "synthetic_probability_composition_correctness_only_no_field_calibration".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn input() -> HeritageProbabilityBenchmarkInputV1 {
        HeritageProbabilityBenchmarkInputV1 {
            scenario_id: "non-degenerate-event-graph-1".to_owned(),
            scenario_manifest_sha256: digest('a'),
            evaluation_protocol_sha256: digest('b'),
            evaluator_build_sha256: digest('c'),
            prior_probability_originally_present: 0.2,
            events: EventProbabilities {
                survival: 0.8,
                exposure_given_survival: 0.6,
                observability_given_survival_and_exposure: 0.7,
                footprint_coverage_given_prior_stages: 0.5,
                conditional_detection_given_prior_stages: 0.9,
            },
            joint_probability_of_detection_per_survey_unit: 0.1512,
            false_positive_probability_if_originally_absent: 0.05,
        }
    }

    #[test]
    fn exact_factorized_and_composite_oracles_agree_for_non_degenerate_case() {
        let report = evaluate_probability_composition_benchmark(&input()).unwrap();
        assert!((report.upstream_event_chain_probability - 0.168).abs() < 1e-12);
        assert!((report.exact_factorized_probability_of_detection - 0.1512).abs() < 1e-12);
        assert!(report.exact_factorized_and_composite_posterior_agree);
        assert!(!report.mutual_independence_assumed);
        assert!(!report.archaeological_absence_claimed);
        assert!(!report.naive_double_count_negative_control.posterior_bias_given_no_detection.unwrap().abs().eq(&0.0));
    }

    #[test]
    fn double_count_negative_control_has_measurable_posterior_bias_and_worse_proper_scores() {
        let report = evaluate_probability_composition_benchmark(&input()).unwrap();
        let negative = report.naive_double_count_negative_control;
        assert!(report.naive_double_count_has_nonzero_bias);
        assert!(negative.posterior_bias_given_no_detection.unwrap().abs() > 0.01);
        assert!(negative.expected_brier_score > report.exact_factorized.expected_brier_score);
        assert!(
            negative.expected_log_loss.unwrap()
                > report.exact_factorized.expected_log_loss.unwrap()
        );
    }

    #[test]
    fn hand_calculated_reference_scores_match_the_exact_oracle() {
        let report = evaluate_probability_composition_benchmark(&input()).unwrap();
        assert!((report.exact_factorized.posterior_present_given_detection.unwrap()
            - 0.43052391799544415)
            .abs()
            < 1e-12);
        assert!((report.exact_factorized.posterior_present_given_no_detection.unwrap()
            - 0.18258475305455174)
            .abs()
            < 1e-12);
        assert!((report.exact_factorized.expected_brier_score - 0.1559853690412771).abs() < 1e-12);
        assert!((report.exact_factorized.expected_log_loss.unwrap() - 0.4899123026529449).abs() < 1e-12);
        assert!((report.naive_double_count_negative_control.expected_brier_score
            - 0.1596401679441963)
            .abs()
            < 1e-12);
        assert!((report.naive_double_count_negative_control.expected_log_loss.unwrap()
            - 0.4991231437756762)
            .abs()
            < 1e-12);
    }

    #[test]
    fn exact_methods_agree_and_double_count_bias_persists_across_synthetic_contexts() {
        let cases = [
            (0.3, [0.95, 0.8, 0.4, 0.65, 0.75], 0.02),
            (0.45, [0.7, 0.5, 0.8, 0.75, 0.6], 0.08),
            (0.1, [0.9, 0.9, 0.9, 0.4, 0.95], 0.01),
        ];
        for (index, (prior, p, false_positive)) in cases.into_iter().enumerate() {
            let mut value = input();
            value.scenario_id = format!("matrix-case-{index}");
            value.prior_probability_originally_present = prior;
            value.events = EventProbabilities {
                survival: p[0],
                exposure_given_survival: p[1],
                observability_given_survival_and_exposure: p[2],
                footprint_coverage_given_prior_stages: p[3],
                conditional_detection_given_prior_stages: p[4],
            };
            value.joint_probability_of_detection_per_survey_unit =
                p.into_iter().product::<f64>();
            value.false_positive_probability_if_originally_absent = false_positive;

            let report = evaluate_probability_composition_benchmark(&value).unwrap();
            assert!(report.exact_factorized_and_composite_posterior_agree);
            assert!(report.naive_double_count_has_nonzero_bias);
            assert!(
                report.naive_double_count_negative_control.expected_brier_score
                    > report.exact_factorized.expected_brier_score
            );
            assert!(
                report.naive_double_count_negative_control.expected_log_loss.unwrap()
                    > report.exact_factorized.expected_log_loss.unwrap()
            );
        }
    }

    #[test]
    fn joint_probability_must_match_the_exact_conditional_chain() {
        let mut value = input();
        value.joint_probability_of_detection_per_survey_unit = 0.2;
        assert_eq!(
            evaluate_probability_composition_benchmark(&value),
            Err(HeritageProbabilityBenchmarkError::JointProbabilityDoesNotMatchConditionalChain)
        );
    }

    #[test]
    fn non_finite_and_out_of_range_probabilities_fail_closed() {
        let mut value = input();
        value.events.observability_given_survival_and_exposure = f64::NAN;
        assert_eq!(
            evaluate_probability_composition_benchmark(&value),
            Err(HeritageProbabilityBenchmarkError::InvalidProbability(
                "observability_given_survival_and_exposure"
            ))
        );

        let mut value = input();
        value.false_positive_probability_if_originally_absent = 1.01;
        assert_eq!(
            evaluate_probability_composition_benchmark(&value),
            Err(HeritageProbabilityBenchmarkError::InvalidProbability(
                "false_positive_probability_if_originally_absent"
            ))
        );
    }

    #[test]
    fn degenerate_zero_likelihood_is_reported_as_undefined_posterior() {
        let mut value = input();
        value.prior_probability_originally_present = 1.0;
        value.joint_probability_of_detection_per_survey_unit = 0.0;
        value.events.conditional_detection_given_prior_stages = 0.0;
        let report = evaluate_probability_composition_benchmark(&value).unwrap();
        assert_eq!(report.exact_factorized.posterior_present_given_detection, None);
        assert_eq!(report.exact_factorized.posterior_present_given_no_detection, Some(1.0));
    }

    #[test]
    fn only_exact_non_overlapping_factor_sets_are_accepted() {
        let factorized = [
            CompositionFactor::Survival,
            CompositionFactor::ExposureGivenSurvival,
            CompositionFactor::ObservabilityGivenSurvivalAndExposure,
            CompositionFactor::FootprintCoverageGivenPriorStages,
            CompositionFactor::ConditionalDetectionGivenPriorStages,
        ];
        assert!(validate_composition_factor_scope(
            CompositionMode::FactorizedChain,
            &factorized
        )
        .is_ok());
        assert!(validate_composition_factor_scope(
            CompositionMode::CompositeSurveyUnit,
            &[CompositionFactor::JointPerSurveyUnit]
        )
        .is_ok());
    }

    #[test]
    fn joint_pod_cannot_be_mixed_with_upstream_or_conditional_factors() {
        assert_eq!(
            validate_composition_factor_scope(
                CompositionMode::CompositeSurveyUnit,
                &[
                    CompositionFactor::JointPerSurveyUnit,
                    CompositionFactor::Survival
                ]
            ),
            Err(HeritageProbabilityBenchmarkError::InvalidFactorScope(
                CompositionMode::CompositeSurveyUnit
            ))
        );
        assert_eq!(
            validate_composition_factor_scope(
                CompositionMode::FactorizedChain,
                &[
                    CompositionFactor::Survival,
                    CompositionFactor::ExposureGivenSurvival,
                    CompositionFactor::ObservabilityGivenSurvivalAndExposure,
                    CompositionFactor::FootprintCoverageGivenPriorStages,
                    CompositionFactor::ConditionalDetectionGivenPriorStages,
                    CompositionFactor::JointPerSurveyUnit,
                ]
            ),
            Err(HeritageProbabilityBenchmarkError::InvalidFactorScope(
                CompositionMode::FactorizedChain
            ))
        );
    }

    #[test]
    fn duplicate_and_reordered_factors_are_rejected() {
        assert_eq!(
            validate_composition_factor_scope(
                CompositionMode::CompositeSurveyUnit,
                &[CompositionFactor::JointPerSurveyUnit, CompositionFactor::JointPerSurveyUnit]
            ),
            Err(HeritageProbabilityBenchmarkError::DuplicateFactor(
                CompositionFactor::JointPerSurveyUnit
            ))
        );
        assert_eq!(
            validate_composition_factor_scope(
                CompositionMode::FactorizedChain,
                &[CompositionFactor::ExposureGivenSurvival, CompositionFactor::Survival]
            ),
            Err(HeritageProbabilityBenchmarkError::InvalidFactorScope(
                CompositionMode::FactorizedChain
            ))
        );
    }
}
