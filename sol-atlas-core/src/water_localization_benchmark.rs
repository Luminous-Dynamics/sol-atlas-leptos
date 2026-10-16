// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Leakage-resistant, leave-one-withdrawal-magnitude-out localization baseline.
//!
//! This evaluates synthetic pressure signatures only. It is not a leak detector,
//! calibrated hydraulic model, sensor-placement optimizer, or field validation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

const SCHEMA_VERSION: &str = "water-scenario-corpus-run/v0.1";
const EVALUATION_VERSION: &str = "water-localization-evaluation/v0.1";
const EPSILON: f64 = 1.0e-12;

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RunManifestInput {
    schema_version: String,
    base_input_sha256: String,
    unit_system: String,
    flow_unit: String,
    pressure_unit: String,
    observation_node_ids: Vec<String>,
    scenario_count: usize,
    completed_count: usize,
    failed_count: usize,
    run_state: String,
    scenarios: Vec<ScenarioInput>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ScenarioInput {
    scenario_id: String,
    ground_truth_class: String,
    target_junction_id: Option<String>,
    additional_demand: f64,
    run_state: String,
    pressure_readings: Vec<PressureReadingInput>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PressureReadingInput {
    node_id: String,
    state: String,
    pressure_value: Option<f64>,
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct LabeledFeature {
    scenario_id: String,
    target: String,
    magnitude: f64,
    features: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalizationEvaluationEnvelope {
    pub schema_version: String,
    pub source_manifest_sha256: String,
    pub source_topology_sha256: String,
    pub evaluation: LocalizationEvaluation,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalizationEvaluation {
    pub schema_version: String,
    pub run_state: String,
    pub unit_system: String,
    pub flow_unit: String,
    pub pressure_unit: String,
    pub topology_node_count: usize,
    pub topology_link_count: usize,
    pub distance_semantics: String,
    pub baseline_scenario_id: String,
    pub negative_control_count: usize,
    pub fault_scenario_count: usize,
    pub observation_node_ids: Vec<String>,
    pub candidate_target_count: usize,
    pub held_out_magnitudes: Vec<f64>,
    pub top_k: usize,
    pub models: Vec<LocalizationModelEvaluation>,
    pub claim_ceiling: String,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalizationModelEvaluation {
    pub model_id: String,
    pub distance_metric: String,
    pub test_count: usize,
    pub top1_accuracy: f64,
    pub top_k_accuracy: f64,
    pub mean_reciprocal_rank: f64,
    pub mean_topological_error_hops: Option<f64>,
    pub p95_topological_error_hops: Option<usize>,
    pub worst_topological_error_hops: Option<usize>,
    pub disconnected_prediction_count: usize,
    pub predictions: Vec<LocalizationPrediction>,
    pub folds: Vec<LocalizationFoldEvaluation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalizationFoldEvaluation {
    pub held_out_magnitude: f64,
    pub test_count: usize,
    pub top1_accuracy: f64,
    pub top_k_accuracy: f64,
    pub mean_reciprocal_rank: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalizationPrediction {
    pub fold_magnitude: f64,
    pub scenario_id: String,
    pub actual_target: String,
    pub predicted_top1: String,
    pub actual_rank: usize,
    pub top_k_hit: bool,
    pub topological_error_hops: Option<usize>,
    pub ranked_candidates: Vec<RankedCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RankedCandidate {
    pub target_junction_id: String,
    pub distance: f64,
    pub rank: usize,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn units_match(unit_system: &str, flow_unit: &str, pressure_unit: &str) -> bool {
    matches!(
        (unit_system, flow_unit, pressure_unit),
        ("LPS", "L/s", "m") | ("GPM", "GPM", "psi")
    )
}


#[derive(Debug, Clone, PartialEq)]
struct NetworkTopology {
    junction_ids: BTreeSet<String>,
    adjacency: BTreeMap<String, Vec<String>>,
    link_count: usize,
}

impl NetworkTopology {
    fn shortest_path_hops(&self, source: &str, target: &str) -> Option<usize> {
        if source == target {
            return self.adjacency.contains_key(source).then_some(0);
        }
        if !self.adjacency.contains_key(source) || !self.adjacency.contains_key(target) {
            return None;
        }
        let mut queue = VecDeque::from([source.to_owned()]);
        let mut distances = BTreeMap::from([(source.to_owned(), 0usize)]);
        while let Some(current) = queue.pop_front() {
            let next_distance = distances[&current] + 1;
            if let Some(neighbours) = self.adjacency.get(&current) {
                for neighbour in neighbours {
                    if distances.contains_key(neighbour) {
                        continue;
                    }
                    if neighbour.as_str() == target {
                        return Some(next_distance);
                    }
                    distances.insert(neighbour.clone(), next_distance);
                    queue.push_back(neighbour.clone());
                }
            }
        }
        None
    }
}

fn valid_topology_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 31
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn inp_section_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    (trimmed.starts_with('[') && trimmed.ends_with(']'))
        .then(|| trimmed[1..trimmed.len() - 1].trim().to_ascii_uppercase())
}

/// Parse an undirected topological graph from the exact, hash-bound EPANET INP.
/// This is a link-hop metric, not hydraulic distance, pipe length, or flow direction.
fn parse_network_topology(
    input: &[u8],
    expected_unit_system: &str,
) -> Result<NetworkTopology, String> {
    let text = std::str::from_utf8(input)
        .map_err(|_| "topology source INP is not UTF-8".to_owned())?;
    if !text.is_ascii() {
        return Err("topology source INP must be ASCII".to_owned());
    }

    let mut section = String::new();
    let mut junction_ids = BTreeSet::new();
    let mut declared_nodes = BTreeSet::new();
    let mut link_records: Vec<(String, String, String)> = Vec::new();
    let mut link_ids = BTreeSet::new();
    let mut unit_declarations = Vec::new();
    let mut saw_junctions = false;
    let mut saw_pipes = false;
    let mut saw_options = false;

    for line in text.lines() {
        if let Some(name) = inp_section_name(line) {
            section = name;
            saw_junctions |= section == "JUNCTIONS";
            saw_pipes |= section == "PIPES";
            saw_options |= section == "OPTIONS";
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if section == "OPTIONS" && fields[0].eq_ignore_ascii_case("UNITS") {
            if fields.len() != 2 {
                return Err(format!("malformed OPTIONS UNITS row: {line:?}"));
            }
            unit_declarations.push(fields[1].to_ascii_uppercase());
            continue;
        }
        match section.as_str() {
            "JUNCTIONS" | "RESERVOIRS" | "TANKS" => {
                let id = fields[0];
                if !valid_topology_identifier(id) {
                    return Err(format!("invalid topology node identifier {id:?}"));
                }
                if !declared_nodes.insert(id.to_owned()) {
                    return Err(format!("duplicate topology node identifier {id:?}"));
                }
                if section == "JUNCTIONS" {
                    junction_ids.insert(id.to_owned());
                }
            }
            "PIPES" | "PUMPS" | "VALVES" => {
                if fields.len() < 3 {
                    return Err(format!("malformed {section} topology row: {line:?}"));
                }
                let link_id = fields[0];
                let from = fields[1];
                let to = fields[2];
                if !valid_topology_identifier(link_id)
                    || !valid_topology_identifier(from)
                    || !valid_topology_identifier(to)
                {
                    return Err(format!("invalid link/node identifier in {section} row: {line:?}"));
                }
                if !link_ids.insert(link_id.to_owned()) {
                    return Err(format!("duplicate topology link identifier {link_id:?}"));
                }
                if from == to {
                    return Err(format!("self-loop topology link {link_id:?} is unsupported"));
                }
                link_records.push((link_id.to_owned(), from.to_owned(), to.to_owned()));
            }
            _ => {}
        }
    }

    if !saw_junctions || junction_ids.is_empty() {
        return Err("topology source must declare a non-empty [JUNCTIONS] section".to_owned());
    }
    if !saw_pipes {
        return Err("topology source must declare a [PIPES] section".to_owned());
    }
    if !saw_options || unit_declarations.len() != 1 {
        return Err("topology source must declare exactly one OPTIONS UNITS value".to_owned());
    }
    if unit_declarations[0] != expected_unit_system {
        return Err(format!(
            "topology source unit system {} differs from run manifest {}",
            unit_declarations[0], expected_unit_system
        ));
    }
    if link_records.is_empty() {
        return Err("topology source declares no pipe, pump, or valve links".to_owned());
    }
    for (link_id, from, to) in &link_records {
        if !declared_nodes.contains(from) || !declared_nodes.contains(to) {
            return Err(format!(
                "topology link {link_id:?} references an undeclared endpoint ({from:?}, {to:?})"
            ));
        }
    }

    let mut adjacency: BTreeMap<String, Vec<String>> = declared_nodes
        .iter()
        .map(|node| (node.clone(), Vec::new()))
        .collect();
    let mut unique_edges = BTreeSet::new();
    for (_, from, to) in &link_records {
        let (left, right) = if from <= to {
            (from.to_owned(), to.to_owned())
        } else {
            (to.to_owned(), from.to_owned())
        };
        if !unique_edges.insert((left.clone(), right.clone())) {
            continue;
        }
        adjacency.get_mut(&left).expect("declared endpoint").push(right.clone());
        adjacency.get_mut(&right).expect("declared endpoint").push(left);
    }
    for neighbours in adjacency.values_mut() {
        neighbours.sort();
        neighbours.dedup();
    }

    Ok(NetworkTopology {
        junction_ids,
        adjacency,
        link_count: link_records.len(),
    })
}

fn pressure_vector(
    scenario: &ScenarioInput,
    expected_sensor_ids: &[String],
) -> Result<Vec<f64>, String> {
    if scenario.run_state != "Completed" {
        return Err(format!(
            "scenario {} is not Completed",
            scenario.scenario_id
        ));
    }
    if scenario.pressure_readings.len() != expected_sensor_ids.len() {
        return Err(format!(
            "scenario {} has {} pressure readings; expected {}",
            scenario.scenario_id,
            scenario.pressure_readings.len(),
            expected_sensor_ids.len()
        ));
    }

    let mut by_id = BTreeMap::new();
    for reading in &scenario.pressure_readings {
        if reading.state != "measured" || reading.reason.is_some() {
            return Err(format!(
                "scenario {} has unavailable pressure for {}",
                scenario.scenario_id, reading.node_id
            ));
        }
        let value = reading.pressure_value.ok_or_else(|| {
            format!(
                "scenario {} has null pressure for {}",
                scenario.scenario_id, reading.node_id
            )
        })?;
        if !value.is_finite() {
            return Err(format!(
                "scenario {} has non-finite pressure for {}",
                scenario.scenario_id, reading.node_id
            ));
        }
        if by_id.insert(reading.node_id.as_str(), value).is_some() {
            return Err(format!(
                "scenario {} has duplicate pressure reading for {}",
                scenario.scenario_id, reading.node_id
            ));
        }
    }

    let actual_ids: BTreeSet<&str> = by_id.keys().copied().collect();
    let expected_ids: BTreeSet<&str> =
        expected_sensor_ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(format!(
            "scenario {} sensor IDs differ from manifest observation_node_ids",
            scenario.scenario_id
        ));
    }

    expected_sensor_ids
        .iter()
        .map(|sensor_id| {
            by_id
                .get(sensor_id.as_str())
                .copied()
                .ok_or_else(|| format!("missing pressure reading for {sensor_id}"))
        })
        .collect()
}

fn l2_norm(vector: &[f64]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn distance(left: &[f64], right: &[f64], metric: &str) -> f64 {
    match metric {
        "euclidean" => left
            .iter()
            .zip(right)
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt(),
        "cosine" => {
            let left_norm = l2_norm(left);
            let right_norm = l2_norm(right);
            if left_norm <= EPSILON || right_norm <= EPSILON {
                // A zero pressure-change signature carries no directional signal.
                return 1.0;
            }
            let dot = left
                .iter()
                .zip(right)
                .map(|(a, b)| a * b)
                .sum::<f64>();
            let similarity = (dot / (left_norm * right_norm)).clamp(-1.0, 1.0);
            1.0 - similarity
        }
        _ => f64::INFINITY,
    }
}

fn rank_candidates(
    features: &[f64],
    training: &[LabeledFeature],
    candidate_targets: &[String],
    metric: &str,
) -> Vec<RankedCandidate> {
    let mut candidates: Vec<RankedCandidate> = candidate_targets
        .iter()
        .map(|target| {
            let nearest_distance = training
                .iter()
                .filter(|example| example.target == *target)
                .map(|example| distance(features, &example.features, metric))
                .min_by(|left, right| left.total_cmp(right))
                .unwrap_or(f64::INFINITY);
            RankedCandidate {
                target_junction_id: target.clone(),
                distance: nearest_distance,
                rank: 0,
            }
        })
        .collect();

    candidates.sort_by(|left, right| {
        left.distance
            .total_cmp(&right.distance)
            .then_with(|| left.target_junction_id.cmp(&right.target_junction_id))
    });
    for (index, candidate) in candidates.iter_mut().enumerate() {
        candidate.rank = index + 1;
    }
    candidates
}

fn validate_and_build_features(
    input: &[u8],
) -> Result<(RunManifestInput, String, Vec<LabeledFeature>, Vec<String>, String), String> {
    let manifest: RunManifestInput = serde_json::from_slice(input)
        .map_err(|error| format!("invalid run manifest JSON: {error}"))?;
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "schema_version must be {SCHEMA_VERSION}, got {:?}",
            manifest.schema_version
        ));
    }
    if manifest.run_state != "Completed"
        || manifest.failed_count != 0
        || manifest.completed_count != manifest.scenario_count
        || manifest.scenarios.len() != manifest.scenario_count
    {
        return Err("run manifest must be fully Completed with no failed scenarios".to_owned());
    }
    if !units_match(
        &manifest.unit_system,
        &manifest.flow_unit,
        &manifest.pressure_unit,
    ) {
        return Err("manifest unit system and pressure/flow units are inconsistent".to_owned());
    }
    if manifest.observation_node_ids.is_empty() {
        return Err("manifest observation_node_ids must not be empty".to_owned());
    }
    let sensor_set: BTreeSet<&str> = manifest
        .observation_node_ids
        .iter()
        .map(String::as_str)
        .collect();
    if sensor_set.len() != manifest.observation_node_ids.len() {
        return Err("manifest observation_node_ids contains duplicates".to_owned());
    }

    let mut scenario_ids = HashSet::new();
    for scenario in &manifest.scenarios {
        if scenario.scenario_id.is_empty() || !scenario_ids.insert(scenario.scenario_id.as_str()) {
            return Err(format!(
                "empty or duplicate scenario_id {:?}",
                scenario.scenario_id
            ));
        }
        if !scenario.additional_demand.is_finite() || scenario.additional_demand < 0.0 {
            return Err(format!(
                "scenario {} has invalid additional_demand",
                scenario.scenario_id
            ));
        }
    }

    let baselines: Vec<&ScenarioInput> = manifest
        .scenarios
        .iter()
        .filter(|scenario| {
            scenario.ground_truth_class == "no_fault"
                && scenario.target_junction_id.is_none()
                && scenario.additional_demand == 0.0
        })
        .collect();
    if baselines.len() != 1 {
        return Err(format!(
            "expected exactly one no_fault baseline, found {}",
            baselines.len()
        ));
    }
    let baseline = baselines[0];
    let baseline_pressures = pressure_vector(baseline, &manifest.observation_node_ids)?;

    let mut labels = BTreeSet::new();
    let mut features = Vec::new();
    for scenario in &manifest.scenarios {
        if scenario.scenario_id == baseline.scenario_id {
            continue;
        }
        if scenario.ground_truth_class != "synthetic_withdrawal" {
            return Err(format!(
                "scenario {} has unsupported ground_truth_class {:?}",
                scenario.scenario_id, scenario.ground_truth_class
            ));
        }
        if scenario.additional_demand <= 0.0 {
            return Err(format!(
                "scenario {} must have positive additional_demand",
                scenario.scenario_id
            ));
        }
        let target = scenario.target_junction_id.as_ref().ok_or_else(|| {
            format!("scenario {} has no target junction label", scenario.scenario_id)
        })?;
        if target.is_empty() {
            return Err(format!(
                "scenario {} has an empty target junction label",
                scenario.scenario_id
            ));
        }
        labels.insert(target.clone());
        let pressures = pressure_vector(scenario, &manifest.observation_node_ids)?;
        let feature_vector: Vec<f64> = pressures
            .iter()
            .zip(&baseline_pressures)
            .map(|(pressure, baseline)| pressure - baseline)
            .collect();
        if feature_vector.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "scenario {} produced a non-finite pressure-change signature",
                scenario.scenario_id
            ));
        }
        features.push(LabeledFeature {
            scenario_id: scenario.scenario_id.clone(),
            target: target.clone(),
            magnitude: scenario.additional_demand,
            features: feature_vector,
        });
    }

    if labels.len() < 2 {
        return Err("localization benchmark requires at least two target junctions".to_owned());
    }
    // Candidate classes are the declared target junctions represented in the
    // frozen scenario corpus. They are distinct from the observation/sensor set:
    // four fixed sensors may be used to discriminate among dozens of targets.
    // Every target must have training representation at every held-out magnitude
    // below, so a class cannot be silently absent from a fold.
    let candidates: Vec<String> = labels.into_iter().collect();
    let mut magnitudes: Vec<f64> = features.iter().map(|feature| feature.magnitude).collect();
    magnitudes.sort_by(|left, right| left.total_cmp(right));
    magnitudes.dedup_by(|left, right| *left == *right);
    if magnitudes.len() < 2 {
        return Err(
            "leave-one-magnitude-out evaluation requires at least two withdrawal magnitudes"
                .to_owned(),
        );
    }

    for magnitude in &magnitudes {
        for target in &candidates {
            if !features
                .iter()
                .any(|feature| feature.magnitude == *magnitude && feature.target == *target)
            {
                return Err(format!(
                    "target {target} has no scenario at held-out magnitude {magnitude}"
                ));
            }
        }
    }

    let baseline_scenario_id = baseline.scenario_id.clone();
    Ok((
        manifest,
        baseline_scenario_id,
        features,
        candidates,
        EVALUATION_VERSION.to_owned(),
    ))
}

fn evaluate_model(
    model_id: &str,
    metric: &str,
    features: &[LabeledFeature],
    candidates: &[String],
    magnitudes: &[f64],
    topology: &NetworkTopology,
    top_k: usize,
) -> LocalizationModelEvaluation {
    let mut predictions = Vec::new();
    let mut folds = Vec::new();

    for held_out_magnitude in magnitudes {
        let training: Vec<LabeledFeature> = features
            .iter()
            .filter(|example| example.magnitude != *held_out_magnitude)
            .cloned()
            .collect();
        let testing: Vec<&LabeledFeature> = features
            .iter()
            .filter(|example| example.magnitude == *held_out_magnitude)
            .collect();
        let mut fold_top1 = 0usize;
        let mut fold_topk = 0usize;
        let mut fold_reciprocal_rank = 0.0;

        for test in testing {
            // Ground-truth target is only read after ranking has been produced.
            // The prediction function receives test features and labelled training
            // examples, never the held-out test target.
            let ranked = rank_candidates(&test.features, &training, candidates, metric);
            let actual_rank = ranked
                .iter()
                .find(|candidate| candidate.target_junction_id == test.target)
                .map(|candidate| candidate.rank)
                .unwrap_or(candidates.len() + 1);
            let predicted_top1 = ranked
                .first()
                .map(|candidate| candidate.target_junction_id.clone())
                .unwrap_or_default();
            let top_k_hit = actual_rank <= top_k;
            if actual_rank == 1 {
                fold_top1 += 1;
            }
            if top_k_hit {
                fold_topk += 1;
            }
            fold_reciprocal_rank += 1.0 / actual_rank as f64;

            let topological_error_hops =
                topology.shortest_path_hops(&test.target, &predicted_top1);
            predictions.push(LocalizationPrediction {
                fold_magnitude: *held_out_magnitude,
                scenario_id: test.scenario_id.clone(),
                actual_target: test.target.clone(),
                predicted_top1,
                actual_rank,
                top_k_hit,
                topological_error_hops,
                ranked_candidates: ranked,
            });
        }

        let denominator = predictions
            .iter()
            .filter(|prediction| prediction.fold_magnitude == *held_out_magnitude)
            .count() as f64;
        folds.push(LocalizationFoldEvaluation {
            held_out_magnitude: *held_out_magnitude,
            test_count: denominator as usize,
            top1_accuracy: fold_top1 as f64 / denominator,
            top_k_accuracy: fold_topk as f64 / denominator,
            mean_reciprocal_rank: fold_reciprocal_rank / denominator,
        });
    }

    predictions.sort_by(|left, right| {
        left.fold_magnitude
            .total_cmp(&right.fold_magnitude)
            .then_with(|| left.scenario_id.cmp(&right.scenario_id))
    });
    let test_count = predictions.len();
    let top1_accuracy = predictions
        .iter()
        .filter(|prediction| prediction.actual_rank == 1)
        .count() as f64
        / test_count as f64;
    let top_k_accuracy = predictions.iter().filter(|prediction| prediction.top_k_hit).count() as f64
        / test_count as f64;
    let mean_reciprocal_rank = predictions
        .iter()
        .map(|prediction| 1.0 / prediction.actual_rank as f64)
        .sum::<f64>()
        / test_count as f64;

    let disconnected_prediction_count = predictions
        .iter()
        .filter(|prediction| prediction.topological_error_hops.is_none())
        .count();
    let mut topological_errors: Vec<usize> = predictions
        .iter()
        .filter_map(|prediction| prediction.topological_error_hops)
        .collect();
    topological_errors.sort_unstable();
    let mean_topological_error_hops = if topological_errors.is_empty() {
        None
    } else {
        Some(
            topological_errors.iter().sum::<usize>() as f64
                / topological_errors.len() as f64,
        )
    };
    let p95_topological_error_hops = if topological_errors.is_empty() {
        None
    } else {
        // Nearest-rank p95: sorted[ceil(0.95 * n) - 1].
        let rank = (topological_errors.len() * 95 + 99) / 100;
        Some(topological_errors[rank.saturating_sub(1)])
    };
    let worst_topological_error_hops = topological_errors.last().copied();

    LocalizationModelEvaluation {
        model_id: model_id.to_owned(),
        distance_metric: metric.to_owned(),
        test_count,
        top1_accuracy,
        top_k_accuracy,
        mean_reciprocal_rank,
        mean_topological_error_hops,
        p95_topological_error_hops,
        worst_topological_error_hops,
        disconnected_prediction_count,
        predictions,
        folds,
    }
}

/// Evaluate only complete, hash-bound solver manifests. The no-fault row defines
/// a reference pressure field; its label is excluded from localization candidates.
pub fn evaluate_localization_manifest(
    input: &[u8],
    source_inp: &[u8],
) -> Result<LocalizationEvaluationEnvelope, String> {
    let (manifest, baseline_id, features, candidates, version) =
        validate_and_build_features(input)?;
    if sha256(source_inp) != manifest.base_input_sha256 {
        return Err("source INP SHA-256 does not match run manifest base_input_sha256".to_owned());
    }
    let topology = parse_network_topology(source_inp, &manifest.unit_system)?;
    for node_id in &manifest.observation_node_ids {
        if !topology.junction_ids.contains(node_id) {
            return Err(format!(
                "declared observation node {node_id:?} is not a junction in the source topology"
            ));
        }
    }
    for feature in &features {
        if !topology.junction_ids.contains(&feature.target) {
            return Err(format!(
                "target junction {:?} is not in the source topology",
                feature.target
            ));
        }
    }
    let mut magnitudes: Vec<f64> = features.iter().map(|feature| feature.magnitude).collect();
    magnitudes.sort_by(|left, right| left.total_cmp(right));
    magnitudes.dedup_by(|left, right| *left == *right);
    let top_k = candidates.len().min(3);

    let models = vec![
        evaluate_model(
            "euclidean-nearest-example",
            "euclidean",
            &features,
            &candidates,
            &magnitudes,
            &topology,
            top_k,
        ),
        evaluate_model(
            "cosine-nearest-example",
            "cosine",
            &features,
            &candidates,
            &magnitudes,
            &topology,
            top_k,
        ),
    ];
    let evaluation = LocalizationEvaluation {
        schema_version: version,
        run_state: "Completed".to_owned(),
        unit_system: manifest.unit_system,
        flow_unit: manifest.flow_unit,
        pressure_unit: manifest.pressure_unit,
        topology_node_count: topology.adjacency.len(),
        topology_link_count: topology.link_count,
        distance_semantics: "unweighted_undirected_link_hops".to_owned(),
        baseline_scenario_id: baseline_id,
        negative_control_count: 1,
        fault_scenario_count: features.len(),
        observation_node_ids: manifest.observation_node_ids,
        candidate_target_count: candidates.len(),
        held_out_magnitudes: magnitudes,
        top_k,
        models,
        claim_ceiling: "synthetic_withdrawal_localization_baseline_only".to_owned(),
        nonclaims: vec![
            "These labels identify synthetic junction-demand perturbations, not physical leaks.".to_owned(),
            "Candidate classes come from labelled target junctions in the frozen scenario corpus; pressure features come only from the observation nodes declared in the run manifest.".to_owned(),
            "Held-out folds separate declared withdrawal magnitudes; they do not hold out topology, demand patterns, model parameters, sensors, or real operating conditions.".to_owned(),
            "No leak-detection sensitivity, false-alarm rate, field accuracy, sensor-placement optimality, or operational readiness is established.".to_owned(),
            "This small nearest-example baseline is a reproducibility check, not a comparison with state-of-the-art ML, GNN, or Transformer methods.".to_owned(),
        ],
    };

    Ok(LocalizationEvaluationEnvelope {
        schema_version: "water-localization-evaluation-envelope/v0.1".to_owned(),
        source_manifest_sha256: sha256(input),
        source_topology_sha256: sha256(source_inp),
        evaluation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pressure(node: &str, value: f64) -> serde_json::Value {
        json!({
            "node_id": node,
            "state": "measured",
            "pressure_value": value,
            "reason": null
        })
    }

    fn scenario(
        id: &str,
        ground_truth_class: &str,
        target: Option<&str>,
        magnitude: f64,
        p1: f64,
        p2: f64,
    ) -> serde_json::Value {
        json!({
            "scenario_id": id,
            "ground_truth_class": ground_truth_class,
            "target_junction_id": target,
            "additional_demand": magnitude,
            "run_state": "Completed",
            "pressure_readings": [pressure("S1", p1), pressure("S2", p2)],
            "failure_reasons": [],
            "solver_exit_code": 0
        })
    }

    const SYNTHETIC_INP: &[u8] = b"[JUNCTIONS]\nS1 0 0\nS2 0 0\nS3 0 0\n[RESERVOIRS]\nR1 100\n[PIPES]\nP1 S1 S2 100 10 100 0 OPEN\nP2 S2 R1 100 10 100 0 OPEN\n[OPTIONS]\nUNITS LPS\n[END]\n";

    fn synthetic_manifest() -> Vec<u8> {
        json!({
            "schema_version": SCHEMA_VERSION,
            "base_input_sha256": sha256(SYNTHETIC_INP),
            "unit_system": "LPS",
            "flow_unit": "L/s",
            "pressure_unit": "m",
            "observation_node_ids": ["S1", "S2"],
            "scenario_count": 5,
            "completed_count": 5,
            "failed_count": 0,
            "run_state": "Completed",
            "scenarios": [
                scenario("base", "no_fault", None, 0.0, 100.0, 100.0),
                scenario("a-small", "synthetic_withdrawal", Some("S1"), 1.0, 98.0, 99.0),
                scenario("b-small", "synthetic_withdrawal", Some("S2"), 1.0, 100.0, 97.0),
                scenario("a-large", "synthetic_withdrawal", Some("S1"), 2.0, 96.0, 98.0),
                scenario("b-large", "synthetic_withdrawal", Some("S2"), 2.0, 100.0, 94.0)
            ]
        }).to_string().into_bytes()
    }

    #[test]
    fn leave_one_magnitude_out_returns_two_folds_and_ranked_predictions() {
        let result = evaluate_localization_manifest(&synthetic_manifest(), SYNTHETIC_INP).unwrap();
        assert_eq!(result.evaluation.held_out_magnitudes, vec![1.0, 2.0]);
        assert_eq!(result.evaluation.fault_scenario_count, 4);
        assert_eq!(result.evaluation.candidate_target_count, 2);
        assert_eq!(result.evaluation.models.len(), 2);
        for model in &result.evaluation.models {
            assert_eq!(model.test_count, 4);
            assert_eq!(model.folds.len(), 2);
            assert!((model.top1_accuracy - 1.0).abs() < 1.0e-9);
            assert!((model.top_k_accuracy - 1.0).abs() < 1.0e-9);
            assert!((model.mean_reciprocal_rank - 1.0).abs() < 1.0e-9);
        }
    }

    #[test]
    fn no_fault_baseline_is_not_a_localization_candidate() {
        let result = evaluate_localization_manifest(&synthetic_manifest(), SYNTHETIC_INP).unwrap();
        for model in &result.evaluation.models {
            assert!(model.predictions.iter().all(|prediction| {
                prediction
                    .ranked_candidates
                    .iter()
                    .all(|candidate| candidate.target_junction_id != "no_fault")
            }));
        }
    }

    #[test]
    fn incomplete_solver_manifest_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&synthetic_manifest()).unwrap();
        value["failed_count"] = json!(1);
        value["run_state"] = json!("CompletedWithFailures");
        assert!(evaluate_localization_manifest(value.to_string().as_bytes(), SYNTHETIC_INP).is_err());
    }

    #[test]
    fn sensor_order_is_explicit_and_missing_measurements_fail_closed() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&synthetic_manifest()).unwrap();
        value["scenarios"][1]["pressure_readings"][0]["node_id"] = json!("UNDECLARED");
        assert!(evaluate_localization_manifest(value.to_string().as_bytes(), SYNTHETIC_INP).is_err());
    }

    #[test]
    fn unit_system_mismatch_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&synthetic_manifest()).unwrap();
        value["pressure_unit"] = json!("psi");
        assert!(evaluate_localization_manifest(value.to_string().as_bytes(), SYNTHETIC_INP).is_err());
    }

    #[test]
    fn target_classes_can_exceed_the_sensor_set() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&synthetic_manifest()).unwrap();
        value["scenario_count"] = json!(7);
        value["completed_count"] = json!(7);
        value["scenarios"].as_array_mut().unwrap().push(scenario(
            "c-small",
            "synthetic_withdrawal",
            Some("S3"),
            1.0,
            97.0,
            96.0,
        ));
        value["scenarios"].as_array_mut().unwrap().push(scenario(
            "c-large",
            "synthetic_withdrawal",
            Some("S3"),
            2.0,
            94.0,
            92.0,
        ));
        let result =
            evaluate_localization_manifest(value.to_string().as_bytes(), SYNTHETIC_INP).unwrap();
        assert_eq!(
            result.evaluation.observation_node_ids,
            vec!["S1".to_owned(), "S2".to_owned()]
        );
        assert_eq!(result.evaluation.candidate_target_count, 3);
        for model in &result.evaluation.models {
            assert!(model.predictions.iter().all(|prediction| {
                prediction.ranked_candidates.len() == 3
                    && prediction.ranked_candidates.iter().any(|candidate| {
                        candidate.target_junction_id == "S3"
                    })
            }));
        }
    }

    #[test]
    fn topology_metric_uses_undirected_link_hops_and_preserves_disconnection() {
        let topology = parse_network_topology(SYNTHETIC_INP, "LPS").unwrap();
        assert_eq!(topology.shortest_path_hops("S1", "S2"), Some(1));
        assert_eq!(topology.shortest_path_hops("S1", "R1"), Some(2));
        assert_eq!(topology.shortest_path_hops("S1", "S3"), None);
        assert_eq!(topology.link_count, 2);
    }

    #[test]
    fn topology_source_must_match_manifest_hash() {
        let changed_source = SYNTHETIC_INP
            .iter()
            .copied()
            .chain([b'\n'])
            .collect::<Vec<_>>();
        assert!(
            evaluate_localization_manifest(&synthetic_manifest(), &changed_source)
                .unwrap_err()
                .contains("SHA-256 does not match")
        );
    }

    #[test]
    fn output_reports_topological_error_metrics() {
        let result =
            evaluate_localization_manifest(&synthetic_manifest(), SYNTHETIC_INP).unwrap();
        for model in &result.evaluation.models {
            assert_eq!(model.disconnected_prediction_count, 0);
            assert_eq!(model.mean_topological_error_hops, Some(0.0));
            assert_eq!(model.p95_topological_error_hops, Some(0));
            assert_eq!(model.worst_topological_error_hops, Some(0));
            assert!(model.predictions.iter().all(|prediction| {
                prediction.topological_error_hops == Some(0)
            }));
        }
        assert_eq!(result.evaluation.topology_node_count, 4);
        assert_eq!(result.evaluation.topology_link_count, 2);
    }

    #[test]
    fn one_withdrawal_magnitude_is_not_enough_for_holdout() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&synthetic_manifest()).unwrap();
        value["scenarios"][3]["additional_demand"] = json!(1.0);
        value["scenarios"][4]["additional_demand"] = json!(1.0);
        assert!(
            evaluate_localization_manifest(value.to_string().as_bytes(), SYNTHETIC_INP)
                .unwrap_err()
                .contains("at least two withdrawal magnitudes")
        );
    }
}
