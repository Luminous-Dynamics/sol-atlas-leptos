// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Narrow CCZ Commons semantic and publication boundary.
//!
//! This module models source-stated records, not legal conclusions. Geometry
//! remains owned by the existing Sol Atlas spatial layer. Provenance points to
//! exact source snapshots rather than introducing a second provenance graph.
//! Passing validation is not legal, scientific, or navigation qualification.

use serde::{Deserialize, Serialize};

/// Source-record class. These variants must remain semantically distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CczRecordKind {
    IsaExplorationContract,
    IsaReservedArea,
    AreaOfParticularEnvironmentalInterest,
    NationalMaritimeZone,
    DomesticLawApplication,
    PublicPermitOrOrder,
    ScientificSurveyFootprint,
    ScientificObservation,
    CorporateResourceAssertion,
    Unknown,
}

/// Status reported by the source instrument or publisher, not adjudicated by
/// Sol Atlas. InForce is not equivalent to a mining approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatedStatus {
    NotStated,
    Proposed,
    Issued,
    InForce,
    Expired,
    Revoked,
    Withdrawn,
    Superseded,
    Disputed,
    Unknown,
}

/// Rights review for a source snapshot. Unknown/restricted terms fail closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTermsState {
    OpenRedistributionReviewed,
    MetadataCitationOnlyReviewed,
    ReviewRequired,
    Restricted,
}

/// Maximum publication scope supported by a reviewed source snapshot.
/// OpenDataAndMetadata requires reviewed redistribution terms and an exact
/// artifact digest. A URL alone is not a frozen data artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationScope {
    OpenDataAndMetadata,
    MetadataOnly,
    Blocked,
}

/// An immutable reference to one version of a source or data product.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CczSourceSnapshot {
    pub snapshot_id: String,
    pub source_name: String,
    pub source_url: String,
    /// Retrieval instant, expected to be recorded as an unambiguous timestamp.
    pub retrieved_at: String,
    pub source_version: Option<String>,
    /// SHA-256 of the exact archived bytes, represented as 64 hex characters.
    pub artifact_sha256: Option<String>,
    pub terms_state: SourceTermsState,
    /// Exact license/terms identifier or URL reviewed by a human.
    pub license_or_terms: Option<String>,
    /// Required attribution wording where applicable.
    pub attribution_text: Option<String>,
    /// Stable reference to the terms-review record; not a proof of legal validity.
    pub terms_review_id: Option<String>,
    /// Reviewer or organization identity recorded for the review.
    pub terms_reviewed_by: Option<String>,
    /// Timestamp of the terms review, separate from source retrieval time.
    pub terms_reviewed_at: Option<String>,
}

/// One CCZ feature/record that references a source snapshot.
///
/// This intentionally has no owner or sovereign-rights field: source issuer,
/// instrument class and source-stated status are not legal determinations.
/// Geometry remains in the existing spatial owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CczFeatureRecord {
    pub feature_id: String,
    pub kind: CczRecordKind,
    pub source_snapshot_id: String,
    pub issuer_as_published: Option<String>,
    pub instrument_reference: Option<String>,
    pub status_as_published: SourceStatedStatus,
    /// Inclusive source-claimed validity start, ISO date YYYY-MM-DD.
    pub valid_from: Option<String>,
    /// Source-defined validity end, ISO date YYYY-MM-DD.
    pub valid_until: Option<String>,
    /// SHA-256 of the canonical geometry artifact, if geometry is supplied.
    pub geometry_sha256: Option<String>,
    /// Coordinate reference identifier, e.g. an EPSG code. Never inferred.
    pub coordinate_reference_system: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCode {
    MissingSnapshotId,
    MissingSourceName,
    MissingSourceUrl,
    MissingRetrievalTime,
    MissingLicenseTerms,
    MissingAttribution,
    MissingTermsReviewEvidence,
    InvalidSha256,
    ArtifactNotPinned,
    TermsNotReviewed,
    RestrictedSource,
    MissingFeatureId,
    MissingSourceReference,
    SourceReferenceMismatch,
    MissingIssuer,
    InvalidDate,
    ValidityIntervalInverted,
    GeometryCrsMismatch,
    EmptyCoordinateReferenceSystem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub code: ValidationCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceValidation {
    pub publication_scope: PublicationScope,
    pub issues: Vec<ValidationIssue>,
}

impl SourceValidation {
    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|issue| issue.severity == ValidationSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureValidation {
    pub publication_scope: PublicationScope,
    pub issues: Vec<ValidationIssue>,
}

impl FeatureValidation {
    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|issue| issue.severity == ValidationSeverity::Error)
    }
}

fn issue(
    severity: ValidationSeverity,
    code: ValidationCode,
    message: impl Into<String>,
) -> ValidationIssue {
    ValidationIssue { severity, code, message: message.into() }
}

fn present(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn valid_iso_date(value: &str) -> Option<(i32, u32, u32)> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if !bytes.iter().enumerate().all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit()) {
        return None;
    }
    let year = value[0..4].parse::<i32>().ok()?;
    let month = value[5..7].parse::<u32>().ok()?;
    let day = value[8..10].parse::<u32>().ok()?;
    if year < 1 || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max_day).contains(&day).then_some((year, month, day))
}

/// Determine publication scope without guessing rights or snapshot identity.
pub fn publication_scope(source: &CczSourceSnapshot) -> PublicationScope {
    if !present(&source.snapshot_id)
        || !present(&source.source_name)
        || !present(&source.source_url)
        || !present(&source.retrieved_at)
    {
        return PublicationScope::Blocked;
    }

    let terms_are_reviewed = source.license_or_terms.as_deref().is_some_and(present)
        && source.attribution_text.as_deref().is_some_and(present)
        && source.terms_review_id.as_deref().is_some_and(present)
        && source.terms_reviewed_by.as_deref().is_some_and(present)
        && source.terms_reviewed_at.as_deref().is_some_and(present);
    match source.terms_state {
        SourceTermsState::OpenRedistributionReviewed if !terms_are_reviewed => {
            PublicationScope::Blocked
        }
        SourceTermsState::OpenRedistributionReviewed => match source.artifact_sha256.as_deref() {
            Some(digest) if valid_sha256(digest) => PublicationScope::OpenDataAndMetadata,
            None => PublicationScope::MetadataOnly,
            Some(_) => PublicationScope::Blocked,
        },
        SourceTermsState::MetadataCitationOnlyReviewed if terms_are_reviewed => {
            PublicationScope::MetadataOnly
        }
        SourceTermsState::MetadataCitationOnlyReviewed => PublicationScope::Blocked,
        SourceTermsState::ReviewRequired | SourceTermsState::Restricted => PublicationScope::Blocked,
    }
}

/// Validate snapshot metadata, not publisher truth, scientific accuracy, or
/// license interpretation.
pub fn validate_source_snapshot(source: &CczSourceSnapshot) -> SourceValidation {
    let mut issues = Vec::new();

    for (value, code, label) in [
        (source.snapshot_id.as_str(), ValidationCode::MissingSnapshotId, "snapshot_id"),
        (source.source_name.as_str(), ValidationCode::MissingSourceName, "source_name"),
        (source.source_url.as_str(), ValidationCode::MissingSourceUrl, "source_url"),
        (source.retrieved_at.as_str(), ValidationCode::MissingRetrievalTime, "retrieved_at"),
    ] {
        if !present(value) {
            issues.push(issue(ValidationSeverity::Error, code, format!("{label} must be non-empty")));
        }
    }

    if let Some(digest) = source.artifact_sha256.as_deref() {
        if !valid_sha256(digest) {
            issues.push(issue(
                ValidationSeverity::Error,
                ValidationCode::InvalidSha256,
                "artifact_sha256 must contain exactly 64 hexadecimal characters",
            ));
        }
    } else {
        issues.push(issue(
            ValidationSeverity::Warning,
            ValidationCode::ArtifactNotPinned,
            "no artifact digest is present; exact-byte reproduction is not established",
        ));
    }

    match source.terms_state {
        SourceTermsState::OpenRedistributionReviewed
        | SourceTermsState::MetadataCitationOnlyReviewed => {
            if !source.license_or_terms.as_deref().is_some_and(present) {
                issues.push(issue(
                    ValidationSeverity::Error,
                    ValidationCode::MissingLicenseTerms,
                    "reviewed publication terms must identify the license or terms source",
                ));
            }
            if !source.attribution_text.as_deref().is_some_and(present) {
                issues.push(issue(
                    ValidationSeverity::Error,
                    ValidationCode::MissingAttribution,
                    "reviewed publication terms must provide attribution text",
                ));
            }
            if !source.terms_review_id.as_deref().is_some_and(present)
                || !source.terms_reviewed_by.as_deref().is_some_and(present)
                || !source.terms_reviewed_at.as_deref().is_some_and(present)
            {
                issues.push(issue(
                    ValidationSeverity::Error,
                    ValidationCode::MissingTermsReviewEvidence,
                    "reviewed terms require a review record ID, reviewer identity, and review timestamp",
                ));
            }
        }
        SourceTermsState::ReviewRequired => issues.push(issue(
            ValidationSeverity::Warning,
            ValidationCode::TermsNotReviewed,
            "source terms are not reviewed; publication is blocked",
        )),
        SourceTermsState::Restricted => issues.push(issue(
            ValidationSeverity::Warning,
            ValidationCode::RestrictedSource,
            "source is marked restricted; publication is blocked",
        )),
    }

    SourceValidation { publication_scope: publication_scope(source), issues }
}

/// Validate a feature against its exact source snapshot. Missing/mismatched
/// snapshots block publication; no nearest-version fallback is permitted.
pub fn validate_feature(
    feature: &CczFeatureRecord,
    source: Option<&CczSourceSnapshot>,
) -> FeatureValidation {
    let mut issues = Vec::new();
    if !present(&feature.feature_id) {
        issues.push(issue(
            ValidationSeverity::Error,
            ValidationCode::MissingFeatureId,
            "feature_id must be non-empty",
        ));
    }
    if !present(&feature.source_snapshot_id) {
        issues.push(issue(
            ValidationSeverity::Error,
            ValidationCode::MissingSourceReference,
            "source_snapshot_id must be non-empty",
        ));
    }
    if feature.issuer_as_published.as_deref().is_none_or(|s| !present(s)) {
        issues.push(issue(
            ValidationSeverity::Warning,
            ValidationCode::MissingIssuer,
            "issuer is unknown; retain the record as unresolved rather than inferring one",
        ));
    }

    let start = feature.valid_from.as_deref().map(valid_iso_date);
    let end = feature.valid_until.as_deref().map(valid_iso_date);
    if start.as_ref().is_some_and(|date| date.is_none())
        || end.as_ref().is_some_and(|date| date.is_none())
    {
        issues.push(issue(
            ValidationSeverity::Error,
            ValidationCode::InvalidDate,
            "valid_from and valid_until must use real ISO calendar dates (YYYY-MM-DD)",
        ));
    }
    if let (Some(Some(start)), Some(Some(end))) = (start, end) {
        if start >= end {
            issues.push(issue(
                ValidationSeverity::Error,
                ValidationCode::ValidityIntervalInverted,
                "valid_from must be earlier than valid_until",
            ));
        }
    }

    match (
        feature.geometry_sha256.as_deref(),
        feature.coordinate_reference_system.as_deref(),
    ) {
        (Some(digest), Some(crs)) => {
            if !valid_sha256(digest) {
                issues.push(issue(
                    ValidationSeverity::Error,
                    ValidationCode::InvalidSha256,
                    "geometry_sha256 must contain exactly 64 hexadecimal characters",
                ));
            }
            if !present(crs) {
                issues.push(issue(
                    ValidationSeverity::Error,
                    ValidationCode::EmptyCoordinateReferenceSystem,
                    "coordinate_reference_system must be non-empty when geometry is supplied",
                ));
            }
        }
        (None, None) => {}
        _ => issues.push(issue(
            ValidationSeverity::Error,
            ValidationCode::GeometryCrsMismatch,
            "geometry digest and coordinate reference must be both present or both absent",
        )),
    }

    let scope = match source {
        Some(source) if feature.source_snapshot_id == source.snapshot_id => {
            let report = validate_source_snapshot(source);
            issues.extend(report.issues);
            report.publication_scope
        }
        Some(_) => {
            issues.push(issue(
                ValidationSeverity::Error,
                ValidationCode::SourceReferenceMismatch,
                "feature source reference does not match the supplied exact snapshot",
            ));
            PublicationScope::Blocked
        }
        None => {
            issues.push(issue(
                ValidationSeverity::Error,
                ValidationCode::MissingSourceReference,
                "referenced source snapshot is unavailable; no fallback is permitted",
            ));
            PublicationScope::Blocked
        }
    };

    FeatureValidation { publication_scope: scope, issues }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(terms_state: SourceTermsState, digest: Option<&str>) -> CczSourceSnapshot {
        CczSourceSnapshot {
            snapshot_id: "isa-snapshot-2026-01".into(),
            source_name: "Synthetic ISA fixture".into(),
            source_url: "https://example.invalid/isa-fixture".into(),
            retrieved_at: "2026-10-18T00:00:00Z".into(),
            source_version: Some("fixture-v1".into()),
            artifact_sha256: digest.map(str::to_owned),
            terms_state,
            license_or_terms: Some("https://example.invalid/terms".into()),
            attribution_text: Some("Synthetic fixture only".into()),
            terms_review_id: Some("synthetic-terms-review-01".into()),
            terms_reviewed_by: Some("synthetic-reviewer".into()),
            terms_reviewed_at: Some("2026-10-18T00:00:00Z".into()),
        }
    }

    fn feature(kind: CczRecordKind) -> CczFeatureRecord {
        CczFeatureRecord {
            feature_id: "ccz-fixture-1".into(),
            kind,
            source_snapshot_id: "isa-snapshot-2026-01".into(),
            issuer_as_published: Some("Synthetic issuer".into()),
            instrument_reference: Some("fixture-1".into()),
            status_as_published: SourceStatedStatus::InForce,
            valid_from: Some("2026-01-01".into()),
            valid_until: Some("2027-01-01".into()),
            geometry_sha256: Some("a".repeat(64)),
            coordinate_reference_system: Some("EPSG:4326".into()),
        }
    }

    #[test]
    fn legal_and_scientific_record_classes_are_distinct_in_serde() {
        let contract = serde_json::to_string(&CczRecordKind::IsaExplorationContract).unwrap();
        let apei = serde_json::to_string(&CczRecordKind::AreaOfParticularEnvironmentalInterest).unwrap();
        let national_zone = serde_json::to_string(&CczRecordKind::NationalMaritimeZone).unwrap();
        let corporate_claim = serde_json::to_string(&CczRecordKind::CorporateResourceAssertion).unwrap();
        assert_eq!(contract, "\"isa_exploration_contract\"");
        assert_eq!(apei, "\"area_of_particular_environmental_interest\"");
        assert_eq!(national_zone, "\"national_maritime_zone\"");
        assert_eq!(corporate_claim, "\"corporate_resource_assertion\"");
        assert_ne!(contract, national_zone);
    }

    #[test]
    fn open_source_with_exact_digest_terms_and_attribution_allows_data_publication() {
        let digest = "a".repeat(64);
        let snapshot = source(SourceTermsState::OpenRedistributionReviewed, Some(&digest));
        assert_eq!(publication_scope(&snapshot), PublicationScope::OpenDataAndMetadata);
        assert!(validate_source_snapshot(&snapshot).is_valid());
    }

    #[test]
    fn missing_digest_never_claims_reproducible_public_data() {
        let snapshot = source(SourceTermsState::OpenRedistributionReviewed, None);
        let report = validate_source_snapshot(&snapshot);
        assert_eq!(report.publication_scope, PublicationScope::MetadataOnly);
        assert!(report.issues.iter().any(|i| i.code == ValidationCode::ArtifactNotPinned));
        assert!(report.is_valid(), "missing digest is a warning, not evidence that bytes are corrupt");
    }

    #[test]
    fn missing_license_or_attribution_blocks_publication() {
        let digest = "a".repeat(64);
        let mut snapshot = source(SourceTermsState::OpenRedistributionReviewed, Some(&digest));
        snapshot.attribution_text = None;
        assert_eq!(publication_scope(&snapshot), PublicationScope::Blocked);
        snapshot.attribution_text = Some("Reviewed attribution".into());
        snapshot.license_or_terms = None;
        assert_eq!(publication_scope(&snapshot), PublicationScope::Blocked);
    }

    #[test]
    fn reviewed_terms_require_review_record_reviewer_and_timestamp() {
        let digest = "a".repeat(64);
        let mut snapshot = source(SourceTermsState::OpenRedistributionReviewed, Some(&digest));
        snapshot.terms_review_id = None;
        let report = validate_source_snapshot(&snapshot);
        assert_eq!(report.publication_scope, PublicationScope::Blocked);
        assert!(report.issues.iter().any(|i| i.code == ValidationCode::MissingTermsReviewEvidence));
    }

    #[test]
    fn unknown_terms_fail_closed() {
        let digest = "a".repeat(64);
        let snapshot = source(SourceTermsState::ReviewRequired, Some(&digest));
        let report = validate_source_snapshot(&snapshot);
        assert_eq!(report.publication_scope, PublicationScope::Blocked);
        assert!(report.issues.iter().any(|i| i.code == ValidationCode::TermsNotReviewed));
    }

    #[test]
    fn restricted_terms_fail_closed() {
        let digest = "a".repeat(64);
        let snapshot = source(SourceTermsState::Restricted, Some(&digest));
        assert_eq!(publication_scope(&snapshot), PublicationScope::Blocked);
    }

    #[test]
    fn citation_only_source_allows_metadata_not_artifact_redistribution() {
        let digest = "a".repeat(64);
        let snapshot = source(SourceTermsState::MetadataCitationOnlyReviewed, Some(&digest));
        assert_eq!(publication_scope(&snapshot), PublicationScope::MetadataOnly);
    }

    #[test]
    fn feature_requires_the_exact_source_snapshot() {
        let digest = "a".repeat(64);
        let snapshot = source(SourceTermsState::OpenRedistributionReviewed, Some(&digest));
        let mut record = feature(CczRecordKind::IsaExplorationContract);
        record.source_snapshot_id = "older-or-different-snapshot".into();
        let report = validate_feature(&record, Some(&snapshot));
        assert_eq!(report.publication_scope, PublicationScope::Blocked);
        assert!(report.issues.iter().any(|i| i.code == ValidationCode::SourceReferenceMismatch));
    }

    #[test]
    fn feature_without_snapshot_is_blocked() {
        let report = validate_feature(&feature(CczRecordKind::ScientificObservation), None);
        assert_eq!(report.publication_scope, PublicationScope::Blocked);
        assert!(report.issues.iter().any(|i| i.code == ValidationCode::MissingSourceReference));
    }

    #[test]
    fn geometry_digest_and_crs_must_be_paired() {
        let snapshot = source(SourceTermsState::MetadataCitationOnlyReviewed, None);
        let mut record = feature(CczRecordKind::IsaReservedArea);
        record.coordinate_reference_system = None;
        let report = validate_feature(&record, Some(&snapshot));
        assert!(report.issues.iter().any(|i| i.code == ValidationCode::GeometryCrsMismatch));
    }

    #[test]
    fn impossible_or_inverted_validity_dates_are_rejected() {
        let snapshot = source(SourceTermsState::MetadataCitationOnlyReviewed, None);
        let mut record = feature(CczRecordKind::PublicPermitOrOrder);
        record.valid_from = Some("2025-02-29".into());
        let invalid = validate_feature(&record, Some(&snapshot));
        assert!(invalid.issues.iter().any(|i| i.code == ValidationCode::InvalidDate));

        record.valid_from = Some("2027-01-01".into());
        record.valid_until = Some("2026-01-01".into());
        let inverted = validate_feature(&record, Some(&snapshot));
        assert!(inverted.issues.iter().any(|i| i.code == ValidationCode::ValidityIntervalInverted));
    }

    #[test]
    fn synthetic_fixture_corpus_preserves_distinct_record_classes() {
        let records: Vec<CczFeatureRecord> = serde_json::from_str(include_str!(
            "../tests/fixtures/ccz/semantic-records.json"
        ))
        .unwrap();
        assert_eq!(records.len(), 8);

        let expected = vec![
            CczRecordKind::IsaExplorationContract,
            CczRecordKind::IsaReservedArea,
            CczRecordKind::AreaOfParticularEnvironmentalInterest,
            CczRecordKind::NationalMaritimeZone,
            CczRecordKind::DomesticLawApplication,
            CczRecordKind::CorporateResourceAssertion,
            CczRecordKind::ScientificObservation,
            CczRecordKind::Unknown,
        ];
        assert_eq!(records.iter().map(|record| record.kind).collect::<Vec<_>>(), expected);

        let digest = "e".repeat(64);
        let mut snapshot = source(SourceTermsState::OpenRedistributionReviewed, Some(&digest));
        snapshot.snapshot_id = "synthetic-source-v1".into();
        for record in &records {
            let report = validate_feature(record, Some(&snapshot));
            assert!(report.is_valid(), "fixture {} failed: {:?}", record.feature_id, report.issues);
            assert_eq!(report.publication_scope, PublicationScope::OpenDataAndMetadata);
        }
    }

    #[test]
    fn serde_rejects_unrecognized_owner_field_instead_of_silently_importing_it() {
        let json = r#"{
            "feature_id":"x",
            "kind":"isa_exploration_contract",
            "source_snapshot_id":"s",
            "issuer_as_published":"ISA",
            "instrument_reference":null,
            "status_as_published":"in_force",
            "valid_from":null,
            "valid_until":null,
            "geometry_sha256":null,
            "coordinate_reference_system":null,
            "owner":"country-x"
        }"#;
        assert!(serde_json::from_str::<CczFeatureRecord>(json).is_err());
    }
}
