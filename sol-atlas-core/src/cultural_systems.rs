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
    InterpretationId, ProjectionError, QualificationStatus, SourceSnapshotId, YearInterval,
};

macro_rules! cultural_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl $name {
            pub fn is_valid(&self) -> bool {
                !self.0.trim().is_empty()
            }
        }
    };
}

cultural_id!(PracticeId);
cultural_id!(TraditionId);
cultural_id!(CommunityId);
cultural_id!(TransmissionEventId);
cultural_id!(TransformationEventId);

fn has_duplicate_ids(ids: &[EvidenceId]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    ids.iter().any(|id| !seen.insert(id))
}

fn has_duplicate_source_snapshots(ids: &[SourceSnapshotId]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    ids.iter().any(|id| !seen.insert(id))
}

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
            || has_duplicate_ids(&self.evidence_refs)
            || has_duplicate_source_snapshots(&self.source_snapshots)
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
            && self
                .source_snapshots
                .iter()
                .all(|id| frontier.admits_source(id))
    }
}

/// Projection-side argumentation context. The referenced assessment and
/// interpretation are externally owned; Sol Atlas only carries stable references
/// and the evidence closure needed to replay a projection safely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalArgumentationRefV1 {
    pub assessment: AssessmentId,
    pub interpretation: InterpretationId,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    /// When the assessment was performed; distinct from historical event time.
    pub assessment_time: Option<YearInterval>,
    /// When the interpretation was formulated; distinct from publication/availability.
    pub interpretation_time: Option<YearInterval>,
    /// Earliest epistemic frontier year at which this argumentation record may be used.
    pub available_by: i32,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalArgumentationRefV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.assessment.is_valid()
            || !self.interpretation.is_valid()
            || !self.claim_ref.is_valid()
            || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || !self.evidence_frontier.is_valid()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.available_by
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
            || self.assessment_time.is_some_and(|v| !v.is_valid())
            || self.interpretation_time.is_some_and(|v| !v.is_valid())
        {
            return Err(ProjectionError::InvalidTimeInterval);
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
            && self.evidence_frontier == frontier.frontier_id
            && self.available_by <= frontier.known_by_year
            && frontier.admits_argumentation(&self.assessment, &self.interpretation)
    }
}

/// Argumentation evidence closure deliberately independent from the canonical claim closure.
/// An assessment/interpretation may rely on a subset or an argument-specific superset of
/// evidence while remaining explicitly bound to the same canonical claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalArgumentationEvidenceClosureV1 {
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalArgumentationEvidenceClosureV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.claim_ref.is_valid()
            || self.evidence_refs.is_empty()
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.is_empty()
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || !self.evidence_frontier.is_valid()
            || has_duplicate_ids(&self.evidence_refs)
            || has_duplicate_source_snapshots(&self.source_snapshots)
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
            && self.evidence_frontier == frontier.frontier_id
            && self.evidence_refs.iter().all(|id| frontier.admits(id))
            && self
                .source_snapshots
                .iter()
                .all(|id| frontier.admits_source(id))
    }
}

/// Versioned argumentation reference with an independent evidence/source closure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalArgumentationRefV2 {
    pub assessment: AssessmentId,
    pub interpretation: InterpretationId,
    pub claim_ref: ClaimId,
    pub closure: CulturalArgumentationEvidenceClosureV1,
    pub assessment_time: Option<YearInterval>,
    pub interpretation_time: Option<YearInterval>,
    pub available_by: i32,
}

impl CulturalArgumentationRefV2 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.assessment.is_valid()
            || !self.interpretation.is_valid()
            || !self.claim_ref.is_valid()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        self.closure.validate()?;
        if self.closure.claim_ref != self.claim_ref {
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
        Ok(())
    }

    pub fn is_frontier_safe(
        &self,
        claim: &CanonicalClaimAdmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> bool {
        self.validate().is_ok()
            && self.closure.is_frontier_safe(claim, frontier)
            && self.available_by <= frontier.known_by_year
            && frontier.admits_argumentation(&self.assessment, &self.interpretation)
    }
}

/// A replay set preserves contemporaneous argumentation alternatives without
/// assigning an epistemic winner. Ordering is canonical serialization structure only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalArgumentationSetV1 {
    pub claim_ref: ClaimId,
    pub evidence_frontier: EvidenceFrontierId,
    pub alternatives: Vec<CulturalArgumentationRefV2>,
}

impl CulturalArgumentationSetV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.claim_ref.is_valid()
            || !self.evidence_frontier.is_valid()
            || self.alternatives.is_empty()
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        let mut identities = std::collections::BTreeSet::new();
        for argumentation in &self.alternatives {
            argumentation.validate()?;
            if argumentation.claim_ref != self.claim_ref
                || argumentation.closure.evidence_frontier != self.evidence_frontier
                || !identities.insert((
                    argumentation.assessment.clone(),
                    argumentation.interpretation.clone(),
                ))
            {
                return Err(ProjectionError::EmptyIdentifier);
            }
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
            && self.evidence_frontier == frontier.frontier_id
            && self
                .alternatives
                .iter()
                .all(|argumentation| argumentation.is_frontier_safe(claim, frontier))
    }

    /// Canonicalizes storage order by argument identity. This is not an epistemic ranking.
    pub fn canonicalize(&mut self) {
        self.alternatives.sort_by(|a, b| {
            (a.assessment.clone(), a.interpretation.clone())
                .cmp(&(b.assessment.clone(), b.interpretation.clone()))
        });
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
            && self.community_recognition.iter().all(|recognition| {
                recognition
                    .evidence_refs
                    .iter()
                    .all(|id| frontier.admits(id))
            })
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
        if self.recognition_time.is_some_and(|v| !v.is_valid()) {
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

    /// Returns the reusable claim/evidence/source closure for either cultural
    /// projection variant without coercing its semantic identity.
    pub fn projection_closure(projection: &CulturalProjectionV1) -> CulturalEvidenceClosureV1 {
        projection.evidence_closure()
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
            && self.community_recognition.iter().all(|recognition| {
                recognition
                    .evidence_refs
                    .iter()
                    .all(|id| frontier.admits(id))
            })
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

/// Stable identity of an admitted cultural projection without erasing its
/// semantic relation class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CulturalProjectionIdV1 {
    Transmission(TransmissionEventId),
    Transformation(TransformationEventId),
}

impl CulturalProjectionIdV1 {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Transmission(id) => id.is_valid(),
            Self::Transformation(id) => id.is_valid(),
        }
    }
}

/// Generic admission record for either transmission or transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAdmissionV2 {
    pub projection_id: CulturalProjectionIdV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub evidence_frontier: EvidenceFrontierId,
    pub qualification: QualificationStatus,
    pub access_policy: AccessPolicyV1,
}

impl CulturalProjectionAdmissionV2 {
    pub fn from_projection(
        projection: &CulturalProjectionV1,
        frontier: &EvidenceFrontierV1,
        claim: &CanonicalClaimAdmissionV1,
    ) -> Option<Self> {
        if !projection.is_frontier_safe(claim, frontier) {
            return None;
        }
        let closure = projection.evidence_closure();
        let (projection_id, access_policy) = match projection {
            CulturalProjectionV1::Transmission(v) => (
                CulturalProjectionIdV1::Transmission(v.transmission_id.clone()),
                v.access_policy,
            ),
            CulturalProjectionV1::Transformation(v) => (
                CulturalProjectionIdV1::Transformation(v.transformation_id.clone()),
                v.access_policy,
            ),
        };
        Some(Self {
            projection_id,
            claim_ref: closure.claim_ref,
            evidence_refs: closure.evidence_refs,
            source_snapshots: closure.source_snapshots,
            evidence_frontier: closure.evidence_frontier,
            qualification: closure.qualification,
            access_policy,
        })
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.projection_id.is_valid()
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

/// Generic "why is this visible?" audit for either cultural projection variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalProjectionAuditV2 {
    pub projection_id: CulturalProjectionIdV1,
    pub claim_ref: ClaimId,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub community_recognition_evidence: Vec<EvidenceId>,
    pub assessment: Option<AssessmentId>,
    /// Optional argumentation closure; when present it preserves the assessment/
    /// interpretation provenance used to justify the projection's qualification.
    pub argumentation: Option<CulturalArgumentationRefV1>,
    pub event_time: YearInterval,
    pub qualification: QualificationStatus,
    pub access_policy: AccessPolicyV1,
    pub evidence_frontier: EvidenceFrontierId,
}

impl CulturalProjectionAuditV2 {
    pub fn from_projection(projection: &CulturalProjectionV1) -> Self {
        let (
            projection_id,
            recognition,
            assessment,
            event_time,
            claim_ref,
            evidence_refs,
            source_snapshots,
            qualification,
            access_policy,
            evidence_frontier,
        ) = match projection {
            CulturalProjectionV1::Transmission(v) => (
                CulturalProjectionIdV1::Transmission(v.transmission_id.clone()),
                &v.community_recognition,
                v.assessment.clone(),
                v.event_time,
                v.claim_ref.clone(),
                v.evidence_refs.clone(),
                v.source_snapshots.clone(),
                v.qualification,
                v.access_policy,
                v.evidence_frontier.clone(),
            ),
            CulturalProjectionV1::Transformation(v) => (
                CulturalProjectionIdV1::Transformation(v.transformation_id.clone()),
                &v.community_recognition,
                v.assessment.clone(),
                v.event_time,
                v.claim_ref.clone(),
                v.evidence_refs.clone(),
                v.source_snapshots.clone(),
                v.qualification,
                v.access_policy,
                v.evidence_frontier.clone(),
            ),
        };
        Self {
            projection_id,
            claim_ref,
            evidence_refs,
            source_snapshots,
            community_recognition_evidence: recognition
                .iter()
                .flat_map(|r| r.evidence_refs.iter().cloned())
                .collect(),
            assessment,
            argumentation: None,
            event_time,
            qualification,
            access_policy,
            evidence_frontier,
        }
    }

    /// Attaches an externally resolved argumentation record without changing the
    /// semantic identity or qualification of the projection.
    pub fn with_argumentation(mut self, argumentation: CulturalArgumentationRefV1) -> Self {
        self.argumentation = Some(argumentation);
        self
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.projection_id.is_valid()
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
        if let Some(argumentation) = &self.argumentation {
            argumentation.validate()?;
            if argumentation.claim_ref != self.claim_ref
                || argumentation.evidence_refs != self.evidence_refs
                || argumentation.source_snapshots != self.source_snapshots
                || argumentation.evidence_frontier != self.evidence_frontier
                || self.assessment.as_ref() != Some(&argumentation.assessment)
            {
                return Err(ProjectionError::EmptyIdentifier);
            }
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
            && self
                .source_snapshots
                .iter()
                .all(|id| frontier.admits_source(id))
            && self
                .community_recognition_evidence
                .iter()
                .all(|id| frontier.admits(id))
            && self.argumentation.as_ref().is_none_or(|argumentation| {
                argumentation.is_frontier_safe(
                    &CanonicalClaimAdmissionV1 {
                        claim_ref: self.claim_ref.clone(),
                        evidence_refs: self.evidence_refs.clone(),
                        source_snapshots: self.source_snapshots.clone(),
                        qualification: self.qualification,
                        evidence_frontier: self.evidence_frontier.clone(),
                    },
                    frontier,
                )
            })
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
            && self
                .source_snapshots
                .iter()
                .all(|id| frontier.admits_source(id))
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
        ArgumentationTemporalMetadataV1, EvidenceTemporalMetadataV1,
        SourceSnapshotTemporalMetadataV1,
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
                    artifact_time: Some(YearInterval {
                        from: Some(1940),
                        to: Some(1940),
                    }),
                    publication_time: Some(1941),
                    capture_time: None,
                    available_by: 1942,
                    validity_time: Some(YearInterval {
                        from: Some(1900),
                        to: Some(1950),
                    }),
                },
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:2".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(YearInterval {
                        from: Some(1945),
                        to: Some(1945),
                    }),
                    publication_time: Some(1946),
                    capture_time: None,
                    available_by: 1947,
                    validity_time: Some(YearInterval {
                        from: Some(1900),
                        to: Some(1950),
                    }),
                },
                EvidenceTemporalMetadataV1 {
                    evidence_id: "e:recognition".into(),
                    source_snapshot: "source:1".into(),
                    artifact_time: Some(YearInterval {
                        from: Some(1948),
                        to: Some(1948),
                    }),
                    publication_time: Some(1949),
                    capture_time: None,
                    available_by: 1950,
                    validity_time: Some(YearInterval {
                        from: Some(1940),
                        to: Some(1950),
                    }),
                },
            ],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
            }],
            argumentation_metadata: vec![
                ArgumentationTemporalMetadataV1 {
                    assessment: "assessment:1".into(),
                    interpretation: "interpretation:1".into(),
                    assessment_time: Some(YearInterval {
                        from: Some(1948),
                        to: Some(1948),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1949),
                        to: Some(1949),
                    }),
                    available_by: 1950,
                },
                ArgumentationTemporalMetadataV1 {
                    assessment: "assessment:2".into(),
                    interpretation: "interpretation:2".into(),
                    assessment_time: Some(YearInterval {
                        from: Some(1948),
                        to: Some(1948),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1949),
                        to: Some(1949),
                    }),
                    available_by: 1950,
                },
            ],
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
            event_time: YearInterval {
                from: Some(1900),
                to: Some(1950),
            },
            context: Some("documented translation context".into()),
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1", "e:2"].into_iter().map(Into::into).collect(),
            source_snapshots: vec!["source:1".into()],
            assessment: None,
            qualification: QualificationStatus::Supported,
            community_recognition: vec![CommunityRecognitionV1 {
                community: "community:1".into(),
                practice: "practice:target".into(),
                recognition_time: Some(YearInterval {
                    from: Some(1940),
                    to: None,
                }),
                maintained: true,
                transmitted: true,
                evidence_refs: vec!["e:recognition".into()],
            }],
            access_policy: AccessPolicyV1::CommunityRestricted,
            evidence_frontier: "frontier:1950".into(),
        }
    }

    fn canonical_claim_for_parts(value: &CulturalTransformationV1) -> CanonicalClaimAdmissionV1 {
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
            event_time: YearInterval {
                from: Some(1920),
                to: Some(1950),
            },
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
        assert!(
            CulturalProjectionV1::Transformation(value)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn generic_projection_admission_uses_shared_evidence_closure() {
        let frontier = frontier();
        let transmission = transmission();
        let projection = CulturalProjectionV1::Transmission(transmission.clone());
        let admission = CulturalProjectionAdmissionV2::from_projection(
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
        assert_eq!(
            value.validate(),
            Err(ProjectionError::TransitionWithoutEvidencePath)
        );
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
        let public_admission = CulturalProjectionAdmissionV1::from_transmission(
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

        assert_eq!(
            public_admission.qualification,
            restricted_admission.qualification
        );
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
    fn argumentation_temporal_metadata_blocks_late_interpretation() {
        let frontier = frontier();
        let mut value = transmission();
        value.assessment = Some("assessment:1".into());
        let claim = canonical_claim(&value);
        let argumentation = CulturalArgumentationRefV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: value.claim_ref.clone(),
            evidence_refs: value.evidence_refs.clone(),
            source_snapshots: value.source_snapshots.clone(),
            assessment_time: Some(YearInterval {
                from: Some(1948),
                to: Some(1948),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1960),
                to: Some(1960),
            }),
            available_by: 1960,
            evidence_frontier: frontier.frontier_id.clone(),
        };
        assert!(!argumentation.is_frontier_safe(&claim, &frontier));
    }

    #[test]
    fn argumentation_replay_allows_only_records_available_at_frontier() {
        let mut frontier = frontier();
        frontier.known_by_year = 1960;
        frontier.recompute_manifest_hash().expect("fixture hash");
        let value = transmission();
        let mut value = value;
        value.assessment = Some("assessment:1".into());
        let claim = canonical_claim(&value);
        let argumentation = CulturalArgumentationRefV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: value.claim_ref.clone(),
            evidence_refs: value.evidence_refs.clone(),
            source_snapshots: value.source_snapshots.clone(),
            assessment_time: Some(YearInterval {
                from: Some(1948),
                to: Some(1948),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1955),
                to: Some(1955),
            }),
            available_by: 1956,
            evidence_frontier: frontier.frontier_id.clone(),
        };
        assert!(argumentation.is_frontier_safe(&claim, &frontier));

        let audit =
            CulturalProjectionAuditV2::from_projection(&CulturalProjectionV1::Transmission(value))
                .with_argumentation(argumentation);
        assert!(audit.validate().is_ok());
        assert!(audit.is_frontier_safe(&frontier));
    }

    #[test]
    fn argumentation_audit_cannot_attach_to_different_claim_or_assessment() {
        let frontier = frontier();
        let value = transmission();
        let argumentation = CulturalArgumentationRefV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: value.claim_ref.clone(),
            evidence_refs: value.evidence_refs.clone(),
            source_snapshots: value.source_snapshots.clone(),
            assessment_time: Some(YearInterval {
                from: Some(1948),
                to: Some(1948),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1949),
                to: Some(1949),
            }),
            available_by: 1950,
            evidence_frontier: frontier.frontier_id.clone(),
        };
        let mut audit =
            CulturalProjectionAuditV2::from_projection(&CulturalProjectionV1::Transmission(value))
                .with_argumentation(argumentation);
        audit.claim_ref = "claim:other".into();
        assert_eq!(audit.validate(), Err(ProjectionError::EmptyIdentifier));
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
    #[test]
    fn argumentation_v2_can_use_subset_of_canonical_evidence() {
        let frontier = frontier();
        let value = transmission();
        let claim = canonical_claim(&value);
        let argumentation = CulturalArgumentationRefV2 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: value.claim_ref.clone(),
            closure: CulturalArgumentationEvidenceClosureV1 {
                claim_ref: value.claim_ref.clone(),
                evidence_refs: vec!["e:1".into()],
                source_snapshots: vec!["source:1".into()],
                evidence_frontier: frontier.frontier_id.clone(),
            },
            assessment_time: Some(YearInterval {
                from: Some(1948),
                to: Some(1948),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1949),
                to: Some(1949),
            }),
            available_by: 1950,
        };
        assert!(argumentation.is_frontier_safe(&claim, &frontier));
    }

    #[test]
    fn argumentation_set_preserves_competing_alternatives_without_ranking() {
        let frontier = frontier();
        let value = transmission();
        let claim = canonical_claim(&value);
        let mut set = CulturalArgumentationSetV1 {
            claim_ref: value.claim_ref.clone(),
            evidence_frontier: frontier.frontier_id.clone(),
            alternatives: vec![
                CulturalArgumentationRefV2 {
                    assessment: "assessment:2".into(),
                    interpretation: "interpretation:2".into(),
                    claim_ref: value.claim_ref.clone(),
                    closure: CulturalArgumentationEvidenceClosureV1 {
                        claim_ref: value.claim_ref.clone(),
                        evidence_refs: vec!["e:2".into()],
                        source_snapshots: vec!["source:1".into()],
                        evidence_frontier: frontier.frontier_id.clone(),
                    },
                    assessment_time: Some(YearInterval {
                        from: Some(1948),
                        to: Some(1948),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1949),
                        to: Some(1949),
                    }),
                    available_by: 1950,
                },
                CulturalArgumentationRefV2 {
                    assessment: "assessment:1".into(),
                    interpretation: "interpretation:1".into(),
                    claim_ref: value.claim_ref.clone(),
                    closure: CulturalArgumentationEvidenceClosureV1 {
                        claim_ref: value.claim_ref.clone(),
                        evidence_refs: vec!["e:1".into()],
                        source_snapshots: vec!["source:1".into()],
                        evidence_frontier: frontier.frontier_id.clone(),
                    },
                    assessment_time: Some(YearInterval {
                        from: Some(1948),
                        to: Some(1948),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1949),
                        to: Some(1949),
                    }),
                    available_by: 1950,
                },
            ],
        };
        assert!(set.is_frontier_safe(&claim, &frontier));
        set.canonicalize();
        assert_eq!(set.alternatives.len(), 2);
        assert_eq!(set.alternatives[0].assessment, "assessment:1".into());
        assert_eq!(set.alternatives[1].assessment, "assessment:2".into());
    }

    #[test]
    fn canonical_claim_admission_rejects_duplicate_closure_members() {
        let mut value = canonical_claim(&transmission());
        value.evidence_refs.push("e:1".into());
        assert!(value.validate().is_err());

        let mut value = canonical_claim(&transmission());
        value.source_snapshots.push("source:1".into());
        assert!(value.validate().is_err());
    }

    #[test]
    fn argumentation_closure_rejects_duplicate_members() {
        let frontier = frontier();
        let value = transmission();
        let claim = canonical_claim(&value);
        let mut closure = CulturalArgumentationEvidenceClosureV1 {
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: vec!["e:1", "e:1"].into_iter().map(Into::into).collect(),
            source_snapshots: vec!["source:1"].into_iter().map(Into::into).collect(),
            evidence_frontier: frontier.frontier_id.clone(),
        };
        assert!(closure.validate().is_err());

        closure.evidence_refs = vec!["e:1"].into_iter().map(Into::into).collect();
        closure.source_snapshots = vec!["source:1", "source:1"]
            .into_iter()
            .map(Into::into)
            .collect();
        assert!(closure.validate().is_err());
    }

    #[test]
    fn argumentation_set_rejects_duplicate_identity() {
        let frontier = frontier();
        let value = transmission();
        let claim = canonical_claim(&value);
        let argumentation = CulturalArgumentationRefV2 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            claim_ref: value.claim_ref.clone(),
            closure: CulturalArgumentationEvidenceClosureV1 {
                claim_ref: value.claim_ref.clone(),
                evidence_refs: vec!["e:1".into()],
                source_snapshots: vec!["source:1".into()],
                evidence_frontier: frontier.frontier_id.clone(),
            },
            assessment_time: Some(YearInterval {
                from: Some(1948),
                to: Some(1948),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1949),
                to: Some(1949),
            }),
            available_by: 1950,
        };
        let set = CulturalArgumentationSetV1 {
            claim_ref: value.claim_ref.clone(),
            evidence_frontier: frontier.frontier_id.clone(),
            alternatives: vec![argumentation.clone(), argumentation],
        };
        assert!(!set.is_frontier_safe(&claim, &frontier));
    }
}
