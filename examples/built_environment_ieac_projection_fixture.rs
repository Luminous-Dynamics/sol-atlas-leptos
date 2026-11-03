use sol_atlas_core::built_environment_case::{
    ArtifactKind, BuiltEnvironmentCase, DataRights, DecisionDisposition, DecisionRecord,
    EngineeringConclusion, EngineeringDiscipline, EngineeringReceipt, EvidenceRecord,
    EvidenceState, ReviewLevel, SolverRunStatus,
};
use sol_atlas_core::industry_assurance_projection::{
    ProjectionContext, ProjectionDisposition, project_built_environment_case,
};
use std::collections::BTreeMap;
use std::error::Error;

fn digest(byte: u8) -> String {
    std::iter::repeat_n(byte, 64).map(char::from).collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let evidence = EvidenceRecord {
        evidence_id: "evidence-fixture-001".into(),
        subject_id: "design-fixture-001".into(),
        field_path: "geometry.revision".into(),
        source_id: "urn:fixture:evidence-registry".into(),
        source_payload_sha256: Some(digest(b'a')),
        evidence_state: EvidenceState::Verified,
        data_rights: DataRights::PublicDomain,
        license_id: Some("CC0-1.0".into()),
        jurisdiction: "synthetic-jurisdiction".into(),
        observed_at_unix_seconds: Some(1_762_000_000),
    };
    let artifact = sol_atlas_core::built_environment_case::ArtifactRecord {
        artifact_id: "artifact-fixture-001".into(),
        design_id: "design-fixture-001".into(),
        family_id: "design-model".into(),
        revision: 1,
        kind: ArtifactKind::DesignModel,
        content_sha256: digest(b'b'),
        parent_artifacts: vec![],
        evidence_ids: vec!["evidence-fixture-001".into()],
        jurisdiction: "synthetic-jurisdiction".into(),
        created_at_unix_seconds: 1_762_000_100,
    };
    let receipt = EngineeringReceipt {
        receipt_id: "receipt-fixture-001".into(),
        design_id: "design-fixture-001".into(),
        input_artifact_id: "artifact-fixture-001".into(),
        input_artifact_sha256: digest(b'b'),
        discipline: EngineeringDiscipline::Structural,
        solver_id: Some("fixture-solver".into()),
        solver_version: Some("1.0.0".into()),
        run_status: SolverRunStatus::Completed,
        conclusion: EngineeringConclusion::PassWithinDeclaredScope,
        review_level: ReviewLevel::IndependentlyChecked,
        reviewer_id: Some("fixture-reviewer".into()),
        assumption_evidence_ids: vec!["evidence-fixture-001".into()],
        declared_scope: "synthetic structural calculation".into(),
    };
    let decision = DecisionRecord {
        event_id: "decision-fixture-001".into(),
        design_id: "design-fixture-001".into(),
        target_artifact_id: "artifact-fixture-001".into(),
        target_artifact_sha256: digest(b'b'),
        disposition: DecisionDisposition::Approve,
        actor_id: "fixture-actor".into(),
        authority_evidence_id: Some("evidence-fixture-001".into()),
        engineering_receipt_ids: vec!["receipt-fixture-001".into()],
        decided_at_unix_seconds: 1_762_000_300,
        conditions: vec!["synthetic-only".into()],
        supersedes_event_id: None,
    };
    let case = BuiltEnvironmentCase {
        case_id: "case-projection-fixture-001".into(),
        jurisdiction: "synthetic-jurisdiction".into(),
        evidence: vec![evidence],
        artifacts: vec![artifact],
        engineering_receipts: vec![receipt],
        decisions: vec![decision],
    };
    let context = ProjectionContext {
        captured_at_utc: "2026-11-03T12:00:00Z".into(),
        projector_build_sha256: Some(digest(b'd')),
        mapping_profile_sha256: digest(b'e'),
        native_identity_namespaces: BTreeMap::from([
            ("evidence:evidence-fixture-001".into(), "urn:fixture:evidence-registry".into()),
            ("artifact:artifact-fixture-001".into(), "urn:fixture:design-repository".into()),
            ("engineering_receipt:receipt-fixture-001".into(), "urn:fixture:solver-registry".into()),
            ("decision:decision-fixture-001".into(), "urn:fixture:authority-registry".into()),
        ]),
    };

    let output = project_built_environment_case(&case, &context);
    if output.disposition != ProjectionDisposition::ProjectedWithKnownLoss {
        return Err(format!(
            "synthetic projector fixture did not reconcile: {}",
            serde_json::to_string(&output.projection_receipt)?
        )
        .into());
    }
    if output.graph["action_authority"] != "none"
        || output.graph["claim_ceiling"] != "incomplete_or_indeterminate"
    {
        return Err("projector fixture exceeded the conservative incomplete claim ceiling".into());
    }
    println!("{}", serde_json::to_string_pretty(&output.graph)?);
    Ok(())
}
