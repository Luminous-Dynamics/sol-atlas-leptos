// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-closed resolution for semantic ontology mappings.
//!
//! A mapping context identifies the external vocabulary used by a renderer.
//! A resolution additionally proves that the exact mapping resolves through
//! the canonical claim, evidence/source closure, and temporal frontier that
//! made the projection admissible.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{
    ClaimId, EvidenceFrontierId, EvidenceFrontierV1, EvidenceId, ProjectionError,
    QualificationStatus, SourceSnapshotId,
};
use crate::cultural_systems::CanonicalClaimAdmissionV1;
use crate::ontology_context::{OntologyMappingContextV1, OntologyMappingRelationV1};
use crate::ontology_mapping::OntologyMappingV2;

/// Content-addressed proof that one ontology mapping resolves to the exact
/// canonical claim/evidence/source closure used by the projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyMappingResolutionV1 {
    pub mapping: OntologyMappingContextV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub qualification: QualificationStatus,
    pub evidence_frontier: EvidenceFrontierId,
    pub resolution_hash: String,
}

impl OntologyMappingResolutionV1 {
    pub fn from_mapping(
        mapping: &OntologyMappingV2,
        relation: OntologyMappingRelationV1,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> Result<Self, ProjectionError> {
        if !mapping.is_frontier_safe(claim, frontier) {
            return Err(ProjectionError::EmptyIdentifier);
        }
        let context = OntologyMappingContextV1::from_mapping(mapping, relation)?;
        let mut resolution = Self {
            mapping: context,
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            qualification: claim.qualification,
            evidence_frontier: frontier.frontier_id.clone(),
            resolution_hash: String::new(),
        };
        resolution.recompute_hash()?;
        Ok(resolution)
    }

    fn canonical_payload(
        &self,
    ) -> (
        &OntologyMappingContextV1,
        &ClaimId,
        &Vec<EvidenceId>,
        &Vec<SourceSnapshotId>,
        &QualificationStatus,
        &EvidenceFrontierId,
    ) {
        (
            &self.mapping,
            &self.claim_ref,
            &self.evidence_refs,
            &self.source_snapshots,
            &self.qualification,
            &self.evidence_frontier,
        )
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let bytes = serde_json::to_vec(&self.canonical_payload())
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.resolution_hash = self.computed_hash()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        self.mapping.validate()?;
        if !self.claim_ref.is_valid()
            || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || !self.evidence_frontier.is_valid()
            || self.resolution_hash.trim().is_empty()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.mapping.qualification != self.qualification {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.resolution_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    pub fn is_frontier_safe(
        &self,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> bool {
        self.validate().is_ok()
            && claim.is_frontier_safe(frontier)
            && self.claim_ref == claim.claim_ref
            && self.evidence_refs == claim.evidence_refs
            && self.source_snapshots == claim.source_snapshots
            && self.qualification == claim.qualification
            && self.evidence_frontier == frontier.frontier_id
            && self.mapping.qualification == claim.qualification
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1, YearInterval,
    };
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyReleaseStatusV1,
    };

    fn fixture() -> (OntologyMappingV2, CanonicalClaimAdmissionV1, EvidenceFrontierV1) {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:1"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(),
                source_snapshot: "source:1".into(),
                artifact_time: Some(1940),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
                validity_time: Some(YearInterval { from: Some(1940), to: Some(1950) }),
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
            }],
            argumentation_metadata: vec![],
        };
        frontier.recompute_manifest_hash().expect("fixture hash");
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
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
        (mapping, claim, frontier)
    }

    #[test]
    fn resolution_binds_mapping_to_exact_closure() {
        let (mapping, claim, frontier) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &frontier,
        ).expect("resolution");
        assert!(resolution.validate().is_ok());
        assert!(resolution.is_frontier_safe(&claim, &frontier));
    }

    #[test]
    fn evidence_or_source_substitution_breaks_frontier_safety() {
        let (mapping, claim, frontier) = fixture();
        let mut resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &frontier,
        ).expect("resolution");
        resolution.evidence_refs = vec!["e:other".into()];
        resolution.recompute_hash().expect("rehash");
        assert!(!resolution.is_frontier_safe(&claim, &frontier));

        let mut resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &frontier,
        ).expect("resolution");
        resolution.source_snapshots = vec!["source:other".into()];
        resolution.recompute_hash().expect("rehash");
        assert!(!resolution.is_frontier_safe(&claim, &frontier));
    }

    #[test]
    fn claim_or_frontier_substitution_breaks_frontier_safety() {
        let (mapping, claim, frontier) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &frontier,
        ).expect("resolution");

        let other_claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:other".into(),
            ..claim.clone()
        };
        assert!(!resolution.is_frontier_safe(&other_claim, &frontier));

        let mut other_frontier = frontier.clone();
        other_frontier.frontier_id = "frontier:1960".into();
        other_frontier.recompute_manifest_hash().expect("rehash");
        assert!(!resolution.is_frontier_safe(&claim, &other_frontier));
    }

    #[test]
    fn semantic_tamper_invalidates_content_address() {
        let (mapping, claim, frontier) = fixture();
        let mut resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping, OntologyMappingRelationV1::Exact, &claim, &frontier,
        ).expect("resolution");
        resolution.mapping.external_term = "E8_Acquisition".into();
        assert!(resolution.validate().is_err());
    }
}
