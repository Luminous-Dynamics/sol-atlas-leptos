// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-reversible, temporal projection contracts for civilizational and
//! state-formation views. These types describe what a renderer may display;
//! they do not adjudicate canonical historical truth.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

macro_rules! id_type {
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

id_type!(EntityId);
id_type!(SnapshotId);
id_type!(TransitionId);
id_type!(ClaimId);
id_type!(EvidenceId);
id_type!(SourceSnapshotId);
id_type!(EvidenceFrontierId);
id_type!(HypothesisId);
id_type!(InterpretationId);
id_type!(AssessmentId);
id_type!(GeometryRef);

/// Year-based interval; bounds are inclusive and may be open-ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct YearInterval {
    pub from: Option<i32>,
    pub to: Option<i32>,
}

impl YearInterval {
    pub fn is_valid(&self) -> bool {
        matches!((self.from, self.to), (Some(a), Some(b)) if a <= b)
            || self.from.is_none()
            || self.to.is_none()
    }

    pub fn contains(&self, year: i32) -> bool {
        self.from.is_none_or(|from| year >= from) && self.to.is_none_or(|to| year <= to)
    }

    /// Returns whether two inclusive intervals share any year.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.is_valid()
            && other.is_valid()
            && self
                .to
                .is_none_or(|to| other.from.is_none_or(|from| from <= to))
            && other
                .to
                .is_none_or(|to| self.from.is_none_or(|from| from <= to))
    }
}

/// Epistemic qualification is display metadata, never a truth probability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationStatus {
    Established,
    Supported,
    Contested,
    Speculative,
    Refuted,
    Unresolved,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialSemantics {
    DeFactoControl,
    DeJureClaim,
    TreatyRecognizedBoundary,
    AdministrativeBoundary,
    HistoricalCulturalRegion,
    ApproximateExtent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionClass {
    Formation,
    Dissolution,
    Succession,
    Merger,
    Federation,
    Partition,
    Secession,
    Conquest,
    Annexation,
    Colonization,
    Decolonization,
    ConstitutionalTransformation,
    AdministrativeTransfer,
    TreatySettlement,
    BoundaryRevision,
    InstitutionalContinuation,
    InstitutionalReplacement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryProjection {
    pub geometry_ref: GeometryRef,
    pub semantics: SpatialSemantics,
    /// True only when source evidence supports the represented precision.
    pub exact: bool,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualificationSummary {
    pub status: QualificationStatus,
    pub assessment: Option<AssessmentId>,
    pub claim_refs: Vec<ClaimId>,
    pub unresolved: Vec<String>,
    pub contested: bool,
}

/// A point-in-time projection of an entity's historically evidenced state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateSnapshotV1 {
    pub entity_id: EntityId,
    pub snapshot_id: SnapshotId,
    pub valid_time: YearInterval,
    pub geometries: Vec<GeometryProjection>,
    pub institution_refs: Vec<EntityId>,
    pub constitutional_refs: Vec<EntityId>,
    pub relation_refs: Vec<ClaimId>,
    /// Direct evidence path for claims that make the snapshot itself visible.
    pub evidence_refs: Vec<EvidenceId>,
    /// Source snapshots for the snapshot-level evidence path.
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub evidence_frontier: EvidenceFrontierId,
    pub qualification: QualificationSummary,
}

impl StateSnapshotV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.entity_id.is_valid() || !self.snapshot_id.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if !self.valid_time.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if !self.evidence_frontier.is_valid() {
            return Err(ProjectionError::MissingEvidenceFrontier);
        }
        if self.evidence_refs.is_empty() || self.source_snapshots.is_empty() {
            return Err(ProjectionError::SnapshotWithoutEvidencePath);
        }
        if self.relation_refs.iter().any(|id| !id.is_valid())
            || self
                .qualification
                .claim_refs
                .iter()
                .any(|id| !id.is_valid())
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self.institution_refs.iter().any(|id| !id.is_valid())
            || self.constitutional_refs.iter().any(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        for geometry in &self.geometries {
            if !geometry.geometry_ref.is_valid() {
                return Err(ProjectionError::EmptyIdentifier);
            }
            if geometry.evidence.is_empty() {
                return Err(ProjectionError::GeometryWithoutEvidence);
            }
            if geometry.evidence.iter().any(|id| !id.is_valid()) {
                return Err(ProjectionError::EmptyIdentifier);
            }
            if !geometry.exact && geometry.semantics != SpatialSemantics::ApproximateExtent {
                return Err(ProjectionError::ApproximateGeometryMislabelled);
            }
        }
        if self.qualification.contested
            && self.qualification.status != QualificationStatus::Contested
        {
            return Err(ProjectionError::ContestedStatusMismatch);
        }
        Ok(())
    }
}

/// Explicit event connecting one or more historical entities, without
/// assuming a single linear predecessor/successor lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalTransitionV1 {
    pub transition_id: TransitionId,
    pub event_time: YearInterval,
    pub classes: BTreeSet<TransitionClass>,
    pub source_entities: Vec<EntityId>,
    pub target_entities: Vec<EntityId>,
    pub spatial_scope: Vec<GeometryProjection>,
    pub mechanism: Option<String>,
    pub claim_refs: Vec<ClaimId>,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub competing_hypotheses: Vec<HypothesisId>,
    pub assessment: Option<AssessmentId>,
    pub qualification: QualificationStatus,
    /// Uncertainty bounds are descriptive year ranges, not confidence scores.
    pub uncertainty: Option<YearInterval>,
}

impl HistoricalTransitionV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.transition_id.is_valid()
            || self.source_entities.iter().any(|id| !id.is_valid())
            || self.target_entities.iter().any(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if !self.event_time.is_valid() || self.uncertainty.is_some_and(|v| !v.is_valid()) {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if self.classes.is_empty() {
            return Err(ProjectionError::MissingTransitionClass);
        }
        if self.source_entities.is_empty() && self.target_entities.is_empty() {
            return Err(ProjectionError::TransitionWithoutParticipants);
        }
        if self.claim_refs.is_empty()
            || self.evidence_refs.is_empty()
            || self.source_snapshots.is_empty()
        {
            return Err(ProjectionError::TransitionWithoutEvidencePath);
        }
        if self.claim_refs.iter().any(|id| !id.is_valid())
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self.competing_hypotheses.iter().any(|id| !id.is_valid())
            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        for geometry in &self.spatial_scope {
            if !geometry.geometry_ref.is_valid() {
                return Err(ProjectionError::EmptyIdentifier);
            }
            if geometry.evidence.is_empty() {
                return Err(ProjectionError::GeometryWithoutEvidence);
            }
            if geometry.evidence.iter().any(|id| !id.is_valid()) {
                return Err(ProjectionError::EmptyIdentifier);
            }
            if !geometry.exact && geometry.semantics != SpatialSemantics::ApproximateExtent {
                return Err(ProjectionError::ApproximateGeometryMislabelled);
            }
        }
        Ok(())
    }
}

/// Identifies exactly what a renderer is showing and preserves the route back
/// to claims, evidence, source snapshots, temporal scope, qualification, and
/// the evidence frontier used for admission.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionRef {
    Snapshot(SnapshotId),
    Transition(TransitionId),
}

impl ProjectionRef {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Snapshot(id) => id.is_valid(),
            Self::Transition(id) => id.is_valid(),
        }
    }
}

/// An executable "why is this visible?" audit record. It is a projection-level
/// object, not a claim that the underlying history is settled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionAuditV1 {
    pub projection: ProjectionRef,
    pub claim_refs: Vec<ClaimId>,
    pub evidence_refs: Vec<EvidenceId>,
    pub source_snapshots: Vec<SourceSnapshotId>,
    pub hypothesis_refs: Vec<HypothesisId>,
    pub assessment: Option<AssessmentId>,
    pub temporal_scope: YearInterval,
    pub qualification: QualificationStatus,
    pub evidence_frontier: EvidenceFrontierId,
}

impl ProjectionAuditV1 {
    fn geometry_evidence(geometries: &[GeometryProjection]) -> Vec<EvidenceId> {
        geometries
            .iter()
            .flat_map(|geometry| geometry.evidence.iter().cloned())
            .collect()
    }

    pub fn for_snapshot(snapshot: &StateSnapshotV1) -> Self {
        let evidence_refs = snapshot
            .evidence_refs
            .iter()
            .chain(Self::geometry_evidence(&snapshot.geometries).iter())
            .cloned()
            .collect();
        let claim_refs = snapshot
            .relation_refs
            .iter()
            .chain(snapshot.qualification.claim_refs.iter())
            .cloned()
            .collect::<Vec<_>>();

        Self {
            projection: ProjectionRef::Snapshot(snapshot.snapshot_id.clone()),
            claim_refs,
            evidence_refs,
            source_snapshots: snapshot.source_snapshots.clone(),
            hypothesis_refs: Vec::new(),
            assessment: snapshot.qualification.assessment.clone(),
            temporal_scope: snapshot.valid_time,
            qualification: snapshot.qualification.status,
            evidence_frontier: snapshot.evidence_frontier.clone(),
        }
    }

    pub fn for_transition(
        transition: &HistoricalTransitionV1,
        frontier: &EvidenceFrontierV1,
    ) -> Self {
        let evidence_refs = transition
            .evidence_refs
            .iter()
            .chain(
                transition
                    .spatial_scope
                    .iter()
                    .flat_map(|geometry| geometry.evidence.iter()),
            )
            .cloned()
            .collect();

        Self {
            projection: ProjectionRef::Transition(transition.transition_id.clone()),
            claim_refs: transition.claim_refs.clone(),
            evidence_refs,
            source_snapshots: transition.source_snapshots.clone(),
            hypothesis_refs: transition.competing_hypotheses.clone(),
            assessment: transition.assessment.clone(),
            temporal_scope: transition.event_time,
            qualification: transition.qualification,
            evidence_frontier: frontier.frontier_id.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.projection.is_valid()
            || self.claim_refs.iter().any(|id| !id.is_valid())
            || self.evidence_refs.iter().any(|id| !id.is_valid())
            || self.source_snapshots.iter().any(|id| !id.is_valid())
            || self.hypothesis_refs.iter().any(|id| !id.is_valid())
            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if !self.temporal_scope.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if !self.evidence_frontier.is_valid() {
            return Err(ProjectionError::MissingEvidenceFrontier);
        }
        if self.claim_refs.is_empty() || self.evidence_refs.is_empty() {
            return Err(ProjectionError::AuditWithoutEvidencePath);
        }
        if self.source_snapshots.is_empty() {
            return Err(ProjectionError::AuditWithoutSourcePath);
        }
        Ok(())
    }
}

/// Temporal provenance metadata for an evidence item.
///
/// These timestamps are deliberately separate: an artifact may be ancient while
/// its publication, capture, or availability to an evidence pipeline is much later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceTemporalMetadataV1 {
    pub evidence_id: EvidenceId,
    pub source_snapshot: SourceSnapshotId,
    pub artifact_time: Option<YearInterval>,
    pub publication_time: Option<i32>,
    pub capture_time: Option<i32>,
    pub available_by: i32,
    pub validity_time: Option<YearInterval>,
}

impl EvidenceTemporalMetadataV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.evidence_id.is_valid() || !self.source_snapshot.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.available_by < self.publication_time.unwrap_or(self.available_by)
            || self.available_by < self.capture_time.unwrap_or(self.available_by)
        {
            return Err(ProjectionError::InvalidEvidenceTemporalMetadata);
        }
        if self.artifact_time.is_some_and(|v| !v.is_valid())
            || self.validity_time.is_some_and(|v| !v.is_valid())
        {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        Ok(())
    }

    /// Whether this evidence could have been available at the requested frontier.
    pub fn available_at(&self, known_by_year: i32) -> bool {
        self.validate().is_ok() && self.available_by <= known_by_year
    }
}

/// Temporal availability metadata for an immutable source snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSnapshotTemporalMetadataV1 {
    pub source_snapshot: SourceSnapshotId,
    pub publication_time: Option<i32>,
    pub capture_time: Option<i32>,
    pub available_by: i32,
}

impl SourceSnapshotTemporalMetadataV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.source_snapshot.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.available_by < self.publication_time.unwrap_or(self.available_by)
            || self.available_by < self.capture_time.unwrap_or(self.available_by)
        {
            return Err(ProjectionError::InvalidEvidenceTemporalMetadata);
        }
        Ok(())
    }

    pub fn available_at(&self, known_by_year: i32) -> bool {
        self.validate().is_ok() && self.available_by <= known_by_year
    }
}

/// Temporal metadata for externally-owned assessment/interpretation records.
///
/// The frontier commits this metadata so historical replay cannot silently use a
/// later scholarly assessment or interpretation merely because its evidence is old.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentationTemporalMetadataV1 {
    pub assessment: AssessmentId,
    pub interpretation: InterpretationId,
    pub assessment_time: Option<YearInterval>,
    pub interpretation_time: Option<YearInterval>,
    pub available_by: i32,
}

impl ArgumentationTemporalMetadataV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.assessment.is_valid() || !self.interpretation.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.assessment_time.is_some_and(|v| !v.is_valid())
            || self.interpretation_time.is_some_and(|v| !v.is_valid())
        {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        // V1 treats available_by as availability of the completed external
        // argumentation record. Thus an explicit temporal end must not occur
        // after the record becomes available; an open-ended extent still cannot
        // begin after that availability boundary.
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
        {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        Ok(())
    }

    pub fn available_at(&self, known_by_year: i32) -> bool {
        self.validate().is_ok() && self.available_by <= known_by_year
    }
}

/// A projection can be reconstructed only from evidence admitted by this
/// frontier; callers must filter derived assessments by the same boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFrontierV1 {
    pub frontier_id: EvidenceFrontierId,
    pub known_by_year: i32,
    /// Parent frontier creates an append-only temporal lineage.
    pub parent_frontier: Option<EvidenceFrontierId>,
    /// Version of the admission policy used to construct this frontier.
    pub policy_version: String,
    /// SHA-256 over the canonical manifest fields excluding this hash.
    pub manifest_hash: String,
    pub admitted_evidence: BTreeSet<EvidenceId>,
    pub admitted_sources: BTreeSet<SourceSnapshotId>,
    /// Immutable temporal metadata used to verify that admission is not anachronistic.
    pub evidence_metadata: Vec<EvidenceTemporalMetadataV1>,
    pub source_metadata: Vec<SourceSnapshotTemporalMetadataV1>,
    /// Immutable temporal metadata for assessment/interpretation records used by projections.
    pub argumentation_metadata: Vec<ArgumentationTemporalMetadataV1>,
}

/// An ordered, verifiable lineage of temporal evidence frontiers.
///
/// The chain is deliberately supplied as an explicit sequence: a frontier stores only
/// its parent's stable ID and cannot dereference repository state by itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFrontierChainV1 {
    pub frontiers: Vec<EvidenceFrontierV1>,
}

impl EvidenceFrontierChainV1 {
    /// Strict chain validation for reproducible replay. Every frontier in the
    /// supplied ancestry must carry complete source availability metadata.
    pub fn validate_strict(&self) -> Result<(), ProjectionError> {
        if self.frontiers.is_empty() {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        let mut frontier_ids = BTreeSet::new();
        if self
            .frontiers
            .iter()
            .any(|frontier| !frontier_ids.insert(frontier.frontier_id.clone()))
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        if self.frontiers[0].parent_frontier.is_some() {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        self.frontiers[0].validate_temporal_manifest_strict()?;
        for pair in self.frontiers.windows(2) {
            let parent = &pair[0];
            let child = &pair[1];
            child.validate_temporal_manifest_strict()?;
            child.validate_extension_of(parent)?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.frontiers.is_empty() {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        let mut frontier_ids = BTreeSet::new();
        if self
            .frontiers
            .iter()
            .any(|frontier| !frontier_ids.insert(frontier.frontier_id.clone()))
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        if self.frontiers[0].parent_frontier.is_some() {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        self.frontiers[0].validate_temporal_manifest()?;

        for pair in self.frontiers.windows(2) {
            let parent = &pair[0];
            let child = &pair[1];
            child.validate_extension_of(parent)?;
        }

        Ok(())
    }

    pub fn current(&self) -> Option<&EvidenceFrontierV1> {
        self.frontiers.last()
    }
}

/// Explicit version of the canonicalization contract used by frontier manifests.
///
/// V1 intentionally canonicalizes evidence/source metadata ordering but preserves
/// argumentation metadata ordering. That behavior is compatibility-sensitive because
/// existing manifest hashes are content-addressed artifacts.
///
/// A future canonicalization change MUST introduce a new versioned contract rather
/// than silently changing the meaning of an existing manifest hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFrontierManifestCanonicalizationV1;

impl EvidenceFrontierManifestCanonicalizationV1 {
    pub const VERSION: &'static str = "evidence-frontier-manifest-c14n-v1";
}

impl EvidenceFrontierV1 {
    /// Computes the content hash using the explicitly versioned V1 manifest
    /// canonicalization contract.
    pub fn computed_manifest_hash(&self) -> Result<String, ProjectionError> {
        self.computed_manifest_hash_with(EvidenceFrontierManifestCanonicalizationV1)
    }

    pub fn computed_manifest_hash_with(
        &self,
        _canonicalization: EvidenceFrontierManifestCanonicalizationV1,
    ) -> Result<String, ProjectionError> {
        // Metadata is a vector for serialization compatibility, but V1 manifest
        // identity must not depend on caller-provided ordering for evidence/source
        // metadata. Argumentation metadata remains order-sensitive in V1. The
        // version label is deliberately not included in the payload so V1
        // reproduces the already-issued manifest hashes byte-for-byte.
        let mut canonical_metadata = self.evidence_metadata.clone();
        canonical_metadata.sort_by(|a, b| a.evidence_id.cmp(&b.evidence_id));
        let mut canonical_source_metadata = self.source_metadata.clone();
        canonical_source_metadata.sort_by(|a, b| a.source_snapshot.cmp(&b.source_snapshot));

        let payload = (
            &self.known_by_year,
            &self.parent_frontier,
            &self.policy_version,
            &self.admitted_evidence,
            &self.admitted_sources,
            &canonical_metadata,
            &canonical_source_metadata,
            &self.argumentation_metadata,
        );
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn verify_manifest_hash(&self) -> Result<(), ProjectionError> {
        if self.manifest_hash != self.computed_manifest_hash()? {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    /// Recomputes the manifest hash after constructing or extending a frontier.
    pub fn recompute_manifest_hash(&mut self) -> Result<(), ProjectionError> {
        self.manifest_hash = self.computed_manifest_hash()?;
        Ok(())
    }

    /// Validates that this frontier is a strict append-only extension of its parent.
    ///
    /// The parent must be supplied by the caller because the frontier itself stores only
    /// the parent's stable identifier. A valid extension may advance the evidence horizon
    /// and add evidence/sources, but it may not rewrite already-admitted metadata.
    pub fn validate_extension_of(&self, parent: &Self) -> Result<(), ProjectionError> {
        if !self
            .parent_frontier
            .as_ref()
            .is_some_and(|id| id == &parent.frontier_id)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        parent.validate_temporal_manifest()?;
        self.validate_temporal_manifest()?;

        if self.known_by_year < parent.known_by_year
            || self.policy_version != parent.policy_version
            || !self
                .admitted_evidence
                .is_superset(&parent.admitted_evidence)
            || !self.admitted_sources.is_superset(&parent.admitted_sources)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        for parent_metadata in &parent.evidence_metadata {
            let Some(child_metadata) = self
                .evidence_metadata
                .iter()
                .find(|metadata| metadata.evidence_id == parent_metadata.evidence_id)
            else {
                return Err(ProjectionError::InvalidEvidenceFrontierManifest);
            };
            if child_metadata != parent_metadata {
                return Err(ProjectionError::InvalidEvidenceFrontierManifest);
            }
        }

        for parent_metadata in &parent.source_metadata {
            let Some(child_metadata) = self
                .source_metadata
                .iter()
                .find(|metadata| metadata.source_snapshot == parent_metadata.source_snapshot)
            else {
                return Err(ProjectionError::InvalidEvidenceFrontierManifest);
            };
            if child_metadata != parent_metadata {
                return Err(ProjectionError::InvalidEvidenceFrontierManifest);
            }
        }

        // V1 argumentation metadata is representation-sensitive. Inherited
        // argumentation therefore has to remain an exact prefix: reordering an
        // inherited record is a historical rewrite, not an append-only extension.
        if !self
            .argumentation_metadata
            .starts_with(&parent.argumentation_metadata)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        Ok(())
    }

    pub fn admits(&self, evidence: &EvidenceId) -> bool {
        if !self.admitted_evidence.contains(evidence) {
            return false;
        }
        // Legacy manifests remain readable during migration; populated metadata
        // turns on the stronger temporal eligibility check.
        if self.evidence_metadata.is_empty() {
            return true;
        }
        self.evidence_metadata.iter().any(|m| {
            if &m.evidence_id != evidence || !m.available_at(self.known_by_year) {
                return false;
            }
            self.source_metadata.is_empty()
                || self.source_metadata.iter().any(|source| {
                    source.source_snapshot == m.source_snapshot
                        && source.available_at(self.known_by_year)
                })
        })
    }

    /// Strict validation for reproducible replay. Unlike the migration-compatible
    /// validator, this requires temporal metadata for every admitted source snapshot.
    pub fn validate_temporal_manifest_strict(&self) -> Result<(), ProjectionError> {
        self.validate_temporal_manifest()?;
        if self.source_metadata.len() != self.admitted_sources.len() {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        let source_ids = self
            .source_metadata
            .iter()
            .map(|metadata| metadata.source_snapshot.clone())
            .collect::<BTreeSet<_>>();
        if source_ids.len() != self.admitted_sources.len() || source_ids != self.admitted_sources {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        for evidence_metadata in &self.evidence_metadata {
            if !source_ids.contains(&evidence_metadata.source_snapshot) {
                return Err(ProjectionError::UnadmittedSourceMetadata);
            }
        }

        Ok(())
    }

    /// Validates the frontier's admission manifest against its temporal metadata.
    pub fn validate_temporal_manifest(&self) -> Result<(), ProjectionError> {
        if !self.frontier_id.is_valid()
            || self.policy_version.trim().is_empty()
            || self.manifest_hash.trim().is_empty()
            || self
                .parent_frontier
                .as_ref()
                .is_some_and(|id| id == &self.frontier_id)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        self.verify_manifest_hash()?;
        if self.evidence_metadata.is_empty() {
            if !self.admitted_evidence.is_empty() {
                return Err(ProjectionError::InvalidEvidenceFrontierManifest);
            }
        } else {
            if self.evidence_metadata.len() != self.admitted_evidence.len()
                || self
                    .evidence_metadata
                    .iter()
                    .map(|m| &m.evidence_id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != self.evidence_metadata.len()
            {
                return Err(ProjectionError::InvalidEvidenceFrontierManifest);
            }
            for metadata in &self.evidence_metadata {
                metadata.validate()?;
                if metadata.available_by > self.known_by_year {
                    return Err(ProjectionError::LaterEvidenceInFrontier);
                }
                if !self.admitted_evidence.contains(&metadata.evidence_id) {
                    return Err(ProjectionError::UnadmittedEvidenceMetadata);
                }
                if !self.admitted_sources.contains(&metadata.source_snapshot) {
                    return Err(ProjectionError::UnadmittedSourceMetadata);
                }
            }
        }

        if !self.source_metadata.is_empty() {
            let mut seen_sources = BTreeSet::new();
            for metadata in &self.source_metadata {
                metadata.validate()?;
                if metadata.available_by > self.known_by_year
                    || !self.admitted_sources.contains(&metadata.source_snapshot)
                    || !seen_sources.insert(metadata.source_snapshot.clone())
                {
                    return Err(ProjectionError::UnadmittedSourceMetadata);
                }
            }
        }

        if !self.argumentation_metadata.is_empty() {
            let mut seen_argumentation = BTreeSet::new();
            for metadata in &self.argumentation_metadata {
                metadata.validate()?;
                if metadata.available_by > self.known_by_year
                    || !seen_argumentation
                        .insert((metadata.assessment.clone(), metadata.interpretation.clone()))
                {
                    return Err(if metadata.available_by > self.known_by_year {
                        ProjectionError::LaterEvidenceInFrontier
                    } else {
                        ProjectionError::DuplicateArgumentationMetadata
                    });
                }
            }
        }

        Ok(())
    }

    /// Whether an externally-owned assessment/interpretation pair is committed to and
    /// available at this frontier.
    ///
    /// This is intentionally pair-scoped rather than claim-scoped: the frontier records
    /// temporal availability of the external argumentation record. Claim-specific binding
    /// is enforced by CulturalArgumentationRefV3 and its evidence closure, so frontier
    /// admission must not be mistaken for permission to reuse an argument for another claim.
    pub fn admits_argumentation(
        &self,
        assessment: &AssessmentId,
        interpretation: &InterpretationId,
    ) -> bool {
        self.argumentation_metadata.iter().any(|metadata| {
            &metadata.assessment == assessment
                && &metadata.interpretation == interpretation
                && metadata.available_at(self.known_by_year)
        })
    }

    /// A projection is frontier-safe only when every referenced evidence and
    /// source snapshot has been admitted. This deliberately does not inspect
    /// historical truth or infer missing evidence.
    pub fn admits_transition(&self, transition: &HistoricalTransitionV1) -> bool {
        transition
            .evidence_refs
            .iter()
            .chain(
                transition
                    .spatial_scope
                    .iter()
                    .flat_map(|geometry| geometry.evidence.iter()),
            )
            .all(|id| self.admits(id))
            && transition
                .source_snapshots
                .iter()
                .all(|id| self.admits_source(id))
    }

    pub(crate) fn admits_source(&self, source: &SourceSnapshotId) -> bool {
        if !self.admitted_sources.contains(source) {
            return false;
        }
        self.source_metadata.is_empty()
            || self.source_metadata.iter().any(|metadata| {
                &metadata.source_snapshot == source && metadata.available_at(self.known_by_year)
            })
    }

    pub fn admits_snapshot(&self, snapshot: &StateSnapshotV1) -> bool {
        snapshot.evidence_frontier == self.frontier_id
            && snapshot.evidence_refs.iter().all(|id| self.admits(id))
            && snapshot
                .source_snapshots
                .iter()
                .all(|id| self.admits_source(id))
            && snapshot
                .geometries
                .iter()
                .all(|g| g.evidence.iter().all(|id| self.admits(id)))
    }

    pub fn admits_audit(&self, audit: &ProjectionAuditV1) -> bool {
        audit.evidence_frontier == self.frontier_id
            && audit.evidence_refs.iter().all(|id| self.admits(id))
            && audit
                .source_snapshots
                .iter()
                .all(|id| self.admits_source(id))
    }

    /// Returns true when every evidence reference carried by a snapshot,
    /// including spatial evidence, is admitted by this frontier.
    pub fn admits_snapshot_evidence(&self, snapshot: &StateSnapshotV1) -> bool {
        snapshot
            .evidence_refs
            .iter()
            .chain(
                snapshot
                    .geometries
                    .iter()
                    .flat_map(|geometry| geometry.evidence.iter()),
            )
            .all(|id| self.admits(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionError {
    EmptyIdentifier,
    InvalidTimeInterval,
    MissingEvidenceFrontier,
    GeometryWithoutEvidence,
    ApproximateGeometryMislabelled,
    ContestedStatusMismatch,
    MissingTransitionClass,
    TransitionWithoutParticipants,
    IrreversibleTransition,
    SnapshotWithoutEvidencePath,
    TransitionWithoutEvidencePath,
    AuditWithoutEvidencePath,
    AuditWithoutSourcePath,
    InvalidSnapshot,
    InvalidTransition,
    InvalidEvidenceTemporalMetadata,
    InvalidEvidenceFrontierManifest,
    LaterEvidenceInFrontier,
    UnadmittedEvidenceMetadata,
    UnadmittedSourceMetadata,
    DuplicateArgumentationMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geometry(semantics: SpatialSemantics, exact: bool) -> GeometryProjection {
        GeometryProjection {
            geometry_ref: "geom:1".into(),
            semantics,
            exact,
            evidence: vec!["evidence:1".into()],
        }
    }

    fn snapshot() -> StateSnapshotV1 {
        StateSnapshotV1 {
            entity_id: "state:alpha".into(),
            snapshot_id: "snapshot:alpha:1900".into(),
            valid_time: YearInterval {
                from: Some(1900),
                to: Some(1949),
            },
            geometries: vec![geometry(SpatialSemantics::AdministrativeBoundary, true)],
            institution_refs: vec![],
            constitutional_refs: vec![],
            relation_refs: vec!["claim:relation".into()],
            evidence_refs: vec!["evidence:1".into()],
            source_snapshots: vec!["source-snapshot:archive".into()],
            evidence_frontier: "frontier:1949".into(),
            qualification: QualificationSummary {
                status: QualificationStatus::Supported,
                assessment: Some("assessment:1".into()),
                claim_refs: vec!["claim:qualification".into()],
                unresolved: vec![],
                contested: false,
            },
        }
    }

    fn transition() -> HistoricalTransitionV1 {
        HistoricalTransitionV1 {
            transition_id: "transition:1".into(),
            event_time: YearInterval {
                from: Some(1947),
                to: Some(1947),
            },
            classes: [TransitionClass::Partition].into_iter().collect(),
            source_entities: vec!["polity:old".into(), "territory:shared".into()],
            target_entities: vec!["state:new-a".into(), "state:new-b".into()],
            spatial_scope: vec![geometry(SpatialSemantics::ApproximateExtent, false)],
            mechanism: Some("documented constitutional and territorial process".into()),
            claim_refs: vec!["claim:transition".into()],
            evidence_refs: vec!["evidence:partition".into()],
            source_snapshots: vec!["source-snapshot:archive".into()],
            competing_hypotheses: vec!["hypothesis:a".into(), "hypothesis:b".into()],
            assessment: None,
            qualification: QualificationStatus::Contested,
            uncertainty: Some(YearInterval {
                from: Some(1946),
                to: Some(1948),
            }),
        }
    }

    #[test]
    fn snapshot_rejects_invalid_interval_and_missing_frontier() {
        let mut value = snapshot();
        value.valid_time = YearInterval {
            from: Some(1950),
            to: Some(1900),
        };
        assert_eq!(value.validate(), Err(ProjectionError::InvalidTimeInterval));
        let mut value = snapshot();
        value.evidence_frontier = "".into();
        assert_eq!(
            value.validate(),
            Err(ProjectionError::MissingEvidenceFrontier)
        );
    }

    #[test]
    fn snapshot_reports_missing_evidence_path_distinctly() {
        let mut value = snapshot();
        value.evidence_refs.clear();
        assert_eq!(
            value.validate(),
            Err(ProjectionError::SnapshotWithoutEvidencePath)
        );
    }

    #[test]
    fn transition_reports_missing_evidence_path_distinctly() {
        let mut value = transition();
        value.evidence_refs.clear();
        assert_eq!(
            value.validate(),
            Err(ProjectionError::TransitionWithoutEvidencePath)
        );
    }

    #[test]
    fn spatial_precision_must_be_explicit_and_supported() {
        let mut value = snapshot();
        value.geometries = vec![geometry(SpatialSemantics::AdministrativeBoundary, false)];
        assert_eq!(
            value.validate(),
            Err(ProjectionError::ApproximateGeometryMislabelled)
        );
        value.geometries = vec![geometry(SpatialSemantics::ApproximateExtent, false)];
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn transition_supports_many_to_many_and_competing_hypotheses() {
        let value = transition();
        assert_eq!(value.source_entities.len(), 2);
        assert_eq!(value.target_entities.len(), 2);
        assert_eq!(value.competing_hypotheses.len(), 2);
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn transition_requires_reversible_evidence_path() {
        let mut value = transition();
        value.source_snapshots.clear();
        assert_eq!(
            value.validate(),
            Err(ProjectionError::TransitionWithoutEvidencePath)
        );
    }

    #[test]
    fn evidence_availability_is_distinct_from_artifact_time() {
        let metadata = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:old".into(),
            source_snapshot: "source:archive".into(),
            artifact_time: Some(YearInterval {
                from: Some(1200),
                to: Some(1200),
            }),
            publication_time: Some(1800),
            capture_time: Some(1900),
            available_by: 1950,
            validity_time: Some(YearInterval {
                from: Some(1200),
                to: Some(1200),
            }),
        };
        assert!(metadata.available_at(1950));
        assert!(!metadata.available_at(1940));
    }

    #[test]
    fn argumentation_availability_requires_temporal_extent_to_be_available() {
        let base = ArgumentationTemporalMetadataV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            assessment_time: Some(YearInterval {
                from: Some(1940),
                to: Some(1950),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1941),
                to: Some(1949),
            }),
            available_by: 1950,
        };
        assert_eq!(base.validate(), Ok(()));

        let mut before_end = base.clone();
        before_end.available_by = 1949;
        assert_eq!(
            before_end.validate(),
            Err(ProjectionError::InvalidTimeInterval)
        );
    }

    #[test]
    fn argumentation_open_ended_extent_still_requires_known_start_before_availability() {
        let mut value = ArgumentationTemporalMetadataV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            assessment_time: Some(YearInterval {
                from: Some(1950),
                to: None,
            }),
            interpretation_time: None,
            available_by: 1950,
        };
        assert_eq!(value.validate(), Ok(()));

        value.available_by = 1949;
        assert_eq!(value.validate(), Err(ProjectionError::InvalidTimeInterval));
    }

    #[test]
    fn evidence_availability_at_frontier_is_inclusive() {
        let metadata = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:cutoff".into(),
            source_snapshot: "source:cutoff".into(),
            artifact_time: Some(YearInterval {
                from: Some(1800),
                to: Some(1800),
            }),
            publication_time: Some(1900),
            capture_time: None,
            available_by: 1950,
            validity_time: None,
        };
        assert!(!metadata.available_at(1949));
        assert!(metadata.available_at(1950));
        assert!(metadata.available_at(1951));
    }

    #[test]
    fn source_snapshot_availability_at_frontier_is_inclusive() {
        let metadata = SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:cutoff".into(),
            publication_time: Some(1940),
            capture_time: Some(1950),
            available_by: 1950,
        };
        assert!(!metadata.available_at(1949));
        assert!(metadata.available_at(1950));
    }

    #[test]
    fn argumentation_frontier_admission_uses_record_availability_not_temporal_extent() {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:a".into(),
                source_snapshot: "source:a".into(),
                artifact_time: Some(YearInterval {
                    from: Some(1200),
                    to: Some(1200),
                }),
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1950,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:a".into(),
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1950,
            }],
            argumentation_metadata: vec![ArgumentationTemporalMetadataV1 {
                assessment: "assessment:old".into(),
                interpretation: "interpretation:old".into(),
                assessment_time: Some(YearInterval {
                    from: Some(1200),
                    to: Some(1200),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1300),
                    to: Some(1300),
                }),
                available_by: 1950,
            }],
        };
        frontier.recompute_manifest_hash().unwrap();

        assert!(
            frontier.admits_argumentation(&"assessment:old".into(), &"interpretation:old".into())
        );

        frontier.known_by_year = 1949;
        assert!(
            !frontier.admits_argumentation(&"assessment:old".into(), &"interpretation:old".into())
        );
    }

    #[test]
    fn later_frontier_horizon_does_not_make_earlier_metadata_available() {
        let metadata = ArgumentationTemporalMetadataV1 {
            assessment: "assessment:later".into(),
            interpretation: "interpretation:later".into(),
            assessment_time: Some(YearInterval {
                from: Some(1800),
                to: Some(1800),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1800),
                to: Some(1800),
            }),
            available_by: 1951,
        };
        assert!(!metadata.available_at(1950));
        assert!(metadata.available_at(1951));
    }

    #[test]
    fn argumentation_availability_can_be_unknown_until_a_later_frontier() {
        let value = ArgumentationTemporalMetadataV1 {
            assessment: "assessment:1".into(),
            interpretation: "interpretation:1".into(),
            assessment_time: Some(YearInterval {
                from: Some(1940),
                to: Some(1950),
            }),
            interpretation_time: None,
            available_by: 1950,
        };
        assert!(!value.available_at(1949));
        assert!(value.available_at(1950));
    }

    #[test]
    fn frontier_temporal_manifest_blocks_anachronistic_evidence() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:archive".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: Some(YearInterval {
                    from: Some(1200),
                    to: Some(1200),
                }),
                publication_time: Some(1800),
                capture_time: None,
                available_by: 1800,
                validity_time: None,
            }],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        assert_eq!(frontier.validate_temporal_manifest(), Ok(()));
        assert!(frontier.admits(&"evidence:old".into()));
    }

    #[test]
    fn frontier_rejects_metadata_discovered_after_frontier() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:later".into()].into_iter().collect(),
            admitted_sources: ["source:archive".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:later".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: Some(YearInterval {
                    from: Some(1200),
                    to: Some(1200),
                }),
                publication_time: Some(1950),
                capture_time: None,
                available_by: 1950,
                validity_time: None,
            }],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        assert_eq!(
            frontier.validate_temporal_manifest(),
            Err(ProjectionError::LaterEvidenceInFrontier)
        );
        assert!(!frontier.admits(&"evidence:later".into()));
    }

    #[test]
    fn frontier_excludes_later_evidence_unless_admitted() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:old".into()].into_iter().collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        assert!(frontier.admits(&"evidence:old".into()));
        assert!(!frontier.admits(&"evidence:discovered-later".into()));
    }

    #[test]
    fn frontier_rejects_transition_with_later_evidence_or_source() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:old".into()].into_iter().collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        let mut value = transition();
        value.evidence_refs = vec!["evidence:old".into()];
        value.source_snapshots = vec!["source:later".into()];
        assert!(!frontier.admits_transition(&value));

        value.source_snapshots = vec!["source:old".into()];
        assert!(frontier.admits_transition(&value));
    }

    #[test]
    fn snapshot_frontier_identity_is_explicit() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:1".into()].into_iter().collect(),
            admitted_sources: ["source:1".into()].into_iter().collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        assert!(frontier.admits_snapshot(&snapshot()));

        let mut later = snapshot();
        later.evidence_frontier = "frontier:1950".into();
        assert!(!frontier.admits_snapshot(&later));
    }

    #[test]
    fn audit_is_reversible_and_frontier_safe() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:1".into(), "evidence:partition".into()]
                .into_iter()
                .collect(),
            admitted_sources: ["source-snapshot:archive"]
                .into_iter()
                .map(Into::into)
                .collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };

        let snapshot_audit = ProjectionAuditV1::for_snapshot(&snapshot());
        assert_eq!(snapshot_audit.validate(), Ok(()));
        assert_eq!(snapshot_audit.claim_refs.len(), 2);
        assert!(frontier.admits_audit(&snapshot_audit));

        let transition_audit = ProjectionAuditV1::for_transition(&transition(), &frontier);
        assert_eq!(transition_audit.validate(), Ok(()));
        assert_eq!(transition_audit.evidence_refs.len(), 2);
        assert!(frontier.admits_audit(&transition_audit));

        let mut later = transition_audit;
        later.evidence_refs.push("evidence:discovered-later".into());
        assert!(!frontier.admits_audit(&later));
    }

    #[test]
    fn audit_requires_evidence_path() {
        let mut audit = ProjectionAuditV1::for_snapshot(&snapshot());
        audit.evidence_refs.clear();
        assert_eq!(
            audit.validate(),
            Err(ProjectionError::AuditWithoutEvidencePath)
        );
    }

    #[test]
    fn transition_audit_requires_source_snapshot() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:partition"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source-snapshot:archive"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        let mut audit = ProjectionAuditV1::for_transition(&transition(), &frontier);
        audit.source_snapshots.clear();
        assert_eq!(
            audit.validate(),
            Err(ProjectionError::AuditWithoutSourcePath)
        );
    }

    #[test]
    fn transition_audit_cannot_hide_spatial_evidence() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:partition"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source-snapshot:archive"].into_iter().collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        let audit = ProjectionAuditV1::for_transition(&transition(), &frontier);
        assert!(!frontier.admits_audit(&audit));
    }

    #[test]
    fn snapshot_audit_includes_relation_claims() {
        let audit = ProjectionAuditV1::for_snapshot(&snapshot());
        assert!(audit.claim_refs.contains(&"claim:relation".into()));
        assert!(audit.claim_refs.contains(&"claim:qualification".into()));
    }

    #[test]
    fn frontier_chain_validates_transitive_append_only_lineage() {
        let metadata = |evidence_id: &str, source: &str, year: i32| EvidenceTemporalMetadataV1 {
            evidence_id: evidence_id.into(),
            source_snapshot: source.into(),
            artifact_time: None,
            publication_time: Some(year),
            capture_time: None,
            available_by: year,
            validity_time: None,
        };

        let mut root = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![metadata("evidence:a", "source:a", 1900)],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        root.recompute_manifest_hash().unwrap();

        let mut middle = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: Some(root.frontier_id.clone()),
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into(), "evidence:b".into()]
                .into_iter()
                .collect(),
            admitted_sources: ["source:a".into(), "source:b".into()].into_iter().collect(),
            evidence_metadata: vec![
                metadata("evidence:a", "source:a", 1900),
                metadata("evidence:b", "source:b", 1950),
            ],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        middle.recompute_manifest_hash().unwrap();

        let mut current = EvidenceFrontierV1 {
            frontier_id: "frontier:2000".into(),
            known_by_year: 2000,
            parent_frontier: Some(middle.frontier_id.clone()),
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: [
                "evidence:a".into(),
                "evidence:b".into(),
                "evidence:c".into(),
            ]
            .into_iter()
            .collect(),
            admitted_sources: ["source:a".into(), "source:b".into(), "source:c".into()]
                .into_iter()
                .collect(),
            evidence_metadata: vec![
                metadata("evidence:a", "source:a", 1900),
                metadata("evidence:b", "source:b", 1950),
                metadata("evidence:c", "source:c", 2000),
            ],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        current.recompute_manifest_hash().unwrap();

        let chain = EvidenceFrontierChainV1 {
            frontiers: vec![root, middle, current],
        };
        assert_eq!(chain.validate(), Ok(()));
        assert_eq!(chain.current().unwrap().frontier_id, "frontier:2000".into());
    }

    #[test]
    fn frontier_chain_rejects_rewritten_ancestor() {
        let metadata = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:a".into(),
            source_snapshot: "source:a".into(),
            artifact_time: None,
            publication_time: Some(1900),
            capture_time: None,
            available_by: 1900,
            validity_time: None,
        };
        let mut root = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![metadata.clone()],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        root.recompute_manifest_hash().unwrap();

        let mut child = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: Some(root.frontier_id.clone()),
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![metadata],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        child.recompute_manifest_hash().unwrap();

        root.policy_version = "rewritten".into();
        root.recompute_manifest_hash().unwrap();

        let chain = EvidenceFrontierChainV1 {
            frontiers: vec![root, child],
        };
        assert_eq!(
            chain.validate(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn manifest_hash_is_independent_of_metadata_order() {
        let metadata_a = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:a".into(),
            source_snapshot: "source:a".into(),
            artifact_time: None,
            publication_time: Some(1900),
            capture_time: None,
            available_by: 1900,
            validity_time: None,
        };
        let metadata_b = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:b".into(),
            source_snapshot: "source:b".into(),
            artifact_time: None,
            publication_time: Some(1901),
            capture_time: None,
            available_by: 1901,
            validity_time: None,
        };

        let mut first = EvidenceFrontierV1 {
            frontier_id: "frontier:1901".into(),
            known_by_year: 1901,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into(), "evidence:b".into()]
                .into_iter()
                .collect(),
            admitted_sources: ["source:a".into(), "source:b".into()].into_iter().collect(),
            evidence_metadata: vec![metadata_a.clone(), metadata_b.clone()],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        let mut second = first.clone();
        second.evidence_metadata.reverse();

        first.recompute_manifest_hash().unwrap();
        second.recompute_manifest_hash().unwrap();

        assert_eq!(first.manifest_hash, second.manifest_hash);
        assert_eq!(first.validate_temporal_manifest(), Ok(()));
        assert_eq!(second.validate_temporal_manifest(), Ok(()));
    }

    #[test]
    fn manifest_hash_detects_policy_and_admission_tampering() {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1901".into(),
            known_by_year: 1901,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:a".into(),
                source_snapshot: "source:a".into(),
                artifact_time: None,
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1900,
                validity_time: None,
            }],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        frontier.recompute_manifest_hash().unwrap();

        frontier.policy_version = "v2".into();
        assert_eq!(
            frontier.verify_manifest_hash(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );

        frontier.policy_version = "v1".into();
        frontier.admitted_evidence.insert("evidence:b".into());
        assert_eq!(
            frontier.verify_manifest_hash(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn manifest_canonicalization_v1_is_explicit_and_argumentation_order_sensitive() {
        let metadata = |assessment: &str, interpretation: &str| ArgumentationTemporalMetadataV1 {
            assessment: assessment.into(),
            interpretation: interpretation.into(),
            assessment_time: Some(YearInterval {
                from: Some(1990),
                to: Some(1990),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1995),
                to: Some(1995),
            }),
            available_by: 1996,
        };

        let mut first = EvidenceFrontierV1 {
            frontier_id: "frontier:c14n-v1".into(),
            known_by_year: 2000,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![
                metadata("assessment:a", "interpretation:a"),
                metadata("assessment:b", "interpretation:b"),
            ],
        };
        let mut second = first.clone();
        second.argumentation_metadata.reverse();

        let first_hash = first
            .computed_manifest_hash_with(EvidenceFrontierManifestCanonicalizationV1)
            .unwrap();
        let second_hash = second
            .computed_manifest_hash_with(EvidenceFrontierManifestCanonicalizationV1)
            .unwrap();

        assert_eq!(
            EvidenceFrontierManifestCanonicalizationV1::VERSION,
            "evidence-frontier-manifest-c14n-v1"
        );
        assert_ne!(first_hash, second_hash);

        first.recompute_manifest_hash().unwrap();
        assert_eq!(first.manifest_hash, first_hash);
    }

    #[test]
    fn duplicate_argumentation_metadata_has_a_dedicated_validation_error() {
        let metadata = ArgumentationTemporalMetadataV1 {
            assessment: "assessment:a".into(),
            interpretation: "interpretation:a".into(),
            assessment_time: Some(YearInterval {
                from: Some(1990),
                to: Some(1990),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1995),
                to: Some(1995),
            }),
            available_by: 1996,
        };

        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:duplicate-argumentation".into(),
            known_by_year: 2000,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:a".into(),
                source_snapshot: "source:a".into(),
                artifact_time: None,
                publication_time: Some(1990),
                capture_time: None,
                available_by: 1990,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:a".into(),
                publication_time: Some(1990),
                capture_time: None,
                available_by: 1990,
            }],
            argumentation_metadata: vec![metadata.clone(), metadata],
        };
        frontier.recompute_manifest_hash().unwrap();

        assert_eq!(
            frontier.validate_temporal_manifest(),
            Err(ProjectionError::DuplicateArgumentationMetadata)
        );
    }

    #[test]
    fn argumentation_metadata_is_committed_to_frontier_hash() {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:2000".into(),
            known_by_year: 2000,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:a".into(),
                source_snapshot: "source:a".into(),
                artifact_time: None,
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1900,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:a".into(),
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1900,
            }],
            argumentation_metadata: vec![ArgumentationTemporalMetadataV1 {
                assessment: "assessment:a".into(),
                interpretation: "interpretation:a".into(),
                assessment_time: Some(YearInterval {
                    from: Some(1990),
                    to: Some(1990),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1995),
                    to: Some(1995),
                }),
                available_by: 1996,
            }],
        };
        frontier.recompute_manifest_hash().unwrap();
        assert!(frontier.admits_argumentation(&"assessment:a".into(), &"interpretation:a".into()));

        frontier.argumentation_metadata[0].available_by = 2001;
        assert_eq!(
            frontier.verify_manifest_hash(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn frontier_extension_is_append_only_and_metadata_immutable() {
        let parent_metadata = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:a".into(),
            source_snapshot: "source:a".into(),
            artifact_time: None,
            publication_time: Some(1900),
            capture_time: None,
            available_by: 1900,
            validity_time: None,
        };
        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![parent_metadata.clone()],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        parent.recompute_manifest_hash().unwrap();

        let child_metadata = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:b".into(),
            source_snapshot: "source:b".into(),
            artifact_time: None,
            publication_time: Some(1901),
            capture_time: None,
            available_by: 1901,
            validity_time: None,
        };
        let mut child = EvidenceFrontierV1 {
            frontier_id: "frontier:1901".into(),
            known_by_year: 1901,
            parent_frontier: Some(parent.frontier_id.clone()),
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into(), "evidence:b".into()]
                .into_iter()
                .collect(),
            admitted_sources: ["source:a".into(), "source:b".into()].into_iter().collect(),
            evidence_metadata: vec![parent_metadata.clone(), child_metadata],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        child.recompute_manifest_hash().unwrap();

        assert_eq!(child.validate_extension_of(&parent), Ok(()));

        child.evidence_metadata[0].available_by = 1899;
        child.recompute_manifest_hash().unwrap();
        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );

        child.evidence_metadata[0] = parent_metadata;
        child.parent_frontier = Some("frontier:wrong".into());
        child.recompute_manifest_hash().unwrap();
        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn frontier_chain_rejects_duplicate_frontier_identity() {
        let mut root = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:1".into()].into_iter().collect(),
            admitted_sources: ["source:1".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(),
                source_snapshot: "source:1".into(),
                artifact_time: None,
                publication_time: Some(1890),
                capture_time: None,
                available_by: 1900,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1890),
                capture_time: None,
                available_by: 1900,
            }],
            argumentation_metadata: vec![],
        };
        root.recompute_manifest_hash().unwrap();

        let mut child = root.clone();
        child.known_by_year = 1901;
        child.parent_frontier = Some(root.frontier_id.clone());
        // Same frontier ID, but a valid-looking independently hashed manifest.
        child.admitted_evidence.insert("e:2".into());
        child.evidence_metadata.push(EvidenceTemporalMetadataV1 {
            evidence_id: "e:2".into(),
            source_snapshot: "source:1".into(),
            artifact_time: None,
            publication_time: Some(1891),
            capture_time: None,
            available_by: 1901,
            validity_time: None,
        });
        child.recompute_manifest_hash().unwrap();

        let chain = EvidenceFrontierChainV1 {
            frontiers: vec![root, child],
        };

        assert_eq!(
            chain.validate(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
        assert_eq!(
            chain.validate_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn frontier_extension_preserves_inherited_source_and_argumentation_metadata() {
        let inherited_source = SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1890),
            capture_time: Some(1895),
            available_by: 1900,
        };
        let inherited_argumentation = ArgumentationTemporalMetadataV1 {
            assessment: "assessment:old".into(),
            interpretation: "interpretation:old".into(),
            assessment_time: Some(YearInterval {
                from: Some(1880),
                to: Some(1880),
            }),
            interpretation_time: Some(YearInterval {
                from: Some(1885),
                to: Some(1885),
            }),
            available_by: 1900,
        };
        let evidence = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:archive".into(),
            source_snapshot: inherited_source.source_snapshot.clone(),
            artifact_time: None,
            publication_time: Some(1890),
            capture_time: None,
            available_by: 1900,
            validity_time: None,
        };

        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:archive".into()].into_iter().collect(),
            admitted_sources: ["source:archive".into()].into_iter().collect(),
            evidence_metadata: vec![evidence.clone()],
            source_metadata: vec![inherited_source.clone()],
            argumentation_metadata: vec![inherited_argumentation.clone()],
        };
        parent.recompute_manifest_hash().unwrap();

        let mut child = parent.clone();
        child.frontier_id = "frontier:1910".into();
        child.known_by_year = 1910;
        child.parent_frontier = Some(parent.frontier_id.clone());
        child.recompute_manifest_hash().unwrap();

        assert_eq!(child.validate_extension_of(&parent), Ok(()));
        assert_eq!(child.source_metadata[0], inherited_source);
        assert_eq!(child.argumentation_metadata[0], inherited_argumentation);

        child.source_metadata[0].available_by = 1905;
        child.recompute_manifest_hash().unwrap();
        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );

        child.source_metadata[0] = inherited_source;
        child.argumentation_metadata[0].assessment_time = Some(YearInterval {
            from: Some(1881),
            to: Some(1881),
        });
        child.recompute_manifest_hash().unwrap();
        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );

        child.source_metadata[0] = parent.source_metadata[0].clone();
        child.argumentation_metadata[0] = parent.argumentation_metadata[0].clone();
        child.evidence_metadata = vec![evidence];
        child.recompute_manifest_hash().unwrap();
        assert_eq!(child.validate_extension_of(&parent), Ok(()));
    }

    #[test]
    fn argumentation_metadata_is_validated_when_evidence_metadata_is_absent() {
        let mut frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:argumentation-only".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: BTreeSet::new(),
            admitted_sources: BTreeSet::new(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![ArgumentationTemporalMetadataV1 {
                assessment: "assessment:later".into(),
                interpretation: "interpretation:later".into(),
                assessment_time: Some(YearInterval {
                    from: Some(1940),
                    to: Some(1940),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1940),
                    to: Some(1940),
                }),
                available_by: 1951,
            }],
        };
        frontier.recompute_manifest_hash().unwrap();

        assert_eq!(
            frontier.validate_temporal_manifest(),
            Err(ProjectionError::LaterEvidenceInFrontier)
        );
    }

    #[test]
    fn frontier_extension_allows_canonical_metadata_reordering() {
        let metadata_a = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:a".into(),
            source_snapshot: "source:a".into(),
            artifact_time: None,
            publication_time: Some(1890),
            capture_time: None,
            available_by: 1900,
            validity_time: None,
        };
        let metadata_b = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:b".into(),
            source_snapshot: "source:b".into(),
            artifact_time: None,
            publication_time: Some(1891),
            capture_time: None,
            available_by: 1901,
            validity_time: None,
        };
        let source_a = SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:a".into(),
            publication_time: Some(1890),
            capture_time: None,
            available_by: 1900,
        };
        let source_b = SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:b".into(),
            publication_time: Some(1891),
            capture_time: None,
            available_by: 1901,
        };

        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:c14n-parent".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![metadata_a.clone()],
            source_metadata: vec![source_a.clone()],
            argumentation_metadata: vec![],
        };
        parent.recompute_manifest_hash().unwrap();

        let mut child = EvidenceFrontierV1 {
            frontier_id: "frontier:c14n-child".into(),
            known_by_year: 1901,
            parent_frontier: Some(parent.frontier_id.clone()),
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a", "evidence:b"]
                .into_iter()
                .map(EvidenceId::from)
                .collect(),
            admitted_sources: ["source:a", "source:b"]
                .into_iter()
                .map(SourceSnapshotId::from)
                .collect(),
            evidence_metadata: vec![metadata_b, metadata_a],
            source_metadata: vec![source_b, source_a],
            argumentation_metadata: vec![],
        };
        child.recompute_manifest_hash().unwrap();

        assert_eq!(child.validate_extension_of(&parent), Ok(()));
        assert_eq!(
            child.verify_manifest_hash(),
            Ok(()),
            "canonical metadata reordering must remain self-consistent"
        );
    }

    #[test]
    fn frontier_extension_rejects_policy_version_change() {
        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:policy-parent".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: BTreeSet::new(),
            admitted_sources: BTreeSet::new(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        parent.recompute_manifest_hash().unwrap();

        let mut child = parent.clone();
        child.frontier_id = "frontier:policy-child".into();
        child.known_by_year = 1901;
        child.parent_frontier = Some(parent.frontier_id.clone());
        child.policy_version = "v2".into();
        child.recompute_manifest_hash().unwrap();

        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn frontier_extension_rejects_known_by_year_regression() {
        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:horizon-parent".into(),
            known_by_year: 1901,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: BTreeSet::new(),
            admitted_sources: BTreeSet::new(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![],
        };
        parent.recompute_manifest_hash().unwrap();

        let mut child = parent.clone();
        child.frontier_id = "frontier:horizon-child".into();
        child.known_by_year = 1900;
        child.parent_frontier = Some(parent.frontier_id.clone());
        child.recompute_manifest_hash().unwrap();

        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn frontier_extension_requires_admission_sets_to_be_monotonic() {
        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:admission-parent".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["evidence:a".into()].into_iter().collect(),
            admitted_sources: ["source:a".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:a".into(),
                source_snapshot: "source:a".into(),
                artifact_time: None,
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1900,
                validity_time: None,
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:a".into(),
                publication_time: Some(1900),
                capture_time: None,
                available_by: 1900,
            }],
            argumentation_metadata: vec![],
        };
        parent.recompute_manifest_hash().unwrap();

        let mut child = parent.clone();
        child.frontier_id = "frontier:admission-child".into();
        child.known_by_year = 1901;
        child.parent_frontier = Some(parent.frontier_id.clone());
        child.admitted_evidence.clear();
        child.evidence_metadata.clear();
        child.admitted_sources.clear();
        child.source_metadata.clear();
        child.recompute_manifest_hash().unwrap();

        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn frontier_extension_rejects_reordering_inherited_argumentation() {
        let mut parent = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: BTreeSet::new(),
            admitted_sources: BTreeSet::new(),
            evidence_metadata: vec![],
            source_metadata: vec![],
            argumentation_metadata: vec![
                ArgumentationTemporalMetadataV1 {
                    assessment: "assessment:a".into(),
                    interpretation: "interpretation:a".into(),
                    assessment_time: Some(YearInterval {
                        from: Some(1800),
                        to: Some(1800),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1800),
                        to: Some(1800),
                    }),
                    available_by: 1900,
                },
                ArgumentationTemporalMetadataV1 {
                    assessment: "assessment:b".into(),
                    interpretation: "interpretation:b".into(),
                    assessment_time: Some(YearInterval {
                        from: Some(1801),
                        to: Some(1801),
                    }),
                    interpretation_time: Some(YearInterval {
                        from: Some(1801),
                        to: Some(1801),
                    }),
                    available_by: 1900,
                },
            ],
        };
        parent.recompute_manifest_hash().unwrap();
        let mut child = parent.clone();
        child.frontier_id = "frontier:1901".into();
        child.known_by_year = 1901;
        child.parent_frontier = Some(parent.frontier_id.clone());
        child.argumentation_metadata.reverse();
        child.recompute_manifest_hash().unwrap();

        assert_eq!(
            child.validate_extension_of(&parent),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }
}
