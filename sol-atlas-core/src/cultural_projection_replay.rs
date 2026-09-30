// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Content-addressed replay receipts for V5 projection audits.
//!
//! A replay receipt is a descriptive proof-of-replay artifact: it records the
//! exact closure identities and verified frontier lineage used by a V5 audit.
//! It carries no new qualification, confidence score, ranking, or truth claim.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{
    AssessmentId, ClaimId, EvidenceFrontierId, EvidenceId, EvidenceFrontierChainV1,
    ProjectionError, QualificationStatus, SourceSnapshotId,
};
use crate::cultural_argumentation::{CulturalArgumentationKindV1, CulturalArgumentationRefV3};
use crate::cultural_projection_audit_v5::CulturalProjectionAuditV5;
use crate::cultural_systems::{CanonicalClaimAdmissionV1, CulturalProjectionIdV1};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayArgumentationIdentityV1 {
    pub kind: CulturalArgumentationKindV1,
    pub assessment: AssessmentId,
    pub interpretation: crate::civilizational::InterpretationId,
    pub semantic_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayOntologyResolutionIdentityV1 {
    pub mapping_id: String,
    pub resolution_hash: String,
}

/// Descriptive, content-addressed evidence of a successful V5 replay.
///
/// This receipt is deliberately not an authority layer. Its qualification is
/// copied from the already-validated audit and cannot upgrade or downgrade it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V5ReplayReceiptV1 {
    pub audit_semantic_hash: String,
    pub projection_id: CulturalProjectionIdV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub argumentation: Vec<ReplayArgumentationIdentityV1>,
    pub ontology_resolutions: Vec<ReplayOntologyResolutionIdentityV1>,
    pub frontier_lineage: Vec<(EvidenceFrontierId, String)>,
    pub leaf_frontier: EvidenceFrontierId,
    pub qualification: QualificationStatus,
    pub receipt_hash: String,
}

impl V5ReplayReceiptV1 {
    pub fn from_audit_and_chain(
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<Self, ProjectionError> {
        audit.validate_against_frontier_chain(chain, claim)?;

        let frontier_lineage = chain
            .frontiers
            .iter()
            .map(|frontier| (frontier.frontier_id.clone(), frontier.manifest_hash.clone()))
            .collect();

        let mut argumentation = audit
            .argumentation
            .iter()
            .map(|value| ReplayArgumentationIdentityV1 {
                kind: value.kind,
                assessment: value.assessment.clone(),
                interpretation: value.interpretation.clone(),
                semantic_hash: value.semantic_hash.clone(),
            })
            .collect::<Vec<_>>();

        let mut ontology_resolutions = audit
            .base
            .resolutions
            .iter()
            .map(|resolution| ReplayOntologyResolutionIdentityV1 {
                mapping_id: resolution.mapping.mapping_id.clone(),
                resolution_hash: resolution.resolution_hash.clone(),
            })
            .collect::<Vec<_>>();

        argumentation.sort_by(|a, b| {
            (a.kind, a.assessment.clone(), a.interpretation.clone(), a.semantic_hash.clone())
                .cmp(&(b.kind, b.assessment.clone(), b.interpretation.clone(), b.semantic_hash.clone()))
        });
        ontology_resolutions.sort_by(|a, b| {
            (a.mapping_id.clone(), a.resolution_hash.clone())
                .cmp(&(b.mapping_id.clone(), b.resolution_hash.clone()))
        });

        let mut receipt = Self {
            audit_semantic_hash: audit.semantic_hash.clone(),
            projection_id: audit.projection_id().clone(),
            claim_ref: audit.claim_ref().clone(),
            evidence_refs: audit.base.base.evidence_refs.clone(),
            source_snapshots: audit.base.base.source_snapshots.clone(),
            argumentation,
            ontology_resolutions,
            frontier_lineage,
            leaf_frontier: chain.current()
                .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?
                .frontier_id
                .clone(),
            qualification: audit.qualification(),
            receipt_hash: String::new(),
        };
        receipt.evidence_refs.sort();
        receipt.evidence_refs.dedup();
        receipt.source_snapshots.sort();
        receipt.source_snapshots.dedup();
        receipt.recompute_hash()?;
        receipt.validate_against_audit_and_chain(audit, chain, claim)?;
        Ok(receipt)
    }

    pub fn validate_against_audit_and_chain(
        &self,
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<(), ProjectionError> {
        audit.validate_against_frontier_chain(chain, claim)?;
        let expected = Self::from_validated_inputs(audit, chain)?;

        if self != &expected {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        if self.receipt_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    fn from_validated_inputs(
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
    ) -> Result<Self, ProjectionError> {
        let leaf = chain
            .current()
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;

        let mut argumentation = audit
            .argumentation
            .iter()
            .map(|value| ReplayArgumentationIdentityV1 {
                kind: value.kind,
                assessment: value.assessment.clone(),
                interpretation: value.interpretation.clone(),
                semantic_hash: value.semantic_hash.clone(),
            })
            .collect::<Vec<_>>();
        argumentation.sort_by(|a, b| {
            (a.kind, a.assessment.clone(), a.interpretation.clone(), a.semantic_hash.clone())
                .cmp(&(b.kind, b.assessment.clone(), b.interpretation.clone(), b.semantic_hash.clone()))
        });

        let mut ontology_resolutions = audit
            .base
            .resolutions
            .iter()
            .map(|resolution| ReplayOntologyResolutionIdentityV1 {
                mapping_id: resolution.mapping.mapping_id.clone(),
                resolution_hash: resolution.resolution_hash.clone(),
            })
            .collect::<Vec<_>>();
        ontology_resolutions.sort_by(|a, b| {
            (a.mapping_id.clone(), a.resolution_hash.clone())
                .cmp(&(b.mapping_id.clone(), b.resolution_hash.clone()))
        });

        let mut evidence_refs = audit.base.base.evidence_refs.clone();
        evidence_refs.sort();
        evidence_refs.dedup();
        let mut source_snapshots = audit.base.base.source_snapshots.clone();
        source_snapshots.sort();
        source_snapshots.dedup();

        let mut receipt = Self {
            audit_semantic_hash: audit.semantic_hash.clone(),
            projection_id: audit.projection_id().clone(),
            claim_ref: audit.claim_ref().clone(),
            evidence_refs,
            source_snapshots,
            argumentation,
            ontology_resolutions,
            frontier_lineage: chain.frontiers
                .iter()
                .map(|frontier| (frontier.frontier_id.clone(), frontier.manifest_hash.clone()))
                .collect(),
            leaf_frontier: leaf.frontier_id.clone(),
            qualification: audit.qualification(),
            receipt_hash: String::new(),
        };
        receipt.recompute_hash()?;
        Ok(receipt)
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let payload = (
            &self.audit_semantic_hash,
            &self.projection_id,
            &self.claim_ref,
            &self.evidence_refs,
            &self.source_snapshots,
            &self.argumentation,
            &self.ontology_resolutions,
            &self.frontier_lineage,
            &self.leaf_frontier,
            &self.qualification,
        );
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.receipt_hash = self.computed_hash()?;
        Ok(())
    }

    pub fn canonicalize(&mut self) {
        self.evidence_refs.sort();
        self.evidence_refs.dedup();
        self.source_snapshots.sort();
        self.source_snapshots.dedup();
        self.argumentation.sort_by(|a, b| {
            (a.kind, a.assessment.clone(), a.interpretation.clone(), a.semantic_hash.clone())
                .cmp(&(b.kind, b.assessment.clone(), b.interpretation.clone(), b.semantic_hash.clone()))
        });
        self.ontology_resolutions.sort_by(|a, b| {
            (a.mapping_id.clone(), a.resolution_hash.clone())
                .cmp(&(b.mapping_id.clone(), b.resolution_hash.clone()))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceFrontierV1, EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1,
        YearInterval,
    };
    use crate::cultural_argumentation::CulturalArgumentationKindV1;
    use crate::cultural_projection_audit_v5::CulturalProjectionAuditV5;
    use crate::cultural_systems::{
        CanonicalClaimAdmissionV1, CulturalArgumentationEvidenceClosureV1,
    };
    use crate::ontology_context::OntologyMappingRelationV1;
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn fixture() -> (
        CulturalProjectionAuditV5,
        CanonicalClaimAdmissionV1,
        EvidenceFrontierChainV1,
    ) {
        let mut root = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:1", "e:2"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:1".into(), source_snapshot: "source:1".into(),
                    artifact_time: Some(1940), publication_time: Some(1941),
                    capture_time: None, available_by: 1942,
                    validity_time: Some(YearInterval { from: Some(1940), to: Some(1950) }),
                },
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:2".into(), source_snapshot: "source:1".into(),
                    artifact_time: Some(1945), publication_time: Some(1946),
                    capture_time: None, available_by: 1947,
                    validity_time: Some(YearInterval { from: Some(1940), to: Some(1950) }),
                },
            ],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(), publication_time: Some(1941),
                capture_time: None, available_by: 1942,
            }],
            argumentation_metadata: vec![crate::civilizational::ArgumentationTemporalMetadataV1 {
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
        child.recompute_manifest_hash().expect("child hash");

        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into(), "e:2".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: child.frontier_id.clone(),
        };

        let base = crate::cultural_systems::CulturalProjectionAuditV2 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            community_recognition_evidence: vec![],
            assessment: Some("assessment:1".into()),
            argumentation: None,
            event_time: YearInterval { from: Some(1900), to: Some(1950) },
            qualification: claim.qualification,
            access_policy: crate::cultural_systems::AccessPolicyV1::Public,
            evidence_frontier: child.frontier_id.clone(),
        };

        let mapping = OntologyMappingV2::from_claim(
            "mapping:1", OntologyMappingStandardV1::CidocCrm, "7.4",
            OntologyReleaseStatusV1::Draft, "E7_Activity",
            OntologyMappingKindV1::Class, &claim,
        );
        let resolution = crate::ontology_resolution::OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &child,
        ).expect("resolution");
        let v4 = crate::cultural_projection_audit_v4::CulturalProjectionAuditV4::from_v2(
            base, vec![resolution],
        ).expect("v4 audit");

        let argumentation = CulturalArgumentationRefV3::from_v2(
            CulturalArgumentationKindV1::InferenceMaking,
            CulturalArgumentationRefV2 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                claim_ref: claim.claim_ref.clone(),
                closure: CulturalArgumentationEvidenceClosureV1 {
                    claim_ref: claim.claim_ref.clone(),
                    evidence_refs: vec!["e:1".into()],
                    source_snapshots: vec!["source:1".into()],
                    evidence_frontier: child.frontier_id.clone(),
                },
                assessment_time: Some(YearInterval { from: Some(1948), to: Some(1948) }),
                interpretation_time: Some(YearInterval { from: Some(1949), to: Some(1949) }),
                available_by: 1950,
            },
            &child,
        ).expect("typed argumentation");

        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation])
            .expect("v5 audit");

        (audit, claim, EvidenceFrontierChainV1 {
            frontiers: vec![root, child],
        })
    }

    #[test]
    fn receipt_round_trip_validates() {
        let (audit, claim, chain) = fixture();
        let receipt = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("receipt");
        assert_eq!(
            receipt.validate_against_audit_and_chain(&audit, &chain, &claim),
            Ok(())
        );
    }

    #[test]
    fn receipt_hash_changes_when_closure_changes() {
        let (audit, claim, chain) = fixture();
        let mut receipt = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.evidence_refs.push("e:extra".into());
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(receipt.validate_against_audit_and_chain(&audit, &chain, &claim).is_err());
    }

    #[test]
    fn receipt_hash_changes_when_lineage_changes() {
        let (audit, claim, chain) = fixture();
        let mut receipt = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.frontier_lineage[0].1 = "tampered-manifest".into();
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(receipt.validate_against_audit_and_chain(&audit, &chain, &claim).is_err());
    }

    #[test]
    fn receipt_is_order_independent_for_set_like_closure() {
        let (audit, claim, chain) = fixture();
        let mut receipt = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.evidence_refs.reverse();
        receipt.source_snapshots.reverse();
        receipt.canonicalize();
        receipt.recompute_hash().expect("rehash");
        assert_eq!(original, receipt.receipt_hash);
    }

    #[test]
    fn receipt_cannot_change_qualification() {
        let (audit, claim, chain) = fixture();
        let mut receipt = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("receipt");
        receipt.qualification = QualificationStatus::Established;
        receipt.recompute_hash().expect("rehash");
        assert!(receipt.validate_against_audit_and_chain(&audit, &chain, &claim).is_err());
        assert_eq!(audit.qualification(), QualificationStatus::Supported);
    }

    #[test]
    fn receipt_rejects_unrelated_frontier_lineage() {
        let (audit, claim, chain) = fixture();
        let receipt = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("receipt");
        let mut unrelated = chain.clone();
        unrelated.frontiers[0].frontier_id = "frontier:unrelated".into();
        unrelated.frontiers[0].recompute_manifest_hash().expect("hash");
        assert!(receipt.validate_against_audit_and_chain(&audit, &chain, &claim).is_ok());
        assert!(receipt.validate_against_audit_and_chain(&audit, &unrelated, &claim).is_err());
    }
}
