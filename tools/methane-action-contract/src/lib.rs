//! Fail-closed admission contract for methane observation benchmarks.
//!
//! This validates record structure and checks declared digests against supplied bytes. It does not
//! authenticate reviewers, establish legal rights,
//! calibrate instruments, validate emission estimates, or prove source attribution.

pub mod census;
pub mod comparison;
pub mod split;
pub mod mitigation;

use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataUse {
    ResearchNonCommercial,
    InternalNonCommercial,
    Commercial,
    Operational,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermsReview {
    NotReviewed,
    Rejected {
        reviewer_id: String,
        reason: String,
        terms_uri: String,
        terms_sha256: String,
        reviewed_at_utc: String,
    },
    Approved {
        reviewer_id: String,
        approved_use: DataUse,
        terms_uri: String,
        terms_sha256: String,
        reviewed_at_utc: String,
        reviewed_source_uri: String,
        reviewed_source_version: String,
        reviewed_source_sha256: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactProvenance {
    pub source_uri: String,
    pub source_version: String,
    /// Claimed SHA-256 of exact source bytes; verification is external to this crate.
    pub artifact_sha256: String,
    pub retrieved_at_utc: String,
    pub license_id: String,
    pub attribution: String,
    pub terms_review: TermsReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactRole {
    Dataset,
    Terms,
    Calibration,
    WorkOrder,
    VerificationRecord,
}

/// Exact bytes fetched for an artifact. Digests are recomputed at admission time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactSnapshot<'a> {
    pub role: ArtifactRole,
    pub uri: &'a str,
    pub bytes: &'a [u8],
}

struct SnapshotExpectation<'a> {
    role: ArtifactRole,
    uri: &'a str,
    expected_sha256: &'a str,
    missing_code: &'static str,
    duplicate_code: &'static str,
    mismatch_code: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationKind {
    RemoteSensingPlume,
    InSituMeasurement,
    InventoryEstimate,
    ModelInference,
    SyntheticScenario,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GasSpecies {
    Methane,
    CarbonDioxide,
    Other(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Quantification {
    Quantified {
        emission_rate_kg_h: f64,
        /// Non-negative absolute bound around the rate; not assumed to be Gaussian or 1-sigma.
        uncertainty_kg_h: f64,
        detection_limit_kg_h: f64,
    },
    /// A non-detection is not an observation of zero emissions.
    NonDetect { detection_limit_kg_h: f64 },
    /// Preserve unquantified values as unknown; never coerce them to zero.
    NotQuantified { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QualityFlag {
    Qualified,
    Degraded { reason: String },
    Quarantined { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeasurementCalibration {
    pub instrument_id: String,
    pub calibration_uri: String,
    pub calibration_sha256: String,
    pub valid_from_utc: String,
    pub valid_until_utc: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    /// Unique acquisition/observation identity, not a source or facility identity.
    pub observation_id: String,
    pub kind: ObservationKind,
    pub gas_species: GasSpecies,
    /// Sensor/platform identity; required for remote-sensing and in-situ evidence.
    pub instrument_id: Option<String>,
    /// Retrieval or measurement-method/version identity for fair instrument comparisons.
    pub processing_method: Option<String>,
    pub observed_at_utc: String,
    pub quality: QualityFlag,
    pub quantification: Quantification,
    pub provenance: ArtifactProvenance,
    /// Required for in-situ measurements; optional for other evidence classes.
    pub calibration: Option<MeasurementCalibration>,
}

/// Newtypes prevent source identities from being passed as facility identities.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FacilityId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssociationReview {
    Unreviewed,
    Reviewed { reviewer_id: String },
    Rejected { reviewer_id: String, reason: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceAssociation {
    pub association_id: String,
    pub observation_id: String,
    pub source_id: SourceId,
    /// An optional facility match is separate from source association.
    pub facility_id: Option<FacilityId>,
    pub method: String,
    /// A score in [0, 1], not a calibrated probability unless separately validated.
    pub match_score: f64,
    pub review: AssociationReview,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub record_id: String,
    pub field: String,
    pub code: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub violations: Vec<Violation>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }

    fn push(&mut self, record_id: &str, field: &str, code: &'static str) {
        self.violations.push(Violation {
            record_id: record_id.to_owned(),
            field: field.to_owned(),
            code,
        });
    }
}

/// Validate the entire batch; callers must reject it if any violation exists.
/// Source and terms digests are recomputed from the matching supplied byte snapshots.
pub fn validate_batch(
    observations: &[Observation],
    associations: &[SourceAssociation],
    required_use: DataUse,
    required_kind: ObservationKind,
    required_gas: &GasSpecies,
    artifact_snapshots: &[ArtifactSnapshot<'_>],
) -> ValidationReport {
    let mut report = ValidationReport::default();
    if observations.is_empty() {
        report.push("__batch__", "observations", "empty_observation_batch");
    }
    let mut observation_ids = HashSet::new();

    for obs in observations {
        let id = obs.observation_id.as_str();
        if id.trim().is_empty() {
            report.push(id, "observation_id", "empty_id");
        }
        if !observation_ids.insert(id.to_owned()) {
            report.push(id, "observation_id", "duplicate_observation_id");
        }
        if !is_utc_timestamp(&obs.observed_at_utc) {
            report.push(id, "observed_at_utc", "invalid_utc_timestamp");
        }
        if &obs.gas_species != required_gas {
            report.push(id, "gas_species", "gas_species_mismatch");
        }
        if obs.kind != required_kind {
            report.push(id, "kind", "observation_kind_mismatch");
        }
        validate_observation_method(&mut report, obs);
        validate_provenance(
            &mut report,
            id,
            &obs.provenance,
            required_use,
            artifact_snapshots,
        );
        validate_calibration(&mut report, obs, artifact_snapshots);

        match &obs.quality {
            QualityFlag::Qualified => {}
            QualityFlag::Degraded { reason } if !reason.trim().is_empty() => {}
            QualityFlag::Degraded { .. } => {
                report.push(id, "quality.reason", "empty_quality_reason");
            }
            QualityFlag::Quarantined { reason } => {
                if reason.trim().is_empty() {
                    report.push(id, "quality.reason", "empty_quality_reason");
                }
                report.push(id, "quality", "quarantined_observation");
            }
        }

        match &obs.quantification {
            Quantification::Quantified {
                emission_rate_kg_h,
                uncertainty_kg_h,
                detection_limit_kg_h,
            } => {
                if !emission_rate_kg_h.is_finite() || *emission_rate_kg_h <= 0.0 {
                    report.push(
                        id,
                        "quantification.emission_rate_kg_h",
                        "invalid_emission_rate",
                    );
                }
                if !uncertainty_kg_h.is_finite() || *uncertainty_kg_h < 0.0 {
                    report.push(id, "quantification.uncertainty_kg_h", "invalid_uncertainty");
                }
                if !detection_limit_kg_h.is_finite() || *detection_limit_kg_h <= 0.0 {
                    report.push(
                        id,
                        "quantification.detection_limit_kg_h",
                        "invalid_detection_limit",
                    );
                }
            }
            Quantification::NonDetect {
                detection_limit_kg_h,
            } => {
                if !detection_limit_kg_h.is_finite() || *detection_limit_kg_h <= 0.0 {
                    report.push(
                        id,
                        "quantification.detection_limit_kg_h",
                        "invalid_detection_limit",
                    );
                }
            }
            Quantification::NotQuantified { reason } if reason.trim().is_empty() => {
                report.push(id, "quantification.reason", "empty_quantification_reason");
            }
            Quantification::NotQuantified { .. } => {}
        }
    }

    let mut association_ids = HashSet::new();
    for assoc in associations {
        let id = assoc.association_id.as_str();
        if id.trim().is_empty() {
            report.push(id, "association_id", "empty_id");
        }
        if !association_ids.insert(id.to_owned()) {
            report.push(id, "association_id", "duplicate_association_id");
        }
        if !observation_ids.contains(&assoc.observation_id) {
            report.push(id, "observation_id", "unknown_observation_reference");
        }
        if assoc.source_id.0.trim().is_empty() {
            report.push(id, "source_id", "empty_source_id");
        }
        if assoc
            .facility_id
            .as_ref()
            .is_some_and(|facility| facility.0.trim().is_empty())
        {
            report.push(id, "facility_id", "empty_optional_facility_id");
        }
        if assoc.method.trim().is_empty() {
            report.push(id, "method", "empty_association_method");
        }
        if !assoc.match_score.is_finite() || !(0.0..=1.0).contains(&assoc.match_score) {
            report.push(id, "match_score", "invalid_match_score");
        }

        match &assoc.review {
            AssociationReview::Unreviewed => {
                report.push(id, "review", "association_unreviewed");
            }
            AssociationReview::Reviewed { reviewer_id } if reviewer_id.trim().is_empty() => {
                report.push(id, "review.reviewer_id", "empty_reviewer_id");
            }
            AssociationReview::Rejected {
                reviewer_id,
                reason,
            } => {
                if reviewer_id.trim().is_empty() {
                    report.push(id, "review.reviewer_id", "empty_reviewer_id");
                }
                if reason.trim().is_empty() {
                    report.push(id, "review.reason", "empty_review_reason");
                }
                report.push(id, "review", "association_rejected");
            }
            AssociationReview::Reviewed { .. } => {}
        }
    }

    report.violations.sort_by_key(|violation| {
        (
            violation.record_id.clone(),
            violation.field.clone(),
            violation.code,
        )
    });
    report
}

fn validate_observation_method(report: &mut ValidationReport, observation: &Observation) {
    let id = observation.observation_id.as_str();
    let is_instrument_observation = matches!(
        observation.kind,
        ObservationKind::RemoteSensingPlume | ObservationKind::InSituMeasurement
    );
    if !is_instrument_observation {
        return;
    }

    match observation.instrument_id.as_deref() {
        Some(instrument_id) if !instrument_id.trim().is_empty() => {}
        Some(_) => report.push(id, "instrument_id", "empty_instrument_id"),
        None => report.push(id, "instrument_id", "instrument_id_missing"),
    }
    match observation.processing_method.as_deref() {
        Some(method) if !method.trim().is_empty() => {}
        Some(_) => report.push(id, "processing_method", "empty_processing_method"),
        None => report.push(id, "processing_method", "processing_method_missing"),
    }

    if let (Some(instrument_id), Some(calibration)) = (
        observation.instrument_id.as_deref(),
        observation.calibration.as_ref(),
    ) && instrument_id != calibration.instrument_id
    {
        report.push(
            id,
            "calibration.instrument_id",
            "calibration_instrument_mismatch",
        );
    }
}

fn validate_calibration(
    report: &mut ValidationReport,
    observation: &Observation,
    snapshots: &[ArtifactSnapshot<'_>],
) {
    let id = observation.observation_id.as_str();
    let Some(calibration) = observation.calibration.as_ref() else {
        if observation.kind == ObservationKind::InSituMeasurement {
            report.push(id, "calibration", "in_situ_calibration_missing");
        }
        return;
    };

    if calibration.instrument_id.trim().is_empty() {
        report.push(id, "calibration.instrument_id", "empty_instrument_id");
    }
    if !calibration.calibration_uri.starts_with("https://") {
        report.push(
            id,
            "calibration.calibration_uri",
            "calibration_uri_not_https",
        );
    }
    if !is_utc_timestamp(&calibration.valid_from_utc) {
        report.push(id, "calibration.valid_from_utc", "invalid_utc_timestamp");
    }
    if !is_utc_timestamp(&calibration.valid_until_utc) {
        report.push(id, "calibration.valid_until_utc", "invalid_utc_timestamp");
    }
    if is_utc_timestamp(&calibration.valid_from_utc)
        && is_utc_timestamp(&calibration.valid_until_utc)
    {
        if calibration.valid_from_utc > calibration.valid_until_utc {
            report.push(id, "calibration.validity", "calibration_interval_inverted");
        } else if is_utc_timestamp(&observation.observed_at_utc)
            && (observation.observed_at_utc < calibration.valid_from_utc
                || observation.observed_at_utc > calibration.valid_until_utc)
        {
            report.push(
                id,
                "calibration.validity",
                "observation_outside_calibration_interval",
            );
        }
    }

    if !is_sha256_hex(&calibration.calibration_sha256) {
        report.push(
            id,
            "calibration.calibration_sha256",
            "invalid_sha256_format",
        );
    }
    validate_snapshot_digest(
        report,
        id,
        "calibration.calibration_sha256",
        SnapshotExpectation {
            role: ArtifactRole::Calibration,
            uri: &calibration.calibration_uri,
            expected_sha256: &calibration.calibration_sha256,
            missing_code: "calibration_artifact_bytes_missing",
            duplicate_code: "duplicate_calibration_artifact_snapshot",
            mismatch_code: "calibration_digest_mismatch",
        },
        snapshots,
    );
}

fn validate_provenance(
    report: &mut ValidationReport,
    record_id: &str,
    provenance: &ArtifactProvenance,
    required_use: DataUse,
    artifact_snapshots: &[ArtifactSnapshot<'_>],
) {
    if !provenance.source_uri.starts_with("https://") {
        report.push(record_id, "provenance.source_uri", "source_uri_not_https");
    }
    if provenance.source_version.trim().is_empty() {
        report.push(
            record_id,
            "provenance.source_version",
            "empty_source_version",
        );
    }
    if !is_sha256_hex(&provenance.artifact_sha256) {
        report.push(
            record_id,
            "provenance.artifact_sha256",
            "invalid_sha256_format",
        );
    }
    validate_snapshot_digest(
        report,
        record_id,
        "provenance.artifact_sha256",
        SnapshotExpectation {
            role: ArtifactRole::Dataset,
            uri: &provenance.source_uri,
            expected_sha256: &provenance.artifact_sha256,
            missing_code: "source_artifact_bytes_missing",
            duplicate_code: "duplicate_source_artifact_snapshot",
            mismatch_code: "source_digest_mismatch",
        },
        artifact_snapshots,
    );
    if !is_utc_timestamp(&provenance.retrieved_at_utc) {
        report.push(
            record_id,
            "provenance.retrieved_at_utc",
            "invalid_utc_timestamp",
        );
    }
    if provenance.license_id.trim().is_empty() {
        report.push(record_id, "provenance.license_id", "missing_license_id");
    }
    if provenance.attribution.trim().is_empty() {
        report.push(record_id, "provenance.attribution", "missing_attribution");
    }

    match &provenance.terms_review {
        TermsReview::NotReviewed => {
            report.push(record_id, "provenance.terms_review", "terms_not_reviewed");
        }
        TermsReview::Rejected {
            reviewer_id,
            reason,
            terms_uri,
            terms_sha256,
            reviewed_at_utc,
        } => {
            validate_terms_metadata(
                report,
                record_id,
                reviewer_id,
                terms_uri,
                terms_sha256,
                reviewed_at_utc,
            );
            validate_snapshot_digest(
                report,
                record_id,
                "provenance.terms_review.terms_sha256",
                SnapshotExpectation {
                    role: ArtifactRole::Terms,
                    uri: terms_uri,
                    expected_sha256: terms_sha256,
                    missing_code: "terms_artifact_bytes_missing",
                    duplicate_code: "duplicate_terms_artifact_snapshot",
                    mismatch_code: "terms_digest_mismatch",
                },
                artifact_snapshots,
            );
            if reason.trim().is_empty() {
                report.push(
                    record_id,
                    "provenance.terms_review.reason",
                    "empty_review_reason",
                );
            }
            report.push(record_id, "provenance.terms_review", "terms_rejected");
        }
        TermsReview::Approved {
            reviewer_id,
            approved_use,
            terms_uri,
            terms_sha256,
            reviewed_at_utc,
            reviewed_source_uri,
            reviewed_source_version,
            reviewed_source_sha256,
        } => {
            validate_terms_metadata(
                report,
                record_id,
                reviewer_id,
                terms_uri,
                terms_sha256,
                reviewed_at_utc,
            );
            validate_snapshot_digest(
                report,
                record_id,
                "provenance.terms_review.terms_sha256",
                SnapshotExpectation {
                    role: ArtifactRole::Terms,
                    uri: terms_uri,
                    expected_sha256: terms_sha256,
                    missing_code: "terms_artifact_bytes_missing",
                    duplicate_code: "duplicate_terms_artifact_snapshot",
                    mismatch_code: "terms_digest_mismatch",
                },
                artifact_snapshots,
            );
            if reviewed_source_uri != &provenance.source_uri {
                report.push(
                    record_id,
                    "provenance.terms_review.reviewed_source_uri",
                    "terms_source_uri_mismatch",
                );
            }
            if reviewed_source_version != &provenance.source_version {
                report.push(
                    record_id,
                    "provenance.terms_review.reviewed_source_version",
                    "terms_source_version_mismatch",
                );
            }
            if reviewed_source_sha256 != &provenance.artifact_sha256 {
                report.push(
                    record_id,
                    "provenance.terms_review.reviewed_source_sha256",
                    "terms_source_digest_mismatch",
                );
            }
            if *approved_use != required_use {
                report.push(
                    record_id,
                    "provenance.terms_review.approved_use",
                    "declared_use_mismatch",
                );
            }
        }
    }
}

fn validate_terms_metadata(
    report: &mut ValidationReport,
    record_id: &str,
    reviewer_id: &str,
    terms_uri: &str,
    terms_sha256: &str,
    reviewed_at_utc: &str,
) {
    if reviewer_id.trim().is_empty() {
        report.push(
            record_id,
            "provenance.terms_review.reviewer_id",
            "empty_reviewer_id",
        );
    }
    if !terms_uri.starts_with("https://") {
        report.push(
            record_id,
            "provenance.terms_review.terms_uri",
            "terms_uri_not_https",
        );
    }
    if !is_sha256_hex(terms_sha256) {
        report.push(
            record_id,
            "provenance.terms_review.terms_sha256",
            "invalid_sha256_format",
        );
    }
    if !is_utc_timestamp(reviewed_at_utc) {
        report.push(
            record_id,
            "provenance.terms_review.reviewed_at_utc",
            "invalid_utc_timestamp",
        );
    }
}

/// Compute lowercase SHA-256 over the exact supplied bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// SHA-256 of the exact executable image running this process.
///
/// This binds a benchmark receipt to a concrete evaluator binary. It does not
/// establish who built the binary or make an untrusted binary trustworthy.
pub fn executable_sha256() -> std::io::Result<String> {
    let executable_path = std::env::current_exe()?;
    let executable_bytes = std::fs::read(executable_path)?;
    Ok(sha256_hex(&executable_bytes))
}

fn validate_snapshot_digest(
    report: &mut ValidationReport,
    record_id: &str,
    field: &str,
    expectation: SnapshotExpectation<'_>,
    snapshots: &[ArtifactSnapshot<'_>],
) {
    let mut matching = snapshots
        .iter()
        .filter(|snapshot| snapshot.role == expectation.role && snapshot.uri == expectation.uri);
    let Some(snapshot) = matching.next() else {
        report.push(record_id, field, expectation.missing_code);
        return;
    };
    if matching.next().is_some() {
        report.push(record_id, field, expectation.duplicate_code);
        return;
    }
    if snapshot.bytes.is_empty() {
        report.push(record_id, field, "empty_artifact_bytes");
    }
    if !is_sha256_hex(expectation.expected_sha256) {
        return;
    }
    let actual_sha256 = sha256_hex(snapshot.bytes);
    if actual_sha256 != expectation.expected_sha256 {
        report.push(record_id, field, expectation.mismatch_code);
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Strict UTC seconds form: YYYY-MM-DDTHH:MM:SSZ. Adapters normalize offsets/fractions.
pub(crate) fn is_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }

    let number = |range: std::ops::Range<usize>| -> Option<u32> {
        let part = &bytes[range];
        if part.iter().all(|digit| digit.is_ascii_digit()) {
            std::str::from_utf8(part).ok()?.parse().ok()
        } else {
            None
        }
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0..4),
        number(5..7),
        number(8..10),
        number(11..13),
        number(14..16),
        number(17..19),
    ) else {
        return false;
    };

    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap_year =
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };
    (1..=month_days).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA_ARTIFACT_BYTES: &[u8] = b"synthetic methane source artifact v1";
    const TERMS_ARTIFACT_BYTES: &[u8] = b"synthetic terms snapshot v1";
    const CALIBRATION_ARTIFACT_BYTES: &[u8] = b"synthetic instrument calibration v1";

    fn artifact_snapshots() -> [ArtifactSnapshot<'static>; 3] {
        [
            ArtifactSnapshot {
                role: ArtifactRole::Dataset,
                uri: "https://data.example.test/methane.csv",
                bytes: DATA_ARTIFACT_BYTES,
            },
            ArtifactSnapshot {
                role: ArtifactRole::Terms,
                uri: "https://data.example.test/terms",
                bytes: TERMS_ARTIFACT_BYTES,
            },
            ArtifactSnapshot {
                role: ArtifactRole::Calibration,
                uri: "https://data.example.test/calibration.csv",
                bytes: CALIBRATION_ARTIFACT_BYTES,
            },
        ]
    }

    fn provenance() -> ArtifactProvenance {
        ArtifactProvenance {
            source_uri: "https://data.example.test/methane.csv".into(),
            source_version: "fixture-v1".into(),
            artifact_sha256: sha256_hex(DATA_ARTIFACT_BYTES),
            retrieved_at_utc: "2026-10-19T12:30:00Z".into(),
            license_id: "CC-BY-4.0".into(),
            attribution: "Synthetic fixture provider".into(),
            terms_review: TermsReview::Approved {
                reviewer_id: "reviewer-1".into(),
                approved_use: DataUse::ResearchNonCommercial,
                terms_uri: "https://data.example.test/terms".into(),
                terms_sha256: sha256_hex(TERMS_ARTIFACT_BYTES),
                reviewed_at_utc: "2026-10-19T12:00:00Z".into(),
                reviewed_source_uri: "https://data.example.test/methane.csv".into(),
                reviewed_source_version: "fixture-v1".into(),
                reviewed_source_sha256: sha256_hex(DATA_ARTIFACT_BYTES),
            },
        }
    }

    fn observation(id: &str, quantification: Quantification) -> Observation {
        Observation {
            observation_id: id.into(),
            kind: ObservationKind::RemoteSensingPlume,
            gas_species: GasSpecies::Methane,
            instrument_id: Some("satellite-1".into()),
            processing_method: Some("methane-retrieval-v1".into()),
            observed_at_utc: "2026-10-18T10:11:12Z".into(),
            quality: QualityFlag::Qualified,
            quantification,
            provenance: provenance(),
            calibration: None,
        }
    }

    fn quantified() -> Quantification {
        Quantification::Quantified {
            emission_rate_kg_h: 12.0,
            uncertainty_kg_h: 2.5,
            detection_limit_kg_h: 1.0,
        }
    }

    fn association(observation_id: &str) -> SourceAssociation {
        SourceAssociation {
            association_id: "assoc-1".into(),
            observation_id: observation_id.into(),
            source_id: SourceId("source-7".into()),
            facility_id: None,
            method: "reviewed-geospatial-match-v1".into(),
            match_score: 0.8,
            review: AssociationReview::Reviewed {
                reviewer_id: "reviewer-2".into(),
            },
        }
    }

    fn calibrated_observation(id: &str) -> Observation {
        let mut obs = observation(id, quantified());
        obs.kind = ObservationKind::InSituMeasurement;
        obs.instrument_id = Some("meter-1".into());
        obs.processing_method = Some("ground-meter-method-v1".into());
        obs.calibration = Some(MeasurementCalibration {
            instrument_id: "meter-1".into(),
            calibration_uri: "https://data.example.test/calibration.csv".into(),
            calibration_sha256: sha256_hex(CALIBRATION_ARTIFACT_BYTES),
            valid_from_utc: "2026-01-01T00:00:00Z".into(),
            valid_until_utc: "2026-12-31T23:59:59Z".into(),
        });
        obs
    }

    #[test]
    fn accepts_in_situ_measurement_with_pinned_calibration_in_validity_window() {
        let report = validate_batch(
            &[calibrated_observation("meter-reading-1")],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::InSituMeasurement,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(report.is_valid(), "{report:?}");
    }

    #[test]
    fn rejects_remote_sensing_observation_without_instrument_or_method_identity() {
        let mut obs = observation("plume-no-instrument", quantified());
        obs.instrument_id = None;
        obs.processing_method = None;
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "instrument_id_missing")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "processing_method_missing")
        );
    }

    #[test]
    fn rejects_calibration_for_a_different_instrument() {
        let mut obs = calibrated_observation("meter-reading-wrong-instrument");
        obs.calibration.as_mut().unwrap().instrument_id = "meter-2".into();
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::InSituMeasurement,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "calibration_instrument_mismatch")
        );
    }

    #[test]
    fn rejects_in_situ_measurement_without_calibration() {
        let mut obs = observation("meter-reading-no-calibration", quantified());
        obs.kind = ObservationKind::InSituMeasurement;
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::InSituMeasurement,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "in_situ_calibration_missing")
        );
    }

    #[test]
    fn rejects_in_situ_measurement_outside_calibration_interval() {
        let mut obs = calibrated_observation("meter-reading-expired-calibration");
        obs.observed_at_utc = "2027-01-01T00:00:00Z".into();
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::InSituMeasurement,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "observation_outside_calibration_interval")
        );
    }

    #[test]
    fn rejects_tampered_calibration_bytes() {
        let mut snapshots = artifact_snapshots();
        snapshots[2] = ArtifactSnapshot {
            role: ArtifactRole::Calibration,
            uri: "https://data.example.test/calibration.csv",
            bytes: b"tampered calibration",
        };
        let report = validate_batch(
            &[calibrated_observation("meter-reading-tampered-calibration")],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::InSituMeasurement,
            &GasSpecies::Methane,
            &snapshots,
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "calibration_digest_mismatch")
        );
    }

    #[test]
    fn accepts_well_formed_observation_and_separate_source_association() {
        let report = validate_batch(
            &[observation("plume-1", quantified())],
            &[association("plume-1")],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(report.is_valid(), "{report:?}");
    }

    #[test]
    fn rejects_mixing_inventory_estimates_into_plume_benchmark() {
        let mut obs = observation("inventory-1", quantified());
        obs.kind = ObservationKind::InventoryEstimate;
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "observation_kind_mismatch")
        );
    }

    #[test]
    fn rejects_cross_species_rate_mixing() {
        let mut obs = observation("co2-1", quantified());
        obs.gas_species = GasSpecies::CarbonDioxide;
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "gas_species_mismatch")
        );
    }

    #[test]
    fn rejects_duplicate_observation_ids_instead_of_double_counting() {
        let report = validate_batch(
            &[
                observation("plume-1", quantified()),
                observation("plume-1", quantified()),
            ],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "duplicate_observation_id")
        );
    }

    #[test]
    fn non_detection_is_explicit_not_zero() {
        let report = validate_batch(
            &[observation(
                "plume-2",
                Quantification::NonDetect {
                    detection_limit_kg_h: 0.2,
                },
            )],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(report.is_valid(), "{report:?}");
        assert!(matches!(
            observation(
                "x",
                Quantification::NonDetect {
                    detection_limit_kg_h: 0.2,
                }
            )
            .quantification,
            Quantification::NonDetect { .. }
        ));
    }

    #[test]
    fn rejects_negative_non_finite_and_zero_detected_rates() {
        for rate in [-1.0, 0.0, f64::NAN, f64::INFINITY] {
            let quantification = Quantification::Quantified {
                emission_rate_kg_h: rate,
                uncertainty_kg_h: 0.1,
                detection_limit_kg_h: 0.1,
            };
            let report = validate_batch(
                &[observation("plume-x", quantification)],
                &[],
                DataUse::ResearchNonCommercial,
                ObservationKind::RemoteSensingPlume,
                &GasSpecies::Methane,
                &artifact_snapshots(),
            );
            assert!(
                report
                    .violations
                    .iter()
                    .any(|v| v.code == "invalid_emission_rate")
            );
        }
    }

    #[test]
    fn validates_calendar_timestamps() {
        assert!(is_utc_timestamp("2024-02-29T00:00:00Z"));
        assert!(!is_utc_timestamp("2025-02-29T00:00:00Z"));
        assert!(!is_utc_timestamp("2026-13-01T00:00:00Z"));
        assert!(!is_utc_timestamp("2026-01-01T24:00:00Z"));

        let mut obs = observation("plume-3", quantified());
        obs.observed_at_utc = "2026-02-30T10:11:12Z".into();
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_utc_timestamp")
        );
    }

    #[test]
    fn refuses_unreviewed_or_mismatched_data_use() {
        let mut obs = observation("plume-4", quantified());
        obs.provenance.terms_review = TermsReview::NotReviewed;
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "terms_not_reviewed")
        );

        let mut commercial = observation("plume-4b", quantified());
        commercial.provenance.terms_review = TermsReview::Approved {
            reviewer_id: "reviewer-1".into(),
            approved_use: DataUse::ResearchNonCommercial,
            terms_uri: "https://data.example.test/terms".into(),
            terms_sha256: sha256_hex(TERMS_ARTIFACT_BYTES),
            reviewed_at_utc: "2026-10-19T12:00:00Z".into(),
            reviewed_source_uri: "https://data.example.test/methane.csv".into(),
            reviewed_source_version: "fixture-v1".into(),
            reviewed_source_sha256: sha256_hex(DATA_ARTIFACT_BYTES),
        };
        let report = validate_batch(
            &[commercial],
            &[],
            DataUse::Commercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "declared_use_mismatch")
        );
    }

    #[test]
    fn terms_review_is_bound_to_exact_source_identity_and_digest() {
        let mut obs = observation("plume-pinned-review", quantified());
        obs.provenance.terms_review = TermsReview::Approved {
            reviewer_id: "reviewer-1".into(),
            approved_use: DataUse::ResearchNonCommercial,
            terms_uri: "https://data.example.test/terms".into(),
            terms_sha256: sha256_hex(TERMS_ARTIFACT_BYTES),
            reviewed_at_utc: "2026-10-19T12:00:00Z".into(),
            reviewed_source_uri: "https://data.example.test/other.csv".into(),
            reviewed_source_version: "fixture-v2".into(),
            reviewed_source_sha256: "c".repeat(64),
        };
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "terms_source_uri_mismatch")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "terms_source_version_mismatch")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "terms_source_digest_mismatch")
        );
    }

    #[test]
    fn rejects_unpinned_terms_review_metadata() {
        let mut obs = observation("plume-terms", quantified());
        obs.provenance.terms_review = TermsReview::Approved {
            reviewer_id: String::new(),
            approved_use: DataUse::ResearchNonCommercial,
            terms_uri: "file:///unreviewable-terms".into(),
            terms_sha256: "invalid".into(),
            reviewed_at_utc: "not-a-timestamp".into(),
            reviewed_source_uri: "https://data.example.test/methane.csv".into(),
            reviewed_source_version: "fixture-v1".into(),
            reviewed_source_sha256: sha256_hex(DATA_ARTIFACT_BYTES),
        };
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "empty_reviewer_id")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "terms_uri_not_https")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_sha256_format")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_utc_timestamp")
        );
    }

    #[test]
    fn rejects_malformed_digest_and_missing_attribution() {
        let mut obs = observation("plume-5", quantified());
        obs.provenance.artifact_sha256 = "not-a-digest".into();
        obs.provenance.attribution.clear();
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_sha256_format")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "missing_attribution")
        );
    }

    #[test]
    fn rejects_unknown_observation_reference_and_invalid_association_score() {
        let mut assoc = association("missing-plume");
        assoc.match_score = 1.2;
        let report = validate_batch(
            &[],
            &[assoc],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "unknown_observation_reference")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "invalid_match_score")
        );
    }

    #[test]
    fn preserves_unquantified_as_explicit_unknown() {
        let obs = observation(
            "plume-6",
            Quantification::NotQuantified {
                reason: "retrieval quality below threshold".into(),
            },
        );
        assert!(
            validate_batch(
                &[obs],
                &[],
                DataUse::ResearchNonCommercial,
                ObservationKind::RemoteSensingPlume,
                &GasSpecies::Methane,
                &artifact_snapshots(),
            )
            .is_valid()
        );
    }

    #[test]
    fn quarantined_observation_blocks_batch_admission() {
        let mut obs = observation("plume-7", quantified());
        obs.quality = QualityFlag::Quarantined {
            reason: "stale calibration".into(),
        };
        let report = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "quarantined_observation")
        );
    }

    #[test]
    fn unreviewed_source_association_blocks_admission() {
        let mut assoc = association("plume-review");
        assoc.review = AssociationReview::Unreviewed;
        let report = validate_batch(
            &[observation("plume-review", quantified())],
            &[assoc],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "association_unreviewed")
        );
    }

    #[test]
    fn rejected_source_association_blocks_admission() {
        let mut assoc = association("plume-rejected");
        assoc.review = AssociationReview::Rejected {
            reviewer_id: "reviewer-2".into(),
            reason: "match contradicted by field evidence".into(),
        };
        let report = validate_batch(
            &[observation("plume-rejected", quantified())],
            &[assoc],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "association_rejected")
        );
    }

    #[test]
    fn rejects_duplicate_dataset_and_terms_snapshots() {
        let mut snapshots = artifact_snapshots().to_vec();
        snapshots.push(ArtifactSnapshot {
            role: ArtifactRole::Dataset,
            uri: "https://data.example.test/methane.csv",
            bytes: DATA_ARTIFACT_BYTES,
        });
        snapshots.push(ArtifactSnapshot {
            role: ArtifactRole::Terms,
            uri: "https://data.example.test/terms",
            bytes: TERMS_ARTIFACT_BYTES,
        });
        let report = validate_batch(
            &[observation("plume-duplicate-snapshot", quantified())],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &snapshots,
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "duplicate_source_artifact_snapshot")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "duplicate_terms_artifact_snapshot")
        );
    }

    #[test]
    fn rejects_empty_artifact_bytes() {
        let mut snapshots = artifact_snapshots();
        snapshots[0] = ArtifactSnapshot {
            role: ArtifactRole::Dataset,
            uri: "https://data.example.test/methane.csv",
            bytes: b"",
        };
        let report = validate_batch(
            &[observation("plume-empty-artifact", quantified())],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &snapshots,
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "empty_artifact_bytes")
        );
    }

    #[test]
    fn rejects_empty_observation_batch() {
        let report = validate_batch(
            &[],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "empty_observation_batch")
        );
    }

    #[test]
    fn rejects_noncanonical_uppercase_sha256_strings() {
        assert!(!is_sha256_hex(&"A".repeat(64)));
        assert!(is_sha256_hex(&"a".repeat(64)));
    }

    #[test]
    fn executable_digest_is_canonical_sha256() {
        let digest = executable_sha256().expect("test executable should be readable");
        assert!(is_sha256_hex(&digest));
    }

    #[test]
    fn sha256_helper_matches_standard_known_answer() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn rejects_tampered_source_bytes_even_when_metadata_digest_is_unchanged() {
        let mut snapshots = artifact_snapshots();
        snapshots[0] = ArtifactSnapshot {
            role: ArtifactRole::Dataset,
            uri: "https://data.example.test/methane.csv",
            bytes: b"tampered source bytes",
        };
        let report = validate_batch(
            &[observation("plume-tamper", quantified())],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &snapshots,
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "source_digest_mismatch")
        );
    }

    #[test]
    fn rejects_missing_terms_snapshot() {
        let snapshots = [ArtifactSnapshot {
            role: ArtifactRole::Dataset,
            uri: "https://data.example.test/methane.csv",
            bytes: DATA_ARTIFACT_BYTES,
        }];
        let report = validate_batch(
            &[observation("plume-no-terms", quantified())],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &snapshots,
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "terms_artifact_bytes_missing")
        );
    }

    #[test]
    fn validation_report_order_is_deterministic() {
        let mut obs = observation("plume-8", quantified());
        obs.provenance.attribution.clear();
        obs.observed_at_utc = "bad".into();
        let first = validate_batch(
            &[obs.clone()],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        let second = validate_batch(
            &[obs],
            &[],
            DataUse::ResearchNonCommercial,
            ObservationKind::RemoteSensingPlume,
            &GasSpecies::Methane,
            &artifact_snapshots(),
        );
        assert_eq!(first, second);
        assert!(first.violations.windows(2).all(|window| {
            (&window[0].record_id, &window[0].field, window[0].code)
                <= (&window[1].record_id, &window[1].field, window[1].code)
        }));
    }
}
