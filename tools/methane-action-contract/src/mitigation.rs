//! Contract for before/after methane-rate evidence after an intervention.
//!
//! Produces conservative rate-reduction bounds, not annualized tonnes avoided or causal
//! proof. It requires qualified in-situ readings, reviewed source links, exact-byte
//! work-order/verification evidence, and repeated post-intervention measurements.
//! Quantification uncertainty is interpreted as a symmetric absolute bound, not 1-sigma.

use crate::{
    ArtifactRole, ArtifactSnapshot, AssociationReview, DataUse, GasSpecies, Observation,
    ObservationKind, Quantification, SourceAssociation, SourceId, ValidationReport,
    is_utc_timestamp, sha256_hex, validate_batch,
};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairVerificationStatus {
    Verified,
    NotVerified,
    Disputed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterventionRecord {
    pub intervention_id: String,
    pub source_id: SourceId,
    pub completed_at_utc: String,
    pub method: String,
    pub status: RepairVerificationStatus,
    /// Submitter-declared identity; this crate does not authenticate the person.
    pub independent_verifier_id: String,
    pub work_order_uri: String,
    pub work_order_sha256: String,
    pub verification_report_uri: String,
    pub verification_report_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MitigationPolicy {
    pub minimum_followup_count: usize,
    pub minimum_followup_span_seconds: u64,
}

pub struct MitigationAssessmentRequest<'data, 'bytes> {
    pub intervention: &'data InterventionRecord,
    pub baseline_observation_id: &'data str,
    pub followup_observation_ids: &'data [String],
    pub observations: &'data [Observation],
    pub associations: &'data [SourceAssociation],
    pub required_use: DataUse,
    pub artifact_snapshots: &'data [ArtifactSnapshot<'bytes>],
    pub policy: MitigationPolicy,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MitigationEstimate {
    pub baseline_rate_kg_h: f64,
    /// Only populated when every follow-up has a quantified point estimate.
    pub median_followup_rate_kg_h: Option<f64>,
    /// Baseline minus median follow-up rate; absent for non-detect follow-ups.
    pub observed_reduction_kg_h: Option<f64>,
    /// Conservative interval using the baseline bounds and worst/best follow-up bounds.
    pub conservative_reduction_interval_kg_h: (f64, f64),
    pub followup_count: usize,
    /// True only when the lower bound is positive across all submitted follow-ups.
    pub sustained_reduction_supported: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MitigationAssessmentReport {
    pub violations: Vec<crate::Violation>,
    pub estimate: Option<MitigationEstimate>,
}

impl MitigationAssessmentReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty() && self.estimate.is_some()
    }

    fn push(&mut self, record_id: &str, field: &str, code: &'static str) {
        self.violations.push(crate::Violation {
            record_id: record_id.to_owned(),
            field: field.to_owned(),
            code,
        });
    }
}

#[derive(Clone, Copy, Debug)]
struct RateBounds {
    point: Option<f64>,
    lower: f64,
    upper: f64,
}

struct InterventionArtifactExpectation<'a> {
    field: &'a str,
    role: ArtifactRole,
    uri: &'a str,
    expected_sha256: &'a str,
    missing_code: &'static str,
    duplicate_code: &'static str,
    mismatch_code: &'static str,
}

/// Assess a pre-intervention reading against repeated follow-ups for one source.
///
/// All referenced observations must exist exactly once. The corpus must include the
/// exact bytes for source, terms, calibration, work-order and verification snapshots.
/// This returns rate bounds only; it never extrapolates to tonnes avoided over an assumed
/// duration.
pub fn assess_mitigation(
    request: MitigationAssessmentRequest<'_, '_>,
) -> MitigationAssessmentReport {
    let MitigationAssessmentRequest {
        intervention,
        baseline_observation_id,
        followup_observation_ids,
        observations,
        associations,
        required_use,
        artifact_snapshots,
        policy,
    } = request;
    let mut report = MitigationAssessmentReport::default();
    let intervention_id = intervention.intervention_id.as_str();

    if intervention_id.trim().is_empty() {
        report.push(intervention_id, "intervention_id", "empty_intervention_id");
    }
    if intervention.source_id.0.trim().is_empty() {
        report.push(intervention_id, "source_id", "empty_source_id");
    }
    if intervention.method.trim().is_empty() {
        report.push(intervention_id, "method", "empty_intervention_method");
    }
    if intervention.independent_verifier_id.trim().is_empty() {
        report.push(
            intervention_id,
            "independent_verifier_id",
            "empty_independent_verifier_id",
        );
    }
    if !is_utc_timestamp(&intervention.completed_at_utc) {
        report.push(intervention_id, "completed_at_utc", "invalid_utc_timestamp");
    }
    match intervention.status {
        RepairVerificationStatus::Verified => {}
        RepairVerificationStatus::NotVerified => {
            report.push(intervention_id, "status", "intervention_not_verified");
        }
        RepairVerificationStatus::Disputed => {
            report.push(
                intervention_id,
                "status",
                "intervention_verification_disputed",
            );
        }
    }
    if policy.minimum_followup_count < 2 || policy.minimum_followup_span_seconds == 0 {
        report.push(intervention_id, "policy", "invalid_mitigation_policy");
    }
    if followup_observation_ids.len() < policy.minimum_followup_count {
        report.push(
            intervention_id,
            "followup_observation_ids",
            "insufficient_followup_count",
        );
    }

    validate_intervention_artifact(
        &mut report,
        intervention_id,
        InterventionArtifactExpectation {
            field: "work_order",
            role: ArtifactRole::WorkOrder,
            uri: &intervention.work_order_uri,
            expected_sha256: &intervention.work_order_sha256,
            missing_code: "work_order_bytes_missing",
            duplicate_code: "duplicate_work_order_snapshot",
            mismatch_code: "work_order_digest_mismatch",
        },
        artifact_snapshots,
    );
    validate_intervention_artifact(
        &mut report,
        intervention_id,
        InterventionArtifactExpectation {
            field: "verification_report",
            role: ArtifactRole::VerificationRecord,
            uri: &intervention.verification_report_uri,
            expected_sha256: &intervention.verification_report_sha256,
            missing_code: "verification_report_bytes_missing",
            duplicate_code: "duplicate_verification_snapshot",
            mismatch_code: "verification_report_digest_mismatch",
        },
        artifact_snapshots,
    );

    let mut required_ids = Vec::with_capacity(followup_observation_ids.len() + 1);
    required_ids.push(baseline_observation_id.to_owned());
    required_ids.extend(followup_observation_ids.iter().cloned());
    let mut seen_ids = BTreeSet::new();
    for observation_id in &required_ids {
        if observation_id.trim().is_empty() {
            report.push(
                intervention_id,
                "observation_id",
                "empty_referenced_observation_id",
            );
        }
        if !seen_ids.insert(observation_id.as_str()) {
            report.push(
                observation_id,
                "observation_id",
                "duplicate_mitigation_observation_id",
            );
        }
    }

    let mut observations_by_id: HashMap<&str, Vec<&Observation>> = HashMap::new();
    for observation in observations {
        observations_by_id
            .entry(observation.observation_id.as_str())
            .or_default()
            .push(observation);
    }

    let mut readings: Vec<&Observation> = Vec::with_capacity(required_ids.len());
    for observation_id in &required_ids {
        match observations_by_id.get(observation_id.as_str()) {
            None => report.push(
                observation_id,
                "observation_id",
                "referenced_observation_missing",
            ),
            Some(matches) if matches.len() != 1 => {
                report.push(observation_id, "observation_id", "ambiguous_observation_id");
            }
            Some(matches) => readings.push(matches[0]),
        }
    }

    if readings.len() == required_ids.len() {
        let reading_ids: BTreeSet<&str> = readings
            .iter()
            .map(|reading| reading.observation_id.as_str())
            .collect();
        let relevant_associations: Vec<SourceAssociation> = associations
            .iter()
            .filter(|association| reading_ids.contains(association.observation_id.as_str()))
            .cloned()
            .collect();
        let owned_readings: Vec<Observation> =
            readings.iter().map(|reading| (*reading).clone()).collect();

        let validation = validate_batch(
            &owned_readings,
            &relevant_associations,
            required_use,
            ObservationKind::InSituMeasurement,
            &GasSpecies::Methane,
            artifact_snapshots,
        );
        copy_validation_violations(&mut report, validation);

        for reading in &readings {
            validate_source_link(
                &mut report,
                &reading.observation_id,
                &intervention.source_id,
                &relevant_associations,
            );
        }

        let baseline = readings[0];
        if is_utc_timestamp(&baseline.observed_at_utc)
            && is_utc_timestamp(&intervention.completed_at_utc)
            && baseline.observed_at_utc >= intervention.completed_at_utc
        {
            report.push(
                &baseline.observation_id,
                "observed_at_utc",
                "baseline_not_before_intervention",
            );
        }

        let baseline_instrument = baseline.instrument_id.as_deref();
        let baseline_method = baseline.processing_method.as_deref();
        for followup in readings.iter().skip(1) {
            if is_utc_timestamp(&followup.observed_at_utc)
                && is_utc_timestamp(&intervention.completed_at_utc)
                && followup.observed_at_utc <= intervention.completed_at_utc
            {
                report.push(
                    &followup.observation_id,
                    "observed_at_utc",
                    "followup_not_after_intervention",
                );
            }
            if followup.instrument_id.as_deref() != baseline_instrument {
                report.push(
                    &followup.observation_id,
                    "instrument_id",
                    "mitigation_instrument_mismatch",
                );
            }
            if followup.processing_method.as_deref() != baseline_method {
                report.push(
                    &followup.observation_id,
                    "processing_method",
                    "mitigation_method_mismatch",
                );
            }
        }

        let timestamps: Vec<i64> = readings
            .iter()
            .skip(1)
            .filter_map(|reading| utc_timestamp_seconds(&reading.observed_at_utc))
            .collect();
        if timestamps.len() == followup_observation_ids.len() && !timestamps.is_empty() {
            let earliest = *timestamps.iter().min().unwrap();
            let latest = *timestamps.iter().max().unwrap();
            if latest.saturating_sub(earliest) < policy.minimum_followup_span_seconds as i64 {
                report.push(
                    intervention_id,
                    "followup_observation_ids",
                    "insufficient_followup_span",
                );
            }
        } else {
            report.push(
                intervention_id,
                "followup_observation_ids",
                "invalid_followup_timestamp",
            );
        }
    }

    if !report.violations.is_empty() {
        sort_violations(&mut report);
        return report;
    }

    let baseline = readings[0];
    let (baseline_rate, baseline_uncertainty) = match &baseline.quantification {
        Quantification::Quantified {
            emission_rate_kg_h,
            uncertainty_kg_h,
            ..
        } => (*emission_rate_kg_h, *uncertainty_kg_h),
        Quantification::NonDetect { .. } | Quantification::NotQuantified { .. } => {
            report.push(
                &baseline.observation_id,
                "quantification",
                "baseline_not_quantified",
            );
            return report;
        }
    };

    let baseline_lower = (baseline_rate - baseline_uncertainty).max(0.0);
    let baseline_upper = baseline_rate + baseline_uncertainty;
    let mut followup_bounds = Vec::with_capacity(followup_observation_ids.len());
    let mut followup_points = Vec::with_capacity(followup_observation_ids.len());
    let mut all_followups_quantified = true;

    for followup in readings.iter().skip(1) {
        let bounds = match &followup.quantification {
            Quantification::Quantified {
                emission_rate_kg_h,
                uncertainty_kg_h,
                ..
            } => RateBounds {
                point: Some(*emission_rate_kg_h),
                lower: (*emission_rate_kg_h - *uncertainty_kg_h).max(0.0),
                upper: *emission_rate_kg_h + *uncertainty_kg_h,
            },
            Quantification::NonDetect {
                detection_limit_kg_h,
            } => {
                all_followups_quantified = false;
                RateBounds {
                    point: None,
                    lower: 0.0,
                    upper: *detection_limit_kg_h,
                }
            }
            Quantification::NotQuantified { .. } => {
                report.push(
                    &followup.observation_id,
                    "quantification",
                    "followup_not_quantified",
                );
                continue;
            }
        };
        if let Some(point) = bounds.point {
            followup_points.push(point);
        }
        followup_bounds.push(bounds);
    }

    if !report.violations.is_empty() || followup_bounds.len() != followup_observation_ids.len() {
        sort_violations(&mut report);
        return report;
    }

    let largest_followup_upper = followup_bounds
        .iter()
        .map(|bounds| bounds.upper)
        .fold(f64::NEG_INFINITY, f64::max);
    let smallest_followup_lower = followup_bounds
        .iter()
        .map(|bounds| bounds.lower)
        .fold(f64::INFINITY, f64::min);
    followup_points.sort_by(f64::total_cmp);
    let median_followup_rate_kg_h = if all_followups_quantified {
        median_sorted(&followup_points)
    } else {
        None
    };
    let observed_reduction_kg_h =
        median_followup_rate_kg_h.map(|post_rate| baseline_rate - post_rate);
    let reduction_interval = (
        baseline_lower - largest_followup_upper,
        baseline_upper - smallest_followup_lower,
    );

    report.estimate = Some(MitigationEstimate {
        baseline_rate_kg_h: baseline_rate,
        median_followup_rate_kg_h,
        observed_reduction_kg_h,
        conservative_reduction_interval_kg_h: reduction_interval,
        followup_count: followup_observation_ids.len(),
        sustained_reduction_supported: reduction_interval.0 > 0.0,
    });
    report
}

fn validate_intervention_artifact(
    report: &mut MitigationAssessmentReport,
    record_id: &str,
    expectation: InterventionArtifactExpectation<'_>,
    snapshots: &[ArtifactSnapshot<'_>],
) {
    if !expectation.uri.starts_with("https://") {
        report.push(
            record_id,
            expectation.field,
            "intervention_artifact_uri_not_https",
        );
    }
    if !is_canonical_sha256(expectation.expected_sha256) {
        report.push(
            record_id,
            expectation.field,
            "invalid_intervention_artifact_sha256",
        );
    }
    let mut matches = snapshots
        .iter()
        .filter(|snapshot| snapshot.role == expectation.role && snapshot.uri == expectation.uri);
    let Some(snapshot) = matches.next() else {
        report.push(record_id, expectation.field, expectation.missing_code);
        return;
    };
    if matches.next().is_some() {
        report.push(record_id, expectation.field, expectation.duplicate_code);
        return;
    }
    if snapshot.bytes.is_empty() {
        report.push(record_id, expectation.field, "empty_intervention_artifact");
    }
    if is_canonical_sha256(expectation.expected_sha256)
        && sha256_hex(snapshot.bytes) != expectation.expected_sha256
    {
        report.push(record_id, expectation.field, expectation.mismatch_code);
    }
}

fn validate_source_link(
    report: &mut MitigationAssessmentReport,
    observation_id: &str,
    required_source_id: &SourceId,
    associations: &[SourceAssociation],
) {
    let links: Vec<&SourceAssociation> = associations
        .iter()
        .filter(|association| association.observation_id == observation_id)
        .collect();
    if links.len() != 1 {
        report.push(
            observation_id,
            "source_association",
            if links.is_empty() {
                "source_association_missing"
            } else {
                "source_association_ambiguous"
            },
        );
        return;
    }
    let association = links[0];
    if &association.source_id != required_source_id {
        report.push(observation_id, "source_id", "intervention_source_mismatch");
    }
    if association.method.trim().is_empty() {
        report.push(
            observation_id,
            "source_association.method",
            "empty_association_method",
        );
    }
    if !association.match_score.is_finite() || !(0.0..=1.0).contains(&association.match_score) {
        report.push(
            observation_id,
            "source_association.match_score",
            "invalid_match_score",
        );
    }
    match &association.review {
        AssociationReview::Reviewed { reviewer_id } if !reviewer_id.trim().is_empty() => {}
        AssociationReview::Reviewed { .. } => {
            report.push(
                observation_id,
                "source_association.review",
                "empty_reviewer_id",
            );
        }
        AssociationReview::Unreviewed => {
            report.push(
                observation_id,
                "source_association.review",
                "association_unreviewed",
            );
        }
        AssociationReview::Rejected { .. } => {
            report.push(
                observation_id,
                "source_association.review",
                "association_rejected",
            );
        }
    }
}

fn copy_validation_violations(
    report: &mut MitigationAssessmentReport,
    validation: ValidationReport,
) {
    report.violations.extend(validation.violations);
}

fn sort_violations(report: &mut MitigationAssessmentReport) {
    report.violations.sort_by(|left, right| {
        (&left.record_id, &left.field, left.code).cmp(&(&right.record_id, &right.field, right.code))
    });
    report.violations.dedup();
}

fn utc_timestamp_seconds(value: &str) -> Option<i64> {
    if !is_utc_timestamp(value) {
        return None;
    }
    let year = value.get(0..4)?.parse::<i64>().ok()?;
    let month = value.get(5..7)?.parse::<usize>().ok()?;
    let day = value.get(8..10)?.parse::<i64>().ok()?;
    let hour = value.get(11..13)?.parse::<i64>().ok()?;
    let minute = value.get(14..16)?.parse::<i64>().ok()?;
    let second = value.get(17..19)?.parse::<i64>().ok()?;
    let previous_year = year - 1;
    let mut days =
        365 * previous_year + previous_year / 4 - previous_year / 100 + previous_year / 400;
    const DAYS_BEFORE_MONTH: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    days += DAYS_BEFORE_MONTH[month - 1];
    if month > 2 && is_leap_year(year) {
        days += 1;
    }
    days += day - 1;
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn is_leap_year(year: i64) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
    use crate::{ArtifactProvenance, DataUse, FacilityId, TermsReview};

    const SOURCE_BYTES: &[u8] = b"synthetic source content v1";
    const TERMS_BYTES: &[u8] = b"synthetic terms snapshot v1";
    const CALIBRATION_BYTES: &[u8] = b"synthetic calibration certificate v1";
    const WORK_ORDER_BYTES: &[u8] = b"synthetic work order v1";
    const VERIFICATION_BYTES: &[u8] = b"synthetic independent verification report v1";

    fn provenance() -> ArtifactProvenance {
        let source_sha256 = sha256_hex(SOURCE_BYTES);
        ArtifactProvenance {
            source_uri: "https://data.example.test/source.csv".into(),
            source_version: "fixture-v1".into(),
            artifact_sha256: source_sha256.clone(),
            retrieved_at_utc: "2026-01-01T00:00:00Z".into(),
            license_id: "CC-BY-NC-SA-4.0".into(),
            attribution: "Synthetic fixture".into(),
            terms_review: TermsReview::Approved {
                reviewer_id: "terms-reviewer".into(),
                approved_use: DataUse::ResearchNonCommercial,
                terms_uri: "https://data.example.test/terms".into(),
                terms_sha256: sha256_hex(TERMS_BYTES),
                reviewed_at_utc: "2026-01-01T00:00:00Z".into(),
                reviewed_source_uri: "https://data.example.test/source.csv".into(),
                reviewed_source_version: "fixture-v1".into(),
                reviewed_source_sha256: source_sha256,
            },
        }
    }

    fn observation(
        observation_id: &str,
        observed_at_utc: &str,
        quantification: Quantification,
    ) -> Observation {
        Observation {
            observation_id: observation_id.into(),
            kind: ObservationKind::InSituMeasurement,
            gas_species: GasSpecies::Methane,
            instrument_id: Some("meter-1".into()),
            processing_method: Some("direct-meter-method-v1".into()),
            observed_at_utc: observed_at_utc.into(),
            quality: crate::QualityFlag::Qualified,
            quantification,
            provenance: provenance(),
            calibration: Some(crate::MeasurementCalibration {
                instrument_id: "meter-1".into(),
                calibration_uri: "https://data.example.test/calibration".into(),
                calibration_sha256: sha256_hex(CALIBRATION_BYTES),
                valid_from_utc: "2026-01-01T00:00:00Z".into(),
                valid_until_utc: "2026-12-31T23:59:59Z".into(),
            }),
        }
    }

    fn association(observation_id: &str, source_id: &str) -> SourceAssociation {
        SourceAssociation {
            association_id: format!("association-{observation_id}"),
            observation_id: observation_id.into(),
            source_id: SourceId(source_id.into()),
            facility_id: Some(FacilityId("facility-1".into())),
            method: "reviewed-meter-to-source-link".into(),
            match_score: 1.0,
            review: AssociationReview::Reviewed {
                reviewer_id: "source-reviewer".into(),
            },
        }
    }

    fn intervention() -> InterventionRecord {
        InterventionRecord {
            intervention_id: "repair-1".into(),
            source_id: SourceId("source-1".into()),
            completed_at_utc: "2026-01-02T00:00:00Z".into(),
            method: "replace defective valve".into(),
            status: RepairVerificationStatus::Verified,
            independent_verifier_id: "field-verifier-1".into(),
            work_order_uri: "https://data.example.test/work-order".into(),
            work_order_sha256: sha256_hex(WORK_ORDER_BYTES),
            verification_report_uri: "https://data.example.test/verification".into(),
            verification_report_sha256: sha256_hex(VERIFICATION_BYTES),
        }
    }

    fn snapshots() -> [ArtifactSnapshot<'static>; 5] {
        [
            ArtifactSnapshot {
                role: ArtifactRole::Dataset,
                uri: "https://data.example.test/source.csv",
                bytes: SOURCE_BYTES,
            },
            ArtifactSnapshot {
                role: ArtifactRole::Terms,
                uri: "https://data.example.test/terms",
                bytes: TERMS_BYTES,
            },
            ArtifactSnapshot {
                role: ArtifactRole::Calibration,
                uri: "https://data.example.test/calibration",
                bytes: CALIBRATION_BYTES,
            },
            ArtifactSnapshot {
                role: ArtifactRole::WorkOrder,
                uri: "https://data.example.test/work-order",
                bytes: WORK_ORDER_BYTES,
            },
            ArtifactSnapshot {
                role: ArtifactRole::VerificationRecord,
                uri: "https://data.example.test/verification",
                bytes: VERIFICATION_BYTES,
            },
        ]
    }

    fn quant(rate: f64, uncertainty: f64) -> Quantification {
        Quantification::Quantified {
            emission_rate_kg_h: rate,
            uncertainty_kg_h: uncertainty,
            detection_limit_kg_h: 1.0,
        }
    }

    fn readings(
        baseline: Quantification,
        followup1: Quantification,
        followup2: Quantification,
    ) -> (Vec<Observation>, Vec<SourceAssociation>) {
        (
            vec![
                observation("before", "2026-01-01T00:00:00Z", baseline),
                observation("after-1", "2026-01-04T00:00:00Z", followup1),
                observation("after-2", "2026-01-06T00:00:00Z", followup2),
            ],
            vec![
                association("before", "source-1"),
                association("after-1", "source-1"),
                association("after-2", "source-1"),
            ],
        )
    }

    fn assess(
        intervention: &InterventionRecord,
        observations: &[Observation],
        associations: &[SourceAssociation],
        artifacts: &[ArtifactSnapshot<'_>],
    ) -> MitigationAssessmentReport {
        let followup_ids = ["after-1".into(), "after-2".into()];
        assess_mitigation(MitigationAssessmentRequest {
            intervention,
            baseline_observation_id: "before",
            followup_observation_ids: &followup_ids,
            observations,
            associations,
            required_use: DataUse::ResearchNonCommercial,
            artifact_snapshots: artifacts,
            policy: MitigationPolicy {
                minimum_followup_count: 2,
                minimum_followup_span_seconds: 86_400,
            },
        })
    }

    #[test]
    fn estimates_bounded_rate_reduction_from_verified_repeated_followups() {
        let (observations, associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(report.is_valid(), "{report:?}");
        let estimate = report.estimate.unwrap();
        assert_eq!(estimate.baseline_rate_kg_h, 100.0);
        assert_eq!(estimate.median_followup_rate_kg_h, Some(17.5));
        assert_eq!(estimate.observed_reduction_kg_h, Some(82.5));
        assert_eq!(estimate.conservative_reduction_interval_kg_h, (72.0, 94.0));
        assert!(estimate.sustained_reduction_supported);
        assert_eq!(estimate.followup_count, 2);
    }

    #[test]
    fn post_intervention_nondetections_produce_bounds_not_zero_emission_claims() {
        let (observations, associations) = readings(
            quant(100.0, 5.0),
            Quantification::NonDetect {
                detection_limit_kg_h: 10.0,
            },
            Quantification::NonDetect {
                detection_limit_kg_h: 10.0,
            },
        );
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(report.is_valid(), "{report:?}");
        let estimate = report.estimate.unwrap();
        assert_eq!(estimate.median_followup_rate_kg_h, None);
        assert_eq!(estimate.observed_reduction_kg_h, None);
        assert_eq!(estimate.conservative_reduction_interval_kg_h, (85.0, 105.0));
        assert!(estimate.sustained_reduction_supported);
    }

    #[test]
    fn overlapping_uncertainty_does_not_claim_sustained_reduction() {
        let (observations, associations) =
            readings(quant(100.0, 50.0), quant(80.0, 30.0), quant(70.0, 40.0));
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(report.is_valid(), "{report:?}");
        let estimate = report.estimate.unwrap();
        assert!(estimate.conservative_reduction_interval_kg_h.0 < 0.0);
        assert!(!estimate.sustained_reduction_supported);
    }

    #[test]
    fn rejects_unverified_intervention() {
        let (observations, associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        let mut event = intervention();
        event.status = RepairVerificationStatus::NotVerified;
        let report = assess(&event, &observations, &associations, &snapshots());
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "intervention_not_verified")
        );
    }

    #[test]
    fn rejects_tampered_verification_report_bytes() {
        let (observations, associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        let mut artifacts = snapshots();
        artifacts[4] = ArtifactSnapshot {
            role: ArtifactRole::VerificationRecord,
            uri: "https://data.example.test/verification",
            bytes: b"tampered report",
        };
        let report = assess(&intervention(), &observations, &associations, &artifacts);
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "verification_report_digest_mismatch")
        );
    }

    #[test]
    fn rejects_followup_associated_with_another_source() {
        let (observations, mut associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        associations[2].source_id = SourceId("source-other".into());
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "intervention_source_mismatch")
        );
    }

    #[test]
    fn rejects_followups_that_do_not_span_the_policy_window() {
        let (mut observations, associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        observations[2].observed_at_utc = "2026-01-04T12:00:00Z".into();
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "insufficient_followup_span")
        );
    }

    #[test]
    fn rejects_baseline_after_repair_completion() {
        let (mut observations, associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        observations[0].observed_at_utc = "2026-01-03T00:00:00Z".into();
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "baseline_not_before_intervention")
        );
    }

    #[test]
    fn rejects_cross_instrument_before_after_comparisons() {
        let (mut observations, associations) =
            readings(quant(100.0, 5.0), quant(20.0, 3.0), quant(15.0, 4.0));
        observations[1].instrument_id = Some("meter-2".into());
        observations[1].calibration.as_mut().unwrap().instrument_id = "meter-2".into();
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "mitigation_instrument_mismatch")
        );
    }

    #[test]
    fn rejects_baseline_non_detection_for_point_rate_reduction() {
        let (observations, associations) = readings(
            Quantification::NonDetect {
                detection_limit_kg_h: 10.0,
            },
            quant(20.0, 3.0),
            quant(15.0, 4.0),
        );
        let report = assess(&intervention(), &observations, &associations, &snapshots());
        assert!(!report.is_valid());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "baseline_not_quantified")
        );
    }
}
