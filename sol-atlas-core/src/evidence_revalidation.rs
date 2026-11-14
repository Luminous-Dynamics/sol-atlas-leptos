// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic downstream invalidation for evidence-dependent claims.
//!
//! When a source, coordinate transform, calibration or model revision changes,
//! descendants must not continue to appear current. This module computes the
//! affected dependency closure and an upstream-before-downstream revalidation
//! order. It does not mutate an evidence store, authenticate source revisions,
//! or promote any candidate claim to an authoritative fact.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyNodeKind {
    SourceEvidence,
    SpatialTransform,
    SensorCalibration,
    Observation,
    Event,
    AssetHypothesis,
    GeometryClaim,
    NetworkHypothesis,
    Decision,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyNodeStatus {
    Current,
    Stale,
    Superseded,
    Conflicted,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceDependencyNode {
    pub node_id: String,
    pub node_kind: DependencyNodeKind,
    /// Immutable revision identifier, source digest, or qualified model receipt.
    pub revision_id: String,
    /// Direct upstream inputs. Revalidation order follows these edges.
    pub dependency_ids: Vec<String>,
    pub status: DependencyNodeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceDependencyGraph {
    pub graph_id: String,
    pub snapshot_id: String,
    pub nodes: Vec<EvidenceDependencyNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevalidationPlan {
    pub graph_id: String,
    pub snapshot_id: String,
    /// Root inputs whose revisions changed.
    pub changed_node_ids: Vec<String>,
    /// Direct and transitive descendants, excluding the changed roots.
    pub invalidated_node_ids: Vec<String>,
    /// Descendants in deterministic upstream-before-downstream order.
    pub revalidation_order: Vec<String>,
    /// Nodes neither changed nor downstream of a changed input.
    pub unaffected_node_ids: Vec<String>,
    pub status: String,
    pub claim_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceDependencyError {
    EmptyIdentifier(&'static str),
    EmptyGraph,
    DuplicateNode(String),
    DuplicateDependency(String, String),
    SelfDependency(String),
    UnknownDependency(String, String),
    UnknownChangedNode(String),
    DependencyCycle(Vec<String>),
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

/// Return a canonical topological ordering. BTree-based ready queues make the
/// output independent of the authored node order when multiple nodes are ready.
fn topological_order(
    graph: &EvidenceDependencyGraph,
) -> Result<Vec<String>, Vec<EvidenceDependencyError>> {
    let mut errors = Vec::new();
    if !non_empty(&graph.graph_id) {
        errors.push(EvidenceDependencyError::EmptyIdentifier("graph_id"));
    }
    if !non_empty(&graph.snapshot_id) {
        errors.push(EvidenceDependencyError::EmptyIdentifier("snapshot_id"));
    }

    if graph.nodes.is_empty() {
        errors.push(EvidenceDependencyError::EmptyGraph);
    }

    let mut nodes = BTreeMap::new();
    for node in &graph.nodes {
        if !non_empty(&node.node_id) {
            errors.push(EvidenceDependencyError::EmptyIdentifier("node_id"));
        }
        if !non_empty(&node.revision_id) {
            errors.push(EvidenceDependencyError::EmptyIdentifier("revision_id"));
        }
        if nodes.insert(node.node_id.as_str(), node).is_some() {
            errors.push(EvidenceDependencyError::DuplicateNode(
                node.node_id.clone(),
            ));
        }
    }

    let mut indegree: BTreeMap<&str, usize> =
        nodes.keys().map(|node_id| (*node_id, 0usize)).collect();
    let mut children: BTreeMap<&str, BTreeSet<&str>> =
        nodes.keys().map(|node_id| (*node_id, BTreeSet::new())).collect();

    for node in &graph.nodes {
        let mut local_dependencies = BTreeSet::new();
        for dependency_id in &node.dependency_ids {
            if !local_dependencies.insert(dependency_id.as_str()) {
                errors.push(EvidenceDependencyError::DuplicateDependency(
                    node.node_id.clone(),
                    dependency_id.clone(),
                ));
                continue;
            }
            if dependency_id == &node.node_id {
                errors.push(EvidenceDependencyError::SelfDependency(
                    node.node_id.clone(),
                ));
                continue;
            }
            if !nodes.contains_key(dependency_id.as_str()) {
                errors.push(EvidenceDependencyError::UnknownDependency(
                    node.node_id.clone(),
                    dependency_id.clone(),
                ));
                continue;
            }
            if let Some(value) = indegree.get_mut(node.node_id.as_str()) {
                *value += 1;
            }
            if let Some(descendants) = children.get_mut(dependency_id.as_str()) {
                descendants.insert(node.node_id.as_str());
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut ready: BTreeSet<&str> = indegree
        .iter()
        .filter_map(|(node_id, degree)| (*degree == 0).then_some(*node_id))
        .collect();
    let mut order = Vec::with_capacity(nodes.len());

    while let Some(node_id) = ready.pop_first() {
        order.push(node_id.to_string());
        if let Some(descendants) = children.get(node_id) {
            for child_id in descendants {
                if let Some(degree) = indegree.get_mut(*child_id) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(*child_id);
                    }
                }
            }
        }
    }

    if order.len() != nodes.len() {
        let cyclic_nodes = indegree
            .iter()
            .filter_map(|(node_id, degree)| (*degree > 0).then_some((*node_id).to_string()))
            .collect();
        return Err(vec![EvidenceDependencyError::DependencyCycle(
            cyclic_nodes,
        )]);
    }

    Ok(order)
}

/// Validate graph identity, node revisions, dependency references and DAG
/// acyclicity. A valid graph is still only a dependency declaration; it does
/// not prove the declared relationships are complete or scientifically true.
pub fn validate_evidence_dependency_graph(
    graph: &EvidenceDependencyGraph,
) -> Result<(), Vec<EvidenceDependencyError>> {
    topological_order(graph).map(|_| ())
}

/// Compute the direct + transitive descendants of changed nodes and return a
/// deterministic revalidation plan. Changed roots are reported separately:
/// the caller must first verify/accept the new root revision, then revalidate
/// descendants in the returned order.
pub fn build_revalidation_plan(
    graph: &EvidenceDependencyGraph,
    changed_node_ids: &[String],
) -> Result<RevalidationPlan, Vec<EvidenceDependencyError>> {
    let order = topological_order(graph)?;
    let node_ids: BTreeSet<String> =
        graph.nodes.iter().map(|node| node.node_id.clone()).collect();

    let mut errors = Vec::new();
    let mut changed = BTreeSet::new();
    for changed_id in changed_node_ids {
        if !node_ids.contains(changed_id) {
            errors.push(EvidenceDependencyError::UnknownChangedNode(
                changed_id.clone(),
            ));
        } else {
            changed.insert(changed_id.clone());
        }
    }
    if changed.is_empty() && errors.is_empty() {
        errors.push(EvidenceDependencyError::EmptyIdentifier(
            "changed_node_ids",
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut children: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for node in &graph.nodes {
        for dependency_id in &node.dependency_ids {
            children
                .entry(dependency_id.as_str())
                .or_default()
                .insert(node.node_id.as_str());
        }
    }

    let mut invalidated = BTreeSet::new();
    let mut queue: VecDeque<String> = changed.iter().cloned().collect();
    while let Some(current) = queue.pop_front() {
        if let Some(descendants) = children.get(current.as_str()) {
            for descendant in descendants {
                let descendant = (*descendant).to_string();
                if !changed.contains(&descendant) && invalidated.insert(descendant.clone()) {
                    queue.push_back(descendant);
                }
            }
        }
    }

    let revalidation_order: Vec<String> = order
        .into_iter()
        .filter(|node_id| invalidated.contains(node_id))
        .collect();
    let unaffected_node_ids: Vec<String> = node_ids
        .difference(&changed)
        .filter(|node_id| !invalidated.contains(*node_id))
        .cloned()
        .collect();

    Ok(RevalidationPlan {
        graph_id: graph.graph_id.clone(),
        snapshot_id: graph.snapshot_id.clone(),
        changed_node_ids: changed.into_iter().collect(),
        invalidated_node_ids: invalidated.into_iter().collect(),
        revalidation_order,
        unaffected_node_ids,
        status: "revalidation_plan_structural_only_no_receipts_granted".to_string(),
        claim_boundary: "This plan identifies descendants of changed declared inputs. It does not verify a replacement source revision, re-run a sensor/transform/model, approve a candidate claim, or authorize owner-system edits.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/evidence-dependency-graph-synthetic-v0.1.json"
    );

    fn graph() -> EvidenceDependencyGraph {
        serde_json::from_str(FIXTURE).expect("evidence dependency fixture parses")
    }

    #[test]
    fn synthetic_dependency_graph_is_a_valid_dag() {
        assert!(validate_evidence_dependency_graph(&graph()).is_ok());
    }

    #[test]
    fn source_revision_change_invalidates_all_transitive_descendants() {
        let graph = graph();
        let plan = build_revalidation_plan(&graph, &["source:gpr-survey-a".to_string()]).unwrap();
        assert_eq!(plan.invalidated_node_ids.len(), 7);
        assert_eq!(plan.revalidation_order.first().unwrap(), "transform:route-a");
        assert_eq!(plan.revalidation_order.last().unwrap(), "export:partner-view-a");
        assert!(plan.unaffected_node_ids.contains(&"calibration:gpr-400mhz-a".to_string()));
        assert!(plan.unaffected_node_ids.contains(&"source:unrelated".to_string()));
    }

    #[test]
    fn calibration_change_invalidates_observations_but_not_spatial_transform() {
        let plan = build_revalidation_plan(
            &graph(),
            &["calibration:gpr-400mhz-a".to_string()],
        )
        .unwrap();
        assert!(!plan.invalidated_node_ids.contains(&"transform:route-a".to_string()));
        assert!(plan.invalidated_node_ids.contains(&"observation:gpr-a".to_string()));
        assert!(plan.invalidated_node_ids.contains(&"decision:survey-plan-a".to_string()));
    }

    #[test]
    fn multiple_changed_roots_have_deduplicated_sorted_closure() {
        let plan = build_revalidation_plan(
            &graph(),
            &[
                "source:gpr-survey-a".to_string(),
                "calibration:gpr-400mhz-a".to_string(),
            ],
        )
        .unwrap();
        let unique: BTreeSet<&str> =
            plan.invalidated_node_ids.iter().map(String::as_str).collect();
        assert_eq!(unique.len(), plan.invalidated_node_ids.len());
        assert!(plan.invalidated_node_ids.contains(&"decision:survey-plan-a".to_string()));
    }

    #[test]
    fn empty_dependency_graph_is_rejected() {
        let mut graph = graph();
        graph.nodes.clear();
        assert!(validate_evidence_dependency_graph(&graph)
            .unwrap_err()
            .contains(&EvidenceDependencyError::EmptyGraph));
    }

    #[test]
    fn cycles_are_rejected() {
        let mut graph = graph();
        graph.nodes[0].dependency_ids.push("export:partner-view-a".to_string());
        assert!(matches!(
            validate_evidence_dependency_graph(&graph),
            Err(errors) if errors.iter().any(|error| matches!(error, EvidenceDependencyError::DependencyCycle(_)))
        ));
    }

    #[test]
    fn self_dependencies_are_rejected() {
        let mut graph = graph();
        let node_id = graph.nodes[0].node_id.clone();
        graph.nodes[0].dependency_ids.push(node_id.clone());
        assert!(validate_evidence_dependency_graph(&graph)
            .unwrap_err()
            .contains(&EvidenceDependencyError::SelfDependency(node_id)));
    }

    #[test]
    fn unknown_dependency_and_unknown_change_are_rejected() {
        let mut invalid_graph = graph();
        let first_node_id = invalid_graph.nodes[0].node_id.clone();
        invalid_graph.nodes[0].dependency_ids.push("missing:source".to_string());
        assert!(validate_evidence_dependency_graph(&invalid_graph)
            .unwrap_err()
            .contains(&EvidenceDependencyError::UnknownDependency(
                first_node_id,
                "missing:source".to_string()
            )));

        assert_eq!(
            build_revalidation_plan(&graph(), &["missing:node".to_string()]),
            Err(vec![EvidenceDependencyError::UnknownChangedNode(
                "missing:node".to_string()
            )])
        );
    }

    #[test]
    fn revalidation_order_is_deterministic_under_node_order_permutation() {
        let mut first = graph();
        let mut second = graph();
        second.nodes.reverse();
        let changed = vec!["source:gpr-survey-a".to_string()];
        assert_eq!(
            build_revalidation_plan(&first, &changed).unwrap(),
            build_revalidation_plan(&second, &changed).unwrap()
        );
    }
}
