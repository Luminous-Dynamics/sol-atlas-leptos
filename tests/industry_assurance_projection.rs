use sol_atlas_core::built_environment_case::{
    ArtifactKind, BuiltEnvironmentCase, DataRights, DecisionDisposition, DecisionRecord,
    EngineeringConclusion, EngineeringDiscipline, EngineeringReceipt, EvidenceRecord,
    EvidenceState, ReviewLevel, SolverRunStatus,
};
use sol_atlas_core::industry_assurance_projection::{
    ProjectionContext, ProjectionDisposition, project_built_environment_case,
};

fn digest(byte: u8) -> String {
    std::iter::repeat_n(byte, 64).map(char::from).collect()
}

fn fixture_case() -> BuiltEnvironmentCase {
    let evidence = EvidenceRecord {
        evidence_id: "evidence-001".into(),
        subject_id: "design-001".into(),
        field_path: "geometry.revision".into(),
        source_id: "urn:fixture:source-registry".into(),
        source_payload_sha256: Some(digest(b'a')),
        evidence_state: EvidenceState::Verified,
        data_rights: DataRights::PublicDomain,
        license_id: Some("CC0-1.0".into()),
        jurisdiction: "synthetic-jurisdiction".into(),
        observed_at_unix_seconds: Some(1_762_000_000),
    };

    let artifact = sol_atlas_core::built_environment_case::ArtifactRecord {
        artifact_id: "artifact-001".into(),
        design_id: "design-001".into(),
        family_id: "design-model".into(),
        revision: 1,
        kind: ArtifactKind::DesignModel,
        content_sha256: digest(b'b'),
        parent_artifacts: vec![],
        evidence_ids: vec!["evidence-001".into()],
        jurisdiction: "synthetic-jurisdiction".into(),
        created_at_unix_seconds: 1_762_000_100,
    };

    let receipt = EngineeringReceipt {
        receipt_id: "receipt-001".into(),
        design_id: "design-001".into(),
        input_artifact_id: "artifact-001".into(),
        input_artifact_sha256: digest(b'b'),
        discipline: EngineeringDiscipline::Structural,
        solver_id: Some("fixture-solver".into()),
        solver_version: Some("1.0.0".into()),
        run_status: SolverRunStatus::Completed,
        conclusion: EngineeringConclusion::PassWithinDeclaredScope,
        review_level: ReviewLevel::IndependentlyChecked,
        reviewer_id: Some("fixture-reviewer".into()),
        assumption_evidence_ids: vec!["evidence-001".into()],
        declared_scope: "synthetic structural calculation".into(),
    };

    let decision = DecisionRecord {
        event_id: "decision-001".into(),
        design_id: "design-001".into(),
        target_artifact_id: "artifact-001".into(),
        target_artifact_sha256: digest(b'b'),
        disposition: DecisionDisposition::Approve,
        actor_id: "fixture-actor".into(),
        authority_evidence_id: Some("evidence-001".into()),
        engineering_receipt_ids: vec!["receipt-001".into()],
        decided_at_unix_seconds: 1_762_000_300,
        conditions: vec!["synthetic-only".into()],
        supersedes_event_id: None,
    };

    BuiltEnvironmentCase {
        case_id: "case-001".into(),
        jurisdiction: "synthetic-jurisdiction".into(),
        evidence: vec![evidence],
        artifacts: vec![artifact],
        engineering_receipts: vec![receipt],
        decisions: vec![decision],
    }
}

fn context() -> ProjectionContext {
    ProjectionContext {
        captured_at_utc: "2026-11-03T12:00:00Z".into(),
        projector_build_sha256: Some(digest(b'd')),
        mapping_profile_sha256: digest(b'e'),
    }
}

#[test]
fn public_projector_emits_identity_preserving_graph_and_receipt() {
    let output = project_built_environment_case(&fixture_case(), &context());
    assert_eq!(output.disposition, ProjectionDisposition::CompleteForDeclaredScope);
    assert_eq!(
        output.graph["schema_version"],
        "luminous.ieac-assurance-dependency-graph.v0.2"
    );
    assert_eq!(output.graph["claim_ceiling"], "record_only");
    assert_eq!(output.graph["action_authority"], "none");
    assert_eq!(output.graph["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(output.graph["edges"].as_array().unwrap().len(), 6);
    assert_eq!(
        output.projection_receipt["disposition"],
        "complete_for_declared_scope"
    );
    assert_eq!(output.projection_receipt["verification_claim"], "none");
    assert_eq!(output.projection_receipt["action_authority"], "none");
    let ids = output.graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["node_id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 4);
    assert!(ids.iter().all(|id| id.contains(":identity-sha256:")));
    assert!(output.graph["graph_snapshot_sha256"]
        .as_str()
        .is_some_and(|value| value.len() == 64));
}
