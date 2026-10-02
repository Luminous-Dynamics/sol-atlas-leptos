// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Additive typed-argumentation audit V5.
//!
//! V5 layers CRMinf-aligned argumentation kinds onto the V4 semantic audit.
//! It preserves V1/V2 argumentation contracts while making the typed
//! argumentation records part of the projection's content-addressed audit.
//!
//! The argumentation kind is semantic identity only. It is not a confidence
//! score, ranking, or qualification upgrade.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{
    AssessmentId, EvidenceFrontierId, ProjectionError, QualificationStatus,
};
use crate::cultural_argumentation::CulturalArgumentationRefV3;
use crate::cultural_projection_audit_v4::CulturalProjectionAuditV4;
use crate::cultural_systems::CulturalProjectionIdV1;

fn has_duplicate_argumentation_identity(values: &[CulturalArgumentationRefV3]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    values.iter().any(|value| {
        !seen.insert((
            value.kind,
            value.assessment.clone(),
            value.interpretation.clone(),
        ))
    })
}

/// Content-addressed V5 audit carrying typed argumentation alternatives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAuditV5 {
    pub base: CulturalProjectionAuditV4,
    /// Competing argumentation records are retained without epistemic ranking.
    pub argumentation: Vec<CulturalArgumentationRefV3>,
    pub semantic_hash: String,
}

impl CulturalProjectionAuditV5 {
    pub fn from_v4(
        base: CulturalProjectionAuditV4,
        argumentation: Vec<CulturalArgumentationRefV3>,
    ) -> Result<Self, ProjectionError> {
        let mut audit = Self {
            base,
            argumentation,
            semantic_hash: String::new(),
        };
        audit.recompute_hash()?;
        audit.validate()?;
        Ok(audit)
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        self.base.validate()?;
        if self.semantic_hash.trim().is_empty()
            || has_duplicate_argumentation_identity(&self.argumentation)
        {
            return Err(ProjectionError::EmptyIdentifier);
        }

        for value in &self.argumentation {
            value.validate()?;
            if value.claim_ref != *self.base.claim_ref()
                || value.evidence_frontier != *self.base.evidence_frontier()
            {
                return Err(ProjectionError::EmptyIdentifier);
            }
        }

        if let Some(legacy) = self.base.base.argumentation.as_ref() {
            if !self.argumentation.iter().any(|value| {
                value.assessment == legacy.assessment
                    && value.interpretation == legacy.interpretation
                    && value.claim_ref == legacy.claim_ref
                    && value.closure.evidence_refs == legacy.evidence_refs
                    && value.closure.source_snapshots == legacy.source_snapshots
                    && value.closure.evidence_frontier == legacy.evidence_frontier
                    && value.assessment_time == legacy.assessment_time
                    && value.interpretation_time == legacy.interpretation_time
                    && value.available_by == legacy.available_by
            }) {
                return Err(ProjectionError::EmptyIdentifier);
            }
        }

        if self.semantic_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let mut argumentation = self.argumentation.clone();
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
        let payload = (
            &self.base.semantic_hash,
            &self.base.base.projection_id,
            &self.base.base.claim_ref,
            &self.base.base.evidence_frontier,
            &self.base.base.qualification,
            &argumentation,
        );
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.semantic_hash = self.computed_hash()?;
        Ok(())
    }

    pub fn canonicalize(&mut self) {
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
    }

    pub fn is_frontier_safe(
        &self,
        frontier: &crate::civilizational::EvidenceFrontierV1,
        claim: &crate::cultural_systems::CanonicalClaimAdmissionV1,
    ) -> bool {
        self.validate().is_ok()
            && self.base.is_frontier_safe(frontier, claim)
            && self
                .argumentation
                .iter()
                .all(|value| value.is_frontier_safe(claim, frontier))
    }

    /// Validates the complete V5 audit against an explicit append-only
    /// frontier lineage. The selected frontier must be the verified chain leaf;
    /// every canonical claim/evidence/source closure, typed argumentation
    /// record, and ontology resolution is then checked against that same leaf.
    ///
    /// This is intentionally stronger than `is_frontier_safe`: a caller cannot
    /// present a valid later frontier while asking the audit to masquerade as
    /// an earlier historical view.
    pub fn validate_against_frontier_chain(
        &self,
        chain: &crate::civilizational::EvidenceFrontierChainV1,
        claim: &crate::cultural_systems::CanonicalClaimAdmissionV1,
    ) -> Result<(), ProjectionError> {
        chain.validate_strict()?;
        let frontier = chain
            .current()
            .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;

        if self.evidence_frontier() != &frontier.frontier_id
            || claim.evidence_frontier != frontier.frontier_id
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        if !claim.is_frontier_safe(frontier) {
            return Err(ProjectionError::AuditWithoutEvidencePath);
        }

        self.validate()?;
        if !self.is_frontier_safe(frontier, claim) {
            return Err(ProjectionError::AuditWithoutEvidencePath);
        }

        Ok(())
    }

    /// Boolean convenience wrapper for chain-aware replay/admission.
    pub fn is_chain_safe(
        &self,
        chain: &crate::civilizational::EvidenceFrontierChainV1,
        claim: &crate::cultural_systems::CanonicalClaimAdmissionV1,
    ) -> bool {
        self.validate_against_frontier_chain(chain, claim).is_ok()
    }
    pub fn projection_id(&self) -> &CulturalProjectionIdV1 {
        self.base.projection_id()
    }

    pub fn claim_ref(&self) -> &crate::civilizational::ClaimId {
        self.base.claim_ref()
    }

    pub fn evidence_frontier(&self) -> &EvidenceFrontierId {
        self.base.evidence_frontier()
    }

    pub fn qualification(&self) -> QualificationStatus {
        self.base.qualification()
    }

    pub fn argumentation_identities(
        &self,
    ) -> Vec<(
        crate::cultural_argumentation::CulturalArgumentationKindV1,
        AssessmentId,
        crate::civilizational::InterpretationId,
    )> {
        let mut identities = self
            .argumentation
            .iter()
            .map(|value| {
                (
                    value.kind,
                    value.assessment.clone(),
                    value.interpretation.clone(),
                )
            })
            .collect::<Vec<_>>();
        identities.sort();
        identities
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
    use crate::cultural_systems::{
        CanonicalClaimAdmissionV1, CulturalArgumentationEvidenceClosureV1,
    };
    use crate::ontology_context::OntologyMappingRelationV1;
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn fixture() -> (
        CulturalProjectionAuditV4,
        CanonicalClaimAdmissionV1,
        EvidenceFrontierV1,
        CulturalArgumentationRefV3,
    ) {
        let mut frontier = EvidenceFrontierV1 {
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
                    artifact_time: Some(1945),
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
        frontier.recompute_manifest_hash().expect("frontier hash");

        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into(), "e:2".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: frontier.frontier_id.clone(),
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
            evidence_frontier: frontier.frontier_id.clone(),
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
            &frontier,
        )
        .expect("resolution");
        let v4 = CulturalProjectionAuditV4::from_v2(base, vec![resolution]).expect("v4 audit");

        let value = CulturalArgumentationRefV3::from_v2(
            CulturalArgumentationKindV1::InferenceMaking,
            crate::cultural_systems::CulturalArgumentationRefV2 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                claim_ref: claim.claim_ref.clone(),
                closure: CulturalArgumentationEvidenceClosureV1 {
                    claim_ref: claim.claim_ref.clone(),
                    evidence_refs: vec!["e:1".into()],
                    source_snapshots: vec!["source:1".into()],
                    evidence_frontier: frontier.frontier_id.clone(),
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
            &frontier,
        )
        .expect("typed argumentation");
        (v4, claim, frontier, value)
    }

    #[test]
    fn v5_binds_typed_argumentation_to_v4_identity() {
        let (v4, claim, frontier, argumentation) = fixture();
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");
        assert!(audit.validate().is_ok());
        assert!(audit.is_frontier_safe(&frontier, &claim));
    }

    #[test]
    fn changing_argumentation_kind_changes_audit_hash() {
        let (v4, _claim, _frontier, argumentation) = fixture();
        let mut audit =
            CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");
        let original = audit.semantic_hash.clone();
        audit.argumentation[0].kind = CulturalArgumentationKindV1::MeaningComprehension;
        assert_ne!(original, audit.computed_hash().expect("hash"));
        assert!(audit.validate().is_err());
    }

    #[test]
    fn v5_rejects_argumentation_for_another_claim() {
        let (v4, _claim, frontier, mut argumentation) = fixture();
        argumentation.claim_ref = "claim:other".into();
        argumentation.closure.claim_ref = "claim:other".into();
        argumentation.recompute_hash().expect("rehash");
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]);
        assert!(audit.is_err());
        let _ = frontier;
    }

    #[test]
    fn v5_preserves_competing_argumentation_without_ranking() {
        let (v4, claim, mut frontier, first) = fixture();
        let mut second = first.clone();
        second.assessment = "assessment:2".into();
        second.interpretation = "interpretation:2".into();
        second.closure.evidence_refs = vec!["e:2".into()];
        second.recompute_hash().expect("rehash");

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
        frontier.recompute_manifest_hash().expect("frontier hash");
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![first, second]).expect("v5 audit");
        assert_eq!(audit.argumentation_identities().len(), 2);
        assert!(audit.is_frontier_safe(&frontier, &claim));
    }

    #[test]
    fn v5_rejects_typed_record_that_drifts_from_legacy_argumentation() {
        let (mut v4, _claim, _frontier, mut typed) = fixture();
        v4.base.argumentation = Some(crate::cultural_systems::CulturalArgumentationRefV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into(), "e:2".into()],
            source_snapshots: vec!["source:1".into()],
            assessment_time: Some(YearInterval {
                from: Some(1948),
                to: Some(1948),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1949),
                to: Some(1949),
            }),
            available_by: 1950,
            evidence_frontier: "frontier:1950".into(),
        });
        typed.closure.evidence_refs = vec!["e:1".into()];
        typed.recompute_hash().expect("rehash");
        let result = CulturalProjectionAuditV5::from_v4(v4, vec![typed]);
        assert_eq!(result, Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn v5_rejects_duplicate_typed_identity() {
        let (v4, _claim, _frontier, first) = fixture();
        let duplicate = first.clone();
        let result = CulturalProjectionAuditV5::from_v4(v4, vec![first, duplicate]);
        assert_eq!(result, Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn v5_argumentation_cannot_upgrade_qualification() {
        let (v4, mut claim, frontier, argumentation) = fixture();
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");
        claim.qualification = QualificationStatus::Established;
        assert_eq!(audit.qualification(), QualificationStatus::Supported);
        assert!(!audit.is_frontier_safe(&frontier, &claim));
        claim.qualification = QualificationStatus::Supported;
        assert!(audit.is_frontier_safe(&frontier, &claim));
    }

    #[test]
    fn v5_chain_replay_requires_selected_frontier_to_be_verified_leaf() {
        let (v4, claim, child, argumentation) = fixture();
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");
        let mut root = child.clone();
        root.frontier_id = "frontier:1949".into();
        root.known_by_year = 1949;
        root.parent_frontier = None;
        root.argumentation_metadata[0].available_by = 1949;
        root.recompute_manifest_hash().expect("root hash");
        let chain = crate::civilizational::EvidenceFrontierChainV1 {
            frontiers: vec![root],
        };
        assert_eq!(
            audit.validate_against_frontier_chain(&chain, &claim),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
        assert!(!audit.is_chain_safe(&chain, &claim));
    }

    #[test]
    fn v5_chain_replay_accepts_complete_same_leaf_closure() {
        let (v4, claim, frontier, argumentation) = fixture();
        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");
        let mut root = frontier.clone();
        root.parent_frontier = None;
        root.recompute_manifest_hash().expect("root hash");
        let chain = crate::civilizational::EvidenceFrontierChainV1 {
            frontiers: vec![root],
        };
        assert!(audit.is_chain_safe(&chain, &claim));
    }
}
