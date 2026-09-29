// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-reversible cultural-systems projection contracts.
//!
//! This module deliberately models a small, composable vertical slice rather
//! than attempting to encode an entire anthropology ontology. Cultural
//! transmission is a claim about a relationship; it is never inferred from
//! similarity, co-location, graph topology, or community recognition alone.

use serde::{Deserialize, Serialize};

use crate::civilizational::{
    AssessmentId, ClaimId, EntityId, EvidenceFrontierId, EvidenceFrontierV1, EvidenceId,
    ProjectionError, QualificationStatus, SourceSnapshotId, YearInterval,
};

macro_rules! cultural_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self { Self(value.to_owned()) }
        }

        impl $name {
            pub fn is_valid(&self) -> bool { !self.0.trim().is_empty() }
        }
    };
}

cultural_id!(PracticeId);
cultural_id!(TraditionId);
cultural_id!(CommunityId);
cultural_id!(TransmissionEventId);
cultural_id!(TransformationEventId);

/// Projection-side resolution context produced by an external canonical-claim
/// adapter. Sol Atlas stores the referenced claim, evidence/source closure and
/// frontier context; it does not own or authenticate the canonical claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalClaimAdmissionV1 {
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub qualification: QualificationStatus,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CanonicalClaimAdmissionV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.claim_ref.is_valid()
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

    pub fn is_frontier_safe(&self, frontier: &EvidenceFrontierV1) -> bool {
        self.validate().is_ok()
            && frontier.validate_temporal_manifest_strict().is_ok()
            && self.evidence_frontier == frontier.frontier_id
            && self.evidence_refs.iter().all(|id| frontier.admits(id))
            && self.source_snapshots.iter().all(|id| frontier.admits_source(id))
    }
}

/// A documented mechanism by which a practice, technique, concept, or tradition
/// may have moved between social or geographic contexts. The enum describes the
/// asserted mechanism; it does not establish that the mechanism occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransmissionMode {
    Taught,
    Apprenticed,
    Translated,
    Copied,
    MigratedWith,
    Traded,
    Observed,
    Institutionalized,
}

/// A documented kind of cultural transformation. The class describes the
/// asserted relationship; it does not by itself establish why the change occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CulturalTransformationClass {
    AdaptedFrom,
    ReinterpretedFrom,
    HybridizedWith,
    LocalizedAs,
    StandardizedAs,
    RevivedFrom,
    ReplacedBy,
    SuppressedBy,
}

/// An evidence-backed cultural transformation assertion.
///
/// This deliberately shares the same evidence closure and stewardship boundary
/// as transmission, so future language, knowledge, institutional and
/// material-culture slices do not invent incompatible provenance models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalTransformationV1 {
    pub transformation_id: TransformationEventId,
    pub source: EntityId,
    pub target: EntityId,
    pub class: CulturalTransformationClass,
    pub event_time: YearInterval,
    pub context: Option<String>,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub assessment: Option<AssessmentId>,
    pub qualification: QualificationStatus,
    pub community_recognition: Vec<CommunityRecognitionV1>,
    pub access_policy: AccessPolicyV1,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalTransformationV1 {
    pub fn evidence_closure(&self) -> CulturalEvidenceClosureV1 {
        CulturalEvidenceClosureV1 {
            claim_ref: self.claim_ref.clone(),
            evidence_refs: self.evidence_refs.clone(),
            source_snapshots: self.source_snapshots.clone(),
            assessment: self.assessment.clone(),
            qualification: self.qualification,
            evidence_frontier: self.evidence_frontier.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.transformation_id.is_valid()
            || !self.source.is_valid()
            || !self.target.is_valid()
            || !self.claim_ref.is_valid()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if !self.event_time.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if self.evidence_refs.is_empty() || self.source_snapshots.is_empty() {
            return Err(ProjectionError::TransitionWithoutEvidencePath);
        }
        for recognition in &self.community_recognition {
            recognition.validate()?;
        }
        Ok(())
    }

    pub fn is_frontier_safe(
        &self,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> bool {
        self.validate().is_ok()
            && self.evidence_closure().is_frontier_safe(claim, frontier)
            && self
                .community_recognition
                .iter()
                .all(|recognition| recognition.evidence_refs.iter().all(|id| frontier.admits(id)))
    }
}

/// Community participation/recognition is an independent dimension of a
/// cultural record. It must never upgrade epistemic qualification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityRecognitionV1 {
    pub community: CommunityId,
    pub practice: PracticeId,
    pub recognition_time: Option<YearInterval>,
    pub maintained: bool,
    pub transmitted: bool,
    pub evidence_refs: Vec<EvidenceId>,
}

impl CommunityRecognitionV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.community.is_valid() || !self.practice.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.recognition_time.is_some_and(|v| !v.is_valid())
        {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if self.evidence_refs.is_empty() || self.evidence_refs.iter().any(|id| !id.is_valid()) {
            return Err(ProjectionError::SnapshotWithoutEvidencePath);
        }
        Ok(())
    }
}

/// Access/stewardship metadata is orthogonal to truth or qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessPolicyV1 {
    Public,
    CommunityRestricted,
    SacredOrRestricted,
    Sensitive,
    Embargoed,
    Unknown,
}

/// Reusable canonical-claim/evidence/source closure for cultural projections.
///
/// The canonical claim itself remains externally owned. This object is the
/// projection-side materialization of the claim's evidence path plus its
/// qualification and temporal frontier. Keeping the closure explicit lets
/// transmission, transformation, language, knowledge and institutional slices
/// share the same admission boundary without creating a second claim registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalEvidenceClosureV1 {
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub assessment: Option<AssessmentId>,
    pub qualification: QualificationStatus,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalEvidenceClosureV1 {
    pub fn from_claim(claim: &CanonicalClaimAdmissionV1, assessment: Option<AssessmentId>) -> Self {
        Self {
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            assessment,
            qualification: claim.qualification,
            evidence_frontier: claim.evidence_frontier.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.claim_ref.is_valid()
            || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
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
            && claim.validate().is_ok()
            && frontier.validate_temporal_manifest_strict().is_ok()
            && self.claim_ref == claim.claim_ref
            && self.evidence_refs == claim.evidence_refs
            && self.source_snapshots == claim.source_snapshots
            && self.qualification == claim.qualification
            && self.evidence_frontier == frontier.frontier_id
            && claim.is_frontier_safe(frontier)
    }
}

/// An evidence-backed cultural transmission assertion.
///
/// source and target are deliberately generic entity IDs: endpoints may be
/// practices, traditions, communities, institutions, places, artifacts,
/// languages, techniques, or other canonical entities. Endpoint typing belongs
/// to the canonical claim layer, not this projection contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalTransmissionV1 {
    pub transmission_id: TransmissionEventId,
    pub source: EntityId,
    pub target: EntityId,
    pub mode: TransmissionMode,
    pub event_time: YearInterval,
    pub context: Option<String>,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub assessment: Option<AssessmentId>,
    pub qualification: QualificationStatus,
    pub community_recognition: Vec<CommunityRecognitionV1>,
    pub access_policy: AccessPolicyV1,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalTransmissionV1 {
    /// Materializes the reusable claim/evidence/source closure carried by this
    /// projection. Keeping this conversion centralized prevents future cultural
    /// projection types from implementing subtly different closure semantics.
    pub fn evidence_closure(&self) -> CulturalEvidenceClosureV1 {
        CulturalEvidenceClosureV1 {
            claim_ref: self.claim_ref.clone(),
            evidence_refs: self.evidence_refs.clone(),
            source_snapshots: self.source_snapshots.clone(),
            assessment: self.assessment.clone(),
            qualification: self.qualification,
            evidence_frontier: self.evidence_frontier.clone(),
        }
    }

    /// Generic admission constructor shared by all cultural projection variants.
    pub fn from_projection(
        projection: &CulturalProjectionV1,
        frontier: &EvidenceFrontierV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Option<Self> {
        if !projection.is_frontier_safe(claim, frontier) {
            return None;
        }

        let closure = projection.evidence_closure();
        let transmission_id = match projection {
            CulturalProjectionV1::Transmission(value) => value.transmission_id.clone(),
            CulturalProjectionV1::Transformation(value) => value.transformation_id.clone(),
        };

        Some(Self {
            transmission_id,
            claim_ref: closure.claim_ref,
            evidence_refs: closure.evidence_refs,
            source_snapshots: closure.source_snapshots,
            evidence_frontier: closure.evidence_frontier,
            qualification: closure.qualification,
            access_policy: match projection {
                CulturalProjectionV1::Transmission(value) => value.access_policy,
                CulturalProjectionV1::Transformation(value) => value.access_policy,
            },
        })
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.transmission_id.is_valid()
            || !self.source.is_valid()
            || !self.target.is_valid()
            || !self.claim_ref.is_valid()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if !self.event_time.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if self.evidence_refs.is_empty() || self.source_snapshots.is_empty() {
            return Err(ProjectionError::TransitionWithoutEvidencePath);
        }
        for recognition in &self.community_recognition {
            recognition.validate()?;
        }
        Ok(())
    }

    /// Admission is intentionally stronger than structural validation:
    /// the canonical claim and its complete evidence/source closure must be
    /// represented, and every referenced object must be in the same frontier.
    pub fn is_frontier_safe(
        &self,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> bool {
        self.validate().is_ok()
            && self.evidence_closure().is_frontier_safe(claim, frontier)
            && self
                .community_recognition
                .iter()
                .all(|recognition| recognition.evidence_refs.iter().all(|id| frontier.admits(id)))
    }
}

/// Renderer-neutral union for cultural-system projections.
///
/// Keeping transmission and transformation as explicit variants prevents a
/// generic graph edge from erasing the semantic class of the asserted relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CulturalProjectionV1 {
    Transmission(CulturalTransmissionV1),
    Transformation(CulturalTransformationV1),
}

impl CulturalProjectionV1 {
    pub fn evidence_closure(&self) -> CulturalEvidenceClosureV1 {
        match self {
            Self::Transmission(value) => value.evidence_closure(),
            Self::Transformation(value) => value.evidence_closure(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        match self {
            Self::Transmission(value) => value.validate(),
            Self::Transformation(value) => value.validate(),
        }
    }

    pub fn is_frontier_safe(
        &self,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> bool {
        match self {
            Self::Transmission(value) => value.is_frontier_safe(claim, frontier),
            Self::Transformation(value) => value.is_frontier_safe(claim, frontier),
        }
    }
}

/// Projection-level "why is this visible?" record for a cultural
/// transmission. It intentionally contains no causal conclusion beyond the
/// externally resolved canonical claim reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAuditV1 {
    pub transmission_id: TransmissionEventId,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub community_recognition_evidence: Vec<EvidenceId>,
    pub assessment: Option<AssessmentId>,
    pub event_time: YearInterval,
    pub qualification: QualificationStatus,
    pub access_policy: AccessPolicyV1,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalProjectionAuditV1 {
    pub fn from_transmission(transmission: &CulturalTransmissionV1) -> Self {
        let community_recognition_evidence = transmission
            .community_recognition
            .iter()
            .flat_map(|recognition| recognition.evidence_refs.iter().cloned())
            .collect();

        Self {
            transmission_id: transmission.transmission_id.clone(),
            claim_ref: transmission.claim_ref.clone(),
            evidence_refs: transmission.evidence_refs.clone(),
            source_snapshots: transmission.source_snapshots.clone(),
            community_recognition_evidence,
            assessment: transmission.assessment.clone(),
            event_time: transmission.event_time,
            qualification: transmission.qualification,
            access_policy: transmission.access_policy,
            evidence_frontier: transmission.evidence_frontier.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.transmission_id.is_valid()
            || !self.claim_ref.is_valid()
            || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self
                .community_recognition_evidence
                .iter()
                .any(|id| !id.is_valid())
            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
            || !self.evidence_frontier.is_valid()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if !self.event_time.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        Ok(())
    }

    pub fn is_frontier_safe(&self, frontier: &EvidenceFrontierV1) -> bool {
        self.validate().is_ok()
            && frontier.validate_temporal_manifest_strict().is_ok()
            && self.evidence_frontier == frontier.frontier_id
            && self.evidence_refs.iter().all(|id| frontier.admits(id))
            && self.source_snapshots.iter().all(|id| frontier.admits_source(id))
            && self
                .community_recognition_evidence
                .iter()
                .all(|id| frontier.admits(id))
    }
}

/// Explicit projection admission. Keeping this separate prevents a renderer
/// from treating a raw cultural assertion as admitted merely because it has
/// a useful-looking graph edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAdmissionV1 {
    pub transmission_id: TransmissionEventId,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub evidence_frontier: EvidenceFrontierId,
    pub qualification: QualificationStatus,
    pub access_policy: AccessPolicyV1,
}

impl CulturalProjectionAdmissionV1 {
    /// Creates the legacy transmission-shaped admission record. New consumers
    /// should prefer the generic projection union when they need both relation
    /// classes.
    pub fn from_transmission(
        transmission: &CulturalTransmissionV1,
        frontier: &EvidenceFrontierV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Option<Self> {
        if !transmission.is_frontier_safe(claim, frontier) {
            return None;
        }

        Some(Self {
            transmission_id: transmission.transmission_id.clone(),
            claim_ref: transmission.claim_ref.clone(),
            evidence_refs: transmission.evidence_refs.clone(),
            source_snapshots: transmission.source_snapshots.clone(),
            evidence_frontier: frontier.frontier_id.clone(),
            qualification: transmission.qualification,
            access_policy: transmission.access_policy,
        })
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.transmission_id.is_valid()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceTemporalMetadataV1, SourceSnapshotTemporalMetadataV1,
    };

    fn frontier() -> EvidenceFrontierV1 {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:1", "e:2", "e:recognition"]
                .into_iter()
                .map(Into::into)
                .collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:1".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(1940),
                    publication_time: Some(1941),
                    capture_time: None,
                    available_by: 1942,
                    validity_time: Some(YearInterval { from: Some(1900), to: Some(1950) }),
                },
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:2".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(1945),
                    publication_time: Some(1946),
                    capture_time: None,
                    available_by: 1947,
                    validity_time: Some(YearInterval { from: Some(1900), to: Some(1950) }),
                },
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:recognition".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(1948),
                    publication_time: Some(1949),
                    capture_time: None,
                    available_by: 1950,
                    validity_time: Some(YearInterval { from: Some(1940), to: Some(1950) }),
                },
            ],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
            }],
        };
        frontier.recompute_manifest_hash().expect("fixture hash");
        frontier
    }

    fn canonical_claim(value: &CulturalTransmissionV1) -> CanonicalClaimAdmissionV1 {
        CanonicalClaimAdmissionV1 {
            claim_ref: value.claim_ref.clone(),
            evidence_refs: value.evidence_refs.clone(),
            source_snapshots: value.source_snapshots.clone(),
            qualification: value.qualification,
            evidence_frontier: value.evidence_frontier.clone(),
        }
    }

    fn transmission() -> CulturalTransmissionV1 {
        CulturalTransmissionV1 {
            transmission_id: "transmission:1".into(),
            source: "practice:source".into(),
            target: "practice:target".into(),
            mode: TransmissionMode::Translated,
            event_time: YearInterval { from: Some(1900), to: Some(1950) },
            context: Some("documented translation context".into()),
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1", "e:2"].into_iter().map(Into::into).collect(),
            source_snapshots: vec!["source:1".into()],
            assessment: None,
            qualification: QualificationStatus::Supported,
            community_recognition: vec![CommunityRecognitionV1 {
                community: "community:1".into(),
                practice: "practice:target".into(),
                recognition_time: Some(YearInterval { from: Some(1940), to: None }),
                maintained: true,
                transmitted: true,
                evidence_refs: vec!["e:recognition".into()],
            }],
            access_policy: AccessPolicyV1::CommunityRestricted,
            evidence_frontier: "frontier:1950".into(),
        }
    }

    fn canonical_claim_for_parts(
        value: &CulturalTransformationV1,
    ) -> CanonicalClaimAdmissionV1 {
        CanonicalClaimAdmissionV1 {
            claim_ref: value.claim_ref.clone(),
            evidence_refs: value.evidence_refs.clone(),
            source_snapshots: value.source_snapshots.clone(),
            qualification: value.qualification,
            evidence_frontier: value.evidence_frontier.clone(),
        }
    }

    #[test]
    fn transformation_uses_the_same_claim_and_frontier_boundary() {
        let frontier = frontier();
        let value = CulturalTransformationV1 {
            transformation_id: "transformation:1".into(),
            source: "practice:source".into(),
            target: "practice:target".into(),
            class: CulturalTransformationClass::LocalizedAs,
            event_time: YearInterval { from: Some(1920), to: Some(1950) },
            context: Some("documented local adaptation".into()),
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1", "e:2"].into_iter().map(Into::into).collect(),
            source_snapshots: vec!["source:1".into()],
            assessment: None,
            qualification: QualificationStatus::Supported,
            community_recognition: vec![],
            access_policy: AccessPolicyV1::Public,
            evidence_frontier: "frontier:1950".into(),
        };
        assert!(value.is_frontier_safe(&canonical_claim_for_parts(&value), &frontier));
        assert!(CulturalProjectionV1::Transformation(value).validate().is_ok());
    }

    #[test]
    fn generic_projection_admission_uses_shared_evidence_closure() {
        let frontier = frontier();
        let transmission = transmission();
        let projection = CulturalProjectionV1::Transmission(transmission.clone());
        let admission = CulturalProjectionAdmissionV1::from_projection(
            &projection,
            &frontier,
            &canonical_claim(&transmission),
        )
        .expect("frontier-safe projection");

        assert_eq!(admission.claim_ref, "claim:1".into());
        assert_eq!(admission.evidence_refs, vec!["e:1".into(), "e:2".into()]);
        assert_eq!(admission.source_snapshots, vec!["source:1".into()]);
    }

    #[test]
    fn cultural_transmission_requires_claim_and_evidence_closure() {
        let mut value = transmission();
        value.claim_ref = "".into();
        assert_eq!(value.validate(), Err(ProjectionError::EmptyIdentifier));

        value = transmission();
        value.evidence_refs.clear();
        assert_eq!(value.validate(), Err(ProjectionError::TransitionWithoutEvidencePath));
    }

    #[test]
    fn canonical_claim_resolution_is_required() {
        let frontier = frontier();
        let value = transmission();
        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:wrong".into(),
            evidence_refs: value.evidence_refs.clone(),
            source_snapshots: value.source_snapshots.clone(),
            qualification: value.qualification,
            evidence_frontier: value.evidence_frontier.clone(),
        };
        assert!(!value.is_frontier_safe(&claim, &frontier));
    }

    #[test]
    fn frontier_blocks_late_transmission_evidence() {
        let frontier = frontier();
        let mut value = transmission();
        let claim = canonical_claim(&value);
        assert!(value.is_frontier_safe(&claim, &frontier));

        value.evidence_refs.push("e:later".into());
        assert!(!value.is_frontier_safe(&claim, &frontier));
    }

    #[test]
    fn source_temporal_gate_applies_to_cultural_transmission() {
        let mut frontier = frontier();
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:1".into(),
            publication_time: Some(1951),
            capture_time: None,
            available_by: 1951,
        }];
        frontier.recompute_manifest_hash().expect("fixture hash");
        let value = transmission();
        assert!(!value.is_frontier_safe(&canonical_claim(&value), &frontier));
    }

    #[test]
    fn community_recognition_does_not_upgrade_qualification() {
        let frontier = frontier();
        let mut value = transmission();
        value.qualification = QualificationStatus::Speculative;

        let claim = canonical_claim(&value);
        let admission =
            CulturalProjectionAdmissionV1::from_transmission(&value, &frontier, &claim).unwrap();

        assert_eq!(admission.qualification, QualificationStatus::Speculative);
        assert_eq!(admission.access_policy, AccessPolicyV1::CommunityRestricted);
    }

    #[test]
    fn access_policy_is_not_an_epistemic_status() {
        let frontier = frontier();
        let mut value = transmission();
        value.access_policy = AccessPolicyV1::Public;
        let public_admission =
            CulturalProjectionAdmissionV1::from_transmission(
                &value,
                &frontier,
                &canonical_claim(&value),
            )
            .unwrap();

        value.access_policy = AccessPolicyV1::SacredOrRestricted;
        let restricted_admission = CulturalProjectionAdmissionV1::from_transmission(
            &value,
            &frontier,
            &canonical_claim(&value),
        )
        .unwrap();

        assert_eq!(public_admission.qualification, restricted_admission.qualification);
    }

    #[test]
    fn audit_preserves_the_full_reversibility_path() {
        let value = transmission();
        let audit = CulturalProjectionAuditV1::from_transmission(&value);
        assert!(audit.validate().is_ok());
        assert_eq!(audit.claim_ref, value.claim_ref);
        assert_eq!(audit.evidence_refs, value.evidence_refs);
        assert_eq!(audit.source_snapshots, value.source_snapshots);
        assert_eq!(
            audit.community_recognition_evidence,
            vec!["e:recognition".into()]
        );
        assert_eq!(audit.qualification, value.qualification);
        assert_eq!(audit.access_policy, value.access_policy);
    }

    #[test]
    fn audit_blocks_unadmitted_community_evidence() {
        let frontier = frontier();
        let value = transmission();
        let mut audit = CulturalProjectionAuditV1::from_transmission(&value);
        audit.community_recognition_evidence.push("e:late".into());
        assert!(!audit.is_frontier_safe(&frontier));
    }

    #[test]
    fn admission_is_explicit_and_reversible() {
        let frontier = frontier();
        let value = transmission();
        let admission = CulturalProjectionAdmissionV1::from_transmission(
            &value,
            &frontier,
            &canonical_claim(&value),
        )
        .unwrap();

        assert!(admission.validate().is_ok());
        assert_eq!(admission.claim_ref, "claim:1".into());
        assert_eq!(admission.transmission_id, "transmission:1".into());
        assert_eq!(admission.evidence_frontier, "frontier:1950".into());
    }
}
