// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Typed, temporal dependency relationships for spatial-economic intelligence.
//!
//! A relationship is a model claim supported by evidence records; it is not
//! automatically a fact. Reachability is structural impact propagation, not a
//! probability, causal estimate, or prediction of real-world loss.

use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Relationship vocabulary. Only the first four variants mean that the subject
/// depends on the object and are traversed by dependency-impact queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    DependsOn,
    SuppliedBy,
    ServedBy,
    RoutesThrough,
    LocatedAt,
    Produces,
    Transports,
    CommitsTo,
    DeliversTo,
    OwnedBy,
}

impl RelationKind {
    pub const fn is_dependency_edge(self) -> bool {
        matches!(
            self,
            Self::DependsOn | Self::SuppliedBy | Self::ServedBy | Self::RoutesThrough
        )
    }
}

/// A typed, time-bounded relationship between stable entity identifiers.
///
/// Direction matters: for `pump supplied_by feeder`, the pump is the dependent
/// subject and the feeder is the prerequisite object. Validity intervals are
/// UTC timestamps in canonical `YYYY-MM-DDTHH:MM:SSZ` form; intervals are
/// half-open [valid_from, valid_until). At least one supporting evidence ID is
/// required, but the presence of evidence is not a truth guarantee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialRelation {
    pub id: String,
    pub subject_entity_id: String,
    pub kind: RelationKind,
    pub object_entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphValidationError {
    EmptyRelationId,
    EmptySubjectEntity,
    EmptyObjectEntity,
    SelfRelation,
    DuplicateRelationId,
    MissingSupportingEvidence,
    EmptySupportingEvidenceId,
    DuplicateSupportingEvidenceId,
    UnknownEvidenceReference,
    InvalidValidFrom,
    InvalidValidUntil,
    InvalidValidityWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyQueryError {
    InvalidAsOfTimestamp,
    InvalidRelationValidity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyImpact {
    pub entity_id: String,
    /// Number of dependency edges from the requested prerequisite.
    pub distance: usize,
    /// Deterministic shortest path chosen by sorted relation IDs. This is one
    /// explanatory path, not a claim that other paths do not exist.
    pub relation_path_ids: Vec<String>,
    /// Evidence references aligned one-to-one with relation_path_ids.
    pub supporting_evidence_by_relation: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub relations: Vec<SpatialRelation>,
}

impl SpatialRelation {
    fn is_valid_at(&self, at: &str) -> bool {
        self.valid_from.as_deref().is_none_or(|start| start <= at)
            && self.valid_until.as_deref().is_none_or(|end| at < end)
    }
}

impl DependencyGraph {
    /// Validate shape, unique relationship IDs, time profile and evidence links.
    ///
    /// Call this before running queries. Valid references establish traceability
    /// only; the caller must separately assess evidence state, independence,
    /// rights, provenance and domain-specific meaning.
    pub fn validate(
        &self,
        evidence_records: &[EvidenceRecord],
    ) -> Result<(), Vec<GraphValidationError>> {
        let evidence_ids: BTreeSet<&str> =
            evidence_records.iter().map(|record| record.id.as_str()).collect();
        let mut relation_ids = BTreeSet::new();
        let mut errors = Vec::new();

        for relation in &self.relations {
            if relation.id.trim().is_empty() {
                errors.push(GraphValidationError::EmptyRelationId);
            }
            if !relation_ids.insert(relation.id.as_str()) {
                errors.push(GraphValidationError::DuplicateRelationId);
            }
            if relation.subject_entity_id.trim().is_empty() {
                errors.push(GraphValidationError::EmptySubjectEntity);
            }
            if relation.object_entity_id.trim().is_empty() {
                errors.push(GraphValidationError::EmptyObjectEntity);
            }
            if !relation.subject_entity_id.trim().is_empty()
                && relation.subject_entity_id == relation.object_entity_id
            {
                errors.push(GraphValidationError::SelfRelation);
            }
            if relation.supporting_evidence_ids.is_empty() {
                errors.push(GraphValidationError::MissingSupportingEvidence);
            }

            let mut relation_evidence_ids = BTreeSet::new();
            for evidence_id in &relation.supporting_evidence_ids {
                if evidence_id.trim().is_empty() {
                    errors.push(GraphValidationError::EmptySupportingEvidenceId);
                }
                if !relation_evidence_ids.insert(evidence_id.as_str()) {
                    errors.push(GraphValidationError::DuplicateSupportingEvidenceId);
                }
                if !evidence_ids.contains(evidence_id.as_str()) {
                    errors.push(GraphValidationError::UnknownEvidenceReference);
                }
            }

            if let Some(start) = relation.valid_from.as_deref() {
                if !is_canonical_utc_timestamp(start) {
                    errors.push(GraphValidationError::InvalidValidFrom);
                }
            }
            if let Some(end) = relation.valid_until.as_deref() {
                if !is_canonical_utc_timestamp(end) {
                    errors.push(GraphValidationError::InvalidValidUntil);
                }
            }
            if let (Some(start), Some(end)) = (
                relation.valid_from.as_deref(),
                relation.valid_until.as_deref(),
            ) {
                if is_canonical_utc_timestamp(start)
                    && is_canonical_utc_timestamp(end)
                    && start >= end
                {
                    errors.push(GraphValidationError::InvalidValidityWindow);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Find transitive dependents of an entity as of a canonical UTC timestamp.
    ///
    /// An edge `A supplied_by B` means A depends on B, so a query rooted at B
    /// finds A. Only dependency kinds are traversed; semantic relations like
    /// `located_at` or `owned_by` are not treated as failure propagation.
    ///
    /// The result uses breadth-first traversal, stable relation-ID ordering,
    /// cycle protection, shortest explanatory paths, and per-edge evidence IDs.
    /// It does not estimate probability, severity, physical distance or causal
    /// impact. Validate the graph against evidence records before using it.
    pub fn affected_dependents_at(
        &self,
        prerequisite_entity_id: &str,
        at: &str,
    ) -> Result<Vec<DependencyImpact>, DependencyQueryError> {
        if !is_canonical_utc_timestamp(at) {
            return Err(DependencyQueryError::InvalidAsOfTimestamp);
        }

        let mut adjacency: BTreeMap<String, Vec<&SpatialRelation>> = BTreeMap::new();
        for relation in &self.relations {
            if let Some(start) = relation.valid_from.as_deref() {
                if !is_canonical_utc_timestamp(start) {
                    return Err(DependencyQueryError::InvalidRelationValidity);
                }
            }
            if let Some(end) = relation.valid_until.as_deref() {
                if !is_canonical_utc_timestamp(end) {
                    return Err(DependencyQueryError::InvalidRelationValidity);
                }
            }
            if let (Some(start), Some(end)) = (
                relation.valid_from.as_deref(),
                relation.valid_until.as_deref(),
            ) {
                if start >= end {
                    return Err(DependencyQueryError::InvalidRelationValidity);
                }
            }

            if relation.kind.is_dependency_edge() && relation.is_valid_at(at) {
                adjacency
                    .entry(relation.object_entity_id.clone())
                    .or_default()
                    .push(relation);
            }
        }
        for edges in adjacency.values_mut() {
            edges.sort_by(|left, right| left.id.cmp(&right.id));
        }

        let mut visited = BTreeSet::new();
        visited.insert(prerequisite_entity_id.to_owned());
        let mut queue: VecDeque<(String, Vec<String>, Vec<Vec<String>>)> = VecDeque::new();
        queue.push_back((prerequisite_entity_id.to_owned(), Vec::new(), Vec::new()));
        let mut impacts = Vec::new();

        while let Some((current, path, evidence_path)) = queue.pop_front() {
            if let Some(edges) = adjacency.get(&current) {
                for edge in edges {
                    let dependent = edge.subject_entity_id.clone();
                    if !visited.insert(dependent.clone()) {
                        continue;
                    }

                    let mut next_path = path.clone();
                    next_path.push(edge.id.clone());
                    let mut next_evidence_path = evidence_path.clone();
                    next_evidence_path.push(edge.supporting_evidence_ids.clone());
                    impacts.push(DependencyImpact {
                        entity_id: dependent.clone(),
                        distance: next_path.len(),
                        relation_path_ids: next_path.clone(),
                        supporting_evidence_by_relation: next_evidence_path.clone(),
                    });
                    queue.push_back((dependent, next_path, next_evidence_path));
                }
            }
        }

        Ok(impacts)
    }
}

/// This strict timestamp profile is intentionally narrower than RFC 3339:
/// fixed-width UTC seconds make lexical interval comparison safe and auditable.
pub fn is_canonical_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }

    for (index, byte) in bytes.iter().enumerate() {
        if !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit() {
            return false;
        }
    }

    let year = decimal_component(&bytes[0..4]);
    let month = decimal_component(&bytes[5..7]);
    let day = decimal_component(&bytes[8..10]);
    let hour = decimal_component(&bytes[11..13]);
    let minute = decimal_component(&bytes[14..16]);
    let second = decimal_component(&bytes[17..19]);

    if year == 0 || !(1..=12).contains(&month) || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute) || !(0..=59).contains(&second)
    {
        return false;
    }

    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };
    (1..=month_days).contains(&day)
}

fn decimal_component(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0, |value, byte| value * 10 + u32::from(byte - b'0'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::{
        EvidenceKind, EvidenceState, Money, Provenance, Quantity, SourceArtifact, SpatialContext,
        TimeContext, EVIDENCE_SCHEMA_VERSION,
    };

    fn evidence(id: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:field-team".into(),
            subject_id: "urn:example:asset".into(),
            spatial: SpatialContext {
                entity_id: "urn:example:asset".into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: "2026-10-15T10:00:00Z".into(),
                recorded_at: "2026-10-15T10:01:00Z".into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic evidence record for a unit test".into(),
            quantity: Some(Quantity { value: 1.0, unit: "item".into(), uncertainty_abs: None }),
            money: Some(Money { amount_decimal: "1.00".into(), currency: "ZAR".into() }),
            provenance: Provenance {
                method: "synthetic_test".into(),
                source_artifacts: vec![SourceArtifact {
                    id: format!("{id}:source"),
                    uri: format!("https://example.invalid/{id}"),
                    role: "synthetic_fixture".into(),
                    sha256: None,
                }],
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:test".into(),
        }
    }

    fn relation(
        id: &str,
        subject: &str,
        kind: RelationKind,
        object: &str,
        evidence_id: &str,
    ) -> SpatialRelation {
        SpatialRelation {
            id: id.into(),
            subject_entity_id: subject.into(),
            kind,
            object_entity_id: object.into(),
            valid_from: None,
            valid_until: None,
            supporting_evidence_ids: vec![evidence_id.into()],
        }
    }

    #[test]
    fn finds_direct_and_transitive_dependents_with_edge_evidence() {
        let graph = DependencyGraph {
            relations: vec![
                relation("edge-field-pump", "field", RelationKind::DependsOn, "pump", "ev-field-pump"),
                relation("edge-pump-feeder", "pump", RelationKind::SuppliedBy, "feeder", "ev-pump-feeder"),
            ],
        };
        let results = graph.affected_dependents_at("feeder", "2026-10-15T10:00:00Z").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entity_id, "pump");
        assert_eq!(results[0].distance, 1);
        assert_eq!(results[0].relation_path_ids, vec!["edge-pump-feeder"]);
        assert_eq!(results[0].supporting_evidence_by_relation, vec![vec!["ev-pump-feeder"]]);
        assert_eq!(results[1].entity_id, "field");
        assert_eq!(results[1].distance, 2);
        assert_eq!(results[1].relation_path_ids, vec!["edge-pump-feeder", "edge-field-pump"]);
    }

    #[test]
    fn ignores_non_dependency_relationships_and_is_deterministic() {
        let graph = DependencyGraph {
            relations: vec![
                relation("z-field-pump", "field", RelationKind::DependsOn, "pump", "ev-field-pump"),
                relation("located-warehouse", "warehouse", RelationKind::LocatedAt, "feeder", "ev-located"),
                relation("a-pump-feeder", "pump", RelationKind::SuppliedBy, "feeder", "ev-pump-feeder"),
            ],
        };
        let first = graph.affected_dependents_at("feeder", "2026-10-15T10:00:00Z").unwrap();
        let second = graph.affected_dependents_at("feeder", "2026-10-15T10:00:00Z").unwrap();
        assert_eq!(first, second);
        assert_eq!(first.iter().map(|impact| impact.entity_id.as_str()).collect::<Vec<_>>(), vec!["pump", "field"]);
        assert!(!first.iter().any(|impact| impact.entity_id == "warehouse"));
    }

    #[test]
    fn traversal_is_cycle_safe_and_never_returns_the_root() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-feeder", "pump", RelationKind::SuppliedBy, "feeder", "ev-a"),
                relation("feeder-pump", "feeder", RelationKind::DependsOn, "pump", "ev-b"),
            ],
        };
        let results = graph.affected_dependents_at("feeder", "2026-10-15T10:00:00Z").unwrap();
        assert_eq!(results.iter().map(|item| item.entity_id.as_str()).collect::<Vec<_>>(), vec!["pump"]);
    }

    #[test]
    fn half_open_validity_windows_are_respected() {
        let mut edge = relation("edge", "pump", RelationKind::SuppliedBy, "feeder", "ev");
        edge.valid_from = Some("2026-10-15T10:00:00Z".into());
        edge.valid_until = Some("2026-10-15T11:00:00Z".into());
        let graph = DependencyGraph { relations: vec![edge] };
        assert_eq!(graph.affected_dependents_at("feeder", "2026-10-15T09:59:59Z").unwrap().len(), 0);
        assert_eq!(graph.affected_dependents_at("feeder", "2026-10-15T10:00:00Z").unwrap().len(), 1);
        assert_eq!(graph.affected_dependents_at("feeder", "2026-10-15T10:59:59Z").unwrap().len(), 1);
        assert_eq!(graph.affected_dependents_at("feeder", "2026-10-15T11:00:00Z").unwrap().len(), 0);
    }

    #[test]
    fn rejects_noncanonical_or_impossible_timestamps() {
        assert!(is_canonical_utc_timestamp("2026-10-15T10:00:00Z"));
        assert!(is_canonical_utc_timestamp("2024-02-29T00:00:00Z"));
        assert!(!is_canonical_utc_timestamp("2026-02-29T00:00:00Z"));
        assert!(!is_canonical_utc_timestamp("2026-10-15T12:00:00+02:00"));
        assert!(!is_canonical_utc_timestamp("2026-13-15T10:00:00Z"));
        assert!(!is_canonical_utc_timestamp("2026-10-15T10:00:00.100Z"));
    }

    #[test]
    fn reports_unknown_evidence_and_invalid_intervals() {
        let mut edge = relation("edge", "pump", RelationKind::SuppliedBy, "feeder", "missing-evidence");
        edge.valid_from = Some("2026-10-15T11:00:00Z".into());
        edge.valid_until = Some("2026-10-15T10:00:00Z".into());
        let graph = DependencyGraph { relations: vec![edge] };
        let errors = graph.validate(&[evidence("some-other-evidence")]).unwrap_err();
        assert!(errors.contains(&GraphValidationError::UnknownEvidenceReference));
        assert!(errors.contains(&GraphValidationError::InvalidValidityWindow));
        assert_eq!(
            graph.affected_dependents_at("feeder", "2026-10-15T10:00:00Z"),
            Err(DependencyQueryError::InvalidRelationValidity)
        );
    }

    #[test]
    fn validates_relation_ids_endpoints_and_evidence_references() {
        let valid_ev = evidence("ev-valid");
        let mut graph = DependencyGraph {
            relations: vec![
                relation("edge", "pump", RelationKind::SuppliedBy, "feeder", "ev-valid"),
                relation("edge", "pump", RelationKind::SuppliedBy, "feeder", "ev-valid"),
                relation("", "", RelationKind::DependsOn, "", ""),
            ],
        };
        let errors = graph.validate(&[valid_ev]).unwrap_err();
        assert!(errors.contains(&GraphValidationError::DuplicateRelationId));
        assert!(errors.contains(&GraphValidationError::EmptyRelationId));
        assert!(errors.contains(&GraphValidationError::EmptySubjectEntity));
        assert!(errors.contains(&GraphValidationError::EmptyObjectEntity));
        assert!(errors.contains(&GraphValidationError::MissingSupportingEvidence));
        assert!(errors.contains(&GraphValidationError::UnknownEvidenceReference));
        graph.relations[0].subject_entity_id = "same".into();
        graph.relations[0].object_entity_id = "same".into();
        assert!(graph.validate(&[evidence("ev-valid")]).unwrap_err().contains(&GraphValidationError::SelfRelation));
    }

    #[test]
    fn relation_schema_matches_rust_kind_names() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/spatial-economic-relation-v0.1.schema.json"
        ))
        .unwrap();
        let kinds = schema["properties"]["kind"]["enum"].as_array().unwrap();
        let expected = [
            ("depends_on", RelationKind::DependsOn),
            ("supplied_by", RelationKind::SuppliedBy),
            ("served_by", RelationKind::ServedBy),
            ("routes_through", RelationKind::RoutesThrough),
            ("located_at", RelationKind::LocatedAt),
            ("produces", RelationKind::Produces),
            ("transports", RelationKind::Transports),
            ("commits_to", RelationKind::CommitsTo),
            ("delivers_to", RelationKind::DeliversTo),
            ("owned_by", RelationKind::OwnedBy),
        ];
        for (name, kind) in expected {
            assert!(kinds.contains(&serde_json::json!(name)));
            assert_eq!(serde_json::to_value(kind).unwrap(), serde_json::json!(name));
        }
    }
}
