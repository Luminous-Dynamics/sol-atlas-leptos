// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic renderer projection for the capability atlas.
//!
//! This module is intentionally a view-model boundary:
//! canonical capability semantics stay in capability.rs, while the renderer
//! receives stable node/edge references, provenance, claim ceilings, and
//! reversible spatial aggregation hints.
//!
//! No projection value means "true", "available", "qualified", or "safe" by
//! itself. Those meanings remain owned by the corresponding semantic/evidence
//! systems.

use crate::capability::{
    CapabilityGraph, CapabilityGraphError, CapabilityId, CapabilityInstance, CapabilityLifecycle,
    CapabilityAvailability, CapabilityProvenance,
};
use crate::types::DataKind;
use h3o::{LatLng, Resolution};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ProjectionNodeKind {
    Capability,
    Instance,
}

impl ProjectionNodeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Capability => "capability",
            Self::Instance => "instance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ProjectionEdgeKind {
    InstanceOf,
    Required,
    Alternative,
}

impl ProjectionEdgeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::InstanceOf => "instance-of",
            Self::Required => "required",
            Self::Alternative => "alternative",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionNode {
    pub node_id: String,
    pub kind: ProjectionNodeKind,
    pub capability_id: CapabilityId,
    pub label: String,
    pub provenance: CapabilityProvenance,
    pub claim_ceiling: String,
    pub evidence_refs: Vec<String>,
    pub lifecycle: Option<CapabilityLifecycle>,
    pub availability: Option<CapabilityAvailability>,
    pub qualification_reference_present: bool,
    /// H3 is an aggregation/cache key, never the node identity.
    pub h3_cell: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionEdge {
    pub from: String,
    pub to: String,
    pub kind: ProjectionEdgeKind,
    pub rationale: Option<String>,
}

/// Local matching state for an evidence identifier.
///
/// This does not mean an external artifact was retrieved, authenticated, or
/// verified. The projection currently has no external evidence resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceReferenceStatus {
    /// The identifier appears in the owning capability's declared evidence list.
    DeclaredByCapability,
    /// An instance references an identifier not declared by its capability.
    NotDeclaredByCapability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReference {
    pub owner_node_id: String,
    pub evidence_id: String,
    pub status: EvidenceReferenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct H3CellSummary {
    /// Canonical lower-case H3 index text.
    pub cell: String,
    /// Exact instance IDs retained so aggregation is reversible.
    pub member_instance_ids: Vec<String>,
    pub capability_ids: Vec<CapabilityId>,
    pub scenario_instance_count: usize,
    pub externally_qualified_reference_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasCapabilityProjectionV1 {
    pub root_capability_id: CapabilityId,
    pub nodes: Vec<ProjectionNode>,
    pub edges: Vec<ProjectionEdge>,
    pub evidence_refs: Vec<EvidenceReference>,
    pub h3_cells: Vec<H3CellSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionError {
    Graph(CapabilityGraphError),
    DuplicateInstanceId(String),
    MissingCapabilityForInstance { instance_id: String, capability_id: CapabilityId },
    InvalidInstanceLocation { instance_id: String },
}

impl From<CapabilityGraphError> for ProjectionError {
    fn from(value: CapabilityGraphError) -> Self {
        Self::Graph(value)
    }
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(err) => write!(f, "capability graph error: {err}"),
            Self::DuplicateInstanceId(id) => write!(f, "duplicate instance id: {id}"),
            Self::MissingCapabilityForInstance { instance_id, capability_id } => {
                write!(f, "instance {instance_id} references missing capability {capability_id}")
            }
            Self::InvalidInstanceLocation { instance_id } => {
                write!(f, "instance {instance_id} has invalid geographic coordinates")
            }
        }
    }
}

impl AtlasCapabilityProjectionV1 {
    /// Project a root capability and its declared required/alternative neighborhood.
    ///
    /// Required dependencies are included transitively. Alternative candidates
    /// are included as explicit edge targets but are never rewritten as required.
    pub fn build(
        graph: &CapabilityGraph,
        instances: &[CapabilityInstance],
        root: &CapabilityId,
        h3_resolution: Resolution,
    ) -> Result<Self, ProjectionError> {
        let required = graph.required_closure(root)?;
        let required_ids: BTreeSet<CapabilityId> = required.iter().cloned().collect();

        let mut capability_ids = required_ids.clone();

        // Alternatives are shown explicitly so a viewer can distinguish
        // declared choices from requirements. Their own dependencies are not
        // silently pulled into the required closure.
        for id in &required {
            if let Some(capability) = graph.get(id) {
                for alternative in capability.sorted_alternatives() {
                    capability_ids.insert(alternative.candidate);
                    capability_ids.insert(alternative.replaces);
                }
            }
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut evidence_refs = Vec::new();

        for capability_id in capability_ids.iter() {
            let capability = graph
                .get(capability_id)
                .expect("capability_ids are derived from graph");

            let node_id = format!("capability:{capability_id}");
            let mut capability_evidence_refs: Vec<String> = capability
                .evidence
                .iter()
                .map(|e| e.evidence_id.clone())
                .collect();
            capability_evidence_refs.sort();
            capability_evidence_refs.dedup();
            nodes.push(ProjectionNode {
                node_id: node_id.clone(),
                kind: ProjectionNodeKind::Capability,
                capability_id: capability.id.clone(),
                label: capability.name.clone(),
                provenance: capability.provenance.clone(),
                claim_ceiling: capability.claim_ceiling.clone(),
                evidence_refs: capability_evidence_refs,
                lifecycle: None,
                availability: None,
                qualification_reference_present: false,
                h3_cell: None,
            });

            for evidence in &capability.evidence {
                evidence_refs.push(EvidenceReference {
                    owner_node_id: node_id.clone(),
                    evidence_id: evidence.evidence_id.clone(),
                    status: EvidenceReferenceStatus::DeclaredByCapability,
                });
            }
        }

        // Only nodes in the root's required closure emit required/alternative
        // edges. Alternative candidates may be shown as nodes for comparison,
        // but their own dependencies are not silently asserted as required.
        for capability_id in required_ids.iter() {
            let capability = graph
                .get(capability_id)
                .expect("required closure is derived from graph");
            for dependency in capability.sorted_required_dependencies() {
                if required_ids.contains(&dependency) {
                    edges.push(ProjectionEdge {
                        from: format!("capability:{capability_id}"),
                        to: format!("capability:{dependency}"),
                        kind: ProjectionEdgeKind::Required,
                        rationale: None,
                    });
                }
            }
            for alternative in capability.sorted_alternatives() {
                if capability_ids.contains(&alternative.candidate) {
                    edges.push(ProjectionEdge {
                        from: format!("capability:{capability_id}"),
                        to: format!("capability:{}", alternative.candidate),
                        kind: ProjectionEdgeKind::Alternative,
                        rationale: Some(alternative.rationale),
                    });
                }
            }
        }

        let mut instance_ids = BTreeSet::new();
        for instance in instances {
            // Validate input graph integrity before applying projection scope:
            // an invalid external reference must not disappear as a filtered node.
            if graph.get(&instance.capability_id).is_none() {
                return Err(ProjectionError::MissingCapabilityForInstance {
                    instance_id: instance.instance_id.clone(),
                    capability_id: instance.capability_id.clone(),
                });
            }
            if !instance_ids.insert(instance.instance_id.clone()) {
                return Err(ProjectionError::DuplicateInstanceId(
                    instance.instance_id.clone(),
                ));
            }
            if !capability_ids.contains(&instance.capability_id) {
                continue;
            }
            let h3_cell = LatLng::new(instance.location.lat(), instance.location.lon())
                .map(|ll| ll.to_cell(h3_resolution).to_string())
                .ok();
            if h3_cell.is_none() {
                return Err(ProjectionError::InvalidInstanceLocation {
                    instance_id: instance.instance_id.clone(),
                });
            }

            let node_id = format!("instance:{}", instance.instance_id);
            nodes.push(ProjectionNode {
                node_id: node_id.clone(),
                kind: ProjectionNodeKind::Instance,
                capability_id: instance.capability_id.clone(),
                label: instance.location.label().to_string(),
                provenance: instance.provenance.clone(),
                claim_ceiling: instance.claim_ceiling.clone(),
                evidence_refs: {
                    let mut refs = instance.evidence_refs.clone();
                    refs.sort();
                    refs.dedup();
                    refs
                },
                lifecycle: Some(instance.state.lifecycle),
                availability: Some(instance.state.availability),
                qualification_reference_present: instance.is_qualified(),
                h3_cell,
            });

            edges.push(ProjectionEdge {
                from: node_id,
                to: format!("capability:{}", instance.capability_id),
                kind: ProjectionEdgeKind::InstanceOf,
                rationale: Some("instance instantiates capability definition".into()),
            });

            for evidence_id in &instance.evidence_refs {
                let declared_by_capability = graph
                    .get(&instance.capability_id)
                    .map(|c| c.evidence.iter().any(|e| &e.evidence_id == evidence_id))
                    .unwrap_or(false);
                evidence_refs.push(EvidenceReference {
                    owner_node_id: format!("instance:{}", instance.instance_id),
                    evidence_id: evidence_id.clone(),
                    status: if declared_by_capability {
                        EvidenceReferenceStatus::DeclaredByCapability
                    } else {
                        EvidenceReferenceStatus::NotDeclaredByCapability
                    },
                });
            }
        }

        nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        edges.sort_by(|a, b| {
            a.from
                .cmp(&b.from)
                .then_with(|| a.to.cmp(&b.to))
                .then_with(|| a.kind.cmp(&b.kind))
                .then_with(|| a.rationale.cmp(&b.rationale))
        });
        evidence_refs.sort_by(|a, b| {
            a.owner_node_id
                .cmp(&b.owner_node_id)
                .then_with(|| a.evidence_id.cmp(&b.evidence_id))
        });

        let h3_cells = summarize_h3(&nodes);

        Ok(Self {
            root_capability_id: root.clone(),
            nodes,
            edges,
            evidence_refs,
            h3_cells,
        })
    }

    /// Projection identity is stable for identical semantic input.
    pub fn deterministic_key(&self) -> String {
        serde_json::to_string(self).expect("projection contains only serializable fields")
    }
}

fn summarize_h3(nodes: &[ProjectionNode]) -> Vec<H3CellSummary> {
    let mut by_cell: BTreeMap<String, H3CellSummary> = BTreeMap::new();

    for node in nodes.iter().filter(|n| n.kind == ProjectionNodeKind::Instance) {
        let Some(cell) = &node.h3_cell else { continue };
        let entry = by_cell.entry(cell.clone()).or_insert_with(|| H3CellSummary {
            cell: cell.clone(),
            member_instance_ids: Vec::new(),
            capability_ids: Vec::new(),
            scenario_instance_count: 0,
            externally_qualified_reference_count: 0,
        });

        entry.member_instance_ids.push(
            node.node_id
                .strip_prefix("instance:")
                .unwrap_or(&node.node_id)
                .to_string(),
        );
        if !entry.capability_ids.contains(&node.capability_id) {
            entry.capability_ids.push(node.capability_id.clone());
        }
        if node.provenance.kind == DataKind::Scenario {
            entry.scenario_instance_count += 1;
        }
        if node.qualification_reference_present {
            entry.externally_qualified_reference_count += 1;
        }
    }

    let mut result: Vec<_> = by_cell.into_values().collect();
    for cell in &mut result {
        cell.member_instance_ids.sort();
        cell.capability_ids.sort();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::bootstrap_fixture;

    #[test]
    fn projection_preserves_capability_instance_separation() {
        let fixture = bootstrap_fixture();
        let projection = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[fixture.instance.clone()],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();

        assert!(projection
            .nodes
            .iter()
            .any(|n| n.kind == ProjectionNodeKind::Capability
                && n.capability_id == fixture.root));
        assert!(projection
            .nodes
            .iter()
            .any(|n| n.kind == ProjectionNodeKind::Instance
                && n.capability_id == fixture.root));
        assert!(projection
            .edges
            .iter()
            .any(|e| e.from == "instance:fixture.bootstrap.node"));
    }

    #[test]
    fn projection_is_deterministic_under_instance_input_order() {
        let fixture = bootstrap_fixture();
        let mut instances = vec![fixture.instance.clone()];
        let p1 = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &instances,
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();
        instances.reverse();
        let p2 = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &instances,
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();

        assert_eq!(p1.nodes, p2.nodes);
        assert_eq!(p1.edges, p2.edges);
        assert_eq!(p1.deterministic_key(), p2.deterministic_key());
    }

    #[test]
    fn scenario_ceiling_survives_projection() {
        let fixture = bootstrap_fixture();
        let projection = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[fixture.instance],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();

        for node in projection.nodes {
            assert_eq!(node.provenance.kind, DataKind::Scenario);
            assert!(node.claim_ceiling.contains("synthetic"));
        }
    }

    #[test]
    fn h3_aggregation_is_reversible_to_instance_ids() {
        let fixture = bootstrap_fixture();
        let projection = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[fixture.instance.clone()],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();

        assert_eq!(projection.h3_cells.len(), 1);
        assert_eq!(
            projection.h3_cells[0].member_instance_ids,
            vec!["fixture.bootstrap.node"]
        );
        assert_eq!(
            projection.h3_cells[0].capability_ids,
            vec![fixture.root]
        );
    }

    #[test]
    fn missing_instance_capability_fails_closed() {
        let fixture = bootstrap_fixture();
        let mut instance = fixture.instance;
        instance.capability_id = CapabilityId::new("missing").unwrap();

        let err = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[instance],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProjectionError::MissingCapabilityForInstance { .. }
        ));
    }

    #[test]
    fn duplicate_in_scope_instance_ids_fail_closed() {
        let fixture = bootstrap_fixture();
        let instances = vec![fixture.instance.clone(), fixture.instance];
        let err = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &instances,
            &fixture.root,
            Resolution::Two,
        )
        .unwrap_err();
        assert!(matches!(err, ProjectionError::DuplicateInstanceId(_)));
    }

    #[test]
    fn alternative_candidate_dependencies_do_not_become_root_requirements() {
        use crate::capability::{Capability, CapabilityAlternative};

        let fixture = bootstrap_fixture();
        let energy_id = CapabilityId::new("energy").unwrap();
        let mut definitions: Vec<Capability> =
            fixture.graph.capabilities().cloned().collect();
        let energy_idx = definitions
            .iter()
            .position(|capability| capability.id == energy_id)
            .unwrap();

        let candidate_id = CapabilityId::new("energy-backup").unwrap();
        let mut candidate = definitions[energy_idx].clone();
        candidate.id = candidate_id.clone();
        candidate.name = "Energy backup (candidate)".into();
        // Knowledge is already in the root closure. A naive projection that
        // loops over every displayed node would incorrectly emit this as a
        // required edge from the alternative.
        candidate.required_dependencies = vec![CapabilityId::new("knowledge").unwrap()];
        candidate.alternatives.clear();

        definitions[energy_idx].alternatives.push(CapabilityAlternative {
            candidate: candidate_id.clone(),
            replaces: energy_id.clone(),
            rationale: "explicit comparison candidate".into(),
            evidence_refs: vec![],
        });
        definitions.push(candidate);

        let graph = CapabilityGraph::new(definitions).unwrap();
        let projection = AtlasCapabilityProjectionV1::build(
            &graph,
            &[],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();

        assert!(projection.edges.iter().any(|edge| {
            edge.from == "capability:energy"
                && edge.to == "capability:energy-backup"
                && edge.kind == ProjectionEdgeKind::Alternative
        }));
        assert!(!projection.edges.iter().any(|edge| {
            edge.from == "capability:energy-backup"
                && edge.to == "capability:knowledge"
                && edge.kind == ProjectionEdgeKind::Required
        }));
    }

    #[test]
    fn evidence_reference_status_does_not_claim_external_resolution() {
        let fixture = bootstrap_fixture();
        let mut instance = fixture.instance.clone();
        instance.evidence_refs = vec!["external-record-not-in-capability".into()];

        let projection = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[instance],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();

        let reference = projection
            .evidence_refs
            .iter()
            .find(|reference| {
                reference.owner_node_id == "instance:fixture.bootstrap.node"
                    && reference.evidence_id == "external-record-not-in-capability"
            })
            .unwrap();
        assert_eq!(
            reference.status,
            EvidenceReferenceStatus::NotDeclaredByCapability
        );

        // The state is declaration-local, not a statement about remote truth,
        // content availability, integrity verification, or qualification.
        assert!(projection.evidence_refs.iter().all(|reference| {
            reference.status == EvidenceReferenceStatus::DeclaredByCapability
                || reference.status == EvidenceReferenceStatus::NotDeclaredByCapability
        }));
    }

    #[test]
    fn unknown_capability_references_fail_before_scope_filtering() {
        let fixture = bootstrap_fixture();
        let mut instance = fixture.instance;
        instance.capability_id = CapabilityId::new("missing").unwrap();
        let err = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[instance],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProjectionError::MissingCapabilityForInstance { .. }
        ));
    }
}
