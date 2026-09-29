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

/// Community participation/recognition is an independent dimension of a
/// cultural record. It must never upgrade epistemic qualification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityRecognitionV1 {
    pub community: CommunityId,
    pub practice: PracticeId,
    pub recognized_from: Option<YearInterval>,
    pub recognized_to: Option<YearInterval>,
    pub maintained: bool,
    pub transmitted: bool,
    pub evidence_refs: Vec<EvidenceId>,
}

impl CommunityRecognitionV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if !self.community.is_valid() || !self.practice.is_valid() {
            return Err(ProjectionError::EmptyIdentifier);
        }
        if self.recognized_from.is_some_and(|v| !v.is_valid())
            || self.recognized_to.is_some_and(|v| !v.is_valid())
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
    pub fn is_frontier_safe(&self, frontier: &EvidenceFrontierV1) -> bool {
        self.validate().is_ok()
            && self.evidence_frontier == frontier.frontier_id
            && frontier.admits(&self.evidence_refs[0])
            && self.evidence_refs.iter().all(|id| frontier.admits(id))
            && self.source_snapshots.iter().all(|id| frontier.admits_source(id))
            && self
                .community_recognition
                .iter()
                .all(|recognition| recognition.evidence_refs.iter().all(|id| frontier.admits(id)))
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
    pub fn from_transmission(
        transmission: &CulturalTransmissionV1,
        frontier: &EvidenceFrontierV1,
    ) -> Option<Self> {
        if !transmission.is_frontier_safe(frontier) {
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
    use crate::civilizational::SourceSnapshotTemporalMetadataV1;

    fn frontier() -> EvidenceFrontierV1 {
        EvidenceFrontierV1 {
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
            evidence_metadata: vec![],
            source_metadata: vec![],
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
                recognized_from: Some(YearInterval { from: Some(1940), to: None }),
                recognized_to: None,
                maintained: true,
                transmitted: true,
                evidence_refs: vec!["e:recognition".into()],
            }],
            access_policy: AccessPolicyV1::CommunityRestricted,
            evidence_frontier: "frontier:1950".into(),
        }
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
    fn frontier_blocks_late_transmission_evidence() {
        let frontier = frontier();
        let mut value = transmission();
        assert!(value.is_frontier_safe(&frontier));

        value.evidence_refs.push("e:later".into());
        assert!(!value.is_frontier_safe(&frontier));
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
        assert!(!transmission().is_frontier_safe(&frontier));
    }

    #[test]
    fn community_recognition_does_not_upgrade_qualification() {
        let frontier = frontier();
        let mut value = transmission();
        value.qualification = QualificationStatus::Speculative;

        let admission = CulturalProjectionAdmissionV1::from_transmission(&value, &frontier).unwrap();

        assert_eq!(admission.qualification, QualificationStatus::Speculative);
        assert_eq!(admission.access_policy, AccessPolicyV1::CommunityRestricted);
    }

    #[test]
    fn access_policy_is_not_an_epistemic_status() {
        let frontier = frontier();
        let mut value = transmission();
        value.access_policy = AccessPolicyV1::Public;
        let public_admission =
            CulturalProjectionAdmissionV1::from_transmission(&value, &frontier).unwrap();

        value.access_policy = AccessPolicyV1::SacredOrRestricted;
        let restricted_admission =
            CulturalProjectionAdmissionV1::from_transmission(&value, &frontier).unwrap();

        assert_eq!(public_admission.qualification, restricted_admission.qualification);
    }

    #[test]
    fn admission_is_explicit_and_reversible() {
        let frontier = frontier();
        let admission =
            CulturalProjectionAdmissionV1::from_transmission(&transmission(), &frontier).unwrap();

        assert!(admission.validate().is_ok());
        assert_eq!(admission.claim_ref, "claim:1".into());
        assert_eq!(admission.transmission_id, "transmission:1".into());
        assert_eq!(admission.evidence_frontier, "frontier:1950".into());
    }
}
