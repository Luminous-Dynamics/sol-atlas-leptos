// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Versioned, deterministic validation reports for Sol Atlas.
//!
//! Reports describe the results of a completed or incomplete validation run.
//! They never establish the truth of a claim, the authenticity of a source,
//! successful digest verification, or external qualification. A report that
//! says the structure is valid means only that the named ruleset found no
//! structural violations within the declared scope.

use crate::provenance::IntegrityReference;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// Current wire-schema version. Unknown versions fail closed during decode.
pub const VALIDATION_REPORT_SCHEMA_VERSION: u16 = 1;

/// The kinds of records that may be anchored in a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Artifact,
    Activity,
    Agent,
    Assertion,
    Capability,
    CapabilityInstance,
    Projection,
    Document,
}

/// The validation surface described by a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationScope {
    ProvenanceGraph,
    ProvenanceAssertionGraph,
    CapabilityGraph,
    CapabilityProjection,
    ImportedDocument,
    ExternalShapeValidation,
}

/// The strongest claim this module allows a report to make.
///
/// In particular, structural validity is not source verification, authenticity,
/// truth adjudication, causal proof, or qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationClaimCeiling {
    StructuralValidationOnly,
}

/// Whether the validation procedure itself finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationExecutionStatus {
    Completed,
    Partial,
    Failed,
}

/// A narrowly scoped result for structural validation; never an overall trust
/// or qualification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralResult {
    Valid,
    Invalid,
    NotEvaluated,
}

/// A typed, validated pointer to a record covered by the report.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RecordAnchor {
    kind: RecordKind,
    id: String,
}

impl RecordAnchor {
    pub fn new(
        kind: RecordKind,
        id: impl Into<String>,
    ) -> Result<Self, ValidationReportError> {
        let id = id.into();
        if !is_token(&id) {
            return Err(ValidationReportError::InvalidAnchorId);
        }
        Ok(Self { kind, id })
    }

    pub fn kind(&self) -> RecordKind {
        self.kind
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordAnchorUnchecked {
    kind: RecordKind,
    id: String,
}

impl<'de> Deserialize<'de> for RecordAnchor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RecordAnchorUnchecked::deserialize(deserializer)?;
        Self::new(raw.kind, raw.id).map_err(<D::Error as serde::de::Error>::custom)
    }
}

/// Stable machine-readable classification for a validation finding.
///
/// Severity and dimension are derived from the code. They are also encoded in
/// the wire format for consumers, and the deserializer rejects mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationFindingCode {
    MalformedRecord,
    InvalidIdentifier,
    DuplicateRecord,
    DanglingReference,
    InvalidActivityInterval,
    DerivedFromCycle,
    ExternalSourceUnresolved,
    IntegrityReferenceUnverified,
    AssertionConflictPreserved,
    QualificationUnknown,
    AssessmentReferenceUnverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Violation,
    Warning,
    Information,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingDimension {
    Structure,
    ExternalSourceResolution,
    IntegrityVerification,
    AssertionConflict,
    Qualification,
}

impl ValidationFindingCode {
    pub fn severity(self) -> FindingSeverity {
        match self {
            Self::MalformedRecord
            | Self::InvalidIdentifier
            | Self::DuplicateRecord
            | Self::DanglingReference
            | Self::InvalidActivityInterval
            | Self::DerivedFromCycle => FindingSeverity::Violation,
            Self::ExternalSourceUnresolved
            | Self::IntegrityReferenceUnverified
            | Self::AssertionConflictPreserved
            | Self::QualificationUnknown
            | Self::AssessmentReferenceUnverified => FindingSeverity::Warning,
        }
    }

    pub fn dimension(self) -> FindingDimension {
        match self {
            Self::MalformedRecord
            | Self::InvalidIdentifier
            | Self::DuplicateRecord
            | Self::DanglingReference
            | Self::InvalidActivityInterval
            | Self::DerivedFromCycle => FindingDimension::Structure,
            Self::ExternalSourceUnresolved => FindingDimension::ExternalSourceResolution,
            Self::IntegrityReferenceUnverified => FindingDimension::IntegrityVerification,
            Self::AssertionConflictPreserved => FindingDimension::AssertionConflict,
            Self::QualificationUnknown | Self::AssessmentReferenceUnverified => {
                FindingDimension::Qualification
            }
        }
    }

    fn is_structural_violation(self) -> bool {
        self.dimension() == FindingDimension::Structure
            && self.severity() == FindingSeverity::Violation
    }
}

/// One stable, anchorable finding. Human detail is explanatory only; consumers
/// should branch on `code`, not parse this text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ValidationFinding {
    code: ValidationFindingCode,
    severity: FindingSeverity,
    subject: Option<RecordAnchor>,
    field_path: Option<String>,
    detail: Option<String>,
}

impl ValidationFinding {
    pub fn new(
        code: ValidationFindingCode,
        subject: Option<RecordAnchor>,
        field_path: Option<String>,
        detail: Option<String>,
    ) -> Result<Self, ValidationReportError> {
        if field_path
            .as_deref()
            .is_some_and(|path| !is_non_empty_trimmed(path))
        {
            return Err(ValidationReportError::InvalidFieldPath);
        }
        if detail
            .as_deref()
            .is_some_and(|value| !is_non_empty_trimmed(value))
        {
            return Err(ValidationReportError::InvalidDetail);
        }
        Ok(Self {
            code,
            severity: code.severity(),
            subject,
            field_path,
            detail,
        })
    }

    pub fn code(&self) -> ValidationFindingCode {
        self.code
    }

    pub fn severity(&self) -> FindingSeverity {
        self.severity
    }

    pub fn dimension(&self) -> FindingDimension {
        self.code.dimension()
    }

    pub fn subject(&self) -> Option<&RecordAnchor> {
        self.subject.as_ref()
    }

    pub fn field_path(&self) -> Option<&str> {
        self.field_path.as_deref()
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Convert a provenance-graph validation error into a stable report finding.
    ///
    /// The returned code is the machine-readable contract; the formatted detail
    /// is explanatory text and must not be parsed by downstream consumers.
    pub fn from_provenance_error(
        error: &ProvenanceError,
    ) -> Result<Self, ValidationReportError> {
        use ProvenanceError as E;

        let (code, subject, field_path) = match error {
            E::DuplicateArtifact(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Artifact, id.as_str().to_owned())),
                Some("artifacts.id"),
            ),
            E::DuplicateActivity(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Activity, id.as_str().to_owned())),
                Some("activities.id"),
            ),
            E::DuplicateAgent(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Agent, id.as_str().to_owned())),
                Some("agents.id"),
            ),
            E::DuplicateRelation(_) => (
                ValidationFindingCode::DuplicateRecord,
                None,
                Some("relations"),
            ),
            E::MissingArtifact { .. } | E::MissingActivity { .. } | E::MissingAgent { .. } => (
                ValidationFindingCode::DanglingReference,
                None,
                Some("relations"),
            ),
            E::InvalidActivityInterval { activity, .. } => (
                ValidationFindingCode::InvalidActivityInterval,
                Some((RecordKind::Activity, activity.as_str().to_owned())),
                Some("activities.started_at_unix_ms/ended_at_unix_ms"),
            ),
            E::InvalidIntegrityReference { artifact, .. } => (
                ValidationFindingCode::MalformedRecord,
                Some((RecordKind::Artifact, artifact.as_str().to_owned())),
                Some("artifacts.integrity"),
            ),
            E::DerivedFromCycle => (
                ValidationFindingCode::DerivedFromCycle,
                None,
                Some("relations.derived_from"),
            ),
            E::UnknownArtifact(_) => (
                ValidationFindingCode::DanglingReference,
                None,
                Some("artifact_id"),
            ),
        };

        finding_from_domain_error(code, subject, field_path, error.to_string())
    }

    /// Convert an assertion-ledger validation error into a stable report finding.
    ///
    /// Missing endpoints are anchored to the assertion that refers to them,
    /// not to the absent endpoint (which may not exist in the report's subjects).
    pub fn from_assertion_graph_error(
        error: &AssertionGraphError,
    ) -> Result<Self, ValidationReportError> {
        use AssertionGraphError as E;

        let (code, subject, field_path) = match error {
            E::DuplicateArtifact(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Artifact, id.as_str().to_owned())),
                Some("artifacts.id"),
            ),
            E::DuplicateActivity(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Activity, id.as_str().to_owned())),
                Some("activities.id"),
            ),
            E::DuplicateAgent(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Agent, id.as_str().to_owned())),
                Some("agents.id"),
            ),
            E::DuplicateAssertion(id) => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Assertion, id.as_str().to_owned())),
                Some("assertions.id"),
            ),
            E::DuplicateSource { assertion_id, .. } => (
                ValidationFindingCode::DuplicateRecord,
                Some((RecordKind::Assertion, assertion_id.as_str().to_owned())),
                Some("assertions.sources"),
            ),
            E::InvalidArtifactIntegrity { artifact_id, .. } => (
                ValidationFindingCode::MalformedRecord,
                Some((RecordKind::Artifact, artifact_id.as_str().to_owned())),
                Some("artifacts.integrity"),
            ),
            E::InvalidActivityInterval { activity_id, .. } => (
                ValidationFindingCode::InvalidActivityInterval,
                Some((RecordKind::Activity, activity_id.as_str().to_owned())),
                Some("activities.started_at_unix_ms/ended_at_unix_ms"),
            ),
            E::MissingArtifact { assertion_id, .. }
            | E::MissingActivity { assertion_id, .. }
            | E::MissingAgent { assertion_id, .. } => (
                ValidationFindingCode::DanglingReference,
                Some((RecordKind::Assertion, assertion_id.as_str().to_owned())),
                Some("assertions.relation"),
            ),
            E::InvalidExternalAssessment { assertion_id, .. } => (
                ValidationFindingCode::MalformedRecord,
                Some((RecordKind::Assertion, assertion_id.as_str().to_owned())),
                Some("assertions.assessment_ref"),
            ),
        };

        finding_from_domain_error(code, subject, field_path, error.to_string())
    }
}

fn finding_from_domain_error(
    code: ValidationFindingCode,
    subject: Option<(RecordKind, String)>,
    field_path: Option<&str>,
    detail: String,
) -> Result<ValidationFinding, ValidationReportError> {
    let subject = subject
        .map(|(kind, id)| RecordAnchor::new(kind, id))
        .transpose()?;
    ValidationFinding::new(code, subject, field_path.map(str::to_owned), Some(detail))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationFindingUnchecked {
    code: ValidationFindingCode,
    severity: FindingSeverity,
    subject: Option<RecordAnchor>,
    field_path: Option<String>,
    detail: Option<String>,
}

impl<'de> Deserialize<'de> for ValidationFinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ValidationFindingUnchecked::deserialize(deserializer)?;
        let expected_severity = raw.code.severity();
        if raw.severity != expected_severity {
            return Err(serde::de::Error::custom(
                ValidationReportError::SeverityMismatch,
            ));
        }
        Self::new(raw.code, raw.subject, raw.field_path, raw.detail)
            .map_err(<D::Error as serde::de::Error>::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationReportError {
    InvalidRulesetId,
    InvalidAnchorId,
    InvalidFieldPath,
    InvalidDetail,
    InvalidIssuer,
    DuplicateSubject(RecordAnchor),
    DuplicateFinding(ValidationFinding),
    UnknownAnchor(RecordAnchor),
    UnsupportedSchemaVersion(u16),
    ClaimCeilingMismatch,
    SeverityMismatch,
}

impl fmt::Display for ValidationReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRulesetId => {
                write!(f, "ruleset identifier must be a non-empty token")
            }
            Self::InvalidAnchorId => write!(f, "record anchor ID must be a non-empty token"),
            Self::InvalidFieldPath => {
                write!(f, "field path must be non-empty, trimmed, and control-free")
            }
            Self::InvalidDetail => {
                write!(f, "finding detail must be non-empty, trimmed, and control-free")
            }
            Self::InvalidIssuer => {
                write!(f, "report issuer must be non-empty, trimmed, and control-free")
            }
            Self::DuplicateSubject(anchor) => {
                write!(f, "duplicate report subject: {:?}/{}", anchor.kind, anchor.id)
            }
            Self::DuplicateFinding(finding) => {
                write!(f, "duplicate validation finding: {:?}", finding.code)
            }
            Self::UnknownAnchor(anchor) => {
                write!(
                    f,
                    "finding references unknown report subject: {:?}/{}",
                    anchor.kind, anchor.id
                )
            }
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported validation report schema version: {version}")
            }
            Self::ClaimCeilingMismatch => {
                write!(f, "report claim ceiling exceeds structural validation")
            }
            Self::SeverityMismatch => {
                write!(f, "finding severity does not match its stable finding code")
            }
        }
    }
}

impl std::error::Error for ValidationReportError {}

/// A portable, versioned validation artifact.
///
/// Serialization order is stable: subjects and findings are normalized by the
/// constructor, and duplicate entries and dangling anchors are rejected rather
/// than silently removed. The optional content digest is metadata only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationReport {
    schema_version: u16,
    ruleset_id: String,
    scope: ValidationScope,
    execution_status: ValidationExecutionStatus,
    claim_ceiling: ValidationClaimCeiling,
    subjects: Vec<RecordAnchor>,
    findings: Vec<ValidationFinding>,
    issuer: Option<String>,
    content_digest: Option<IntegrityReference>,
}

impl ValidationReport {
    pub fn new(
        ruleset_id: impl Into<String>,
        scope: ValidationScope,
        execution_status: ValidationExecutionStatus,
        subjects: Vec<RecordAnchor>,
        findings: Vec<ValidationFinding>,
        issuer: Option<String>,
        content_digest: Option<IntegrityReference>,
    ) -> Result<Self, ValidationReportError> {
        let ruleset_id = ruleset_id.into();
        if !is_token(&ruleset_id) {
            return Err(ValidationReportError::InvalidRulesetId);
        }
        if issuer
            .as_deref()
            .is_some_and(|value| !is_non_empty_trimmed(value))
        {
            return Err(ValidationReportError::InvalidIssuer);
        }

        let mut subjects = subjects;
        subjects.sort();
        let mut subject_set = BTreeSet::new();
        for subject in &subjects {
            if !subject_set.insert(subject.clone()) {
                return Err(ValidationReportError::DuplicateSubject(subject.clone()));
            }
        }

        let mut findings = findings;
        findings.sort();
        let mut finding_set = BTreeSet::new();
        for finding in &findings {
            if !finding_set.insert(finding.clone()) {
                return Err(ValidationReportError::DuplicateFinding(finding.clone()));
            }
            if let Some(subject) = finding.subject() {
                if !subject_set.contains(subject) {
                    return Err(ValidationReportError::UnknownAnchor(subject.clone()));
                }
            }
        }

        Ok(Self {
            schema_version: VALIDATION_REPORT_SCHEMA_VERSION,
            ruleset_id,
            scope,
            execution_status,
            claim_ceiling: ValidationClaimCeiling::StructuralValidationOnly,
            subjects,
            findings,
            issuer,
            content_digest,
        })
    }

    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn ruleset_id(&self) -> &str {
        &self.ruleset_id
    }

    pub fn scope(&self) -> ValidationScope {
        self.scope
    }

    pub fn execution_status(&self) -> ValidationExecutionStatus {
        self.execution_status
    }

    pub fn claim_ceiling(&self) -> ValidationClaimCeiling {
        self.claim_ceiling
    }

    pub fn subjects(&self) -> &[RecordAnchor] {
        &self.subjects
    }

    pub fn findings(&self) -> &[ValidationFinding] {
        &self.findings
    }

    pub fn issuer(&self) -> Option<&str> {
        self.issuer.as_deref()
    }

    pub fn content_digest(&self) -> Option<&IntegrityReference> {
        self.content_digest.as_ref()
    }

    /// Structural outcome only. This method never returns an evidence or
    /// qualification verdict. Partial/failed runs without a known violation
    /// remain `NotEvaluated`, not valid.
    pub fn structural_result(&self) -> StructuralResult {
        if self
            .findings
            .iter()
            .any(|finding| finding.code.is_structural_violation())
        {
            StructuralResult::Invalid
        } else if self.execution_status == ValidationExecutionStatus::Completed {
            StructuralResult::Valid
        } else {
            StructuralResult::NotEvaluated
        }
    }

    /// Stable JSON. This is a serialization artifact, not a cryptographic
    /// digest or proof that the report issuer is trusted.
    pub fn deterministic_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationReportUnchecked {
    schema_version: u16,
    ruleset_id: String,
    scope: ValidationScope,
    execution_status: ValidationExecutionStatus,
    claim_ceiling: ValidationClaimCeiling,
    subjects: Vec<RecordAnchor>,
    findings: Vec<ValidationFinding>,
    issuer: Option<String>,
    content_digest: Option<IntegrityReference>,
}

impl<'de> Deserialize<'de> for ValidationReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ValidationReportUnchecked::deserialize(deserializer)?;
        if raw.schema_version != VALIDATION_REPORT_SCHEMA_VERSION {
            return Err(<D::Error as serde::de::Error>::custom(
                ValidationReportError::UnsupportedSchemaVersion(raw.schema_version),
            ));
        }
        if raw.claim_ceiling != ValidationClaimCeiling::StructuralValidationOnly {
            return Err(<D::Error as serde::de::Error>::custom(
                ValidationReportError::ClaimCeilingMismatch,
            ));
        }
        let report = Self::new(
            raw.ruleset_id,
            raw.scope,
            raw.execution_status,
            raw.subjects,
            raw.findings,
            raw.issuer,
            raw.content_digest,
        )
        .map_err(<D::Error as serde::de::Error>::custom)?;
        Ok(report)
    }
}

fn is_token(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value.chars().any(char::is_whitespace)
        && !value.chars().any(char::is_control)
}

fn is_non_empty_trimmed(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(kind: RecordKind, id: &str) -> RecordAnchor {
        RecordAnchor::new(kind, id).unwrap()
    }

    fn finding(
        code: ValidationFindingCode,
        subject: Option<RecordAnchor>,
        path: Option<&str>,
    ) -> ValidationFinding {
        ValidationFinding::new(
            code,
            subject,
            path.map(str::to_owned),
            None,
        )
        .unwrap()
    }

    fn report(
        subjects: Vec<RecordAnchor>,
        findings: Vec<ValidationFinding>,
        status: ValidationExecutionStatus,
    ) -> ValidationReport {
        ValidationReport::new(
            "sol-atlas-provenance-v1",
            ValidationScope::ProvenanceAssertionGraph,
            status,
            subjects,
            findings,
            Some("sol-atlas-core".into()),
            None,
        )
        .unwrap()
    }

    #[test]
    fn provenance_errors_map_to_stable_findings_and_record_anchors() {
        let error = ProvenanceError::InvalidActivityInterval {
            activity: crate::provenance::ActivityId::new("activity-7").unwrap(),
            started_at_unix_ms: 20,
            ended_at_unix_ms: 10,
        };

        let finding = ValidationFinding::from_provenance_error(&error).unwrap();
        assert_eq!(
            finding.code(),
            ValidationFindingCode::InvalidActivityInterval
        );
        assert_eq!(
            finding.subject(),
            Some(&anchor(RecordKind::Activity, "activity-7"))
        );
        assert_eq!(
            finding.field_path(),
            Some("activities.started_at_unix_ms/ended_at_unix_ms")
        );
        assert_eq!(finding.detail(), Some(error.to_string().as_str()));
    }

    #[test]
    fn missing_assertion_endpoint_anchors_the_assertion_not_the_absent_record() {
        let error = AssertionGraphError::MissingArtifact {
            assertion_id: crate::provenance_assertions::AssertionId::new("assertion-4").unwrap(),
            artifact_id: crate::provenance::ArtifactId::new("missing-artifact").unwrap(),
        };

        let finding = ValidationFinding::from_assertion_graph_error(&error).unwrap();
        assert_eq!(finding.code(), ValidationFindingCode::DanglingReference);
        assert_eq!(
            finding.subject(),
            Some(&anchor(RecordKind::Assertion, "assertion-4"))
        );
        assert_eq!(finding.field_path(), Some("assertions.relation"));
        assert!(finding.detail().unwrap().contains("missing-artifact"));
    }

    #[test]
    fn cycle_errors_map_to_a_structural_violation_without_inventing_an_anchor() {
        let finding =
            ValidationFinding::from_provenance_error(&ProvenanceError::DerivedFromCycle).unwrap();

        assert_eq!(finding.code(), ValidationFindingCode::DerivedFromCycle);
        assert_eq!(finding.severity(), FindingSeverity::Violation);
        assert_eq!(finding.subject(), None);
        assert_eq!(finding.field_path(), Some("relations.derived_from"));
    }

    #[test]
    fn normalizes_subjects_and_findings_for_deterministic_json() {
        let artifact_a = anchor(RecordKind::Artifact, "artifact-a");
        let artifact_b = anchor(RecordKind::Artifact, "artifact-b");
        let first_finding = finding(
            ValidationFindingCode::IntegrityReferenceUnverified,
            Some(artifact_a.clone()),
            Some("integrity"),
        );
        let second_finding = finding(
            ValidationFindingCode::QualificationUnknown,
            Some(artifact_b.clone()),
            Some("assessment_ref"),
        );

        let left = report(
            vec![artifact_b.clone(), artifact_a.clone()],
            vec![second_finding.clone(), first_finding.clone()],
            ValidationExecutionStatus::Completed,
        );
        let right = report(
            vec![artifact_a, artifact_b],
            vec![first_finding, second_finding],
            ValidationExecutionStatus::Completed,
        );

        assert_eq!(
            left.deterministic_json().unwrap(),
            right.deterministic_json().unwrap()
        );
        assert_eq!(left.schema_version(), VALIDATION_REPORT_SCHEMA_VERSION);
    }

    #[test]
    fn duplicate_subjects_and_duplicate_findings_fail_closed() {
        let subject = anchor(RecordKind::Artifact, "artifact-a");
        let repeated = finding(
            ValidationFindingCode::IntegrityReferenceUnverified,
            Some(subject.clone()),
            Some("integrity"),
        );

        assert!(matches!(
            ValidationReport::new(
                "rules-v1",
                ValidationScope::ProvenanceGraph,
                ValidationExecutionStatus::Completed,
                vec![subject.clone(), subject.clone()],
                vec![],
                None,
                None,
            ),
            Err(ValidationReportError::DuplicateSubject(_))
        ));
        assert!(matches!(
            ValidationReport::new(
                "rules-v1",
                ValidationScope::ProvenanceGraph,
                ValidationExecutionStatus::Completed,
                vec![subject],
                vec![repeated.clone(), repeated],
                None,
                None,
            ),
            Err(ValidationReportError::DuplicateFinding(_))
        ));
    }

    #[test]
    fn unknown_subject_anchor_is_rejected() {
        let subject = anchor(RecordKind::Artifact, "artifact-a");
        let unknown = anchor(RecordKind::Assertion, "assertion-missing");
        let finding = finding(
            ValidationFindingCode::DanglingReference,
            Some(unknown),
            Some("relation.artifact"),
        );
        assert!(matches!(
            ValidationReport::new(
                "rules-v1",
                ValidationScope::ProvenanceGraph,
                ValidationExecutionStatus::Completed,
                vec![subject],
                vec![finding],
                None,
                None,
            ),
            Err(ValidationReportError::UnknownAnchor(_))
        ));
    }

    #[test]
    fn non_structural_findings_do_not_become_evidence_or_qualification_passes() {
        let subject = anchor(RecordKind::Assertion, "assertion-a");
        let findings = vec![
            finding(
                ValidationFindingCode::ExternalSourceUnresolved,
                Some(subject.clone()),
                Some("sources[0]"),
            ),
            finding(
                ValidationFindingCode::IntegrityReferenceUnverified,
                Some(subject.clone()),
                Some("artifact.integrity"),
            ),
            finding(
                ValidationFindingCode::AssertionConflictPreserved,
                Some(subject.clone()),
                Some("relation"),
            ),
            finding(
                ValidationFindingCode::QualificationUnknown,
                Some(subject.clone()),
                Some("assessment_ref"),
            ),
        ];
        let report = report(
            vec![subject],
            findings,
            ValidationExecutionStatus::Completed,
        );

        assert_eq!(report.structural_result(), StructuralResult::Valid);
        assert_eq!(
            report.claim_ceiling(),
            ValidationClaimCeiling::StructuralValidationOnly
        );
        assert!(report.findings().iter().all(|item| {
            item.severity() == FindingSeverity::Warning
                && item.dimension() != FindingDimension::Structure
        }));
        let json = report.deterministic_json().unwrap();
        assert!(json.contains("external_source_unresolved"));
        assert!(json.contains("integrity_reference_unverified"));
        assert!(json.contains("assertion_conflict_preserved"));
        assert!(json.contains("qualification_unknown"));
        assert!(json.contains("structural_validation_only"));
        assert!(!json.contains("evidence_verified"));
        assert!(!json.contains("qualified"));
    }

    #[test]
    fn incomplete_run_is_not_reported_as_structurally_valid() {
        let subject = anchor(RecordKind::Artifact, "artifact-a");
        let partial = report(
            vec![subject.clone()],
            vec![],
            ValidationExecutionStatus::Partial,
        );
        assert_eq!(partial.structural_result(), StructuralResult::NotEvaluated);

        let known_violation = report(
            vec![subject.clone()],
            vec![finding(
                ValidationFindingCode::DanglingReference,
                Some(subject),
                Some("relations[0].artifact"),
            )],
            ValidationExecutionStatus::Partial,
        );
        assert_eq!(known_violation.structural_result(), StructuralResult::Invalid);
    }

    #[test]
    fn report_deserialization_reuses_all_constructor_invariants() {
        let subject = anchor(RecordKind::Artifact, "artifact-a");
        let report = report(
            vec![subject.clone()],
            vec![finding(
                ValidationFindingCode::ExternalSourceUnresolved,
                Some(subject),
                Some("sources[0]"),
            )],
            ValidationExecutionStatus::Completed,
        );
        let json = report.deterministic_json().unwrap();
        let decoded: ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);

        let wrong_schema = json.replace("\"schema_version\":1", "\"schema_version\":999");
        assert!(serde_json::from_str::<ValidationReport>(&wrong_schema).is_err());

        let wrong_severity = json.replace("\"severity\":\"warning\"", "\"severity\":\"violation\"");
        assert!(serde_json::from_str::<ValidationReport>(&wrong_severity).is_err());

        let unknown_anchor = json.replacen(
            "\"id\":\"artifact-a\"",
            "\"id\":\"artifact-missing\"",
            1,
        );
        assert!(serde_json::from_str::<ValidationReport>(&unknown_anchor).is_err());
    }

    #[test]
    fn standalone_anchor_deserialization_cannot_bypass_validation() {
        assert!(
            serde_json::from_str::<RecordAnchor>(
                r#"{"kind":"artifact","id":"bad id"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn ruleset_and_field_paths_are_validated() {
        let subject = anchor(RecordKind::Artifact, "artifact-a");
        assert!(
            ValidationReport::new(
                "rules v1",
                ValidationScope::ProvenanceGraph,
                ValidationExecutionStatus::Completed,
                vec![subject.clone()],
                vec![],
                None,
                None,
            )
            .is_err()
        );
        assert!(
            ValidationFinding::new(
                ValidationFindingCode::MalformedRecord,
                Some(subject),
                Some("  ".into()),
                None,
            )
            .is_err()
        );
    }
}
