// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! End-to-end synthetic source descriptor -> BuiltEnvironmentCase -> IEAC v0.2.
//! It is a schema/replay fixture, not evidence of engineering approval.

use sha2::{Digest, Sha256};
use sol_atlas_core::built_environment_case::{
    ArtifactKind, ArtifactRecord, BuiltEnvironmentCase, DataRights, DecisionDisposition,
    DecisionRecord, EngineeringConclusion, EngineeringDiscipline, EngineeringReceipt,
    EvidenceRecord, EvidenceState, ReviewLevel, SolverRunStatus,
};
use sol_atlas_core::built_environment_source_descriptor::{
    NativeIdentityComponent, PayloadDigestBasis, SourceAttestationDescriptor,
    SourceDescriptorCatalog, SourceObservationTime, SourcePayloadDescriptor,
    SourceRecordDescriptor, SourceRecordKind, SourceRightsDescriptor, SourceRightsState,
    SourceSnapshotDescriptor, validate_source_descriptor_catalog,
};
use sol_atlas_core::industry_assurance_projection::{
    ProjectionContext, project_built_environment_case_with_descriptors,
};
use std::collections::BTreeMap;
use std::error::Error;

fn digest(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn descriptor(
    kind: SourceRecordKind,
    case_record_id: &str,
    owner_namespace: &str,
    components: Vec<(&str, &str)>,
    payload_sha: char,
    media_type: &str,
    schema_id: &str,
) -> SourceRecordDescriptor {
    SourceRecordDescriptor {
        record_kind: kind,
        case_record_id: case_record_id.into(),
        owner_namespace: owner_namespace.into(),
        native_identity_components: components
            .into_iter()
            .map(|(name, value)| NativeIdentityComponent {
                name: name.into(),
                value: value.into(),
            })
            .collect(),
        source_payload: SourcePayloadDescriptor {
            sha256: Some(digest(payload_sha)),
            digest_basis: PayloadDigestBasis::SourceOwnerAttested,
            bytes_retrieved: false,
            byte_length: None,
            payload_object_ref: None,
            media_type: Some(media_type.into()),
            schema_id: Some(schema_id.into()),
            schema_version: Some("1.0".into()),
            schema_sha256: Some(digest('e')),
        },
        source_snapshot: SourceSnapshotDescriptor {
            source_id: owner_namespace.into(),
            snapshot_id: format!("snapshot-{case_record_id}"),
            snapshot_sha256: digest('f'),
            captured_at: "2026-11-03T12:00:00Z".into(),
            retrieved_at: "2026-11-03T12:01:00Z".into(),
        },
        observation_time: SourceObservationTime {
            observed_at: Some("2026-11-03T11:59:00Z".into()),
            recorded_at: "2026-11-03T12:00:00Z".into(),
        },
        rights: SourceRightsDescriptor {
            state: SourceRightsState::Verified,
            permitted_uses: vec!["impact_analysis".into(), "engineering_review".into()],
            rights_evidence_ref: Some("synthetic:rights-evidence:001".into()),
        },
        attestation: SourceAttestationDescriptor {
            statement_ref: Some("synthetic:owner-attestation:001".into()),
            key_id: Some("urn:synthetic:key:001".into()),
            trust_evaluation: "not_evaluated_by_descriptor_catalog".into(),
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let case = BuiltEnvironmentCase {
        case_id: "case-descriptor-projection-fixture-001".into(),
        jurisdiction: "synthetic-jurisdiction".into(),
        evidence: vec![EvidenceRecord {
            evidence_id: "evidence-fixture-001".into(),
            subject_id: "design-fixture-001".into(),
            field_path: "geometry.revision".into(),
            source_id: "survey-registry".into(),
            source_payload_sha256: Some(digest('a')),
            evidence_state: EvidenceState::Verified,
            data_rights: DataRights::PublicDomain,
            license_id: Some("CC0-1.0".into()),
            jurisdiction: "synthetic-jurisdiction".into(),
            observed_at_unix_seconds: Some(1_762_000_000),
        }],
        artifacts: vec![ArtifactRecord {
            artifact_id: "artifact-fixture-001".into(),
            design_id: "design-fixture-001".into(),
            family_id: "design-model".into(),
            revision: 1,
            kind: ArtifactKind::DesignModel,
            content_sha256: digest('b'),
            parent_artifacts: vec![],
            evidence_ids: vec!["evidence-fixture-001".into()],
            jurisdiction: "synthetic-jurisdiction".into(),
            created_at_unix_seconds: 1_762_000_100,
        }],
        engineering_receipts: vec![EngineeringReceipt {
            receipt_id: "receipt-fixture-001".into(),
            design_id: "design-fixture-001".into(),
            input_artifact_id: "artifact-fixture-001".into(),
            input_artifact_sha256: digest('b'),
            discipline: EngineeringDiscipline::Structural,
            solver_id: Some("synthetic-solver".into()),
            solver_version: Some("1.0.0".into()),
            run_status: SolverRunStatus::Completed,
            conclusion: EngineeringConclusion::PassWithinDeclaredScope,
            review_level: ReviewLevel::IndependentlyChecked,
            reviewer_id: Some("synthetic-reviewer".into()),
            assumption_evidence_ids: vec!["evidence-fixture-001".into()],
            declared_scope: "synthetic structural calculation only".into(),
        }],
        decisions: vec![DecisionRecord {
            event_id: "decision-fixture-001".into(),
            design_id: "design-fixture-001".into(),
            target_artifact_id: "artifact-fixture-001".into(),
            target_artifact_sha256: digest('b'),
            disposition: DecisionDisposition::Conditional,
            actor_id: "synthetic-actor".into(),
            authority_evidence_id: Some("evidence-fixture-001".into()),
            engineering_receipt_ids: vec!["receipt-fixture-001".into()],
            decided_at_unix_seconds: 1_762_000_300,
            conditions: vec!["synthetic-only".into()],
            supersedes_event_id: None,
        }],
    };

    let descriptors = vec![
        descriptor(
            SourceRecordKind::Evidence,
            "evidence-fixture-001",
            "urn:synthetic:survey-owner",
            vec![
                ("native_id", "survey-registry"),
                ("native_record_id", "evidence-fixture-001"),
                ("native_revision", "1"),
                ("field_path", "geometry.revision"),
                ("subject_id", "design-fixture-001"),
                ("jurisdiction", "synthetic-jurisdiction"),
            ],
            'a',
            "application/json",
            "urn:synthetic:survey-evidence",
        ),
        descriptor(
            SourceRecordKind::Artifact,
            "artifact-fixture-001",
            "urn:synthetic:design-owner",
            vec![
                ("native_id", "design-fixture-001"),
                ("native_record_id", "artifact-fixture-001"),
                ("native_revision", "1"),
                ("artifact_family", "design-model"),
                ("jurisdiction", "synthetic-jurisdiction"),
            ],
            'b',
            "application/ifc",
            "urn:buildingSMART:ifc",
        ),
        descriptor(
            SourceRecordKind::EngineeringReceipt,
            "receipt-fixture-001",
            "urn:synthetic:solver-owner",
            vec![
                ("native_id", "design-fixture-001"),
                ("native_record_id", "receipt-fixture-001"),
                ("native_revision", "1"),
            ],
            'c',
            "application/json",
            "urn:synthetic:solver-receipt",
        ),
        descriptor(
            SourceRecordKind::Decision,
            "decision-fixture-001",
            "urn:synthetic:authority-owner",
            vec![
                ("native_id", "design-fixture-001"),
                ("native_record_id", "decision-fixture-001"),
                ("native_revision", "1"),
            ],
            'd',
            "application/json",
            "urn:synthetic:engineering-decision",
        ),
    ];

    let case_bytes = serde_json::to_vec(&case)?;
    let catalog = SourceDescriptorCatalog {
        schema_version: "luminous.built-environment-source-descriptor-catalog.v0.1".into(),
        catalog_id: "catalog-descriptor-projection-fixture-001".into(),
        case_id: case.case_id.clone(),
        case_representation_sha256: sha256_hex(&case_bytes),
        captured_at: "2026-11-03T12:02:00Z".into(),
        descriptors,
        claim_ceiling: "descriptor_integrity_only".into(),
        action_authority: "none".into(),
        limitations: vec![
            "Synthetic-only descriptor inputs.".into(),
            "The fixture does not retrieve or independently hash source bytes.".into(),
            "Owner namespaces, rights evidence, and attestations are not authenticated here.".into(),
            "No engineering result, approval, or actuation authority is established.".into(),
        ],
    };

    let descriptor_validation = validate_source_descriptor_catalog(&case, &catalog);
    let context = ProjectionContext {
        captured_at_utc: "2026-11-03T12:02:00Z".into(),
        projector_build_sha256: None,
        mapping_profile_sha256: digest('e'),
        native_identity_namespaces: BTreeMap::new(),
    };
    let output = project_built_environment_case_with_descriptors(&case, &context, &catalog);

    if descriptor_validation.disposition.as_str() != "incomplete_or_indeterminate"
        || output.projection.disposition.as_str() != "projected_with_known_loss"
        || output.projection.graph["claim_ceiling"] != "incomplete_or_indeterminate"
        || output.projection.graph["action_authority"] != "none"
    {
        return Err("descriptor composition did not retain its conservative claim ceiling".into());
    }

    let expected = &descriptor_validation.identity_sha256_by_case_record;
    let projected = output.projection.projection_receipt["node_mappings"]
        .as_array()
        .ok_or("projector receipt has no node_mappings array")?;
    for (record_key, identity_sha) in expected {
        let mapping = projected
            .iter()
            .find(|mapping| mapping["native_key"] == record_key)
            .ok_or_else(|| format!("missing projected mapping for {record_key}"))?;
        if mapping["identity_sha256"].as_str() != Some(identity_sha.as_str()) {
            return Err(format!("identity mismatch for {record_key}").into());
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "fixture_status": "synthetic_only",
            "descriptor_validation": descriptor_validation,
            "graph": output.projection.graph,
            "projection_receipt": output.projection.projection_receipt
        }))?
    );
    Ok(())
}
