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
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProjectionEdgeKind {
    Required,
    Alternative,
}

impl ProjectionEdgeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Alternative => "alternative",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionEdge {
    pub from: String,
    pub to: String,
    pub kind: ProjectionEdgeKind,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceReference {
    pub owner_node_id: String,
    pub evidence_id: String,
    pub resolved_anchor: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H3CellSummary {
    /// Canonical lower-case H3 index text.
    pub cell: String,
    /// Exact instance IDs retained so aggregation is reversible.
    pub member_instance_ids: Vec<String>,
    pub capability_ids: Vec<CapabilityId>,
    pub scenario_instance_count: usize,
    pub externally_qualified_reference_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

        let mut capability_ids = BTreeSet::new();
        capability_ids.extend(required.iter().cloned());

        // Alternatives are shown explicitly so a viewer can distinguish
        // declared choices from requirements. Their own dependencies are not
        // silently pulled into the required closure.
        for id in required {
            if let Some(capability) = graph.get(&id) {
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
            nodes.push(ProjectionNode {
                node_id: node_id.clone(),
                kind: ProjectionNodeKind::Capability,
                capability_id: capability.id.clone(),
                label: capability.name.clone(),
                provenance: capability.provenance.clone(),
                claim_ceiling: capability.claim_ceiling.clone(),
                evidence_refs: capability
                    .evidence
                    .iter()
                    .map(|e| e.evidence_id.clone())
                    .collect(),
                lifecycle: None,
                availability: None,
                qualification_reference_present: false,
                h3_cell: None,
            });

            for evidence in &capability.evidence {
                evidence_refs.push(EvidenceReference {
                    owner_node_id: node_id.clone(),
                    evidence_id: evidence.evidence_id.clone(),
                    resolved_anchor: true,
                });
            }
        }

        // Required edges come only from the root's transitive required closure.
        for capability_id in capability_ids.iter() {
            let capability = graph
                .get(capability_id)
                .expect("capability_ids are derived from graph");
            for dependency in capability.sorted_required_dependencies() {
                if capability_ids.contains(&dependency) {
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
            if !capability_ids.contains(&instance.capability_id) {
                continue;
            }
            if !instance_ids.insert(instance.instance_id.clone()) {
                return Err(ProjectionError::DuplicateInstanceId(
                    instance.instance_id.clone(),
                ));
            }
            let h3_cell = LatLng::new(instance.location.lat, instance.location.lon)
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
                label: instance.location.label.clone(),
                provenance: instance.provenance.clone(),
                claim_ceiling: instance.claim_ceiling.clone(),
                evidence_refs: instance.evidence_refs.clone(),
                lifecycle: Some(instance.state.lifecycle),
                availability: Some(instance.state.availability),
                qualification_reference_present: instance.is_qualified(),
                h3_cell,
            });

            edges.push(ProjectionEdge {
                from: node_id,
                to: format!("capability:{}", instance.capability_id),
                kind: ProjectionEdgeKind::Required,
                rationale: Some("instance instantiates capability definition".into()),
            });

            for evidence_id in &instance.evidence_refs {
                let resolved_anchor = graph
                    .get(&instance.capability_id)
                    .map(|c| c.evidence.iter().any(|e| &e.evidence_id == evidence_id))
                    .unwrap_or(false);
                evidence_refs.push(EvidenceReference {
                    owner_node_id: format!("instance:{}", instance.instance_id),
                    evidence_id: evidence_id.clone(),
                    resolved_anchor,
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
        let mut s = String::new();
        s.push_str(self.root_capability_id.as_str());
        for node in &self.nodes {
            s.push('|');
            s.push_str(&node.node_id);
            s.push(':');
            s.push_str(node.kind.label());
            s.push(':');
            s.push_str(node.provenance.kind.label());
        }
        for edge in &self.edges {
            s.push('|');
            s.push_str(&edge.from);
            s.push('>');
            s.push_str(&edge.to);
            s.push(':');
            s.push_str(edge.kind.label());
        }
        s
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
    fn missing_instance_capability_is_rejected_when_in_scope() {
        let fixture = bootstrap_fixture();
        let mut instance = fixture.instance;
        instance.capability_id = CapabilityId::new("missing").unwrap();

        // The instance is out of the selected capability neighborhood, so it
        // must not become a floating renderer node.
        let projection = AtlasCapabilityProjectionV1::build(
            &fixture.graph,
            &[instance],
            &fixture.root,
            Resolution::Two,
        )
        .unwrap();
        assert!(!projection
            .nodes
            .iter()
            .any(|n| n.kind == ProjectionNodeKind::Instance));
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
}
