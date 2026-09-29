// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-reversible, temporal projection contracts for civilizational and
//! state-formation views. These types describe what a renderer may display;
//! they do not adjudicate canonical historical truth.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

macro_rules! id_type {
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

id_type!(EntityId);
id_type!(SnapshotId);
id_type!(TransitionId);
id_type!(ClaimId);
id_type!(EvidenceId);
id_type!(SourceSnapshotId);
id_type!(EvidenceFrontierId);
id_type!(HypothesisId);
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
            && self.to.is_none_or(|to| other.from.is_none_or(|from| from <= to))
            && other.to.is_none_or(|to| self.from.is_none_or(|from| from <= to))
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
        for geometry in &self.geometries {
            if !geometry.geometry_ref.is_valid() {
                return Err(ProjectionError::EmptyIdentifier);
            }
            if geometry.evidence.is_empty() {
                return Err(ProjectionError::GeometryWithoutEvidence);
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
        if self.claim_refs.is_empty() || self.evidence_refs.is_empty() || self.source_snapshots.is_empty() {
            return Err(ProjectionError::IrreversibleTransition);
        }
        for geometry in &self.spatial_scope {
            if geometry.evidence.is_empty() {
                return Err(ProjectionError::GeometryWithoutEvidence);
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
        let evidence_refs = Self::geometry_evidence(&snapshot.geometries);
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
            source_snapshots: Vec::new(),
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
        if matches!(self.projection, ProjectionRef::Transition(_))
            && self.source_snapshots.is_empty()
        {
            return Err(ProjectionError::IrreversibleTransition);
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

/// A projection can be reconstructed only from evidence admitted by this
/// frontier; callers must filter derived assessments by the same boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFrontierV1 {
    pub frontier_id: EvidenceFrontierId,
    pub known_by_year: i32,
    pub admitted_evidence: BTreeSet<EvidenceId>,
    pub admitted_sources: BTreeSet<SourceSnapshotId>,
    /// Immutable temporal metadata used to verify that admission is not anachronistic.
    pub evidence_metadata: Vec<EvidenceTemporalMetadataV1>,
}

impl EvidenceFrontierV1 {
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
            &m.evidence_id == evidence && m.available_at(self.known_by_year)
        })
    }

    /// Validates the frontier's admission manifest against its temporal metadata.
    pub fn validate_temporal_manifest(&self) -> Result<(), ProjectionError> {
        if !self.frontier_id.is_valid() || self.evidence_metadata.is_empty() && !self.admitted_evidence.is_empty() {
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
        Ok(())
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
            .all(|id| self.admitted_evidence.contains(id))
            && transition
                .source_snapshots
                .iter()
                .all(|id| self.admitted_sources.contains(id))
    }

    pub fn admits_snapshot(&self, snapshot: &StateSnapshotV1) -> bool {
        snapshot.evidence_frontier == self.frontier_id
            && snapshot.geometries.iter().all(|g| {
                g.evidence.iter().all(|id| self.admitted_evidence.contains(id))
            })
    }

    pub fn admits_audit(&self, audit: &ProjectionAuditV1) -> bool {
        audit.evidence_frontier == self.frontier_id
            && audit.evidence_refs.iter().all(|id| self.admitted_evidence.contains(id))
            && audit.source_snapshots.iter().all(|id| self.admitted_sources.contains(id))
    }

    /// Returns true when every evidence reference carried by a snapshot,
    /// including spatial evidence, is admitted by this frontier.
    pub fn admits_snapshot_evidence(&self, snapshot: &StateSnapshotV1) -> bool {
        snapshot
            .geometries
            .iter()
            .flat_map(|geometry| geometry.evidence.iter())
            .all(|id| self.admitted_evidence.contains(id))
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
    AuditWithoutEvidencePath,
    InvalidSnapshot,
    InvalidTransition,
    InvalidEvidenceTemporalMetadata,
    InvalidEvidenceFrontierManifest,
    LaterEvidenceInFrontier,
    UnadmittedEvidenceMetadata,
    UnadmittedSourceMetadata,
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
            valid_time: YearInterval { from: Some(1900), to: Some(1949) },
            geometries: vec![geometry(SpatialSemantics::AdministrativeBoundary, true)],
            institution_refs: vec![],
            constitutional_refs: vec![],
            relation_refs: vec!["claim:relation".into()],
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
            event_time: YearInterval { from: Some(1947), to: Some(1947) },
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
            uncertainty: Some(YearInterval { from: Some(1946), to: Some(1948) }),
        }
    }

    #[test]
    fn snapshot_rejects_invalid_interval_and_missing_frontier() {
        let mut value = snapshot();
        value.valid_time = YearInterval { from: Some(1950), to: Some(1900) };
        assert_eq!(value.validate(), Err(ProjectionError::InvalidTimeInterval));
        let mut value = snapshot();
        value.evidence_frontier = "".into();
        assert_eq!(value.validate(), Err(ProjectionError::MissingEvidenceFrontier));
    }

    #[test]
    fn spatial_precision_must_be_explicit_and_supported() {
        let mut value = snapshot();
        value.geometries = vec![geometry(SpatialSemantics::AdministrativeBoundary, false)];
        assert_eq!(value.validate(), Err(ProjectionError::ApproximateGeometryMislabelled));
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
        assert_eq!(value.validate(), Err(ProjectionError::IrreversibleTransition));
    }

    #[test]
    fn evidence_availability_is_distinct_from_artifact_time() {
        let metadata = EvidenceTemporalMetadataV1 {
            evidence_id: "evidence:old".into(),
            source_snapshot: "source:archive".into(),
            artifact_time: Some(YearInterval { from: Some(1200), to: Some(1200) }),
            publication_time: Some(1800),
            capture_time: Some(1900),
            available_by: 1950,
            validity_time: Some(YearInterval { from: Some(1200), to: Some(1200) }),
        };
        assert!(metadata.available_at(1950));
        assert!(!metadata.available_at(1940));
    }

    #[test]
    fn frontier_temporal_manifest_blocks_anachronistic_evidence() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:archive".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: Some(YearInterval { from: Some(1200), to: Some(1200) }),
                publication_time: Some(1800),
                capture_time: None,
                available_by: 1800,
                validity_time: None,
            }],
        };
        assert_eq!(frontier.validate_temporal_manifest(), Ok(()));
        assert!(frontier.admits(&"evidence:old".into()));
    }

    #[test]
    fn frontier_rejects_metadata_discovered_after_frontier() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            admitted_evidence: ["evidence:later".into()].into_iter().collect(),
            admitted_sources: ["source:archive".into()].into_iter().collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "evidence:later".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: Some(YearInterval { from: Some(1200), to: Some(1200) }),
                publication_time: Some(1950),
                capture_time: None,
                available_by: 1950,
                validity_time: None,
            }],
        };
        assert_eq!(frontier.validate_temporal_manifest(), Err(ProjectionError::LaterEvidenceInFrontier));
        assert!(!frontier.admits(&"evidence:later".into()));
    }

    #[test]
    fn frontier_excludes_later_evidence_unless_admitted() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:old".into()].into_iter().collect(),
            evidence_metadata: vec![],
        };
        assert!(frontier.admits(&"evidence:old".into()));
        assert!(!frontier.admits(&"evidence:discovered-later".into()));
    }

    #[test]
    fn frontier_rejects_transition_with_later_evidence_or_source() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:old".into()].into_iter().collect(),
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
            admitted_evidence: ["evidence:1".into()].into_iter().collect(),
            admitted_sources: ["source:1".into()].into_iter().collect(),
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
            admitted_evidence: [
                "evidence:1".into(),
                "evidence:partition".into(),
            ]
            .into_iter()
            .collect(),
            admitted_sources: ["source-snapshot:archive"].into_iter().collect(),
            evidence_metadata: vec![],
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
        assert_eq!(audit.validate(), Err(ProjectionError::AuditWithoutEvidencePath));
    }

    #[test]
    fn transition_audit_requires_source_snapshot() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            admitted_evidence: ["evidence:partition"].into_iter().collect(),
            admitted_sources: ["source-snapshot:archive"].into_iter().collect(),
        };
        let mut audit = ProjectionAuditV1::for_transition(&transition(), &frontier);
        audit.source_snapshots.clear();
        assert_eq!(audit.validate(), Err(ProjectionError::IrreversibleTransition));
    }

    #[test]
    fn transition_audit_cannot_hide_spatial_evidence() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            admitted_evidence: ["evidence:partition"].into_iter().collect(),
            admitted_sources: ["source-snapshot:archive"].into_iter().collect(),
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
    fn year_interval_uses_inclusive_bounds() {
        let interval = YearInterval { from: Some(-300), to: Some(-200) };
        assert!(interval.contains(-300));
        assert!(interval.contains(-200));
        assert!(!interval.contains(-199));
    }
}
