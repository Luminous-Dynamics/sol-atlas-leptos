// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Composed metamorphic laws for append-only evidence frontiers.
//!
//! These tests deliberately combine individually valid transformations so that
//! a later mutation cannot hide behind an otherwise-valid normalization step.

use sol_atlas_core::civilizational::{
    ArgumentationTemporalMetadataV1, EvidenceFrontierV1, EvidenceTemporalMetadataV1,
    ProjectionError, SourceSnapshotTemporalMetadataV1, YearInterval,
};

fn evidence(id: &str, source: &str, available_by: i32) -> EvidenceTemporalMetadataV1 {
    EvidenceTemporalMetadataV1 {
        evidence_id: id.into(),
        source_snapshot: source.into(),
        artifact_time: None,
        publication_time: Some(available_by),
        capture_time: None,
        available_by,
        validity_time: None,
    }
}

fn source(id: &str, available_by: i32) -> SourceSnapshotTemporalMetadataV1 {
    SourceSnapshotTemporalMetadataV1 {
        source_snapshot: id.into(),
        publication_time: Some(available_by),
        capture_time: None,
        available_by,
    }
}

fn argumentation(
    assessment: &str,
    interpretation: &str,
    available_by: i32,
) -> ArgumentationTemporalMetadataV1 {
    ArgumentationTemporalMetadataV1 {
        assessment: assessment.into(),
        interpretation: interpretation.into(),
        assessment_time: Some(YearInterval {
            from: Some(available_by),
            to: Some(available_by),
        }),
        interpretation_time: Some(YearInterval {
            from: Some(available_by),
            to: Some(available_by),
        }),
        available_by,
    }
}

#[test]
fn canonical_reorder_plus_append_cannot_hide_inherited_metadata_mutation() {
    let mut parent = EvidenceFrontierV1 {
        frontier_id: "frontier:1900".into(),
        known_by_year: 1900,
        parent_frontier: None,
        policy_version: "v1".into(),
        manifest_hash: String::new(),
        admitted_evidence: ["evidence:a", "evidence:b"]
            .into_iter()
            .map(Into::into)
            .collect(),
        admitted_sources: ["source:a", "source:b"]
            .into_iter()
            .map(Into::into)
            .collect(),
        evidence_metadata: vec![
            evidence("evidence:a", "source:a", 1900),
            evidence("evidence:b", "source:b", 1900),
        ],
        source_metadata: vec![source("source:a", 1900), source("source:b", 1900)],
        argumentation_metadata: vec![],
    };
    parent.recompute_manifest_hash().unwrap();

    let mut child = parent.clone();
    child.frontier_id = "frontier:1902".into();
    child.known_by_year = 1902;
    child.parent_frontier = Some(parent.frontier_id.clone());

    child.admitted_evidence.insert("evidence:c".into());
    child.admitted_sources.insert("source:c".into());
    child
        .evidence_metadata
        .push(evidence("evidence:c", "source:c", 1902));
    child.source_metadata.push(source("source:c", 1902));

    // Canonical evidence/source metadata is set-like, so reordering is valid.
    child.evidence_metadata.reverse();
    child.source_metadata.reverse();
    child.recompute_manifest_hash().unwrap();
    assert_eq!(child.validate_extension_of(&parent), Ok(()));

    // A later mutation of inherited metadata must still be rejected even though
    // the child has been normalized/reordered and legitimately extended.
    child.evidence_metadata[2].available_by = 1901;
    child.recompute_manifest_hash().unwrap();
    assert_eq!(
        child.validate_extension_of(&parent),
        Err(ProjectionError::InvalidEvidenceFrontierManifest)
    );
}

#[test]
fn argumentation_append_does_not_weaken_exact_prefix_after_reordering_attempt() {
    let a = argumentation("assessment:a", "interpretation:a", 1900);
    let b = argumentation("assessment:b", "interpretation:b", 1900);
    let c = argumentation("assessment:c", "interpretation:c", 1902);

    let mut parent = EvidenceFrontierV1 {
        frontier_id: "frontier:1900".into(),
        known_by_year: 1900,
        parent_frontier: None,
        policy_version: "v1".into(),
        manifest_hash: String::new(),
        admitted_evidence: Default::default(),
        admitted_sources: Default::default(),
        evidence_metadata: vec![],
        source_metadata: vec![],
        argumentation_metadata: vec![a.clone(), b.clone()],
    };
    parent.recompute_manifest_hash().unwrap();

    let mut child = parent.clone();
    child.frontier_id = "frontier:1902".into();
    child.known_by_year = 1902;
    child.parent_frontier = Some(parent.frontier_id.clone());
    child.argumentation_metadata.push(c.clone());
    child.recompute_manifest_hash().unwrap();
    assert_eq!(child.validate_extension_of(&parent), Ok(()));

    // Appending new argumentation is legal; reordering inherited records is not.
    child.argumentation_metadata = vec![b, a, c];
    child.recompute_manifest_hash().unwrap();
    assert_eq!(
        child.validate_extension_of(&parent),
        Err(ProjectionError::InvalidEvidenceFrontierManifest)
    );
}

#[test]
fn horizon_advance_plus_metadata_substitution_still_requires_inherited_identity() {
    let inherited = evidence("evidence:a", "source:a", 1900);
    let inherited_source = source("source:a", 1900);

    let mut parent = EvidenceFrontierV1 {
        frontier_id: "frontier:1900".into(),
        known_by_year: 1900,
        parent_frontier: None,
        policy_version: "v1".into(),
        manifest_hash: String::new(),
        admitted_evidence: ["evidence:a"].into_iter().map(Into::into).collect(),
        admitted_sources: ["source:a"].into_iter().map(Into::into).collect(),
        evidence_metadata: vec![inherited.clone()],
        source_metadata: vec![inherited_source.clone()],
        argumentation_metadata: vec![],
    };
    parent.recompute_manifest_hash().unwrap();

    let mut child = parent.clone();
    child.frontier_id = "frontier:1920".into();
    child.known_by_year = 1920;
    child.parent_frontier = Some(parent.frontier_id.clone());
    child.recompute_manifest_hash().unwrap();
    assert_eq!(child.validate_extension_of(&parent), Ok(()));

    // The substituted metadata remains temporally valid at the new horizon,
    // so only the inherited-record identity rule can catch the rewrite.
    child.evidence_metadata[0].available_by = 1910;
    child.source_metadata[0].available_by = 1910;
    child.recompute_manifest_hash().unwrap();
    assert!(child.validate_temporal_manifest().is_ok());
    assert_eq!(
        child.validate_extension_of(&parent),
        Err(ProjectionError::InvalidEvidenceFrontierManifest)
    );
}
