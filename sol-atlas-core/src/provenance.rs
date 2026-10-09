// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic, evidence-aware provenance lineage for Sol Atlas.
//!
//! This is a typed, PROV-inspired domain model, not a claim of full W3C PROV
//! serialization or conformance. It records asserted relationships between
//! artifact snapshots, activities, and agents. A graph being valid means its
//! internal references and lineage structure are consistent; it does not
//! establish the truth, quality, authenticity, or qualification of any artifact.
//! A digest reference is not evidence that a digest was checked. Timestamps
//! describe reported temporal extents and do not imply causality.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierProblem {
    Empty,
    SurroundingWhitespace,
    Whitespace,
    ControlCharacter,
}

impl fmt::Display for IdentifierProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "identifier must not be empty"),
            Self::SurroundingWhitespace => {
                write!(f, "identifier must not have surrounding whitespace")
            }
            Self::Whitespace => write!(f, "identifier must not contain whitespace"),
            Self::ControlCharacter => write!(f, "identifier must not contain control characters"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierError {
    kind: &'static str,
    problem: IdentifierProblem,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.problem)
    }
}

impl std::error::Error for IdentifierError {}

macro_rules! validated_identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                let problem = if value.is_empty() {
                    Some(IdentifierProblem::Empty)
                } else if value.trim() != value {
                    Some(IdentifierProblem::SurroundingWhitespace)
                } else if value.chars().any(char::is_control) {
                    Some(IdentifierProblem::ControlCharacter)
                } else if value.chars().any(char::is_whitespace) {
                    Some(IdentifierProblem::Whitespace)
                } else {
                    None
                };

                if let Some(problem) = problem {
                    return Err(IdentifierError {
                        kind: stringify!($name),
                        problem,
                    });
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(<D::Error as serde::de::Error>::custom)
            }
        }
    };
}

validated_identifier!(ArtifactId);
validated_identifier!(ActivityId);
validated_identifier!(AgentId);

/// A locator for a digest value. It identifies a claim about content integrity;
/// it does not record verification status or prove that verification occurred.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IntegrityReference {
    pub algorithm: String,
    pub value: String,
}

impl IntegrityReference {
    pub fn new(
        algorithm: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, IntegrityReferenceError> {
        let algorithm = algorithm.into();
        let value = value.into();
        if !is_non_whitespace_token(&algorithm) {
            return Err(IntegrityReferenceError::InvalidAlgorithm);
        }
        if !is_non_whitespace_token(&value) {
            return Err(IntegrityReferenceError::InvalidValue);
        }
        Ok(Self { algorithm, value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityReferenceError {
    InvalidAlgorithm,
    InvalidValue,
}

impl fmt::Display for IntegrityReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAlgorithm => write!(f, "integrity algorithm must be a non-empty token"),
            Self::InvalidValue => write!(f, "integrity value must be a non-empty token"),
        }
    }
}

impl std::error::Error for IntegrityReferenceError {}

fn is_non_whitespace_token(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value.chars().any(char::is_whitespace)
        && !value.chars().any(char::is_control)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSnapshot {
    pub id: ArtifactId,
    pub label: String,
    pub media_type: Option<String>,
    pub integrity: Option<IntegrityReference>,
    pub source_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: ActivityId,
    pub activity_type: String,
    /// Reported Unix epoch time in milliseconds; not a trusted clock assertion.
    pub started_at_unix_ms: Option<i64>,
    /// Reported Unix epoch time in milliseconds; not a trusted clock assertion.
    pub ended_at_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: AgentId,
    pub agent_type: String,
    pub label: String,
}

/// Typed relations model asserted lineage links; they do not assert the
/// correctness or authority of the related artifacts or agents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProvenanceRelation {
    Used {
        activity: ActivityId,
        artifact: ArtifactId,
    },
    Generated {
        activity: ActivityId,
        artifact: ArtifactId,
    },
    AssociatedWith {
        activity: ActivityId,
        agent: AgentId,
    },
    DerivedFrom {
        artifact: ArtifactId,
        source: ArtifactId,
    },
    AttributedTo {
        artifact: ArtifactId,
        agent: AgentId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceError {
    DuplicateArtifact(ArtifactId),
    DuplicateActivity(ActivityId),
    DuplicateAgent(AgentId),
    DuplicateRelation(ProvenanceRelation),
    MissingArtifact {
        relation: ProvenanceRelation,
        artifact: ArtifactId,
    },
    MissingActivity {
        relation: ProvenanceRelation,
        activity: ActivityId,
    },
    MissingAgent {
        relation: ProvenanceRelation,
        agent: AgentId,
    },
    InvalidActivityInterval {
        activity: ActivityId,
        started_at_unix_ms: i64,
        ended_at_unix_ms: i64,
    },
    InvalidIntegrityReference {
        artifact: ArtifactId,
        problem: IntegrityReferenceError,
    },
    DerivedFromCycle,
    UnknownArtifact(ArtifactId),
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateArtifact(id) => write!(f, "duplicate artifact id: {id}"),
            Self::DuplicateActivity(id) => write!(f, "duplicate activity id: {id}"),
            Self::DuplicateAgent(id) => write!(f, "duplicate agent id: {id}"),
            Self::DuplicateRelation(relation) => {
                write!(f, "duplicate provenance relation: {relation:?}")
            }
            Self::MissingArtifact { relation, artifact } => {
                write!(f, "relation {relation:?} references missing artifact {artifact}")
            }
            Self::MissingActivity { relation, activity } => {
                write!(f, "relation {relation:?} references missing activity {activity}")
            }
            Self::MissingAgent { relation, agent } => {
                write!(f, "relation {relation:?} references missing agent {agent}")
            }
            Self::InvalidActivityInterval {
                activity,
                started_at_unix_ms,
                ended_at_unix_ms,
            } => write!(
                f,
                "activity {activity} ends at {ended_at_unix_ms} before it starts at {started_at_unix_ms}"
            ),
            Self::InvalidIntegrityReference { artifact, problem } => {
                write!(f, "artifact {artifact} has invalid integrity reference: {problem}")
            }
            Self::DerivedFromCycle => write!(f, "derived-from relations must be acyclic"),
            Self::UnknownArtifact(id) => write!(f, "unknown artifact id: {id}"),
        }
    }
}

impl std::error::Error for ProvenanceError {}

/// A normalized provenance graph. Private storage plus validating deserialization
/// prevents callers from accidentally treating an unchecked JSON graph as valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvenanceGraph {
    artifacts: Vec<ArtifactSnapshot>,
    activities: Vec<ActivityRecord>,
    agents: Vec<AgentRecord>,
    relations: Vec<ProvenanceRelation>,
}

impl ProvenanceGraph {
    pub fn new(
        mut artifacts: Vec<ArtifactSnapshot>,
        mut activities: Vec<ActivityRecord>,
        mut agents: Vec<AgentRecord>,
        relations: Vec<ProvenanceRelation>,
    ) -> Result<Self, ProvenanceError> {
        artifacts.sort_by(|a, b| a.id.cmp(&b.id));
        activities.sort_by(|a, b| a.id.cmp(&b.id));
        agents.sort_by(|a, b| a.id.cmp(&b.id));

        let mut artifact_by_id = BTreeMap::new();
        for artifact in &artifacts {
            if artifact_by_id.insert(artifact.id.clone(), artifact).is_some() {
                return Err(ProvenanceError::DuplicateArtifact(artifact.id.clone()));
            }
            if let Some(integrity) = &artifact.integrity {
                if let Err(problem) =
                    IntegrityReference::new(integrity.algorithm.clone(), integrity.value.clone())
                {
                    return Err(ProvenanceError::InvalidIntegrityReference {
                        artifact: artifact.id.clone(),
                        problem,
                    });
                }
            }
        }

        let mut activity_ids = BTreeSet::new();
        for activity in &activities {
            if !activity_ids.insert(activity.id.clone()) {
                return Err(ProvenanceError::DuplicateActivity(activity.id.clone()));
            }
            if let (Some(start), Some(end)) =
                (activity.started_at_unix_ms, activity.ended_at_unix_ms)
            {
                if end < start {
                    return Err(ProvenanceError::InvalidActivityInterval {
                        activity: activity.id.clone(),
                        started_at_unix_ms: start,
                        ended_at_unix_ms: end,
                    });
                }
            }
        }

        let mut agent_ids = BTreeSet::new();
        for agent in &agents {
            if !agent_ids.insert(agent.id.clone()) {
                return Err(ProvenanceError::DuplicateAgent(agent.id.clone()));
            }
        }

        let mut unique_relations = BTreeSet::new();
        for relation in relations {
            if !unique_relations.insert(relation.clone()) {
                return Err(ProvenanceError::DuplicateRelation(relation));
            }
        }
        let relations: Vec<_> = unique_relations.into_iter().collect();

        for relation in &relations {
            match relation {
                ProvenanceRelation::Used { activity, artifact }
                | ProvenanceRelation::Generated { activity, artifact } => {
                    if !activity_ids.contains(activity) {
                        return Err(ProvenanceError::MissingActivity {
                            relation: relation.clone(),
                            activity: activity.clone(),
                        });
                    }
                    if !artifact_by_id.contains_key(artifact) {
                        return Err(ProvenanceError::MissingArtifact {
                            relation: relation.clone(),
                            artifact: artifact.clone(),
                        });
                    }
                }
                ProvenanceRelation::AssociatedWith { activity, agent } => {
                    if !activity_ids.contains(activity) {
                        return Err(ProvenanceError::MissingActivity {
                            relation: relation.clone(),
                            activity: activity.clone(),
                        });
                    }
                    if !agent_ids.contains(agent) {
                        return Err(ProvenanceError::MissingAgent {
                            relation: relation.clone(),
                            agent: agent.clone(),
                        });
                    }
                }
                ProvenanceRelation::DerivedFrom { artifact, source } => {
                    for id in [artifact, source] {
                        if !artifact_by_id.contains_key(id) {
                            return Err(ProvenanceError::MissingArtifact {
                                relation: relation.clone(),
                                artifact: id.clone(),
                            });
                        }
                    }
                }
                ProvenanceRelation::AttributedTo { artifact, agent } => {
                    if !artifact_by_id.contains_key(artifact) {
                        return Err(ProvenanceError::MissingArtifact {
                            relation: relation.clone(),
                            artifact: artifact.clone(),
                        });
                    }
                    if !agent_ids.contains(agent) {
                        return Err(ProvenanceError::MissingAgent {
                            relation: relation.clone(),
                            agent: agent.clone(),
                        });
                    }
                }
            }
        }

        let graph = Self {
            artifacts,
            activities,
            agents,
            relations,
        };
        graph.ensure_derived_from_acyclic()?;
        Ok(graph)
    }

    pub fn artifacts(&self) -> &[ArtifactSnapshot] {
        &self.artifacts
    }

    pub fn activities(&self) -> &[ActivityRecord] {
        &self.activities
    }

    pub fn agents(&self) -> &[AgentRecord] {
        &self.agents
    }

    pub fn relations(&self) -> &[ProvenanceRelation] {
        &self.relations
    }

    /// Stable JSON for semantically identical input, independent of insertion
    /// order. This is a serialization identity, not a cryptographic digest.
    pub fn deterministic_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Return all transitive source artifacts for the selected artifact, excluding
    /// itself. Results are sorted by typed ID, not by inferred temporal or causal order.
    pub fn lineage_ancestors(
        &self,
        artifact: &ArtifactId,
    ) -> Result<Vec<ArtifactId>, ProvenanceError> {
        if !self.artifacts.iter().any(|entry| &entry.id == artifact) {
            return Err(ProvenanceError::UnknownArtifact(artifact.clone()));
        }

        let mut sources: BTreeMap<ArtifactId, Vec<ArtifactId>> = BTreeMap::new();
        for relation in &self.relations {
            if let ProvenanceRelation::DerivedFrom { artifact, source } = relation {
                sources.entry(artifact.clone()).or_default().push(source.clone());
            }
        }

        let mut pending = vec![artifact.clone()];
        let mut visited = BTreeSet::new();
        while let Some(current) = pending.pop() {
            if let Some(next) = sources.get(&current) {
                // Reverse push yields stable traversal, though the public result
                // is sorted independently.
                for source in next.iter().rev() {
                    if visited.insert(source.clone()) {
                        pending.push(source.clone());
                    }
                }
            }
        }
        visited.remove(artifact);
        Ok(visited.into_iter().collect())
    }

    fn ensure_derived_from_acyclic(&self) -> Result<(), ProvenanceError> {
        let mut outgoing: BTreeMap<ArtifactId, Vec<ArtifactId>> = self
            .artifacts
            .iter()
            .map(|artifact| (artifact.id.clone(), Vec::new()))
            .collect();
        let mut indegree: BTreeMap<ArtifactId, usize> = self
            .artifacts
            .iter()
            .map(|artifact| (artifact.id.clone(), 0))
            .collect();

        // Orient edges derived artifact -> source artifact. A cycle is invalid
        // regardless of edge orientation, and Kahn's algorithm avoids recursion.
        for relation in &self.relations {
            if let ProvenanceRelation::DerivedFrom { artifact, source } = relation {
                outgoing.entry(artifact.clone()).or_default().push(source.clone());
                *indegree.entry(source.clone()).or_default() += 1;
            }
        }
        let mut ready: VecDeque<ArtifactId> = indegree
            .iter()
            .filter_map(|(id, degree)| (*degree == 0).then_some(id.clone()))
            .collect();
        let mut visited = 0usize;

        while let Some(current) = ready.pop_front() {
            visited += 1;
            if let Some(neighbors) = outgoing.get(&current) {
                for next in neighbors {
                    let degree = indegree
                        .get_mut(next)
                        .expect("all relation endpoints were validated");
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push_back(next.clone());
                    }
                }
            }
        }

        if visited != self.artifacts.len() {
            return Err(ProvenanceError::DerivedFromCycle);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct ProvenanceGraphUnchecked {
    artifacts: Vec<ArtifactSnapshot>,
    activities: Vec<ActivityRecord>,
    agents: Vec<AgentRecord>,
    relations: Vec<ProvenanceRelation>,
}

impl<'de> Deserialize<'de> for ProvenanceGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ProvenanceGraphUnchecked::deserialize(deserializer)?;
        Self::new(raw.artifacts, raw.activities, raw.agents, raw.relations)
            .map_err(<D::Error as serde::de::Error>::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(id: &str) -> ArtifactSnapshot {
        ArtifactSnapshot {
            id: ArtifactId::new(id).unwrap(),
            label: id.into(),
            media_type: Some("application/json".into()),
            integrity: None,
            source_uri: None,
        }
    }

    fn activity(id: &str, start: Option<i64>, end: Option<i64>) -> ActivityRecord {
        ActivityRecord {
            id: ActivityId::new(id).unwrap(),
            activity_type: "build".into(),
            started_at_unix_ms: start,
            ended_at_unix_ms: end,
        }
    }

    fn agent(id: &str) -> AgentRecord {
        AgentRecord {
            id: AgentId::new(id).unwrap(),
            agent_type: "system".into(),
            label: id.into(),
        }
    }

    #[test]
    fn identifiers_reject_empty_whitespace_and_deserialized_bypass() {
        assert!(ArtifactId::new("").is_err());
        assert!(ActivityId::new(" leading").is_err());
        assert!(AgentId::new("two words").is_err());
        assert!(serde_json::from_str::<ArtifactId>(r#""bad id""#).is_err());
    }

    #[test]
    fn normalizes_order_for_deterministic_json() {
        let artifacts = vec![artifact("b"), artifact("a")];
        let activities = vec![activity("act-2", None, None), activity("act-1", None, None)];
        let agents = vec![agent("z"), agent("x")];
        let relations = vec![
            ProvenanceRelation::AttributedTo {
                artifact: ArtifactId::new("b").unwrap(),
                agent: AgentId::new("z").unwrap(),
            },
            ProvenanceRelation::AttributedTo {
                artifact: ArtifactId::new("a").unwrap(),
                agent: AgentId::new("x").unwrap(),
            },
        ];
        let first = ProvenanceGraph::new(
            artifacts.clone(), activities.clone(), agents.clone(), relations.clone(),
        ).unwrap();
        let second = ProvenanceGraph::new(
            artifacts.into_iter().rev().collect(),
            activities.into_iter().rev().collect(),
            agents.into_iter().rev().collect(),
            relations.into_iter().rev().collect(),
        ).unwrap();
        assert_eq!(first.deterministic_json().unwrap(), second.deterministic_json().unwrap());
    }

    #[test]
    fn rejects_dangling_references() {
        let relation = ProvenanceRelation::Used {
            activity: ActivityId::new("build-1").unwrap(),
            artifact: ArtifactId::new("missing").unwrap(),
        };
        let err = ProvenanceGraph::new(
            vec![], vec![activity("build-1", None, None)], vec![], vec![relation],
        ).unwrap_err();
        assert!(matches!(err, ProvenanceError::MissingArtifact { .. }));
    }

    #[test]
    fn rejects_duplicate_relations_instead_of_silently_deduplicating() {
        let relation = ProvenanceRelation::DerivedFrom {
            artifact: ArtifactId::new("b").unwrap(),
            source: ArtifactId::new("a").unwrap(),
        };
        let err = ProvenanceGraph::new(
            vec![artifact("a"), artifact("b")], vec![], vec![], vec![relation.clone(), relation],
        ).unwrap_err();
        assert!(matches!(err, ProvenanceError::DuplicateRelation(_)));
    }

    #[test]
    fn derived_from_must_be_acyclic() {
        let relations = vec![
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("a").unwrap(),
                source: ArtifactId::new("b").unwrap(),
            },
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("b").unwrap(),
                source: ArtifactId::new("a").unwrap(),
            },
        ];
        let err = ProvenanceGraph::new(
            vec![artifact("a"), artifact("b")], vec![], vec![], relations,
        ).unwrap_err();
        assert_eq!(err, ProvenanceError::DerivedFromCycle);
    }

    #[test]
    fn invalid_activity_interval_is_rejected() {
        let err = ProvenanceGraph::new(
            vec![], vec![activity("bad", Some(20), Some(10))], vec![], vec![],
        ).unwrap_err();
        assert!(matches!(err, ProvenanceError::InvalidActivityInterval { .. }));
    }

    #[test]
    fn deserialization_uses_graph_validation() {
        let json = r#"{
            "artifacts": [],
            "activities": [{
                "id": "bad",
                "activity_type": "build",
                "started_at_unix_ms": 20,
                "ended_at_unix_ms": 10
            }],
            "agents": [],
            "relations": []
        }"#;
        assert!(serde_json::from_str::<ProvenanceGraph>(json).is_err());
    }

    #[test]
    fn lineage_closure_is_transitive_and_deterministically_sorted() {
        let relations = vec![
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("final").unwrap(),
                source: ArtifactId::new("middle").unwrap(),
            },
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("middle").unwrap(),
                source: ArtifactId::new("source").unwrap(),
            },
        ];
        let graph = ProvenanceGraph::new(
            vec![artifact("source"), artifact("middle"), artifact("final")],
            vec![], vec![], relations,
        ).unwrap();
        assert_eq!(
            graph.lineage_ancestors(&ArtifactId::new("final").unwrap()).unwrap(),
            vec![ArtifactId::new("middle").unwrap(), ArtifactId::new("source").unwrap()]
        );
    }

    #[test]
    fn integrity_reference_is_validated_at_graph_boundary() {
        let mut bad = artifact("report");
        bad.integrity = Some(IntegrityReference {
            algorithm: "sha 256".into(),
            value: "abcd".into(),
        });
        let err = ProvenanceGraph::new(vec![bad], vec![], vec![], vec![]).unwrap_err();
        assert!(matches!(err, ProvenanceError::InvalidIntegrityReference { .. }));
    }
}
