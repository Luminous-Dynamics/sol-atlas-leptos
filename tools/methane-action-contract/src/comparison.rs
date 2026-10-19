//! Paired, cluster-aware candidate-versus-baseline comparison.
//!
//! Both submissions are scored against the same complete case census. Uncertainty
//! intervals resample predeclared clusters (e.g. site × overpass), not individual
//! rows, preserving within-cluster dependence. This is not a detector, and the
//! intervals are descriptive bootstrap intervals, not regulatory certification.

use crate::census::{
    BenchmarkCase, EvaluationReport, ReferenceState, SystemDisposition,
    SystemSubmission, evaluate_census,
};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComparisonViolation {
    pub system_id: String,
    pub record_id: String,
    pub code: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PairedRateComparison {
    pub baseline_rate: Option<f64>,
    pub candidate_rate: Option<f64>,
    /// Candidate minus baseline; negative is favorable for false-alert rate.
    pub candidate_minus_baseline: Option<f64>,
    /// Percentile interval for the candidate-minus-baseline difference, resampling
    /// whole site/scene clusters. Absent if too few usable replicates remain.
    pub cluster_bootstrap_ci95: Option<(f64, f64)>,
    pub denominator: usize,
    /// Discordant cases where the baseline's event occurred but the candidate's did not.
    /// For recall, an event is a true detection; for false-alert rate, it is a false alert.
    pub baseline_only_events: usize,
    /// Discordant cases where the candidate's event occurred but the baseline's did not.
    pub candidate_only_events: usize,
    pub bootstrap_replicates_used: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PairedComparisonReport {
    pub valid: bool,
    pub baseline_system_id: String,
    pub candidate_system_id: String,
    pub cluster_count: usize,
    pub bootstrap_replicates_requested: usize,
    pub seed: u64,
    pub recall: PairedRateComparison,
    pub false_alert_rate: PairedRateComparison,
    pub violations: Vec<ComparisonViolation>,
}

impl PairedComparisonReport {
    pub fn is_valid(&self) -> bool {
        self.valid && self.violations.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ClusterCounts {
    positive_cases: usize,
    baseline_true_positives: usize,
    candidate_true_positives: usize,
    negative_cases: usize,
    baseline_false_positives: usize,
    candidate_false_positives: usize,
}

impl ClusterCounts {
    fn add(&mut self, other: Self) {
        self.positive_cases += other.positive_cases;
        self.baseline_true_positives += other.baseline_true_positives;
        self.candidate_true_positives += other.candidate_true_positives;
        self.negative_cases += other.negative_cases;
        self.baseline_false_positives += other.baseline_false_positives;
        self.candidate_false_positives += other.candidate_false_positives;
    }
}

pub fn compare_submissions(
    cases: &[BenchmarkCase],
    baseline: &SystemSubmission,
    candidate: &SystemSubmission,
    bootstrap_replicates: usize,
    seed: u64,
) -> PairedComparisonReport {
    let mut report = PairedComparisonReport {
        baseline_system_id: baseline.system_id.clone(),
        candidate_system_id: candidate.system_id.clone(),
        bootstrap_replicates_requested: bootstrap_replicates,
        seed,
        ..PairedComparisonReport::default()
    };

    if bootstrap_replicates == 0 {
        report.violations.push(ComparisonViolation {
            system_id: "__comparison__".into(),
            record_id: "__bootstrap__".into(),
            code: "zero_bootstrap_replicates",
        });
        return report;
    }

    let baseline_evaluation = evaluate_census(cases, baseline);
    let candidate_evaluation = evaluate_census(cases, candidate);
    copy_evaluation_violations(&mut report, &baseline_evaluation, &baseline.system_id);
    copy_evaluation_violations(&mut report, &candidate_evaluation, &candidate.system_id);
    if !report.violations.is_empty() {
        report.violations.sort_by(|a, b| {
            (&a.system_id, &a.record_id, a.code).cmp(&(&b.system_id, &b.record_id, b.code))
        });
        return report;
    }

    let baseline_responses: HashMap<&str, &crate::census::CaseResponse> = baseline
        .responses
        .iter()
        .map(|response| (response.case_id.as_str(), response))
        .collect();
    let candidate_responses: HashMap<&str, &crate::census::CaseResponse> = candidate
        .responses
        .iter()
        .map(|response| (response.case_id.as_str(), response))
        .collect();

    let mut clusters: BTreeMap<String, ClusterCounts> = BTreeMap::new();
    let mut recall_baseline_only = 0;
    let mut recall_candidate_only = 0;
    let mut alert_baseline_only = 0;
    let mut alert_candidate_only = 0;

    for case in cases {
        let baseline_response = baseline_responses[case.case_id.as_str()];
        let candidate_response = candidate_responses[case.case_id.as_str()];
        let baseline_detected = matches!(
            &baseline_response.disposition,
            SystemDisposition::Detected { .. }
        );
        let candidate_detected = matches!(
            &candidate_response.disposition,
            SystemDisposition::Detected { .. }
        );
        let cluster = clusters.entry(case.cluster_id.clone()).or_default();

        match &case.reference {
            ReferenceState::MeteredEmission { .. } => {
                cluster.positive_cases += 1;
                cluster.baseline_true_positives += usize::from(u8::from(baseline_detected));
                cluster.candidate_true_positives += usize::from(u8::from(candidate_detected));
                match (baseline_detected, candidate_detected) {
                    (true, false) => recall_baseline_only += 1,
                    (false, true) => recall_candidate_only += 1,
                    _ => {}
                }
            }
            ReferenceState::MeteredZero => {
                cluster.negative_cases += 1;
                cluster.baseline_false_positives += usize::from(baseline_detected);
                cluster.candidate_false_positives += usize::from(candidate_detected);
                match (baseline_detected, candidate_detected) {
                    (true, false) => alert_baseline_only += 1,
                    (false, true) => alert_candidate_only += 1,
                    _ => {}
                }
            }
            ReferenceState::ReferenceUnavailable { .. } => {}
        }
    }

    let cluster_values: Vec<ClusterCounts> = clusters.values().copied().collect();
    report.cluster_count = cluster_values.len();

    let overall = cluster_values.iter().copied().fold(ClusterCounts::default(), |mut a, b| {
        a.add(b);
        a
    });

    report.recall = paired_rate(
        overall.baseline_true_positives,
        overall.candidate_true_positives,
        overall.positive_cases,
        recall_baseline_only,
        recall_candidate_only,
    );
    report.false_alert_rate = paired_rate(
        overall.baseline_false_positives,
        overall.candidate_false_positives,
        overall.negative_cases,
        alert_baseline_only,
        alert_candidate_only,
    );

    if report.cluster_count >= 2 {
        let mut rng = DeterministicRng::new(seed);
        let mut recall_deltas = Vec::with_capacity(bootstrap_replicates);
        let mut alert_deltas = Vec::with_capacity(bootstrap_replicates);

        for _ in 0..bootstrap_replicates {
            let mut sample = ClusterCounts::default();
            for _ in 0..cluster_values.len() {
                let index = (rng.next_u64() % cluster_values.len() as u64) as usize;
                sample.add(cluster_values[index]);
            }
            if sample.positive_cases > 0 {
                let baseline_rate =
                    sample.baseline_true_positives as f64 / sample.positive_cases as f64;
                let candidate_rate =
                    sample.candidate_true_positives as f64 / sample.positive_cases as f64;
                recall_deltas.push(candidate_rate - baseline_rate);
            }
            if sample.negative_cases > 0 {
                let baseline_rate =
                    sample.baseline_false_positives as f64 / sample.negative_cases as f64;
                let candidate_rate =
                    sample.candidate_false_positives as f64 / sample.negative_cases as f64;
                alert_deltas.push(candidate_rate - baseline_rate);
            }
        }
        attach_interval(&mut report.recall, recall_deltas, bootstrap_replicates);
        attach_interval(
            &mut report.false_alert_rate,
            alert_deltas,
            bootstrap_replicates,
        );
    }

    report.valid = true;
    report
}

fn copy_evaluation_violations(
    report: &mut PairedComparisonReport,
    evaluation: &EvaluationReport,
    system_id: &str,
) {
    report
        .violations
        .extend(evaluation.violations.iter().map(|violation| ComparisonViolation {
            system_id: system_id.to_owned(),
            record_id: violation.record_id.clone(),
            code: violation.code,
        }));
}

fn paired_rate(
    baseline_events: usize,
    candidate_events: usize,
    denominator: usize,
    baseline_only_events: usize,
    candidate_only_events: usize,
) -> PairedRateComparison {
    let baseline_rate = ratio(baseline_events, denominator);
    let candidate_rate = ratio(candidate_events, denominator);
    PairedRateComparison {
        baseline_rate,
        candidate_rate,
        candidate_minus_baseline: baseline_rate.zip(candidate_rate).map(|(base, next)| next - base),
        cluster_bootstrap_ci95: None,
        denominator,
        baseline_only_events,
        candidate_only_events,
        bootstrap_replicates_used: 0,
    }
}

fn attach_interval(
    metric: &mut PairedRateComparison,
    mut deltas: Vec<f64>,
    requested_replicates: usize,
) {
    deltas.sort_by(f64::total_cmp);
    metric.bootstrap_replicates_used = deltas.len();
    if deltas.len() >= requested_replicates.saturating_mul(4).div_ceil(5) {
        metric.cluster_bootstrap_ci95 =
            percentile(&deltas, 0.025).zip(percentile(&deltas, 0.975));
    }
}

fn percentile(sorted: &[f64], probability: f64) -> Option<f64> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&probability) {
        return None;
    }
    let index = probability * (sorted.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    if lower == upper {
        Some(sorted[lower])
    } else {
        let fraction = index - lower as f64;
        Some(sorted[lower] * (1.0 - fraction) + sorted[upper] * fraction)
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

/// Small, fixed PRNG to make bootstrap resampling reproducible across runtimes.
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::census::{CaseResponse, CoverageState, SystemDisposition};

    fn cases() -> Vec<BenchmarkCase> {
        vec![
            BenchmarkCase {
                case_id: "p-hit".into(),
                stratum_id: "low-rate".into(),
                cluster_id: "site-a".into(),
                reference: ReferenceState::MeteredEmission { emission_rate_kg_h: 10.0 },
            },
            BenchmarkCase {
                case_id: "p-miss".into(),
                stratum_id: "low-rate".into(),
                cluster_id: "site-b".into(),
                reference: ReferenceState::MeteredEmission { emission_rate_kg_h: 5.0 },
            },
            BenchmarkCase {
                case_id: "n-fp".into(),
                stratum_id: "background".into(),
                cluster_id: "site-a".into(),
                reference: ReferenceState::MeteredZero,
            },
            BenchmarkCase {
                case_id: "n-tn".into(),
                stratum_id: "background".into(),
                cluster_id: "site-b".into(),
                reference: ReferenceState::MeteredZero,
            },
            BenchmarkCase {
                case_id: "p-no-coverage".into(),
                stratum_id: "low-rate".into(),
                cluster_id: "site-c".into(),
                reference: ReferenceState::MeteredEmission { emission_rate_kg_h: 7.0 },
            },
        ]
    }

    fn response(
        case_id: &str,
        disposition: SystemDisposition,
        coverage: CoverageState,
    ) -> CaseResponse {
        CaseResponse { case_id: case_id.into(), disposition, coverage }
    }

    fn submission(id: &str, responses: Vec<CaseResponse>) -> SystemSubmission {
        SystemSubmission {
            system_id: id.into(),
            version: "1.0.0".into(),
            build_sha256: if id == "baseline" { "a".repeat(64) } else { "c".repeat(64) },
            configuration_sha256: if id == "baseline" { "b".repeat(64) } else { "d".repeat(64) },
            responses,
        }
    }

    fn baseline() -> SystemSubmission {
        submission("baseline", vec![
            response("p-hit", SystemDisposition::Detected { emission_rate_kg_h: 9.0 }, CoverageState::Covered),
            response("p-miss", SystemDisposition::ExplicitNonDetect, CoverageState::Covered),
            response("n-fp", SystemDisposition::Detected { emission_rate_kg_h: 1.0 }, CoverageState::Covered),
            response("n-tn", SystemDisposition::ExplicitNonDetect, CoverageState::Covered),
            response("p-no-coverage", SystemDisposition::NoOutput, CoverageState::NotTasked { reason: "not tasked".into() }),
        ])
    }

    fn candidate() -> SystemSubmission {
        submission("candidate", vec![
            response("p-hit", SystemDisposition::Detected { emission_rate_kg_h: 10.1 }, CoverageState::Covered),
            response("p-miss", SystemDisposition::Detected { emission_rate_kg_h: 4.8 }, CoverageState::Covered),
            response("n-fp", SystemDisposition::ExplicitNonDetect, CoverageState::Covered),
            response("n-tn", SystemDisposition::ExplicitNonDetect, CoverageState::Covered),
            response("p-no-coverage", SystemDisposition::NoOutput, CoverageState::NotTasked { reason: "not tasked".into() }),
        ])
    }

    #[test]
    fn computes_paired_delta_and_cluster_bootstrap_intervals() {
        let report = compare_submissions(&cases(), &baseline(), &candidate(), 1_000, 42);
        assert!(report.is_valid(), "{report:?}");
        assert_eq!(report.cluster_count, 3);
        assert_eq!(report.recall.denominator, 3);
        assert_eq!(report.recall.baseline_rate, Some(1.0 / 3.0));
        assert_eq!(report.recall.candidate_rate, Some(2.0 / 3.0));
        assert!((report.recall.candidate_minus_baseline.unwrap() - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(report.recall.baseline_only_events, 0);
        assert_eq!(report.recall.candidate_only_events, 1);
        assert_eq!(report.false_alert_rate.denominator, 2);
        assert_eq!(report.false_alert_rate.baseline_rate, Some(0.5));
        assert_eq!(report.false_alert_rate.candidate_rate, Some(0.0));
        assert!((report.false_alert_rate.candidate_minus_baseline.unwrap() + 0.5).abs() < 1e-12);
        assert_eq!(report.false_alert_rate.baseline_only_events, 1);
        assert_eq!(report.false_alert_rate.candidate_only_events, 0);
        assert!(report.recall.cluster_bootstrap_ci95.is_some());
        assert!(report.false_alert_rate.cluster_bootstrap_ci95.is_some());
        assert_eq!(report.recall.bootstrap_replicates_used, 1_000);
    }

    #[test]
    fn paired_bootstrap_is_reproducible_for_fixed_seed() {
        let first = compare_submissions(&cases(), &baseline(), &candidate(), 1_000, 1234);
        let second = compare_submissions(&cases(), &baseline(), &candidate(), 1_000, 1234);
        assert_eq!(first, second);
    }

    #[test]
    fn invalid_submission_suppresses_paired_metrics() {
        let mut incomplete = candidate();
        incomplete.responses.pop();
        let report = compare_submissions(&cases(), &baseline(), &incomplete, 1_000, 42);
        assert!(!report.is_valid());
        assert!(report.recall.candidate_minus_baseline.is_none());
        assert!(report.violations.iter().any(|v| v.system_id == "candidate" && v.code == "missing_response_row"));
    }

    #[test]
    fn zero_bootstrap_replicates_are_rejected() {
        let report = compare_submissions(&cases(), &baseline(), &candidate(), 0, 42);
        assert!(!report.is_valid());
        assert!(report.violations.iter().any(|v| v.code == "zero_bootstrap_replicates"));
    }
}
