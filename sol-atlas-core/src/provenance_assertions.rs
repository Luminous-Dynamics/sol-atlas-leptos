// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Identity-preserving records of provenance assertions.
//!
//! Unlike ProvenanceGraph, this module records claims about relationships.
//! It intentionally preserves independent and conflicting assertions, even
//! when their combined derived_from claims form a cycle. Such a cycle is a
//! conflict in asserted accounts, not a canonical lineage graph to traverse.
//! An external locator and an assessment reference are declarations only:
//! neither means a source was retrieved, verified, trusted, or accepted.
//! Reported timestamps are not trusted-clock or causality evidence.

use crate::provenance::{
    ActivityId, ActivityRecord, AgentId, AgentRecord, ArtifactId, ArtifactSnapshot,
    IntegrityReference, IntegrityReferenceError, ProvenanceRelation,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertionIdentifierProblem {
    Empty,
    SurroundingWhitespace,
    Whitespace,
    ControlCharacter,
}

impl fmt::Display for AssertionIdentifierProblem {
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
pub struct AssertionIdentifierError {
    kind: &'static str,
    problem: AssertionIdentifierProblem,
}

impl fmt::Display for AssertionIdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.problem)
    }
}

impl std::error::Error for AssertionIdentifierError {}

macro_rules! assertion_identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, AssertionIdentifierError> {
                let value = value.into();
                let problem = if value.is_empty() {
                    Some(AssertionIdentifierProblem::Empty)
                } else if value.trim() != value {
                    Some(AssertionIdentifierProblem::SurroundingWhitespace)
                } else if value.chars().any(char::is_control) {
                    Some(AssertionIdentifierProblem::ControlCharacter)
                } else if value.chars().any(char::is_whitespace) {
                    Some(AssertionIdentifierProblem::Whitespace)
                } else {
                    None
                };

                if let Some(problem) = problem {
                    return Err(AssertionIdentifierError {
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

assertion_identifier!(AssertionId);

/// A locator string supplied for an external source. Its syntax and remote
/// target are deliberately not treated as validated URI semantics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ExternalSourceLocator(String);

impl ExternalSourceLocator {
    pub fn new(value: impl Into<String>) -> Result<Self, ExternalSourceLocatorError> {
        let value = value.into();
        if value.is_empty() || value.trim() != value {
            return Err(ExternalSourceLocatorError::EmptyOrSurroundedByWhitespace);
        }
        if value.chars().any(char::is_control) {
            return Err(ExternalSourceLocatorError::ControlCharacter);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ExternalSourceLocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(<D::Error as serde::de::Error>::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalSourceLocatorError {
    EmptyOrSurroundedByWhitespace,
    ControlCharacter,
}

impl fmt::Display for ExternalSourceLocatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyOrSurroundedByWhitespace => {
                write!(f, "external source locator must be non-empty and trimmed")
            }
            Self::ControlCharacter => {
                write!(
                    f,
                    "external source locator must not contain control characters"
                )
            }
        }
    }
}

impl std::error::Error for ExternalSourceLocatorError {}

/// External locator state supported by this module. A locator can be
/// recorded but not promoted to resolved without a separate resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSourceResolution {
    Unresolved,
}

/// Source reference offered as the basis for an assertion.
/// External references explicitly retain their unresolved status: this module
/// has no external retrieval, identity, or integrity verifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssertionSource {
    LocalArtifact {
        artifact_id: ArtifactId,
    },
    External {
        locator: ExternalSourceLocator,
        resolution: ExternalSourceResolution,
    },
}

/// A pointer to an external assessment/qualification record. The pointer does
/// not mean the record exists, is authentic, or qualifies the related claim.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExternalAssessmentReference {
    pub authority: String,
    pub reference: String,
}

impl ExternalAssessmentReference {
    pub fn new(
        authority: impl Into<String>,
        reference: impl Into<String>,
    ) -> Result<Self, AssessmentReferenceError> {
        let authority = authority.into();
        let reference = reference.into();
        if !is_non_empty_trimmed(&authority) {
            return Err(AssessmentReferenceError::InvalidAuthority);
        }
        if !is_non_empty_trimmed(&reference) {
            return Err(AssessmentReferenceError::InvalidReference);
        }
        if contains_control(&authority) || contains_control(&reference) {
            return Err(AssessmentReferenceError::ControlCharacter);
        }
        Ok(Self {
            authority,
            reference,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentReferenceError {
    InvalidAuthority,
    InvalidReference,
    ControlCharacter,
}

impl fmt::Display for AssessmentReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAuthority => {
                write!(f, "assessment authority must be non-empty and trimmed")
            }
            Self::InvalidReference => {
                write!(f, "assessment reference must be non-empty and trimmed")
            }
            Self::ControlCharacter => {
                write!(f, "assessment fields must not contain control characters")
            }
        }
    }
}

impl std::error::Error for AssessmentReferenceError {}

fn is_non_empty_trimmed(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}

fn contains_control(value: &str) -> bool {
    value.chars().any(char::is_control)
}

/// A source-attributed assertion of a relation. The relation payload is
/// distinct from assertion identity, actor, supporting sources, and reported
/// time so independent accounts can be preserved without deduplication.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProvenanceAssertion {
    pub id: AssertionId,
    pub relation: ProvenanceRelation,
    pub asserted_by: Option<AgentId>,
    pub sources: Vec<AssertionSource>,
    /// Reported Unix epoch milliseconds; not trusted time or causal order.
    pub reported_at_unix_ms: Option<i64>,
    /// External pointer only; no local qualification state is inferred.
    pub assessment_ref: Option<ExternalAssessmentReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionGraphError {
    DuplicateArtifact(ArtifactId),
    DuplicateActivity(ActivityId),
    DuplicateAgent(AgentId),
    DuplicateAssertion(AssertionId),
    DuplicateSource {
        assertion_id: AssertionId,
        source: AssertionSource,
    },
    InvalidArtifactIntegrity {
        artifact_id: ArtifactId,
        problem: IntegrityReferenceError,
    },
    InvalidActivityInterval {
        activity_id: ActivityId,
        started_at_unix_ms: i64,
        ended_at_unix_ms: i64,
    },
    MissingArtifact {
        assertion_id: AssertionId,
        artifact_id: ArtifactId,
    },
    MissingActivity {
        assertion_id: AssertionId,
        activity_id: ActivityId,
    },
    MissingAgent {
        assertion_id: AssertionId,
        agent_id: AgentId,
    },
    InvalidExternalAssessment {
        assertion_id: AssertionId,
        problem: AssessmentReferenceError,
    },
}

impl fmt::Display for AssertionGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateArtifact(id) => write!(f, "duplicate artifact id: {id}"),
            Self::DuplicateActivity(id) => write!(f, "duplicate activity id: {id}"),
            Self::DuplicateAgent(id) => write!(f, "duplicate agent id: {id}"),
            Self::DuplicateAssertion(id) => write!(f, "duplicate assertion id: {id}"),
            Self::DuplicateSource {
                assertion_id,
                source,
            } => {
                write!(f, "assertion {assertion_id} repeats source {source:?}")
            }
            Self::InvalidArtifactIntegrity {
                artifact_id,
                problem,
            } => {
                write!(
                    f,
                    "artifact {artifact_id} has invalid integrity reference: {problem}"
                )
            }
            Self::InvalidActivityInterval {
                activity_id,
                started_at_unix_ms,
                ended_at_unix_ms,
            } => write!(
                f,
                "activity {activity_id} ends at {ended_at_unix_ms} before it starts at {started_at_unix_ms}"
            ),
            Self::MissingArtifact {
                assertion_id,
                artifact_id,
            } => {
                write!(
                    f,
                    "assertion {assertion_id} references missing artifact {artifact_id}"
                )
            }
            Self::MissingActivity {
                assertion_id,
                activity_id,
            } => {
                write!(
                    f,
                    "assertion {assertion_id} references missing activity {activity_id}"
                )
            }
            Self::MissingAgent {
                assertion_id,
                agent_id,
            } => {
                write!(
                    f,
                    "assertion {assertion_id} references missing agent {agent_id}"
                )
            }
            Self::InvalidExternalAssessment {
                assertion_id,
                problem,
            } => {
                write!(
                    f,
                    "assertion {assertion_id} has invalid assessment reference: {problem}"
                )
            }
        }
    }
}

impl std::error::Error for AssertionGraphError {}

/// A normalized assertion ledger. Local endpoints are checked, but the ledger
/// deliberately does not adjudicate conflicting relations or reject cycles
/// across unadjudicated assertions. Build a canonical ProvenanceGraph only
/// from an externally selected/adjudicated relation set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvenanceAssertionGraph {
    artifacts: Vec<ArtifactSnapshot>,
    activities: Vec<ActivityRecord>,
    agents: Vec<AgentRecord>,
    assertions: Vec<ProvenanceAssertion>,
}

impl ProvenanceAssertionGraph {
    pub fn new(
        mut artifacts: Vec<ArtifactSnapshot>,
        mut activities: Vec<ActivityRecord>,
        mut agents: Vec<AgentRecord>,
        mut assertions: Vec<ProvenanceAssertion>,
    ) -> Result<Self, AssertionGraphError> {
        artifacts.sort_by(|a, b| a.id.cmp(&b.id));
        activities.sort_by(|a, b| a.id.cmp(&b.id));
        agents.sort_by(|a, b| a.id.cmp(&b.id));
        assertions.sort_by(|a, b| a.id.cmp(&b.id));

        let mut artifact_ids = BTreeSet::new();
        for artifact in &artifacts {
            if !artifact_ids.insert(artifact.id.clone()) {
                return Err(AssertionGraphError::DuplicateArtifact(artifact.id.clone()));
            }
            if let Some(integrity) = &artifact.integrity {
                if let Err(problem) =
                    IntegrityReference::new(integrity.algorithm.clone(), integrity.value.clone())
                {
                    return Err(AssertionGraphError::InvalidArtifactIntegrity {
                        artifact_id: artifact.id.clone(),
                        problem,
                    });
                }
            }
        }

        let mut activity_ids = BTreeSet::new();
        for activity in &activities {
            if !activity_ids.insert(activity.id.clone()) {
                return Err(AssertionGraphError::DuplicateActivity(activity.id.clone()));
            }
            if let (Some(start), Some(end)) =
                (activity.started_at_unix_ms, activity.ended_at_unix_ms)
            {
                if end < start {
                    return Err(AssertionGraphError::InvalidActivityInterval {
                        activity_id: activity.id.clone(),
                        started_at_unix_ms: start,
                        ended_at_unix_ms: end,
                    });
                }
            }
        }

        let mut agent_ids = BTreeSet::new();
        for agent in &agents {
            if !agent_ids.insert(agent.id.clone()) {
                return Err(AssertionGraphError::DuplicateAgent(agent.id.clone()));
            }
        }

        let mut assertion_ids = BTreeSet::new();
        for assertion in &assertions {
            if !assertion_ids.insert(assertion.id.clone()) {
                return Err(AssertionGraphError::DuplicateAssertion(
                    assertion.id.clone(),
                ));
            }

            if let Some(asserted_by) = &assertion.asserted_by {
                if !agent_ids.contains(asserted_by) {
                    return Err(AssertionGraphError::MissingAgent {
                        assertion_id: assertion.id.clone(),
                        agent_id: asserted_by.clone(),
                    });
                }
            }

            let mut unique_sources = BTreeSet::new();
            for source in &assertion.sources {
                if !unique_sources.insert(source.clone()) {
                    return Err(AssertionGraphError::DuplicateSource {
                        assertion_id: assertion.id.clone(),
                        source: source.clone(),
                    });
                }
                if let AssertionSource::LocalArtifact { artifact_id } = source {
                    if !artifact_ids.contains(artifact_id) {
                        return Err(AssertionGraphError::MissingArtifact {
                            assertion_id: assertion.id.clone(),
                            artifact_id: artifact_id.clone(),
                        });
                    }
                }
            }

            if let Some(assessment) = &assertion.assessment_ref {
                if let Err(problem) = ExternalAssessmentReference::new(
                    assessment.authority.clone(),
                    assessment.reference.clone(),
                ) {
                    return Err(AssertionGraphError::InvalidExternalAssessment {
                        assertion_id: assertion.id.clone(),
                        problem,
                    });
                }
            }

            validate_relation_endpoints(assertion, &artifact_ids, &activity_ids, &agent_ids)?;
        }

        for assertion in &mut assertions {
            assertion.sources.sort();
        }

        Ok(Self {
            artifacts,
            activities,
            agents,
            assertions,
        })
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

    pub fn assertions(&self) -> &[ProvenanceAssertion] {
        &self.assertions
    }

    /// Deterministic serialization of the assertion ledger. This is not a
    /// cryptographic digest and does not imply trust in any assertion.
    pub fn deterministic_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

fn validate_relation_endpoints(
    assertion: &ProvenanceAssertion,
    artifact_ids: &BTreeSet<ArtifactId>,
    activity_ids: &BTreeSet<ActivityId>,
    agent_ids: &BTreeSet<AgentId>,
) -> Result<(), AssertionGraphError> {
    let id = &assertion.id;
    match &assertion.relation {
        ProvenanceRelation::Used { activity, artifact }
        | ProvenanceRelation::Generated { activity, artifact } => {
            if !activity_ids.contains(activity) {
                return Err(AssertionGraphError::MissingActivity {
                    assertion_id: id.clone(),
                    activity_id: activity.clone(),
                });
            }
            if !artifact_ids.contains(artifact) {
                return Err(AssertionGraphError::MissingArtifact {
                    assertion_id: id.clone(),
                    artifact_id: artifact.clone(),
                });
            }
        }
        ProvenanceRelation::AssociatedWith { activity, agent } => {
            if !activity_ids.contains(activity) {
                return Err(AssertionGraphError::MissingActivity {
                    assertion_id: id.clone(),
                    activity_id: activity.clone(),
                });
            }
            if !agent_ids.contains(agent) {
                return Err(AssertionGraphError::MissingAgent {
                    assertion_id: id.clone(),
                    agent_id: agent.clone(),
                });
            }
        }
        ProvenanceRelation::DerivedFrom { artifact, source } => {
            for artifact_id in [artifact, source] {
                if !artifact_ids.contains(artifact_id) {
                    return Err(AssertionGraphError::MissingArtifact {
                        assertion_id: id.clone(),
                        artifact_id: artifact_id.clone(),
                    });
                }
            }
        }
        ProvenanceRelation::AttributedTo { artifact, agent } => {
            if !artifact_ids.contains(artifact) {
                return Err(AssertionGraphError::MissingArtifact {
                    assertion_id: id.clone(),
                    artifact_id: artifact.clone(),
                });
            }
            if !agent_ids.contains(agent) {
                return Err(AssertionGraphError::MissingAgent {
                    assertion_id: id.clone(),
                    agent_id: agent.clone(),
                });
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct AssertionGraphUnchecked {
    artifacts: Vec<ArtifactSnapshot>,
    activities: Vec<ActivityRecord>,
    agents: Vec<AgentRecord>,
    assertions: Vec<ProvenanceAssertion>,
}

impl<'de> Deserialize<'de> for ProvenanceAssertionGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = AssertionGraphUnchecked::deserialize(deserializer)?;
        Self::new(raw.artifacts, raw.activities, raw.agents, raw.assertions)
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

    fn agent(id: &str) -> AgentRecord {
        AgentRecord {
            id: AgentId::new(id).unwrap(),
            agent_type: "reporter".into(),
            label: id.into(),
        }
    }

    fn activity(id: &str) -> ActivityRecord {
        ActivityRecord {
            id: ActivityId::new(id).unwrap(),
            activity_type: "inspection".into(),
            started_at_unix_ms: None,
            ended_at_unix_ms: None,
        }
    }

    fn assertion(
        id: &str,
        relation: ProvenanceRelation,
        source: AssertionSource,
    ) -> ProvenanceAssertion {
        ProvenanceAssertion {
            id: AssertionId::new(id).unwrap(),
            relation,
            asserted_by: Some(AgentId::new("agent-a").unwrap()),
            sources: vec![source],
            reported_at_unix_ms: None,
            assessment_ref: None,
        }
    }

    fn fixture_entities() -> (Vec<ArtifactSnapshot>, Vec<ActivityRecord>, Vec<AgentRecord>) {
        (
            vec![artifact("a"), artifact("b")],
            vec![activity("act")],
            vec![agent("agent-a"), agent("agent-b")],
        )
    }

    #[test]
    fn identifier_deserialization_cannot_bypass_validation() {
        assert!(serde_json::from_str::<AssertionId>(r#""bad id""#).is_err());
        assert!(serde_json::from_str::<ExternalSourceLocator>(r#""  source""#).is_err());
    }

    #[test]
    fn distinct_assertion_ids_preserve_identical_relation_content() {
        let relation = ProvenanceRelation::DerivedFrom {
            artifact: ArtifactId::new("b").unwrap(),
            source: ArtifactId::new("a").unwrap(),
        };
        let source = AssertionSource::LocalArtifact {
            artifact_id: ArtifactId::new("a").unwrap(),
        };
        let first = assertion("assertion-1", relation.clone(), source.clone());
        let mut second = assertion("assertion-2", relation.clone(), source);
        second.asserted_by = Some(AgentId::new("agent-b").unwrap());

        let (artifacts, activities, agents) = fixture_entities();
        let graph =
            ProvenanceAssertionGraph::new(artifacts, activities, agents, vec![second, first])
                .unwrap();

        assert_eq!(graph.assertions().len(), 2);
        assert_eq!(graph.assertions()[0].relation, relation);
        assert_eq!(graph.assertions()[0].id.as_str(), "assertion-1");
        assert_eq!(graph.assertions()[1].id.as_str(), "assertion-2");
    }

    #[test]
    fn conflicting_and_cyclic_unadjudicated_accounts_are_preserved() {
        let forward = assertion(
            "assertion-ab",
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("a").unwrap(),
                source: ArtifactId::new("b").unwrap(),
            },
            AssertionSource::External {
                locator: ExternalSourceLocator::new("urn:source:report-1").unwrap(),
                resolution: ExternalSourceResolution::Unresolved,
            },
        );
        let reverse = assertion(
            "assertion-ba",
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("b").unwrap(),
                source: ArtifactId::new("a").unwrap(),
            },
            AssertionSource::External {
                locator: ExternalSourceLocator::new("urn:source:report-2").unwrap(),
                resolution: ExternalSourceResolution::Unresolved,
            },
        );

        let (artifacts, activities, agents) = fixture_entities();
        let graph =
            ProvenanceAssertionGraph::new(artifacts, activities, agents, vec![reverse, forward])
                .unwrap();

        assert_eq!(graph.assertions().len(), 2);
        assert_ne!(
            graph.assertions()[0].relation,
            graph.assertions()[1].relation
        );
        let serialized = graph.deterministic_json().unwrap();
        assert!(serialized.contains(r#""resolution":"unresolved""#));
    }

    #[test]
    fn local_source_and_relation_endpoints_must_exist() {
        let invalid = assertion(
            "assertion-missing-source",
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("b").unwrap(),
                source: ArtifactId::new("a").unwrap(),
            },
            AssertionSource::LocalArtifact {
                artifact_id: ArtifactId::new("missing").unwrap(),
            },
        );
        let (artifacts, activities, agents) = fixture_entities();
        let err = ProvenanceAssertionGraph::new(artifacts, activities, agents, vec![invalid])
            .unwrap_err();
        assert!(matches!(err, AssertionGraphError::MissingArtifact { .. }));
    }

    #[test]
    fn assertion_ledger_validates_embedded_activity_and_artifact_invariants() {
        let mut bad_artifact = artifact("bad-integrity");
        bad_artifact.integrity = Some(IntegrityReference {
            algorithm: "sha 256".into(),
            value: "abcd".into(),
        });
        let (mut artifacts, activities, agents) = fixture_entities();
        artifacts.push(bad_artifact);
        let err =
            ProvenanceAssertionGraph::new(artifacts, activities.clone(), agents.clone(), vec![])
                .unwrap_err();
        assert!(matches!(
            err,
            AssertionGraphError::InvalidArtifactIntegrity { .. }
        ));

        let mut invalid = activity("time-reversed");
        invalid.started_at_unix_ms = Some(20);
        invalid.ended_at_unix_ms = Some(10);
        let (artifacts, mut activities, agents) = fixture_entities();
        activities.push(invalid);
        let err = ProvenanceAssertionGraph::new(artifacts, activities, agents, vec![]).unwrap_err();
        assert!(matches!(
            err,
            AssertionGraphError::InvalidActivityInterval { .. }
        ));
    }

    #[test]
    fn duplicate_assertion_ids_fail_closed() {
        let source = AssertionSource::External {
            locator: ExternalSourceLocator::new("urn:source:one").unwrap(),
            resolution: ExternalSourceResolution::Unresolved,
        };
        let relation = ProvenanceRelation::DerivedFrom {
            artifact: ArtifactId::new("b").unwrap(),
            source: ArtifactId::new("a").unwrap(),
        };
        let first = assertion("same", relation.clone(), source.clone());
        let second = assertion("same", relation, source);

        let (artifacts, activities, agents) = fixture_entities();
        let err = ProvenanceAssertionGraph::new(artifacts, activities, agents, vec![first, second])
            .unwrap_err();
        assert!(matches!(err, AssertionGraphError::DuplicateAssertion(_)));
    }

    #[test]
    fn deterministic_json_is_independent_of_insertion_order() {
        let one = assertion(
            "one",
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("b").unwrap(),
                source: ArtifactId::new("a").unwrap(),
            },
            AssertionSource::External {
                locator: ExternalSourceLocator::new("urn:source:one").unwrap(),
                resolution: ExternalSourceResolution::Unresolved,
            },
        );
        let two = assertion(
            "two",
            ProvenanceRelation::AttributedTo {
                artifact: ArtifactId::new("a").unwrap(),
                agent: AgentId::new("agent-b").unwrap(),
            },
            AssertionSource::External {
                locator: ExternalSourceLocator::new("urn:source:two").unwrap(),
                resolution: ExternalSourceResolution::Unresolved,
            },
        );

        let (a1, activities1, agents1) = fixture_entities();
        let left =
            ProvenanceAssertionGraph::new(a1, activities1, agents1, vec![two.clone(), one.clone()])
                .unwrap();

        let (a2, activities2, agents2) = fixture_entities();
        let right = ProvenanceAssertionGraph::new(
            a2.into_iter().rev().collect(),
            activities2.into_iter().rev().collect(),
            agents2.into_iter().rev().collect(),
            vec![one, two],
        )
        .unwrap();

        assert_eq!(
            left.deterministic_json().unwrap(),
            right.deterministic_json().unwrap()
        );
    }

    #[test]
    fn assertion_graph_deserialization_uses_validation_boundary() {
        let json = r#"{
            "artifacts": [],
            "activities": [],
            "agents": [],
            "assertions": [{
                "id": "bad assertion id",
                "relation": {
                    "kind": "derived_from",
                    "artifact": "x",
                    "source": "y"
                },
                "asserted_by": null,
                "sources": [],
                "reported_at_unix_ms": null,
                "assessment_ref": null
            }]
        }"#;
        assert!(serde_json::from_str::<ProvenanceAssertionGraph>(json).is_err());
    }

    #[test]
    fn assessment_reference_is_a_pointer_not_automatic_qualification() {
        let mut record = assertion(
            "assessment-ref",
            ProvenanceRelation::DerivedFrom {
                artifact: ArtifactId::new("b").unwrap(),
                source: ArtifactId::new("a").unwrap(),
            },
            AssertionSource::External {
                locator: ExternalSourceLocator::new("urn:source:report").unwrap(),
                resolution: ExternalSourceResolution::Unresolved,
            },
        );
        record.assessment_ref = Some(
            ExternalAssessmentReference::new("urn:authority:example", "assessment-42").unwrap(),
        );

        let (artifacts, activities, agents) = fixture_entities();
        let graph =
            ProvenanceAssertionGraph::new(artifacts, activities, agents, vec![record]).unwrap();
        let assessment = graph.assertions()[0].assessment_ref.as_ref().unwrap();
        assert_eq!(assessment.reference, "assessment-42");
    }
}
