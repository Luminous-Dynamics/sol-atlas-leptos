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
    assert_eq!(output.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
    assert_eq!(
        output.graph["schema_version"],
        "luminous.ieac-assurance-dependency-graph.v0.2"
    );
    assert_eq!(output.graph["claim_ceiling"], "incomplete_or_indeterminate");
    assert_eq!(output.graph["action_authority"], "none");
    assert_eq!(output.graph["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(output.graph["edges"].as_array().unwrap().len(), 6);
    let mut gold_relationships = output.graph["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| {
            let from = output.graph["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|node| node["node_id"] == edge["from_node_id"])
                .unwrap()["node_type"]
                .as_str()
                .unwrap()
                .to_owned();
            let to = output.graph["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|node| node["node_id"] == edge["to_node_id"])
                .unwrap()["node_type"]
                .as_str()
                .unwrap()
                .to_owned();
            (from, to, edge["relation"].as_str().unwrap().to_owned())
        })
        .collect::<Vec<_>>();
    gold_relationships.sort();
    let mut expected_relationships = vec![
        ("source_evidence".to_owned(), "engineering_artifact".to_owned(), "evidence_for".to_owned()),
        ("engineering_artifact".to_owned(), "engineering_receipt".to_owned(), "verified_by".to_owned()),
        ("source_evidence".to_owned(), "engineering_receipt".to_owned(), "evidence_for".to_owned()),
        ("engineering_artifact".to_owned(), "decision".to_owned(), "evidence_for".to_owned()),
        ("engineering_receipt".to_owned(), "decision".to_owned(), "evidence_for".to_owned()),
        ("source_evidence".to_owned(), "decision".to_owned(), "authorized_by".to_owned()),
    ];
    expected_relationships.sort();
    assert_eq!(gold_relationships, expected_relationships);
    assert_eq!(
        output.projection_receipt["disposition"],
        "projected_with_known_loss"
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


#[test]
fn identical_evidence_payloads_keep_distinct_source_record_identities() {
    let mut case = fixture_case();
    let mut second = case.evidence[0].clone();
    second.evidence_id = "evidence-002".into();
    second.subject_id = "design-002".into();
    case.evidence.push(second);
    let output = project_built_environment_case(&case, &context());
    let evidence_nodes = output.graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["node_type"] == "source_evidence")
        .collect::<Vec<_>>();
    assert_eq!(evidence_nodes.len(), 2);
    assert_eq!(evidence_nodes[0]["artifact_sha256"], evidence_nodes[1]["artifact_sha256"]);
    assert_ne!(evidence_nodes[0]["identity_sha256"], evidence_nodes[1]["identity_sha256"]);
}

#[test]
fn missing_source_payload_digest_is_preserved_but_not_projected_as_evidence_node() {
    let mut case = fixture_case();
    case.evidence[0].source_payload_sha256 = None;
    let output = project_built_environment_case(&case, &context());
    assert_eq!(output.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
    assert_eq!(
        output.graph["claim_ceiling"],
        "projected_with_known_loss"
    );
    assert!(output.projection_receipt["projected_payloads"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["record_key"] == "evidence:evidence-001"
            && row["node_id"].is_null()
            && row["payload_bytes_utf8"].as_str().is_some()));
}

#[test]
fn missing_decision_authority_is_not_inferred() {
    let mut case = fixture_case();
    case.decisions[0].authority_evidence_id = None;
    let output = project_built_environment_case(&case, &context());
    assert_eq!(output.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
    assert!(output.projection_receipt["unmapped_relationships"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["reason_code"] == "DECISION_AUTHORITY_EVIDENCE_NOT_DECLARED"));
}

#[test]
fn reordered_input_preserves_graph_digest_but_changes_exact_input_receipt() {
    let case = fixture_case();
    let mut reordered = case.clone();
    reordered.evidence.reverse();
    reordered.artifacts.reverse();
    reordered.engineering_receipts.reverse();
    reordered.decisions.reverse();

    let first = project_built_environment_case(&case, &context());
    let second = project_built_environment_case(&reordered, &context());
    assert_eq!(
        first.graph["graph_snapshot_sha256"],
        second.graph["graph_snapshot_sha256"]
    );
    assert_ne!(
        first.projection_receipt["source_case_representation"]["sha256"],
        second.projection_receipt["source_case_representation"]["sha256"]
    );
    assert_ne!(
        first.projection_receipt["projection_receipt_sha256"],
        second.projection_receipt["projection_receipt_sha256"]
    );
}

#[test]
fn duplicate_dependency_edges_in_source_case_are_rejected_not_deduplicated() {
    let mut case = fixture_case();
    case.artifacts[0].evidence_ids.push("evidence-001".into());
    let output = project_built_environment_case(&case, &context());
    assert_eq!(output.disposition, ProjectionDisposition::InvalidInput);
    assert!(output.projection_receipt["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["code"] == "DUPLICATE_PROJECTED_EDGE_ENDPOINT_PAIR"));
}

#[test]
fn source_artifact_parent_cycle_is_rejected() {
    let mut case = fixture_case();
    case.artifacts[0].parent_artifacts.push(
        sol_atlas_core::built_environment_case::ArtifactReference {
            artifact_id: "artifact-001".into(),
            content_sha256: digest(b'b'),
        }
    );
    let output = project_built_environment_case(&case, &context());
    assert_eq!(output.disposition, ProjectionDisposition::InvalidInput);
    assert!(output.projection_receipt["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["code"] == "PROJECTED_DEPENDENCY_CYCLE"));
}

#[test]
fn mismatched_parent_and_solver_input_digests_do_not_create_edges() {
    let mut case = fixture_case();
    let mut parent = case.artifacts[0].clone();
    parent.artifact_id = "artifact-parent".into();
    parent.content_sha256 = digest(b'f');
    case.artifacts[0].parent_artifacts.push(
        sol_atlas_core::built_environment_case::ArtifactReference {
            artifact_id: parent.artifact_id.clone(),
            content_sha256: digest(b'e'),
        }
    );
    case.artifacts.push(parent);
    case.artifacts[0].parent_artifacts[0].artifact_id = "missing-parent".into();
    case.engineering_receipts[0].input_artifact_sha256 = digest(b'f');

    let output = project_built_environment_case(&case, &context());
    assert_eq!(output.disposition, ProjectionDisposition::ProjectedWithKnownLoss);
    let unmapped = output.projection_receipt["unmapped_relationships"]
        .as_array()
        .unwrap();
    assert!(unmapped.iter().any(|row| row["reason_code"] == "PARENT_ARTIFACT_NOT_FOUND"));
    assert!(unmapped.iter().any(|row| row["reason_code"] == "RECEIPT_INPUT_DIGEST_OR_PROJECTION_MISMATCH"));
}


#[test]
fn one_thousand_digest_rebinding_mutations_never_map_a_false_parent_edge() {
    let allowed_hex = b"0123456789abcde";
    for seed in 0..1_000usize {
        let mut case = fixture_case();
        let mut parent = case.artifacts[0].clone();
        parent.artifact_id = "artifact-parent".into();
        parent.revision = 0;
        parent.content_sha256 = digest(b'f');
        parent.evidence_ids.clear();

        case.artifacts[0].parent_artifacts.push(
            sol_atlas_core::built_environment_case::ArtifactReference {
                artifact_id: "artifact-parent".into(),
                content_sha256: digest(allowed_hex[seed % allowed_hex.len()]),
            }
        );
        case.artifacts.push(parent);

        let output = project_built_environment_case(&case, &context());
        assert_eq!(
            output.disposition,
            ProjectionDisposition::ProjectedWithKnownLoss,
            "mutation seed {seed} must be loss-accounted"
        );
        assert!(!output.graph["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["relation"] == "derived_from"));
        assert!(output.projection_receipt["unmapped_relationships"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["reason_code"] == "PARENT_ARTIFACT_DIGEST_OR_PROJECTION_MISMATCH"),
            "mutation seed {seed} must have an explicit digest mismatch finding"
        );
    }
}
