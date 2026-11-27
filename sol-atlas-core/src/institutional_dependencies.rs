// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-bound institutional relationship projection for Sol Atlas.
//!
//! This module is a projection adapter, not an institutional graph authority.
//! It only projects explicit upstream relationship records; it never infers
//! relationships from co-location, visual proximity, shared categories or
//! network metrics. Evidence semantics are reused from the shared evidence
//! envelope rather than redefined here.

use crate::evidence::{
    EpistemicClass, EvidenceError, EvidenceMetadata, Missingness, SpatialScope, VerificationStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Person,
    Population,
    Community,
    Organization,
    Institution,
    InfrastructureAsset,
    Resource,
    Service,
    Route,
    Territory,
    Verifier,
    Authority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub entity_id: String,
    pub entity_kind: EntityKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    DependsOn,
    ControlsAccessTo,
    Operates,
    OwnsOrStewards,
    Funds,
    Supplies,
    SubstitutesFor,
    CanVeto,
    SetsAgendaFor,
    VerifiesOrQualifies,
    DelegatesTo,
    RoutesThrough,
    BearsCostOf,
    BenefitsFrom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipScope {
    /// Stable domain identifier, e.g. "water_service" or "electricity".
    pub domain_id: String,
    /// Optional narrower jurisdiction, service, resource or contract scope.
    pub scope_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureClass {
    /// This relationship may be included in a public projection.
    Public,
    /// The relationship has already been safely aggregated; no person-level
    /// endpoint or protected detail is present in this projection record.
    AggregatedOnly,
    /// Relationship details are restricted to an authorized audience.
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstitutionalRelationship {
    pub relationship_id: String,
    pub subject: EntityRef,
    pub object: EntityRef,
    pub relationship_type: RelationshipType,
    pub scope: RelationshipScope,
    /// Time at which this exact relationship claim entered the evidence
    /// system. This is distinct from event validity, source observation time
    /// and source publication time.
    pub known_by_unix_seconds: Option<i64>,
    /// Reuses the common evidence contract. It remains the source of
    /// epistemic class, missingness, temporal/spatial extent and verification.
    pub evidence: EvidenceMetadata,
    pub disclosure: DisclosureClass,
}

impl InstitutionalRelationship {
    /// Structural validation only. This does not establish that the claimed
    /// relationship is true, legally effective or operationally current.
    pub fn validate(&self) -> Result<(), RelationshipError> {
        if self.relationship_id.trim().is_empty() {
            return Err(RelationshipError::EmptyField("relationship_id"));
        }
        validate_entity(&self.subject, "subject")?;
        validate_entity(&self.object, "object")?;
        if self.subject == self.object {
            return Err(RelationshipError::SelfRelationship);
        }
        if self.scope.domain_id.trim().is_empty() {
            return Err(RelationshipError::EmptyField("scope.domain_id"));
        }
        if (self.subject.entity_kind == EntityKind::Person
            || self.object.entity_kind == EntityKind::Person)
            && self.disclosure != DisclosureClass::Restricted
        {
            return Err(RelationshipError::PersonEndpointNotRestricted);
        }
        self.evidence
            .validate()
            .map_err(RelationshipError::InvalidEvidence)?;
        Ok(())
    }
}

fn validate_entity(entity: &EntityRef, field: &'static str) -> Result<(), RelationshipError> {
    if entity.entity_id.trim().is_empty() {
        Err(RelationshipError::EmptyField(field))
    } else {
        Ok(())
    }
}

/// Audience classification supplied by the calling application.
///
/// The caller must establish Mycelix/host authorization before selecting
/// `Authorized`. This enum is a projection input, not an access-control
/// credential or security boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionAudience {
    Public,
    Authorized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedRelationship {
    pub relationship_id: String,
    pub subject: EntityRef,
    pub object: EntityRef,
    pub relationship_type: RelationshipType,
    pub scope: RelationshipScope,
    pub known_by_unix_seconds: Option<i64>,
    /// Preserve the complete evidence envelope so source references,
    /// derivation lineage, limitations, rights and alternative qualifications
    /// remain inspectable after projection.
    pub evidence: EvidenceMetadata,
    pub disclosure: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipProjection {
    pub visible: Vec<ProjectedRelationship>,
    pub excluded_future_evidence: Vec<String>,
    pub unknown_known_by_time: Vec<String>,
    pub outside_validity_interval: Vec<String>,
}

impl RelationshipProjection {
    pub fn empty() -> Self {
        Self {
            visible: Vec::new(),
            excluded_future_evidence: Vec::new(),
            unknown_known_by_time: Vec::new(),
            outside_validity_interval: Vec::new(),
        }
    }
}

/// Project explicitly supplied relationships at a selected valid-time and
/// evidence frontier.
///
/// The function does not calculate centrality, infer dependencies or create
/// edges from spatial proximity. Missing known-by time is preserved in its
/// own unresolved bucket and is never silently interpreted as "no edge".
pub fn project_relationships(
    relationships: &[InstitutionalRelationship],
    valid_at_unix_seconds: i64,
    evidence_frontier_unix_seconds: i64,
    audience: ProjectionAudience,
) -> Result<RelationshipProjection, RelationshipError> {
    let mut projection = RelationshipProjection::empty();

    for relationship in relationships {
        // Restricted relationship records are opaque to public projections:
        // do not validate, classify, or expose their identifiers or metadata.
        if relationship.disclosure == DisclosureClass::Restricted
            && audience != ProjectionAudience::Authorized
        {
            continue;
        }

        relationship.validate()?;

        let temporal = relationship.evidence.temporal;
        let outside_validity = temporal
            .valid_from_unix_seconds
            .is_some_and(|start| start > valid_at_unix_seconds)
            || temporal
                .valid_until_unix_seconds
                .is_some_and(|end| end < valid_at_unix_seconds);

        if outside_validity {
            projection
                .outside_validity_interval
                .push(relationship.relationship_id.clone());
            continue;
        }

        let Some(known_by) = relationship.known_by_unix_seconds else {
            projection
                .unknown_known_by_time
                .push(relationship.relationship_id.clone());
            continue;
        };

        if known_by > evidence_frontier_unix_seconds {
            projection
                .excluded_future_evidence
                .push(relationship.relationship_id.clone());
            continue;
        }

        projection.visible.push(ProjectedRelationship {
            relationship_id: relationship.relationship_id.clone(),
            subject: relationship.subject.clone(),
            object: relationship.object.clone(),
            relationship_type: relationship.relationship_type,
            scope: relationship.scope.clone(),
            known_by_unix_seconds: relationship.known_by_unix_seconds,
            evidence: relationship.evidence.clone(),
            disclosure: relationship.disclosure,
        });
    }

    Ok(projection)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipError {
    EmptyField(&'static str),
    SelfRelationship,
    PersonEndpointNotRestricted,
    InvalidEvidence(EvidenceError),
}

impl std::fmt::Display for RelationshipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::SelfRelationship => write!(f, "relationship subject and object must differ"),
            Self::PersonEndpointNotRestricted => {
                write!(f, "relationships with person endpoints require restricted disclosure")
            }
            Self::InvalidEvidence(error) => write!(f, "invalid relationship evidence: {error}"),
        }
    }
}

impl std::error::Error for RelationshipError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{
        EpistemicClass, EvidenceMetadata, Measurement, Missingness, SourceRef, SpatialScope,
        TemporalExtent, VerificationStatus,
    };

    const VALID_AT: i64 = 1_700_000_500;
    const FRONTIER: i64 = 1_700_001_000;

    fn evidence(class: EpistemicClass) -> EvidenceMetadata {
        let is_present = class == EpistemicClass::ObservedMeasurement
            || class == EpistemicClass::ScenarioAssumption;
        EvidenceMetadata {
            record_id: "claim-water-provider-01".into(),
            epistemic_class: class,
            missingness: if is_present {
                Missingness::Present
            } else {
                Missingness::Unknown
            },
            temporal: TemporalExtent {
                valid_from_unix_seconds: Some(1_700_000_000),
                valid_until_unix_seconds: Some(1_700_010_000),
                observed_at_unix_seconds: Some(1_700_000_400),
                published_at_unix_seconds: Some(1_700_000_450),
            },
            spatial: SpatialScope::Region {
                region_id: "basin-test-01".into(),
                boundary_source_id: Some("boundary-source-v1".into()),
                crs: "EPSG:4326".into(),
            },
            sources: vec![SourceRef {
                source_id: "public-source-v1".into(),
                locator: "https://example.invalid/source".into(),
                source_version: Some("v1".into()),
                snapshot_digest: Some(format!("sha256:{}", "b".repeat(64))),
                licence_id: Some("CC-BY-4.0".into()),
                attribution: Some("Example source".into()),
                retrieved_at_unix_seconds: Some(1_700_000_450),
            }],
            measurement: if is_present {
                Some(Measurement {
                    quantity: "declared_capacity".into(),
                    value: 10.0,
                    unit: "m3_per_hour".into(),
                    standard_uncertainty: None,
                    method_ref: Some("method-v1".into()),
                    aggregation_window: Some("hour".into()),
                })
            } else {
                None
            },
            model_ref: None,
            scenario_ref: if class == EpistemicClass::ScenarioAssumption {
                Some("synthetic-basin-scenario-v1".into())
            } else {
                None
            },
            hypothesis_ref: None,
            derivation_refs: Vec::new(),
            verifier_refs: Vec::new(),
            rights_and_access: vec!["public".into()],
            limitations: vec!["synthetic fixture unless independently sourced".into()],
            verification_status: VerificationStatus::StructurallyValidated,
        }
    }

    fn relationship(
        id: &str,
        known_by: Option<i64>,
        disclosure: DisclosureClass,
        class: EpistemicClass,
    ) -> InstitutionalRelationship {
        InstitutionalRelationship {
            relationship_id: id.into(),
            subject: EntityRef {
                entity_id: "community-01".into(),
                entity_kind: EntityKind::Community,
            },
            object: EntityRef {
                entity_id: "water-asset-01".into(),
                entity_kind: EntityKind::InfrastructureAsset,
            },
            relationship_type: RelationshipType::DependsOn,
            scope: RelationshipScope {
                domain_id: "water_service".into(),
                scope_id: Some("basin-test-01".into()),
            },
            known_by_unix_seconds: known_by,
            evidence: evidence(class),
            disclosure,
        }
    }

    #[test]
    fn only_explicit_relationships_are_projected() {
        // Two records with the same spatial scope but no relationship claim
        // produce no graph edge. The projection never synthesizes adjacency.
        let result =
            project_relationships(&[], VALID_AT, FRONTIER, ProjectionAudience::Public).unwrap();
        assert!(result.visible.is_empty());
    }

    #[test]
    fn future_known_by_time_is_excluded_at_earlier_frontier() {
        let item = relationship(
            "future-edge",
            Some(FRONTIER + 1),
            DisclosureClass::Public,
            EpistemicClass::CompiledSecondarySource,
        );
        let result =
            project_relationships(&[item], VALID_AT, FRONTIER, ProjectionAudience::Public).unwrap();
        assert!(result.visible.is_empty());
        assert_eq!(result.excluded_future_evidence, vec!["future-edge"]);
    }

    #[test]
    fn missing_known_by_time_is_unresolved_not_absent() {
        let item = relationship(
            "unknown-frontier-edge",
            None,
            DisclosureClass::Public,
            EpistemicClass::CompiledSecondarySource,
        );
        let result =
            project_relationships(&[item], VALID_AT, FRONTIER, ProjectionAudience::Public).unwrap();
        assert!(result.visible.is_empty());
        assert_eq!(result.unknown_known_by_time, vec!["unknown-frontier-edge"]);
    }

    #[test]
    fn validity_interval_is_independent_of_evidence_frontier() {
        let mut item = relationship(
            "not-yet-valid",
            Some(FRONTIER - 1),
            DisclosureClass::Public,
            EpistemicClass::CompiledSecondarySource,
        );
        item.evidence.temporal.valid_from_unix_seconds = Some(VALID_AT + 10);
        let result =
            project_relationships(&[item], VALID_AT, FRONTIER, ProjectionAudience::Public).unwrap();
        assert!(result.visible.is_empty());
        assert_eq!(result.outside_validity_interval, vec!["not-yet-valid"]);
    }

    #[test]
    fn restricted_relationship_is_not_visible_to_public_audience() {
        let item = relationship(
            "restricted-edge",
            Some(FRONTIER - 1),
            DisclosureClass::Restricted,
            EpistemicClass::CompiledSecondarySource,
        );
        let public = project_relationships(
            &[item.clone()],
            VALID_AT,
            FRONTIER,
            ProjectionAudience::Public,
        )
        .unwrap();
        assert!(public.visible.is_empty());
        assert!(public.excluded_future_evidence.is_empty());
        assert!(public.unknown_known_by_time.is_empty());
        assert!(public.outside_validity_interval.is_empty());

        let authorized =
            project_relationships(&[item], VALID_AT, FRONTIER, ProjectionAudience::Authorized)
                .unwrap();
        assert_eq!(authorized.visible.len(), 1);
    }

    #[test]
    fn restricted_relationship_metadata_is_opaque_to_public_projection() {
        let future = relationship(
            "secret-future",
            Some(FRONTIER + 1),
            DisclosureClass::Restricted,
            EpistemicClass::CompiledSecondarySource,
        );
        let unknown = relationship(
            "secret-unknown",
            None,
            DisclosureClass::Restricted,
            EpistemicClass::CompiledSecondarySource,
        );
        let mut outside = relationship(
            "secret-outside",
            Some(FRONTIER - 1),
            DisclosureClass::Restricted,
            EpistemicClass::CompiledSecondarySource,
        );
        outside.evidence.temporal.valid_from_unix_seconds = Some(VALID_AT + 1);

        let mut malformed = relationship(
            "secret-malformed",
            Some(FRONTIER + 1),
            DisclosureClass::Restricted,
            EpistemicClass::CompiledSecondarySource,
        );
        malformed.evidence.temporal.valid_until_unix_seconds = Some(10);

        let result = project_relationships(
            &[future, unknown, outside, malformed],
            VALID_AT,
            FRONTIER,
            ProjectionAudience::Public,
        )
        .unwrap();

        assert!(result.visible.is_empty());
        assert!(result.excluded_future_evidence.is_empty());
        assert!(result.unknown_known_by_time.is_empty());
        assert!(result.outside_validity_interval.is_empty());
    }

    #[test]
    fn person_endpoint_requires_restricted_disclosure() {
        let mut item = relationship(
            "person-edge",
            Some(FRONTIER - 1),
            DisclosureClass::Public,
            EpistemicClass::CompiledSecondarySource,
        );
        item.subject = EntityRef {
            entity_id: "person-001".into(),
            entity_kind: EntityKind::Person,
        };
        assert_eq!(
            item.validate(),
            Err(RelationshipError::PersonEndpointNotRestricted)
        );
    }

    #[test]
    fn scenario_assumption_remains_scenario_in_projected_edge() {
        let item = relationship(
            "scenario-edge",
            Some(FRONTIER - 1),
            DisclosureClass::Public,
            EpistemicClass::ScenarioAssumption,
        );
        let result =
            project_relationships(&[item], VALID_AT, FRONTIER, ProjectionAudience::Public).unwrap();
        assert_eq!(result.visible.len(), 1);
        assert_eq!(
            result.visible[0].evidence.epistemic_class,
            EpistemicClass::ScenarioAssumption
        );
        assert_eq!(result.visible[0].evidence.missingness, Missingness::Present);
        assert_eq!(
            result.visible[0].evidence.record_id,
            "claim-water-provider-01"
        );
        assert_eq!(result.visible[0].evidence.sources.len(), 1);
    }

    #[test]
    fn malformed_evidence_prevents_projection() {
        let mut item = relationship(
            "malformed-edge",
            Some(FRONTIER - 1),
            DisclosureClass::Public,
            EpistemicClass::CompiledSecondarySource,
        );
        item.evidence.temporal.valid_from_unix_seconds = Some(20);
        item.evidence.temporal.valid_until_unix_seconds = Some(10);
        assert!(matches!(
            item.validate(),
            Err(RelationshipError::InvalidEvidence(
                EvidenceError::InvalidTemporalInterval { .. }
            ))
        ));
    }
}
