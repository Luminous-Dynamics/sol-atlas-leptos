// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Additive, semantic-aware cultural projection audit.
//!
//! V3 preserves the complete V2 evidence/argumentation audit while pinning the
//! ontology mapping context used for rendering. This closes the final semantic
//! replay boundary without changing the canonical claim authority.

use serde::{Deserialize, Serialize};

use crate::civilizational::ProjectionError;
use crate::cultural_systems::{
    CulturalArgumentationRefV1, CulturalProjectionAuditV2, CulturalProjectionIdV1,
};
use crate::projection_semantics::ProjectionSemanticEnvelopeV1;

/// Reproducible "why was this rendered?" audit envelope.
///
/// The semantic envelope is validated against the same projection, claim,
/// frontier, and qualification as the underlying V2 audit. It cannot upgrade
/// qualification or substitute for the canonical evidence closure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAuditV3 {
    pub base: CulturalProjectionAuditV2,
    pub semantic_context: ProjectionSemanticEnvelopeV1,
}

impl CulturalProjectionAuditV3 {
    pub fn from_v2(
        base: CulturalProjectionAuditV2,
        semantic_context: ProjectionSemanticEnvelopeV1,
    ) -> Result<Self, ProjectionError> {
        let audit = Self {
            base,
            semantic_context,
        };
        audit.validate()?;
        Ok(audit)
    }

    /// Attaches an already-resolved semantic envelope to a V2 audit.
    pub fn from_v2_with_semantic_context(
        base: CulturalProjectionAuditV2,
        semantic_context: ProjectionSemanticEnvelopeV1,
    ) -> Result<Self, ProjectionError> {
        Self::from_v2(base, semantic_context)
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        self.base.validate()?;
        self.semantic_context.validate()?;

        if self.semantic_context.projection_id != self.base.projection_id
            || self.semantic_context.claim_ref != self.base.claim_ref
            || self.semantic_context.evidence_frontier != self.base.evidence_frontier
            || self.semantic_context.qualification != self.base.qualification
        {
            return Err(ProjectionError::EmptyIdentifier);
        }

        if self
            .semantic_context
            .mappings
            .iter()
            .any(|mapping| mapping.qualification != self.base.qualification)
        {
            return Err(ProjectionError::EmptyIdentifier);
        }

        Ok(())
    }

    pub fn is_frontier_safe(&self, frontier: &crate::civilizational::EvidenceFrontierV1) -> bool {
        self.validate().is_ok()
            && self.base.is_frontier_safe(frontier)
            && self.semantic_context.evidence_frontier == frontier.frontier_id
    }

    /// Reciprocal validation against the exact projection, canonical claim,
    /// and frontier that produced this semantic audit.
    pub fn validate_against_projection(
        &self,
        projection: &crate::cultural_systems::CulturalProjectionV1,
        frontier: &crate::civilizational::EvidenceFrontierV1,
        claim: &crate::cultural_systems::CanonicalClaimAdmissionV1,
    ) -> Result<(), ProjectionError> {
        self.validate_against_projection_and_semantic_context(
            projection,
            frontier,
            claim,
            &self.semantic_context,
        )
    }

    /// Reciprocal validation against the exact semantic context used by the
    /// caller. The context remains externally owned, so source binding must be
    /// supplied explicitly rather than inferred from its own content hash.
    pub fn validate_against_projection_and_semantic_context(
        &self,
        projection: &crate::cultural_systems::CulturalProjectionV1,
        frontier: &crate::civilizational::EvidenceFrontierV1,
        claim: &crate::cultural_systems::CanonicalClaimAdmissionV1,
        expected_context: &ProjectionSemanticEnvelopeV1,
    ) -> Result<(), ProjectionError> {
        self.validate()?;
        self.base
            .validate_against_projection(projection, frontier, claim)?;

        if self.semantic_context != *expected_context
            || self.semantic_context.projection_id != self.base.projection_id
            || self.semantic_context.claim_ref != self.base.claim_ref
            || self.semantic_context.evidence_frontier != frontier.frontier_id
            || self.semantic_context.qualification != self.base.qualification
        {
            return Err(ProjectionError::AuditWithoutEvidencePath);
        }

        Ok(())
    }

    pub fn argumentation(&self) -> Option<&CulturalArgumentationRefV1> {
        self.base.argumentation.as_ref()
    }

    pub fn projection_id(&self) -> &CulturalProjectionIdV1 {
        &self.base.projection_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceFrontierV1, EvidenceTemporalMetadataV1, QualificationStatus,
        SourceSnapshotTemporalMetadataV1, YearInterval,
    };
    use crate::cultural_systems::{
        CanonicalClaimAdmissionV1, CulturalProjectionV1, CulturalTransmissionV1,
    };
    use crate::ontology_context::{OntologyMappingContextV1, OntologyMappingRelationV1};
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn frontier() -> EvidenceFrontierV1 {
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
        frontier
    }

    fn audit() -> CulturalProjectionAuditV2 {
        let projection = CulturalProjectionV1::Transmission(CulturalTransmissionV1 {
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
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            assessment: None,
            qualification: QualificationStatus::Supported,
            community_recognition: vec![],
            access_policy: crate::cultural_systems::AccessPolicyV1::Public,
            evidence_frontier: "frontier:1950".into(),
        });
        let frontier = frontier();
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
        };
        CulturalProjectionAuditV2::from_projection_at(&projection, &frontier, &claim)
            .expect("frontier-safe audit")
    }

    fn semantic_context() -> ProjectionSemanticEnvelopeV1 {
        let frontier = frontier();
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
        let context =
            OntologyMappingContextV1::from_mapping(&mapping, OntologyMappingRelationV1::Exact)
                .expect("mapping context");

        let mut envelope = ProjectionSemanticEnvelopeV1 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: "claim:1".into(),
            evidence_frontier: frontier.frontier_id,
            qualification: QualificationStatus::Supported,
            mappings: vec![context],
            envelope_hash: String::new(),
        };
        envelope.recompute_hash().expect("semantic hash");
        envelope
    }

    #[test]
    fn v3_audit_binds_semantic_context_to_v2_identity() {
        let value =
            CulturalProjectionAuditV3::from_v2(audit(), semantic_context()).expect("v3 audit");
        assert!(value.validate().is_ok());
        assert!(value.is_frontier_safe(&frontier()));
    }

    #[test]
    fn v3_audit_reciprocal_validation_rejects_projection_rebinding() {
        let frontier = frontier();
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
                evidence_refs: vec!["e:1".into()],
                source_snapshots: vec!["source:1".into()],
                assessment: None,
                qualification: QualificationStatus::Supported,
                community_recognition: vec![],
                access_policy: crate::cultural_systems::AccessPolicyV1::Public,
                evidence_frontier: "frontier:1950".into(),
            },
        );
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
        };
        let value =
            CulturalProjectionAuditV3::from_v2(audit(), semantic_context()).expect("v3 audit");

        let mut tampered = value;
        tampered.base.access_policy = crate::cultural_systems::AccessPolicyV1::Sensitive;
        assert_eq!(
            tampered.validate_against_projection(&projection, &frontier, &claim),
            Err(ProjectionError::AuditWithoutEvidencePath)
        );
    }

    #[test]
    fn semantic_context_cannot_change_claim_or_frontier() {
        let mut context = semantic_context();
        context.claim_ref = "claim:other".into();
        context.recompute_hash().expect("rehashed tamper fixture");
        assert!(CulturalProjectionAuditV3::from_v2(audit(), context).is_err());

        let mut context = semantic_context();
        context.evidence_frontier = "frontier:1960".into();
        context.recompute_hash().expect("rehashed tamper fixture");
        assert!(CulturalProjectionAuditV3::from_v2(audit(), context).is_err());
    }

    #[test]
    fn semantic_mapping_cannot_upgrade_qualification() {
        let mut context = semantic_context();
        context.mappings[0].qualification = QualificationStatus::Established;
        context.mappings[0]
            .recompute_hash()
            .expect("mapping rehash");
        context.recompute_hash().expect("envelope rehash");
        assert!(CulturalProjectionAuditV3::from_v2(audit(), context).is_err());
    }

    #[test]
    fn v3_reciprocal_validation_rejects_rebound_semantic_context() {
        let audit = CulturalProjectionAuditV3::from_v2(audit(), semantic_context())
            .expect("v3 audit");
        let frontier = frontier();
        let projection = CulturalProjectionV1::Transmission(CulturalTransmissionV1 {
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
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            assessment: None,
            qualification: QualificationStatus::Supported,
            community_recognition: vec![],
            access_policy: crate::cultural_systems::AccessPolicyV1::Public,
            evidence_frontier: "frontier:1950".into(),
        });
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: "frontier:1950".into(),
        };

        let mut rebound_context = audit.semantic_context.clone();
        rebound_context.mappings[0].external_term = "E8_Acquisition".into();
        rebound_context.mappings[0]
            .recompute_hash()
            .expect("rebound mapping hash");
        rebound_context.recompute_hash().expect("rebound envelope hash");

        assert_eq!(
            audit.validate_against_projection_and_semantic_context(
                &projection,
                &frontier,
                &claim,
                &rebound_context,
            ),
            Err(ProjectionError::AuditWithoutEvidencePath)
        );
    }

    #[test]
    fn semantic_mapping_tamper_invalidates_audit() {
        let mut context = semantic_context();
        context.mappings[0].external_term = "E8_Acquisition".into();
        assert!(CulturalProjectionAuditV3::from_v2(audit(), context).is_err());
    }
}
