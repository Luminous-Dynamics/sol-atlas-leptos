// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned evidence graph for built-environment design and approval records.
//!
//! The graph binds evidence, model artifacts, engineering receipts and decisions to
//! exact identifiers and content digests. Passing validation means only that this
//! provenance contract is internally consistent under the caller's policy. It is
//! not a permit, structural certification, or guarantee that any underlying claim is true.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Verified,
    Estimated,
    Unknown,
    Disputed,
    Stale,
    Restricted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRights {
    PublicDomain,
    OpenLicense,
    Restricted,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    SiteBrief,
    SiteModel,
    DesignModel,
    IfcModel,
    DrawingPackage,
    StructuralAnalysis,
    GeotechnicalReport,
    WindAnalysis,
    SeismicAnalysis,
    FireLifeSafety,
    AccessibilityReview,
    EnergyAnalysis,
    MaterialSchedule,
    ConstructionPlan,
    InspectionRecord,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineeringDiscipline {
    Structural,
    Geotechnical,
    Wind,
    Seismic,
    FireLifeSafety,
    Accessibility,
    Energy,
    Materials,
    Constructability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverRunStatus {
    Completed,
    NotRun,
    Failed,
    NonConverged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineeringConclusion {
    PassWithinDeclaredScope,
    Fail,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewLevel {
    None,
    InternalReproduced,
    IndependentlyChecked,
    QualifiedProfessional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionDisposition {
    Approve,
    Conditional,
    Reject,
    Revoke,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub evidence_id: String,
    pub subject_id: String,
    pub field_path: String,
    pub source_id: String,
    pub source_payload_sha256: Option<String>,
    pub evidence_state: EvidenceState,
    pub data_rights: DataRights,
    pub license_id: Option<String>,
    pub jurisdiction: String,
    pub observed_at_unix_seconds: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub artifact_id: String,
    pub content_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub artifact_id: String,
    pub design_id: String,
    /// Artifacts in the same family are revisions of the same logical deliverable.
    pub family_id: String,
    pub revision: u64,
    pub kind: ArtifactKind,
    pub content_sha256: String,
    pub parent_artifacts: Vec<ArtifactReference>,
    pub evidence_ids: Vec<String>,
    pub jurisdiction: String,
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineeringReceipt {
    pub receipt_id: String,
    pub design_id: String,
    pub input_artifact_id: String,
    pub input_artifact_sha256: String,
    pub discipline: EngineeringDiscipline,
    pub solver_id: Option<String>,
    pub solver_version: Option<String>,
    pub run_status: SolverRunStatus,
    pub conclusion: EngineeringConclusion,
    pub review_level: ReviewLevel,
    pub reviewer_id: Option<String>,
    pub assumption_evidence_ids: Vec<String>,
    pub declared_scope: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub event_id: String,
    pub design_id: String,
    pub target_artifact_id: String,
    pub target_artifact_sha256: String,
    pub disposition: DecisionDisposition,
    pub actor_id: String,
    pub authority_evidence_id: Option<String>,
    pub engineering_receipt_ids: Vec<String>,
    pub decided_at_unix_seconds: i64,
    pub conditions: Vec<String>,
    pub supersedes_event_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuiltEnvironmentCase {
    pub case_id: String,
    pub jurisdiction: String,
    pub evidence: Vec<EvidenceRecord>,
    pub artifacts: Vec<ArtifactRecord>,
    pub engineering_receipts: Vec<EngineeringReceipt>,
    pub decisions: Vec<DecisionRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceGraphPolicy {
    pub approvable_artifact_kinds: BTreeSet<ArtifactKind>,
    pub required_disciplines: BTreeSet<EngineeringDiscipline>,
    pub minimum_review_level: ReviewLevel,
    pub accepted_evidence_states: BTreeSet<EvidenceState>,
    pub accepted_data_rights: BTreeSet<DataRights>,
    pub require_authority_evidence_for_approval: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCode {
    EmptyIdentifier,
    DuplicateEvidenceId,
    DuplicateArtifactId,
    DuplicateArtifactRevision,
    DuplicateReceiptId,
    DuplicateDecisionId,
    InvalidDigest,
    VerifiedEvidenceMissingDigest,
    EvidenceRightsUnresolved,
    EvidenceJurisdictionMismatch,
    FutureTimestamp,
    MissingEvidenceReference,
    MissingParentArtifact,
    ArtifactDependencyCycle,
    MissingInputArtifact,
    StaleInputDigest,
    StaleParentArtifactDigest,
    EngineeringReceiptMismatch,
    ApprovalHasConditions,
    SolverMetadataMissing,
    ConclusionStatusMismatch,
    ProfessionalReviewerMissing,
    MissingAuthorityEvidence,
    AuthorityEvidenceNotAccepted,
    MissingEngineeringReceipt,
    EngineeringReceiptNotAccepted,
    MissingRequiredDiscipline,
    ApprovalForSupersededArtifact,
    ArtifactKindNotApprovable,
    InvalidSupersessionReference,
    SupersessionTimeReversal,
    SupersessionDesignMismatch,
    InvalidCasePolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevalidationStatus {
    NoImpact,
    RevalidationRequired,
    GraphInvalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactNodeKind {
    Artifact,
    EngineeringReceipt,
    Decision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactReason {
    SourceArtifactDigestChanged,
    DescendantArtifactDependsOnChangedAncestor,
    EngineeringReceiptInputArtifactChanged,
    DecisionTargetArtifactAffected,
    DecisionDependsOnAffectedReceipt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactGraphErrorCode {
    EmptyChangedArtifactId,
    InvalidProposedDigest,
    ChangedArtifactMissing,
    DuplicateArtifactId,
    InvalidArtifactIdentity,
    InvalidArtifactDigest,
    MissingParentArtifact,
    StaleParentArtifactDigest,
    ArtifactDependencyCycle,
    DuplicateEvidenceId,
    MissingArtifactEvidence,
    DuplicateEngineeringReceiptId,
    MissingReceiptInputArtifact,
    MissingReceiptAssumptionEvidence,
    DuplicateDecisionId,
    MissingDecisionTargetArtifact,
    MissingDecisionEngineeringReceipt,
    MissingDecisionAuthorityEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactGraphIssue {
    pub code: ImpactGraphErrorCode,
    pub item_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactRecord {
    pub node_kind: ImpactNodeKind,
    pub node_id: String,
    pub reason: ImpactReason,
    /// Canonical shortest artifact path from the changed source to this node.
    /// For receipts/decisions, the path terminates at the artifact/receipt they consume.
    pub via_artifact_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RevalidationFrontier {
    pub status: RevalidationStatus,
    pub changed_artifact_id: String,
    pub previous_content_sha256: Option<String>,
    pub proposed_content_sha256: String,
    pub impacts: Vec<ImpactRecord>,
    pub graph_issues: Vec<ImpactGraphIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub code: ValidationCode,
    pub item_id: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvidenceGraphError {
    EmptyCaseId,
    EmptyJurisdiction,
    EmptyApprovableKinds,
    InvalidPolicyStates,
    InvalidPolicyRights,
    EmptyRequiredDisciplineSet,
}

impl fmt::Display for EvidenceGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCaseId => write!(f, "case ID must not be empty"),
            Self::EmptyJurisdiction => write!(f, "case jurisdiction must not be empty"),
            Self::EmptyApprovableKinds => {
                write!(
                    f,
                    "policy must declare at least one approvable artifact kind"
                )
            }
            Self::InvalidPolicyStates => {
                write!(f, "policy must declare accepted evidence states")
            }
            Self::InvalidPolicyRights => write!(f, "policy must declare accepted data rights"),
            Self::EmptyRequiredDisciplineSet => {
                write!(f, "policy must declare required engineering disciplines")
            }
        }
    }
}

impl std::error::Error for EvidenceGraphError {}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn issue(
    issues: &mut Vec<ValidationIssue>,
    code: ValidationCode,
    item_id: impl Into<String>,
    message: impl Into<String>,
) {
    issues.push(ValidationIssue {
        code,
        item_id: item_id.into(),
        message: message.into(),
    });
}

fn duplicates<'a>(values: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut repeated = BTreeSet::new();
    for value in values {
        if !seen.insert(value.to_owned()) {
            repeated.insert(value.to_owned());
        }
    }
    repeated
}

fn has_dependency_cycle(artifacts: &BTreeMap<String, &ArtifactRecord>) -> bool {
    // Kahn's topological pass avoids recursive DFS stack growth on large,
    // source-controlled artifact graphs. Missing parents are reported by the
    // caller and are deliberately excluded from this cycle-only calculation.
    let mut in_degree: BTreeMap<String, usize> = artifacts
        .keys()
        .map(|artifact_id| (artifact_id.clone(), 0))
        .collect();
    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();

    for (child_id, artifact) in artifacts {
        for parent in &artifact.parent_artifacts {
            if artifacts.contains_key(&parent.artifact_id) {
                if let Some(degree) = in_degree.get_mut(child_id) {
                    *degree = degree.saturating_add(1);
                }
                children_by_parent
                    .entry(parent.artifact_id.clone())
                    .or_default()
                    .push(child_id.clone());
            }
        }
    }

    let mut ready = in_degree
        .iter()
        .filter_map(|(artifact_id, degree)| {
            (*degree == 0).then_some(artifact_id.clone())
        })
        .collect::<BTreeSet<_>>();
    let mut visited_count = 0usize;

    while let Some(artifact_id) = ready.pop_first() {
        visited_count = visited_count.saturating_add(1);
        if let Some(children) = children_by_parent.get(&artifact_id) {
            for child_id in children {
                if let Some(degree) = in_degree.get_mut(child_id) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        ready.insert(child_id.clone());
                    }
                }
            }
        }
    }

    visited_count != artifacts.len()
}

fn accepted_evidence<'a>(
    evidence_id: &str,
    evidence_by_id: &BTreeMap<&'a str, &'a EvidenceRecord>,
    policy: &EvidenceGraphPolicy,
    jurisdiction: &str,
) -> bool {
    let Some(evidence) = evidence_by_id.get(evidence_id).copied() else {
        return false;
    };
    policy
        .accepted_evidence_states
        .contains(&evidence.evidence_state)
        && policy.accepted_data_rights.contains(&evidence.data_rights)
        && evidence.jurisdiction == jurisdiction
        && nonempty(&evidence.source_id)
        && evidence
            .source_payload_sha256
            .as_ref()
            .is_some_and(|digest| valid_sha256(digest))
}

fn required_receipt_is_accepted(
    receipt: &EngineeringReceipt,
    target: &ArtifactRecord,
    policy: &EvidenceGraphPolicy,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
    jurisdiction: &str,
) -> bool {
    receipt.design_id == target.design_id
        && receipt.input_artifact_id == target.artifact_id
        && receipt.input_artifact_sha256 == target.content_sha256
        && receipt.run_status == SolverRunStatus::Completed
        && receipt.conclusion == EngineeringConclusion::PassWithinDeclaredScope
        && receipt.review_level >= policy.minimum_review_level
        && receipt
            .solver_id
            .as_ref()
            .is_some_and(|solver_id| nonempty(solver_id))
        && receipt
            .solver_version
            .as_ref()
            .is_some_and(|version| nonempty(version))
        && nonempty(&receipt.declared_scope)
        && !receipt.assumption_evidence_ids.is_empty()
        && receipt
            .assumption_evidence_ids
            .iter()
            .all(|evidence_id| accepted_evidence(evidence_id, evidence_by_id, policy, jurisdiction))
        && (receipt.review_level != ReviewLevel::QualifiedProfessional
            || receipt
                .reviewer_id
                .as_ref()
                .is_some_and(|reviewer| nonempty(reviewer)))
}

fn latest_artifact_revision<'a>(
    artifact: &ArtifactRecord,
    artifacts: &'a [ArtifactRecord],
) -> Option<&'a ArtifactRecord> {
    artifacts
        .iter()
        .filter(|candidate| {
            candidate.design_id == artifact.design_id && candidate.family_id == artifact.family_id
        })
        .max_by(|a, b| {
            a.revision
                .cmp(&b.revision)
                .then_with(|| a.artifact_id.cmp(&b.artifact_id))
        })
}

/// Validate all local references and decision claims under an explicit project policy.
///
/// A clean result means the graph is internally consistent and the configured review
/// gates are represented by the exact approved artifact's receipts. It does not establish
/// that source facts are true, the law was correctly interpreted, or the structure is safe.
pub fn validate_case(
    case: &BuiltEnvironmentCase,
    policy: &EvidenceGraphPolicy,
    evaluation_time_unix_seconds: i64,
) -> Result<Vec<ValidationIssue>, EvidenceGraphError> {
    if !nonempty(&case.case_id) {
        return Err(EvidenceGraphError::EmptyCaseId);
    }
    if !nonempty(&case.jurisdiction) {
        return Err(EvidenceGraphError::EmptyJurisdiction);
    }
    if policy.approvable_artifact_kinds.is_empty() {
        return Err(EvidenceGraphError::EmptyApprovableKinds);
    }
    if policy.accepted_evidence_states.is_empty() {
        return Err(EvidenceGraphError::InvalidPolicyStates);
    }
    if policy.accepted_data_rights.is_empty() {
        return Err(EvidenceGraphError::InvalidPolicyRights);
    }
    if policy.required_disciplines.is_empty() {
        return Err(EvidenceGraphError::EmptyRequiredDisciplineSet);
    }

    let mut issues = Vec::new();

    for id in duplicates(case.evidence.iter().map(|e| e.evidence_id.as_str())) {
        issue(
            &mut issues,
            ValidationCode::DuplicateEvidenceId,
            id.clone(),
            "evidence identifiers must be unique",
        );
    }
    for id in duplicates(case.artifacts.iter().map(|a| a.artifact_id.as_str())) {
        issue(
            &mut issues,
            ValidationCode::DuplicateArtifactId,
            id.clone(),
            "artifact identifiers must be unique",
        );
    }
    for id in duplicates(
        case.engineering_receipts
            .iter()
            .map(|r| r.receipt_id.as_str()),
    ) {
        issue(
            &mut issues,
            ValidationCode::DuplicateReceiptId,
            id.clone(),
            "engineering receipt identifiers must be unique",
        );
    }
    for id in duplicates(case.decisions.iter().map(|d| d.event_id.as_str())) {
        issue(
            &mut issues,
            ValidationCode::DuplicateDecisionId,
            id.clone(),
            "decision event identifiers must be unique",
        );
    }

    let evidence_by_id: BTreeMap<&str, &EvidenceRecord> = case
        .evidence
        .iter()
        .map(|evidence| (evidence.evidence_id.as_str(), evidence))
        .collect();
    let artifacts_by_id: BTreeMap<&str, &ArtifactRecord> = case
        .artifacts
        .iter()
        .map(|artifact| (artifact.artifact_id.as_str(), artifact))
        .collect();
    let receipts_by_id: BTreeMap<&str, &EngineeringReceipt> = case
        .engineering_receipts
        .iter()
        .map(|receipt| (receipt.receipt_id.as_str(), receipt))
        .collect();
    let decisions_by_id: BTreeMap<&str, &DecisionRecord> = case
        .decisions
        .iter()
        .map(|decision| (decision.event_id.as_str(), decision))
        .collect();

    for evidence in &case.evidence {
        if !nonempty(&evidence.evidence_id)
            || !nonempty(&evidence.subject_id)
            || !nonempty(&evidence.field_path)
            || !nonempty(&evidence.source_id)
            || !nonempty(&evidence.jurisdiction)
        {
            issue(
                &mut issues,
                ValidationCode::EmptyIdentifier,
                evidence.evidence_id.clone(),
                "evidence record has an empty identity, source, subject, field path, or jurisdiction",
            );
        }
        if evidence
            .source_payload_sha256
            .as_ref()
            .is_some_and(|digest| !valid_sha256(digest))
        {
            issue(
                &mut issues,
                ValidationCode::InvalidDigest,
                evidence.evidence_id.clone(),
                "evidence payload digest must be a 64-character hexadecimal SHA-256",
            );
        }
        if evidence.evidence_state == EvidenceState::Verified
            && evidence
                .source_payload_sha256
                .as_ref()
                .is_none_or(|digest| !valid_sha256(digest))
        {
            issue(
                &mut issues,
                ValidationCode::VerifiedEvidenceMissingDigest,
                evidence.evidence_id.clone(),
                "verified evidence must bind an exact source-payload digest",
            );
        }
        if matches!(
            evidence.data_rights,
            DataRights::OpenLicense | DataRights::Restricted
        ) && evidence
            .license_id
            .as_ref()
            .is_none_or(|license| !nonempty(license))
        {
            issue(
                &mut issues,
                ValidationCode::EvidenceRightsUnresolved,
                evidence.evidence_id.clone(),
                "open/restricted data rights require an explicit license or terms identifier",
            );
        }
        if evidence.jurisdiction != case.jurisdiction {
            issue(
                &mut issues,
                ValidationCode::EvidenceJurisdictionMismatch,
                evidence.evidence_id.clone(),
                "evidence jurisdiction does not match the case jurisdiction",
            );
        }
        if evidence
            .observed_at_unix_seconds
            .is_some_and(|timestamp| timestamp > evaluation_time_unix_seconds)
        {
            issue(
                &mut issues,
                ValidationCode::FutureTimestamp,
                evidence.evidence_id.clone(),
                "evidence observation timestamp is after the evaluation time",
            );
        }
    }

    for artifact in &case.artifacts {
        if !nonempty(&artifact.artifact_id)
            || !nonempty(&artifact.design_id)
            || !nonempty(&artifact.family_id)
            || !nonempty(&artifact.jurisdiction)
            || artifact.revision == 0
        {
            issue(
                &mut issues,
                ValidationCode::EmptyIdentifier,
                artifact.artifact_id.clone(),
                "artifact requires non-empty IDs/jurisdiction and a positive revision",
            );
        }
        if !valid_sha256(&artifact.content_sha256) {
            issue(
                &mut issues,
                ValidationCode::InvalidDigest,
                artifact.artifact_id.clone(),
                "artifact digest must be a 64-character hexadecimal SHA-256",
            );
        }
        if artifact.jurisdiction != case.jurisdiction {
            issue(
                &mut issues,
                ValidationCode::EvidenceJurisdictionMismatch,
                artifact.artifact_id.clone(),
                "artifact jurisdiction does not match the case jurisdiction",
            );
        }
        if artifact.created_at_unix_seconds > evaluation_time_unix_seconds {
            issue(
                &mut issues,
                ValidationCode::FutureTimestamp,
                artifact.artifact_id.clone(),
                "artifact timestamp is after the evaluation time",
            );
        }
        for evidence_id in &artifact.evidence_ids {
            if !evidence_by_id.contains_key(evidence_id.as_str()) {
                issue(
                    &mut issues,
                    ValidationCode::MissingEvidenceReference,
                    artifact.artifact_id.clone(),
                    format!("artifact references missing evidence {evidence_id}"),
                );
            }
        }
        for parent in &artifact.parent_artifacts {
            match artifacts_by_id.get(parent.artifact_id.as_str()).copied() {
                None => issue(
                    &mut issues,
                    ValidationCode::MissingParentArtifact,
                    artifact.artifact_id.clone(),
                    format!(
                        "artifact references missing parent artifact {}",
                        parent.artifact_id
                    ),
                ),
                Some(parent_record) if parent_record.content_sha256 != parent.content_sha256 => {
                    issue(
                        &mut issues,
                        ValidationCode::StaleParentArtifactDigest,
                        artifact.artifact_id.clone(),
                        format!(
                            "artifact dependency {} does not match the pinned parent digest",
                            parent.artifact_id
                        ),
                    );
                }
                Some(_) => {}
            }
            if !valid_sha256(&parent.content_sha256) {
                issue(
                    &mut issues,
                    ValidationCode::InvalidDigest,
                    artifact.artifact_id.clone(),
                    format!(
                        "parent artifact {} has an invalid pinned digest",
                        parent.artifact_id
                    ),
                );
            }
        }
    }

    let mut revision_keys = BTreeSet::new();
    for artifact in &case.artifacts {
        let key = (
            artifact.design_id.as_str(),
            artifact.family_id.as_str(),
            artifact.revision,
        );
        if !revision_keys.insert(key) {
            issue(
                &mut issues,
                ValidationCode::DuplicateArtifactRevision,
                artifact.artifact_id.clone(),
                "two artifacts in the same design family use the same revision number",
            );
        }
    }

    let artifact_refs: BTreeMap<String, &ArtifactRecord> = case
        .artifacts
        .iter()
        .map(|artifact| (artifact.artifact_id.clone(), artifact))
        .collect();
    if has_dependency_cycle(&artifact_refs) {
        issue(
            &mut issues,
            ValidationCode::ArtifactDependencyCycle,
            case.case_id.clone(),
            "artifact parent relationships must form a directed acyclic graph",
        );
    }

    for receipt in &case.engineering_receipts {
        if !nonempty(&receipt.receipt_id)
            || !nonempty(&receipt.design_id)
            || !nonempty(&receipt.input_artifact_id)
        {
            issue(
                &mut issues,
                ValidationCode::EmptyIdentifier,
                receipt.receipt_id.clone(),
                "engineering receipt identity, design, and input artifact are required",
            );
        }
        match artifacts_by_id
            .get(receipt.input_artifact_id.as_str())
            .copied()
        {
            None => issue(
                &mut issues,
                ValidationCode::MissingInputArtifact,
                receipt.receipt_id.clone(),
                "engineering receipt references a missing input artifact",
            ),
            Some(artifact) => {
                if artifact.design_id != receipt.design_id
                    || artifact.content_sha256 != receipt.input_artifact_sha256
                {
                    issue(
                        &mut issues,
                        ValidationCode::StaleInputDigest,
                        receipt.receipt_id.clone(),
                        "engineering receipt must bind the exact digest and design of its input artifact",
                    );
                }
            }
        }
        if receipt.run_status == SolverRunStatus::Completed
            && (receipt.solver_id.as_ref().is_none_or(|id| !nonempty(id))
                || receipt
                    .solver_version
                    .as_ref()
                    .is_none_or(|version| !nonempty(version)))
        {
            issue(
                &mut issues,
                ValidationCode::SolverMetadataMissing,
                receipt.receipt_id.clone(),
                "completed solver runs require solver identity and version",
            );
        }
        if receipt.conclusion == EngineeringConclusion::PassWithinDeclaredScope
            && receipt.run_status != SolverRunStatus::Completed
        {
            issue(
                &mut issues,
                ValidationCode::ConclusionStatusMismatch,
                receipt.receipt_id.clone(),
                "a passing conclusion requires a completed solver run",
            );
        }
        if !nonempty(&receipt.declared_scope) {
            issue(
                &mut issues,
                ValidationCode::SolverMetadataMissing,
                receipt.receipt_id.clone(),
                "engineering receipt must declare the scope of its conclusion",
            );
        }
        if receipt.review_level == ReviewLevel::QualifiedProfessional
            && receipt
                .reviewer_id
                .as_ref()
                .is_none_or(|reviewer| !nonempty(reviewer))
        {
            issue(
                &mut issues,
                ValidationCode::ProfessionalReviewerMissing,
                receipt.receipt_id.clone(),
                "qualified-professional review level requires a reviewer identity",
            );
        }
        for evidence_id in &receipt.assumption_evidence_ids {
            if !evidence_by_id.contains_key(evidence_id.as_str()) {
                issue(
                    &mut issues,
                    ValidationCode::MissingEvidenceReference,
                    receipt.receipt_id.clone(),
                    format!("receipt references missing assumption evidence {evidence_id}"),
                );
            }
        }
    }

    for decision in &case.decisions {
        if !nonempty(&decision.event_id)
            || !nonempty(&decision.design_id)
            || !nonempty(&decision.target_artifact_id)
            || !nonempty(&decision.actor_id)
        {
            issue(
                &mut issues,
                ValidationCode::EmptyIdentifier,
                decision.event_id.clone(),
                "decision event identity, design, target artifact, and actor are required",
            );
        }
        if decision.decided_at_unix_seconds > evaluation_time_unix_seconds {
            issue(
                &mut issues,
                ValidationCode::FutureTimestamp,
                decision.event_id.clone(),
                "decision timestamp is after the evaluation time",
            );
        }
        let target = artifacts_by_id
            .get(decision.target_artifact_id.as_str())
            .copied();
        match target {
            None => issue(
                &mut issues,
                ValidationCode::MissingInputArtifact,
                decision.event_id.clone(),
                "decision references a missing target artifact",
            ),
            Some(target) => {
                if target.design_id != decision.design_id
                    || target.content_sha256 != decision.target_artifact_sha256
                {
                    issue(
                        &mut issues,
                        ValidationCode::StaleInputDigest,
                        decision.event_id.clone(),
                        "decision must name the exact artifact digest and design revision",
                    );
                }

                if decision.disposition == DecisionDisposition::Approve {
                    if !decision.conditions.is_empty() {
                        issue(
                            &mut issues,
                            ValidationCode::ApprovalHasConditions,
                            decision.event_id.clone(),
                            "approval with outstanding conditions must be recorded as conditional, not approved",
                        );
                    }
                    if !policy.approvable_artifact_kinds.contains(&target.kind) {
                        issue(
                            &mut issues,
                            ValidationCode::ArtifactKindNotApprovable,
                            decision.event_id.clone(),
                            "this artifact kind is not configured as approvable by policy",
                        );
                    }
                    if latest_artifact_revision(target, &case.artifacts)
                        .is_some_and(|latest| latest.artifact_id != target.artifact_id)
                    {
                        issue(
                            &mut issues,
                            ValidationCode::ApprovalForSupersededArtifact,
                            decision.event_id.clone(),
                            "approval targets a superseded artifact revision",
                        );
                    }
                    if policy.require_authority_evidence_for_approval {
                        match decision
                            .authority_evidence_id
                            .as_deref()
                            .and_then(|id| evidence_by_id.get(id).copied())
                        {
                            None => issue(
                                &mut issues,
                                ValidationCode::MissingAuthorityEvidence,
                                decision.event_id.clone(),
                                "approval requires a referenced authority/authorization evidence record",
                            ),
                            Some(evidence)
                                if evidence.evidence_state != EvidenceState::Verified
                                    || !policy
                                        .accepted_evidence_states
                                        .contains(&evidence.evidence_state)
                                    || !policy
                                        .accepted_data_rights
                                        .contains(&evidence.data_rights)
                                    || evidence.jurisdiction != case.jurisdiction =>
                            {
                                issue(
                                    &mut issues,
                                    ValidationCode::AuthorityEvidenceNotAccepted,
                                    decision.event_id.clone(),
                                    "authority evidence is not accepted/verified under the case policy",
                                );
                            }
                            Some(_) => {}
                        }
                    }

                    for evidence_id in &target.evidence_ids {
                        if !accepted_evidence(
                            evidence_id,
                            &evidence_by_id,
                            policy,
                            &case.jurisdiction,
                        ) {
                            issue(
                                &mut issues,
                                ValidationCode::EvidenceRightsUnresolved,
                                decision.event_id.clone(),
                                format!(
                                    "approved artifact depends on evidence {evidence_id} that is not accepted under policy"
                                ),
                            );
                        }
                    }

                    let mut accepted_disciplines = BTreeSet::new();
                    for receipt_id in &decision.engineering_receipt_ids {
                        match receipts_by_id.get(receipt_id.as_str()).copied() {
                            None => issue(
                                &mut issues,
                                ValidationCode::MissingEngineeringReceipt,
                                decision.event_id.clone(),
                                format!(
                                    "decision references missing engineering receipt {receipt_id}"
                                ),
                            ),
                            Some(receipt) => {
                                if !required_receipt_is_accepted(
                                    receipt,
                                    target,
                                    policy,
                                    &evidence_by_id,
                                    &case.jurisdiction,
                                ) {
                                    issue(
                                        &mut issues,
                                        ValidationCode::EngineeringReceiptNotAccepted,
                                        decision.event_id.clone(),
                                        format!(
                                            "receipt {receipt_id} is not a passing, reviewed result for the exact approved artifact"
                                        ),
                                    );
                                } else {
                                    accepted_disciplines.insert(receipt.discipline);
                                }
                            }
                        }
                    }
                    for discipline in &policy.required_disciplines {
                        if !accepted_disciplines.contains(discipline) {
                            issue(
                                &mut issues,
                                ValidationCode::MissingRequiredDiscipline,
                                decision.event_id.clone(),
                                format!(
                                    "approval lacks an accepted engineering receipt for {discipline:?}"
                                ),
                            );
                        }
                    }
                }
            }
        }

        if let Some(supersedes_id) = &decision.supersedes_event_id {
            if supersedes_id == &decision.event_id {
                issue(
                    &mut issues,
                    ValidationCode::InvalidSupersessionReference,
                    decision.event_id.clone(),
                    "a decision event cannot supersede itself",
                );
            }
            match decisions_by_id.get(supersedes_id.as_str()).copied() {
                None => issue(
                    &mut issues,
                    ValidationCode::InvalidSupersessionReference,
                    decision.event_id.clone(),
                    format!("supersedes event {supersedes_id} does not exist"),
                ),
                Some(previous) => {
                    if previous.design_id != decision.design_id {
                        issue(
                            &mut issues,
                            ValidationCode::SupersessionDesignMismatch,
                            decision.event_id.clone(),
                            "superseded decision must belong to the same design",
                        );
                    }
                    if decision.decided_at_unix_seconds < previous.decided_at_unix_seconds {
                        issue(
                            &mut issues,
                            ValidationCode::SupersessionTimeReversal,
                            decision.event_id.clone(),
                            "decision supersession timestamp precedes the superseded event",
                        );
                    }
                    if decision.disposition == DecisionDisposition::Revoke
                        && (previous.disposition != DecisionDisposition::Approve
                            || previous.target_artifact_sha256 != decision.target_artifact_sha256)
                    {
                        issue(
                            &mut issues,
                            ValidationCode::InvalidSupersessionReference,
                            decision.event_id.clone(),
                            "revocation must reference an approval for the same artifact digest",
                        );
                    }
                }
            }
        } else if decision.disposition == DecisionDisposition::Revoke {
            issue(
                &mut issues,
                ValidationCode::InvalidSupersessionReference,
                decision.event_id.clone(),
                "revocation must reference the approval event it revokes",
            );
        }
    }

    issues.sort_by(|a, b| {
        a.item_id
            .cmp(&b.item_id)
            .then_with(|| a.code.cmp(&b.code))
            .then_with(|| a.message.cmp(&b.message))
    });
    Ok(issues)
}


/// Compute the exact downstream revalidation frontier for a proposed artifact digest change.
///
/// This is an advisory graph query. It never mutates the case, rewrites historical
/// receipts, or turns any result into an approval. A malformed or incomplete
/// dependency graph returns GraphInvalid rather than a partial frontier.
pub fn compute_revalidation_frontier(
    case: &BuiltEnvironmentCase,
    changed_artifact_id: &str,
    proposed_content_sha256: &str,
) -> RevalidationFrontier {
    let mut graph_issues = Vec::new();
    let mut add_issue = |code, item_id: String| {
        graph_issues.push(ImpactGraphIssue { code, item_id });
    };

    if !nonempty(changed_artifact_id) {
        add_issue(
            ImpactGraphErrorCode::EmptyChangedArtifactId,
            changed_artifact_id.to_owned(),
        );
    }
    if !valid_sha256(proposed_content_sha256) {
        add_issue(
            ImpactGraphErrorCode::InvalidProposedDigest,
            changed_artifact_id.to_owned(),
        );
    }

    let duplicate_artifacts = duplicates(case.artifacts.iter().map(|item| item.artifact_id.as_str()));
    for id in duplicate_artifacts {
        add_issue(ImpactGraphErrorCode::DuplicateArtifactId, id);
    }
    let duplicate_evidence = duplicates(case.evidence.iter().map(|item| item.evidence_id.as_str()));
    for id in duplicate_evidence {
        add_issue(ImpactGraphErrorCode::DuplicateEvidenceId, id);
    }
    let duplicate_receipts = duplicates(
        case.engineering_receipts.iter().map(|item| item.receipt_id.as_str()),
    );
    for id in duplicate_receipts {
        add_issue(ImpactGraphErrorCode::DuplicateEngineeringReceiptId, id);
    }
    let duplicate_decisions = duplicates(case.decisions.iter().map(|item| item.event_id.as_str()));
    for id in duplicate_decisions {
        add_issue(ImpactGraphErrorCode::DuplicateDecisionId, id);
    }

    let artifacts_by_id: BTreeMap<&str, &ArtifactRecord> = case
        .artifacts
        .iter()
        .map(|item| (item.artifact_id.as_str(), item))
        .collect();
    let evidence_ids: BTreeSet<&str> = case
        .evidence
        .iter()
        .map(|item| item.evidence_id.as_str())
        .collect();
    let receipt_ids: BTreeSet<&str> = case
        .engineering_receipts
        .iter()
        .map(|item| item.receipt_id.as_str())
        .collect();

    let changed_artifact = artifacts_by_id.get(changed_artifact_id).copied();
    if changed_artifact.is_none() {
        add_issue(
            ImpactGraphErrorCode::ChangedArtifactMissing,
            changed_artifact_id.to_owned(),
        );
    }

    for artifact in &case.artifacts {
        if !nonempty(&artifact.artifact_id)
            || !nonempty(&artifact.design_id)
            || !nonempty(&artifact.family_id)
        {
            add_issue(
                ImpactGraphErrorCode::InvalidArtifactIdentity,
                artifact.artifact_id.clone(),
            );
        }
        if !valid_sha256(&artifact.content_sha256) {
            add_issue(
                ImpactGraphErrorCode::InvalidArtifactDigest,
                artifact.artifact_id.clone(),
            );
        }
        for evidence_id in &artifact.evidence_ids {
            if !evidence_ids.contains(evidence_id.as_str()) {
                add_issue(
                    ImpactGraphErrorCode::MissingArtifactEvidence,
                    artifact.artifact_id.clone(),
                );
            }
        }
        for parent in &artifact.parent_artifacts {
            match artifacts_by_id.get(parent.artifact_id.as_str()).copied() {
                None => add_issue(
                    ImpactGraphErrorCode::MissingParentArtifact,
                    artifact.artifact_id.clone(),
                ),
                Some(parent_record) if parent_record.content_sha256 != parent.content_sha256 => {
                    add_issue(
                        ImpactGraphErrorCode::StaleParentArtifactDigest,
                        artifact.artifact_id.clone(),
                    );
                }
                Some(_) => {}
            }
            if !valid_sha256(&parent.content_sha256) {
                add_issue(
                    ImpactGraphErrorCode::InvalidArtifactDigest,
                    artifact.artifact_id.clone(),
                );
            }
        }
    }

    let artifact_map: BTreeMap<String, &ArtifactRecord> = case
        .artifacts
        .iter()
        .map(|item| (item.artifact_id.clone(), item))
        .collect();
    if has_dependency_cycle(&artifact_map) {
        add_issue(
            ImpactGraphErrorCode::ArtifactDependencyCycle,
            case.case_id.clone(),
        );
    }

    for receipt in &case.engineering_receipts {
        if !artifacts_by_id.contains_key(receipt.input_artifact_id.as_str()) {
            add_issue(
                ImpactGraphErrorCode::MissingReceiptInputArtifact,
                receipt.receipt_id.clone(),
            );
        }
        for evidence_id in &receipt.assumption_evidence_ids {
            if !evidence_ids.contains(evidence_id.as_str()) {
                add_issue(
                    ImpactGraphErrorCode::MissingReceiptAssumptionEvidence,
                    receipt.receipt_id.clone(),
                );
            }
        }
    }

    for decision in &case.decisions {
        if !artifacts_by_id.contains_key(decision.target_artifact_id.as_str()) {
            add_issue(
                ImpactGraphErrorCode::MissingDecisionTargetArtifact,
                decision.event_id.clone(),
            );
        }
        for receipt_id in &decision.engineering_receipt_ids {
            if !receipt_ids.contains(receipt_id.as_str()) {
                add_issue(
                    ImpactGraphErrorCode::MissingDecisionEngineeringReceipt,
                    decision.event_id.clone(),
                );
            }
        }
        if decision
            .authority_evidence_id
            .as_ref()
            .is_some_and(|id| !evidence_ids.contains(id.as_str()))
        {
            add_issue(
                ImpactGraphErrorCode::MissingDecisionAuthorityEvidence,
                decision.event_id.clone(),
            );
        }
    }

    graph_issues.sort_by(|left, right| {
        left.item_id
            .cmp(&right.item_id)
            .then_with(|| left.code.cmp(&right.code))
    });
    graph_issues.dedup();
    let previous_content_sha256 = changed_artifact.map(|item| item.content_sha256.clone());

    let mut frontier = RevalidationFrontier {
        status: RevalidationStatus::GraphInvalid,
        changed_artifact_id: changed_artifact_id.to_owned(),
        previous_content_sha256: previous_content_sha256.clone(),
        proposed_content_sha256: proposed_content_sha256.to_owned(),
        impacts: Vec::new(),
        graph_issues,
    };
    if !frontier.graph_issues.is_empty() {
        return frontier;
    }

    let Some(source) = changed_artifact else {
        return frontier;
    };
    if source.content_sha256 == proposed_content_sha256 {
        frontier.status = RevalidationStatus::NoImpact;
        return frontier;
    }

    // Build parent -> child adjacency. Sorting makes the chosen shortest path
    // stable even when the case's artifact vector is permuted.
    let mut children_by_parent: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for artifact in &case.artifacts {
        for parent in &artifact.parent_artifacts {
            children_by_parent
                .entry(parent.artifact_id.clone())
                .or_default()
                .push(artifact.artifact_id.clone());
        }
    }
    for children in children_by_parent.values_mut() {
        children.sort();
        children.dedup();
    }

    let mut affected_paths = BTreeMap::<String, Vec<String>>::new();
    let mut queue = vec![changed_artifact_id.to_owned()];
    affected_paths.insert(
        changed_artifact_id.to_owned(),
        vec![changed_artifact_id.to_owned()],
    );
    let mut queue_index = 0usize;
    while queue_index < queue.len() {
        let parent_id = queue[queue_index].clone();
        queue_index += 1;
        let Some(children) = children_by_parent.get(&parent_id) else {
            continue;
        };
        let parent_path = affected_paths
            .get(&parent_id)
            .cloned()
            .unwrap_or_else(|| vec![parent_id.clone()]);
        for child_id in children {
            if affected_paths.contains_key(child_id) {
                continue;
            }
            let mut path = parent_path.clone();
            path.push(child_id.clone());
            affected_paths.insert(child_id.clone(), path);
            queue.push(child_id.clone());
        }
    }

    let mut impacts = Vec::new();
    for (artifact_id, path) in &affected_paths {
        impacts.push(ImpactRecord {
            node_kind: ImpactNodeKind::Artifact,
            node_id: artifact_id.clone(),
            reason: if artifact_id == changed_artifact_id {
                ImpactReason::SourceArtifactDigestChanged
            } else {
                ImpactReason::DescendantArtifactDependsOnChangedAncestor
            },
            via_artifact_ids: path.clone(),
        });
    }

    // A changed artifact invalidates receipts that consumed that exact artifact
    // (or a descendant). It does not mutate the upstream evidence records linked
    // to the artifact; evidence changes require a distinct evidence-source query.
    let mut affected_receipt_paths = BTreeMap::<String, Vec<String>>::new();
    for receipt in &case.engineering_receipts {
        if let Some(path) = affected_paths.get(&receipt.input_artifact_id).cloned() {
            affected_receipt_paths.insert(receipt.receipt_id.clone(), path.clone());
            impacts.push(ImpactRecord {
                node_kind: ImpactNodeKind::EngineeringReceipt,
                node_id: receipt.receipt_id.clone(),
                reason: ImpactReason::EngineeringReceiptInputArtifactChanged,
                via_artifact_ids: path,
            });
        }
    }

    for decision in &case.decisions {
        let target_path = affected_paths.get(&decision.target_artifact_id).cloned();
        let receipt_path = decision
            .engineering_receipt_ids
            .iter()
            .filter_map(|receipt_id| affected_receipt_paths.get(receipt_id))
            .min()
            .cloned();
        let target_is_affected = target_path.is_some();
        if let Some(path) = target_path.or(receipt_path) {
            impacts.push(ImpactRecord {
                node_kind: ImpactNodeKind::Decision,
                node_id: decision.event_id.clone(),
                reason: if target_is_affected {
                    ImpactReason::DecisionTargetArtifactAffected
                } else {
                    ImpactReason::DecisionDependsOnAffectedReceipt
                },
                via_artifact_ids: path,
            });
        }
    }

    impacts.sort_by(|left, right| {
        left.node_kind
            .cmp(&right.node_kind)
            .then_with(|| left.node_id.cmp(&right.node_id))
            .then_with(|| left.reason.cmp(&right.reason))
            .then_with(|| left.via_artifact_ids.cmp(&right.via_artifact_ids))
    });
    impacts.dedup();
    frontier.status = if impacts.is_empty() {
        RevalidationStatus::NoImpact
    } else {
        RevalidationStatus::RevalidationRequired
    };
    frontier.impacts = impacts;
    frontier
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> EvidenceGraphPolicy {
        EvidenceGraphPolicy {
            approvable_artifact_kinds: [ArtifactKind::DesignModel].into_iter().collect(),
            required_disciplines: [
                EngineeringDiscipline::Structural,
                EngineeringDiscipline::Geotechnical,
            ]
            .into_iter()
            .collect(),
            minimum_review_level: ReviewLevel::QualifiedProfessional,
            accepted_evidence_states: [EvidenceState::Verified].into_iter().collect(),
            accepted_data_rights: [DataRights::PublicDomain, DataRights::OpenLicense]
                .into_iter()
                .collect(),
            require_authority_evidence_for_approval: true,
        }
    }

    fn digest(ch: char) -> String {
        std::iter::repeat_n(ch, 64).collect()
    }

    fn evidence(id: &str) -> EvidenceRecord {
        EvidenceRecord {
            evidence_id: id.into(),
            subject_id: "design-1".into(),
            field_path: "design.assumption".into(),
            source_id: format!("source-{id}"),
            source_payload_sha256: Some(digest('a')),
            evidence_state: EvidenceState::Verified,
            data_rights: DataRights::OpenLicense,
            license_id: Some("CC-BY-4.0".into()),
            jurisdiction: "ZA-GP-JHB".into(),
            observed_at_unix_seconds: Some(1_792_195_200),
        }
    }

    fn artifact(id: &str, revision: u64, sha: char) -> ArtifactRecord {
        ArtifactRecord {
            artifact_id: id.into(),
            design_id: "design-1".into(),
            family_id: "primary-model".into(),
            revision,
            kind: ArtifactKind::DesignModel,
            content_sha256: digest(sha),
            parent_artifacts: Vec::new(),
            evidence_ids: vec!["evidence-1".into()],
            jurisdiction: "ZA-GP-JHB".into(),
            created_at_unix_seconds: 1_792_195_200,
        }
    }

    fn receipt(
        id: &str,
        discipline: EngineeringDiscipline,
        artifact_id: &str,
        artifact_sha: String,
    ) -> EngineeringReceipt {
        EngineeringReceipt {
            receipt_id: id.into(),
            design_id: "design-1".into(),
            input_artifact_id: artifact_id.into(),
            input_artifact_sha256: artifact_sha,
            discipline,
            solver_id: Some("independent-solver".into()),
            solver_version: Some("1.0.0".into()),
            run_status: SolverRunStatus::Completed,
            conclusion: EngineeringConclusion::PassWithinDeclaredScope,
            review_level: ReviewLevel::QualifiedProfessional,
            reviewer_id: Some(format!("reviewer-{id}")),
            assumption_evidence_ids: vec!["evidence-1".into()],
            declared_scope: "synthetic bounded reference case only".into(),
        }
    }

    fn approval(artifact_id: &str, sha: String) -> DecisionRecord {
        DecisionRecord {
            event_id: "approval-1".into(),
            design_id: "design-1".into(),
            target_artifact_id: artifact_id.into(),
            target_artifact_sha256: sha,
            disposition: DecisionDisposition::Approve,
            actor_id: "authorized-reviewer".into(),
            authority_evidence_id: Some("authority-1".into()),
            engineering_receipt_ids: vec!["structural-1".into(), "geotechnical-1".into()],
            decided_at_unix_seconds: 1_792_195_200,
            conditions: Vec::new(),
            supersedes_event_id: None,
        }
    }

    fn valid_case() -> BuiltEnvironmentCase {
        let model = artifact("model-1", 1, 'b');
        BuiltEnvironmentCase {
            case_id: "synthetic-case".into(),
            jurisdiction: "ZA-GP-JHB".into(),
            evidence: vec![evidence("evidence-1"), evidence("authority-1")],
            artifacts: vec![model.clone()],
            engineering_receipts: vec![
                receipt(
                    "structural-1",
                    EngineeringDiscipline::Structural,
                    &model.artifact_id,
                    model.content_sha256.clone(),
                ),
                receipt(
                    "geotechnical-1",
                    EngineeringDiscipline::Geotechnical,
                    &model.artifact_id,
                    model.content_sha256.clone(),
                ),
            ],
            decisions: vec![approval(&model.artifact_id, model.content_sha256.clone())],
        }
    }

    fn case_with_transitive_revalidation_chain() -> BuiltEnvironmentCase {
        let mut case = valid_case();
        let model = case.artifacts[0].clone();

        let mut drawing = artifact("drawing-1", 1, 'd');
        drawing.family_id = "drawing-package".into();
        drawing.kind = ArtifactKind::DrawingPackage;
        drawing.parent_artifacts.push(ArtifactReference {
            artifact_id: model.artifact_id.clone(),
            content_sha256: model.content_sha256.clone(),
        });

        let mut plan = artifact("plan-1", 1, 'e');
        plan.family_id = "construction-plan".into();
        plan.kind = ArtifactKind::ConstructionPlan;
        plan.parent_artifacts.push(ArtifactReference {
            artifact_id: drawing.artifact_id.clone(),
            content_sha256: drawing.content_sha256.clone(),
        });

        let mut unrelated = artifact("unrelated-model", 1, 'f');
        unrelated.family_id = "unrelated-family".into();

        let structural = receipt(
            "structural-plan",
            EngineeringDiscipline::Structural,
            &plan.artifact_id,
            plan.content_sha256.clone(),
        );
        let geotechnical = receipt(
            "geotechnical-plan",
            EngineeringDiscipline::Geotechnical,
            &plan.artifact_id,
            plan.content_sha256.clone(),
        );

        let mut plan_approval = approval(&plan.artifact_id, plan.content_sha256.clone());
        plan_approval.event_id = "approval-plan".into();
        plan_approval.engineering_receipt_ids = vec![
            structural.receipt_id.clone(),
            geotechnical.receipt_id.clone(),
        ];

        case.artifacts.extend([drawing, plan, unrelated]);
        case.engineering_receipts.extend([structural, geotechnical]);
        case.decisions.push(plan_approval);
        case
    }

    fn generated_dependency_case(node_count: usize) -> BuiltEnvironmentCase {
        let mut case = valid_case();
        let mut artifact_digests = BTreeMap::new();
        artifact_digests.insert("model-1".to_string(), digest('b'));

        for index in 0..node_count {
            let id = format!("generated-{index:04}");
            let sha = b"cdef0123456789ab"[index % 16] as char;
            let mut item = artifact(&id, 1, sha);
            item.family_id = format!("generated-family-{index:04}");

            let parent_indices: Vec<usize> = if index == 0 {
                Vec::new()
            } else if index % 11 == 0 {
                Vec::new()
            } else if index % 7 == 0 {
                vec![index - 2, index - 1]
            } else if index % 5 == 0 {
                vec![index / 2]
            } else {
                vec![index - 1]
            };
            if index == 0 {
                item.parent_artifacts.push(ArtifactReference {
                    artifact_id: "model-1".into(),
                    content_sha256: digest('b'),
                });
            } else {
                for parent_index in parent_indices {
                    let parent_id = format!("generated-{parent_index:04}");
                    item.parent_artifacts.push(ArtifactReference {
                        artifact_id: parent_id.clone(),
                        content_sha256: artifact_digests[&parent_id].clone(),
                    });
                }
            }
            artifact_digests.insert(id, item.content_sha256.clone());
            case.artifacts.push(item);
        }

        for index in (0..node_count).filter(|index| index % 3 == 0) {
            let artifact_id = format!("generated-{index:04}");
            let receipt_id = format!("generated-receipt-{index:04}");
            let input_digest = artifact_digests[&artifact_id].clone();
            case.engineering_receipts.push(receipt(
                &receipt_id,
                if index % 2 == 0 {
                    EngineeringDiscipline::Structural
                } else {
                    EngineeringDiscipline::Geotechnical
                },
                &artifact_id,
                input_digest.clone(),
            ));

            if index % 2 == 0 {
                let mut decision = approval(&artifact_id, input_digest);
                decision.event_id = format!("generated-approval-{index:04}");
                decision.engineering_receipt_ids = vec![receipt_id];
                case.decisions.push(decision);
            }
        }
        case
    }

    /// Deliberately simple reference oracle: repeated full edge scans until
    /// no new descendant can be reached. It does not use production adjacency,
    /// BFS helpers, or the production frontier output.
    fn reference_revalidation_nodes(
        case: &BuiltEnvironmentCase,
        source_artifact_id: &str,
    ) -> BTreeSet<(ImpactNodeKind, String, ImpactReason)> {
        let mut affected_artifacts = BTreeSet::from([source_artifact_id.to_owned()]);
        loop {
            let prior_count = affected_artifacts.len();
            for candidate in &case.artifacts {
                if candidate.parent_artifacts.iter().any(|parent| {
                    affected_artifacts.contains(&parent.artifact_id)
                }) {
                    affected_artifacts.insert(candidate.artifact_id.clone());
                }
            }
            if affected_artifacts.len() == prior_count {
                break;
            }
        }

        let affected_receipts = case
            .engineering_receipts
            .iter()
            .filter(|receipt| affected_artifacts.contains(&receipt.input_artifact_id))
            .map(|receipt| receipt.receipt_id.clone())
            .collect::<BTreeSet<_>>();

        let mut expected = BTreeSet::new();
        for artifact_id in &affected_artifacts {
            expected.insert((
                ImpactNodeKind::Artifact,
                artifact_id.clone(),
                if artifact_id == source_artifact_id {
                    ImpactReason::SourceArtifactDigestChanged
                } else {
                    ImpactReason::DescendantArtifactDependsOnChangedAncestor
                },
            ));
        }
        for receipt_id in &affected_receipts {
            expected.insert((
                ImpactNodeKind::EngineeringReceipt,
                receipt_id.clone(),
                ImpactReason::EngineeringReceiptInputArtifactChanged,
            ));
        }
        for decision in &case.decisions {
            let target_affected = affected_artifacts.contains(&decision.target_artifact_id);
            let receipt_affected = decision
                .engineering_receipt_ids
                .iter()
                .any(|receipt_id| affected_receipts.contains(receipt_id));
            if target_affected || receipt_affected {
                expected.insert((
                    ImpactNodeKind::Decision,
                    decision.event_id.clone(),
                    if target_affected {
                        ImpactReason::DecisionTargetArtifactAffected
                    } else {
                        ImpactReason::DecisionDependsOnAffectedReceipt
                    },
                ));
            }
        }
        expected
    }

    fn frontier_nodes(
        frontier: &RevalidationFrontier,
    ) -> BTreeSet<(ImpactNodeKind, String, ImpactReason)> {
        frontier
            .impacts
            .iter()
            .map(|impact| (impact.node_kind, impact.node_id.clone(), impact.reason))
            .collect()
    }

    #[test]
    fn production_frontier_matches_independent_full_scan_oracle_on_generated_dags() {
        let case = generated_dependency_case(128);
        for source_id in ["model-1", "generated-0000", "generated-0035", "generated-0100"] {
            let frontier = compute_revalidation_frontier(&case, source_id, &digest('9'));
            assert_eq!(
                frontier.status,
                RevalidationStatus::RevalidationRequired,
                "source {source_id}"
            );
            assert!(frontier.graph_issues.is_empty(), "source {source_id}");
            assert_eq!(
                frontier_nodes(&frontier),
                reference_revalidation_nodes(&case, source_id),
                "source {source_id}"
            );
        }
    }

    #[test]
    fn unchanged_artifact_digest_has_no_revalidation_impact() {
        let case = valid_case();
        let frontier =
            compute_revalidation_frontier(&case, "model-1", &digest('b'));
        assert_eq!(frontier.status, RevalidationStatus::NoImpact);
        assert!(frontier.graph_issues.is_empty());
        assert!(frontier.impacts.is_empty());
        assert_eq!(frontier.previous_content_sha256, Some(digest('b')));
    }

    #[test]
    fn changed_artifact_reports_transitive_artifact_receipt_and_decision_frontier() {
        let case = case_with_transitive_revalidation_chain();
        let frontier =
            compute_revalidation_frontier(&case, "model-1", &digest('c'));

        assert_eq!(frontier.status, RevalidationStatus::RevalidationRequired);
        assert!(frontier.graph_issues.is_empty());
        assert_eq!(frontier.previous_content_sha256, Some(digest('b')));

        let artifact_impacts = frontier
            .impacts
            .iter()
            .filter(|impact| impact.node_kind == ImpactNodeKind::Artifact)
            .collect::<Vec<_>>();
        assert_eq!(
            artifact_impacts
                .iter()
                .map(|impact| impact.node_id.as_str())
                .collect::<BTreeSet<_>>(),
            ["drawing-1", "model-1", "plan-1"]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
        let plan_impact = artifact_impacts
            .iter()
            .find(|impact| impact.node_id == "plan-1")
            .unwrap();
        assert_eq!(
            plan_impact.via_artifact_ids,
            vec!["model-1", "drawing-1", "plan-1"]
        );

        let receipt_impacts = frontier
            .impacts
            .iter()
            .filter(|impact| impact.node_kind == ImpactNodeKind::EngineeringReceipt)
            .map(|impact| impact.node_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            receipt_impacts,
            ["structural-1", "geotechnical-1", "structural-plan", "geotechnical-plan"]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );

        let decision_impacts = frontier
            .impacts
            .iter()
            .filter(|impact| impact.node_kind == ImpactNodeKind::Decision)
            .map(|impact| impact.node_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            decision_impacts,
            ["approval-1", "approval-plan"].into_iter().collect::<BTreeSet<_>>()
        );

        assert!(!frontier.impacts.iter().any(|impact| {
            impact.node_id == "unrelated-model"
                || impact.node_id == "unrelated-family"
        }));
        assert!(frontier.impacts.iter().all(|impact| {
            impact.node_kind != ImpactNodeKind::Decision
                || impact.reason == ImpactReason::DecisionTargetArtifactAffected
                || impact.reason == ImpactReason::DecisionDependsOnAffectedReceipt
        }));
    }

    #[test]
    fn revalidation_frontier_is_deterministic_under_input_vector_permutation() {
        let case = case_with_transitive_revalidation_chain();
        let expected = compute_revalidation_frontier(&case, "model-1", &digest('c'));

        let mut permuted = case.clone();
        permuted.artifacts.reverse();
        permuted.engineering_receipts.reverse();
        permuted.decisions.reverse();
        permuted.evidence.reverse();

        let actual = compute_revalidation_frontier(&permuted, "model-1", &digest('c'));
        assert_eq!(actual, expected);
    }

    #[test]
    fn malformed_dependency_graph_returns_graph_invalid_without_partial_impacts() {
        let mut case = valid_case();
        let mut child = artifact("drawing-1", 1, 'd');
        child.parent_artifacts.push(ArtifactReference {
            artifact_id: "missing-parent".into(),
            content_sha256: digest('b'),
        });
        case.artifacts.push(child);

        let frontier = compute_revalidation_frontier(&case, "model-1", &digest('c'));
        assert_eq!(frontier.status, RevalidationStatus::GraphInvalid);
        assert!(frontier.impacts.is_empty());
        assert!(frontier.graph_issues.iter().any(|issue| {
            issue.code == ImpactGraphErrorCode::MissingParentArtifact
        }));
    }

    #[test]
    fn cyclic_dependency_graph_returns_graph_invalid() {
        let mut case = valid_case();
        let mut second = artifact("model-2", 2, 'd');
        second.parent_artifacts.push(ArtifactReference {
            artifact_id: "model-1".into(),
            content_sha256: digest('b'),
        });
        case.artifacts[0].parent_artifacts.push(ArtifactReference {
            artifact_id: "model-2".into(),
            content_sha256: digest('d'),
        });
        case.artifacts.push(second);

        let frontier = compute_revalidation_frontier(&case, "model-1", &digest('c'));
        assert_eq!(frontier.status, RevalidationStatus::GraphInvalid);
        assert!(frontier.impacts.is_empty());
        assert!(frontier.graph_issues.iter().any(|issue| {
            issue.code == ImpactGraphErrorCode::ArtifactDependencyCycle
        }));
    }

    #[test]
    fn invalid_proposed_digest_returns_graph_invalid() {
        let case = valid_case();
        let frontier = compute_revalidation_frontier(&case, "model-1", "not-a-digest");
        assert_eq!(frontier.status, RevalidationStatus::GraphInvalid);
        assert!(frontier.impacts.is_empty());
        assert!(frontier.graph_issues.iter().any(|issue| {
            issue.code == ImpactGraphErrorCode::InvalidProposedDigest
        }));
    }

    #[test]
    fn changed_artifact_does_not_invalidate_unrelated_upstream_evidence_or_receipts() {
        let mut case = case_with_transitive_revalidation_chain();
        let mut unrelated_receipt = receipt(
            "unrelated-receipt",
            EngineeringDiscipline::Structural,
            "unrelated-model",
            digest('f'),
        );
        unrelated_receipt.assumption_evidence_ids = vec!["evidence-1".into()];
        case.engineering_receipts.push(unrelated_receipt);

        let frontier = compute_revalidation_frontier(&case, "model-1", &digest('c'));
        assert!(!frontier.impacts.iter().any(|impact| {
            impact.node_id == "unrelated-receipt"
        }));
    }

    #[test]
    fn exact_inputs_and_reviewed_receipts_form_a_consistent_approval_record() {
        let case = valid_case();
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(result.is_empty(), "{result:#?}");
    }

    #[test]
    fn changed_artifact_digest_invalidates_old_engineering_receipt() {
        let mut case = valid_case();
        let current = &mut case.artifacts[0];
        current.content_sha256 = digest('c');
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::StaleInputDigest)
        );
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::EngineeringReceiptNotAccepted)
        );
    }

    #[test]
    fn approval_for_superseded_revision_is_blocked() {
        let mut case = valid_case();
        case.artifacts.push(artifact("model-2", 2, 'd'));
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::ApprovalForSupersededArtifact)
        );
    }

    #[test]
    fn unknown_authority_evidence_cannot_authorize_approval() {
        let mut case = valid_case();
        case.evidence[1].evidence_state = EvidenceState::Unknown;
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::AuthorityEvidenceNotAccepted)
        );
    }

    #[test]
    fn failed_or_nonconverged_solver_cannot_satisfy_a_required_discipline() {
        let mut case = valid_case();
        case.engineering_receipts[0].run_status = SolverRunStatus::NonConverged;
        case.engineering_receipts[0].conclusion = EngineeringConclusion::Indeterminate;
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::EngineeringReceiptNotAccepted)
        );
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::MissingRequiredDiscipline)
        );
    }

    #[test]
    fn missing_evidence_reference_and_parent_are_explicit_issues() {
        let mut case = valid_case();
        case.artifacts[0].evidence_ids.push("missing-source".into());
        case.artifacts[0].parent_artifacts.push(ArtifactReference {
            artifact_id: "missing-parent".into(),
            content_sha256: digest('d'),
        });
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::MissingEvidenceReference)
        );
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::MissingParentArtifact)
        );
    }

    #[test]
    fn changed_parent_digest_invalidates_child_lineage() {
        let mut case = valid_case();
        let mut child = artifact("drawing-1", 1, 'd');
        child.family_id = "drawing-package".into();
        child.parent_artifacts.push(ArtifactReference {
            artifact_id: "model-1".into(),
            content_sha256: digest('c'),
        });
        case.artifacts.push(child);
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::StaleParentArtifactDigest)
        );
    }

    #[test]
    fn receipts_with_stale_assumption_evidence_cannot_support_approval() {
        let mut case = valid_case();
        case.engineering_receipts[0].assumption_evidence_ids = vec!["stale-assumption".into()];
        case.evidence.push(EvidenceRecord {
            evidence_id: "stale-assumption".into(),
            evidence_state: EvidenceState::Stale,
            ..evidence("stale-assumption")
        });
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::EngineeringReceiptNotAccepted)
        );
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::MissingRequiredDiscipline)
        );
    }

    #[test]
    fn approval_with_conditions_must_not_be_reported_as_unconditional() {
        let mut case = valid_case();
        case.decisions[0]
            .conditions
            .push("Await final local authority sign-off".into());
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::ApprovalHasConditions)
        );
    }

    #[test]
    fn cycle_detection_is_iterative_for_deep_artifact_chains() {
        let node_count = 10_000usize;
        let mut nodes = (0..node_count)
            .map(|index| artifact(&format!("deep-node-{index:05}"), 1, 'b'))
            .collect::<Vec<_>>();
        for index in 1..node_count {
            nodes[index].parent_artifacts.push(ArtifactReference {
                artifact_id: format!("deep-node-{:05}", index - 1),
                content_sha256: digest('b'),
            });
        }
        let acyclic = nodes
            .iter()
            .map(|node| (node.artifact_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        assert!(!has_dependency_cycle(&acyclic));

        nodes[0].parent_artifacts.push(ArtifactReference {
            artifact_id: format!("deep-node-{:05}", node_count - 1),
            content_sha256: digest('b'),
        });
        let cyclic = nodes
            .iter()
            .map(|node| (node.artifact_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        assert!(has_dependency_cycle(&cyclic));
    }

    #[test]
    fn cyclic_artifact_lineage_is_rejected() {
        let mut case = valid_case();
        let mut second = artifact("model-2", 2, 'd');
        second.parent_artifacts.push(ArtifactReference {
            artifact_id: "model-1".into(),
            content_sha256: digest('b'),
        });
        case.artifacts[0].parent_artifacts.push(ArtifactReference {
            artifact_id: "model-2".into(),
            content_sha256: digest('d'),
        });
        case.artifacts.push(second);
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::ArtifactDependencyCycle)
        );
    }

    #[test]
    fn malformed_hash_and_missing_professional_reviewer_are_flagged() {
        let mut case = valid_case();
        case.artifacts[0].content_sha256 = "not-a-digest".into();
        case.engineering_receipts[0].reviewer_id = None;
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::InvalidDigest)
        );
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::ProfessionalReviewerMissing)
        );
    }

    #[test]
    fn decision_event_cannot_supersede_itself() {
        let mut case = valid_case();
        case.decisions[0].supersedes_event_id = Some("approval-1".into());
        let result = validate_case(&case, &policy(), 1_792_195_200).unwrap();
        assert!(
            result
                .iter()
                .any(|issue| issue.code == ValidationCode::InvalidSupersessionReference)
        );
    }

    #[test]
    fn revocation_must_reference_the_approved_event() {
        let mut case = valid_case();
        case.decisions.push(DecisionRecord {
            event_id: "revoke-1".into(),
            design_id: "design-1".into(),
            target_artifact_id: "model-1".into(),
            target_artifact_sha256: digest('b'),
            disposition: DecisionDisposition::Revoke,
            actor_id: "authority-1".into(),
            authority_evidence_id: Some("authority-1".into()),
            engineering_receipt_ids: Vec::new(),
            decided_at_unix_seconds: 1_792_195_300,
            conditions: Vec::new(),
            supersedes_event_id: Some("approval-1".into()),
        });
        let result = validate_case(&case, &policy(), 1_792_195_300).unwrap();
        assert!(result.is_empty(), "{result:#?}");
    }
}
