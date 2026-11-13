// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Candidate underground-utility inference graph.
//!
//! This is an inference/provenance model, not a MUDDI replacement or an
//! authoritative utility registry. Geometry is referenced by existing
//! spatial-claim IDs; canonical owner asset identifiers are deliberately not
//! minted here. Event-to-asset association probability may sum to less than 1,
//! preserving explicit abstention mass.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const PROBABILITY_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateClaimStatus {
    Candidate,
    Conflicted,
    Withdrawn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityKind {
    Water,
    Wastewater,
    Stormwater,
    Gas,
    Electric,
    Fibre,
    Telecommunications,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedUtilityEvent {
    pub event_id: String,
    pub observation_id: String,
    pub source_evidence_id: String,
    pub acquisition_id: String,
    pub spatial_support_id: String,
    pub sensor_configuration_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateUtilityAsset {
    /// Must be namespaced as candidate:...; this is not a MUDDI ObjectID or
    /// owner-system asset identifier.
    pub candidate_asset_id: String,
    pub target_kinds: Vec<UtilityKind>,
    pub geometry_claim_id: String,
    pub supporting_evidence_ids: Vec<String>,
    pub claim_status: CandidateClaimStatus,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventAssetAssociation {
    pub event_id: String,
    pub candidate_asset_id: String,
    /// Posterior association probability for this event->candidate pair.
    /// Mass not assigned to any candidate remains an implicit abstention.
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateNetworkNode {
    pub candidate_node_id: String,
    pub geometry_claim_id: String,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateNetworkRelation {
    Continuation,
    Junction,
    CrossingUnresolved,
    NearIntersectionUnresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateNetworkConnection {
    pub connection_id: String,
    pub from_node_candidate_id: String,
    pub to_node_candidate_id: String,
    pub relation: CandidateNetworkRelation,
    pub supporting_evidence_ids: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtilityInferenceGraph {
    pub graph_id: String,
    pub status: String,
    pub geography: String,
    pub claim_boundary: String,
    pub observed_events: Vec<ObservedUtilityEvent>,
    pub candidate_assets: Vec<CandidateUtilityAsset>,
    pub event_asset_associations: Vec<EventAssetAssociation>,
    pub candidate_nodes: Vec<CandidateNetworkNode>,
    pub candidate_connections: Vec<CandidateNetworkConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UtilityInferenceError {
    EmptyIdentifier(&'static str),
    DuplicateId(&'static str, String),
    InvalidGraphClaimBoundary,
    InvalidCandidateAssetNamespace(String),
    CandidateAssetWithoutEvents(String),
    UnknownAssociationEvent(String),
    UnknownAssociationAsset(String),
    InvalidAssociationProbability,
    AssociationMassExceedsOne(String),
    InvalidConfidence(String),
    UnknownConnectionNode(String),
    SelfConnection(String),
    ConnectionWithoutEvidence(String),
    CandidateAssetWithoutEvidence(String),
    CandidateNodeWithoutEvidence(String),
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn unique_ids<'a>(
    values: impl Iterator<Item = &'a str>,
    kind: &'static str,
    errors: &mut Vec<UtilityInferenceError>,
) -> BTreeSet<&'a str> {
    let mut ids = BTreeSet::new();
    for id in values {
        if !non_empty(id) {
            errors.push(UtilityInferenceError::EmptyIdentifier(kind));
        } else if !ids.insert(id) {
            errors.push(UtilityInferenceError::DuplicateId(kind, id.to_string()));
        }
    }
    ids
}

/// Validate structural and probabilistic invariants. This does not validate
/// geometry, source authenticity, utility ownership, physical truth or survey
/// qualification. Those remain external, independently reviewed claims.
pub fn validate_utility_inference_graph(
    graph: &UtilityInferenceGraph,
) -> Vec<UtilityInferenceError> {
    let mut errors = Vec::new();

    for (field, value) in [
        ("graph_id", graph.graph_id.as_str()),
        ("status", graph.status.as_str()),
        ("geography", graph.geography.as_str()),
    ] {
        if !non_empty(value) {
            errors.push(UtilityInferenceError::EmptyIdentifier(field));
        }
    }

    if graph.status != "synthetic_candidate_utility_inference_not_authoritative"
        || graph.geography != "abstract_non_geographic"
        || graph.claim_boundary.trim().len() < 20
    {
        errors.push(UtilityInferenceError::InvalidGraphClaimBoundary);
    }

    let event_ids = unique_ids(
        graph.observed_events.iter().map(|event| event.event_id.as_str()),
        "event_id",
        &mut errors,
    );
    let asset_ids = unique_ids(
        graph
            .candidate_assets
            .iter()
            .map(|asset| asset.candidate_asset_id.as_str()),
        "candidate_asset_id",
        &mut errors,
    );
    let node_ids = unique_ids(
        graph
            .candidate_nodes
            .iter()
            .map(|node| node.candidate_node_id.as_str()),
        "candidate_node_id",
        &mut errors,
    );
    let connection_ids = unique_ids(
        graph
            .candidate_connections
            .iter()
            .map(|connection| connection.connection_id.as_str()),
        "connection_id",
        &mut errors,
    );

    let mut event_association_mass: BTreeMap<&str, f64> = BTreeMap::new();
    let mut event_association_count: BTreeMap<&str, usize> = BTreeMap::new();

    for event in &graph.observed_events {
        for (field, value) in [
            ("observation_id", event.observation_id.as_str()),
            ("source_evidence_id", event.source_evidence_id.as_str()),
            ("acquisition_id", event.acquisition_id.as_str()),
            ("spatial_support_id", event.spatial_support_id.as_str()),
            ("sensor_configuration_id", event.sensor_configuration_id.as_str()),
        ] {
            if !non_empty(value) {
                errors.push(UtilityInferenceError::EmptyIdentifier(field));
            }
        }
    }

    for asset in &graph.candidate_assets {
        if !asset.candidate_asset_id.starts_with("candidate:")
            || asset.candidate_asset_id.len() <= "candidate:".len()
        {
            errors.push(UtilityInferenceError::InvalidCandidateAssetNamespace(
                asset.candidate_asset_id.clone(),
            ));
        }
        if !non_empty(&asset.geometry_claim_id) {
            errors.push(UtilityInferenceError::EmptyIdentifier("geometry_claim_id"));
        }
        if asset.supporting_evidence_ids.is_empty()
            || asset.supporting_evidence_ids.iter().any(|id| !non_empty(id))
        {
            errors.push(UtilityInferenceError::CandidateAssetWithoutEvidence(
                asset.candidate_asset_id.clone(),
            ));
        }
        if !valid_probability(asset.confidence) {
            errors.push(UtilityInferenceError::InvalidConfidence(
                asset.candidate_asset_id.clone(),
            ));
        }
        if asset.target_kinds.is_empty() {
            errors.push(UtilityInferenceError::EmptyIdentifier("target_kinds"));
        }
    }

    for association in &graph.event_asset_associations {
        if !event_ids.contains(association.event_id.as_str()) {
            errors.push(UtilityInferenceError::UnknownAssociationEvent(
                association.event_id.clone(),
            ));
        }
        if !asset_ids.contains(association.candidate_asset_id.as_str()) {
            errors.push(UtilityInferenceError::UnknownAssociationAsset(
                association.candidate_asset_id.clone(),
            ));
        }
        if !valid_probability(association.probability) {
            errors.push(UtilityInferenceError::InvalidAssociationProbability);
            continue;
        }
        *event_association_mass
            .entry(association.event_id.as_str())
            .or_insert(0.0) += association.probability;
        *event_association_count
            .entry(association.candidate_asset_id.as_str())
            .or_insert(0) += 1;
    }

    for (event_id, mass) in event_association_mass {
        if mass > 1.0 + PROBABILITY_TOLERANCE {
            errors.push(UtilityInferenceError::AssociationMassExceedsOne(
                event_id.to_string(),
            ));
        }
    }

    for asset_id in &asset_ids {
        if event_association_count.get(asset_id).copied().unwrap_or(0) == 0 {
            errors.push(UtilityInferenceError::CandidateAssetWithoutEvents(
                (*asset_id).to_string(),
            ));
        }
    }

    for node in &graph.candidate_nodes {
        if !non_empty(&node.geometry_claim_id) {
            errors.push(UtilityInferenceError::EmptyIdentifier("node_geometry_claim_id"));
        }
        if node.supporting_evidence_ids.is_empty()
            || node.supporting_evidence_ids.iter().any(|id| !non_empty(id))
        {
            errors.push(UtilityInferenceError::CandidateNodeWithoutEvidence(
                node.candidate_node_id.clone(),
            ));
        }
    }

    for connection in &graph.candidate_connections {
        if !node_ids.contains(connection.from_node_candidate_id.as_str()) {
            errors.push(UtilityInferenceError::UnknownConnectionNode(
                connection.from_node_candidate_id.clone(),
            ));
        }
        if !node_ids.contains(connection.to_node_candidate_id.as_str()) {
            errors.push(UtilityInferenceError::UnknownConnectionNode(
                connection.to_node_candidate_id.clone(),
            ));
        }
        if connection.from_node_candidate_id == connection.to_node_candidate_id {
            errors.push(UtilityInferenceError::SelfConnection(
                connection.connection_id.clone(),
            ));
        }
        if connection.supporting_evidence_ids.is_empty()
            || connection.supporting_evidence_ids.iter().any(|id| !non_empty(id))
        {
            errors.push(UtilityInferenceError::ConnectionWithoutEvidence(
                connection.connection_id.clone(),
            ));
        }
        if !valid_probability(connection.confidence) {
            errors.push(UtilityInferenceError::InvalidConfidence(
                connection.connection_id.clone(),
            ));
        }
    }

    // Keep deterministic error order for stable qualification receipts.
    let _ = connection_ids;
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_graph() -> UtilityInferenceGraph {
        UtilityInferenceGraph {
            graph_id: "utility-inference-synthetic-001".to_string(),
            status: "synthetic_candidate_utility_inference_not_authoritative".to_string(),
            geography: "abstract_non_geographic".to_string(),
            claim_boundary: "Candidate asset graph only; no authoritative asset identity or physical utility truth.".to_string(),
            observed_events: vec![
                ObservedUtilityEvent {
                    event_id: "event-1".to_string(),
                    observation_id: "observation-1".to_string(),
                    source_evidence_id: "gpr-evidence-1".to_string(),
                    acquisition_id: "acquisition-a".to_string(),
                    spatial_support_id: "support-a".to_string(),
                    sensor_configuration_id: "gpr-400mhz-v1".to_string(),
                },
                ObservedUtilityEvent {
                    event_id: "event-2".to_string(),
                    observation_id: "observation-2".to_string(),
                    source_evidence_id: "gpr-evidence-2".to_string(),
                    acquisition_id: "acquisition-b".to_string(),
                    spatial_support_id: "support-b".to_string(),
                    sensor_configuration_id: "gpr-400mhz-v1".to_string(),
                },
                ObservedUtilityEvent {
                    event_id: "event-clutter".to_string(),
                    observation_id: "observation-clutter".to_string(),
                    source_evidence_id: "gpr-evidence-clutter".to_string(),
                    acquisition_id: "acquisition-c".to_string(),
                    spatial_support_id: "support-c".to_string(),
                    sensor_configuration_id: "gpr-400mhz-v1".to_string(),
                },
            ],
            candidate_assets: vec![CandidateUtilityAsset {
                candidate_asset_id: "candidate:utility-1".to_string(),
                target_kinds: vec![UtilityKind::Water, UtilityKind::Unknown],
                geometry_claim_id: "geometry-claim-1".to_string(),
                supporting_evidence_ids: vec![
                    "gpr-evidence-1".to_string(),
                    "gpr-evidence-2".to_string(),
                ],
                claim_status: CandidateClaimStatus::Candidate,
                confidence: 0.72,
            }],
            event_asset_associations: vec![
                EventAssetAssociation {
                    event_id: "event-1".to_string(),
                    candidate_asset_id: "candidate:utility-1".to_string(),
                    probability: 0.70,
                },
                EventAssetAssociation {
                    event_id: "event-2".to_string(),
                    candidate_asset_id: "candidate:utility-1".to_string(),
                    probability: 0.85,
                },
            ],
            candidate_nodes: vec![
                CandidateNetworkNode {
                    candidate_node_id: "candidate-node:a".to_string(),
                    geometry_claim_id: "node-geometry-a".to_string(),
                    supporting_evidence_ids: vec!["gpr-evidence-1".to_string()],
                },
                CandidateNetworkNode {
                    candidate_node_id: "candidate-node:b".to_string(),
                    geometry_claim_id: "node-geometry-b".to_string(),
                    supporting_evidence_ids: vec!["gpr-evidence-2".to_string()],
                },
            ],
            candidate_connections: vec![CandidateNetworkConnection {
                connection_id: "candidate-connection-1".to_string(),
                from_node_candidate_id: "candidate-node:a".to_string(),
                to_node_candidate_id: "candidate-node:b".to_string(),
                relation: CandidateNetworkRelation::Continuation,
                supporting_evidence_ids: vec!["gpr-evidence-1".to_string(), "gpr-evidence-2".to_string()],
                confidence: 0.65,
            }],
        }
    }

    #[test]
    fn multiple_events_can_support_one_candidate_asset_without_creating_multiple_assets() {
        let graph = valid_graph();
        assert!(validate_utility_inference_graph(&graph).is_empty());
        assert_eq!(graph.candidate_assets.len(), 1);
        assert_eq!(graph.event_asset_associations.len(), 2);
    }

    #[test]
    fn unassigned_event_probability_is_preserved_as_abstention_mass() {
        let graph = valid_graph();
        let assigned: f64 = graph
            .event_asset_associations
            .iter()
            .filter(|association| association.event_id == "event-1")
            .map(|association| association.probability)
            .sum();
        assert!(assigned < 1.0);
        assert!((1.0 - assigned - 0.30).abs() < PROBABILITY_TOLERANCE);
        assert!(validate_utility_inference_graph(&graph).is_empty());
    }

    #[test]
    fn event_association_mass_cannot_exceed_one() {
        let mut graph = valid_graph();
        graph.event_asset_associations.push(EventAssetAssociation {
            event_id: "event-1".to_string(),
            candidate_asset_id: "candidate:utility-1".to_string(),
            probability: 0.5,
        });
        assert!(validate_utility_inference_graph(&graph)
            .contains(&UtilityInferenceError::AssociationMassExceedsOne("event-1".to_string())));
    }

    #[test]
    fn candidate_asset_must_not_claim_owner_or_muddi_identity() {
        let mut graph = valid_graph();
        graph.candidate_assets[0].candidate_asset_id = "owner-system:ASSET-0001".to_string();
        assert!(validate_utility_inference_graph(&graph).contains(
            &UtilityInferenceError::InvalidCandidateAssetNamespace(
                "owner-system:ASSET-0001".to_string()
            )
        ));
    }

    #[test]
    fn event_and_candidate_asset_references_must_resolve() {
        let mut graph = valid_graph();
        graph.event_asset_associations[0].event_id = "missing-event".to_string();
        graph.event_asset_associations[1].candidate_asset_id = "candidate:missing".to_string();
        let errors = validate_utility_inference_graph(&graph);
        assert!(errors.contains(&UtilityInferenceError::UnknownAssociationEvent("missing-event".to_string())));
        assert!(errors.contains(&UtilityInferenceError::UnknownAssociationAsset("candidate:missing".to_string())));
    }

    #[test]
    fn candidate_asset_without_event_support_is_rejected() {
        let mut graph = valid_graph();
        graph.candidate_assets[0].candidate_asset_id = "candidate:unsupported".to_string();
        assert!(validate_utility_inference_graph(&graph)
            .contains(&UtilityInferenceError::CandidateAssetWithoutEvents("candidate:unsupported".to_string())));
    }

    #[test]
    fn unresolved_crossing_is_preserved_as_non_connection() {
        let mut graph = valid_graph();
        graph.candidate_connections[0].relation = CandidateNetworkRelation::CrossingUnresolved;
        assert!(validate_utility_inference_graph(&graph).is_empty());
        assert_eq!(
            graph.candidate_connections[0].relation,
            CandidateNetworkRelation::CrossingUnresolved
        );
    }

    #[test]
    fn network_connection_requires_evidence_and_known_nodes() {
        let mut graph = valid_graph();
        graph.candidate_connections[0].supporting_evidence_ids.clear();
        graph.candidate_connections[0].to_node_candidate_id = "missing-node".to_string();
        let errors = validate_utility_inference_graph(&graph);
        assert!(errors.contains(&UtilityInferenceError::ConnectionWithoutEvidence(
            "candidate-connection-1".to_string()
        )));
        assert!(errors.contains(&UtilityInferenceError::UnknownConnectionNode("missing-node".to_string())));
    }
}
