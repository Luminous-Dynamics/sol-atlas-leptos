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

use crate::provenance::{IntegrityReference, ProvenanceError, ProvenanceRelation};
use crate::provenance_assertions::{
    AssertionGraphError, AssertionSource, ExternalSourceResolution, ProvenanceAssertionGraph,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    occurrence_key: Option<String>,
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
            occurrence_key: None,
        })
    }

    /// Attach a stable per-occurrence discriminator when one report legitimately
    /// contains multiple findings at the same code, subject, and field path.
    ///
    /// The key is an identity discriminator, not a secret, signature, or proof.
    pub fn with_occurrence_key(
        mut self,
        occurrence_key: impl Into<String>,
    ) -> Result<Self, ValidationReportError> {
        let occurrence_key = occurrence_key.into();
        if !is_token(&occurrence_key) {
            return Err(ValidationReportError::InvalidOccurrenceKey);
        }
        self.occurrence_key = Some(occurrence_key);
        Ok(self)
    }

    pub fn occurrence_key(&self) -> Option<&str> {
        self.occurrence_key.as_deref()
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

        let finding = finding_from_domain_error(code, subject, field_path)?;
        match provenance_error_occurrence_key(error) {
            Some(key) => finding.with_occurrence_key(key),
            None => Ok(finding),
        }
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

        let finding = finding_from_domain_error(code, subject, field_path)?;
        match assertion_error_occurrence_key(error) {
            Some(key) => finding.with_occurrence_key(key),
            None => Ok(finding),
        }
    }
}

fn provenance_error_occurrence_key(error: &ProvenanceError) -> Option<String> {
    use ProvenanceError as E;

    let identity = match error {
        E::DuplicateRelation(relation) => identity_material(&[
            "duplicate_relation",
            &provenance_relation_identity(relation),
        ]),
        E::MissingArtifact { relation, artifact } => identity_material(&[
            "missing_artifact",
            &provenance_relation_identity(relation),
            artifact.as_str(),
        ]),
        E::MissingActivity { relation, activity } => identity_material(&[
            "missing_activity",
            &provenance_relation_identity(relation),
            activity.as_str(),
        ]),
        E::MissingAgent { relation, agent } => identity_material(&[
            "missing_agent",
            &provenance_relation_identity(relation),
            agent.as_str(),
        ]),
        E::UnknownArtifact(artifact) => {
            identity_material(&["unknown_artifact", artifact.as_str()])
        }
        _ => return None,
    };
    Some(occurrence_fingerprint(&identity))
}

fn assertion_error_occurrence_key(error: &AssertionGraphError) -> Option<String> {
    use AssertionGraphError as E;

    let identity = match error {
        E::DuplicateSource { assertion_id, source } => identity_material(&[
            "duplicate_source",
            assertion_id.as_str(),
            &assertion_source_identity(source),
        ]),
        E::MissingArtifact { assertion_id, artifact_id } => identity_material(&[
            "missing_artifact",
            assertion_id.as_str(),
            artifact_id.as_str(),
        ]),
        E::MissingActivity { assertion_id, activity_id } => identity_material(&[
            "missing_activity",
            assertion_id.as_str(),
            activity_id.as_str(),
        ]),
        E::MissingAgent { assertion_id, agent_id } => identity_material(&[
            "missing_agent",
            assertion_id.as_str(),
            agent_id.as_str(),
        ]),
        _ => return None,
    };
    Some(occurrence_fingerprint(&identity))
}

/// Length-prefix components before hashing so identifiers containing separators
/// cannot alias another tuple of identity fields.
fn identity_material(parts: &[&str]) -> String {
    let mut material = String::new();
    for part in parts {
        material.push_str(&part.len().to_string());
        material.push(':');
        material.push_str(part);
    }
    material
}

fn provenance_relation_identity(relation: &ProvenanceRelation) -> String {
    match relation {
        ProvenanceRelation::Used { activity, artifact } => {
            identity_material(&["used", activity.as_str(), artifact.as_str()])
        }
        ProvenanceRelation::Generated { activity, artifact } => {
            identity_material(&["generated", activity.as_str(), artifact.as_str()])
        }
        ProvenanceRelation::AssociatedWith { activity, agent } => {
            identity_material(&["associated_with", activity.as_str(), agent.as_str()])
        }
        ProvenanceRelation::DerivedFrom { artifact, source } => {
            identity_material(&["derived_from", artifact.as_str(), source.as_str()])
        }
        ProvenanceRelation::AttributedTo { artifact, agent } => {
            identity_material(&["attributed_to", artifact.as_str(), agent.as_str()])
        }
    }
}

fn assertion_source_identity(source: &AssertionSource) -> String {
    match source {
        AssertionSource::LocalArtifact { artifact_id } => {
            identity_material(&["local_artifact", artifact_id.as_str()])
        }
        AssertionSource::External {
            locator,
            resolution: ExternalSourceResolution::Unresolved,
        } => identity_material(&["external", "unresolved", locator.as_str()]),
    }
}

/// Deterministic SHA-256 identifier used only to distinguish finding
/// occurrences. It is not a signature, integrity proof, or secret-keyed token.
fn occurrence_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let mut key = String::with_capacity(4 + digest.len() * 2);
    key.push_str("occ-");
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest.iter() {
        key.push(HEX[(*byte >> 4) as usize] as char);
        key.push(HEX[(*byte & 0x0f) as usize] as char);
    }
    key
}

fn finding_detail(code: ValidationFindingCode) -> &'static str {
    match code {
        ValidationFindingCode::MalformedRecord => "record failed domain validation",
        ValidationFindingCode::InvalidIdentifier => "identifier failed validation",
        ValidationFindingCode::DuplicateRecord => "duplicate record or relationship",
        ValidationFindingCode::DanglingReference => "referenced record is absent",
        ValidationFindingCode::InvalidActivityInterval => "activity time interval is invalid",
        ValidationFindingCode::DerivedFromCycle => "derived-from relation is cyclic",
        ValidationFindingCode::ExternalSourceUnresolved => "external source has not been resolved",
        ValidationFindingCode::IntegrityReferenceUnverified => {
            "integrity metadata is present but the digest was not verified"
        }
        ValidationFindingCode::AssertionConflictPreserved => {
            "conflicting assertion was preserved without adjudication"
        }
        ValidationFindingCode::QualificationUnknown => "qualification has not been established",
        ValidationFindingCode::AssessmentReferenceUnverified => {
            "assessment reference has not been retrieved or authenticated"
        }
    }
}

fn finding_from_domain_error(
    code: ValidationFindingCode,
    subject: Option<(RecordKind, String)>,
    field_path: Option<&str>,
) -> Result<ValidationFinding, ValidationReportError> {
    let subject = subject
        .map(|(kind, id)| RecordAnchor::new(kind, id))
        .transpose()?;
    ValidationFinding::new(
        code,
        subject,
        field_path.map(str::to_owned),
        Some(finding_detail(code).into()),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationFindingUnchecked {
    code: ValidationFindingCode,
    severity: FindingSeverity,
    subject: Option<RecordAnchor>,
    field_path: Option<String>,
    detail: Option<String>,
    #[serde(default)]
    occurrence_key: Option<String>,
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
        let mut finding = Self::new(raw.code, raw.subject, raw.field_path, raw.detail)
            .map_err(<D::Error as serde::de::Error>::custom)?;
        if let Some(key) = raw.occurrence_key {
            finding = finding
                .with_occurrence_key(key)
                .map_err(<D::Error as serde::de::Error>::custom)?;
        }
        Ok(finding)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationReportError {
    InvalidRulesetId,
    InvalidAnchorId,
    InvalidFieldPath,
    InvalidDetail,
    InvalidOccurrenceKey,
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
            Self::InvalidOccurrenceKey => {
                write!(f, "finding occurrence key must be a non-empty token")
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
        let mut finding_identity_set: BTreeSet<(
            ValidationFindingCode,
            Option<RecordAnchor>,
            Option<String>,
            Option<String>,
        )> = BTreeSet::new();
        for finding in &findings {
            // Detail text is explanatory, not identity. Findings share an
            // identity only when code, subject, field path, and occurrence key
            // all match; adapters use an opaque key for distinct unanchored errors.
            let identity = (
                finding.code,
                finding.subject.clone(),
                finding.field_path.clone(),
                finding.occurrence_key.clone(),
            );
            if !finding_identity_set.insert(identity) {
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

    /// Build a completed, structural-only report for an already validated
    /// assertion graph. Non-local evidence remains explicitly unresolved.
    ///
    /// This does not retrieve external locators, verify integrity digests,
    /// authenticate assessment records, or adjudicate assertions. Derived-from
    /// cycles are reported as preserved conflicts, not repaired or discarded.
    pub fn from_assertion_graph(
        ruleset_id: impl Into<String>,
        graph: &ProvenanceAssertionGraph,
        issuer: Option<String>,
    ) -> Result<Self, ValidationReportError> {
        let mut subjects = Vec::new();
        let mut findings = Vec::new();

        for artifact in graph.artifacts() {
            let anchor = RecordAnchor::new(RecordKind::Artifact, artifact.id.as_str())?;
            if artifact.integrity.is_some() {
                findings.push(ValidationFinding::new(
                    ValidationFindingCode::IntegrityReferenceUnverified,
                    Some(anchor.clone()),
                    Some(format!("artifacts[{}].integrity", artifact.id.as_str())),
                    Some(
                        "integrity reference is present, but this report did not verify the digest"
                            .into(),
                    ),
                )?);
            }
            if artifact.source_uri.is_some() {
                findings.push(ValidationFinding::new(
                    ValidationFindingCode::ExternalSourceUnresolved,
                    Some(anchor.clone()),
                    Some(format!("artifacts[{}].source_uri", artifact.id.as_str())),
                    Some(
                        "artifact source locator has not been retrieved or resolved"
                            .into(),
                    ),
                )?);
            }
            subjects.push(anchor);
        }

        for activity in graph.activities() {
            subjects.push(RecordAnchor::new(
                RecordKind::Activity,
                activity.id.as_str(),
            )?);
        }

        for agent in graph.agents() {
            subjects.push(RecordAnchor::new(RecordKind::Agent, agent.id.as_str())?);
        }

        for assertion in graph.assertions() {
            let anchor = RecordAnchor::new(RecordKind::Assertion, assertion.id.as_str())?;
            for (index, source) in assertion.sources.iter().enumerate() {
                if let AssertionSource::External {
                    resolution: ExternalSourceResolution::Unresolved,
                    ..
                } = source
                {
                    findings.push(ValidationFinding::new(
                        ValidationFindingCode::ExternalSourceUnresolved,
                        Some(anchor.clone()),
                        Some(format!(
                            "assertions[{}].sources[{index}]",
                            assertion.id.as_str()
                        )),
                        Some(
                            "external source locator is recorded but has not been resolved".into(),
                        ),
                    )?);
                }
            }

            if assertion.assessment_ref.is_some() {
                findings.push(ValidationFinding::new(
                    ValidationFindingCode::AssessmentReferenceUnverified,
                    Some(anchor.clone()),
                    Some(format!(
                        "assertions[{}].assessment_ref",
                        assertion.id.as_str()
                    )),
                    Some(
                        "external assessment pointer has not been retrieved or authenticated"
                            .into(),
                    ),
                )?);
                findings.push(ValidationFinding::new(
                    ValidationFindingCode::QualificationUnknown,
                    Some(anchor.clone()),
                    Some(format!(
                        "assertions[{}].assessment_ref",
                        assertion.id.as_str()
                    )),
                    Some(
                        "an assessment pointer does not establish that the assertion is qualified"
                            .into(),
                    ),
                )?);
            }
            subjects.push(anchor);
        }

        let adjacency = derived_from_adjacency(graph);
        for assertion in graph.assertions() {
            if let ProvenanceRelation::DerivedFrom { artifact, source } = &assertion.relation {
                if reachable_in_graph(&adjacency, source.as_str(), artifact.as_str()) {
                    let anchor =
                        RecordAnchor::new(RecordKind::Assertion, assertion.id.as_str())?;
                    findings.push(ValidationFinding::new(
                        ValidationFindingCode::AssertionConflictPreserved,
                        Some(anchor),
                        Some(format!(
                            "assertions[{}].relation",
                            assertion.id.as_str()
                        )),
                        Some(
                            "derived-from cycle detected; all assertions were preserved"
                                .into(),
                        ),
                    )?);
                }
            }
        }

        Self::new(
            ruleset_id,
            ValidationScope::ProvenanceAssertionGraph,
            ValidationExecutionStatus::Completed,
            subjects,
            findings,
            issuer,
            None,
        )
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

fn derived_from_adjacency(
    graph: &ProvenanceAssertionGraph,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for assertion in graph.assertions() {
        if let ProvenanceRelation::DerivedFrom { artifact, source } = &assertion.relation {
            adjacency
                .entry(artifact.as_str().to_owned())
                .or_default()
                .insert(source.as_str().to_owned());
        }
    }
    adjacency
}

fn reachable_in_graph(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    start: &str,
    target: &str,
) -> bool {
    let mut pending = vec![start.to_owned()];
    let mut visited = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if current.as_str() == target {
            return true;
        }
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(next) = adjacency.get(&current) {
            pending.extend(next.iter().cloned());
        }
    }
    false
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
        assert_eq!(finding.detail(), Some("activity time interval is invalid"));
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
        assert_eq!(
            finding.detail(),
            Some("referenced record is absent")
        );
        assert!(!finding.detail().unwrap().contains("missing-artifact"));
    }

    #[test]
    fn assertion_error_details_redact_external_source_locators() {
        let secret_locator = "https://sources.example/private?token=super-secret";
        let error = AssertionGraphError::DuplicateSource {
            assertion_id: crate::provenance_assertions::AssertionId::new("assertion-redacted")
                .unwrap(),
            source: crate::provenance_assertions::AssertionSource::External {
                locator: crate::provenance_assertions::ExternalSourceLocator::new(secret_locator)
                    .unwrap(),
                resolution: crate::provenance_assertions::ExternalSourceResolution::Unresolved,
            },
        };

        let finding = ValidationFinding::from_assertion_graph_error(&error).unwrap();
        assert_eq!(finding.code(), ValidationFindingCode::DuplicateRecord);
        assert_eq!(
            finding.subject(),
            Some(&anchor(RecordKind::Assertion, "assertion-redacted"))
        );
        let detail = finding.detail().unwrap();
        assert!(!detail.contains(secret_locator));
        assert!(!detail.contains("super-secret"));
        assert!(finding.occurrence_key().unwrap().starts_with("occ-"));
        let encoded = serde_json::to_string(&finding).unwrap();
        assert!(!encoded.contains(secret_locator));
        assert!(!encoded.contains("super-secret"));
    }

    #[test]
    fn unanchored_domain_findings_keep_distinct_occurrences() {
        let first_relation = ProvenanceRelation::DerivedFrom {
            artifact: crate::provenance::ArtifactId::new("artifact-a").unwrap(),
            source: crate::provenance::ArtifactId::new("missing-a").unwrap(),
        };
        let second_relation = ProvenanceRelation::DerivedFrom {
            artifact: crate::provenance::ArtifactId::new("artifact-b").unwrap(),
            source: crate::provenance::ArtifactId::new("missing-b").unwrap(),
        };
        let first = ValidationFinding::from_provenance_error(
            &ProvenanceError::MissingArtifact {
                relation: first_relation,
                artifact: crate::provenance::ArtifactId::new("missing-a").unwrap(),
            },
        )
        .unwrap();
        let second = ValidationFinding::from_provenance_error(
            &ProvenanceError::MissingArtifact {
                relation: second_relation,
                artifact: crate::provenance::ArtifactId::new("missing-b").unwrap(),
            },
        )
        .unwrap();

        assert_eq!(first.code(), ValidationFindingCode::DanglingReference);
        assert_eq!(second.code(), ValidationFindingCode::DanglingReference);
        assert_eq!(first.subject(), None);
        assert_eq!(second.subject(), None);
        assert_eq!(first.field_path(), second.field_path());
        assert_ne!(first.occurrence_key(), second.occurrence_key());
        assert!(!first.detail().unwrap().contains("missing-a"));
        assert!(!second.detail().unwrap().contains("missing-b"));

        let combined = report(
            vec![],
            vec![first, second],
            ValidationExecutionStatus::Completed,
        );
        assert_eq!(combined.findings().len(), 2);

        let json = combined.deterministic_json().unwrap();
        assert!(!json.contains("missing-a"));
        assert!(!json.contains("missing-b"));
        let decoded: ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, combined);

        let mut invalid_key: serde_json::Value = serde_json::from_str(&json).unwrap();
        invalid_key["findings"][0]["occurrence_key"] = "contains whitespace".into();
        assert!(serde_json::from_value::<ValidationReport>(invalid_key).is_err());
    }

    #[test]
    fn occurrence_identity_frames_identifiers_that_contain_separators() {
        let first_relation = ProvenanceRelation::DerivedFrom {
            artifact: crate::provenance::ArtifactId::new("a:b").unwrap(),
            source: crate::provenance::ArtifactId::new("c").unwrap(),
        };
        let second_relation = ProvenanceRelation::DerivedFrom {
            artifact: crate::provenance::ArtifactId::new("a").unwrap(),
            source: crate::provenance::ArtifactId::new("b:c").unwrap(),
        };
        assert_ne!(
            provenance_relation_identity(&first_relation),
            provenance_relation_identity(&second_relation)
        );

        let first = ValidationFinding::from_provenance_error(
            &ProvenanceError::DuplicateRelation(first_relation),
        )
        .unwrap();
        let second = ValidationFinding::from_provenance_error(
            &ProvenanceError::DuplicateRelation(second_relation),
        )
        .unwrap();
        assert_eq!(first.code(), ValidationFindingCode::DuplicateRecord);
        assert_eq!(second.code(), ValidationFindingCode::DuplicateRecord);
        assert_ne!(first.occurrence_key(), second.occurrence_key());
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
    fn assertion_graph_report_surfaces_unresolved_evidence_without_overclaiming() {
        use crate::provenance::{
            ActivityRecord, AgentRecord, ArtifactId, ArtifactSnapshot, IntegrityReference,
        };
        use crate::provenance_assertions::{
            AssertionId, AssertionSource, ExternalAssessmentReference, ExternalSourceLocator,
            ExternalSourceResolution, ProvenanceAssertion, ProvenanceAssertionGraph,
        };

        let artifact = ArtifactSnapshot {
            id: ArtifactId::new("artifact-a").unwrap(),
            label: "artifact-a".into(),
            media_type: Some("application/json".into()),
            integrity: Some(IntegrityReference::new("sha256", "deadbeef").unwrap()),
            source_uri: Some("urn:artifact:source-a".into()),
        };
        let second_artifact = ArtifactSnapshot {
            id: ArtifactId::new("artifact-b").unwrap(),
            label: "artifact-b".into(),
            media_type: None,
            integrity: None,
            source_uri: None,
        };
        let assertion = ProvenanceAssertion {
            id: AssertionId::new("assertion-a").unwrap(),
            relation: ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("artifact-b").unwrap(),
                source: ArtifactId::new("artifact-a").unwrap(),
            },
            asserted_by: None,
            sources: vec![AssertionSource::External {
                locator: ExternalSourceLocator::new("urn:source:report-a").unwrap(),
                resolution: ExternalSourceResolution::Unresolved,
            }],
            reported_at_unix_ms: None,
            assessment_ref: Some(
                ExternalAssessmentReference::new("example-authority", "assessment-12").unwrap(),
            ),
        };
        let graph = ProvenanceAssertionGraph::new(
            vec![artifact, second_artifact],
            Vec::<ActivityRecord>::new(),
            Vec::<AgentRecord>::new(),
            vec![assertion],
        )
        .unwrap();

        let report = ValidationReport::from_assertion_graph(
            "sol-atlas-provenance-v1",
            &graph,
            Some("sol-atlas-core".into()),
        )
        .unwrap();

        assert_eq!(report.execution_status(), ValidationExecutionStatus::Completed);
        assert_eq!(report.structural_result(), StructuralResult::Valid);
        assert_eq!(
            report.claim_ceiling(),
            ValidationClaimCeiling::StructuralValidationOnly
        );
        assert_eq!(report.subjects().len(), 3);
        assert!(report.findings().iter().any(|finding| {
            finding.code() == ValidationFindingCode::ExternalSourceUnresolved
                && finding.subject() == Some(&anchor(RecordKind::Assertion, "assertion-a"))
        }));
        assert!(report.findings().iter().any(|finding| {
            finding.code() == ValidationFindingCode::IntegrityReferenceUnverified
                && finding.subject() == Some(&anchor(RecordKind::Artifact, "artifact-a"))
        }));
        assert!(report.findings().iter().any(|finding| {
            finding.code() == ValidationFindingCode::ExternalSourceUnresolved
                && finding.subject() == Some(&anchor(RecordKind::Artifact, "artifact-a"))
                && finding.field_path() == Some("artifacts[artifact-a].source_uri")
        }));
        assert!(report.findings().iter().any(|finding| {
            finding.code() == ValidationFindingCode::AssessmentReferenceUnverified
        }));
        assert!(report.findings().iter().any(|finding| {
            finding.code() == ValidationFindingCode::QualificationUnknown
        }));
        let json = report.deterministic_json().unwrap();
        assert!(json.contains("external_source_unresolved"));
        assert!(json.contains("integrity_reference_unverified"));
        assert!(json.contains("assessment_reference_unverified"));
        assert!(!json.contains("deadbeef"));
        assert!(!json.contains("urn:artifact:source-a"));
        assert!(!json.contains("urn:source:report-a"));
        assert!(!json.contains("example-authority"));
        assert!(!json.contains("assessment-12"));
        assert!(json.contains("qualification_unknown"));
        assert!(!json.contains("evidence_verified"));
    }

    #[test]
    fn assertion_graph_report_flags_cycles_without_repairing_the_ledger() {
        use crate::provenance::{
            ActivityRecord, AgentRecord, ArtifactId, ArtifactSnapshot,
        };
        use crate::provenance_assertions::{
            AssertionId, ProvenanceAssertion, ProvenanceAssertionGraph,
        };

        let artifact = |id: &str| ArtifactSnapshot {
            id: ArtifactId::new(id).unwrap(),
            label: id.into(),
            media_type: None,
            integrity: None,
            source_uri: None,
        };
        let assertion = |id: &str, artifact_id: &str, source_id: &str| ProvenanceAssertion {
            id: AssertionId::new(id).unwrap(),
            relation: ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new(artifact_id).unwrap(),
                source: ArtifactId::new(source_id).unwrap(),
            },
            asserted_by: None,
            sources: vec![],
            reported_at_unix_ms: None,
            assessment_ref: None,
        };
        let graph = ProvenanceAssertionGraph::new(
            vec![artifact("a"), artifact("b")],
            Vec::<ActivityRecord>::new(),
            Vec::<AgentRecord>::new(),
            vec![assertion("assertion-ab", "a", "b"), assertion("assertion-ba", "b", "a")],
        )
        .unwrap();

        let report =
            ValidationReport::from_assertion_graph("rules-v1", &graph, None).unwrap();
        let cycle_findings = report
            .findings()
            .iter()
            .filter(|finding| {
                finding.code() == ValidationFindingCode::AssertionConflictPreserved
            })
            .count();

        assert_eq!(cycle_findings, 2);
        assert_eq!(graph.assertions().len(), 2);
        assert_eq!(report.structural_result(), StructuralResult::Valid);
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
        let same_identity_different_detail = ValidationFinding::new(
            ValidationFindingCode::IntegrityReferenceUnverified,
            Some(subject.clone()),
            Some("integrity".into()),
            Some("different explanatory wording".into()),
        )
        .unwrap();
        assert!(matches!(
            ValidationReport::new(
                "rules-v1",
                ValidationScope::ProvenanceGraph,
                ValidationExecutionStatus::Completed,
                vec![subject.clone()],
                vec![repeated.clone(), same_identity_different_detail],
                None,
                None,
            ),
            Err(ValidationReportError::DuplicateFinding(_))
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
    fn report_deserialization_rejects_unknown_fields_at_object_boundaries() {
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

        let mut unknown_root: serde_json::Value =
            serde_json::from_str(&json).unwrap();
        unknown_root
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), serde_json::Value::Null);
        assert!(serde_json::from_value::<ValidationReport>(unknown_root).is_err());

        let mut unknown_finding: serde_json::Value =
            serde_json::from_str(&json).unwrap();
        unknown_finding["findings"][0]
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), serde_json::Value::Null);
        assert!(serde_json::from_value::<ValidationReport>(unknown_finding).is_err());

        let mut unknown_anchor: serde_json::Value =
            serde_json::from_str(&json).unwrap();
        unknown_anchor["subjects"][0]
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), serde_json::Value::Null);
        assert!(serde_json::from_value::<ValidationReport>(unknown_anchor).is_err());
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
