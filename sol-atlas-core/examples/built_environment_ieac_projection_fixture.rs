// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emits one synthetic BuiltEnvironmentCase -> IEAC v0.2 projection.
//! Synthetic records are not field evidence, approvals, or safety claims.

use serde_json::json;
use sol_atlas_core::built_environment_case::{
    ArtifactKind, ArtifactRecord, ArtifactReference, BuiltEnvironmentCase, DataRights, DecisionDisposition,
    DecisionRecord, EngineeringConclusion, EngineeringDiscipline, EngineeringReceipt,
    EvidenceRecord, EvidenceState, ReviewLevel, SolverRunStatus,
};
use sol_atlas_core::industry_assurance_projection::{
    ProjectionContext, project_built_environment_case,
};
use std::collections::BTreeMap;

fn digest(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn main() {
    let case = BuiltEnvironmentCase {
        case_id: "synthetic-project-pavilion-001".into(),
        jurisdiction: "synthetic-jurisdiction".into(),
        evidence: vec![
            EvidenceRecord {
                evidence_id: "survey-control-record-001".into(),
                subject_id: "design-revision-2".into(),
                field_path: "survey.control_network".into(),
                source_id: "survey-registry".into(),
                source_payload_sha256: Some(digest('a')),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_000),
            },
            EvidenceRecord {
                evidence_id: "authority-record-001".into(),
                subject_id: "decision-001".into(),
                field_path: "authority.review_grant".into(),
                source_id: "authority-registry".into(),
                source_payload_sha256: Some(digest('a')),
                evidence_state: EvidenceState::Verified,
                data_rights: DataRights::PublicDomain,
                license_id: Some("CC0-1.0".into()),
                jurisdiction: "synthetic-jurisdiction".into(),
                observed_at_unix_seconds: Some(1_762_000_001),
            },
        ],
        artifacts: vec![
            ArtifactRecord {
                artifact_id: "design-model-rev-1".into(),
                design_id: "design-pavilion-001".into(),
                family_id: "primary-design-model".into(),
                revision: 1,
                kind: ArtifactKind::DesignModel,
                content_sha256: digest('b'),
                parent_artifacts: vec![],
                evidence_ids: vec!["survey-control-record-001".into()],
                jurisdiction: "synthetic-jurisdiction".into(),
                created_at_unix_seconds: 1_762_000_100,
            },
            ArtifactRecord {
                artifact_id: "design-model-rev-2".into(),
                design_id: "design-pavilion-001".into(),
                family_id: "primary-design-model".into(),
                revision: 2,
                kind: ArtifactKind::DesignModel,
                content_sha256: digest('c'),
                parent_artifacts: vec![ArtifactReference {
                    artifact_id: "design-model-rev-1".into(),
                    content_sha256: digest('b'),
                }],
                evidence_ids: vec!["survey-control-record-001".into()],
                jurisdiction: "synthetic-jurisdiction".into(),
                created_at_unix_seconds: 1_762_000_200,
            },
        ],
        engineering_receipts: vec![EngineeringReceipt {
            receipt_id: "structural-analysis-receipt-001".into(),
            design_id: "design-pavilion-001".into(),
            input_artifact_id: "design-model-rev-2".into(),
            input_artifact_sha256: digest('c'),
            discipline: EngineeringDiscipline::Structural,
            solver_id: Some("synthetic-frame-solver".into()),
            solver_version: Some("0.1.0".into()),
            run_status: SolverRunStatus::Completed,
            conclusion: EngineeringConclusion::PassWithinDeclaredScope,
            review_level: ReviewLevel::IndependentlyChecked,
            reviewer_id: Some("synthetic-reviewer".into()),
            assumption_evidence_ids: vec!["survey-control-record-001".into()],
            declared_scope: "Synthetic structural model demonstration only".into(),
        }],
        decisions: vec![DecisionRecord {
            event_id: "decision-001".into(),
            design_id: "design-pavilion-001".into(),
            target_artifact_id: "design-model-rev-2".into(),
            target_artifact_sha256: digest('c'),
            disposition: DecisionDisposition::Conditional,
            actor_id: "synthetic-actor".into(),
            authority_evidence_id: Some("authority-record-001".into()),
            engineering_receipt_ids: vec!["structural-analysis-receipt-001".into()],
            decided_at_unix_seconds: 1_762_000_300,
            conditions: vec!["Synthetic fixture; not a real approval".into()],
            supersedes_event_id: None,
        }],
    };

    let namespaces = BTreeMap::from([
        ("evidence:survey-control-record-001".into(), "urn:synthetic:survey-registry".into()),
        ("evidence:authority-record-001".into(), "urn:synthetic:authority-registry".into()),
        ("artifact:design-model-rev-1".into(), "urn:synthetic:design-repository".into()),
        ("artifact:design-model-rev-2".into(), "urn:synthetic:design-repository".into()),
        ("engineering_receipt:structural-analysis-receipt-001".into(), "urn:synthetic:solver-registry".into()),
        ("decision:decision-001".into(), "urn:synthetic:authority-registry".into()),
    ]);
    let context = ProjectionContext {
        captured_at_utc: "2026-11-03T12:00:00Z".into(),
        projector_build_sha256: Some(digest('d')),
        mapping_profile_sha256: digest('e'),
        native_identity_namespaces: namespaces,
    };

    let output = project_built_environment_case(&case, &context);
    println!(
        "{}",
        json!({
            "fixture_status": "synthetic_only",
            "disposition": output.disposition.as_str(),
            "graph": output.graph,
            "projection_receipt": output.projection_receipt,
            "claim_limit": "projection and source metadata loss accounting only; no authentication, engineering verification, approval, or action authority"
        })
    );
}
