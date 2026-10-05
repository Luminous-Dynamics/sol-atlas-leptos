// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reproducible semantic context for rendered projections.
//!
//! A projection is not fully replayable if the ontology vocabulary used to
//! render it is allowed to drift. This envelope records the exact mapping
//! contexts used for a projection without turning those mappings into claims.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{ClaimId, EvidenceFrontierId, ProjectionError, QualificationStatus};
use crate::cultural_systems::CulturalProjectionIdV1;
use crate::ontology_context::OntologyMappingContextV1;

fn has_duplicate_mapping_ids(mappings: &[OntologyMappingContextV1]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    mappings
        .iter()
        .any(|mapping| !seen.insert(mapping.mapping_id.clone()))
}

/// Content-addressed semantic context attached to a projection render.
///
/// Mapping order is canonicalized for deterministic replay. Ordering carries
/// no epistemic meaning and is never used to rank competing interpretations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionSemanticEnvelopeV1 {
    pub projection_id: CulturalProjectionIdV1,
    pub claim_ref: ClaimId,
    pub evidence_frontier: EvidenceFrontierId,
    pub qualification: QualificationStatus,
    pub mappings: Vec<OntologyMappingContextV1>,
    pub envelope_hash: String,
}

impl ProjectionSemanticEnvelopeV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.projection_id.is_valid()
            || !self.claim_ref.is_valid()
            || !self.evidence_frontier.is_valid()
            || self.mappings.is_empty()
            || self.envelope_hash.trim().is_empty()
            || has_duplicate_mapping_ids(&self.mappings)
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        for mapping in &self.mappings {
            mapping.validate()?;
            if mapping.qualification != self.qualification {
                return Err(ProjectionError::EmptyIdentifier);
            }
        }
        if self.envelope_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    pub fn canonicalize(&mut self) {
        self.mappings.sort_by(|a, b| {
            (a.mapping_id.clone(), a.mapping_hash.clone())
                .cmp(&(b.mapping_id.clone(), b.mapping_hash.clone()))
        });
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let mut mappings = self.mappings.clone();
        mappings.sort_by(|a, b| {
            (a.mapping_id.clone(), a.mapping_hash.clone())
                .cmp(&(b.mapping_id.clone(), b.mapping_hash.clone()))
        });
        let payload = (
            &self.projection_id,
            &self.claim_ref,
            &self.evidence_frontier,
            &self.qualification,
            &mappings,
        );
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.envelope_hash = self.computed_hash()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceFrontierV1, EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1,
        YearInterval,
    };
    use crate::cultural_systems::CanonicalClaimAdmissionV1;
    use crate::ontology_context::OntologyMappingRelationV1;
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn mapping(id: &str, term: &str) -> OntologyMappingContextV1 {
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
        frontier.recompute_manifest_hash().expect("fixture hash");
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
        };
        let mapping = OntologyMappingV2::from_claim(
            id,
            OntologyMappingStandardV1::CidocCrm,
            "7.4",
            OntologyReleaseStatusV1::Draft,
            term,
            OntologyMappingKindV1::Class,
            &claim,
        );
        OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
            .expect("mapping context")
    }

    #[test]
    fn envelope_rejects_duplicate_mapping_identity() {
        let first = mapping("mapping:1", "E7_Activity");
        let second = mapping("mapping:1", "I1_Argumentation");
        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: "frontier:1950".into(),
            qualification: QualificationStatus::Supported,
            mappings: vec![first, second],
            envelope_hash: String::new(),
        };
        envelope.recompute_hash().expect("envelope hash");
        assert_eq!(envelope.validate(), Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn envelope_rejects_invalid_projection_identity() {
        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transmission("".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: "frontier:1950".into(),
            qualification: QualificationStatus::Supported,
            mappings: vec![mapping("mapping:1", "E7_Activity")],
            envelope_hash: String::new(),
        };
        envelope.recompute_hash().expect("envelope hash");
        assert_eq!(envelope.validate(), Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn envelope_rejects_mapping_qualification_drift() {
        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: "frontier:1950".into(),
            qualification: QualificationStatus::Supported,
            mappings: vec![mapping("mapping:1", "E7_Activity")],
            envelope_hash: String::new(),
        };
        envelope.mappings[0].qualification = QualificationStatus::Speculative;
        envelope.mappings[0]
            .recompute_hash()
            .expect("rehashed mapping drift fixture");
        envelope.recompute_hash().expect("rehashed drift fixture");
        assert_eq!(envelope.validate(), Err(ProjectionError::EmptyIdentifier));
    }

    #[test]
    fn envelope_hash_binds_mapping_context() {
        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: "frontier:1950".into(),
            qualification: QualificationStatus::Supported,
            mappings: vec![mapping("mapping:1", "E7_Activity")],
            envelope_hash: String::new(),
        };
        envelope.recompute_hash().expect("envelope hash");
        assert!(envelope.validate().is_ok());

        envelope.mappings[0].external_term = "E8_Acquisition".into();
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn canonicalization_is_structural_not_epistemic() {
        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: "frontier:1950".into(),
            qualification: QualificationStatus::Supported,
            mappings: vec![
                mapping("mapping:2", "I1_Argumentation"),
                mapping("mapping:1", "E7_Activity"),
            ],
            envelope_hash: String::new(),
        };
        envelope.recompute_hash().expect("envelope hash");
        envelope.canonicalize();
        assert_eq!(envelope.mappings[0].mapping_id, "mapping:1");
        assert_eq!(envelope.mappings[1].mapping_id, "mapping:2");
        assert!(envelope.validate().is_ok());
    }

    #[test]
    fn qualification_is_preserved_in_semantic_context() {
        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transformation("transformation:1".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: "frontier:1950".into(),
            qualification: QualificationStatus::Speculative,
            mappings: vec![{
                let mut value = mapping("mapping:1", "E7_Activity");
                value.qualification = QualificationStatus::Speculative;
                value.recompute_hash().expect("rehashed mapping");
                value
            }],
            envelope_hash: String::new(),
        };
        envelope.recompute_hash().expect("envelope hash");
        assert_eq!(envelope.qualification, QualificationStatus::Speculative);
        assert!(envelope.validate().is_ok());
    }
}
