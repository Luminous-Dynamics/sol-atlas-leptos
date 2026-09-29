// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-bearing interoperability mappings for cultural projections.
//!
//! A mapping says how a Sol Atlas projection concept can be represented in an
//! external ontology. It is not an ontology assertion and cannot upgrade the
//! epistemic qualification of the mapped claim.

use serde::{Deserialize, Serialize};

use crate::civilizational::{
    ClaimId, EvidenceFrontierId, EvidenceFrontierV1, EvidenceId, ProjectionError,
    QualificationStatus, SourceSnapshotId,
};
use crate::cultural_systems::CanonicalClaimAdmissionV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyMappingStandardV1 {
    CidocCrm,
    CrmInf,
    CrmGeo,
    ProvO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyMappingKindV1 {
    Class,
    Property,
    Individual,
}

/// A mapping from a local projection concept to an external ontology term.
///
/// The mapping is itself evidence-bearing metadata: the claim/evidence/source
/// closure identifies the context in which the mapping is admissible. It never
/// converts a mapping into a canonical historical or cultural fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyMappingV1 {
    pub mapping_id: String,
    pub standard: OntologyMappingStandardV1,
    pub external_term: String,
    pub kind: OntologyMappingKindV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub qualification: QualificationStatus,
    pub evidence_frontier: EvidenceFrontierId,
}

/// Lifecycle status of an external ontology release.
///
/// This is deliberately carried by the mapping rather than inferred from the
/// standard enum: external standards can publish draft and stable releases
/// concurrently, and a mapping must make that choice explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyReleaseStatusV1 {
    Draft,
    ReleaseCandidate,
    Stable,
    Official,
}

/// Version-pinned interoperability mapping. The version/status pair is part of
/// the mapping identity so an external ontology revision cannot silently change
/// the semantics of an existing projection mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyMappingV2 {
    pub mapping_id: String,
    pub standard: OntologyMappingStandardV1,
    pub standard_version: String,
    pub release_status: OntologyReleaseStatusV1,
    pub external_term: String,
    pub kind: OntologyMappingKindV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub qualification: QualificationStatus,
    pub evidence_frontier: EvidenceFrontierId,
}

impl OntologyMappingV2 {
    pub fn from_claim(
        mapping_id: impl Into<String>,
        standard: OntologyMappingStandardV1,
        standard_version: impl Into<String>,
        release_status: OntologyReleaseStatusV1,
        external_term: impl Into<String>,
        kind: OntologyMappingKindV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Self {
        Self {
            mapping_id: mapping_id.into(),
            standard,
            standard_version: standard_version.into(),
            release_status,
            external_term: external_term.into(),
            kind,
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            qualification: claim.qualification,
            evidence_frontier: claim.evidence_frontier.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.mapping_id.trim().is_empty()
            || self.standard_version.trim().is_empty()
            || self.external_term.trim().is_empty()
            || !self.claim_ref.is_valid()
            || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || !self.evidence_frontier.is_valid()
        {
            return Err(ProjectionError::EmptyIdentifier);
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
    }
}

impl OntologyMappingV1 {
    pub fn from_claim(
        mapping_id: impl Into<String>,
        standard: OntologyMappingStandardV1,
        external_term: impl Into<String>,
        kind: OntologyMappingKindV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Self {
        Self {
            mapping_id: mapping_id.into(),
            standard,
            external_term: external_term.into(),
            kind,
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            qualification: claim.qualification,
            evidence_frontier: claim.evidence_frontier.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.mapping_id.trim().is_empty() || self.external_term.trim().is_empty()
            || !self.claim_ref.is_valid() || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || !self.evidence_frontier.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1, YearInterval};

    fn claim() -> (CanonicalClaimAdmissionV1, EvidenceFrontierV1) {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(), known_by_year: 1950,
            parent_frontier: None, policy_version: "v1".into(), manifest_hash: String::new(),
            admitted_evidence: ["e:1"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(), source_snapshot: "source:1".into(),
                artifact_time: Some(1940), publication_time: Some(1941), capture_time: None,
                available_by: 1942, validity_time: Some(YearInterval { from: Some(1940), to: Some(1950) }),
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(), publication_time: Some(1941), capture_time: None, available_by: 1942,
            }],
        };
        frontier.recompute_manifest_hash().expect("fixture hash");
        let c=CanonicalClaimAdmissionV1 { claim_ref:"claim:1".into(), evidence_refs:vec!["e:1".into()], source_snapshots:vec!["source:1".into()], qualification:QualificationStatus::Supported, evidence_frontier:"frontier:1950".into() };
        (c,frontier)
    }

    #[test]
    fn mapping_preserves_claim_qualification() {
        let (claim, frontier)=claim();
        let mapping=OntologyMappingV1::from_claim("mapping:1", OntologyMappingStandardV1::CidocCrm, "E7_Activity", OntologyMappingKindV1::Class, &claim);
        assert!(mapping.is_frontier_safe(&claim,&frontier));
        assert_eq!(mapping.qualification, QualificationStatus::Supported);
    }

    #[test]
    fn mapping_cannot_upgrade_qualification() {
        let (mut claim, frontier)=claim();
        claim.qualification=QualificationStatus::Speculative;
        let mut mapping=OntologyMappingV1::from_claim("mapping:1", OntologyMappingStandardV1::CrmInf, "I4_Proposition_Set", OntologyMappingKindV1::Class, &claim);
        mapping.qualification=QualificationStatus::Established;
        assert!(!mapping.is_frontier_safe(&claim,&frontier));
    }
    #[test]
    fn version_pinned_mapping_requires_explicit_release_metadata() {
        let (claim, frontier) = claim();
        let mapping = OntologyMappingV2::from_claim(
            "mapping:2",
            OntologyMappingStandardV1::CidocCrm,
            "7.4",
            OntologyReleaseStatusV1::Draft,
            "E7_Activity",
            OntologyMappingKindV1::Class,
            &claim,
        );
        assert!(mapping.is_frontier_safe(&claim, &frontier));
        assert_eq!(mapping.standard_version, "7.4");
        assert_eq!(mapping.release_status, OntologyReleaseStatusV1::Draft);
    }

    #[test]
    fn changing_external_standard_version_is_detectable() {
        let (claim, frontier) = claim();
        let mut mapping = OntologyMappingV2::from_claim(
            "mapping:2",
            OntologyMappingStandardV1::CidocCrm,
            "7.4",
            OntologyReleaseStatusV1::Draft,
            "E7_Activity",
            OntologyMappingKindV1::Class,
            &claim,
        );
        assert!(mapping.is_frontier_safe(&claim, &frontier));
        mapping.standard_version = "7.5".into();
        assert!(mapping.validate().is_ok());
        assert_ne!(mapping.standard_version, "7.4");
    }

    #[test]
    fn mapping_release_status_is_explicit_for_draft_targets() {
        let (claim, frontier) = claim();
        let mapping = OntologyMappingV2::from_claim(
            "mapping:2",
            OntologyMappingStandardV1::CrmInf,
            "1.2.1",
            OntologyReleaseStatusV1::Stable,
            "I1_Argumentation",
            OntologyMappingKindV1::Class,
            &claim,
        );
        assert!(mapping.is_frontier_safe(&claim, &frontier));
        assert_eq!(mapping.release_status, OntologyReleaseStatusV1::Stable);
    }

}
