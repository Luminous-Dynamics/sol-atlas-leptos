// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Licence-aware, versioned metadata for scientific dataset snapshots.
//!
//! This module models provenance metadata for exploration and reproducibility.
//! It does not decide whether a scientific proposition is true, whether a
//! source is trustworthy, or whether a dataset is suitable for clinical use.
//!
//! Parse untrusted JSON through `parse_snapshot_v1`, which validates the
//! invariants before returning the public, serializable snapshot type.

use serde::{Deserialize, Serialize};

pub const SCIENTIFIC_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
const MAX_FIELD_LEN: usize = 512;

/// The lifecycle of the actual dataset artifact, kept separate from metadata
/// about an announcement or landing page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCapture {
    /// Only source/release metadata was captured; dataset bytes are not present.
    MetadataOnly,
    /// A content digest is recorded for bytes claimed to have been captured.
    /// This is not proof that the bytes, digest, publisher, or source are authentic.
    DigestRecorded,
}

/// Licence-review state. A public URL or an accessible download does not by
/// itself grant permission to redistribute the content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    Known,
    Unknown,
    ReviewRequired,
}

/// Licence metadata for one specific scope (for example, the dataset rather
/// than the release-announcement webpage).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LicenseEvidence {
    pub status: LicenseStatus,
    pub identifier: Option<String>,
    pub terms_url: Option<String>,
}

/// What the referenced source record represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRecordKind {
    ReleaseAnnouncement,
    DatasetLandingPage,
    DatasetArtifact,
}

/// Versioned scientific dataset metadata. Fields are private so callers can
/// only obtain a validated instance through the parsing/constructor functions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScientificDatasetSnapshotV1 {
    schema_version: u16,
    snapshot_id: String,
    source_provider: String,
    source_record_id: String,
    source_record_kind: SourceRecordKind,
    source_record_url: String,
    source_record_license: LicenseEvidence,
    dataset_name: String,
    dataset_release: String,
    dataset_release_date: Option<String>,
    dataset_landing_url: String,
    dataset_license: LicenseEvidence,
    retrieved_on: String,
    artifact_capture: ArtifactCapture,
    artifact_sha256: Option<String>,
}

impl ScientificDatasetSnapshotV1 {
    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    pub fn source_provider(&self) -> &str {
        &self.source_provider
    }

    pub fn source_record_id(&self) -> &str {
        &self.source_record_id
    }

    pub fn source_record_kind(&self) -> SourceRecordKind {
        self.source_record_kind
    }

    pub fn source_record_url(&self) -> &str {
        &self.source_record_url
    }

    pub fn source_record_license(&self) -> &LicenseEvidence {
        &self.source_record_license
    }

    pub fn dataset_name(&self) -> &str {
        &self.dataset_name
    }

    pub fn dataset_release(&self) -> &str {
        &self.dataset_release
    }

    pub fn dataset_release_date(&self) -> Option<&str> {
        self.dataset_release_date.as_deref()
    }

    pub fn dataset_landing_url(&self) -> &str {
        &self.dataset_landing_url
    }

    pub fn dataset_license(&self) -> &LicenseEvidence {
        &self.dataset_license
    }

    pub fn retrieved_on(&self) -> &str {
        &self.retrieved_on
    }

    pub fn artifact_capture(&self) -> ArtifactCapture {
        self.artifact_capture
    }

    pub fn artifact_sha256(&self) -> Option<&str> {
        self.artifact_sha256.as_deref()
    }
}

/// Stable, machine-readable validation findings for this schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotValidationIssue {
    UnsupportedSchemaVersion,
    InvalidSnapshotId,
    InvalidSourceProvider,
    InvalidSourceRecordId,
    InvalidSourceRecordUrl,
    InvalidSourceRecordLicense,
    InvalidDatasetName,
    InvalidDatasetRelease,
    InvalidDatasetReleaseDate,
    InvalidDatasetLandingUrl,
    InvalidDatasetLicense,
    InvalidRetrievedOn,
    ArtifactDigestMissingOrMalformed,
    UnexpectedDigestForMetadataOnly,
    ArtifactCaptureKindMismatch,
}

/// A parse failure distinguishes invalid JSON from structurally invalid metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotParseError {
    InvalidPayload(String),
    InvalidSnapshot(Vec<SnapshotValidationIssue>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScientificDatasetSnapshotV1 {
    schema_version: u16,
    snapshot_id: String,
    source_provider: String,
    source_record_id: String,
    source_record_kind: SourceRecordKind,
    source_record_url: String,
    source_record_license: LicenseEvidence,
    dataset_name: String,
    dataset_release: String,
    dataset_release_date: Option<String>,
    dataset_landing_url: String,
    dataset_license: LicenseEvidence,
    retrieved_on: String,
    artifact_capture: ArtifactCapture,
    artifact_sha256: Option<String>,
}

/// Deserialize and validate snapshot JSON in a single operation. Untrusted JSON
/// cannot create a public validated snapshot simply by declaring a status flag.
pub fn parse_snapshot_v1(
    json: &str,
) -> Result<ScientificDatasetSnapshotV1, SnapshotParseError> {
    let raw: RawScientificDatasetSnapshotV1 = serde_json::from_str(json)
        .map_err(|error| SnapshotParseError::InvalidPayload(error.to_string()))?;
    validate_raw(raw)
}

fn validate_raw(
    raw: RawScientificDatasetSnapshotV1,
) -> Result<ScientificDatasetSnapshotV1, SnapshotParseError> {
    let mut issues = Vec::new();

    if raw.schema_version != SCIENTIFIC_SNAPSHOT_SCHEMA_VERSION {
        issues.push(SnapshotValidationIssue::UnsupportedSchemaVersion);
    }
    if !is_identifier(&raw.snapshot_id) {
        issues.push(SnapshotValidationIssue::InvalidSnapshotId);
    }
    if !is_bounded_nonblank(&raw.source_provider) {
        issues.push(SnapshotValidationIssue::InvalidSourceProvider);
    }
    if !is_identifier(&raw.source_record_id) {
        issues.push(SnapshotValidationIssue::InvalidSourceRecordId);
    }
    if !is_https_url(&raw.source_record_url) {
        issues.push(SnapshotValidationIssue::InvalidSourceRecordUrl);
    }
    if !valid_license(&raw.source_record_license) {
        issues.push(SnapshotValidationIssue::InvalidSourceRecordLicense);
    }
    if !is_bounded_nonblank(&raw.dataset_name) {
        issues.push(SnapshotValidationIssue::InvalidDatasetName);
    }
    if !is_identifier(&raw.dataset_release) {
        issues.push(SnapshotValidationIssue::InvalidDatasetRelease);
    }
    if raw
        .dataset_release_date
        .as_deref()
        .is_some_and(|date| !is_iso_date(date))
    {
        issues.push(SnapshotValidationIssue::InvalidDatasetReleaseDate);
    }
    if !is_https_url(&raw.dataset_landing_url) {
        issues.push(SnapshotValidationIssue::InvalidDatasetLandingUrl);
    }
    if !valid_license(&raw.dataset_license) {
        issues.push(SnapshotValidationIssue::InvalidDatasetLicense);
    }
    if !is_iso_date(&raw.retrieved_on) {
        issues.push(SnapshotValidationIssue::InvalidRetrievedOn);
    }

    match (raw.artifact_capture, raw.artifact_sha256.as_deref()) {
        (ArtifactCapture::MetadataOnly, None) => {}
        (ArtifactCapture::MetadataOnly, Some(_)) => {
            issues.push(SnapshotValidationIssue::UnexpectedDigestForMetadataOnly);
        }
        (ArtifactCapture::DigestRecorded, Some(digest)) if is_sha256_hex(digest) => {}
        (ArtifactCapture::DigestRecorded, _) => {
            issues.push(SnapshotValidationIssue::ArtifactDigestMissingOrMalformed);
        }
    }

    let artifact_kind_mismatch = matches!(
        (raw.source_record_kind, raw.artifact_capture),
        (
            SourceRecordKind::ReleaseAnnouncement | SourceRecordKind::DatasetLandingPage,
            ArtifactCapture::DigestRecorded
        ) | (
            SourceRecordKind::DatasetArtifact,
            ArtifactCapture::MetadataOnly
        )
    );
    if artifact_kind_mismatch {
        issues.push(SnapshotValidationIssue::ArtifactCaptureKindMismatch);
    }

    if !issues.is_empty() {
        return Err(SnapshotParseError::InvalidSnapshot(issues));
    }

    Ok(ScientificDatasetSnapshotV1 {
        schema_version: raw.schema_version,
        snapshot_id: raw.snapshot_id,
        source_provider: raw.source_provider,
        source_record_id: raw.source_record_id,
        source_record_kind: raw.source_record_kind,
        source_record_url: raw.source_record_url,
        source_record_license: raw.source_record_license,
        dataset_name: raw.dataset_name,
        dataset_release: raw.dataset_release,
        dataset_release_date: raw.dataset_release_date,
        dataset_landing_url: raw.dataset_landing_url,
        dataset_license: raw.dataset_license,
        retrieved_on: raw.retrieved_on,
        artifact_capture: raw.artifact_capture,
        artifact_sha256: raw.artifact_sha256,
    })
}

fn is_bounded_nonblank(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_FIELD_LEN
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FIELD_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn is_https_url(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("https://") else {
        return false;
    };
    !rest.is_empty()
        && !rest.chars().any(char::is_whitespace)
        && rest.split('/').next().is_some_and(|host| host.contains('.'))
        && value.len() <= MAX_FIELD_LEN
}

fn valid_license(license: &LicenseEvidence) -> bool {
    let identifier_is_valid = license
        .identifier
        .as_deref()
        .is_some_and(is_bounded_nonblank);
    let terms_url_is_valid = license
        .terms_url
        .as_deref()
        .is_none_or(is_https_url);

    terms_url_is_valid
        && match license.status {
            LicenseStatus::Known => identifier_is_valid,
            LicenseStatus::Unknown | LicenseStatus::ReviewRequired => {
                license.identifier.is_none()
            }
        }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[0..4].iter().all(|byte| byte.is_ascii_digit())
        || !bytes[5..7].iter().all(|byte| byte.is_ascii_digit())
        || !bytes[8..10].iter().all(|byte| byte.is_ascii_digit())
    {
        return false;
    }
    // Numeric bytes are checked before slicing, so malformed UTF-8 boundaries
    // cannot panic validation on hostile input.
    let year = match std::str::from_utf8(&bytes[0..4])
        .ok()
        .and_then(|part| part.parse::<u32>().ok())
    {
        Some(value) => value,
        None => return false,
    };
    let month = match std::str::from_utf8(&bytes[5..7])
        .ok()
        .and_then(|part| part.parse::<u32>().ok())
    {
        Some(value) => value,
        None => return false,
    };
    let day = match std::str::from_utf8(&bytes[8..10])
        .ok()
        .and_then(|part| part.parse::<u32>().ok())
    {
        Some(value) => value,
        None => return false,
    };
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }

    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../fixtures/open-targets-platform-26-09.json");

    #[test]
    fn fixture_parses_as_metadata_only_without_fabricated_artifact_digest() {
        let snapshot = parse_snapshot_v1(FIXTURE).expect("fixture should validate");
        assert_eq!(snapshot.schema_version(), SCIENTIFIC_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(snapshot.dataset_release(), "26.09");
        assert_eq!(snapshot.artifact_capture(), ArtifactCapture::MetadataOnly);
        assert_eq!(snapshot.artifact_sha256(), None);
        assert_eq!(snapshot.dataset_license().status, LicenseStatus::Known);
        assert_eq!(snapshot.dataset_license().identifier.as_deref(), Some("CC0-1.0"));
        assert_eq!(
            snapshot.source_record_license().status,
            LicenseStatus::ReviewRequired
        );
    }

    #[test]
    fn serialize_then_parse_preserves_validated_snapshot() {
        let snapshot = parse_snapshot_v1(FIXTURE).expect("fixture should validate");
        let json = serde_json::to_string(&snapshot).expect("snapshot serializes");
        let reparsed = parse_snapshot_v1(&json).expect("serialized snapshot revalidates");
        assert_eq!(snapshot, reparsed);
    }

    #[test]
    fn unknown_source_record_license_is_not_promoted_to_known() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["source_record_license"]["status"] = serde_json::json!("unknown");
        let snapshot = parse_snapshot_v1(&value.to_string()).expect("unknown licence is valid");
        assert_eq!(snapshot.source_record_license().status, LicenseStatus::Unknown);
        assert_eq!(snapshot.source_record_license().identifier, None);
    }

    #[test]
    fn unknown_license_cannot_smuggle_an_identifier() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["source_record_license"]["status"] = serde_json::json!("unknown");
        value["source_record_license"]["identifier"] = serde_json::json!("CC0-1.0");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::InvalidSourceRecordLicense)
        ));
    }

    #[test]
    fn unknown_top_level_fields_are_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["unreviewed_extra"] = serde_json::json!(true);
        assert!(matches!(
            parse_snapshot_v1(&value.to_string()),
            Err(SnapshotParseError::InvalidPayload(_))
        ));
    }

    #[test]
    fn malformed_known_license_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["dataset_license"]["identifier"] = serde_json::Value::Null;
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::InvalidDatasetLicense)
        ));
    }

    #[test]
    fn a_digest_cannot_be_attached_to_metadata_only_record() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["artifact_sha256"] =
            serde_json::json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::UnexpectedDigestForMetadataOnly)
        ));
    }

    #[test]
    fn recorded_digest_requires_valid_sha256_syntax() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["artifact_capture"] = serde_json::json!("digest_recorded");
        value["artifact_sha256"] = serde_json::json!("not-a-digest");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::ArtifactDigestMissingOrMalformed)
        ));
    }

    #[test]
    fn recorded_digest_requires_valid_sha256_syntax() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["source_record_kind"] = serde_json::json!("dataset_artifact");
        value["artifact_capture"] = serde_json::json!("digest_recorded");
        value["artifact_sha256"] = serde_json::json!("not-a-digest");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::ArtifactDigestMissingOrMalformed)
        ));
    }

    #[test]
    fn valid_digest_is_recorded_without_claiming_verification() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["source_record_kind"] = serde_json::json!("dataset_artifact");
        value["artifact_capture"] = serde_json::json!("digest_recorded");
        value["artifact_sha256"] =
            serde_json::json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let snapshot = parse_snapshot_v1(&value.to_string()).expect("syntactic digest should validate");
        assert_eq!(snapshot.artifact_capture(), ArtifactCapture::DigestRecorded);
        assert_eq!(
            snapshot.artifact_sha256(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
    }

    #[test]
    fn invalid_date_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["dataset_release_date"] = serde_json::json!("2026-02-30");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::InvalidDatasetReleaseDate)
        ));
    }

    #[test]
    fn non_https_urls_are_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["source_record_url"] =
            serde_json::json!("http://blog.opentargets.org/open-targets-platform-26-09-has-been-released/");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::InvalidSourceRecordUrl)
        ));
    }

    #[test]
    fn unknown_schema_version_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["schema_version"] = serde_json::json!(99);
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::UnsupportedSchemaVersion)
        ));
    }

    #[test]
    fn announcement_cannot_claim_a_dataset_artifact_digest() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["artifact_capture"] = serde_json::json!("digest_recorded");
        value["artifact_sha256"] =
            serde_json::json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::ArtifactCaptureKindMismatch)
        ));
    }

    #[test]
    fn malformed_unicode_date_is_rejected_without_panicking() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["retrieved_on"] = serde_json::json!("20é-01-01");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::InvalidRetrievedOn)
        ));
    }

    #[test]
    fn invalid_retrieval_date_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("fixture JSON is valid");
        value["retrieved_on"] = serde_json::json!("yesterday");
        let result = parse_snapshot_v1(&value.to_string());
        assert!(matches!(
            result,
            Err(SnapshotParseError::InvalidSnapshot(ref issues))
                if issues.contains(&SnapshotValidationIssue::InvalidRetrievedOn)
        ));
    }
}
