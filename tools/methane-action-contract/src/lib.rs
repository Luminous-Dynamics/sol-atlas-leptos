//! Fail-closed admission contract for methane observation benchmarks.
//!
//! This validates record structure and declared provenance only. It does NOT
//! authenticate reviewers, verify digests against bytes, establish legal rights,
//! calibrate instruments, validate emission estimates, or prove source attribution.

use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataUse { ResearchNonCommercial, InternalNonCommercial, Commercial, Operational }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermsReview {
    NotReviewed,
    Rejected { reviewer_id: String, reason: String },
    Approved { reviewer_id: String, approved_use: DataUse, terms_sha256: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactProvenance {
    pub source_uri: String,
    pub source_version: String,
    /// Claimed SHA-256 of exact source bytes; verification occurs outside this crate.
    pub artifact_sha256: String,
    pub retrieved_at_utc: String,
    pub license_id: String,
    pub attribution: String,
    pub terms_review: TermsReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationKind { RemoteSensingPlume, InSituMeasurement, InventoryEstimate, ModelInference, SyntheticScenario }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GasSpecies { Methane, CarbonDioxide, Other(String) }

#[derive(Clone, Debug, PartialEq)]
pub enum Quantification {
    Quantified { emission_rate_kg_h: f64, uncertainty_kg_h: f64, detection_limit_kg_h: f64 },
    /// A non-detection is not an observation of zero emissions.
    NonDetect { detection_limit_kg_h: f64 },
    /// Preserve unquantified values as unknown; never coerce them to zero.
    NotQuantified { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QualityFlag { Qualified, Degraded { reason: String }, Quarantined { reason: String } }

#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    /// Unique acquisition/observation identity, not a source or facility identity.
    pub observation_id: String,
    pub kind: ObservationKind,
    pub gas_species: GasSpecies,
    pub observed_at_utc: String,
    pub quality: QualityFlag,
    pub quantification: Quantification,
    pub provenance: ArtifactProvenance,
}

/// Newtypes stop a source identity being passed where a facility identity is expected.
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
    /// Optional facility match is separate from source association.
    pub facility_id: Option<FacilityId>,
    pub method: String,
    /// Score in [0, 1]; not a calibrated probability unless separately validated.
    pub match_score: f64,
    pub review: AssociationReview,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation { pub record_id: String, pub field: String, pub code: &'static str }

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport { pub violations: Vec<Violation> }

impl ValidationReport {
    pub fn is_valid(&self) -> bool { self.violations.is_empty() }
    fn push(&mut self, record_id: &str, field: &str, code: &'static str) {
        self.violations.push(Violation { record_id: record_id.to_owned(), field: field.to_owned(), code });
    }
}

/// Validates the entire batch; callers must reject the batch if any violation exists.
/// The function deliberately does not silently filter invalid rows.
pub fn validate_batch(observations: &[Observation], associations: &[SourceAssociation], required_use: DataUse, required_gas: &GasSpecies) -> ValidationReport {
    let mut report = ValidationReport::default();
    let mut observation_ids = HashSet::new();

    for obs in observations {
        let id = obs.observation_id.as_str();
        if id.trim().is_empty() { report.push(id, "observation_id", "empty_id"); }
        if !observation_ids.insert(id.to_owned()) { report.push(id, "observation_id", "duplicate_observation_id"); }
        if !is_utc_timestamp(&obs.observed_at_utc) { report.push(id, "observed_at_utc", "invalid_utc_timestamp"); }
        if &obs.gas_species != required_gas { report.push(id, "gas_species", "gas_species_mismatch"); }
        validate_provenance(&mut report, id, &obs.provenance, required_use);

        match &obs.quality {
            QualityFlag::Qualified => {}
            QualityFlag::Degraded { reason } if !reason.trim().is_empty() => {}
            QualityFlag::Degraded { .. } => report.push(id, "quality.reason", "empty_quality_reason"),
            QualityFlag::Quarantined { reason } => {
                if reason.trim().is_empty() { report.push(id, "quality.reason", "empty_quality_reason"); }
                report.push(id, "quality", "quarantined_observation");
            }
        }

        match &obs.quantification {
            Quantification::Quantified { emission_rate_kg_h, uncertainty_kg_h, detection_limit_kg_h } => {
                if !emission_rate_kg_h.is_finite() || *emission_rate_kg_h <= 0.0 {
                    report.push(id, "quantification.emission_rate_kg_h", "invalid_emission_rate");
                }
                if !uncertainty_kg_h.is_finite() || *uncertainty_kg_h < 0.0 {
                    report.push(id, "quantification.uncertainty_kg_h", "invalid_uncertainty");
                }
                if !detection_limit_kg_h.is_finite() || *detection_limit_kg_h <= 0.0 {
                    report.push(id, "quantification.detection_limit_kg_h", "invalid_detection_limit");
                }
            }
            Quantification::NonDetect { detection_limit_kg_h } => {
                if !detection_limit_kg_h.is_finite() || *detection_limit_kg_h <= 0.0 {
                    report.push(id, "quantification.detection_limit_kg_h", "invalid_detection_limit");
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
        if id.trim().is_empty() { report.push(id, "association_id", "empty_id"); }
        if !association_ids.insert(id.to_owned()) { report.push(id, "association_id", "duplicate_association_id"); }
        if !observation_ids.contains(&assoc.observation_id) { report.push(id, "observation_id", "unknown_observation_reference"); }
        if assoc.source_id.0.trim().is_empty() { report.push(id, "source_id", "empty_source_id"); }
        if assoc.facility_id.as_ref().is_some_and(|v| v.0.trim().is_empty()) {
            report.push(id, "facility_id", "empty_optional_facility_id");
        }
        if assoc.method.trim().is_empty() { report.push(id, "method", "empty_association_method"); }
        if !assoc.match_score.is_finite() || !(0.0..=1.0).contains(&assoc.match_score) {
            report.push(id, "match_score", "invalid_match_score");
        }
        match &assoc.review {
            AssociationReview::Unreviewed => {}
            AssociationReview::Reviewed { reviewer_id } if reviewer_id.trim().is_empty() => {
                report.push(id, "review.reviewer_id", "empty_reviewer_id");
            }
            AssociationReview::Rejected { reviewer_id, reason } => {
                if reviewer_id.trim().is_empty() { report.push(id, "review.reviewer_id", "empty_reviewer_id"); }
                if reason.trim().is_empty() { report.push(id, "review.reason", "empty_review_reason"); }
            }
            AssociationReview::Reviewed { .. } => {}
        }
    }

    report.violations.sort_by(|a, b| (&a.record_id, &a.field, a.code).cmp(&(&b.record_id, &b.field, b.code)));
    report
}

fn validate_provenance(report: &mut ValidationReport, record_id: &str, p: &ArtifactProvenance, required_use: DataUse) {
    if !p.source_uri.starts_with("https://") { report.push(record_id, "provenance.source_uri", "source_uri_not_https"); }
    if p.source_version.trim().is_empty() { report.push(record_id, "provenance.source_version", "empty_source_version"); }
    if !is_sha256_hex(&p.artifact_sha256) { report.push(record_id, "provenance.artifact_sha256", "invalid_sha256_format"); }
    if !is_utc_timestamp(&p.retrieved_at_utc) { report.push(record_id, "provenance.retrieved_at_utc", "invalid_utc_timestamp"); }
    if p.license_id.trim().is_empty() { report.push(record_id, "provenance.license_id", "missing_license_id"); }
    if p.attribution.trim().is_empty() { report.push(record_id, "provenance.attribution", "missing_attribution"); }

    match &p.terms_review {
        TermsReview::NotReviewed => report.push(record_id, "provenance.terms_review", "terms_not_reviewed"),
        TermsReview::Rejected { reviewer_id, reason } => {
            if reviewer_id.trim().is_empty() { report.push(record_id, "provenance.terms_review.reviewer_id", "empty_reviewer_id"); }
            if reason.trim().is_empty() { report.push(record_id, "provenance.terms_review.reason", "empty_review_reason"); }
            report.push(record_id, "provenance.terms_review", "terms_rejected");
        }
        TermsReview::Approved { reviewer_id, approved_use, terms_sha256 } => {
            if reviewer_id.trim().is_empty() { report.push(record_id, "provenance.terms_review.reviewer_id", "empty_reviewer_id"); }
            if *approved_use != required_use { report.push(record_id, "provenance.terms_review.approved_use", "declared_use_mismatch"); }
            if !is_sha256_hex(terms_sha256) { report.push(record_id, "provenance.terms_review.terms_sha256", "invalid_sha256_format"); }
        }
    }
}

fn is_sha256_hex(value: &str) -> bool { value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) }

/// Strict UTC seconds form: YYYY-MM-DDTHH:MM:SSZ. Adapters normalize offsets/fractions.
pub fn is_utc_timestamp(value: &str) -> bool {
    let b = value.as_bytes();
    if b.len() != 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T'
        || b[13] != b':' || b[16] != b':' || b[19] != b'Z' { return false; }
    let number = |range: std::ops::Range<usize>| -> Option<u32> {
        let bytes = &b[range];
        if bytes.iter().all(|digit| digit.is_ascii_digit()) {
            std::str::from_utf8(bytes).ok()?.parse().ok()
        } else { None }
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) =
        (number(0..4), number(5..7), number(8..10), number(11..13), number(14..16), number(17..19))
    else { return false };
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 { return false; }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=month_days).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> ArtifactProvenance {
        ArtifactProvenance {
            source_uri: "https://data.example.test/methane.csv".into(),
            source_version: "fixture-v1".into(),
            artifact_sha256: "a".repeat(64),
            retrieved_at_utc: "2026-10-19T12:30:00Z".into(),
            license_id: "CC-BY-4.0".into(),
            attribution: "Synthetic fixture provider".into(),
            terms_review: TermsReview::Approved {
                reviewer_id: "reviewer-1".into(),
                approved_use: DataUse::ResearchNonCommercial,
                terms_sha256: "b".repeat(64),
            },
        }
    }

    fn observation(id: &str, quantification: Quantification) -> Observation {
        Observation {
            observation_id: id.into(),
            kind: ObservationKind::RemoteSensingPlume,
            gas_species: GasSpecies::Methane,
            observed_at_utc: "2026-10-18T10:11:12Z".into(),
            quality: QualityFlag::Qualified,
            quantification,
            provenance: provenance(),
        }
    }

    fn quantified() -> Quantification {
        Quantification::Quantified { emission_rate_kg_h: 12.0, uncertainty_kg_h: 2.5, detection_limit_kg_h: 1.0 }
    }

    fn association(observation_id: &str) -> SourceAssociation {
        SourceAssociation {
            association_id: "assoc-1".into(),
            observation_id: observation_id.into(),
            source_id: SourceId("source-7".into()),
            facility_id: None,
            method: "reviewed-geospatial-match-v1".into(),
            match_score: 0.8,
            review: AssociationReview::Reviewed { reviewer_id: "reviewer-2".into() },
        }
    }

    #[test]
    fn accepts_well_formed_observation_and_separate_source_association() {
        let report = validate_batch(&[observation("plume-1", quantified())], &[association("plume-1")], DataUse::ResearchNonCommercial, &GasSpecies::Methane);
        assert!(report.is_valid(), "{report:?}");
    }

    #[test]
    fn rejects_cross_species_rate_mixing() {
        let mut obs = observation("co2-1", quantified());
        obs.gas_species = GasSpecies::CarbonDioxide;
        let report = validate_batch(&[obs], &[], DataUse::ResearchNonCommercial, &GasSpecies::Methane);
        assert!(report.violations.iter().any(|v| v.code == "gas_species_mismatch"));
    }

    #[test]
    fn rejects_duplicate_observation_ids_instead_of_double_counting() {
        let report = validate_batch(&[observation("plume-1", quantified()), observation("plume-1", quantified())], &[], DataUse::ResearchNonCommercial);
        assert!(report.violations.iter().any(|v| v.code == "duplicate_observation_id"));
    }

    #[test]
    fn non_detection_is_explicit_not_zero() {
        let report = validate_batch(&[observation("plume-2", Quantification::NonDetect { detection_limit_kg_h: 0.2 })], &[], DataUse::ResearchNonCommercial);
        assert!(report.is_valid(), "{report:?}");
        assert!(matches!(observation("x", Quantification::NonDetect { detection_limit_kg_h: 0.2 }).quantification, Quantification::NonDetect { .. }));
    }

    #[test]
    fn rejects_negative_non_finite_and_zero_detected_rates() {
        for rate in [-1.0, 0.0, f64::NAN, f64::INFINITY] {
            let q = Quantification::Quantified { emission_rate_kg_h: rate, uncertainty_kg_h: 0.1, detection_limit_kg_h: 0.1 };
            let report = validate_batch(&[observation("plume-x", q)], &[], DataUse::ResearchNonCommercial);
            assert!(report.violations.iter().any(|v| v.code == "invalid_emission_rate"));
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
        let report = validate_batch(&[obs], &[], DataUse::ResearchNonCommercial);
        assert!(report.violations.iter().any(|v| v.code == "invalid_utc_timestamp"));
    }

    #[test]
    fn refuses_unreviewed_or_mismatched_data_use() {
        let mut obs = observation("plume-4", quantified());
        obs.provenance.terms_review = TermsReview::NotReviewed;
        let report = validate_batch(&[obs.clone()], &[], DataUse::ResearchNonCommercial);
        assert!(report.violations.iter().any(|v| v.code == "terms_not_reviewed"));
        let report = validate_batch(&[obs], &[], DataUse::Commercial, &GasSpecies::Methane);
        assert!(report.violations.iter().any(|v| v.code == "terms_not_reviewed"));
        let mut commercial = observation("plume-4b", quantified());
        commercial.provenance.terms_review = TermsReview::Approved {
            reviewer_id: "reviewer-1".into(), approved_use: DataUse::ResearchNonCommercial, terms_sha256: "b".repeat(64),
        };
        let report = validate_batch(&[commercial], &[], DataUse::Commercial);
        assert!(report.violations.iter().any(|v| v.code == "declared_use_mismatch"));
    }

    #[test]
    fn rejects_malformed_digest_and_missing_attribution() {
        let mut obs = observation("plume-5", quantified());
        obs.provenance.artifact_sha256 = "not-a-digest".into();
        obs.provenance.attribution.clear();
        let report = validate_batch(&[obs], &[], DataUse::ResearchNonCommercial);
        assert!(report.violations.iter().any(|v| v.code == "invalid_sha256_format"));
        assert!(report.violations.iter().any(|v| v.code == "missing_attribution"));
    }

    #[test]
    fn rejects_unknown_observation_reference_and_invalid_association_score() {
        let mut assoc = association("missing-plume");
        assoc.match_score = 1.2;
        let report = validate_batch(&[], &[assoc], DataUse::ResearchNonCommercial);
        assert!(report.violations.iter().any(|v| v.code == "unknown_observation_reference"));
        assert!(report.violations.iter().any(|v| v.code == "invalid_match_score"));
    }

    #[test]
    fn preserves_unquantified_as_explicit_unknown() {
        let obs = observation("plume-6", Quantification::NotQuantified { reason: "retrieval quality below threshold".into() });
        assert!(validate_batch(&[obs], &[], DataUse::ResearchNonCommercial).is_valid());
    }

    #[test]
    fn quarantined_observation_blocks_batch_admission() {
        let mut obs = observation("plume-7", quantified());
        obs.quality = QualityFlag::Quarantined { reason: "stale calibration".into() };
        let report = validate_batch(&[obs], &[], DataUse::ResearchNonCommercial);
        assert!(report.violations.iter().any(|v| v.code == "quarantined_observation"));
    }

    #[test]
    fn validation_report_order_is_deterministic() {
        let mut obs = observation("plume-8", quantified());
        obs.provenance.attribution.clear();
        obs.observed_at_utc = "bad".into();
        let first = validate_batch(&[obs.clone()], &[], DataUse::ResearchNonCommercial);
        let second = validate_batch(&[obs], &[], DataUse::ResearchNonCommercial);
        assert_eq!(first, second);
        assert!(first.violations.windows(2).all(|w| (&w[0].record_id, &w[0].field, w[0].code) <= (&w[1].record_id, &w[1].field, w[1].code)));
    }
}