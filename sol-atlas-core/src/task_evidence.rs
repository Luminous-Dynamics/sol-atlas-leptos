// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-completeness evaluation for revision-bound field-task review packets.
//!
//! A successful evaluation means that the submitted metadata appears complete
//! for a qualified reviewer to inspect. It does not prove that an artifact is
//! genuine, that a measurement is accurate, that a reviewer is qualified, or
//! that real work is safe/authorized. Those checks belong to trusted adapters
//! and the canonical service.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const TASK_EVIDENCE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEvidencePacketV1 {
    pub schema_version: u16,
    pub graph_id: String,
    pub graph_revision: u64,
    pub task_id: String,
    pub procedure_revision: u64,
    pub prepared_by: String,
    pub requirements: Vec<EvidenceRequirementV1>,
    pub evidence: Vec<EvidenceArtifactV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirementV1 {
    pub requirement_id: String,
    pub accepted_kind: EvidenceKind,
    /// If present, matching evidence must be bound to exactly this procedure revision.
    pub required_procedure_revision: Option<u64>,
    /// Requires the assigned reviewer to differ from every artifact recorder.
    pub requires_independent_reviewer: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceArtifactV1 {
    pub evidence_id: String,
    pub requirement_id: String,
    pub kind: EvidenceKind,
    /// Opaque stable reference to the artifact or authoritative source record.
    pub source_ref: String,
    /// SHA-256 of canonical artifact bytes; this module validates encoding only.
    pub digest_sha256: String,
    pub recorded_by: String,
    pub captured_at_unix_ms: Option<u64>,
    pub unit: Option<String>,
    pub instrument_ref: Option<String>,
    pub procedure_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Observation,
    Measurement,
    Photograph,
    TestResult,
    ProcedureChecklist,
    ImportedRecord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationContext<'a> {
    /// Current authoritative projection identity (not supplied by the packet).
    pub graph_id: &'a str,
    /// Current authoritative graph revision.
    pub graph_revision: u64,
    /// Task for which this review packet is being evaluated.
    pub task_id: &'a str,
    /// Reviewer identity selected by the caller/authority layer, if known.
    pub reviewer_id: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDecision {
    Incomplete,
    ReadyForQualifiedReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceIssueCode {
    UnsupportedSchema,
    EmptyPacketIdentity,
    StaleGraphIdentity,
    StaleGraphRevision,
    WrongTask,
    InvalidProcedureRevision,
    EmptyRequirements,
    DuplicateRequirementId,
    DuplicateEvidenceId,
    UnknownRequirement,
    MissingRequiredEvidence,
    WrongEvidenceKind,
    MissingSourceReference,
    MissingRecorder,
    InvalidDigestEncoding,
    MeasurementMissingUnit,
    MeasurementMissingInstrument,
    ProcedureRevisionMismatch,
    IndependentReviewerMissing,
    ReviewerEqualsRecorder,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceIssue {
    pub code: EvidenceIssueCode,
    pub requirement_id: Option<String>,
    pub evidence_id: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceEvaluation {
    pub decision: EvidenceDecision,
    pub required_count: usize,
    pub satisfied_count: usize,
    pub issues: Vec<EvidenceIssue>,
}

impl EvidenceEvaluation {
    /// Metadata completeness percentage; 100% does not mean evidence is authentic
    /// or that the operation is authorized.
    pub fn completeness_percent(&self) -> u8 {
        if self.required_count == 0 {
            return 0;
        }
        ((self.satisfied_count.min(self.required_count) * 100) / self.required_count) as u8
    }

    pub fn is_ready_for_review(&self) -> bool {
        self.decision == EvidenceDecision::ReadyForQualifiedReview && self.issues.is_empty()
    }
}

/// Evaluate a packet against current graph/task identity and an optional assigned reviewer.
///
/// This is a pure completeness/consistency check. Call the canonical service to
/// authorize transitions, verify digests, validate credentials and persist a
/// revision-bound decision.
pub fn evaluate_evidence_packet(
    packet: &TaskEvidencePacketV1,
    context: &VerificationContext<'_>,
) -> EvidenceEvaluation {
    let mut issues = Vec::new();

    if packet.schema_version != TASK_EVIDENCE_SCHEMA_VERSION {
        issues.push(issue(
            EvidenceIssueCode::UnsupportedSchema,
            None,
            None,
            format!(
                "unsupported evidence packet schema {}",
                packet.schema_version
            ),
        ));
    }
    if packet.graph_id.trim().is_empty()
        || packet.task_id.trim().is_empty()
        || packet.prepared_by.trim().is_empty()
    {
        issues.push(issue(
            EvidenceIssueCode::EmptyPacketIdentity,
            None,
            None,
            "graph, task, and preparer identifiers must be non-empty",
        ));
    }
    if packet.graph_id != context.graph_id {
        issues.push(issue(
            EvidenceIssueCode::StaleGraphIdentity,
            None,
            None,
            "packet graph identity does not match current graph",
        ));
    }
    if packet.graph_revision != context.graph_revision {
        issues.push(issue(
            EvidenceIssueCode::StaleGraphRevision,
            None,
            None,
            "packet graph revision does not match the current graph revision",
        ));
    }
    if packet.task_id != context.task_id {
        issues.push(issue(
            EvidenceIssueCode::WrongTask,
            None,
            None,
            "packet task identity does not match the task being reviewed",
        ));
    }
    if packet.procedure_revision == 0 || context.expected_procedure_revision == 0 {
        issues.push(issue(
            EvidenceIssueCode::InvalidProcedureRevision,
            None,
            None,
            "packet and authoritative procedure revisions must be positive",
        ));
    } else if packet.procedure_revision != context.expected_procedure_revision {
        issues.push(issue(
            EvidenceIssueCode::ProcedureRevisionMismatch,
            None,
            None,
            "packet procedure revision does not match the authoritative procedure revision",
        ));
    }
    if context.expected_requirements.is_empty() {
        issues.push(issue(
            EvidenceIssueCode::EmptyRequirements,
            None,
            None,
            "authoritative task contract contains no evidence requirements",
        ));
    }

    // Build the authority-supplied requirement map first. Evidence is always
    // evaluated against this set, never against a caller's self-declared subset.
    let mut requirements_by_id: std::collections::BTreeMap<String, EvidenceRequirementV1> =
        std::collections::BTreeMap::new();
    let mut authoritative_requirements_valid = true;
    for requirement in &context.expected_requirements {
        if requirement.requirement_id.trim().is_empty()
            || requirements_by_id.contains_key(&requirement.requirement_id)
        {
            authoritative_requirements_valid = false;
            issues.push(issue(
                EvidenceIssueCode::DuplicateRequirementId,
                Some(requirement.requirement_id.clone()),
                None,
                "authoritative requirement identifier is empty or duplicated",
            ));
            continue;
        }
        if requirement.required_procedure_revision == Some(0) {
            authoritative_requirements_valid = false;
            issues.push(issue(
                EvidenceIssueCode::InvalidProcedureRevision,
                Some(requirement.requirement_id.clone()),
                None,
                "authoritative required procedure revision must be positive",
            ));
        }
        if let Some(required_revision) = requirement.required_procedure_revision {
            if required_revision != context.expected_procedure_revision {
                authoritative_requirements_valid = false;
                issues.push(issue(
                    EvidenceIssueCode::ProcedureRevisionMismatch,
                    Some(requirement.requirement_id.clone()),
                    None,
                    "authoritative requirement revision disagrees with the task procedure revision",
                ));
            }
        }
        requirements_by_id.insert(requirement.requirement_id.clone(), requirement.clone());
    }

    // A packet can carry a copy for display/transport, but it cannot edit the
    // requirement set and then claim completeness against its own modified copy.
    let mut packet_requirements_by_id: std::collections::BTreeMap<String, EvidenceRequirementV1> =
        std::collections::BTreeMap::new();
    let mut packet_requirements_unique = true;
    for requirement in &packet.requirements {
        if requirement.requirement_id.trim().is_empty()
            || packet_requirements_by_id.contains_key(&requirement.requirement_id)
        {
            packet_requirements_unique = false;
            issues.push(issue(
                EvidenceIssueCode::DuplicateRequirementId,
                Some(requirement.requirement_id.clone()),
                None,
                "packet requirement identifier is empty or duplicated",
            ));
            continue;
        }
        packet_requirements_by_id.insert(requirement.requirement_id.clone(), requirement.clone());
    }
    let requirements_match_authority = packet_requirements_unique
        && packet_requirements_by_id == requirements_by_id
        && packet.requirements.len() == context.expected_requirements.len();
    if !requirements_match_authority {
        issues.push(issue(
            EvidenceIssueCode::RequirementsDoNotMatchAuthority,
            None,
            None,
            "packet requirements must exactly match the authoritative task/procedure requirement set",
        ));
    }

    let mut evidence_ids = BTreeSet::new();
    let mut artifacts_by_requirement: std::collections::BTreeMap<String, Vec<&EvidenceArtifactV1>> =
        std::collections::BTreeMap::new();

    for artifact in &packet.evidence {
        let mut artifact_valid = true;

        if artifact.evidence_id.trim().is_empty()
            || !evidence_ids.insert(artifact.evidence_id.as_str())
        {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::DuplicateEvidenceId,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "evidence identifier is empty or duplicated",
            ));
        }
        if artifact.source_ref.trim().is_empty() {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::MissingSourceReference,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "evidence source reference must be non-empty",
            ));
        }
        if artifact.recorded_by.trim().is_empty() {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::MissingRecorder,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "evidence recorder identity must be non-empty",
            ));
        }
        if !is_lowercase_sha256(&artifact.digest_sha256) {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::InvalidDigestEncoding,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "digest must be 64 lowercase hexadecimal characters",
            ));
        }
        if artifact.kind == EvidenceKind::Measurement
            && artifact
                .unit
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::MeasurementMissingUnit,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "measurement evidence requires a non-empty unit",
            ));
        }
        if artifact.kind == EvidenceKind::Measurement
            && artifact
                .instrument_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::MeasurementMissingInstrument,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "measurement evidence requires an instrument reference",
            ));
        }
        if artifact.procedure_revision == Some(0) {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::InvalidProcedureRevision,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "evidence procedure revision must be positive",
            ));
        }

        let Some(requirement) = requirements_by_id.get(&artifact.requirement_id) else {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::UnknownRequirement,
                Some(artifact.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "evidence references no requirement in the authoritative task/procedure set",
            ));
            continue;
        };

        if artifact.kind != requirement.accepted_kind {
            artifact_valid = false;
            issues.push(issue(
                EvidenceIssueCode::WrongEvidenceKind,
                Some(requirement.requirement_id.clone()),
                Some(artifact.evidence_id.clone()),
                "evidence kind does not match the authoritative requirement",
            ));
        }
        if let Some(required_revision) = requirement.required_procedure_revision {
            if artifact.procedure_revision != Some(required_revision) {
                artifact_valid = false;
                issues.push(issue(
                    EvidenceIssueCode::ProcedureRevisionMismatch,
                    Some(requirement.requirement_id.clone()),
                    Some(artifact.evidence_id.clone()),
                    format!(
                        "evidence procedure revision does not match required revision {required_revision}"
                    ),
                ));
            }
        }

        if requirement.requires_independent_reviewer {
            match context.reviewer_id {
                None if artifact_valid => {
                    artifact_valid = false;
                    issues.push(issue(
                        EvidenceIssueCode::IndependentReviewerMissing,
                        Some(requirement.requirement_id.clone()),
                        Some(artifact.evidence_id.clone()),
                        "an independent reviewer must be assigned",
                    ));
                }
                Some(reviewer) if reviewer.trim().is_empty() && artifact_valid => {
                    artifact_valid = false;
                    issues.push(issue(
                        EvidenceIssueCode::IndependentReviewerMissing,
                        Some(requirement.requirement_id.clone()),
                        Some(artifact.evidence_id.clone()),
                        "reviewer identity must be non-empty",
                    ));
                }
                Some(reviewer) if reviewer == artifact.recorded_by && artifact_valid => {
                    artifact_valid = false;
                    issues.push(issue(
                        EvidenceIssueCode::ReviewerEqualsRecorder,
                        Some(requirement.requirement_id.clone()),
                        Some(artifact.evidence_id.clone()),
                        "independent reviewer must differ from the evidence recorder",
                    ));
                }
                _ => {}
            }
        }

        if artifact_valid {
            artifacts_by_requirement
                .entry(requirement.requirement_id.clone())
                .or_default()
                .push(artifact);
        }
    }

    let mut satisfied_count = 0;
    for requirement in &context.expected_requirements {
        let valid_artifacts = artifacts_by_requirement
            .get(&requirement.requirement_id)
            .map_or(0, Vec::len);
        if valid_artifacts == 0 {
            issues.push(issue(
                EvidenceIssueCode::MissingRequiredEvidence,
                Some(requirement.requirement_id.clone()),
                None,
                "no valid evidence artifact satisfies this authoritative requirement",
            ));
        } else {
            satisfied_count += 1;
        }
    }

    let identity_valid = packet.schema_version == TASK_EVIDENCE_SCHEMA_VERSION
        && packet.graph_id == context.graph_id
        && packet.graph_revision == context.graph_revision
        && packet.task_id == context.task_id
        && packet.graph_id.trim() != ""
        && packet.task_id.trim() != ""
        && packet.prepared_by.trim() != ""
        && packet.procedure_revision == context.expected_procedure_revision
        && context.expected_procedure_revision > 0;

    let ready = issues.is_empty()
        && identity_valid
        && authoritative_requirements_valid
        && requirements_match_authority
        && satisfied_count == context.expected_requirements.len()
        && !context.expected_requirements.is_empty();

    EvidenceEvaluation {
        decision: if ready {
            EvidenceDecision::ReadyForQualifiedReview
        } else {
            EvidenceDecision::Incomplete
        },
        required_count: context.expected_requirements.len(),
        satisfied_count,
        issues,
    }
}
fn issue(
    code: EvidenceIssueCode,
    requirement_id: Option<String>,
    evidence_id: Option<String>,
    detail: impl Into<String>,
) -> EvidenceIssue {
    EvidenceIssue {
        code,
        requirement_id,
        evidence_id,
        detail: detail.into(),
    }
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        EvidenceArtifactV1, EvidenceDecision, EvidenceIssueCode, EvidenceKind,
        EvidenceRequirementV1, TASK_EVIDENCE_SCHEMA_VERSION, TaskEvidencePacketV1,
        VerificationContext, evaluate_evidence_packet,
    };

    fn requirement(id: &str, kind: EvidenceKind, independent: bool) -> EvidenceRequirementV1 {
        EvidenceRequirementV1 {
            requirement_id: id.to_owned(),
            accepted_kind: kind,
            required_procedure_revision: Some(7),
            requires_independent_reviewer: independent,
        }
    }

    fn packet() -> TaskEvidencePacketV1 {
        TaskEvidencePacketV1 {
            schema_version: TASK_EVIDENCE_SCHEMA_VERSION,
            graph_id: "job-graph-demo-001".to_owned(),
            graph_revision: 4,
            task_id: "functional-test".to_owned(),
            procedure_revision: 7,
            prepared_by: "technician-01".to_owned(),
            requirements: vec![
                requirement("observation", EvidenceKind::Observation, false),
                requirement("measurement", EvidenceKind::Measurement, true),
            ],
            evidence: vec![
                EvidenceArtifactV1 {
                    evidence_id: "evidence-observation-01".to_owned(),
                    requirement_id: "observation".to_owned(),
                    kind: EvidenceKind::Observation,
                    source_ref: "urn:luminous:synthetic:observation-01".to_owned(),
                    digest_sha256: "a".repeat(64),
                    recorded_by: "technician-01".to_owned(),
                    captured_at_unix_ms: Some(1_797_000_000_000),
                    unit: None,
                    instrument_ref: None,
                    procedure_revision: Some(7),
                },
                EvidenceArtifactV1 {
                    evidence_id: "evidence-measurement-01".to_owned(),
                    requirement_id: "measurement".to_owned(),
                    kind: EvidenceKind::Measurement,
                    source_ref: "urn:luminous:synthetic:measurement-01".to_owned(),
                    digest_sha256: "b".repeat(64),
                    recorded_by: "technician-01".to_owned(),
                    captured_at_unix_ms: Some(1_797_000_000_010),
                    unit: Some("kPa".to_owned()),
                    instrument_ref: Some("instrument:pressure-gauge-09".to_owned()),
                    procedure_revision: Some(7),
                },
            ],
        }
    }

    fn context<'a>(reviewer_id: Option<&'a str>) -> VerificationContext<'a> {
        VerificationContext {
            graph_id: "job-graph-demo-001",
            graph_revision: 4,
            task_id: "functional-test",
            expected_procedure_revision: 7,
            expected_requirements: vec![
                requirement("observation", EvidenceKind::Observation, false),
                requirement("measurement", EvidenceKind::Measurement, true),
            ],
            reviewer_id,
        }
    }

    fn has_issue(evaluation: &super::EvidenceEvaluation, code: EvidenceIssueCode) -> bool {
        evaluation.issues.iter().any(|item| item.code == code)
    }

    #[test]
    fn complete_packet_is_ready_for_qualified_review_not_marked_verified() {
        let result = evaluate_evidence_packet(&packet(), &context(Some("reviewer-02")));
        assert_eq!(result.decision, EvidenceDecision::ReadyForQualifiedReview);
        assert_eq!(result.required_count, 2);
        assert_eq!(result.satisfied_count, 2);
        assert_eq!(result.completeness_percent(), 100);
        assert!(result.is_ready_for_review());
    }

    #[test]
    fn missing_evidence_blocks_review_readiness() {
        let mut value = packet();
        value.evidence.pop();
        let result = evaluate_evidence_packet(&value, &context(Some("reviewer-02")));
        assert_eq!(result.decision, EvidenceDecision::Incomplete);
        assert_eq!(result.completeness_percent(), 50);
        assert!(has_issue(
            &result,
            EvidenceIssueCode::MissingRequiredEvidence
        ));
    }

    #[test]
    fn stale_graph_revision_and_wrong_task_fail_closed() {
        let value = packet();
        let mut stale = context(Some("reviewer-02"));
        stale.graph_revision = 5;
        assert!(has_issue(
            &evaluate_evidence_packet(&value, &stale),
            EvidenceIssueCode::StaleGraphRevision
        ));
        stale = context(Some("reviewer-02"));
        stale.task_id = "handover";
        assert!(has_issue(
            &evaluate_evidence_packet(&value, &stale),
            EvidenceIssueCode::WrongTask
        ));
    }

    #[test]
    fn malformed_digest_does_not_count_as_satisfying_evidence() {
        let mut value = packet();
        value.evidence[0].digest_sha256 = "A".repeat(64);
        let result = evaluate_evidence_packet(&value, &context(Some("reviewer-02")));
        assert!(has_issue(&result, EvidenceIssueCode::InvalidDigestEncoding));
        assert_eq!(result.satisfied_count, 1);
        assert_eq!(result.decision, EvidenceDecision::Incomplete);
    }

    #[test]
    fn measurement_requires_unit_and_instrument() {
        let mut value = packet();
        value.evidence[1].unit = None;
        value.evidence[1].instrument_ref = None;
        let result = evaluate_evidence_packet(&value, &context(Some("reviewer-02")));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::MeasurementMissingUnit
        ));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::MeasurementMissingInstrument
        ));
        assert_eq!(result.decision, EvidenceDecision::Incomplete);
    }

    #[test]
    fn evidence_and_packet_must_match_the_required_procedure_revision() {
        let mut value = packet();
        value.evidence[1].procedure_revision = Some(6);
        let result = evaluate_evidence_packet(&value, &context(Some("reviewer-02")));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::ProcedureRevisionMismatch
        ));
        assert_eq!(result.decision, EvidenceDecision::Incomplete);
    }

    #[test]
    fn independent_review_requires_a_distinct_nonempty_reviewer() {
        let value = packet();
        let self_review = evaluate_evidence_packet(&value, &context(Some("technician-01")));
        assert!(has_issue(
            &self_review,
            EvidenceIssueCode::ReviewerEqualsRecorder
        ));
        let no_reviewer = evaluate_evidence_packet(&value, &context(None));
        assert!(has_issue(
            &no_reviewer,
            EvidenceIssueCode::IndependentReviewerMissing
        ));
        assert_eq!(no_reviewer.decision, EvidenceDecision::Incomplete);
    }

    #[test]
    fn duplicate_evidence_id_and_unknown_requirement_are_rejected() {
        let mut duplicate = packet();
        duplicate.evidence[1].evidence_id = duplicate.evidence[0].evidence_id.clone();
        let result = evaluate_evidence_packet(&duplicate, &context(Some("reviewer-02")));
        assert!(has_issue(&result, EvidenceIssueCode::DuplicateEvidenceId));
        assert_eq!(result.decision, EvidenceDecision::Incomplete);

        let mut unknown = packet();
        unknown.evidence[0].requirement_id = "undeclared".to_owned();
        let result = evaluate_evidence_packet(&unknown, &context(Some("reviewer-02")));
        assert!(has_issue(&result, EvidenceIssueCode::UnknownRequirement));
        assert_eq!(result.decision, EvidenceDecision::Incomplete);
    }

    #[test]
    fn wrong_kind_and_duplicate_requirements_are_rejected() {
        let mut wrong_kind = packet();
        wrong_kind.evidence[0].kind = EvidenceKind::ImportedRecord;
        let result = evaluate_evidence_packet(&wrong_kind, &context(Some("reviewer-02")));
        assert!(has_issue(&result, EvidenceIssueCode::WrongEvidenceKind));

        let mut duplicate_requirement = packet();
        duplicate_requirement
            .requirements
            .push(duplicate_requirement.requirements[0].clone());
        let result =
            evaluate_evidence_packet(&duplicate_requirement, &context(Some("reviewer-02")));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::DuplicateRequirementId
        ));
    }

    #[test]
    fn packet_cannot_omit_or_rewrite_authoritative_requirements() {
        let mut omitted = packet();
        let missing_id = omitted.requirements[1].requirement_id.clone();
        omitted.requirements.pop();
        omitted
            .evidence
            .retain(|artifact| artifact.requirement_id != missing_id);
        let result = evaluate_evidence_packet(&omitted, &context(Some("reviewer-02")));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::RequirementsDoNotMatchAuthority
        ));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::MissingRequiredEvidence
        ));
        assert_eq!(result.required_count, 2);
        assert_eq!(result.decision, EvidenceDecision::Incomplete);

        let mut rewritten = packet();
        rewritten.requirements[1].accepted_kind = EvidenceKind::Observation;
        rewritten.evidence[1].kind = EvidenceKind::Observation;
        rewritten.evidence[1].unit = None;
        rewritten.evidence[1].instrument_ref = None;
        let result = evaluate_evidence_packet(&rewritten, &context(Some("reviewer-02")));
        assert!(has_issue(
            &result,
            EvidenceIssueCode::RequirementsDoNotMatchAuthority
        ));
        assert!(has_issue(&result, EvidenceIssueCode::WrongEvidenceKind));
        assert_eq!(result.decision, EvidenceDecision::Incomplete);
    }

    #[test]
    fn fixture_deserializes_and_is_ready_for_review() {
        let value: TaskEvidencePacketV1 =
            serde_json::from_str(include_str!("../fixtures/task-evidence-packet-v1.json"))
                .expect("fixture matches task evidence packet schema");
        let result = evaluate_evidence_packet(
            &value,
            &VerificationContext {
                graph_id: "job-graph-demo-001",
                graph_revision: 4,
                task_id: "functional-test",
                expected_procedure_revision: value.procedure_revision,
                expected_requirements: value.requirements.clone(),
                reviewer_id: Some("reviewer-02"),
            },
        );
        assert!(result.is_ready_for_review());
    }
}
