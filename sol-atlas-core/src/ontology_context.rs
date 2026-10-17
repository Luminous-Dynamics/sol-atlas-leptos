// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Content-addressed semantic context for ontology interoperability.
//!
//! A projection can remain historically/evidentially valid while its external
//! vocabulary evolves. This module makes the vocabulary context itself
//! reproducible: the exact standard/version/release status/term/kind and mapping
//! semantics are bound to a deterministic digest.
//!
//! The context is interoperability metadata, not a new claim authority.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::civilizational::{ProjectionError, QualificationStatus};
use crate::ontology_mapping::{
    OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2, OntologyReleaseStatusV1,
};

/// Semantic relationship between a local concept and an external ontology term.
///
/// This is intentionally distinct from whether the external term is a class,
/// property, or individual: a class mapping can be broad or narrow, for example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyMappingRelationV1 {
    Exact,
    Broad,
    Narrow,
    Related,
    Contextual,
}

/// Immutable semantic context captured when a projection is rendered.
///
/// mapping_hash covers all semantic mapping fields plus the relation. It is
/// therefore stronger than merely recording an external ontology version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyMappingContextV1 {
    pub mapping_id: String,
    pub standard: OntologyMappingStandardV1,
    pub standard_version: String,
    pub release_status: OntologyReleaseStatusV1,
    pub external_term: String,
    pub kind: OntologyMappingKindV1,
    pub relation: OntologyMappingRelationV1,
    pub qualification: QualificationStatus,
    pub mapping_hash: String,
}

impl OntologyMappingContextV1 {
    pub fn from_mapping(
        mapping: &OntologyMappingV2,
        relation: OntologyMappingRelationV1,
    ) -> Result<Self, ProjectionError> {
        mapping.validate()?;
        let mut context = Self {
            mapping_id: mapping.mapping_id.clone(),
            standard: mapping.standard,
            standard_version: mapping.standard_version.clone(),
            release_status: mapping.release_status,
            external_term: mapping.external_term.clone(),
            kind: mapping.kind,
            relation,
            qualification: mapping.qualification,
            mapping_hash: String::new(),
        };
        context.recompute_hash()?;
        Ok(context)
    }

    fn canonical_payload(
        &self,
    ) -> (
        &String,
        &OntologyMappingStandardV1,
        &String,
        &OntologyReleaseStatusV1,
        &String,
        &OntologyMappingKindV1,
        &OntologyMappingRelationV1,
        &QualificationStatus,
    ) {
        (
            &self.mapping_id,
            &self.standard,
            &self.standard_version,
            &self.release_status,
            &self.external_term,
            &self.kind,
            &self.relation,
            &self.qualification,
        )
    }

    pub fn computed_hash(&self) -> Result<String, ProjectionError> {
        let bytes = serde_json::to_vec(&self.canonical_payload())
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn recompute_hash(&mut self) -> Result<(), ProjectionError> {
        self.mapping_hash = self.computed_hash()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.mapping_id.trim().is_empty()
            || self.standard_version.trim().is_empty()
            || self.external_term.trim().is_empty()
            || self.mapping_hash.trim().is_empty()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.mapping_hash != self.computed_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    pub fn preserves_mapping(&self, mapping: &OntologyMappingV2) -> bool {
        self.validate().is_ok()
            && mapping.validate().is_ok()
            && self.mapping_id == mapping.mapping_id
            && self.standard == mapping.standard
            && self.standard_version == mapping.standard_version
            && self.release_status == mapping.release_status
            && self.external_term == mapping.external_term
            && self.kind == mapping.kind
            && self.qualification == mapping.qualification
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
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyReleaseStatusV1,
    };

    fn fixture() -> (OntologyMappingV2, EvidenceFrontierV1) {
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
        (
            OntologyMappingV2::from_claim(
                "mapping:1",
                OntologyMappingStandardV1::CidocCrm,
                "7.4",
                OntologyReleaseStatusV1::Draft,
                "E7_Activity",
                OntologyMappingKindV1::Class,
                &claim,
            ),
            frontier,
        )
    }

    #[test]
    fn mapping_context_is_content_addressed() {
        let (mapping, _) = fixture();
        let context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
                .expect("context");
        assert!(context.validate().is_ok());
        assert!(context.preserves_mapping(&mapping));
    }

    #[test]
    fn changing_term_or_relation_invalidates_digest() {
        let (mapping, _) = fixture();
        let mut context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
                .expect("context");
        context.external_term = "E8_Acquisition".into();
        assert!(context.validate().is_err());

        let mut context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
                .expect("context");
        context.relation = OntologyMappingRelationV1::Broad;
        assert!(context.validate().is_err());
    }

    #[test]
    fn version_and_release_status_are_integrity_bound() {
        let (mapping, _) = fixture();
        let mut context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
                .expect("context");
        context.standard_version = "7.5".into();
        assert!(context.validate().is_err());

        let mut context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
                .expect("context");
        context.release_status = OntologyReleaseStatusV1::Stable;
        assert!(context.validate().is_err());
    }

    #[test]
    fn mapping_context_preserves_qualification() {
        let (mut mapping, _) = fixture();
        mapping.qualification = QualificationStatus::Speculative;
        let context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Contextual)
                .expect("context");
        assert_eq!(context.qualification, QualificationStatus::Speculative);
        assert!(context.preserves_mapping(&mapping));
    }
}
