// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mutually exclusive, joint hypotheses for underground utility-network
//! reconstruction.
//!
//! Unlike event-to-asset marginals, each hypothesis describes one internally
//! coherent candidate world: event assignments, assets, geometry claims and
//! network relations are evaluated together. Probability not assigned to an
//! enumerated world is retained as explicit residual/unknown mass.
//!
//! This module validates structural and probability-mass invariants only. It
//! does not authenticate evidence, calibrate posterior probabilities, solve
//! geometry, establish physical truth, or authorize field work.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const PROBABILITY_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisEvidenceStatus {
    SyntheticUncalibrated,
    ResearchUncalibrated,
    CalibrationReceiptReferenced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisObservedEvent {
    pub event_id: String,
    pub source_evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisEventAssignment {
    pub event_id: String,
    /// None is an explicit unassigned / clutter / unresolved disposition.
    pub candidate_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisAsset {
    /// Must remain a non-authoritative candidate namespace.
    pub candidate_asset_id: String,
    pub event_ids: Vec<String>,
    /// A geometry claim is optional: an asset hypothesis may exist before
    /// there is enough evidence to establish a spatial geometry claim.
    pub geometry_claim_id: Option<String>,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisNode {
    pub candidate_node_id: String,
    pub candidate_asset_id: String,
    pub geometry_claim_id: String,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisConnectionRelation {
    Continuation,
    Junction,
    CrossingUnresolved,
    NearIntersectionUnresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisConnection {
    pub connection_id: String,
    pub from_node_candidate_id: String,
    pub to_node_candidate_id: String,
    pub relation: HypothesisConnectionRelation,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkHypothesis {
    pub hypothesis_id: String,
    /// Probability of this complete, mutually exclusive network world.
    pub probability: f64,
    /// Every observed event must appear exactly once in this assignment list.
    pub event_assignments: Vec<HypothesisEventAssignment>,
    pub candidate_assets: Vec<HypothesisAsset>,
    pub candidate_nodes: Vec<HypothesisNode>,
    pub candidate_connections: Vec<HypothesisConnection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkHypothesisSet {
    pub set_id: String,
    pub status: HypothesisEvidenceStatus,
    pub geography: String,
    pub scope_id: String,
    /// Commit or immutable source-snapshot identifier for the evidence pack.
    pub evidence_snapshot_id: String,
    pub calibration_receipt_id: Option<String>,
    pub observed_events: Vec<HypothesisObservedEvent>,
    pub admitted_evidence_ids: Vec<String>,
    /// The listed worlds are mutually exclusive and share one evidence
    /// snapshot, scope, and prediction time.
    pub hypotheses: Vec<NetworkHypothesis>,
    /// Probability reserved for worlds not explicitly enumerated. This is
    /// not an absence probability and must not be redistributed silently.
    pub residual_unknown_probability: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkHypothesisError {
    EmptyIdentifier(&'static str),
    InvalidGeography,
    CalibrationReceiptMissing,
    DuplicateId(&'static str, String),
    InvalidProbability(&'static str, String),
    ProbabilityMassNotConserved,
    EmptyHypothesisSet,
    MissingEventAssignment(String),
    UnknownEvent(String),
    UnknownAsset(String),
    AssetNamespaceInvalid(String),
    AssetWithoutEvents(String),
    DuplicateAssetEvent(String, String),
    AssignmentEvidenceNotBound(String, String),
    AssetAssignmentMismatch(String),
    EvidenceNotAdmitted(String),
    UnknownNode(String),
    NodeAssetUnknown(String),
    SelfConnection(String),
    ConnectionWithoutEvidence(String),
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn evidence_is_admitted(ids: &[String], admitted: &BTreeSet<String>) -> bool {
    !ids.is_empty() && ids.iter().all(|id| admitted.contains(id))
}

/// Validate set identity, explicit residual mass, complete event assignments,
/// source-evidence binding and internally coherent network topology.
///
/// A successful result is structural validation, not a calibrated posterior
/// or physical qualification.
pub fn validate_network_hypothesis_set(set: &NetworkHypothesisSet) -> Vec<NetworkHypothesisError> {
    let mut errors = Vec::new();

    for (field, value) in [
        ("set_id", set.set_id.as_str()),
        ("scope_id", set.scope_id.as_str()),
        ("evidence_snapshot_id", set.evidence_snapshot_id.as_str()),
    ] {
        if !non_empty(value) {
            errors.push(NetworkHypothesisError::EmptyIdentifier(field));
        }
    }

    if set.geography != "abstract_non_geographic" {
        errors.push(NetworkHypothesisError::InvalidGeography);
    }

    if set.status == HypothesisEvidenceStatus::CalibrationReceiptReferenced
        && set.calibration_receipt_id.as_deref().map(non_empty) != Some(true)
    {
        errors.push(NetworkHypothesisError::CalibrationReceiptMissing);
    }

    if !valid_probability(set.residual_unknown_probability) {
        errors.push(NetworkHypothesisError::InvalidProbability(
            "residual_unknown_probability",
            set.residual_unknown_probability.to_string(),
        ));
    }

    let mut admitted = BTreeSet::new();
    for id in &set.admitted_evidence_ids {
        if !non_empty(id) {
            errors.push(NetworkHypothesisError::EmptyIdentifier(
                "admitted_evidence_id",
            ));
        } else if !admitted.insert(id.clone()) {
            errors.push(NetworkHypothesisError::DuplicateId(
                "admitted_evidence_id",
                id.clone(),
            ));
        }
    }

    let mut event_sources: BTreeMap<String, String> = BTreeMap::new();
    for event in &set.observed_events {
        if !non_empty(&event.event_id) {
            errors.push(NetworkHypothesisError::EmptyIdentifier("event_id"));
        }
        if !non_empty(&event.source_evidence_id) {
            errors.push(NetworkHypothesisError::EmptyIdentifier(
                "event_source_evidence_id",
            ));
        }
        if event_sources
            .insert(event.event_id.clone(), event.source_evidence_id.clone())
            .is_some()
        {
            errors.push(NetworkHypothesisError::DuplicateId(
                "event_id",
                event.event_id.clone(),
            ));
        }
        if !admitted.contains(&event.source_evidence_id) {
            errors.push(NetworkHypothesisError::EvidenceNotAdmitted(
                event.source_evidence_id.clone(),
            ));
        }
    }

    if set.hypotheses.is_empty() {
        errors.push(NetworkHypothesisError::EmptyHypothesisSet);
    }

    let mut hypothesis_ids = BTreeSet::new();
    let mut probability_mass = set.residual_unknown_probability;

    for hypothesis in &set.hypotheses {
        if !non_empty(&hypothesis.hypothesis_id) {
            errors.push(NetworkHypothesisError::EmptyIdentifier("hypothesis_id"));
        } else if !hypothesis_ids.insert(hypothesis.hypothesis_id.clone()) {
            errors.push(NetworkHypothesisError::DuplicateId(
                "hypothesis_id",
                hypothesis.hypothesis_id.clone(),
            ));
        }

        if !valid_probability(hypothesis.probability) {
            errors.push(NetworkHypothesisError::InvalidProbability(
                "hypothesis_probability",
                hypothesis.hypothesis_id.clone(),
            ));
        } else {
            probability_mass += hypothesis.probability;
        }

        let mut asset_ids = BTreeSet::new();
        let mut asset_event_ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut asset_evidence: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for asset in &hypothesis.candidate_assets {
            if !non_empty(&asset.candidate_asset_id) {
                errors.push(NetworkHypothesisError::EmptyIdentifier(
                    "candidate_asset_id",
                ));
            }
            if !asset.candidate_asset_id.starts_with("candidate:")
                || asset.candidate_asset_id.len() <= "candidate:".len()
            {
                errors.push(NetworkHypothesisError::AssetNamespaceInvalid(
                    asset.candidate_asset_id.clone(),
                ));
            }
            if !asset_ids.insert(asset.candidate_asset_id.clone()) {
                errors.push(NetworkHypothesisError::DuplicateId(
                    "candidate_asset_id",
                    asset.candidate_asset_id.clone(),
                ));
            }

            if asset.event_ids.is_empty() {
                errors.push(NetworkHypothesisError::AssetWithoutEvents(
                    asset.candidate_asset_id.clone(),
                ));
            }

            let mut ids = BTreeSet::new();
            for event_id in &asset.event_ids {
                if !ids.insert(event_id.clone()) {
                    errors.push(NetworkHypothesisError::DuplicateAssetEvent(
                        asset.candidate_asset_id.clone(),
                        event_id.clone(),
                    ));
                }
            }
            asset_event_ids.insert(asset.candidate_asset_id.clone(), ids);

            let mut evidence = BTreeSet::new();
            for evidence_id in &asset.supporting_evidence_ids {
                if !admitted.contains(evidence_id) {
                    errors.push(NetworkHypothesisError::EvidenceNotAdmitted(
                        evidence_id.clone(),
                    ));
                }
                evidence.insert(evidence_id.clone());
            }
            if evidence.is_empty() {
                errors.push(NetworkHypothesisError::AssetWithoutEvents(
                    asset.candidate_asset_id.clone(),
                ));
            }
            asset_evidence.insert(asset.candidate_asset_id.clone(), evidence);

            if let Some(geometry_id) = asset.geometry_claim_id.as_deref() {
                if !non_empty(geometry_id) {
                    errors.push(NetworkHypothesisError::EmptyIdentifier(
                        "asset_geometry_claim_id",
                    ));
                }
            }
        }

        let mut assigned_events = BTreeMap::new();
        for assignment in &hypothesis.event_assignments {
            if !event_sources.contains_key(&assignment.event_id) {
                errors.push(NetworkHypothesisError::UnknownEvent(
                    assignment.event_id.clone(),
                ));
            }
            if assigned_events
                .insert(
                    assignment.event_id.clone(),
                    assignment.candidate_asset_id.clone(),
                )
                .is_some()
            {
                errors.push(NetworkHypothesisError::DuplicateId(
                    "event_assignment",
                    assignment.event_id.clone(),
                ));
            }

            if let Some(asset_id) = assignment.candidate_asset_id.as_deref() {
                if !asset_ids.contains(asset_id) {
                    errors.push(NetworkHypothesisError::UnknownAsset(asset_id.to_string()));
                } else if let (Some(source_id), Some(evidence_ids)) = (
                    event_sources.get(&assignment.event_id),
                    asset_evidence.get(asset_id),
                ) {
                    if !evidence_ids.contains(source_id) {
                        errors.push(NetworkHypothesisError::AssignmentEvidenceNotBound(
                            assignment.event_id.clone(),
                            asset_id.to_string(),
                        ));
                    }
                }
            }
        }

        for event_id in event_sources.keys() {
            if !assigned_events.contains_key(event_id) {
                errors.push(NetworkHypothesisError::MissingEventAssignment(
                    event_id.clone(),
                ));
            }
        }

        for asset_id in &asset_ids {
            let assigned: BTreeSet<String> = assigned_events
                .iter()
                .filter_map(|(event_id, assigned_asset)| {
                    (assigned_asset.as_deref() == Some(asset_id.as_str()))
                        .then_some(event_id.clone())
                })
                .collect();
            let declared = asset_event_ids.get(asset_id).cloned().unwrap_or_default();
            if assigned != declared {
                errors.push(NetworkHypothesisError::AssetAssignmentMismatch(
                    asset_id.clone(),
                ));
            }
        }

        let mut node_ids = BTreeSet::new();
        for node in &hypothesis.candidate_nodes {
            if !non_empty(&node.candidate_node_id) {
                errors.push(NetworkHypothesisError::EmptyIdentifier("candidate_node_id"));
            }
            if !node_ids.insert(node.candidate_node_id.clone()) {
                errors.push(NetworkHypothesisError::DuplicateId(
                    "candidate_node_id",
                    node.candidate_node_id.clone(),
                ));
            }
            if !asset_ids.contains(&node.candidate_asset_id) {
                errors.push(NetworkHypothesisError::NodeAssetUnknown(
                    node.candidate_asset_id.clone(),
                ));
            }
            if !non_empty(&node.geometry_claim_id) {
                errors.push(NetworkHypothesisError::EmptyIdentifier(
                    "node_geometry_claim_id",
                ));
            }
            if !evidence_is_admitted(&node.supporting_evidence_ids, &admitted) {
                errors.push(NetworkHypothesisError::EvidenceNotAdmitted(
                    node.candidate_node_id.clone(),
                ));
            }
        }

        let mut connection_ids = BTreeSet::new();
        for connection in &hypothesis.candidate_connections {
            if !non_empty(&connection.connection_id) {
                errors.push(NetworkHypothesisError::EmptyIdentifier("connection_id"));
            }
            if !connection_ids.insert(connection.connection_id.clone()) {
                errors.push(NetworkHypothesisError::DuplicateId(
                    "connection_id",
                    connection.connection_id.clone(),
                ));
            }
            if !node_ids.contains(&connection.from_node_candidate_id) {
                errors.push(NetworkHypothesisError::UnknownNode(
                    connection.from_node_candidate_id.clone(),
                ));
            }
            if !node_ids.contains(&connection.to_node_candidate_id) {
                errors.push(NetworkHypothesisError::UnknownNode(
                    connection.to_node_candidate_id.clone(),
                ));
            }
            if connection.from_node_candidate_id == connection.to_node_candidate_id {
                errors.push(NetworkHypothesisError::SelfConnection(
                    connection.connection_id.clone(),
                ));
            }
            if !evidence_is_admitted(&connection.supporting_evidence_ids, &admitted) {
                errors.push(NetworkHypothesisError::ConnectionWithoutEvidence(
                    connection.connection_id.clone(),
                ));
            }
        }
    }

    if (probability_mass - 1.0).abs() > PROBABILITY_TOLERANCE {
        errors.push(NetworkHypothesisError::ProbabilityMassNotConserved);
    }

    errors
}

/// Adjudicated truth for one evaluation case. The residual class is selected
/// only when the reference world is not represented by any listed hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkHypothesisTruth<'a> {
    Listed(&'a str),
    ResidualUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkHypothesisScoreError {
    InvalidHypothesisSet(Vec<NetworkHypothesisError>),
    UnknownTruthHypothesis(String),
    ZeroProbabilityTruth(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkHypothesisScore {
    pub truth_category: String,
    pub category_count: usize,
    /// Natural-log loss (-ln assigned probability); lower is better.
    pub negative_log_probability: f64,
    /// Multiclass Brier score over all listed worlds plus residual unknown.
    /// No category is dropped or renormalized away.
    pub multiclass_brier_score: f64,
}

/// Score one adjudicated world against a submitted finite joint distribution.
///
/// The caller must assign the reference world either to an exact listed
/// hypothesis or to the residual-unknown category. A zero-probability truth
/// returns an explicit error (infinite log loss) rather than being silently
/// floored. This scorer evaluates submitted probabilities; it does not
/// establish their calibration or the quality of the reference adjudication.
pub fn score_network_hypothesis_set(
    set: &NetworkHypothesisSet,
    truth: NetworkHypothesisTruth<'_>,
) -> Result<NetworkHypothesisScore, NetworkHypothesisScoreError> {
    let validation_errors = validate_network_hypothesis_set(set);
    if !validation_errors.is_empty() {
        return Err(NetworkHypothesisScoreError::InvalidHypothesisSet(
            validation_errors,
        ));
    }

    let truth_category = match truth {
        NetworkHypothesisTruth::Listed(hypothesis_id) => {
            if !set
                .hypotheses
                .iter()
                .any(|hypothesis| hypothesis.hypothesis_id == hypothesis_id)
            {
                return Err(NetworkHypothesisScoreError::UnknownTruthHypothesis(
                    hypothesis_id.to_string(),
                ));
            }
            format!("hypothesis:{hypothesis_id}")
        }
        NetworkHypothesisTruth::ResidualUnknown => "residual_unknown".to_string(),
    };

    let truth_probability = match truth {
        NetworkHypothesisTruth::Listed(hypothesis_id) => set
            .hypotheses
            .iter()
            .find(|hypothesis| hypothesis.hypothesis_id == hypothesis_id)
            .map(|hypothesis| hypothesis.probability)
            .unwrap_or(0.0),
        NetworkHypothesisTruth::ResidualUnknown => set.residual_unknown_probability,
    };

    if truth_probability == 0.0 {
        return Err(NetworkHypothesisScoreError::ZeroProbabilityTruth(
            truth_category,
        ));
    }

    let mut brier = 0.0;
    for hypothesis in &set.hypotheses {
        let outcome = match truth {
            NetworkHypothesisTruth::Listed(hypothesis_id) => {
                hypothesis.hypothesis_id == hypothesis_id
            }
            NetworkHypothesisTruth::ResidualUnknown => false,
        };
        let observed = if outcome { 1.0 } else { 0.0 };
        let error = hypothesis.probability - observed;
        brier += error * error;
    }

    let residual_observed = if matches!(truth, NetworkHypothesisTruth::ResidualUnknown) {
        1.0
    } else {
        0.0
    };
    let residual_error = set.residual_unknown_probability - residual_observed;
    brier += residual_error * residual_error;

    Ok(NetworkHypothesisScore {
        truth_category,
        category_count: set.hypotheses.len() + 1,
        negative_log_probability: -truth_probability.ln(),
        multiclass_brier_score: brier,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/network-hypotheses-synthetic-v0.1.json"
    );

    fn fixture() -> NetworkHypothesisSet {
        serde_json::from_str(FIXTURE).expect("synthetic joint hypothesis fixture parses")
    }

    #[test]
    fn synthetic_joint_hypothesis_fixture_is_structurally_valid() {
        let set = fixture();
        let errors = validate_network_hypothesis_set(&set);
        assert!(
            errors.is_empty(),
            "unexpected validation errors: {errors:?}"
        );
    }

    #[test]
    fn probability_mass_must_include_residual_unknown_mass() {
        let mut set = fixture();
        set.residual_unknown_probability = 0.20;
        assert!(
            validate_network_hypothesis_set(&set)
                .contains(&NetworkHypothesisError::ProbabilityMassNotConserved)
        );
    }

    #[test]
    fn every_observed_event_requires_exactly_one_world_assignment() {
        let mut set = fixture();
        set.hypotheses[0].event_assignments.pop();
        assert!(
            validate_network_hypothesis_set(&set)
                .iter()
                .any(|error| matches!(error, NetworkHypothesisError::MissingEventAssignment(_)))
        );
    }

    #[test]
    fn an_assignment_must_be_supported_by_its_event_source() {
        let mut set = fixture();
        set.hypotheses[0].candidate_assets[0]
            .supporting_evidence_ids
            .retain(|id| id != "gpr-evidence-1");
        assert!(validate_network_hypothesis_set(&set).contains(
            &NetworkHypothesisError::AssignmentEvidenceNotBound(
                "event-1".to_string(),
                "candidate:utility-a".to_string(),
            )
        ));
    }

    #[test]
    fn asset_event_membership_must_match_the_world_assignments() {
        let mut set = fixture();
        set.hypotheses[0].candidate_assets[0]
            .event_ids
            .retain(|id| id != "event-2");
        assert!(validate_network_hypothesis_set(&set).contains(
            &NetworkHypothesisError::AssetAssignmentMismatch("candidate:utility-a".to_string(),)
        ));
    }

    #[test]
    fn topology_must_reference_existing_nodes() {
        let mut set = fixture();
        set.hypotheses[0].candidate_connections[0].to_node_candidate_id =
            "candidate-node:missing".to_string();
        assert!(validate_network_hypothesis_set(&set).contains(
            &NetworkHypothesisError::UnknownNode("candidate-node:missing".to_string(),)
        ));
    }

    #[test]
    fn calibration_claim_requires_a_receipt_reference() {
        let mut set = fixture();
        set.status = HypothesisEvidenceStatus::CalibrationReceiptReferenced;
        set.calibration_receipt_id = None;
        assert!(
            validate_network_hypothesis_set(&set)
                .contains(&NetworkHypothesisError::CalibrationReceiptMissing)
        );
    }

    #[test]
    fn joint_world_scorer_scores_a_listed_world_without_renormalizing() {
        let set = fixture();
        let score =
            score_network_hypothesis_set(&set, NetworkHypothesisTruth::Listed("world-one-utility"))
                .unwrap();
        assert_eq!(score.truth_category, "hypothesis:world-one-utility");
        assert_eq!(score.category_count, 4);
        assert!((score.negative_log_probability - -0.62_f64.ln()).abs() < 1e-12);
        assert!((score.multiclass_brier_score - 0.2098).abs() < 1e-12);
    }

    #[test]
    fn joint_world_scorer_scores_the_residual_unknown_category() {
        let set = fixture();
        let score =
            score_network_hypothesis_set(&set, NetworkHypothesisTruth::ResidualUnknown).unwrap();
        assert_eq!(score.truth_category, "residual_unknown");
        assert!((score.negative_log_probability - -0.05_f64.ln()).abs() < 1e-12);
        assert!((score.multiclass_brier_score - 1.3498).abs() < 1e-12);
    }

    #[test]
    fn joint_world_scorer_rejects_zero_probability_truth() {
        let mut set = fixture();
        set.hypotheses[0].probability = 0.0;
        set.hypotheses[1].probability = 0.85;
        assert_eq!(
            score_network_hypothesis_set(&set, NetworkHypothesisTruth::Listed("world-one-utility"),),
            Err(NetworkHypothesisScoreError::ZeroProbabilityTruth(
                "hypothesis:world-one-utility".to_string(),
            ))
        );
    }

    #[test]
    fn joint_world_scorer_rejects_unknown_truth_labels() {
        let set = fixture();
        assert_eq!(
            score_network_hypothesis_set(
                &set,
                NetworkHypothesisTruth::Listed("world-not-in-submission"),
            ),
            Err(NetworkHypothesisScoreError::UnknownTruthHypothesis(
                "world-not-in-submission".to_string(),
            ))
        );
    }
}
