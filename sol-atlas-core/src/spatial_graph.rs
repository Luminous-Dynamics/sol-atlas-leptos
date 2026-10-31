// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Typed, temporal dependency relationships for spatial-economic intelligence.
//!
//! A relationship is a model claim supported by evidence records; it is not
//! automatically a fact. Reachability is structural impact propagation, not a
//! probability, causal estimate, or prediction of real-world loss.

use crate::spatial_economic::{EvidenceRecord, EvidenceState};
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
    DuplicateEvidenceRecordId,
    InvalidEvidenceRecord,
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

/// Evidence-gate outcome for one relationship. These are query-readiness labels,
/// not truth scores and not a substitute for source independence review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceReadiness {
    AssertedOnly,
    IncludesCorroboratedEvidence,
    Blocked,
}

/// Why an evidence reference cannot currently make a relationship traversable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBlockReason {
    ProposedEvidence,
    DisputedEvidence,
    SupersededEvidence,
    WithdrawnEvidence,
    OutsideEvidenceValidity,
    InvalidEvidenceValidity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBlocker {
    pub evidence_id: String,
    pub reason: EvidenceBlockReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationEvidenceAssessment {
    pub relation_id: String,
    pub readiness: EvidenceReadiness,
    pub evidence_states: Vec<EvidenceState>,
    pub blockers: Vec<EvidenceBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssessedDependencyImpact {
    pub impact: DependencyImpact,
    /// Readiness for each relation in impact.relation_path_ids, in the same order.
    pub evidence_readiness_by_relation: Vec<EvidenceReadiness>,
    /// Evidence states aligned with relation_path_ids, and then each edge's support IDs.
    pub evidence_states_by_relation: Vec<Vec<EvidenceState>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssessedDependencyReport {
    pub impacts: Vec<AssessedDependencyImpact>,
    /// All dependency relationships active at the requested instant, including blocked ones.
    pub relation_assessments: Vec<RelationEvidenceAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyAssessmentError {
    InvalidAsOfTimestamp,
    InvalidRelationValidity,
    InvalidGraph(Vec<GraphValidationError>),
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
        let mut evidence_ids = BTreeSet::new();
        let mut errors = Vec::new();
        for evidence in evidence_records {
            if !evidence_ids.insert(evidence.id.as_str()) {
                errors.push(GraphValidationError::DuplicateEvidenceRecordId);
            }
            if evidence.validate().is_err() {
                errors.push(GraphValidationError::InvalidEvidenceRecord);
            }
        }
        let mut relation_ids = BTreeSet::new();

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

    /// Evidence-aware query for current dependency paths.
    ///
    /// Fails closed if the graph has duplicate/invalid evidence records or unknown
    /// evidence references. A relationship is traversable only when every listed
    /// supporting record is Asserted or Corroborated and is valid at `at`.
    /// Proposed, disputed, superseded, withdrawn, invalidly-timed and out-of-window
    /// support blocks that relationship; every blocker is returned for inspection.
    ///
    /// This is an eligibility filter, not a truth oracle. An asserted-only path is
    /// explicitly distinguished from one containing a record labelled corroborated.
    pub fn assess_dependents_at(
        &self,
        prerequisite_entity_id: &str,
        at: &str,
        evidence_records: &[EvidenceRecord],
    ) -> Result<AssessedDependencyReport, DependencyAssessmentError> {
        if !is_canonical_utc_timestamp(at) {
            return Err(DependencyAssessmentError::InvalidAsOfTimestamp);
        }

        self.validate(evidence_records)
            .map_err(DependencyAssessmentError::InvalidGraph)?;

        let evidence_by_id: BTreeMap<&str, &EvidenceRecord> = evidence_records
            .iter()
            .map(|record| (record.id.as_str(), record))
            .collect();

        let mut traversable_relations = Vec::new();
        let mut relation_assessments = Vec::new();

        for relation in &self.relations {
            if let Some(start) = relation.valid_from.as_deref() {
                if !is_canonical_utc_timestamp(start) {
                    return Err(DependencyAssessmentError::InvalidRelationValidity);
                }
            }
            if let Some(end) = relation.valid_until.as_deref() {
                if !is_canonical_utc_timestamp(end) {
                    return Err(DependencyAssessmentError::InvalidRelationValidity);
                }
            }
            if let (Some(start), Some(end)) = (
                relation.valid_from.as_deref(),
                relation.valid_until.as_deref(),
            ) {
                if start >= end {
                    return Err(DependencyAssessmentError::InvalidRelationValidity);
                }
            }

            if !relation.kind.is_dependency_edge() || !relation.is_valid_at(at) {
                continue;
            }

            let mut states = Vec::new();
            let mut blockers = Vec::new();
            let mut includes_corroborated = false;

            for evidence_id in &relation.supporting_evidence_ids {
                let record = evidence_by_id[evidence_id.as_str()];
                states.push(record.state);
                match record.state {
                    EvidenceState::Asserted => {}
                    EvidenceState::Corroborated => includes_corroborated = true,
                    EvidenceState::Proposed => blockers.push(EvidenceBlocker {
                        evidence_id: evidence_id.clone(),
                        reason: EvidenceBlockReason::ProposedEvidence,
                    }),
                    EvidenceState::Disputed => blockers.push(EvidenceBlocker {
                        evidence_id: evidence_id.clone(),
                        reason: EvidenceBlockReason::DisputedEvidence,
                    }),
                    EvidenceState::Superseded => blockers.push(EvidenceBlocker {
                        evidence_id: evidence_id.clone(),
                        reason: EvidenceBlockReason::SupersededEvidence,
                    }),
                    EvidenceState::Withdrawn => blockers.push(EvidenceBlocker {
                        evidence_id: evidence_id.clone(),
                        reason: EvidenceBlockReason::WithdrawnEvidence,
                    }),
                }

                let start = record.time.valid_from.as_deref();
                let end = record.time.valid_until.as_deref();
                let invalid_window = start.is_some_and(|value| !is_canonical_utc_timestamp(value))
                    || end.is_some_and(|value| !is_canonical_utc_timestamp(value))
                    || matches!((start, end), (Some(first), Some(last)) if {
                        is_canonical_utc_timestamp(first)
                            && is_canonical_utc_timestamp(last)
                            && first >= last
                    });

                if invalid_window {
                    blockers.push(EvidenceBlocker {
                        evidence_id: evidence_id.clone(),
                        reason: EvidenceBlockReason::InvalidEvidenceValidity,
                    });
                } else if start.is_some_and(|value| value > at)
                    || end.is_some_and(|value| at >= value)
                {
                    blockers.push(EvidenceBlocker {
                        evidence_id: evidence_id.clone(),
                        reason: EvidenceBlockReason::OutsideEvidenceValidity,
                    });
                }
            }

            let readiness = if !blockers.is_empty() {
                EvidenceReadiness::Blocked
            } else if includes_corroborated {
                EvidenceReadiness::IncludesCorroboratedEvidence
            } else {
                EvidenceReadiness::AssertedOnly
            };

            if readiness != EvidenceReadiness::Blocked {
                traversable_relations.push(relation.clone());
            }

            relation_assessments.push(RelationEvidenceAssessment {
                relation_id: relation.id.clone(),
                readiness,
                evidence_states: states,
                blockers,
            });
        }

        let traversable = DependencyGraph {
            relations: traversable_relations,
        };
        let impacts = traversable
            .affected_dependents_at(prerequisite_entity_id, at)
            .map_err(|error| match error {
                DependencyQueryError::InvalidAsOfTimestamp => {
                    DependencyAssessmentError::InvalidAsOfTimestamp
                }
                DependencyQueryError::InvalidRelationValidity => {
                    DependencyAssessmentError::InvalidRelationValidity
                }
            })?;

        let assessment_by_relation: BTreeMap<&str, &RelationEvidenceAssessment> =
            relation_assessments
                .iter()
                .map(|assessment| (assessment.relation_id.as_str(), assessment))
                .collect();

        let assessed_impacts = impacts
            .into_iter()
            .map(|impact| {
                let evidence_readiness_by_relation = impact
                    .relation_path_ids
                    .iter()
                    .map(|id| assessment_by_relation[id.as_str()].readiness)
                    .collect();
                let evidence_states_by_relation = impact
                    .relation_path_ids
                    .iter()
                    .map(|id| assessment_by_relation[id.as_str()].evidence_states.clone())
                    .collect();
                AssessedDependencyImpact {
                    impact,
                    evidence_readiness_by_relation,
                    evidence_states_by_relation,
                }
            })
            .collect();

        Ok(AssessedDependencyReport {
            impacts: assessed_impacts,
            relation_assessments,
        })
    }

    /// Low-level traversal over graph structure without inspecting evidence state.
    ///
    /// Prefer `assess_dependents_at` for decision-support or operational use.
    /// This method does not validate supporting evidence IDs, source state,
    /// evidence validity periods, independence or truth.
    ///
    /// It finds transitive dependents of an entity as of a canonical UTC timestamp.
    /// An edge `A supplied_by B` means A depends on B, so a query rooted at B
    /// finds A. Only dependency kinds are traversed; semantic relations like
    /// `located_at` or `owned_by` are not treated as failure propagation.
    ///
    /// The result uses breadth-first traversal, stable relation-ID ordering,
    /// cycle protection, shortest explanatory paths, and per-edge evidence IDs.
    /// It does not estimate probability, severity, physical distance or causal
    /// impact.
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

    if year == 0
        || !(1..=12).contains(&month)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
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
        .fold(0, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceState, Money, Provenance, Quantity,
        SourceArtifact, SpatialContext, TimeContext,
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
            quantity: Some(Quantity {
                value: 1.0,
                unit: "item".into(),
                uncertainty_abs: None,
            }),
            money: Some(Money {
                amount_decimal: "1.00".into(),
                currency: "ZAR".into(),
            }),
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
                relation(
                    "edge-field-pump",
                    "field",
                    RelationKind::DependsOn,
                    "pump",
                    "ev-field-pump",
                ),
                relation(
                    "edge-pump-feeder",
                    "pump",
                    RelationKind::SuppliedBy,
                    "feeder",
                    "ev-pump-feeder",
                ),
            ],
        };
        let results = graph
            .affected_dependents_at("feeder", "2026-10-15T10:00:00Z")
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entity_id, "pump");
        assert_eq!(results[0].distance, 1);
        assert_eq!(results[0].relation_path_ids, vec!["edge-pump-feeder"]);
        assert_eq!(
            results[0].supporting_evidence_by_relation,
            vec![vec![String::from("ev-pump-feeder")]]
        );
        assert_eq!(results[1].entity_id, "field");
        assert_eq!(results[1].distance, 2);
        assert_eq!(
            results[1].relation_path_ids,
            vec![
                String::from("edge-pump-feeder"),
                String::from("edge-field-pump")
            ]
        );
    }

    #[test]
    fn ignores_non_dependency_relationships_and_is_deterministic() {
        let graph = DependencyGraph {
            relations: vec![
                relation(
                    "z-field-pump",
                    "field",
                    RelationKind::DependsOn,
                    "pump",
                    "ev-field-pump",
                ),
                relation(
                    "located-warehouse",
                    "warehouse",
                    RelationKind::LocatedAt,
                    "feeder",
                    "ev-located",
                ),
                relation(
                    "a-pump-feeder",
                    "pump",
                    RelationKind::SuppliedBy,
                    "feeder",
                    "ev-pump-feeder",
                ),
            ],
        };
        let first = graph
            .affected_dependents_at("feeder", "2026-10-15T10:00:00Z")
            .unwrap();
        let second = graph
            .affected_dependents_at("feeder", "2026-10-15T10:00:00Z")
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first
                .iter()
                .map(|impact| impact.entity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["pump", "field"]
        );
        assert!(!first.iter().any(|impact| impact.entity_id == "warehouse"));
    }

    #[test]
    fn traversal_is_cycle_safe_and_never_returns_the_root() {
        let graph = DependencyGraph {
            relations: vec![
                relation(
                    "pump-feeder",
                    "pump",
                    RelationKind::SuppliedBy,
                    "feeder",
                    "ev-a",
                ),
                relation(
                    "feeder-pump",
                    "feeder",
                    RelationKind::DependsOn,
                    "pump",
                    "ev-b",
                ),
            ],
        };
        let results = graph
            .affected_dependents_at("feeder", "2026-10-15T10:00:00Z")
            .unwrap();
        assert_eq!(
            results
                .iter()
                .map(|item| item.entity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["pump"]
        );
    }

    #[test]
    fn half_open_validity_windows_are_respected() {
        let mut edge = relation("edge", "pump", RelationKind::SuppliedBy, "feeder", "ev");
        edge.valid_from = Some("2026-10-15T10:00:00Z".into());
        edge.valid_until = Some("2026-10-15T11:00:00Z".into());
        let graph = DependencyGraph {
            relations: vec![edge],
        };
        assert_eq!(
            graph
                .affected_dependents_at("feeder", "2026-10-15T09:59:59Z")
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            graph
                .affected_dependents_at("feeder", "2026-10-15T10:00:00Z")
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            graph
                .affected_dependents_at("feeder", "2026-10-15T10:59:59Z")
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            graph
                .affected_dependents_at("feeder", "2026-10-15T11:00:00Z")
                .unwrap()
                .len(),
            0
        );
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
        let mut edge = relation(
            "edge",
            "pump",
            RelationKind::SuppliedBy,
            "feeder",
            "missing-evidence",
        );
        edge.valid_from = Some("2026-10-15T11:00:00Z".into());
        edge.valid_until = Some("2026-10-15T10:00:00Z".into());
        let graph = DependencyGraph {
            relations: vec![edge],
        };
        let errors = graph
            .validate(&[evidence("some-other-evidence")])
            .unwrap_err();
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
                relation(
                    "edge",
                    "pump",
                    RelationKind::SuppliedBy,
                    "feeder",
                    "ev-valid",
                ),
                relation(
                    "edge",
                    "pump",
                    RelationKind::SuppliedBy,
                    "feeder",
                    "ev-valid",
                ),
                relation("", "", RelationKind::DependsOn, "", ""),
                relation(
                    "missing-support",
                    "asset",
                    RelationKind::DependsOn,
                    "other",
                    "ev-valid",
                ),
            ],
        };
        graph.relations[3].supporting_evidence_ids.clear();
        let errors = graph.validate(&[valid_ev]).unwrap_err();
        assert!(errors.contains(&GraphValidationError::DuplicateRelationId));
        assert!(errors.contains(&GraphValidationError::EmptyRelationId));
        assert!(errors.contains(&GraphValidationError::EmptySubjectEntity));
        assert!(errors.contains(&GraphValidationError::EmptyObjectEntity));
        assert!(errors.contains(&GraphValidationError::MissingSupportingEvidence));
        assert!(errors.contains(&GraphValidationError::EmptySupportingEvidenceId));
        assert!(errors.contains(&GraphValidationError::UnknownEvidenceReference));
        graph.relations[0].subject_entity_id = "same".into();
        graph.relations[0].object_entity_id = "same".into();
        assert!(
            graph
                .validate(&[evidence("ev-valid")])
                .unwrap_err()
                .contains(&GraphValidationError::SelfRelation)
        );
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
    #[test]
    fn synthetic_water_system_graph_fixture_is_valid_and_queryable() {
        let graph: DependencyGraph = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-dependencies.json"
        ))
        .unwrap();
        let evidence_records = [
            evidence("synthetic-evidence-pump-power"),
            evidence("synthetic-evidence-pump-backup-power"),
            evidence("synthetic-evidence-field-pump"),
            evidence("synthetic-evidence-warehouse-location"),
        ];
        graph.validate(&evidence_records).unwrap();

        let impacts = graph
            .affected_dependents_at("synthetic-feeder-east", "2026-10-15T12:00:00Z")
            .unwrap();
        assert_eq!(
            impacts
                .iter()
                .map(|impact| impact.entity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["synthetic-pump-east", "synthetic-field-north"]
        );
        assert!(
            !impacts
                .iter()
                .any(|impact| impact.entity_id == "synthetic-warehouse")
        );
        assert!(
            impacts[0]
                .supporting_evidence_by_relation
                .iter()
                .flatten()
                .any(|id| id == "synthetic-evidence-pump-power")
        );
    }

    #[test]
    fn evidence_aware_query_propagates_only_admissible_edges_and_explains_path_state() {
        let graph = DependencyGraph {
            relations: vec![
                relation(
                    "pump-feeder",
                    "pump",
                    RelationKind::SuppliedBy,
                    "feeder",
                    "ev-pump",
                ),
                relation(
                    "field-pump",
                    "field",
                    RelationKind::DependsOn,
                    "pump",
                    "ev-field",
                ),
                relation(
                    "warehouse-feeder",
                    "warehouse",
                    RelationKind::DependsOn,
                    "feeder",
                    "ev-proposed",
                ),
            ],
        };
        let mut pump_evidence = evidence("ev-pump");
        pump_evidence.state = EvidenceState::Corroborated;
        let field_evidence = evidence("ev-field");
        let mut proposed_evidence = evidence("ev-proposed");
        proposed_evidence.state = EvidenceState::Proposed;

        let report = graph
            .assess_dependents_at(
                "feeder",
                "2026-10-15T10:00:00Z",
                &[pump_evidence, field_evidence, proposed_evidence],
            )
            .unwrap();

        assert_eq!(
            report
                .impacts
                .iter()
                .map(|impact| impact.impact.entity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["pump", "field"]
        );
        assert_eq!(
            report.impacts[0].evidence_readiness_by_relation,
            vec![EvidenceReadiness::IncludesCorroboratedEvidence]
        );
        assert_eq!(
            report.impacts[1].evidence_readiness_by_relation,
            vec![
                EvidenceReadiness::IncludesCorroboratedEvidence,
                EvidenceReadiness::AssertedOnly
            ]
        );
        let warehouse = report
            .relation_assessments
            .iter()
            .find(|assessment| assessment.relation_id == "warehouse-feeder")
            .unwrap();
        assert_eq!(warehouse.readiness, EvidenceReadiness::Blocked);
        assert_eq!(
            warehouse.blockers[0].reason,
            EvidenceBlockReason::ProposedEvidence
        );
    }

    #[test]
    fn evidence_aware_query_blocks_disputed_support_even_when_another_record_is_corroborated() {
        let graph = DependencyGraph {
            relations: vec![SpatialRelation {
                id: "pump-feeder".into(),
                subject_entity_id: "pump".into(),
                kind: RelationKind::SuppliedBy,
                object_entity_id: "feeder".into(),
                valid_from: None,
                valid_until: None,
                supporting_evidence_ids: vec!["ev-corroborated".into(), "ev-disputed".into()],
            }],
        };
        let mut corroborated = evidence("ev-corroborated");
        corroborated.state = EvidenceState::Corroborated;
        let mut disputed = evidence("ev-disputed");
        disputed.state = EvidenceState::Disputed;

        let report = graph
            .assess_dependents_at("feeder", "2026-10-15T10:00:00Z", &[corroborated, disputed])
            .unwrap();
        assert!(report.impacts.is_empty());
        assert_eq!(
            report.relation_assessments[0].readiness,
            EvidenceReadiness::Blocked
        );
        assert!(
            report.relation_assessments[0]
                .blockers
                .iter()
                .any(|blocker| blocker.reason == EvidenceBlockReason::DisputedEvidence)
        );
    }

    #[test]
    fn evidence_validity_window_blocks_stale_support_for_as_of_query() {
        let graph = DependencyGraph {
            relations: vec![relation(
                "pump-feeder",
                "pump",
                RelationKind::SuppliedBy,
                "feeder",
                "ev-expired",
            )],
        };
        let mut expired = evidence("ev-expired");
        expired.time.valid_from = Some("2026-10-01T00:00:00Z".into());
        expired.time.valid_until = Some("2026-10-10T00:00:00Z".into());

        let report = graph
            .assess_dependents_at("feeder", "2026-10-15T10:00:00Z", &[expired])
            .unwrap();
        assert!(report.impacts.is_empty());
        assert_eq!(
            report.relation_assessments[0].blockers[0].reason,
            EvidenceBlockReason::OutsideEvidenceValidity
        );
    }

    #[test]
    fn evidence_aware_query_fails_closed_on_unknown_and_duplicate_evidence_ids() {
        let graph = DependencyGraph {
            relations: vec![relation(
                "pump-feeder",
                "pump",
                RelationKind::SuppliedBy,
                "feeder",
                "ev-pump",
            )],
        };
        let unknown = graph
            .assess_dependents_at(
                "feeder",
                "2026-10-15T10:00:00Z",
                &[evidence("some-other-evidence")],
            )
            .unwrap_err();
        assert!(matches!(
            unknown,
            DependencyAssessmentError::InvalidGraph(errors)
                if errors.contains(&GraphValidationError::UnknownEvidenceReference)
        ));

        let duplicate = graph
            .assess_dependents_at(
                "feeder",
                "2026-10-15T10:00:00Z",
                &[evidence("ev-pump"), evidence("ev-pump")],
            )
            .unwrap_err();
        assert!(matches!(
            duplicate,
            DependencyAssessmentError::InvalidGraph(errors)
                if errors.contains(&GraphValidationError::DuplicateEvidenceRecordId)
        ));

        let mut invalid_record = evidence("ev-pump");
        invalid_record.issuer_id.clear();
        let invalid = graph
            .assess_dependents_at("feeder", "2026-10-15T10:00:00Z", &[invalid_record])
            .unwrap_err();
        assert!(matches!(
            invalid,
            DependencyAssessmentError::InvalidGraph(errors)
                if errors.contains(&GraphValidationError::InvalidEvidenceRecord)
        ));
    }

    #[test]
    fn evidence_aware_query_blocks_malformed_validity_and_rejects_bad_query_time() {
        let graph = DependencyGraph {
            relations: vec![relation(
                "pump-feeder",
                "pump",
                RelationKind::SuppliedBy,
                "feeder",
                "ev-pump",
            )],
        };
        let mut evidence_record = evidence("ev-pump");
        evidence_record.time.valid_from = Some("not-a-timestamp".into());
        let report = graph
            .assess_dependents_at("feeder", "2026-10-15T10:00:00Z", &[evidence_record])
            .unwrap();
        assert!(report.impacts.is_empty());
        assert_eq!(
            report.relation_assessments[0].blockers[0].reason,
            EvidenceBlockReason::InvalidEvidenceValidity
        );
        assert_eq!(
            graph.assess_dependents_at("feeder", "yesterday", &[evidence("ev-pump")]),
            Err(DependencyAssessmentError::InvalidAsOfTimestamp)
        );
    }
}
