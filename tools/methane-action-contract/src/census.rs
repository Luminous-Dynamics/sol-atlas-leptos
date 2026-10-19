//! Complete-census scoring for methane detection benchmark submissions.
//!
//! The evaluator is intentionally independent of detector implementations.
//! Every planned opportunity must have exactly one response row. No-coverage,
//! invalid-scene, abstention, and unscorable-reference states remain visible.
//! This computes benchmark metrics; it does not authenticate a system or prove
//! that a reference label or reported measurement is physically true.

use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceState {
    MeteredEmission { emission_rate_kg_h: f64 },
    MeteredZero,
    ReferenceUnavailable { reason: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkCase {
    pub case_id: String,
    /// Predeclared stratum, e.g. rate band × scene quality × sensor family.
    pub stratum_id: String,
    pub reference: ReferenceState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoverageState {
    Covered,
    NotTasked { reason: String },
    InvalidScene { reason: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum SystemDisposition {
    Detected {
        emission_rate_kg_h: f64,
    },
    ExplicitNonDetect,
    Abstained {
        reason: String,
    },
    /// Valid only when coverage is NotTasked or InvalidScene.
    NoOutput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaseResponse {
    pub case_id: String,
    pub coverage: CoverageState,
    pub disposition: SystemDisposition,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemSubmission {
    pub system_id: String,
    pub version: String,
    /// Declared digest; this module checks its canonical format, not provenance.
    pub build_sha256: String,
    pub configuration_sha256: String,
    pub responses: Vec<CaseResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CensusViolation {
    pub record_id: String,
    pub code: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DetectionCounts {
    pub reference_known_cases: usize,
    pub reference_unavailable_cases: usize,
    pub positive_reference_cases: usize,
    pub negative_reference_cases: usize,
    pub covered_known_cases: usize,
    pub covered_positive_cases: usize,
    pub covered_negative_cases: usize,
    pub not_tasked_positive_cases: usize,
    pub not_tasked_negative_cases: usize,
    pub invalid_scene_positive_cases: usize,
    pub invalid_scene_negative_cases: usize,
    pub true_positives: usize,
    /// Includes positive cases missed because of non-detection, abstention,
    /// no tasking, or invalid scenes.
    pub false_negatives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub abstained_positive_cases: usize,
    pub abstained_negative_cases: usize,
}

impl DetectionCounts {
    /// End-to-end recall: no-tasking and invalid-scene positives remain misses.
    pub fn end_to_end_recall(&self) -> Option<f64> {
        ratio(self.true_positives, self.positive_reference_cases)
    }

    /// Recall among covered positive-reference cases, including abstentions as misses.
    pub fn recall_when_covered(&self) -> Option<f64> {
        ratio(self.true_positives, self.covered_positive_cases)
    }

    /// False-alert rate over every covered, metered-zero opportunity.
    pub fn false_alert_rate_when_covered(&self) -> Option<f64> {
        ratio(self.false_positives, self.covered_negative_cases)
    }

    /// Fraction of all cases with known reference truth that were covered.
    pub fn coverage_rate(&self) -> Option<f64> {
        ratio(self.covered_known_cases, self.reference_known_cases)
    }

    /// Abstention rate over covered cases with a known reference label.
    pub fn abstention_rate_when_covered(&self) -> Option<f64> {
        ratio(
            self.abstained_positive_cases + self.abstained_negative_cases,
            self.covered_known_cases,
        )
    }

    /// Two-sided 95% Wilson score interval for end-to-end recall.
    pub fn end_to_end_recall_ci95(&self) -> Option<(f64, f64)> {
        wilson_interval_95(self.true_positives, self.positive_reference_cases)
    }

    /// Two-sided 95% Wilson score interval for recall conditional on coverage.
    pub fn recall_when_covered_ci95(&self) -> Option<(f64, f64)> {
        wilson_interval_95(self.true_positives, self.covered_positive_cases)
    }

    /// Two-sided 95% Wilson score interval for false-alert rate on covered negatives.
    pub fn false_alert_rate_when_covered_ci95(&self) -> Option<(f64, f64)> {
        wilson_interval_95(self.false_positives, self.covered_negative_cases)
    }

    /// Two-sided 95% Wilson score interval for reference-known coverage.
    pub fn coverage_rate_ci95(&self) -> Option<(f64, f64)> {
        wilson_interval_95(self.covered_known_cases, self.reference_known_cases)
    }

    /// Two-sided 95% Wilson score interval for abstention among covered known cases.
    pub fn abstention_rate_when_covered_ci95(&self) -> Option<(f64, f64)> {
        wilson_interval_95(
            self.abstained_positive_cases + self.abstained_negative_cases,
            self.covered_known_cases,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkMetrics {
    pub system_id: String,
    pub version: String,
    pub build_sha256: String,
    pub configuration_sha256: String,
    pub overall: DetectionCounts,
    pub by_stratum: BTreeMap<String, DetectionCounts>,
    /// Computed only over true-positive detections with metered positive truth.
    pub mean_absolute_log10_rate_error: Option<f64>,
    pub median_absolute_log10_rate_error: Option<f64>,
    pub quantification_sample_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvaluationReport {
    pub violations: Vec<CensusViolation>,
    /// Missing/extra/duplicate rows or malformed states suppress all headline metrics.
    pub metrics: Option<BenchmarkMetrics>,
}

impl EvaluationReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty() && self.metrics.is_some()
    }

    fn push(&mut self, record_id: &str, code: &'static str) {
        self.violations.push(CensusViolation {
            record_id: record_id.to_owned(),
            code,
        });
    }
}

/// Evaluate a submission against the complete, predeclared set of test opportunities.
///
/// Any missing, extra, duplicate, or semantically malformed row invalidates the whole
/// score. The function never treats a missing row as a true negative and never silently
/// filters hard cases out of the denominator.
pub fn evaluate_census(cases: &[BenchmarkCase], submission: &SystemSubmission) -> EvaluationReport {
    let mut report = EvaluationReport::default();

    if cases.is_empty() {
        report.push("__benchmark__", "empty_case_universe");
    }
    if submission.system_id.trim().is_empty() {
        report.push("__submission__", "empty_system_id");
    }
    if submission.version.trim().is_empty() {
        report.push("__submission__", "empty_system_version");
    }
    if !is_canonical_sha256(&submission.build_sha256) {
        report.push("__submission__", "invalid_build_sha256");
    }
    if !is_canonical_sha256(&submission.configuration_sha256) {
        report.push("__submission__", "invalid_configuration_sha256");
    }

    let mut cases_by_id: HashMap<&str, &BenchmarkCase> = HashMap::new();
    for case in cases {
        if case.case_id.trim().is_empty() {
            report.push("__benchmark__", "empty_case_id");
        }
        if case.stratum_id.trim().is_empty() {
            report.push(&case.case_id, "empty_stratum_id");
        }
        if cases_by_id.insert(&case.case_id, case).is_some() {
            report.push(&case.case_id, "duplicate_benchmark_case_id");
        }
        match &case.reference {
            ReferenceState::MeteredEmission { emission_rate_kg_h }
                if !emission_rate_kg_h.is_finite() || *emission_rate_kg_h <= 0.0 =>
            {
                report.push(&case.case_id, "invalid_reference_emission_rate");
            }
            ReferenceState::ReferenceUnavailable { reason } if reason.trim().is_empty() => {
                report.push(&case.case_id, "empty_reference_unavailable_reason");
            }
            _ => {}
        }
    }

    let mut responses_by_id: HashMap<&str, &CaseResponse> = HashMap::new();
    for response in &submission.responses {
        if response.case_id.trim().is_empty() {
            report.push("__submission__", "empty_response_case_id");
        }
        if !cases_by_id.contains_key(response.case_id.as_str()) {
            report.push(&response.case_id, "response_for_unknown_case");
        }
        if responses_by_id
            .insert(&response.case_id, response)
            .is_some()
        {
            report.push(&response.case_id, "duplicate_response_case_id");
        }
        validate_response_shape(&mut report, response);
    }

    for case in cases {
        if !responses_by_id.contains_key(case.case_id.as_str()) {
            report.push(&case.case_id, "missing_response_row");
        }
    }

    report
        .violations
        .sort_by(|left, right| (&left.record_id, left.code).cmp(&(&right.record_id, right.code)));

    if !report.violations.is_empty() {
        return report;
    }

    let mut overall = Accumulator::default();
    let mut per_stratum: BTreeMap<String, Accumulator> = BTreeMap::new();

    for case in cases {
        // Exact census coverage was checked above, so this lookup must succeed.
        let response = responses_by_id[case.case_id.as_str()];
        record_case(&mut overall, case, response);
        record_case(
            per_stratum.entry(case.stratum_id.clone()).or_default(),
            case,
            response,
        );
    }

    let mut errors = overall.absolute_log10_rate_errors;
    errors.sort_by(f64::total_cmp);
    let mean_absolute_log10_rate_error = if errors.is_empty() {
        None
    } else {
        Some(errors.iter().sum::<f64>() / errors.len() as f64)
    };
    let median_absolute_log10_rate_error = median_sorted(&errors);
    let quantification_sample_count = errors.len();

    report.metrics = Some(BenchmarkMetrics {
        system_id: submission.system_id.clone(),
        version: submission.version.clone(),
        build_sha256: submission.build_sha256.clone(),
        configuration_sha256: submission.configuration_sha256.clone(),
        overall: overall.counts,
        by_stratum: per_stratum
            .into_iter()
            .map(|(stratum, accumulator)| (stratum, accumulator.counts))
            .collect(),
        mean_absolute_log10_rate_error,
        median_absolute_log10_rate_error,
        quantification_sample_count,
    });
    report
}

#[derive(Clone, Debug, Default)]
struct Accumulator {
    counts: DetectionCounts,
    absolute_log10_rate_errors: Vec<f64>,
}

fn record_case(accumulator: &mut Accumulator, case: &BenchmarkCase, response: &CaseResponse) {
    if matches!(&case.reference, ReferenceState::ReferenceUnavailable { .. }) {
        accumulator.counts.reference_unavailable_cases += 1;
        return;
    }

    accumulator.counts.reference_known_cases += 1;
    let is_positive = matches!(&case.reference, ReferenceState::MeteredEmission { .. });
    if is_positive {
        accumulator.counts.positive_reference_cases += 1;
    } else {
        accumulator.counts.negative_reference_cases += 1;
    }

    match &response.coverage {
        CoverageState::Covered => {
            accumulator.counts.covered_known_cases += 1;
            if is_positive {
                accumulator.counts.covered_positive_cases += 1;
            } else {
                accumulator.counts.covered_negative_cases += 1;
            }
        }
        CoverageState::NotTasked { .. } => {
            if is_positive {
                accumulator.counts.not_tasked_positive_cases += 1;
            } else {
                accumulator.counts.not_tasked_negative_cases += 1;
            }
        }
        CoverageState::InvalidScene { .. } => {
            if is_positive {
                accumulator.counts.invalid_scene_positive_cases += 1;
            } else {
                accumulator.counts.invalid_scene_negative_cases += 1;
            }
        }
    }

    match (&case.reference, &response.coverage, &response.disposition) {
        (
            ReferenceState::MeteredEmission {
                emission_rate_kg_h: reference_rate,
            },
            CoverageState::Covered,
            SystemDisposition::Detected {
                emission_rate_kg_h: predicted_rate,
            },
        ) => {
            accumulator.counts.true_positives += 1;
            accumulator
                .absolute_log10_rate_errors
                .push((predicted_rate.log10() - reference_rate.log10()).abs());
        }
        (ReferenceState::MeteredEmission { .. }, _, _) => {
            // Every known positive not detected above is a false negative.
            accumulator.counts.false_negatives += 1;
            if matches!(&response.disposition, SystemDisposition::Abstained { .. }) {
                accumulator.counts.abstained_positive_cases += 1;
            }
        }
        (
            ReferenceState::MeteredZero,
            CoverageState::Covered,
            SystemDisposition::Detected { .. },
        ) => accumulator.counts.false_positives += 1,
        (
            ReferenceState::MeteredZero,
            CoverageState::Covered,
            SystemDisposition::ExplicitNonDetect,
        ) => accumulator.counts.true_negatives += 1,
        (
            ReferenceState::MeteredZero,
            CoverageState::Covered,
            SystemDisposition::Abstained { .. },
        ) => accumulator.counts.abstained_negative_cases += 1,
        (ReferenceState::MeteredZero, _, _) => {}
        (ReferenceState::ReferenceUnavailable { .. }, _, _) => {}
    }
}

fn validate_response_shape(report: &mut EvaluationReport, response: &CaseResponse) {
    match &response.coverage {
        CoverageState::Covered => {}
        CoverageState::NotTasked { reason } if reason.trim().is_empty() => {
            report.push(&response.case_id, "empty_not_tasked_reason");
        }
        CoverageState::InvalidScene { reason } if reason.trim().is_empty() => {
            report.push(&response.case_id, "empty_invalid_scene_reason");
        }
        CoverageState::NotTasked { .. } | CoverageState::InvalidScene { .. } => {}
    }

    match &response.disposition {
        SystemDisposition::Detected { emission_rate_kg_h }
            if !emission_rate_kg_h.is_finite() || *emission_rate_kg_h <= 0.0 =>
        {
            report.push(&response.case_id, "invalid_reported_emission_rate");
        }
        SystemDisposition::Abstained { reason } if reason.trim().is_empty() => {
            report.push(&response.case_id, "empty_abstention_reason");
        }
        SystemDisposition::NoOutput if response.coverage == CoverageState::Covered => {
            report.push(&response.case_id, "covered_case_has_no_output");
        }
        SystemDisposition::Detected { .. }
        | SystemDisposition::ExplicitNonDetect
        | SystemDisposition::Abstained { .. }
            if response.coverage != CoverageState::Covered =>
        {
            report.push(&response.case_id, "output_without_coverage");
        }
        _ => {}
    }
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Two-sided 95% Wilson score interval for a binomial proportion.
///
/// This is a descriptive per-system interval. Comparing systems still requires a
/// paired or cluster-aware design when they share sites, scenes, or time windows.
fn wilson_interval_95(successes: usize, trials: usize) -> Option<(f64, f64)> {
    if trials == 0 || successes > trials {
        return None;
    }
    const Z: f64 = 1.959_963_984_540_054;
    let n = trials as f64;
    let p = successes as f64 / n;
    let z_squared = Z * Z;
    let denominator = 1.0 + z_squared / n;
    let center = (p + z_squared / (2.0 * n)) / denominator;
    let half_width =
        Z * ((p * (1.0 - p) / n + z_squared / (4.0 * n * n)).sqrt()) / denominator;
    Some(((center - half_width).max(0.0), (center + half_width).min(1.0)))
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn median_sorted(values: &[f64]) -> Option<f64> {
    match values.len() {
        0 => None,
        len if len % 2 == 1 => Some(values[len / 2]),
        len => Some((values[len / 2 - 1] + values[len / 2]) / 2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cases() -> Vec<BenchmarkCase> {
        vec![
            BenchmarkCase {
                case_id: "p-detected".into(),
                stratum_id: "low-rate".into(),
                reference: ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 10.0,
                },
            },
            BenchmarkCase {
                case_id: "p-missed".into(),
                stratum_id: "low-rate".into(),
                reference: ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 5.0,
                },
            },
            BenchmarkCase {
                case_id: "n-false-alarm".into(),
                stratum_id: "high-background".into(),
                reference: ReferenceState::MeteredZero,
            },
            BenchmarkCase {
                case_id: "n-correct".into(),
                stratum_id: "high-background".into(),
                reference: ReferenceState::MeteredZero,
            },
            BenchmarkCase {
                case_id: "p-not-tasked".into(),
                stratum_id: "high-background".into(),
                reference: ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 7.0,
                },
            },
            BenchmarkCase {
                case_id: "n-invalid-scene".into(),
                stratum_id: "low-rate".into(),
                reference: ReferenceState::MeteredZero,
            },
            BenchmarkCase {
                case_id: "truth-unavailable".into(),
                stratum_id: "unscorable".into(),
                reference: ReferenceState::ReferenceUnavailable {
                    reason: "reference meter fault".into(),
                },
            },
        ]
    }

    fn response(
        case_id: &str,
        coverage: CoverageState,
        disposition: SystemDisposition,
    ) -> CaseResponse {
        CaseResponse {
            case_id: case_id.into(),
            coverage,
            disposition,
        }
    }

    fn submission(responses: Vec<CaseResponse>) -> SystemSubmission {
        SystemSubmission {
            system_id: "baseline-detector".into(),
            version: "1.0.0".into(),
            build_sha256: "a".repeat(64),
            configuration_sha256: "b".repeat(64),
            responses,
        }
    }

    fn valid_responses() -> Vec<CaseResponse> {
        vec![
            response(
                "p-detected",
                CoverageState::Covered,
                SystemDisposition::Detected {
                    emission_rate_kg_h: 9.0,
                },
            ),
            response(
                "p-missed",
                CoverageState::Covered,
                SystemDisposition::ExplicitNonDetect,
            ),
            response(
                "n-false-alarm",
                CoverageState::Covered,
                SystemDisposition::Detected {
                    emission_rate_kg_h: 1.2,
                },
            ),
            response(
                "n-correct",
                CoverageState::Covered,
                SystemDisposition::ExplicitNonDetect,
            ),
            response(
                "p-not-tasked",
                CoverageState::NotTasked {
                    reason: "satellite not tasked".into(),
                },
                SystemDisposition::NoOutput,
            ),
            response(
                "n-invalid-scene",
                CoverageState::InvalidScene {
                    reason: "cloud obstruction".into(),
                },
                SystemDisposition::NoOutput,
            ),
            response(
                "truth-unavailable",
                CoverageState::Covered,
                SystemDisposition::Abstained {
                    reason: "no valid reference label".into(),
                },
            ),
        ]
    }

    #[test]
    fn scores_complete_census_and_keeps_no_coverage_separate_from_true_negative() {
        let report = evaluate_census(&cases(), &submission(valid_responses()));
        assert!(report.is_valid(), "{report:?}");
        let metrics = report.metrics.unwrap();
        let overall = metrics.overall;
        assert_eq!(overall.true_positives, 1);
        assert_eq!(overall.false_negatives, 2);
        assert_eq!(overall.false_positives, 1);
        assert_eq!(overall.true_negatives, 1);
        assert_eq!(overall.not_tasked_positive_cases, 1);
        assert_eq!(overall.invalid_scene_negative_cases, 1);
        assert_eq!(overall.reference_unavailable_cases, 1);
        assert_eq!(overall.negative_reference_cases, 3);
        assert_eq!(overall.covered_negative_cases, 2);
        assert!((overall.end_to_end_recall().unwrap() - (1.0 / 3.0)).abs() < 1e-12);
        assert!((overall.recall_when_covered().unwrap() - 0.5).abs() < 1e-12);
        assert!((overall.false_alert_rate_when_covered().unwrap() - 0.5).abs() < 1e-12);
        assert!((overall.coverage_rate().unwrap() - (4.0 / 6.0)).abs() < 1e-12);
        assert_eq!(metrics.quantification_sample_count, 1);
        let expected_log_error = (9.0_f64.log10() - 10.0_f64.log10()).abs();
        assert!(
            (metrics.median_absolute_log10_rate_error.unwrap() - expected_log_error).abs() < 1e-12
        );
        assert!(metrics.by_stratum.contains_key("low-rate"));
        assert!(metrics.by_stratum.contains_key("high-background"));
    }

    #[test]
    fn wilson_intervals_include_estimates_and_expose_small_sample_uncertainty() {
        let counts = DetectionCounts {
            true_positives: 1,
            positive_reference_cases: 3,
            covered_positive_cases: 2,
            false_positives: 1,
            covered_negative_cases: 2,
            covered_known_cases: 4,
            reference_known_cases: 6,
            ..DetectionCounts::default()
        };
        let recall_interval = counts.end_to_end_recall_ci95().unwrap();
        let recall = counts.end_to_end_recall().unwrap();
        assert!(recall_interval.0 < recall && recall < recall_interval.1);
        assert!(recall_interval.1 - recall_interval.0 > 0.5);

        let false_alert_interval = counts.false_alert_rate_when_covered_ci95().unwrap();
        assert!(false_alert_interval.0 < 0.5 && false_alert_interval.1 > 0.5);
        let coverage_interval = counts.coverage_rate_ci95().unwrap();
        assert!(coverage_interval.0 < (4.0 / 6.0) && coverage_interval.1 > (4.0 / 6.0));
    }

    #[test]
    fn wilson_intervals_are_absent_without_denominators() {
        let counts = DetectionCounts::default();
        assert_eq!(counts.end_to_end_recall_ci95(), None);
        assert_eq!(counts.false_alert_rate_when_covered_ci95(), None);
        assert_eq!(counts.coverage_rate_ci95(), None);
    }

    #[test]
    fn missing_response_suppresses_all_headline_metrics() {
        let mut responses = valid_responses();
        responses.pop();
        let report = evaluate_census(&cases(), &submission(responses));
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "missing_response_row")
        );
    }

    #[test]
    fn duplicate_or_extra_response_invalidates_census() {
        let mut responses = valid_responses();
        responses.push(responses[0].clone());
        responses.push(response(
            "unplanned-case",
            CoverageState::Covered,
            SystemDisposition::ExplicitNonDetect,
        ));
        let report = evaluate_census(&cases(), &submission(responses));
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "duplicate_response_case_id")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "response_for_unknown_case")
        );
    }

    #[test]
    fn no_coverage_detection_is_rejected_instead_of_getting_credit() {
        let mut responses = valid_responses();
        let row = responses
            .iter_mut()
            .find(|r| r.case_id == "p-not-tasked")
            .unwrap();
        row.disposition = SystemDisposition::Detected {
            emission_rate_kg_h: 7.0,
        };
        let report = evaluate_census(&cases(), &submission(responses));
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "output_without_coverage")
        );
    }

    #[test]
    fn covered_case_without_explicit_output_is_rejected() {
        let mut responses = valid_responses();
        let row = responses
            .iter_mut()
            .find(|r| r.case_id == "p-detected")
            .unwrap();
        row.disposition = SystemDisposition::NoOutput;
        let report = evaluate_census(&cases(), &submission(responses));
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "covered_case_has_no_output")
        );
    }

    #[test]
    fn abstentions_count_as_misses_not_silent_exclusions() {
        let mut responses = valid_responses();
        let row = responses
            .iter_mut()
            .find(|r| r.case_id == "p-missed")
            .unwrap();
        row.disposition = SystemDisposition::Abstained {
            reason: "system uncertainty limit".into(),
        };
        let metrics = evaluate_census(&cases(), &submission(responses))
            .metrics
            .unwrap();
        assert_eq!(metrics.overall.false_negatives, 2);
        assert_eq!(metrics.overall.abstained_positive_cases, 1);
    }

    #[test]
    fn unscorable_reference_is_reported_but_never_counted_as_true_negative() {
        let report = evaluate_census(&cases(), &submission(valid_responses()));
        let metrics = report.metrics.unwrap();
        assert_eq!(metrics.overall.reference_unavailable_cases, 1);
        assert_eq!(metrics.overall.true_negatives, 1);
        assert_eq!(metrics.overall.negative_reference_cases, 3);
    }

    #[test]
    fn invalid_build_digest_and_invalid_detected_rate_reject_submission() {
        let mut sub = submission(valid_responses());
        sub.build_sha256 = "UPPERCASE".into();
        let row = sub
            .responses
            .iter_mut()
            .find(|r| r.case_id == "p-detected")
            .unwrap();
        row.disposition = SystemDisposition::Detected {
            emission_rate_kg_h: f64::NAN,
        };
        let report = evaluate_census(&cases(), &sub);
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_build_sha256")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_reported_emission_rate")
        );
    }

    #[test]
    fn duplicate_reference_ids_and_empty_case_universe_fail_closed() {
        let mut duplicate = cases();
        duplicate.push(duplicate[0].clone());
        let report = evaluate_census(&duplicate, &submission(valid_responses()));
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "duplicate_benchmark_case_id")
        );

        let report = evaluate_census(&[], &submission(Vec::new()));
        assert!(report.metrics.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "empty_case_universe")
        );
    }
}
