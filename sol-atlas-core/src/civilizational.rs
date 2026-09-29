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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// A projection can be reconstructed only from evidence admitted by this
/// frontier; callers must filter derived assessments by the same boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFrontierV1 {
    pub frontier_id: EvidenceFrontierId,
    pub known_by_year: i32,
    pub admitted_evidence: BTreeSet<EvidenceId>,
    pub admitted_sources: BTreeSet<SourceSnapshotId>,
}

impl EvidenceFrontierV1 {
    pub fn admits(&self, evidence: &EvidenceId) -> bool {
        self.admitted_evidence.contains(evidence)
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
            relation_refs: vec!["claim:1".into()],
            evidence_frontier: "frontier:1949".into(),
            qualification: QualificationSummary {
                status: QualificationStatus::Supported,
                assessment: Some("assessment:1".into()),
                claim_refs: vec!["claim:1".into()],
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
    fn frontier_excludes_later_evidence_unless_admitted() {
        let frontier = EvidenceFrontierV1 {
            frontier_id: "frontier:1900".into(),
            known_by_year: 1900,
            admitted_evidence: ["evidence:old".into()].into_iter().collect(),
            admitted_sources: ["source:old".into()].into_iter().collect(),
        };
        assert!(frontier.admits(&"evidence:old".into()));
        assert!(!frontier.admits(&"evidence:discovered-later".into()));
    }

    #[test]
    fn year_interval_uses_inclusive_bounds() {
        let interval = YearInterval { from: Some(-300), to: Some(-200) };
        assert!(interval.contains(-300));
        assert!(interval.contains(-200));
        assert!(!interval.contains(-199));
    }
}
