// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Decentralized knowledge-graph integration contracts.
//!
//! Sol Atlas may consume a DKG as a federated discovery and provenance layer,
//! but a graph record is never treated as canonical historical truth merely
//! because it is replicated, signed, popular, or highly connected.
//!
//! The semantic authority remains the canonical claim/evidence/assessment
//! layer. DKG records carry references back to those objects so every rendered
//! relationship remains reversible.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

fn has_duplicate_ids<T: Ord>(ids: &[T]) -> bool {
    let mut seen = BTreeSet::new();
    ids.iter().any(|id| !seen.insert(id))
}

use crate::civilizational::{
    ClaimId, EntityId, EvidenceFrontierId, EvidenceFrontierV1, EvidenceId, QualificationStatus,
    SourceSnapshotId, YearInterval,
};

/// Stable reference to a record in an external/federated decentralized graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DkgRecordRefV1 {
    pub graph_id: String,
    pub record_id: String,
    /// Content identity supplied by the graph adapter, not an epistemic score.
    pub content_hash: String,
}

impl DkgRecordRefV1 {
    pub fn is_valid(&self) -> bool {
        !self.graph_id.trim().is_empty()
            && !self.record_id.trim().is_empty()
            && !self.content_hash.trim().is_empty()
    }
}

/// A graph edge that can be projected into Sol Atlas only through canonical
/// claim/evidence references. The DKG predicate is descriptive metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgStatementV1 {
    pub record: DkgRecordRefV1,
    pub subject: EntityId,
    pub predicate: String,
    pub object: EntityId,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub temporal_scope: YearInterval,
    pub qualification: QualificationStatus,
}

impl DkgStatementV1 {
    pub fn validate(&self) -> bool {
        self.record.is_valid()
            && self.subject.is_valid()
            && self.object.is_valid()
            && !self.predicate.trim().is_empty()
            && self.claim_ref.is_valid()
            && !self.evidence_refs.is_empty()
            && self.evidence_refs.iter().all(|id| id.is_valid())
            && !self.source_snapshots.is_empty()
            && self.source_snapshots.iter().all(|id| id.is_valid())
            && !has_duplicate_ids(&self.evidence_refs)
            && !has_duplicate_ids(&self.source_snapshots)
            && self.temporal_scope.is_valid()
    }

    /// A DKG statement crosses into the temporal projection layer only when
    /// its evidence path is admitted as a closed evidence -> source path.
    pub fn is_frontier_safe(&self, frontier: &EvidenceFrontierV1) -> bool {
        self.validate()
            && frontier.admits_evidence_path(&self.evidence_refs, &self.source_snapshots)
    }
}

/// Explicit admission result. Keeping this separate from the statement makes
/// it impossible for graph topology alone to become an epistemic disposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgProjectionAdmissionV1 {
    pub record: DkgRecordRefV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub evidence_frontier: EvidenceFrontierId,
    /// Content-addressed identity of the selected frontier when available.
    /// Empty is retained only for serialized legacy admissions.
    #[serde(default)]
    pub frontier_manifest_hash: String,
    /// Preserve the source statement's qualification; admission never upgrades it.
    pub qualification: QualificationStatus,
}

impl DkgProjectionAdmissionV1 {
    pub fn from_statement(
        statement: &DkgStatementV1,
        frontier: &EvidenceFrontierV1,
    ) -> Option<Self> {
        if !statement.is_frontier_safe(frontier) {
            return None;
        }

        Some(Self {
            record: statement.record.clone(),
            claim_ref: statement.claim_ref.clone(),
            evidence_refs: statement.evidence_refs.clone(),
            source_snapshots: statement.source_snapshots.clone(),
            evidence_frontier: frontier.frontier_id.clone(),
            frontier_manifest_hash: frontier.manifest_hash.clone(),
            qualification: statement.qualification,
        })
    }

    pub fn validate(&self) -> bool {
        self.record.is_valid()
            && self.claim_ref.is_valid()
            && !self.evidence_refs.is_empty()
            && self.evidence_refs.iter().all(|id| id.is_valid())
            && !self.source_snapshots.is_empty()
            && self.source_snapshots.iter().all(|id| id.is_valid())
            && !has_duplicate_ids(&self.evidence_refs)
            && !has_duplicate_ids(&self.source_snapshots)
            && self.evidence_frontier.is_valid()
            && (self.frontier_manifest_hash.is_empty()
                || (self.frontier_manifest_hash.len() == 64
                    && self
                        .frontier_manifest_hash
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit())))
    }

    /// Reciprocal validation against the exact canonical statement and frontier
    /// that produced this admission. The receipt is descriptive provenance, not
    /// an independent authority, so every receipt field must reconstruct exactly
    /// from the admitted statement and selected frontier.
    pub fn validate_against_statement(
        &self,
        statement: &DkgStatementV1,
        frontier: &EvidenceFrontierV1,
    ) -> bool {
        self.validate()
            && statement.is_frontier_safe(frontier)
            && self.evidence_frontier == frontier.frontier_id
            && self.frontier_manifest_hash == frontier.manifest_hash
            && self.record == statement.record
            && self.claim_ref == statement.claim_ref
            && self.evidence_refs == statement.evidence_refs
            && self.source_snapshots == statement.source_snapshots
            && self.qualification == statement.qualification
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1};

    fn frontier() -> EvidenceFrontierV1 {
        EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:1", "e:2"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        }
    }

    fn statement() -> DkgStatementV1 {
        DkgStatementV1 {
            record: DkgRecordRefV1 {
                graph_id: "dkg:test".into(),
                record_id: "record:1".into(),
                content_hash: "sha256:abc".into(),
            },
            subject: "polity:a".into(),
            predicate: "institutional_continuity".into(),
            object: "state:b".into(),
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into(), "e:2".into()],
            source_snapshots: vec!["source:1".into()],
            temporal_scope: YearInterval {
                from: Some(1900),
                to: Some(1950),
            },
            qualification: QualificationStatus::Supported,
        }
    }

    #[test]
    fn dkg_record_is_not_valid_without_content_identity() {
        let mut record = statement().record;
        record.content_hash.clear();
        assert!(!record.is_valid());
    }

    #[test]
    fn dkg_admission_binds_same_id_frontier_manifest() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:2".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1941,
                validity_time: None,
            },
        ];
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:1".into(),
            publication_time: Some(1940),
            capture_time: None,
            available_by: 1940,
        }];
        frontier.recompute_manifest_hash().unwrap();
        let statement = statement();
        let admission = DkgProjectionAdmissionV1::from_statement(&statement, &frontier)
            .expect("frontier-safe statement");
        assert!(admission.validate_against_statement(&statement, &frontier));

        let mut shadow = frontier.clone();
        shadow.policy_version = "v2".into();
        shadow.recompute_manifest_hash().unwrap();

        assert!(!admission.validate_against_statement(&statement, &shadow));
    }

    #[test]
    fn dkg_admission_rejects_malformed_manifest_and_duplicate_provenance() {
        let frontier = frontier();
        let statement = statement();
        let mut admission = DkgProjectionAdmissionV1::from_statement(&statement, &frontier)
            .expect("frontier-safe statement");

        admission.frontier_manifest_hash = "not-a-sha256".into();
        assert!(!admission.validate());

        let mut admission = DkgProjectionAdmissionV1::from_statement(&statement, &frontier)
            .expect("frontier-safe statement");
        admission.evidence_refs.push("e:1".into());
        assert!(!admission.validate());
    }

    #[test]
    fn graph_statement_requires_canonical_evidence_path() {
        let mut value = statement();
        value.evidence_refs.clear();
        assert!(!value.validate());
    }

    #[test]
    fn frontier_blocks_dkg_evidence_declared_against_another_admitted_source() {
        let mut frontier = frontier();
        frontier.admitted_sources.insert("source:2".into());
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:2".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];

        let admitted = statement();
        assert!(admitted.is_frontier_safe(&frontier));

        let mut mismatched = admitted;
        mismatched.source_snapshots = vec!["source:2".into()];
        assert!(!mismatched.is_frontier_safe(&frontier));
    }

    #[test]
    fn frontier_blocks_dkg_admission_through_a_corrupt_present_manifest() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:2".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:1".into(),
            publication_time: Some(1940),
            capture_time: None,
            available_by: 1940,
        }];
        frontier.recompute_manifest_hash().unwrap();
        frontier.manifest_hash = "corrupt-present-manifest".into();

        assert!(!statement().is_frontier_safe(&frontier));
    }

    #[test]
    fn frontier_blocks_late_or_unknown_evidence() {
        let frontier = frontier();
        let mut value = statement();
        assert!(value.is_frontier_safe(&frontier));

        value.evidence_refs.push("e:later".into());
        assert!(!value.is_frontier_safe(&frontier));
    }

    #[test]
    fn frontier_blocks_late_source_metadata_for_dkg_admission() {
        let mut frontier = frontier();
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:1".into(),
            publication_time: Some(1951),
            capture_time: None,
            available_by: 1951,
        }];
        assert!(!statement().is_frontier_safe(&frontier));
    }

    #[test]
    fn admission_is_explicit_and_reversible() {
        let frontier = frontier();
        let admission = DkgProjectionAdmissionV1::from_statement(&statement(), &frontier).unwrap();

        assert!(admission.validate());
        assert_eq!(admission.claim_ref, "claim:1".into());
        assert_eq!(admission.record.record_id, "record:1");
        assert_eq!(admission.evidence_frontier, "frontier:1950".into());
        assert_eq!(admission.qualification, QualificationStatus::Supported);
    }

    #[test]
    fn admission_rejects_rebound_evidence_even_when_ids_remain_valid() {
        let frontier = frontier();
        let statement = statement();
        let mut admission =
            DkgProjectionAdmissionV1::from_statement(&statement, &frontier).unwrap();
        admission.evidence_refs.reverse();

        assert!(!admission.validate_against_statement(&statement, &frontier));
    }

    #[test]
    fn admission_rejects_rebound_source_even_when_source_is_frontier_admitted() {
        let mut frontier = frontier();
        frontier.admitted_sources.insert("source:2".into());
        let statement = statement();
        let mut admission =
            DkgProjectionAdmissionV1::from_statement(&statement, &frontier).unwrap();
        admission.source_snapshots = vec!["source:2".into()];

        assert!(!admission.validate_against_statement(&statement, &frontier));
    }

    #[test]
    fn admission_rejects_qualification_drift() {
        let frontier = frontier();
        let statement = statement();
        let mut admission =
            DkgProjectionAdmissionV1::from_statement(&statement, &frontier).unwrap();
        admission.qualification = QualificationStatus::Speculative;

        assert!(!admission.validate_against_statement(&statement, &frontier));
    }

    #[test]
    fn graph_topology_cannot_upgrade_qualification() {
        let frontier = frontier();
        let mut value = statement();
        value.qualification = QualificationStatus::Speculative;

        let admission = DkgProjectionAdmissionV1::from_statement(&value, &frontier).unwrap();

        assert_eq!(value.qualification, QualificationStatus::Speculative);
        assert!(admission.validate());
    }
}
