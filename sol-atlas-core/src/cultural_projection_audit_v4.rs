// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-closed semantic audit V4.
//!
//! V4 is additive to V3: it carries ontology mapping resolutions whose exact
//! claim/evidence/source closure is independently frontier-checked.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{ClaimId, EvidenceFrontierId, ProjectionError, QualificationStatus};
use crate::cultural_systems::{CulturalProjectionAuditV2, CulturalProjectionIdV1};
use crate::ontology_resolution::OntologyMappingResolutionV1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAuditV4 {
    pub base: CulturalProjectionAuditV2,
    pub resolutions: Vec<OntologyMappingResolutionV1>,
    pub semantic_hash: String,
}

impl CulturalProjectionAuditV4 {
    pub fn from_v2(base: CulturalProjectionAuditV2,
        resolutions: Vec<OntologyMappingResolutionV1>) -> Result<Self, ProjectionError> {
        let mut audit = Self { base, resolutions, semantic_hash: String::new() };
        audit.recompute_hash()?;
        audit.validate()?;
        Ok(audit)
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        self.base.validate()?;
        if self.resolutions.is_empty() || self.semantic_hash.trim().is_empty() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        for resolution in &self.resolutions {
            resolution.validate()?;
            if resolution.claim_ref != self.base.claim_ref
                || resolution.evidence_frontier != self.base.evidence_frontier
                || resolution.qualification != self.base.qualification {
                return Err(ProjectionError::EmptyIdentifier);
            }
        }
        if self.semantic_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let mut resolutions = self.resolutions.clone();
        resolutions.sort_by(|a, b| {
            (a.mapping.mapping_id.clone(), a.resolution_hash.clone())
                .cmp(&(b.mapping.mapping_id.clone(), b.resolution_hash.clone()))
        });
        let payload = (&self.base.projection_id, &self.base.claim_ref,
            &self.base.evidence_frontier, &self.base.qualification, &resolutions);
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.semantic_hash = self.computed_hash()?;
        Ok(())
    }

    pub fn is_frontier_safe(&self, frontier: &crate::civilizational::EvidenceFrontierV1,
        claims: &[crate::cultural_systems::CanonicalClaimAdmissionV1]) -> bool {
        self.validate().is_ok()
            && self.base.is_frontier_safe(frontier)
            && self.resolutions.iter().all(|resolution| {
                claims.iter().any(|claim| resolution.is_frontier_safe(claim, frontier)
                    && claim.claim_ref == self.base.claim_ref)
            })
    }

    pub fn projection_id(&self) -> &CulturalProjectionIdV1 { &self.base.projection_id }
    pub fn claim_ref(&self) -> &ClaimId { &self.base.claim_ref }
    pub fn evidence_frontier(&self) -> &EvidenceFrontierId { &self.base.evidence_frontier }
    pub fn qualification(&self) -> QualificationStatus { self.base.qualification }
}
