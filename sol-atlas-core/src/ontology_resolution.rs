// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-closed resolution for semantic ontology mappings.
//!
//! A mapping context identifies the external vocabulary used by a renderer.
//! A resolution additionally proves that the exact mapping resolves through
//! the canonical claim, evidence/source closure, and temporal frontier that
//! made the projection admissible. This prevents semantic interoperability
//! metadata from becoming an evidence-free shortcut.

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
        if self.mapping.preserves_mapping_fields_for_resolution(
            &self.claim_ref,
            &self.evidence_refs,
            &self.source_snapshots,
            &self.qualification,
            &self.evidence_frontier,
        ) == false {
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
