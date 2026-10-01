// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Content-addressed receipts for explicitly selected historical V5 replay prefixes.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{EvidenceFrontierChainV1, EvidenceFrontierId, ProjectionError};
use crate::cultural_projection_audit_v5::CulturalProjectionAuditV5;
use crate::cultural_projection_historical_replay::validate_v5_at;
use crate::cultural_projection_replay::V5ReplayReceiptV1;
use crate::cultural_systems::CanonicalClaimAdmissionV1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V5HistoricalReplayReceiptV1 {
    pub selected_frontier: EvidenceFrontierId,
    pub replay: V5ReplayReceiptV1,
    pub receipt_hash: String,
}

impl V5HistoricalReplayReceiptV1 {
    pub fn from_audit_at(
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        frontier_id: &EvidenceFrontierId,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<Self, ProjectionError> {
        validate_v5_at(audit, chain, frontier_id, claim)?;
        let index = chain.frontiers.iter().position(|f| &f.frontier_id == frontier_id)
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;
        let prefix = EvidenceFrontierChainV1 { frontiers: chain.frontiers[..=index].to_vec() };
        let replay = V5ReplayReceiptV1::from_audit_and_chain(audit, &prefix, claim)?;
        let mut receipt = Self {
            selected_frontier: frontier_id.clone(),
            replay,
            receipt_hash: String::new(),
        };
        receipt.recompute_hash()?;
        receipt.validate_against_audit_at(audit, chain, claim)?;
        Ok(receipt)
    }

    pub fn validate_against_audit_at(
        &self,
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<(), ProjectionError> {
        validate_v5_at(audit, chain, &self.selected_frontier, claim)?;
        if self.replay.leaf_frontier != self.selected_frontier {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        let index = chain.frontiers.iter().position(|f| f.frontier_id == self.selected_frontier)
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;
        let prefix = EvidenceFrontierChainV1 { frontiers: chain.frontiers[..=index].to_vec() };
        self.replay.validate_against_audit_and_chain(audit, &prefix, claim)?;
        if self.receipt_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let bytes = serde_json::to_vec(&(&self.selected_frontier, &self.replay.receipt_hash))
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.receipt_hash = self.computed_hash()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        ArgumentationTemporalMetadataV1, EvidenceFrontierV1, EvidenceTemporalMetadataV1,
        QualificationStatus, SourceSnapshotTemporalMetadataV1, YearInterval,
    };
    use crate::cultural_argumentation::{CulturalArgumentationKindV1, CulturalArgumentationRefV3};
    use crate::cultural_systems::{
        AccessPolicyV1, CanonicalClaimAdmissionV1, CulturalArgumentationEvidenceClosureV1,
        CulturalProjectionAuditV2, CulturalProjectionIdV1,
    };
    use crate::ontology_context::OntologyMappingRelationV1;
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn fixture() -> (CulturalProjectionAuditV5, CanonicalClaimAdmissionV1, EvidenceFrontierChainV1) {
        let mut root = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(), known_by_year: 1950, parent_frontier: None,
            policy_version: "v1".into(), manifest_hash: String::new(),
            admitted_evidence: ["e:1"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(), source_snapshot: "source:1".into(),
                artifact_time: Some(YearInterval { from: Some(1940), to: Some(1940) }),
                publication_time: Some(1941), capture_time: None, available_by: 1942,
                validity_time: Some(YearInterval { from: Some(1940), to: Some(1950) }),
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(), publication_time: Some(1941),
                capture_time: None, available_by: 1942,
            }],
            argumentation_metadata: vec![ArgumentationTemporalMetadataV1 {
                assessment: "assessment:1".into(), interpretation: "interpretation:1".into(),
                assessment_time: Some(YearInterval { from: Some(1948), to: Some(1948) }),
                interpretation_time: Some(YearInterval { from: Some(1949), to: Some(1949) }),
                available_by: 1950,
            }],
        };
        root.recompute_manifest_hash().expect("root hash");
        let mut child = root.clone();
        child.frontier_id = "frontier:1951".into();
        child.known_by_year = 1951;
        child.parent_frontier = Some(root.frontier_id.clone());
        child.admitted_evidence.insert("e:2".into());
        child.evidence_metadata.push(EvidenceTemporalMetadataV1 {
            evidence_id: "e:2".into(), source_snapshot: "source:1".into(),
            artifact_time: Some(YearInterval { from: Some(1945), to: Some(1945) }),
            publication_time: Some(1950), capture_time: None, available_by: 1951,
            validity_time: Some(YearInterval { from: Some(1945), to: Some(1951) }),
        });
        child.recompute_manifest_hash().expect("child hash");

        let mut grandchild = child.clone();
        grandchild.frontier_id = "frontier:1952".into();
        grandchild.known_by_year = 1952;
        grandchild.parent_frontier = Some(child.frontier_id.clone());
        grandchild.admitted_evidence.insert("e:3".into());
        grandchild.admitted_sources.insert("source:3".into());
        grandchild.evidence_metadata.push(EvidenceTemporalMetadataV1 {
            evidence_id: "e:3".into(), source_snapshot: "source:3".into(),
            artifact_time: Some(YearInterval { from: Some(1948), to: Some(1948) }),
            publication_time: Some(1951), capture_time: None, available_by: 1952,
            validity_time: Some(YearInterval { from: Some(1948), to: Some(1952) }),
        });
        grandchild.source_metadata.push(SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:3".into(), publication_time: Some(1951),
            capture_time: None, available_by: 1952,
        });
        grandchild.recompute_manifest_hash().expect("grandchild hash");

        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(), evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()], qualification: QualificationStatus::Supported,
            evidence_frontier: root.frontier_id.clone(),
        };
        let base = CulturalProjectionAuditV2 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: claim.claim_ref.clone(), evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(), community_recognition_evidence: vec![],
            assessment: Some("assessment:1".into()), argumentation: None,
            event_time: YearInterval { from: Some(1900), to: Some(1950) },
            qualification: claim.qualification, access_policy: AccessPolicyV1::Public,
            evidence_frontier: root.frontier_id.clone(),
        };
        let mapping = OntologyMappingV2::from_claim(
            "mapping:1", OntologyMappingStandardV1::CidocCrm, "7.4",
            OntologyReleaseStatusV1::Draft, "E7_Activity", OntologyMappingKindV1::Class, &claim,
        );
        let resolution = crate::ontology_resolution::OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &root,
        ).expect("resolution");
        let v4 = crate::cultural_projection_audit_v4::CulturalProjectionAuditV4::from_v2(
            base, vec![resolution],
        ).expect("v4 audit");
        let argumentation = CulturalArgumentationRefV3::from_v2(
            CulturalArgumentationKindV1::InferenceMaking,
            crate::cultural_systems::CulturalArgumentationRefV2 {
                assessment: "assessment:1".into(), interpretation: "interpretation:1".into(),
                claim_ref: claim.claim_ref.clone(),
                closure: CulturalArgumentationEvidenceClosureV1 {
                    claim_ref: claim.claim_ref.clone(), evidence_refs: vec!["e:1".into()],
                    source_snapshots: vec!["source:1".into()],
                    evidence_frontier: root.frontier_id.clone(),
                },
                assessment_time: Some(YearInterval { from: Some(1948), to: Some(1948) }),
                interpretation_time: Some(YearInterval { from: Some(1949), to: Some(1949) }),
                available_by: 1950,
            }, &root,
        ).expect("argumentation");
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");
        (audit, claim, EvidenceFrontierChainV1 { frontiers: vec![root, child, grandchild] })
    }

    fn audit_bound_to_frontier(
        audit: &CulturalProjectionAuditV5,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierId,
    ) -> CulturalProjectionAuditV5 {
        let mut rebound = audit.clone();
        rebound.base.base.evidence_frontier = frontier.clone();
        for resolution in &mut rebound.base.resolutions {
            resolution.evidence_frontier = frontier.clone();
            resolution.recompute_hash().expect("resolution hash");
        }
        rebound.base.recompute_hash().expect("v4 hash");
        for argumentation in &mut rebound.argumentation {
            argumentation.closure.evidence_frontier = frontier.clone();
            argumentation.recompute_hash().expect("argumentation hash");
        }
        rebound.recompute_hash().expect("v5 hash");
        assert_eq!(rebound.claim_ref(), &claim.claim_ref);
        rebound.validate().expect("rebound audit");
        rebound
    }

    #[test]
    fn independent_epoch_receipts_bind_to_their_exact_verified_prefixes() {
        let (root_audit, root_claim, chain) = fixture();
        let mut receipts = Vec::new();

        for (index, frontier) in chain.frontiers.iter().enumerate() {
            let claim = CanonicalClaimAdmissionV1 {
                evidence_frontier: frontier.frontier_id.clone(),
                ..root_claim.clone()
            };
            let audit = audit_bound_to_frontier(&root_audit, &claim, &frontier.frontier_id);
            let receipt = V5HistoricalReplayReceiptV1::from_audit_at(
                &audit,
                &chain,
                &frontier.frontier_id,
                &claim,
            )
            .expect("epoch receipt");
            assert_eq!(receipt.replay.frontier_lineage.len(), index + 1);
            assert_eq!(
                receipt.replay.leaf_frontier,
                frontier.frontier_id
            );
            receipts.push(receipt);
        }

        assert_ne!(receipts[0].receipt_hash, receipts[1].receipt_hash);
        assert_ne!(receipts[1].receipt_hash, receipts[2].receipt_hash);
        assert_ne!(receipts[0].receipt_hash, receipts[2].receipt_hash);

        let (root_claim_for_cross_check, child_claim, grandchild_claim) = (
            root_claim.clone(),
            CanonicalClaimAdmissionV1 {
                evidence_frontier: chain.frontiers[1].frontier_id.clone(),
                ..root_claim.clone()
            },
            CanonicalClaimAdmissionV1 {
                evidence_frontier: chain.frontiers[2].frontier_id.clone(),
                ..root_claim
            },
        );
        let root_audit_for_cross_check = audit_bound_to_frontier(
            &root_audit,
            &root_claim_for_cross_check,
            &chain.frontiers[0].frontier_id,
        );
        let child_audit = audit_bound_to_frontier(
            &root_audit,
            &child_claim,
            &chain.frontiers[1].frontier_id,
        );
        let grandchild_audit = audit_bound_to_frontier(
            &root_audit,
            &grandchild_claim,
            &chain.frontiers[2].frontier_id,
        );

        assert!(receipts[0]
            .validate_against_audit_at(&child_audit, &chain, &child_claim)
            .is_err());
        assert!(receipts[1]
            .validate_against_audit_at(&grandchild_audit, &chain, &grandchild_claim)
            .is_err());
        assert!(receipts[2]
            .validate_against_audit_at(&root_audit_for_cross_check, &chain, &root_claim_for_cross_check)
            .is_err());
    }

    #[test]
    fn selected_prefix_receipt_round_trips() {
        let (audit, claim, chain) = fixture();
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(&audit, &chain, &"frontier:1950".into(), &claim).expect("receipt");
        assert_eq!(receipt.validate_against_audit_at(&audit, &chain, &claim), Ok(()));
        assert_eq!(receipt.replay.frontier_lineage.len(), 1);
        assert_eq!(receipt.replay.frontier_lineage[0].0, "frontier:1950".into());
        assert_eq!(receipt.selected_frontier, "frontier:1950".into());
    }

    #[test]
    fn later_frontier_is_not_in_receipt_lineage() {
        let (audit, claim, chain) = fixture();
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(&audit, &chain, &"frontier:1950".into(), &claim).expect("receipt");
        assert_eq!(receipt.replay.frontier_lineage.len(), 1);
        assert!(!receipt.replay.frontier_lineage.iter().any(|(id, _)| {
            id == "frontier:1951" || id == "frontier:1952"
        }));
    }

    #[test]
    fn later_corruption_does_not_invalidate_historical_receipt() {
        let (audit, claim, mut chain) = fixture();
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(&audit, &chain, &"frontier:1950".into(), &claim).expect("receipt");
        chain.frontiers[1].manifest_hash = "corrupt-middle-manifest".into();
        chain.frontiers[2].manifest_hash = "corrupt-later-manifest".into();
        assert_eq!(receipt.validate_against_audit_at(&audit, &chain, &claim), Ok(()));
        assert_eq!(
            chain.validate_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn three_frontiers_have_distinct_verified_manifest_hashes() {
        let (_, _, chain) = fixture();
        assert_eq!(chain.validate_strict(), Ok(()));
        assert_eq!(chain.frontiers.len(), 3);
        assert_ne!(chain.frontiers[0].manifest_hash, chain.frontiers[1].manifest_hash);
        assert_ne!(chain.frontiers[1].manifest_hash, chain.frontiers[2].manifest_hash);
        assert_ne!(chain.frontiers[0].manifest_hash, chain.frontiers[2].manifest_hash);
        assert_eq!(
            chain.frontiers[2].parent_frontier,
            Some("frontier:1951".into())
        );
        assert!(chain.frontiers[2].admitted_evidence.contains("e:3"));
        assert!(chain.frontiers[2].admitted_sources.contains("source:3"));
    }

    #[test]
    fn inherited_manifest_corruption_invalidates_descendants() {
        let (_, _, mut chain) = fixture();
        chain.frontiers[0].manifest_hash = "rewritten-root".into();
        assert_eq!(
            chain.validate_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn receipt_cannot_be_transplanted_to_structurally_similar_later_frontier() {
        let (audit, claim, chain) = fixture();
        let mut receipt = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &"frontier:1950".into(),
            &claim,
        )
        .expect("receipt");
        receipt.selected_frontier = "frontier:1952".into();
        assert!(receipt.validate_against_audit_at(&audit, &chain, &claim).is_err());
    }

    #[test]
    fn receipt_preserves_claim_qualification() {
        let (audit, claim, chain) = fixture();
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &"frontier:1950".into(),
            &claim,
        )
        .expect("receipt");
        assert_eq!(receipt.replay.qualification, claim.qualification);
        assert_eq!(receipt.replay.qualification, QualificationStatus::Supported);
    }

    #[test]
    fn same_id_different_manifest_cannot_substitute_frontier() {
        let (audit, claim, chain) = fixture();
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &"frontier:1950".into(),
            &claim,
        )
        .expect("receipt");

        let mut substituted_root = chain.frontiers[0].clone();
        // Preserve the identifier while changing a manifest-covered field.
        substituted_root.policy_version = "v2".into();
        substituted_root.recompute_manifest_hash().expect("substituted hash");
        assert_eq!(substituted_root.frontier_id, "frontier:1950".into());
        assert_ne!(substituted_root.manifest_hash, chain.frontiers[0].manifest_hash);

        let substituted_chain = EvidenceFrontierChainV1 {
            frontiers: vec![substituted_root],
        };
        assert!(receipt
            .validate_against_audit_at(&audit, &substituted_chain, &claim)
            .is_err());
    }

    #[test]
    fn receipt_hash_binds_selected_frontier() {
        let (audit, claim, chain) = fixture();
        let mut receipt = V5HistoricalReplayReceiptV1::from_audit_at(&audit, &chain, &"frontier:1950".into(), &claim).expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.selected_frontier = "frontier:1951".into();
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(receipt.validate_against_audit_at(&audit, &chain, &claim).is_err());
    }

    #[test]
    fn independently_reordered_equivalent_prefix_preserves_receipt_hash() {
        let (audit, claim, chain) = fixture();
        let frontier_id: EvidenceFrontierId = "frontier:1951".into();
        let child_claim = CanonicalClaimAdmissionV1 {
            evidence_frontier: frontier_id.clone(),
            ..claim.clone()
        };
        let child_audit = audit_bound_to_frontier(&audit, &child_claim, &frontier_id);
        let original = V5HistoricalReplayReceiptV1::from_audit_at(
            &child_audit,
            &chain,
            &frontier_id,
            &child_claim,
        )
        .expect("original receipt");

        // Reconstruct the same verified prefix with a different vector order.
        // Manifest identity is canonical over set-like metadata, while the
        // selected frontier and its lineage remain unchanged.
        let mut equivalent_chain = chain.clone();
        equivalent_chain.frontiers[1].evidence_metadata.reverse();
        equivalent_chain.frontiers[1].recompute_manifest_hash().expect("equivalent child hash");
        assert_eq!(
            equivalent_chain.frontiers[1].manifest_hash,
            chain.frontiers[1].manifest_hash
        );
        assert_eq!(equivalent_chain.validate_strict(), Ok(()));

        let mut equivalent_claim = child_claim.clone();
        equivalent_claim.evidence_refs.reverse();
        equivalent_claim.source_snapshots.reverse();
        let equivalent = V5HistoricalReplayReceiptV1::from_audit_at(
            &child_audit,
            &equivalent_chain,
            &frontier_id,
            &equivalent_claim,
        )
        .expect("equivalent receipt");

        assert_eq!(original.receipt_hash, equivalent.receipt_hash);
        assert_eq!(original.replay.receipt_hash, equivalent.replay.receipt_hash);
        assert_eq!(original.replay.frontier_lineage, equivalent.replay.frontier_lineage);
    }

    #[test]
    fn historical_root_receipt_is_invariant_to_equivalent_descendant_reconstruction() {
        let (audit, claim, chain) = fixture();
        let root_id: EvidenceFrontierId = "frontier:1950".into();
        let original = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &root_id,
            &claim,
        )
        .expect("original root receipt");

        // Reconstruct later frontiers independently while preserving the exact
        // root frontier and all manifest semantics. These descendant changes
        // must not become part of a receipt explicitly selected at the root.
        let mut reconstructed = chain.clone();
        reconstructed.frontiers[1].evidence_metadata.reverse();
        reconstructed.frontiers[1].source_metadata.reverse();
        reconstructed.frontiers[1].recompute_manifest_hash().expect("child hash");
        reconstructed.frontiers[2].evidence_metadata.reverse();
        reconstructed.frontiers[2].source_metadata.reverse();
        reconstructed.frontiers[2].recompute_manifest_hash().expect("grandchild hash");
        assert_eq!(reconstructed.frontiers[0], chain.frontiers[0]);
        assert_eq!(reconstructed.validate_strict(), Ok(()));

        let reconstructed_receipt = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &reconstructed,
            &root_id,
            &claim,
        )
        .expect("reconstructed root receipt");

        assert_eq!(original.receipt_hash, reconstructed_receipt.receipt_hash);
        assert_eq!(original.replay.receipt_hash, reconstructed_receipt.replay.receipt_hash);
        assert_eq!(original.replay.frontier_lineage, reconstructed_receipt.replay.frontier_lineage);
    }

    #[test]
    fn valid_later_frontier_addition_does_not_change_historical_root_receipt() {
        let (audit, claim, mut chain) = fixture();
        let root_id: EvidenceFrontierId = "frontier:1950".into();
        let original = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &root_id,
            &claim,
        )
        .expect("original root receipt");

        let parent = chain.frontiers.last().cloned().expect("grandchild");
        let mut later = parent.clone();
        later.frontier_id = "frontier:1953".into();
        later.known_by_year = 1953;
        later.parent_frontier = Some(parent.frontier_id.clone());
        later.admitted_evidence.insert("e:4".into());
        later.evidence_metadata.push(EvidenceTemporalMetadataV1 {
            evidence_id: "e:4".into(),
            source_snapshot: "source:3".into(),
            artifact_time: Some(YearInterval { from: Some(1950), to: Some(1950) }),
            publication_time: Some(1952),
            capture_time: None,
            available_by: 1953,
            validity_time: Some(YearInterval { from: Some(1950), to: Some(1953) }),
        });
        later.recompute_manifest_hash().expect("later frontier hash");
        chain.frontiers.push(later);

        assert_eq!(chain.validate_strict(), Ok(()));

        let reconstructed = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &root_id,
            &claim,
        )
        .expect("root receipt with later frontier");

        assert_eq!(original.receipt_hash, reconstructed.receipt_hash);
        assert_eq!(original.replay.receipt_hash, reconstructed.replay.receipt_hash);
        assert_eq!(original.replay.frontier_lineage, reconstructed.replay.frontier_lineage);
        assert_eq!(reconstructed.validate_against_audit_at(&audit, &chain, &claim), Ok(()));
        assert_eq!(reconstructed.replay.frontier_lineage.len(), 1);
    }


    #[test]
    fn reconstructed_root_with_changed_availability_cannot_reuse_historical_receipt() {
        let (audit, claim, chain) = fixture();
        let root_id: EvidenceFrontierId = "frontier:1950".into();
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(
            &audit,
            &chain,
            &root_id,
            &claim,
        )
        .expect("original root receipt");

        let mut reconstructed_root = chain.frontiers[0].clone();
        reconstructed_root.evidence_metadata[0].available_by = 1943;
        reconstructed_root.recompute_manifest_hash().expect("reconstructed root hash");

        let reconstructed_chain = EvidenceFrontierChainV1 {
            frontiers: vec![reconstructed_root],
        };

        assert_ne!(
            receipt.replay.frontier_lineage[0].1,
            reconstructed_chain.frontiers[0].manifest_hash
        );
        assert!(receipt
            .validate_against_audit_at(&audit, &reconstructed_chain, &claim)
            .is_err());
    }


    #[test]
    fn child_receipt_rejects_parent_lineage_substitution() {
        let (audit, claim, chain) = fixture();
        let child_id: EvidenceFrontierId = "frontier:1951".into();
        let child_claim = CanonicalClaimAdmissionV1 {
            evidence_frontier: child_id.clone(),
            ..claim
        };
        let child_audit = audit_bound_to_frontier(&audit, &child_claim, &child_id);
        let receipt = V5HistoricalReplayReceiptV1::from_audit_at(
            &child_audit,
            &chain,
            &child_id,
            &child_claim,
        )
        .expect("child receipt");

        // Keep the selected child's own manifest intact, but substitute its
        // parent identity. The receipt must remain bound to the verified
        // historical lineage, not merely to the selected frontier contents.
        let mut substituted = chain.clone();
        substituted.frontiers[1].parent_frontier = Some("frontier:synthetic-parent".into());
        assert_eq!(
            substituted.frontiers[1].manifest_hash,
            chain.frontiers[1].manifest_hash
        );

        assert!(receipt
            .validate_against_audit_at(&child_audit, &substituted, &child_claim)
            .is_err());
    }


    #[test]
    fn child_bound_audit_cannot_validate_at_parent() {
        let (audit, claim, chain) = fixture();
        let child_claim = CanonicalClaimAdmissionV1 { evidence_frontier: "frontier:1951".into(), ..claim };
        assert!(V5HistoricalReplayReceiptV1::from_audit_at(&audit, &chain, &"frontier:1951".into(), &child_claim).is_err());
    }
}
