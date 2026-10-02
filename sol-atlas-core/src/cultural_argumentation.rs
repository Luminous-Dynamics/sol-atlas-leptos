// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed projection-side argumentation semantics.
//!
//! The vocabulary is aligned to CRMinf 1.2.1, but this module does not claim
//! complete CRMinf conformance or introduce a second canonical claim authority.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{
    AssessmentId, EvidenceFrontierId, EvidenceFrontierV1, InterpretationId, ProjectionError,
    YearInterval,
};
use crate::cultural_systems::{
    CanonicalClaimAdmissionV1, CulturalArgumentationEvidenceClosureV1, CulturalArgumentationRefV2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CulturalArgumentationKindV1 {
    InferenceMaking,
    BeliefAdoption,
    ProvenanceAssessment,
    MeaningComprehension,
}

/// Typed argumentation record with a content-addressed semantic envelope.
///
/// The kind is semantic identity, not a confidence score or ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalArgumentationRefV3 {
    pub kind: CulturalArgumentationKindV1,
    pub assessment: AssessmentId,
    pub interpretation: InterpretationId,
    pub claim_ref: crate::civilizational::ClaimId,
    pub closure: CulturalArgumentationEvidenceClosureV1,
    pub assessment_time: Option<YearInterval>,
    pub interpretation_time: Option<YearInterval>,
    pub available_by: i32,
    pub evidence_frontier: EvidenceFrontierId,
    pub semantic_hash: String,
}

impl CulturalArgumentationRefV3 {
    pub fn from_v2(
        kind: CulturalArgumentationKindV1,
        value: CulturalArgumentationRefV2,
        frontier: &EvidenceFrontierV1,
    ) -> Result<Self, ProjectionError> {
        value.validate()?;
        if value.closure.evidence_frontier != frontier.frontier_id {
            return Err(ProjectionError::EmptyIdentifier);
        }
        let mut result = Self {
            kind,
            assessment: value.assessment,
            interpretation: value.interpretation,
            claim_ref: value.claim_ref,
            closure: value.closure,
            assessment_time: value.assessment_time,
            interpretation_time: value.interpretation_time,
            available_by: value.available_by,
            evidence_frontier: frontier.frontier_id.clone(),
            semantic_hash: String::new(),
        };
        result.recompute_hash()?;
        result.validate()?;
        Ok(result)
    }

    fn canonical_payload(
        &self,
    ) -> (
        &CulturalArgumentationKindV1,
        &AssessmentId,
        &InterpretationId,
        &crate::civilizational::ClaimId,
        &CulturalArgumentationEvidenceClosureV1,
        &Option<YearInterval>,
        &Option<YearInterval>,
        &i32,
        &EvidenceFrontierId,
    ) {
        (
            &self.kind,
            &self.assessment,
            &self.interpretation,
            &self.claim_ref,
            &self.closure,
            &self.assessment_time,
            &self.interpretation_time,
            &self.available_by,
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
        self.semantic_hash = self.computed_hash()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.assessment.is_valid()
            || !self.interpretation.is_valid()
            || !self.claim_ref.is_valid()
            || !self.evidence_frontier.is_valid()
            || self.semantic_hash.trim().is_empty()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        self.closure.validate()?;
        if self.closure.claim_ref != self.claim_ref
            || self.closure.evidence_frontier != self.evidence_frontier
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.assessment_time.is_some_and(|v| !v.is_valid())
            || self.interpretation_time.is_some_and(|v| !v.is_valid())
            || self.available_by
                < self
                    .assessment_time
                    .and_then(|v| v.to)
                    .or(self.assessment_time.and_then(|v| v.from))
                    .unwrap_or(self.available_by)
            || self.available_by
                < self
                    .interpretation_time
                    .and_then(|v| v.to)
                    .or(self.interpretation_time.and_then(|v| v.from))
                    .unwrap_or(self.available_by)
        {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if self.semantic_hash != self.computed_hash()? {
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
            && self.claim_ref == claim.claim_ref
            && self.closure.is_frontier_safe(claim, frontier)
            && self.evidence_frontier == frontier.frontier_id
            && self.available_by <= frontier.known_by_year
            && frontier.admits_argumentation(&self.assessment, &self.interpretation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1};
    use crate::cultural_systems::CulturalArgumentationRefV2;

    #[test]
    fn claim_binding_is_not_inherited_from_pair_level_frontier_admission() {
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
                artifact_time: None,
                publication_time: None,
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: None,
                capture_time: None,
                available_by: 1940,
            }],
            argumentation_metadata: vec![crate::civilizational::ArgumentationTemporalMetadataV1 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                assessment_time: None,
                interpretation_time: None,
                available_by: 1940,
            }],
        };
        frontier.recompute_manifest_hash().expect("hash");

        let claim_one = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: crate::civilizational::QualificationStatus::Supported,
            evidence_frontier: frontier.frontier_id.clone(),
        };
        let claim_two = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:2".into(),
            ..claim_one.clone()
        };
        let value = CulturalArgumentationRefV2 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: claim_one.claim_ref.clone(),
            closure: CulturalArgumentationEvidenceClosureV1 {
                claim_ref: claim_one.claim_ref.clone(),
                evidence_refs: claim_one.evidence_refs.clone(),
                source_snapshots: claim_one.source_snapshots.clone(),
                evidence_frontier: frontier.frontier_id.clone(),
            },
            assessment_time: None,
            interpretation_time: None,
            available_by: 1940,
        };
        let argument = CulturalArgumentationRefV3::from_v2(
            CulturalArgumentationKindV1::InferenceMaking,
            value,
            &frontier,
        )
        .expect("argumentation");

        assert!(frontier.admits_argumentation(&argument.assessment, &argument.interpretation,));
        assert!(argument.is_frontier_safe(&claim_one, &frontier));
        assert!(!argument.is_frontier_safe(&claim_two, &frontier));
    }

    #[test]
    fn kind_is_part_of_semantic_identity() {
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
                artifact_time: None,
                publication_time: None,
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: None,
                capture_time: None,
                available_by: 1940,
            }],
            argumentation_metadata: vec![],
        };
        frontier.recompute_manifest_hash().expect("hash");
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: crate::civilizational::QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
        };
        let value = CulturalArgumentationRefV2 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: claim.claim_ref.clone(),
            closure: CulturalArgumentationEvidenceClosureV1 {
                claim_ref: claim.claim_ref.clone(),
                evidence_refs: claim.evidence_refs.clone(),
                source_snapshots: claim.source_snapshots.clone(),
                evidence_frontier: frontier.frontier_id.clone(),
            },
            assessment_time: None,
            interpretation_time: None,
            available_by: 1940,
        };
        frontier.argumentation_metadata.push(
            crate::civilizational::ArgumentationTemporalMetadataV1 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                assessment_time: None,
                interpretation_time: None,
                available_by: 1940,
            },
        );
        frontier.recompute_manifest_hash().expect("rehash");
        let mut a = CulturalArgumentationRefV3::from_v2(
            CulturalArgumentationKindV1::InferenceMaking,
            value.clone(),
            &frontier,
        )
        .expect("v3");
        let original = a.semantic_hash.clone();
        a.kind = CulturalArgumentationKindV1::MeaningComprehension;
        assert_ne!(original, a.computed_hash().expect("hash"));
        assert!(a.validate().is_err());
    }
}
