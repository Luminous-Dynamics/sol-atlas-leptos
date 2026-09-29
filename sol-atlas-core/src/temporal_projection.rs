// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic temporal replay over civilizational projection contracts.
//!
//! This layer decides only whether already-described projection candidates are
//! admissible for a requested map epoch and evidence frontier. It does not
//! infer historical facts, resolve contested interpretations, or upgrade
//! qualification status.

use crate::civilizational::{
    EvidenceFrontierChainV1, EvidenceFrontierId, EvidenceFrontierV1, EvidenceId, HistoricalTransitionV1,
    ProjectionAuditV1, ProjectionError, ProjectionRef, SnapshotId, SourceSnapshotId,
    StateSnapshotV1, TransitionId, YearInterval,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalProjectionRequestV1 {
    pub map_epoch: YearInterval,
    pub evidence_frontier: EvidenceFrontierV1,
}

impl TemporalProjectionRequestV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.map_epoch.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if !self.evidence_frontier.frontier_id.is_valid() {
            return Err(ProjectionError::MissingEvidenceFrontier);
        }
        Ok(())
    }

    /// Strict validation for new/reproducible frontiers. Unlike validate,
    /// this rejects the legacy ID-only admission format and requires a complete
    /// temporal evidence manifest.
    pub fn validate_strict(&self) -> Result<(), ProjectionError> {
        self.validate()?;
        self.evidence_frontier.validate_temporal_manifest_strict()
    }

    /// Validates that this request's selected frontier is the verified leaf of
    /// an explicit append-only frontier lineage.
    pub fn validate_against_frontier_chain(
        &self,
        chain: &EvidenceFrontierChainV1,
    ) -> Result<(), ProjectionError> {
        self.validate()?;
        chain.validate_strict()?;
        if chain
            .current()
            .is_none_or(|frontier| frontier.frontier_id != self.evidence_frontier.frontier_id)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }

    /// Replay candidates into a stable, frontier-safe projection.
    ///
    /// Candidates are validated before filtering so malformed historical data
    /// cannot disappear merely because it falls outside the current map epoch.
    /// Output order is deterministic by stable typed identifiers.
    pub fn project(
        &self,
        snapshots: &[StateSnapshotV1],
        transitions: &[HistoricalTransitionV1],
    ) -> Result<TemporalProjectionSetV1, ProjectionError> {
        self.validate()?;

        for (index, snapshot) in snapshots.iter().enumerate() {
            snapshot.validate().map_err(|_| ProjectionError::InvalidSnapshot)?;
            if snapshots[index + 1..]
                .iter()
                .any(|other| other.snapshot_id == snapshot.snapshot_id)
            {
                return Err(ProjectionError::InvalidSnapshot);
            }
        }
        for (index, transition) in transitions.iter().enumerate() {
            transition
                .validate()
                .map_err(|_| ProjectionError::InvalidTransition)?;
            if transitions[index + 1..]
                .iter()
                .any(|other| other.transition_id == transition.transition_id)
            {
                return Err(ProjectionError::InvalidTransition);
            }
        }
        let mut selected_snapshots = snapshots
            .iter()
            .filter(|snapshot| snapshot.valid_time.overlaps(&self.map_epoch))
            .filter(|snapshot| self.evidence_frontier.admits_snapshot(snapshot))
            .cloned()
            .collect::<Vec<_>>();
        selected_snapshots.sort_by(|a, b| a.snapshot_id.cmp(&b.snapshot_id));

        let mut selected_transitions = transitions
            .iter()
            .filter(|transition| transition.event_time.overlaps(&self.map_epoch))
            .filter(|transition| self.evidence_frontier.admits_transition(transition))
            .cloned()
            .collect::<Vec<_>>();
        selected_transitions.sort_by(|a, b| a.transition_id.cmp(&b.transition_id));

        let mut audits = selected_snapshots
            .iter()
            .map(ProjectionAuditV1::for_snapshot)
            .chain(
                selected_transitions
                    .iter()
                    .map(|transition| ProjectionAuditV1::for_transition(transition, &self.evidence_frontier)),
            )
            .collect::<Vec<_>>();
        audits.sort_by(|a, b| a.projection.cmp(&b.projection));

        let admissions = result_admissions(&selected_snapshots, &selected_transitions, &self.evidence_frontier);

        let result = TemporalProjectionSetV1 {
            map_epoch: self.map_epoch,
            evidence_frontier: self.evidence_frontier.clone(),
            frontier_lineage: vec![self.evidence_frontier.frontier_id.clone()],
            snapshots: selected_snapshots,
            transitions: selected_transitions,
            audits,
            admissions,
        };
        result.validate()?;
        Ok(result)
    }
    /// Strict replay variant that requires a complete temporal evidence manifest.
    pub fn project_strict(
        &self,
        snapshots: &[StateSnapshotV1],
        transitions: &[HistoricalTransitionV1],
    ) -> Result<TemporalProjectionSetV1, ProjectionError> {
        self.validate_strict()?;
        self.project(snapshots, transitions)
    }

    /// Replays against an explicitly verified frontier lineage.
    ///
    /// The returned projection records the ordered frontier IDs used to establish
    /// the selected frontier. The full manifests remain the caller-supplied
    /// verification material; the projection stores only their stable path.
    pub fn project_against_frontier_chain(
        &self,
        chain: &EvidenceFrontierChainV1,
        snapshots: &[StateSnapshotV1],
        transitions: &[HistoricalTransitionV1],
    ) -> Result<TemporalProjectionSetV1, ProjectionError> {
        self.validate_against_frontier_chain(chain)?;
        let mut result = self.project_strict(snapshots, transitions)?;
        result.frontier_lineage = chain
            .frontiers
            .iter()
            .map(|frontier| frontier.frontier_id.clone())
            .collect();
        result.validate_against_frontier_chain(chain)?;
        Ok(result)
    }
}

/// Records the semantic admission decision that made a projection eligible
/// for rendering. Input candidates remain outside this record; the renderer
/// receives only projections that have crossed the requested evidence frontier.
///
/// "Rendered" is intentionally not an epistemic disposition here: rendering is
/// a presentation action, while admission is the core semantic boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionAdmissionV1 {
    pub projection: ProjectionRef,
    pub evidence_frontier: EvidenceFrontierId,
    pub admitted_evidence: Vec<EvidenceId>,
    pub admitted_sources: Vec<SourceSnapshotId>,
}

impl ProjectionAdmissionV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.projection.is_valid()
            || !self.evidence_frontier.is_valid()
            || self.admitted_evidence.iter().any(|id| !id.is_valid())
            || self.admitted_sources.iter().any(|id| !id.is_valid())
        {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.admitted_evidence.is_empty() {
            return Err(ProjectionError::AuditWithoutEvidencePath);
        }
        Ok(())
    }

    pub fn for_snapshot(
        snapshot: &StateSnapshotV1,
        frontier: &EvidenceFrontierV1,
    ) -> Self {
        let admitted_evidence = snapshot
            .evidence_refs
            .iter()
            .chain(
                snapshot
                    .geometries
                    .iter()
                    .flat_map(|geometry| geometry.evidence.iter()),
            )
            .cloned()
            .collect();

        Self {
            projection: ProjectionRef::Snapshot(snapshot.snapshot_id.clone()),
            evidence_frontier: frontier.frontier_id.clone(),
            admitted_evidence,
            admitted_sources: snapshot.source_snapshots.clone(),
        }
    }

    pub fn for_transition(
        transition: &HistoricalTransitionV1,
        frontier: &EvidenceFrontierV1,
    ) -> Self {
        let admitted_evidence = transition
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
            evidence_frontier: frontier.frontier_id.clone(),
            admitted_evidence,
            admitted_sources: transition.source_snapshots.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalProjectionSetV1 {
    pub map_epoch: YearInterval,
    pub evidence_frontier: EvidenceFrontierV1,
    /// Ordered frontier IDs used by chain-aware replay. Legacy replay records
    /// only the selected frontier.
    pub frontier_lineage: Vec<EvidenceFrontierId>,
    pub snapshots: Vec<StateSnapshotV1>,
    pub transitions: Vec<HistoricalTransitionV1>,
    pub audits: Vec<ProjectionAuditV1>,
    pub admissions: Vec<ProjectionAdmissionV1>,
}


fn result_admissions(
    snapshots: &[StateSnapshotV1],
    transitions: &[HistoricalTransitionV1],
    frontier: &EvidenceFrontierV1,
) -> Vec<ProjectionAdmissionV1> {
    let mut admissions = snapshots
        .iter()
        .map(|snapshot| ProjectionAdmissionV1::for_snapshot(snapshot, frontier))
        .chain(
            transitions
                .iter()
                .map(|transition| ProjectionAdmissionV1::for_transition(transition, frontier)),
        )
        .collect::<Vec<_>>();
    admissions.sort_by(|a, b| a.projection.cmp(&b.projection));
    admissions
}

impl TemporalProjectionSetV1 {
    /// Validates the recorded lineage against the full frontier manifests.
    ///
    /// The compact path stored in a projection is not itself cryptographic
    /// proof. This method verifies it against the supplied chain, including
    /// manifest hashes and append-only inheritance.
    pub fn validate_against_frontier_chain(
        &self,
        chain: &EvidenceFrontierChainV1,
    ) -> Result<(), ProjectionError> {
        chain.validate_strict()?;
        let expected_path = chain
            .frontiers
            .iter()
            .map(|frontier| frontier.frontier_id.clone())
            .collect::<Vec<_>>();
        if self.frontier_lineage != expected_path
            || chain
                .current()
                .is_none_or(|frontier| frontier.frontier_id != self.evidence_frontier.frontier_id)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        self.validate()
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.map_epoch.is_valid() {
            return Err(ProjectionError::InvalidTimeInterval);
        }
        if !self.evidence_frontier.frontier_id.is_valid() {
            return Err(ProjectionError::MissingEvidenceFrontier);
        }
        if self.frontier_lineage.is_empty()
            || self.frontier_lineage.last() != Some(&self.evidence_frontier.frontier_id)
            || self.frontier_lineage.iter().any(|id| !id.is_valid())
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }

        let mut previous_snapshot = None;
        for snapshot in &self.snapshots {
            snapshot.validate().map_err(|_| ProjectionError::InvalidSnapshot)?;
            if !snapshot.valid_time.overlaps(&self.map_epoch)
                || !self.evidence_frontier.admits_snapshot(snapshot)
            {
                return Err(ProjectionError::InvalidSnapshot);
            }
            if previous_snapshot.as_ref().is_some_and(|id: &SnapshotId| {
                id >= &snapshot.snapshot_id
            }) {
                return Err(ProjectionError::InvalidSnapshot);
            }
            previous_snapshot = Some(&snapshot.snapshot_id);
        }

        let mut previous_transition = None;
        for transition in &self.transitions {
            transition
                .validate()
                .map_err(|_| ProjectionError::InvalidTransition)?;
            if !transition.event_time.overlaps(&self.map_epoch)
                || !self.evidence_frontier.admits_transition(transition)
            {
                return Err(ProjectionError::InvalidTransition);
            }
            if previous_transition.as_ref().is_some_and(|id: &TransitionId| {
                id >= &transition.transition_id
            }) {
                return Err(ProjectionError::InvalidTransition);
            }
            previous_transition = Some(&transition.transition_id);
        }

        let expected = self.snapshots.len() + self.transitions.len();
        if self.audits.len() != expected || self.admissions.len() != expected {
            return Err(ProjectionError::AuditWithoutEvidencePath);
        }

        let mut previous_projection: Option<&ProjectionRef> = None;
        for audit in &self.audits {
            audit.validate()?;
            if !self.evidence_frontier.admits_audit(audit) {
                return Err(ProjectionError::AuditWithoutEvidencePath);
            }
            match &audit.projection {
                ProjectionRef::Snapshot(id)
                    if !self.snapshots.iter().any(|snapshot| &snapshot.snapshot_id == id) =>
                {
                    return Err(ProjectionError::AuditWithoutEvidencePath);
                }
                ProjectionRef::Transition(id)
                    if !self.transitions.iter().any(|transition| &transition.transition_id == id) =>
                {
                    return Err(ProjectionError::AuditWithoutEvidencePath);
                }
                _ => {}
            }
            if previous_projection.is_some_and(|projection| projection >= &audit.projection) {
                return Err(ProjectionError::AuditWithoutEvidencePath);
            }
            let expected_audit = match &audit.projection {
                ProjectionRef::Snapshot(id) => self
                    .snapshots
                    .iter()
                    .find(|snapshot| &snapshot.snapshot_id == id)
                    .map(ProjectionAuditV1::for_snapshot),
                ProjectionRef::Transition(id) => self
                    .transitions
                    .iter()
                    .find(|transition| &transition.transition_id == id)
                    .map(|transition| {
                        ProjectionAuditV1::for_transition(transition, &self.evidence_frontier)
                    }),
            };

            if expected_audit.as_ref() != Some(audit) {
                return Err(ProjectionError::AuditWithoutEvidencePath);
            }

            previous_projection = Some(&audit.projection);
        }

        let mut previous_admission: Option<&ProjectionRef> = None;
        for admission in &self.admissions {
            admission.validate()?;
            if admission.evidence_frontier != self.evidence_frontier.frontier_id {
                return Err(ProjectionError::AuditWithoutEvidencePath);
            }
            let expected_admission = match &admission.projection {
                ProjectionRef::Snapshot(id) => self
                    .snapshots
                    .iter()
                    .find(|snapshot| &snapshot.snapshot_id == id)
                    .filter(|snapshot| self.evidence_frontier.admits_snapshot_evidence(snapshot))
                    .map(|snapshot| ProjectionAdmissionV1::for_snapshot(snapshot, &self.evidence_frontier)),
                ProjectionRef::Transition(id) => self
                    .transitions
                    .iter()
                    .find(|transition| &transition.transition_id == id)
                    .filter(|transition| self.evidence_frontier.admits_transition(transition))
                    .map(|transition| ProjectionAdmissionV1::for_transition(transition, &self.evidence_frontier)),
            };

            if expected_admission.as_ref() != Some(admission) {
                return Err(ProjectionError::AuditWithoutEvidencePath);
            }
            if previous_admission.is_some_and(|projection| projection >= &admission.projection) {
                return Err(ProjectionError::AuditWithoutEvidencePath);
            }
            previous_admission = Some(&admission.projection);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        EvidenceTemporalMetadataV1, GeometryProjection, QualificationStatus, QualificationSummary, SpatialSemantics,
        TransitionClass,
    };
    use std::collections::BTreeSet;

    fn geometry(evidence: &str) -> GeometryProjection {
        GeometryProjection {
            geometry_ref: "geom:1".into(),
            semantics: SpatialSemantics::AdministrativeBoundary,
            exact: true,
            evidence: vec![evidence.into()],
        }
    }

    fn snapshot(id: &str, interval: YearInterval, frontier: &str, evidence: &str) -> StateSnapshotV1 {
        StateSnapshotV1 {
            entity_id: format!("state:{id}").as_str().into(),
            snapshot_id: id.into(),
            valid_time: interval,
            geometries: vec![geometry(evidence)],
            institution_refs: vec![],
            constitutional_refs: vec![],
            relation_refs: vec!["claim:state".into()],
            evidence_frontier: frontier.into(),
            qualification: QualificationSummary {
                status: QualificationStatus::Supported,
                assessment: None,
                claim_refs: vec!["claim:qualification".into()],
                unresolved: vec![],
                contested: false,
            },
        }
    }

    fn transition(id: &str, interval: YearInterval, evidence: &str) -> HistoricalTransitionV1 {
        HistoricalTransitionV1 {
            transition_id: id.into(),
            event_time: interval,
            classes: [TransitionClass::Formation].into_iter().collect::<BTreeSet<_>>(),
            source_entities: vec!["polity:old".into()],
            target_entities: vec!["state:new".into()],
            spatial_scope: vec![],
            mechanism: None,
            claim_refs: vec!["claim:transition".into()],
            evidence_refs: vec![evidence.into()],
            source_snapshots: vec!["source:archive".into()],
            competing_hypotheses: vec![],
            assessment: None,
            qualification: QualificationStatus::Supported,
            uncertainty: None,
        }
    }

    fn frontier() -> EvidenceFrontierV1 {
        EvidenceFrontierV1 {
            frontier_id: "frontier:1949".into(),
            known_by_year: 1949,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:old", "e:transition"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:archive"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![],
            source_metadata: vec![],
        }
    }

    #[test]
    fn chain_aware_replay_records_verified_frontier_lineage() {
        let mut root = frontier();
        root.frontier_id = "frontier:1940".into();
        root.known_by_year = 1940;
        root.parent_frontier = None;
        root.admitted_evidence = ["e:old"].into_iter().map(Into::into).collect();
        root.admitted_sources = ["source:archive"].into_iter().map(Into::into).collect();
        root.evidence_metadata = vec![EvidenceTemporalMetadataV1 {
            evidence_id: "e:old".into(),
            source_snapshot: "source:archive".into(),
            artifact_time: None,
            publication_time: Some(1939),
            capture_time: None,
            available_by: 1939,
            validity_time: None,
        }];
        root.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1939),
            capture_time: None,
            available_by: 1939,
        }];
        root.recompute_manifest_hash().unwrap();

        let mut leaf = root.clone();
        leaf.frontier_id = "frontier:1949".into();
        leaf.known_by_year = 1949;
        leaf.parent_frontier = Some(root.frontier_id.clone());
        leaf.evidence_metadata.push(EvidenceTemporalMetadataV1 {
            evidence_id: "e:transition".into(),
            source_snapshot: "source:archive".into(),
            artifact_time: None,
            publication_time: Some(1945),
            capture_time: None,
            available_by: 1945,
            validity_time: None,
        });
        leaf.admitted_evidence.insert("e:transition".into());
        leaf.recompute_manifest_hash().unwrap();

        let chain = EvidenceFrontierChainV1 { frontiers: vec![root, leaf.clone()] };
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1945), to: Some(1947) },
            evidence_frontier: leaf,
        };
        let result = request
            .project_against_frontier_chain(&chain, &[snapshot(
                "snapshot:a",
                YearInterval { from: Some(1945), to: Some(1947) },
                "frontier:1949",
                "e:old",
            )], &[])
            .unwrap();

        assert_eq!(
            result.frontier_lineage,
            vec!["frontier:1940".into(), "frontier:1949".into()]
        );
        assert_eq!(result.validate(), Ok(()));
        assert_eq!(result.validate_against_frontier_chain(&chain), Ok(()));

        let mut tampered = result.clone();
        tampered.frontier_lineage[0] = "frontier:forged".into();
        assert_eq!(
            tampered.validate_against_frontier_chain(&chain),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn chain_aware_replay_rejects_non_leaf_selected_frontier() {
        let mut root = frontier();
        root.frontier_id = "frontier:1940".into();
        root.known_by_year = 1940;
        root.admitted_evidence = ["e:old"].into_iter().map(Into::into).collect();
        root.admitted_sources = ["source:archive"].into_iter().map(Into::into).collect();
        root.evidence_metadata = vec![EvidenceTemporalMetadataV1 {
            evidence_id: "e:old".into(),
            source_snapshot: "source:archive".into(),
            artifact_time: None,
            publication_time: Some(1939),
            capture_time: None,
            available_by: 1939,
            validity_time: None,
        }];
        root.recompute_manifest_hash().unwrap();

        let mut leaf = root.clone();
        leaf.frontier_id = "frontier:1949".into();
        leaf.known_by_year = 1949;
        leaf.parent_frontier = Some(root.frontier_id.clone());
        leaf.recompute_manifest_hash().unwrap();

        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1945), to: Some(1947) },
            evidence_frontier: root,
        };
        assert_eq!(
            request.validate_against_frontier_chain(&EvidenceFrontierChainV1 {
                frontiers: vec![request.evidence_frontier.clone(), leaf],
            }),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn strict_request_rejects_legacy_frontier_manifest() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        assert_eq!(
            request.validate_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn strict_request_accepts_complete_temporal_manifest() {
        let mut request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        request.evidence_frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        request.evidence_frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1940),
            capture_time: None,
            available_by: 1940,
        }];
        request.evidence_frontier.recompute_manifest_hash().unwrap();
        assert_eq!(request.validate_strict(), Ok(()));
        let result = request
            .project_strict(
                &[snapshot(
                    "snapshot:a",
                    YearInterval { from: Some(1945), to: Some(1947) },
                    "frontier:1949",
                    "e:old",
                )],
                &[],
            )
            .unwrap();
        assert_eq!(result.snapshots.len(), 1);
    }

    #[test]
    fn strict_manifest_rejects_missing_source_metadata() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.recompute_manifest_hash().unwrap();
        assert_eq!(
            frontier.validate_temporal_manifest_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn strict_manifest_rejects_late_source_metadata() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1951),
            capture_time: None,
            available_by: 1951,
        }];
        frontier.recompute_manifest_hash().unwrap();
        assert_eq!(
            frontier.validate_temporal_manifest_strict(),
            Err(ProjectionError::InvalidEvidenceTemporalMetadata)
        );
    }

    #[test]
    fn strict_manifest_accepts_complete_source_metadata() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1940),
            capture_time: None,
            available_by: 1940,
        }];
        frontier.recompute_manifest_hash().unwrap();
        assert_eq!(frontier.validate_temporal_manifest_strict(), Ok(()));
    }

    #[test]
    fn strict_chain_rejects_incomplete_source_metadata() {
        let mut root = frontier();
        root.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1939),
                capture_time: None,
                available_by: 1939,
                validity_time: None,
            },
        ];
        root.recompute_manifest_hash().unwrap();
        assert_eq!(
            (EvidenceFrontierChainV1 { frontiers: vec![root] }).validate_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn replay_filters_by_epoch_and_frontier() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        let snapshots = vec![
            snapshot("snapshot:b", YearInterval { from: Some(1945), to: Some(1947) }, "frontier:1949", "e:old"),
            snapshot("snapshot:a", YearInterval { from: Some(1930), to: Some(1939) }, "frontier:1949", "e:old"),
            snapshot("snapshot:c", YearInterval { from: Some(1945), to: Some(1947) }, "frontier:later", "e:old"),
        ];
        let transitions = vec![
            transition("transition:b", YearInterval { from: Some(1947), to: Some(1947) }, "e:transition"),
            transition("transition:a", YearInterval { from: Some(1900), to: Some(1901) }, "e:transition"),
        ];

        let result = request.project(&snapshots, &transitions).unwrap();
        assert_eq!(result.snapshots.iter().map(|v| &v.snapshot_id).collect::<Vec<_>>(), vec![&"snapshot:b".into()]);
        assert_eq!(result.transitions.iter().map(|v| &v.transition_id).collect::<Vec<_>>(), vec![&"transition:b".into()]);
        assert_eq!(result.audits.len(), 2);
        assert_eq!(result.admissions.len(), 2);
        assert_eq!(result.audits[0].projection, ProjectionRef::Snapshot("snapshot:b".into()));
        assert_eq!(result.audits[1].projection, ProjectionRef::Transition("transition:b".into()));
    }

    #[test]
    fn replay_rejects_duplicate_candidate_ids() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        let duplicate = snapshot(
            "snapshot:a",
            YearInterval { from: Some(1945), to: Some(1947) },
            "frontier:1949",
            "e:old",
        );
        assert_eq!(
            request.project(&[duplicate.clone(), duplicate], &[]).unwrap_err(),
            ProjectionError::InvalidSnapshot
        );

        let duplicate = transition(
            "transition:a",
            YearInterval { from: Some(1947), to: Some(1947) },
            "e:transition",
        );
        assert_eq!(
            request.project(&[], &[duplicate.clone(), duplicate]).unwrap_err(),
            ProjectionError::InvalidTransition
        );
    }

    #[test]
    fn replay_is_deterministically_sorted() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1900), to: Some(2000) },
            evidence_frontier: frontier(),
        };
        let snapshots = vec![
            snapshot("snapshot:z", YearInterval { from: Some(1950), to: Some(1951) }, "frontier:1949", "e:old"),
            snapshot("snapshot:a", YearInterval { from: Some(1950), to: Some(1951) }, "frontier:1949", "e:old"),
        ];
        let result = request.project(&snapshots, &[]).unwrap();
        assert_eq!(result.snapshots[0].snapshot_id, "snapshot:a".into());
        assert_eq!(result.snapshots[1].snapshot_id, "snapshot:z".into());
    }

    #[test]
    fn replay_rejects_malformed_candidates_even_outside_epoch() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(2000), to: Some(2001) },
            evidence_frontier: frontier(),
        };
        let mut malformed = snapshot(
            "snapshot:bad",
            YearInterval { from: Some(1800), to: Some(1799) },
            "frontier:1949",
            "e:old",
        );
        malformed.geometries[0].evidence.clear();

        let error = request.project(&[malformed], &[]).unwrap_err();
        assert_eq!(error, ProjectionError::InvalidSnapshot);
    }

    #[test]
    fn audit_must_match_the_projected_object_exactly() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        let snapshot = snapshot(
            "snapshot:a",
            YearInterval { from: Some(1945), to: Some(1947) },
            "frontier:1949",
            "e:old",
        );
        let mut result = request.project(&[snapshot], &[]).unwrap();
        result.audits[0].claim_refs.clear();
        assert_eq!(result.validate(), Err(ProjectionError::AuditWithoutEvidencePath));
    }

    #[test]
    fn admission_cannot_be_tampered_without_failing_validation() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        let snapshot = snapshot(
            "snapshot:a",
            YearInterval { from: Some(1945), to: Some(1947) },
            "frontier:1949",
            "e:old",
        );
        let mut result = request.project(&[snapshot], &[]).unwrap();
        result.admissions[0].admitted_evidence.clear();
        assert_eq!(result.validate(), Err(ProjectionError::AuditWithoutEvidencePath));
    }

    #[test]
    fn audit_must_reference_an_included_projection() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(1940), to: Some(1950) },
            evidence_frontier: frontier(),
        };
        let snapshot = snapshot(
            "snapshot:a",
            YearInterval { from: Some(1945), to: Some(1947) },
            "frontier:1949",
            "e:old",
        );
        let mut result = request.project(&[snapshot], &[]).unwrap();
        result.audits[0].projection = ProjectionRef::Snapshot("snapshot:missing".into());
        assert_eq!(result.validate(), Err(ProjectionError::AuditWithoutEvidencePath));
    }

    #[test]
    fn invalid_intervals_do_not_overlap() {
        let invalid = YearInterval { from: Some(10), to: Some(9) };
        let valid = YearInterval { from: Some(9), to: Some(10) };
        assert!(!invalid.overlaps(&valid));
        assert!(!valid.overlaps(&invalid));
    }

    #[test]
    fn open_intervals_overlap_as_unbounded_ranges() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval { from: Some(2000), to: Some(2001) },
            evidence_frontier: frontier(),
        };
        let open_snapshot = snapshot(
            "snapshot:open",
            YearInterval { from: None, to: Some(2000) },
            "frontier:1949",
            "e:old",
        );
        assert_eq!(request.project(&[open_snapshot], &[]).unwrap().snapshots.len(), 1);
    }
}
