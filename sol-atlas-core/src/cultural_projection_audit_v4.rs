// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-closed semantic audit V4.
//!
//! V4 is additive to V3: it carries ontology mapping resolutions whose exact
//! claim/evidence/source closure is independently frontier-checked.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{ClaimId, EvidenceFrontierId, ProjectionError, QualificationStatus};
use crate::cultural_systems::{
    CanonicalClaimAdmissionV1, CulturalProjectionAuditV2, CulturalProjectionIdV1,
};

fn has_duplicate_mapping_ids(resolutions: &[OntologyMappingResolutionV1]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    resolutions
        .iter()
        .any(|resolution| !seen.insert(resolution.mapping.mapping_id.clone()))
}
use crate::ontology_resolution::OntologyMappingResolutionV1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAuditV4 {
    pub base: CulturalProjectionAuditV2,
    pub resolutions: Vec<OntologyMappingResolutionV1>,
    pub semantic_hash: String,
}

impl CulturalProjectionAuditV4 {
    pub fn from_v2(
        base: CulturalProjectionAuditV2,
        resolutions: Vec<OntologyMappingResolutionV1>,
    ) -> Result<Self, ProjectionError> {
        let mut audit = Self {
            base,
            resolutions,
            semantic_hash: String::new(),
        };
        audit.recompute_hash()?;
        audit.validate()?;
        Ok(audit)
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        self.base.validate()?;
        if self.resolutions.is_empty() || self.semantic_hash.trim().is_empty() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if has_duplicate_mapping_ids(&self.resolutions) {
            return Err(ProjectionError::EmptyIdentifier);
        }

        let mut base_evidence_refs = self.base.evidence_refs.clone();
        base_evidence_refs.sort();
        let mut base_source_snapshots = self.base.source_snapshots.clone();
        base_source_snapshots.sort();

        for resolution in &self.resolutions {
            resolution.validate()?;
            let mut resolution_evidence_refs = resolution.evidence_refs.clone();
            resolution_evidence_refs.sort();
            let mut resolution_source_snapshots = resolution.source_snapshots.clone();
            resolution_source_snapshots.sort();
            if resolution.claim_ref != self.base.claim_ref
                || resolution_evidence_refs != base_evidence_refs
                || resolution_source_snapshots != base_source_snapshots
                || resolution.evidence_frontier != self.base.evidence_frontier
                || resolution.qualification != self.base.qualification
            {
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
        // Ontology mapping resolution closure vectors are membership sets, so
        // nested evidence/source ordering must not alter the V4 semantic hash.
        for resolution in &mut resolutions {
            resolution.canonicalize();
        }
        resolutions.sort_by(|a, b| {
            (a.mapping.mapping_id.clone(), a.resolution_hash.clone())
                .cmp(&(b.mapping.mapping_id.clone(), b.resolution_hash.clone()))
        });
        let payload = (
            &self.base.projection_id,
            &self.base.claim_ref,
            &self.base.evidence_frontier,
            &self.base.qualification,
            &resolutions,
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

    pub fn is_frontier_safe(
        &self,
        frontier: &crate::civilizational::EvidenceFrontierV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> bool {
        self.validate().is_ok()
            && self.base.is_frontier_safe(frontier)
            && claim.claim_ref == self.base.claim_ref
            && self.base.evidence_refs == claim.evidence_refs
            && self.base.source_snapshots == claim.source_snapshots
            && self
                .resolutions
                .iter()
                .all(|resolution| resolution.is_frontier_safe(claim, frontier))
    }

    pub fn projection_id(&self) -> &CulturalProjectionIdV1 {
        &self.base.projection_id
    }
    pub fn claim_ref(&self) -> &ClaimId {
        &self.base.claim_ref
    }
    pub fn evidence_frontier(&self) -> &EvidenceFrontierId {
        &self.base.evidence_frontier
    }
    pub fn qualification(&self) -> QualificationStatus {
        self.base.qualification
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceFrontierV1, EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1,
        YearInterval,
    };
    use crate::cultural_systems::{CulturalProjectionAuditV2, CulturalProjectionIdV1};
    use crate::ontology_context::OntologyMappingRelationV1;
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn fixture() -> (
        CulturalProjectionAuditV2,
        CanonicalClaimAdmissionV1,
        EvidenceFrontierV1,
        OntologyMappingV2,
    ) {
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
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
            }],
            argumentation_metadata: vec![],
        };
        frontier.recompute_manifest_hash().expect("frontier hash");
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
        };
        let base = CulturalProjectionAuditV2 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            community_recognition_evidence: vec![],
            assessment: None,
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
        (base, claim, frontier, mapping)
    }

    #[test]
    fn v4_rejects_duplicate_mapping_identity() {
        let (base, claim, frontier, mapping) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &frontier,
        )
        .expect("resolution");
        let mut duplicate = resolution.clone();
        duplicate.mapping.external_term = "E8_Acquisition".into();
        duplicate.mapping.recompute_hash().expect("mapping hash");
        duplicate.recompute_hash().expect("resolution hash");
        let audit = CulturalProjectionAuditV4::from_v2(base, vec![resolution, duplicate]);
        assert_eq!(audit, Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn v4_frontier_safety_rejects_admitted_but_rebound_evidence_closure() {
        let (base, claim, frontier, mapping) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &frontier,
        )
        .expect("resolution");
        let mut audit = CulturalProjectionAuditV4::from_v2(base, vec![resolution]).expect("audit");

        audit.base.evidence_refs = vec!["e:other".into()];
        audit.recompute_hash().expect("audit hash");

        let mut rebound_frontier = frontier.clone();
        rebound_frontier.admitted_evidence.insert("e:other".into());
        rebound_frontier
            .evidence_metadata
            .push(EvidenceTemporalMetadataV1 {
                evidence_id: "e:other".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1949),
                capture_time: None,
                available_by: 1949,
                validity_time: None,
            });
        rebound_frontier
            .recompute_manifest_hash()
            .expect("frontier hash");

        assert!(!audit.is_frontier_safe(&rebound_frontier, &claim));
    }

    #[test]
    fn v4_frontier_safety_is_bound_to_one_canonical_claim() {
        let (base, claim, frontier, mapping) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &frontier,
        )
        .expect("resolution");
        let audit = CulturalProjectionAuditV4::from_v2(base, vec![resolution]).expect("audit");
        let other_claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:other".into(),
            ..claim
        };
        assert!(!audit.is_frontier_safe(&frontier, &other_claim));
    }

    #[test]
    fn v4_validation_rejects_resolution_closure_drift_from_audit() {
        let (base, claim, frontier, mapping) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &frontier,
        )
        .expect("resolution");
        let mut audit = CulturalProjectionAuditV4::from_v2(base, vec![resolution]).expect("audit");

        audit.base.evidence_refs = vec!["e:other".into()];
        assert_eq!(audit.validate(), Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn v4_hash_covers_resolution_identity_and_rejects_tamper() {
        let (base, claim, frontier, mapping) = fixture();
        let resolution = OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &frontier,
        )
        .expect("resolution");
        let mut audit = CulturalProjectionAuditV4::from_v2(base, vec![resolution]).expect("audit");
        audit.resolutions[0].mapping.external_term = "E8_Acquisition".into();
        audit.resolutions[0]
            .mapping
            .recompute_hash()
            .expect("mapping hash");
        audit.resolutions[0]
            .recompute_hash()
            .expect("resolution hash");
        assert!(audit.validate().is_err());
    }
}
