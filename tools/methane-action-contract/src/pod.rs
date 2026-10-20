//! Condition-stratified probability-of-detection (POD) profiles.
//!
//! Reports empirical POD by predeclared release-rate band and condition stratum.
//! Coverage failures and abstentions remain explicit; false-alert rates use only
//! covered metered-zero opportunities. This is descriptive scoring, not a detector.

use crate::census::{
    BenchmarkCase, CaseResponse, CoverageState, EvaluationReport, ReferenceState,
    SystemDisposition, SystemSubmission, evaluate_census, wilson_interval_95,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateBand {
    pub band_id: String,
    pub lower_kg_h: f64,
    /// Exclusive upper bound; absent only for an open-ended final band.
    pub upper_kg_h: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ConditionRateBandResult {
    pub condition_stratum_id: String,
    pub rate_band_id: String,
    pub lower_kg_h: f64,
    pub upper_kg_h: Option<f64>,
    pub positive_opportunities: usize,
    pub detected: usize,
    pub covered_positive_opportunities: usize,
    pub not_tasked_positive: usize,
    pub invalid_scene_positive: usize,
    pub explicit_non_detections: usize,
    pub abstentions: usize,
    pub end_to_end_pod: Option<f64>,
    pub end_to_end_pod_ci95: Option<(f64, f64)>,
    pub pod_when_covered: Option<f64>,
    pub pod_when_covered_ci95: Option<(f64, f64)>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ConditionFalseAlertResult {
    pub condition_stratum_id: String,
    pub metered_zero_opportunities: usize,
    pub covered_zero_opportunities: usize,
    pub false_alerts: usize,
    pub true_non_detections: usize,
    pub not_tasked_zero: usize,
    pub invalid_scene_zero: usize,
    pub false_alert_rate_when_covered: Option<f64>,
    pub false_alert_rate_ci95: Option<(f64, f64)>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DetectionProfile {
    pub system_id: String,
    pub version: String,
    pub build_sha256: String,
    pub configuration_sha256: String,
    pub case_count: usize,
    pub unscorable_reference_cases: usize,
    pub rate_bands: Vec<RateBand>,
    pub by_condition_and_rate_band: Vec<ConditionRateBandResult>,
    pub false_alerts_by_condition: Vec<ConditionFalseAlertResult>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PodProfileReport {
    pub violations: Vec<crate::census::CensusViolation>,
    pub profile: Option<DetectionProfile>,
}

impl PodProfileReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty() && self.profile.is_some()
    }

    fn push(&mut self, record_id: &str, code: &'static str) {
        self.violations.push(crate::census::CensusViolation {
            record_id: record_id.to_owned(),
            code,
        });
    }
}

#[derive(Clone, Debug, Default)]
struct PodAccumulator {
    positives: usize,
    detections: usize,
    covered_positives: usize,
    not_tasked: usize,
    invalid_scene: usize,
    non_detections: usize,
    abstentions: usize,
}

#[derive(Clone, Debug, Default)]
struct AlertAccumulator {
    zeros: usize,
    covered_zeros: usize,
    false_alerts: usize,
    true_non_detections: usize,
    not_tasked: usize,
    invalid_scene: usize,
}

pub fn build_pod_profile(
    cases: &[BenchmarkCase],
    submission: &SystemSubmission,
    rate_bands: &[RateBand],
) -> PodProfileReport {
    let mut report = PodProfileReport::default();
    validate_rate_bands(&mut report, rate_bands);
    let census: EvaluationReport = evaluate_census(cases, submission);
    report.violations.extend(census.violations.iter().cloned());

    if !report.violations.is_empty() {
        sort_violations(&mut report);
        return report;
    }

    let responses: HashMap<&str, &CaseResponse> = submission
        .responses
        .iter()
        .map(|response| (response.case_id.as_str(), response))
        .collect();
    let conditions: BTreeSet<&str> = cases.iter().map(|case| case.stratum_id.as_str()).collect();
    let mut pod: BTreeMap<(String, String), PodAccumulator> = BTreeMap::new();
    let mut alerts: BTreeMap<String, AlertAccumulator> = BTreeMap::new();

    for condition in &conditions {
        alerts.insert((*condition).to_owned(), AlertAccumulator::default());
        for band in rate_bands {
            pod.insert(
                ((*condition).to_owned(), band.band_id.clone()),
                PodAccumulator::default(),
            );
        }
    }

    let mut unscorable = 0;
    for case in cases {
        let response = responses[case.case_id.as_str()];
        match &case.reference {
            ReferenceState::MeteredEmission { emission_rate_kg_h } => {
                let matching = rate_bands
                    .iter()
                    .filter(|band| {
                        *emission_rate_kg_h >= band.lower_kg_h
                            && band
                                .upper_kg_h
                                .is_none_or(|upper| *emission_rate_kg_h < upper)
                    })
                    .collect::<Vec<_>>();
                if matching.len() != 1 {
                    report.push(
                        &case.case_id,
                        "positive_case_outside_or_duplicated_rate_band",
                    );
                    continue;
                }
                let accumulator = pod
                    .get_mut(&(case.stratum_id.clone(), matching[0].band_id.clone()))
                    .expect("all condition/rate-band cells were pre-created");
                accumulator.positives += 1;
                match &response.coverage {
                    CoverageState::Covered => {
                        accumulator.covered_positives += 1;
                        match &response.disposition {
                            SystemDisposition::Detected { .. } => accumulator.detections += 1,
                            SystemDisposition::ExplicitNonDetect => accumulator.non_detections += 1,
                            SystemDisposition::Abstained { .. } => accumulator.abstentions += 1,
                            SystemDisposition::NoOutput => {
                                report.push(&case.case_id, "covered_case_has_no_output");
                            }
                        }
                    }
                    CoverageState::NotTasked { .. } => accumulator.not_tasked += 1,
                    CoverageState::InvalidScene { .. } => accumulator.invalid_scene += 1,
                }
            }
            ReferenceState::MeteredZero => {
                let accumulator = alerts
                    .get_mut(&case.stratum_id)
                    .expect("condition bucket was pre-created");
                accumulator.zeros += 1;
                match (&response.coverage, &response.disposition) {
                    (CoverageState::Covered, SystemDisposition::Detected { .. }) => {
                        accumulator.covered_zeros += 1;
                        accumulator.false_alerts += 1;
                    }
                    (CoverageState::Covered, SystemDisposition::ExplicitNonDetect) => {
                        accumulator.covered_zeros += 1;
                        accumulator.true_non_detections += 1;
                    }
                    (CoverageState::Covered, SystemDisposition::Abstained { .. }) => {
                        accumulator.covered_zeros += 1;
                    }
                    (CoverageState::NotTasked { .. }, _) => accumulator.not_tasked += 1,
                    (CoverageState::InvalidScene { .. }, _) => accumulator.invalid_scene += 1,
                    (CoverageState::Covered, SystemDisposition::NoOutput) => {
                        report.push(&case.case_id, "covered_case_has_no_output");
                    }
                    (_, SystemDisposition::Detected { .. })
                    | (_, SystemDisposition::ExplicitNonDetect)
                    | (_, SystemDisposition::Abstained { .. }) => {
                        report.push(&case.case_id, "output_without_coverage");
                    }
                }
            }
            ReferenceState::ReferenceUnavailable { .. } => unscorable += 1,
        }
    }

    if !report.violations.is_empty() {
        sort_violations(&mut report);
        return report;
    }

    let by_condition_and_rate_band = pod
        .into_iter()
        .map(|((condition, band_id), accumulator)| {
            let band = rate_bands
                .iter()
                .find(|candidate| candidate.band_id == band_id)
                .expect("validated rate band is present");
            ConditionRateBandResult {
                condition_stratum_id: condition,
                rate_band_id: band.band_id.clone(),
                lower_kg_h: band.lower_kg_h,
                upper_kg_h: band.upper_kg_h,
                positive_opportunities: accumulator.positives,
                detected: accumulator.detections,
                covered_positive_opportunities: accumulator.covered_positives,
                not_tasked_positive: accumulator.not_tasked,
                invalid_scene_positive: accumulator.invalid_scene,
                explicit_non_detections: accumulator.non_detections,
                abstentions: accumulator.abstentions,
                end_to_end_pod: ratio(accumulator.detections, accumulator.positives),
                end_to_end_pod_ci95: wilson_interval_95(
                    accumulator.detections,
                    accumulator.positives,
                ),
                pod_when_covered: ratio(accumulator.detections, accumulator.covered_positives),
                pod_when_covered_ci95: wilson_interval_95(
                    accumulator.detections,
                    accumulator.covered_positives,
                ),
            }
        })
        .collect();

    let false_alerts_by_condition = alerts
        .into_iter()
        .map(|(condition, accumulator)| ConditionFalseAlertResult {
            condition_stratum_id: condition,
            metered_zero_opportunities: accumulator.zeros,
            covered_zero_opportunities: accumulator.covered_zeros,
            false_alerts: accumulator.false_alerts,
            true_non_detections: accumulator.true_non_detections,
            not_tasked_zero: accumulator.not_tasked,
            invalid_scene_zero: accumulator.invalid_scene,
            false_alert_rate_when_covered: ratio(
                accumulator.false_alerts,
                accumulator.covered_zeros,
            ),
            false_alert_rate_ci95: wilson_interval_95(
                accumulator.false_alerts,
                accumulator.covered_zeros,
            ),
        })
        .collect();

    report.profile = Some(DetectionProfile {
        system_id: submission.system_id.clone(),
        version: submission.version.clone(),
        build_sha256: submission.build_sha256.clone(),
        configuration_sha256: submission.configuration_sha256.clone(),
        case_count: cases.len(),
        unscorable_reference_cases: unscorable,
        rate_bands: rate_bands.to_vec(),
        by_condition_and_rate_band,
        false_alerts_by_condition,
    });
    report
}

fn validate_rate_bands(report: &mut PodProfileReport, bands: &[RateBand]) {
    if bands.is_empty() {
        report.push("__rate_bands__", "empty_rate_bands");
        return;
    }
    let mut ids = BTreeSet::new();
    let mut previous: Option<&RateBand> = None;
    for (index, band) in bands.iter().enumerate() {
        if band.band_id.trim().is_empty() {
            report.push("__rate_bands__", "empty_rate_band_id");
        }
        if !ids.insert(band.band_id.as_str()) {
            report.push(&band.band_id, "duplicate_rate_band_id");
        }
        if !band.lower_kg_h.is_finite() || band.lower_kg_h < 0.0 {
            report.push(&band.band_id, "invalid_rate_band_lower_bound");
        }
        if let Some(upper) = band.upper_kg_h {
            if !upper.is_finite() || upper <= band.lower_kg_h {
                report.push(&band.band_id, "invalid_rate_band_upper_bound");
            }
        }
        if band.upper_kg_h.is_none() && index + 1 != bands.len() {
            report.push(&band.band_id, "unbounded_rate_band_not_last");
        }
        if let Some(previous_band) = previous {
            if band.lower_kg_h <= previous_band.lower_kg_h {
                report.push(&band.band_id, "rate_bands_not_strictly_ordered");
            }
            if previous_band
                .upper_kg_h
                .is_none_or(|upper| band.lower_kg_h < upper)
            {
                report.push(&band.band_id, "overlapping_rate_bands");
            }
        }
        previous = Some(band);
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn sort_violations(report: &mut PodProfileReport) {
    report
        .violations
        .sort_by(|left, right| (&left.record_id, left.code).cmp(&(&right.record_id, right.code)));
    report.violations.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bands() -> Vec<RateBand> {
        vec![
            RateBand {
                band_id: "low".into(),
                lower_kg_h: 0.0,
                upper_kg_h: Some(10.0),
            },
            RateBand {
                band_id: "mid".into(),
                lower_kg_h: 10.0,
                upper_kg_h: Some(50.0),
            },
            RateBand {
                band_id: "high".into(),
                lower_kg_h: 50.0,
                upper_kg_h: None,
            },
        ]
    }

    fn cases() -> Vec<BenchmarkCase> {
        vec![
            case(
                "positive-low-detected",
                "low-wind",
                "site-a",
                ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 5.0,
                },
            ),
            case(
                "positive-mid-nondetect",
                "low-wind",
                "site-b",
                ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 25.0,
                },
            ),
            case(
                "positive-high-detected",
                "high-wind",
                "site-c",
                ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 75.0,
                },
            ),
            case(
                "positive-low-not-tasked",
                "high-wind",
                "site-d",
                ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 7.0,
                },
            ),
            case(
                "positive-mid-abstained",
                "low-wind",
                "site-e",
                ReferenceState::MeteredEmission {
                    emission_rate_kg_h: 30.0,
                },
            ),
            case(
                "negative-false-alert",
                "low-wind",
                "site-f",
                ReferenceState::MeteredZero,
            ),
            case(
                "negative-true-nondetect",
                "low-wind",
                "site-g",
                ReferenceState::MeteredZero,
            ),
            case(
                "negative-not-tasked",
                "high-wind",
                "site-h",
                ReferenceState::MeteredZero,
            ),
            case(
                "truth-unavailable",
                "low-wind",
                "site-i",
                ReferenceState::ReferenceUnavailable {
                    reason: "reference meter unavailable".into(),
                },
            ),
        ]
    }

    fn case(
        case_id: &str,
        stratum_id: &str,
        cluster_id: &str,
        reference: ReferenceState,
    ) -> BenchmarkCase {
        BenchmarkCase {
            case_id: case_id.into(),
            stratum_id: stratum_id.into(),
            cluster_id: cluster_id.into(),
            reference,
        }
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
            system_id: "condition-aware-example".into(),
            version: "0.0.1-synthetic".into(),
            build_sha256: "a".repeat(64),
            configuration_sha256: "b".repeat(64),
            responses,
        }
    }

    fn responses() -> Vec<CaseResponse> {
        vec![
            response(
                "positive-low-detected",
                CoverageState::Covered,
                SystemDisposition::Detected {
                    emission_rate_kg_h: 4.7,
                },
            ),
            response(
                "positive-mid-nondetect",
                CoverageState::Covered,
                SystemDisposition::ExplicitNonDetect,
            ),
            response(
                "positive-high-detected",
                CoverageState::Covered,
                SystemDisposition::Detected {
                    emission_rate_kg_h: 72.0,
                },
            ),
            response(
                "positive-low-not-tasked",
                CoverageState::NotTasked {
                    reason: "no collection opportunity".into(),
                },
                SystemDisposition::NoOutput,
            ),
            response(
                "positive-mid-abstained",
                CoverageState::Covered,
                SystemDisposition::Abstained {
                    reason: "quality below model threshold".into(),
                },
            ),
            response(
                "negative-false-alert",
                CoverageState::Covered,
                SystemDisposition::Detected {
                    emission_rate_kg_h: 1.2,
                },
            ),
            response(
                "negative-true-nondetect",
                CoverageState::Covered,
                SystemDisposition::ExplicitNonDetect,
            ),
            response(
                "negative-not-tasked",
                CoverageState::NotTasked {
                    reason: "platform unavailable".into(),
                },
                SystemDisposition::NoOutput,
            ),
            response(
                "truth-unavailable",
                CoverageState::Covered,
                SystemDisposition::Abstained {
                    reason: "no reference label".into(),
                },
            ),
        ]
    }

    fn cell<'a>(
        profile: &'a DetectionProfile,
        condition: &str,
        band: &str,
    ) -> &'a ConditionRateBandResult {
        profile
            .by_condition_and_rate_band
            .iter()
            .find(|cell| cell.condition_stratum_id == condition && cell.rate_band_id == band)
            .expect("condition/rate-band cross product contains requested bucket")
    }

    #[test]
    fn builds_condition_and_rate_dependent_pod_curve_without_hiding_no_coverage() {
        let report = build_pod_profile(&cases(), &submission(responses()), &bands());
        assert!(report.is_valid(), "{report:?}");
        let profile = report.profile.unwrap();
        assert_eq!(profile.case_count, 9);
        assert_eq!(profile.unscorable_reference_cases, 1);

        let low = cell(&profile, "low-wind", "low");
        assert_eq!(low.positive_opportunities, 1);
        assert_eq!(low.detected, 1);
        assert_eq!(low.end_to_end_pod, Some(1.0));
        assert_eq!(low.pod_when_covered, Some(1.0));
        assert!(low.end_to_end_pod_ci95.is_some());

        let missed = cell(&profile, "high-wind", "low");
        assert_eq!(missed.positive_opportunities, 1);
        assert_eq!(missed.detected, 0);
        assert_eq!(missed.not_tasked_positive, 1);
        assert_eq!(missed.end_to_end_pod, Some(0.0));
        assert_eq!(missed.pod_when_covered, None);

        let mid = cell(&profile, "low-wind", "mid");
        assert_eq!(mid.positive_opportunities, 2);
        assert_eq!(mid.covered_positive_opportunities, 2);
        assert_eq!(mid.explicit_non_detections, 1);
        assert_eq!(mid.abstentions, 1);
        assert_eq!(mid.end_to_end_pod, Some(0.0));
        assert_eq!(mid.pod_when_covered, Some(0.0));
        assert_eq!(cell(&profile, "high-wind", "mid").end_to_end_pod, None);
    }

    #[test]
    fn false_alert_rate_uses_only_covered_zero_release_opportunities() {
        let profile = build_pod_profile(&cases(), &submission(responses()), &bands())
            .profile
            .unwrap();
        let low = profile
            .false_alerts_by_condition
            .iter()
            .find(|row| row.condition_stratum_id == "low-wind")
            .unwrap();
        assert_eq!(low.metered_zero_opportunities, 2);
        assert_eq!(low.covered_zero_opportunities, 2);
        assert_eq!(low.false_alerts, 1);
        assert_eq!(low.false_alert_rate_when_covered, Some(0.5));
        let high = profile
            .false_alerts_by_condition
            .iter()
            .find(|row| row.condition_stratum_id == "high-wind")
            .unwrap();
        assert_eq!(high.metered_zero_opportunities, 1);
        assert_eq!(high.covered_zero_opportunities, 0);
        assert_eq!(high.not_tasked_zero, 1);
        assert_eq!(high.false_alert_rate_when_covered, None);
    }

    #[test]
    fn rejects_overlapping_or_unordered_rate_bands() {
        let mut invalid = bands();
        invalid[1].lower_kg_h = 5.0;
        let report = build_pod_profile(&cases(), &submission(responses()), &invalid);
        assert!(!report.is_valid());
        assert!(report.profile.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "overlapping_rate_bands")
        );
        let mut reversed = bands();
        reversed.swap(0, 1);
        let report = build_pod_profile(&cases(), &submission(responses()), &reversed);
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "rate_bands_not_strictly_ordered")
        );
    }

    #[test]
    fn rejects_positive_reference_outside_configured_bands() {
        let mut out_of_band = cases();
        out_of_band[0].reference = ReferenceState::MeteredEmission {
            emission_rate_kg_h: 1000.0,
        };
        let report = build_pod_profile(&out_of_band, &submission(responses()), &bands());
        assert!(!report.is_valid());
        assert!(report.profile.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "positive_case_outside_or_duplicated_rate_band")
        );
    }

    #[test]
    fn invalid_complete_census_suppresses_pod_profile() {
        let mut incomplete = responses();
        incomplete.pop();
        let report = build_pod_profile(&cases(), &submission(incomplete), &bands());
        assert!(!report.is_valid());
        assert!(report.profile.is_none());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "missing_response_row")
        );
    }

    #[test]
    fn rejects_empty_or_malformed_bands() {
        let empty = build_pod_profile(&cases(), &submission(responses()), &[]);
        assert!(!empty.is_valid());
        assert!(
            empty
                .violations
                .iter()
                .any(|v| v.code == "empty_rate_bands")
        );
        let mut invalid = bands();
        invalid[0].upper_kg_h = Some(f64::NAN);
        let report = build_pod_profile(&cases(), &submission(responses()), &invalid);
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_rate_band_upper_bound")
        );
    }

    #[test]
    fn upper_bound_is_exclusive_and_lower_bound_inclusive() {
        let mut boundary_cases = cases();
        boundary_cases[0].reference = ReferenceState::MeteredEmission {
            emission_rate_kg_h: 10.0,
        };
        let report = build_pod_profile(&boundary_cases, &submission(responses()), &bands());
        assert!(report.is_valid(), "{report:?}");
        let profile = report.profile.unwrap();
        assert_eq!(cell(&profile, "low-wind", "low").positive_opportunities, 0);
        assert_eq!(cell(&profile, "low-wind", "mid").positive_opportunities, 3);
    }
}
