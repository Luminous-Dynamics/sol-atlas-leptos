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
    AssessmentId, ClaimId, EvidenceFrontierChainV1, EvidenceFrontierId, EvidenceId,
    ProjectionError, QualificationStatus, SourceSnapshotId,
};
use crate::cultural_argumentation::CulturalArgumentationKindV1;
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
            (
                a.kind,
                a.assessment.clone(),
                a.interpretation.clone(),
                a.semantic_hash.clone(),
            )
                .cmp(&(
                    b.kind,
                    b.assessment.clone(),
                    b.interpretation.clone(),
                    b.semantic_hash.clone(),
                ))
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
            leaf_frontier: chain
                .current()
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

    /// Constructs a replay receipt through the strict provenance path.
    ///
    /// This binds the receipt to the exact originating projection as well as
    /// the verified chain leaf. The compatibility constructor remains available
    /// for legacy records.
    pub fn from_projection_and_chain(
        projection: &crate::cultural_systems::CulturalProjectionV1,
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<Self, ProjectionError> {
        let frontier = chain
            .current()
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;
        audit.validate_strong_against_projection(projection, frontier, claim)?;

        let receipt = Self::from_audit_and_chain(audit, chain, claim)?;
        receipt.validate_strong_against_projection(projection, audit, chain, claim)?;
        Ok(receipt)
    }

    /// Validates the receipt all the way back to the exact originating
    /// cultural projection, canonical claim, and verified frontier chain.
    pub fn validate_against_projection(
        &self,
        projection: &crate::cultural_systems::CulturalProjectionV1,
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<(), ProjectionError> {
        // Leaf selection is identifier-based. Require uniqueness across the
        // supplied sequence before selecting a prefix so a later duplicate
        // cannot hide behind an otherwise valid historical-looking prefix.
        chain.validate_unique_frontier_ids()?;
        let index = chain
            .frontiers
            .iter()
            .position(|frontier| frontier.frontier_id == self.leaf_frontier)
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;
        let prefix = EvidenceFrontierChainV1 {
            frontiers: chain.frontiers[..=index].to_vec(),
        };
        let frontier = prefix
            .current()
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;
        audit.validate_against_projection(projection, frontier, claim)?;
        self.validate_against_audit_and_chain(audit, &prefix, claim)
    }

    /// Strict provenance gate for new consumers: the replay receipt must
    /// validate against the originating projection with a fully committed
    /// frontier manifest and projection semantic identity.
    pub fn validate_strong_against_projection(
        &self,
        projection: &crate::cultural_systems::CulturalProjectionV1,
        audit: &CulturalProjectionAuditV5,
        chain: &EvidenceFrontierChainV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Result<(), ProjectionError> {
        let index = chain
            .frontiers
            .iter()
            .position(|frontier| frontier.frontier_id == self.leaf_frontier)
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;
        let prefix = EvidenceFrontierChainV1 {
            frontiers: chain.frontiers[..=index].to_vec(),
        };
        let frontier = prefix
            .current()
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;

        audit.validate_strong_against_projection(projection, frontier, claim)?;
        self.validate_against_audit_and_chain(audit, &prefix, claim)
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
            (
                a.kind,
                a.assessment.clone(),
                a.interpretation.clone(),
                a.semantic_hash.clone(),
            )
                .cmp(&(
                    b.kind,
                    b.assessment.clone(),
                    b.interpretation.clone(),
                    b.semantic_hash.clone(),
                ))
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
            frontier_lineage: chain
                .frontiers
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
            (
                a.kind,
                a.assessment.clone(),
                a.interpretation.clone(),
                a.semantic_hash.clone(),
            )
                .cmp(&(
                    b.kind,
                    b.assessment.clone(),
                    b.interpretation.clone(),
                    b.semantic_hash.clone(),
                ))
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
    use crate::cultural_argumentation::{CulturalArgumentationKindV1, CulturalArgumentationRefV3};
    use crate::cultural_projection_audit_v5::CulturalProjectionAuditV5;
    use crate::cultural_systems::{
        CanonicalClaimAdmissionV1, CulturalArgumentationEvidenceClosureV1,
        CulturalArgumentationRefV2,
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
                    evidence_id: "e:1".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(YearInterval {
                        from: Some(1940),
                        to: Some(1940),
                    }),
                    publication_time: Some(1941),
                    capture_time: None,
                    available_by: 1942,
                    validity_time: Some(YearInterval {
                        from: Some(1940),
                        to: Some(1950),
                    }),
                },
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:2".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(YearInterval {
                        from: Some(1945),
                        to: Some(1945),
                    }),
                    publication_time: Some(1946),
                    capture_time: None,
                    available_by: 1947,
                    validity_time: Some(YearInterval {
                        from: Some(1940),
                        to: Some(1950),
                    }),
                },
            ],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
            }],
            argumentation_metadata: vec![crate::civilizational::ArgumentationTemporalMetadataV1 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                assessment_time: Some(YearInterval {
                    from: Some(1948),
                    to: Some(1948),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1949),
                    to: Some(1949),
                }),
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
            event_time: YearInterval {
                from: Some(1900),
                to: Some(1950),
            },
            qualification: claim.qualification,
            access_policy: crate::cultural_systems::AccessPolicyV1::Public,
            evidence_frontier: child.frontier_id.clone(),
            frontier_manifest_hash: String::new(),
            projection_semantic_hash: String::new(),
        };

        let mapping = OntologyMappingV2::from_claim(
            "mapping:1",
            OntologyMappingStandardV1::CidocCrm,
            "7.4",
            OntologyReleaseStatusV1::Draft,
            "E7_Activity",
            OntologyMappingKindV1::Class,
            &claim,
        );
        let resolution = crate::ontology_resolution::OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &child,
        )
        .expect("resolution");
        let v4 = crate::cultural_projection_audit_v4::CulturalProjectionAuditV4::from_v2(
            base,
            vec![resolution],
        )
        .expect("v4 audit");

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
                assessment_time: Some(YearInterval {
                    from: Some(1948),
                    to: Some(1948),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1949),
                    to: Some(1949),
                }),
                available_by: 1950,
            },
            &child,
        )
        .expect("typed argumentation");

        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");

        (
            audit,
            claim,
            EvidenceFrontierChainV1 {
                frontiers: vec![root, child],
            },
        )
    }

    fn projection_fixture() -> crate::cultural_systems::CulturalProjectionV1 {
        crate::cultural_systems::CulturalProjectionV1::Transmission(
            crate::cultural_systems::CulturalTransmissionV1 {
                transmission_id: "transmission:1".into(),
                source: "practice:source".into(),
                target: "practice:target".into(),
                mode: crate::cultural_systems::TransmissionMode::Translated,
                event_time: YearInterval {
                    from: Some(1900),
                    to: Some(1950),
                },
                context: Some("documented".into()),
                claim_ref: "claim:1".into(),
                evidence_refs: vec!["e:1".into(), "e:2".into()],
                source_snapshots: vec!["source:1".into()],
                assessment: Some("assessment:1".into()),
                qualification: QualificationStatus::Supported,
                community_recognition: vec![],
                access_policy: crate::cultural_systems::AccessPolicyV1::Public,
                evidence_frontier: "frontier:1951".into(),
            },
        )
    }

    #[test]
    fn projection_validation_rejects_duplicate_later_frontier_identity() {
        let (audit, claim, chain) = fixture();
        let receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");

        let mut extended = chain.clone();
        let mut duplicate = extended.frontiers[0].clone();
        duplicate.manifest_hash = "duplicate-later-unvalidated".into();
        extended.frontiers.push(duplicate);

        assert_eq!(
            receipt.validate_against_projection(
                &projection_fixture(),
                &audit,
                &extended,
                &claim,
            ),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
        assert_eq!(
            receipt.validate_strong_against_projection(
                &projection_fixture(),
                &audit,
                &extended,
                &claim,
            ),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn strict_constructor_round_trips_against_originating_projection() {
        let (audit, claim, chain) = fixture();
        let frontier = chain.current().expect("leaf frontier");
        let projection = crate::cultural_systems::CulturalProjectionV1::Transmission(
            crate::cultural_systems::CulturalTransmissionV1 {
                transmission_id: "transmission:1".into(),
                source: "practice:source".into(),
                target: "practice:target".into(),
                mode: crate::cultural_systems::TransmissionMode::Translated,
                event_time: YearInterval {
                    from: Some(1900),
                    to: Some(1950),
                },
                context: Some("documented".into()),
                claim_ref: "claim:1".into(),
                evidence_refs: vec!["e:1".into(), "e:2".into()],
                source_snapshots: vec!["source:1".into()],
                assessment: Some("assessment:1".into()),
                qualification: crate::civilizational::QualificationStatus::Supported,
                community_recognition: vec![],
                access_policy: crate::cultural_systems::AccessPolicyV1::Public,
                evidence_frontier: frontier.frontier_id.clone(),
            },
        );

        let strong_audit = CulturalProjectionAuditV5::from_projection_at(
            &projection,
            frontier,
            &claim,
            audit.base.resolutions.clone(),
            audit.argumentation.clone(),
        )
        .expect("strong audit");
        let receipt = V5ReplayReceiptV1::from_projection_and_chain(
            &projection,
            &strong_audit,
            &chain,
            &claim,
        )
        .expect("strict replay receipt");

        assert_eq!(
            receipt.validate_strong_against_projection(&projection, &strong_audit, &chain, &claim),
            Ok(())
        );
        assert_eq!(receipt.leaf_frontier, frontier.frontier_id);
    }

    #[test]
    fn strict_constructor_rejects_corrupted_ancestry() {
        let (legacy_audit, claim, mut chain) = fixture();
        let frontier = chain.current().expect("leaf frontier").clone();
        let projection = crate::cultural_systems::CulturalProjectionV1::Transmission(
            crate::cultural_systems::CulturalTransmissionV1 {
                transmission_id: "transmission:1".into(),
                source: "practice:source".into(),
                target: "practice:target".into(),
                mode: crate::cultural_systems::TransmissionMode::Translated,
                event_time: YearInterval {
                    from: Some(1900),
                    to: Some(1950),
                },
                context: Some("documented".into()),
                claim_ref: "claim:1".into(),
                evidence_refs: vec!["e:1".into(), "e:2".into()],
                source_snapshots: vec!["source:1".into()],
                assessment: Some("assessment:1".into()),
                qualification: crate::civilizational::QualificationStatus::Supported,
                community_recognition: vec![],
                access_policy: crate::cultural_systems::AccessPolicyV1::Public,
                evidence_frontier: frontier.frontier_id.clone(),
            },
        );
        let strong_audit = CulturalProjectionAuditV5::from_projection_at(
            &projection,
            &frontier,
            &claim,
            legacy_audit.base.resolutions.clone(),
            legacy_audit.argumentation.clone(),
        )
        .expect("strong audit");

        chain.frontiers[0].policy_version = "rewritten-ancestor".into();
        chain.frontiers[0]
            .recompute_manifest_hash()
            .expect("rewritten ancestor hash");

        assert_eq!(
            V5ReplayReceiptV1::from_projection_and_chain(
                &projection,
                &strong_audit,
                &chain,
                &claim
            ),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn receipt_round_trip_validates() {
        let (audit, claim, chain) = fixture();
        let receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        assert_eq!(
            receipt.validate_against_audit_and_chain(&audit, &chain, &claim),
            Ok(())
        );
    }

    #[test]
    fn receipt_reciprocal_projection_validation_rejects_rebinding() {
        let (audit, claim, chain) = fixture();
        let receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");

        let mut projection = crate::cultural_systems::CulturalProjectionV1::Transmission(
            crate::cultural_systems::CulturalTransmissionV1 {
                transmission_id: "transmission:1".into(),
                source: "practice:source".into(),
                target: "practice:target".into(),
                mode: crate::cultural_systems::TransmissionMode::Translated,
                event_time: YearInterval {
                    from: Some(1900),
                    to: Some(1950),
                },
                context: Some("documented".into()),
                claim_ref: "claim:1".into(),
                evidence_refs: vec!["e:1".into(), "e:2".into()],
                source_snapshots: vec!["source:1".into()],
                assessment: Some("assessment:1".into()),
                qualification: crate::civilizational::QualificationStatus::Supported,
                community_recognition: vec![],
                access_policy: crate::cultural_systems::AccessPolicyV1::Public,
                evidence_frontier: "frontier:1951".into(),
            },
        );

        assert_eq!(
            receipt.validate_against_projection(&projection, &audit, &chain, &claim),
            Ok(())
        );

        projection = match projection {
            crate::cultural_systems::CulturalProjectionV1::Transmission(mut value) => {
                value.access_policy = crate::cultural_systems::AccessPolicyV1::Sensitive;
                crate::cultural_systems::CulturalProjectionV1::Transmission(value)
            }
            _ => unreachable!(),
        };
        assert_eq!(
            receipt.validate_against_projection(&projection, &audit, &chain, &claim),
            Err(ProjectionError::AuditWithoutEvidencePath)
        );
    }

    #[test]
    fn strong_replay_validation_rejects_legacy_projection_identity() {
        let (audit, claim, chain) = fixture();
        let receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let projection = crate::cultural_systems::CulturalProjectionV1::Transmission(
            crate::cultural_systems::CulturalTransmissionV1 {
                transmission_id: "transmission:1".into(),
                source: "practice:source".into(),
                target: "practice:target".into(),
                mode: crate::cultural_systems::TransmissionMode::Translated,
                event_time: YearInterval {
                    from: Some(1900),
                    to: Some(1950),
                },
                context: Some("documented".into()),
                claim_ref: "claim:1".into(),
                evidence_refs: vec!["e:1".into(), "e:2".into()],
                source_snapshots: vec!["source:1".into()],
                assessment: Some("assessment:1".into()),
                qualification: QualificationStatus::Supported,
                community_recognition: vec![],
                access_policy: crate::cultural_systems::AccessPolicyV1::Public,
                evidence_frontier: "frontier:1951".into(),
            },
        );

        assert_eq!(
            receipt.validate_strong_against_projection(&projection, &audit, &chain, &claim),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn receipt_hash_changes_when_closure_changes() {
        let (audit, claim, chain) = fixture();
        let mut receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.evidence_refs.push("e:extra".into());
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &chain, &claim)
                .is_err()
        );
    }

    #[test]
    fn receipt_hash_changes_when_lineage_changes() {
        let (audit, claim, chain) = fixture();
        let mut receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.frontier_lineage[0].1 = "tampered-manifest".into();
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &chain, &claim)
                .is_err()
        );
    }

    #[test]
    fn receipt_is_order_independent_for_set_like_closure() {
        let (audit, claim, chain) = fixture();
        let mut receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let original = receipt.receipt_hash.clone();
        receipt.evidence_refs.reverse();
        receipt.source_snapshots.reverse();
        receipt.canonicalize();
        receipt.recompute_hash().expect("rehash");
        assert_eq!(original, receipt.receipt_hash);
    }

    #[test]
    fn independently_reconstructed_equivalent_closure_preserves_receipt_hash() {
        let (audit, claim, chain) = fixture();
        let original = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("original receipt");

        let mut equivalent_chain = chain.clone();
        equivalent_chain.frontiers[0].evidence_metadata.reverse();
        equivalent_chain.frontiers[0].admitted_evidence =
            ["e:2", "e:1"].into_iter().map(Into::into).collect();
        equivalent_chain.frontiers[0].admitted_sources =
            ["source:1"].into_iter().map(Into::into).collect();
        equivalent_chain.frontiers[0]
            .recompute_manifest_hash()
            .expect("equivalent root hash");
        assert_eq!(
            equivalent_chain.frontiers[0].manifest_hash,
            chain.frontiers[0].manifest_hash
        );
        assert_eq!(equivalent_chain.validate_strict(), Ok(()));

        let mut equivalent_audit = audit.clone();
        equivalent_audit.base.base.evidence_refs.reverse();
        if let Some(argumentation) = equivalent_audit.base.base.argumentation.as_mut() {
            argumentation.evidence_refs.reverse();
        }
        equivalent_audit
            .recompute_hash()
            .expect("equivalent audit hash");
        assert_eq!(equivalent_audit.semantic_hash, audit.semantic_hash);
        equivalent_audit.validate().expect("equivalent audit");

        let mut equivalent_claim = claim.clone();
        equivalent_claim.evidence_refs.reverse();
        let equivalent = V5ReplayReceiptV1::from_audit_and_chain(
            &equivalent_audit,
            &equivalent_chain,
            &equivalent_claim,
        )
        .expect("equivalent receipt");

        assert_eq!(original.receipt_hash, equivalent.receipt_hash);
        assert_eq!(original.frontier_lineage, equivalent.frontier_lineage);
    }

    #[test]
    fn argumentation_reconstruction_is_order_independent_but_identity_sensitive() {
        let (mut audit, claim, mut chain) = fixture();

        let mut second = audit.argumentation[0].clone();
        second.assessment = "assessment:2".into();
        second.interpretation = "interpretation:2".into();
        second.closure.evidence_refs = vec!["e:2".into()];
        second.recompute_hash().expect("second argumentation hash");
        audit.argumentation.push(second);

        for frontier in &mut chain.frontiers {
            frontier.argumentation_metadata.push(
                crate::civilizational::ArgumentationTemporalMetadataV1 {
                    assessment: "assessment:2".into(),
                    interpretation: "interpretation:2".into(),
                    assessment_time: Some(YearInterval {
                        from: Some(1948),
                        to: Some(1948),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1949),
                        to: Some(1949),
                    }),
                    available_by: 1950,
                },
            );
            frontier
                .recompute_manifest_hash()
                .expect("argumentation manifest hash");
        }
        audit
            .recompute_hash()
            .expect("multi-argumentation audit hash");
        audit
            .validate_against_frontier_chain(&chain, &claim)
            .expect("multi-argumentation audit");

        let original = V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim)
            .expect("original receipt");

        let mut reordered_audit = audit.clone();
        reordered_audit.argumentation.reverse();
        reordered_audit
            .recompute_hash()
            .expect("reordered audit hash");
        assert_eq!(reordered_audit.semantic_hash, audit.semantic_hash);

        let reordered = V5ReplayReceiptV1::from_audit_and_chain(&reordered_audit, &chain, &claim)
            .expect("reordered receipt");
        assert_eq!(original.receipt_hash, reordered.receipt_hash);
        assert_eq!(original.argumentation, reordered.argumentation);

        let mut identity_changed = reordered_audit.clone();
        identity_changed.argumentation[0].kind = CulturalArgumentationKindV1::MeaningComprehension;
        identity_changed.argumentation[0]
            .recompute_hash()
            .expect("identity hash");
        identity_changed
            .recompute_hash()
            .expect("identity audit hash");
        assert_ne!(
            identity_changed.semantic_hash,
            reordered_audit.semantic_hash
        );
        let identity_changed_receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&identity_changed, &chain, &claim)
                .expect("identity-changed receipt");
        assert_ne!(identity_changed_receipt.receipt_hash, original.receipt_hash);
    }

    #[test]
    fn receipt_hash_changes_when_argumentation_or_ontology_changes() {
        let (audit, claim, chain) = fixture();
        let mut receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let original = receipt.receipt_hash.clone();

        receipt.argumentation[0].semantic_hash = "tampered-argumentation".into();
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &chain, &claim)
                .is_err()
        );

        let mut receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        receipt.ontology_resolutions[0].resolution_hash = "tampered-resolution".into();
        assert_ne!(original, receipt.computed_hash().expect("hash"));
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &chain, &claim)
                .is_err()
        );
    }

    #[test]
    fn receipt_rejects_replaying_child_audit_at_parent_frontier() {
        let (audit, claim, chain) = fixture();
        let receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let parent_only = EvidenceFrontierChainV1 {
            frontiers: vec![chain.frontiers[0].clone()],
        };
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &parent_only, &claim)
                .is_err()
        );
    }

    #[test]
    fn receipt_cannot_change_qualification() {
        let (audit, claim, chain) = fixture();
        let mut receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        receipt.qualification = QualificationStatus::Established;
        receipt.recompute_hash().expect("rehash");
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &chain, &claim)
                .is_err()
        );
        assert_eq!(audit.qualification(), QualificationStatus::Supported);
    }

    #[test]
    fn receipt_rejects_unrelated_frontier_lineage() {
        let (audit, claim, chain) = fixture();
        let receipt =
            V5ReplayReceiptV1::from_audit_and_chain(&audit, &chain, &claim).expect("receipt");
        let mut unrelated = chain.clone();
        unrelated.frontiers[0].frontier_id = "frontier:unrelated".into();
        unrelated.frontiers[0]
            .recompute_manifest_hash()
            .expect("hash");
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &chain, &claim)
                .is_ok()
        );
        assert!(
            receipt
                .validate_against_audit_and_chain(&audit, &unrelated, &claim)
                .is_err()
        );
    }
}
