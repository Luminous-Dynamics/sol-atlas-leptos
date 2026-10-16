// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fair, report-reusing sensor-placement benchmark for completed synthetic EPANET corpora.
//!
//! All compared layouts use the same hash-verified hydraulic scenario reports.
//! Added noise is a dimensionless synthetic stress parameter, not an instrument model.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

const RUN_SCHEMA: &str = "water-scenario-corpus-run/v0.1";
const OUTPUT_SCHEMA: &str = "water-sensor-placement-evaluation/v0.1";
const NOISE_LEVELS: [f64; 3] = [0.0, 0.05, 0.10];
const REPLICATES: usize = 5;
const RANDOM_SEED: u64 = 0x534F_4C41_544C_4153;

#[derive(Debug, Clone, Deserialize)]
struct RunManifest {
    schema_version: String,
    plan_id: String,
    plan_sha256: String,
    base_input_sha256: String,
    unit_system: String,
    flow_unit: String,
    pressure_unit: String,
    observation_node_ids: Vec<String>,
    scenario_count: usize,
    completed_count: usize,
    failed_count: usize,
    run_state: String,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Deserialize)]
struct FrozenPlan {
    schema_version: String,
    plan_id: String,
    unit_system: String,
    base_input: PlanInput,
    observation_node_ids: Vec<String>,
    scenarios: Vec<PlannedScenario>,
}

#[derive(Debug, Clone, Deserialize)]
struct PlanInput {
    sha256: String,
    byte_length: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct PlannedScenario {
    scenario_id: String,
    ground_truth_class: String,
    target_junction_id: Option<String>,
    additional_demand: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct Scenario {
    scenario_id: String,
    ground_truth_class: String,
    target_junction_id: Option<String>,
    additional_demand: f64,
    run_state: String,
    pressure_readings: Vec<PressureReading>,
    report: Option<Artifact>,
}

#[derive(Debug, Clone, Deserialize)]
struct PressureReading {
    node_id: String,
    state: String,
    pressure_value: Option<f64>,
    reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Artifact {
    path: String,
    sha256: String,
    byte_length: u64,
}

#[derive(Debug, Clone)]
struct Topology {
    junctions: Vec<String>,
    adjacency: BTreeMap<String, Vec<String>>,
    link_count: usize,
}

impl Topology {
    fn hops(&self, source: &str, target: &str) -> Option<usize> {
        if source == target {
            return self.adjacency.contains_key(source).then_some(0);
        }
        if !self.adjacency.contains_key(source) || !self.adjacency.contains_key(target) {
            return None;
        }
        let mut queue = VecDeque::from([(source.to_owned(), 0usize)]);
        let mut visited = BTreeSet::from([source.to_owned()]);
        while let Some((node, depth)) = queue.pop_front() {
            for next in self.adjacency.get(&node).into_iter().flatten() {
                if !visited.insert(next.clone()) {
                    continue;
                }
                if next.as_str() == target {
                    return Some(depth + 1);
                }
                queue.push_back((next.clone(), depth + 1));
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
struct CaseFeature {
    scenario_id: String,
    target: String,
    magnitude: f64,
    values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensorPlacementEnvelope {
    pub schema_version: String,
    pub source_manifest_sha256: String,
    pub source_plan_sha256: String,
    pub source_topology_sha256: String,
    pub evaluation: SensorPlacementEvaluation,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensorPlacementEvaluation {
    pub schema_version: String,
    pub plan_id: String,
    pub run_state: String,
    pub unit_system: String,
    pub flow_unit: String,
    pub pressure_unit: String,
    pub topology_node_count: usize,
    pub topology_link_count: usize,
    pub junction_count: usize,
    pub scenario_count: usize,
    pub fault_scenario_count: usize,
    pub candidate_target_count: usize,
    pub sensor_budgets: Vec<usize>,
    pub noise_sigma_fractions: Vec<f64>,
    pub replicate_count: usize,
    pub held_out_magnitudes: Vec<f64>,
    pub strategy_ids: Vec<String>,
    pub results: Vec<SensorPlacementResult>,
    pub claim_ceiling: String,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensorPlacementResult {
    pub strategy_id: String,
    pub sensor_budget: usize,
    pub selected_sensor_node_ids: Vec<String>,
    pub noise_sigma_fraction: f64,
    pub replicate: usize,
    pub held_out_magnitude: f64,
    pub test_count: usize,
    pub top1_accuracy: f64,
    pub top3_accuracy: f64,
    pub mean_reciprocal_rank: f64,
    pub mean_topological_error_hops: Option<f64>,
    pub p95_topological_error_hops: Option<usize>,
    pub worst_topological_error_hops: Option<usize>,
    pub disconnected_prediction_count: usize,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn expected_units(system: &str) -> Result<(&'static str, &'static str), String> {
    match system {
        "LPS" => Ok(("L/s", "m")),
        "GPM" => Ok(("GPM", "psi")),
        _ => Err(format!("unsupported EPANET unit system {system:?}")),
    }
}

fn safe_relative_artifact(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if path.is_empty()
        || path.contains('\\')
        || p.is_absolute()
        || p.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("unsafe report artifact path {path:?}"));
    }
    Ok(())
}

fn parse_topology(input: &[u8], expected_system: &str) -> Result<Topology, String> {
    let text = std::str::from_utf8(input).map_err(|_| "source INP is not UTF-8".to_owned())?;
    if !text.is_ascii() {
        return Err("source INP must be ASCII".to_owned());
    }
    let mut section = String::new();
    let mut all_nodes = BTreeSet::new();
    let mut junctions = BTreeSet::new();
    let mut edges = Vec::<(String, String, String)>::new();
    let mut link_ids = BTreeSet::new();
    let mut units = Vec::<String>::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].trim().to_ascii_uppercase();
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        match section.as_str() {
            "OPTIONS" if fields[0].eq_ignore_ascii_case("UNITS") => {
                if fields.len() != 2 {
                    return Err(format!("malformed OPTIONS UNITS row: {line:?}"));
                }
                units.push(fields[1].to_ascii_uppercase());
            }
            "JUNCTIONS" | "RESERVOIRS" | "TANKS" => {
                let id = fields[0];
                if id.is_empty()
                    || id.len() > 31
                    || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                {
                    return Err(format!("invalid node ID {id:?}"));
                }
                if !all_nodes.insert(id.to_owned()) {
                    return Err(format!("duplicate node ID {id:?}"));
                }
                if section == "JUNCTIONS" {
                    junctions.insert(id.to_owned());
                }
            }
            "PIPES" | "PUMPS" | "VALVES" => {
                if fields.len() < 3 {
                    return Err(format!("malformed {section} row: {line:?}"));
                }
                let id = fields[0].to_owned();
                let from = fields[1].to_owned();
                let to = fields[2].to_owned();
                if !link_ids.insert(id.clone()) {
                    return Err(format!("duplicate link ID {id:?}"));
                }
                edges.push((id, from, to));
            }
            _ => {}
        }
    }
    if junctions.is_empty() {
        return Err("source INP has no [JUNCTIONS] records".to_owned());
    }
    if units.len() != 1 || units[0] != expected_system {
        return Err(format!(
            "source INP units {:?} do not match manifest {:?}",
            units, expected_system
        ));
    }
    if edges.is_empty() {
        return Err("source INP has no pipe, pump or valve links".to_owned());
    }
    let mut adjacency: BTreeMap<String, Vec<String>> =
        all_nodes.iter().map(|id| (id.clone(), Vec::new())).collect();
    for (id, from, to) in &edges {
        if !all_nodes.contains(from) || !all_nodes.contains(to) || from == to {
            return Err(format!("invalid endpoints for link {id:?}: {from:?} -> {to:?}"));
        }
        adjacency.get_mut(from).unwrap().push(to.clone());
        adjacency.get_mut(to).unwrap().push(from.clone());
    }
    for neighbours in adjacency.values_mut() {
        neighbours.sort();
        neighbours.dedup();
    }
    Ok(Topology {
        junctions: junctions.into_iter().collect(),
        adjacency,
        link_count: edges.len(),
    })
}

/// Parse pressure rows only from the first EPANET node-results period.
fn parse_first_period_pressures(
    report: &str,
    junction_ids: &[String],
) -> Result<BTreeMap<String, f64>, String> {
    let expected: BTreeSet<&str> = junction_ids.iter().map(String::as_str).collect();
    let mut output = BTreeMap::new();
    let mut in_nodes = false;
    let mut saw_nodes = false;
    for line in report.lines() {
        if line.contains("Node Results at ") {
            if saw_nodes {
                break;
            }
            saw_nodes = true;
            in_nodes = true;
            continue;
        }
        if in_nodes && line.contains("Link Results at ") {
            break;
        }
        if !in_nodes {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        let Some(id) = fields.first().copied() else {
            continue;
        };
        if !expected.contains(id) {
            continue;
        }
        if fields.len() < 4 {
            return Err(format!("node {id} pressure row has fewer than four fields"));
        }
        let pressure = fields[3]
            .parse::<f64>()
            .map_err(|_| format!("node {id} pressure is not numeric: {:?}", fields[3]))?;
        if !pressure.is_finite() {
            return Err(format!("node {id} pressure is not finite"));
        }
        if output.insert(id.to_owned(), pressure).is_some() {
            return Err(format!("duplicate first-period pressure row for node {id}"));
        }
    }
    if !saw_nodes {
        return Err("report has no Node Results table".to_owned());
    }
    let actual: BTreeSet<&str> = output.keys().map(String::as_str).collect();
    if actual != expected {
        let missing: Vec<&str> = expected.difference(&actual).copied().collect();
        return Err(format!("report is missing junction pressure rows: {missing:?}"));
    }
    Ok(output)
}

fn validate_declared_pressures(
    scenario: &Scenario,
    expected_node_ids: &[String],
    report_pressures: &BTreeMap<String, f64>,
) -> Result<(), String> {
    if scenario.pressure_readings.len() != expected_node_ids.len() {
        return Err(format!(
            "scenario {} manifest has {} pressure readings; expected {}",
            scenario.scenario_id,
            scenario.pressure_readings.len(),
            expected_node_ids.len()
        ));
    }

    let mut by_id = BTreeMap::<String, f64>::new();
    for reading in &scenario.pressure_readings {
        if reading.state != "measured" || reading.reason.is_some() {
            return Err(format!(
                "scenario {} manifest pressure for {} is not a clean measurement",
                scenario.scenario_id, reading.node_id
            ));
        }
        let value = reading.pressure_value.ok_or_else(|| {
            format!(
                "scenario {} manifest pressure for {} is null",
                scenario.scenario_id, reading.node_id
            )
        })?;
        if !value.is_finite() {
            return Err(format!(
                "scenario {} manifest pressure for {} is non-finite",
                scenario.scenario_id, reading.node_id
            ));
        }
        if by_id.insert(reading.node_id.clone(), value).is_some() {
            return Err(format!(
                "scenario {} manifest has duplicate pressure for {}",
                scenario.scenario_id, reading.node_id
            ));
        }
    }

    let expected: BTreeSet<&str> = expected_node_ids.iter().map(String::as_str).collect();
    let actual: BTreeSet<&str> = by_id.keys().map(String::as_str).collect();
    if expected != actual {
        return Err(format!(
            "scenario {} manifest pressure sensor IDs differ from the frozen observation set",
            scenario.scenario_id
        ));
    }

    for node_id in expected_node_ids {
        let declared = by_id.get(node_id).ok_or_else(|| {
            format!(
                "scenario {} manifest is missing pressure for {}",
                scenario.scenario_id, node_id
            )
        })?;
        let reported = report_pressures.get(node_id).ok_or_else(|| {
            format!(
                "scenario {} verified report is missing pressure for {}",
                scenario.scenario_id, node_id
            )
        })?;
        if declared.to_bits() != reported.to_bits() {
            return Err(format!(
                "scenario {} declared pressure for {} does not match the hash-verified report",
                scenario.scenario_id, node_id
            ));
        }
    }
    Ok(())
}

fn read_verified_report(root: &Path, artifact: &Artifact) -> Result<Vec<u8>, String> {
    safe_relative_artifact(&artifact.path)?;
    let root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve corpus root: {e}"))?;
    let candidate = root.join(&artifact.path);
    let path = candidate
        .canonicalize()
        .map_err(|e| format!("cannot resolve report {}: {e}", artifact.path))?;
    if !path.starts_with(&root) {
        return Err(format!("report artifact escapes corpus root: {}", artifact.path));
    }
    let bytes = fs::read(&path).map_err(|e| format!("cannot read report {}: {e}", path.display()))?;
    if bytes.len() as u64 != artifact.byte_length {
        return Err(format!("report byte-length mismatch for {}", artifact.path));
    }
    if sha256(&bytes) != artifact.sha256 {
        return Err(format!("report SHA-256 mismatch for {}", artifact.path));
    }
    Ok(bytes)
}

fn unit_normalization_scales(training: &[CaseFeature], width: usize) -> Vec<f64> {
    (0..width)
        .map(|index| {
            let values: Vec<f64> = training.iter().map(|case| case.values[index]).collect();
            if values.is_empty() {
                return 1.0;
            }
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter().map(|value| (value - mean).powi(2)).sum::<f64>()
                / values.len() as f64;
            variance.sqrt().max(1.0e-9)
        })
        .collect()
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn uniform_open01(state: &mut u64) -> f64 {
    let value = (splitmix64(state) >> 11) as f64;
    (value + 0.5) / ((1u64 << 53) as f64)
}

fn standard_normal(state: &mut u64) -> f64 {
    let u1 = uniform_open01(state);
    let u2 = uniform_open01(state);
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

fn standardized_noisy_features(
    features: &[CaseFeature],
    scales: &[f64],
    sigma_fraction: f64,
    seed: u64,
) -> Vec<CaseFeature> {
    let mut state = seed;
    features
        .iter()
        .map(|case| CaseFeature {
            scenario_id: case.scenario_id.clone(),
            target: case.target.clone(),
            magnitude: case.magnitude,
            values: case
                .values
                .iter()
                .zip(scales)
                .map(|(value, scale)| {
                    let normalized = value / scale;
                    if sigma_fraction == 0.0 {
                        normalized
                    } else {
                        normalized + sigma_fraction * standard_normal(&mut state)
                    }
                })
                .collect(),
        })
        .collect()
}

fn l2_distance(left: &[f64], right: &[f64], sites: &[usize]) -> f64 {
    sites
        .iter()
        .map(|&site| (left[site] - right[site]).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn ranked_targets(
    test: &CaseFeature,
    training: &[CaseFeature],
    candidates: &[String],
    sites: &[usize],
) -> Vec<(String, f64)> {
    let mut ranked: Vec<(String, f64)> = candidates
        .iter()
        .map(|target| {
            let distance = training
                .iter()
                .filter(|example| example.target == *target)
                .map(|example| l2_distance(&test.values, &example.values, sites))
                .fold(f64::INFINITY, f64::min);
            (target.clone(), distance)
        })
        .collect();
    ranked.sort_by(|left, right| left.1.total_cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    ranked
}

fn fixed_site_order(candidates: &[String], declared_sites: &[String]) -> Vec<String> {
    let candidate_set: BTreeSet<&str> = candidates.iter().map(String::as_str).collect();
    let mut order = Vec::new();
    for id in declared_sites {
        if candidate_set.contains(id.as_str()) && !order.contains(id) {
            order.push(id.clone());
        }
    }
    for id in candidates {
        if !order.contains(id) {
            order.push(id.clone());
        }
    }
    order
}

fn coverage_site_order(
    candidates: &[String],
    fixed_order: &[String],
    topology: &Topology,
) -> Vec<String> {
    let mut selected = Vec::<String>::new();
    if let Some(first) = fixed_order.first() {
        selected.push(first.clone());
    } else if let Some(first) = candidates.first() {
        selected.push(first.clone());
    }
    while selected.len() < candidates.len() {
        let next = candidates
            .iter()
            .filter(|candidate| !selected.contains(candidate))
            .max_by(|left, right| {
                let left_min = selected
                    .iter()
                    .map(|chosen| topology.hops(left, chosen).unwrap_or(usize::MAX / 4))
                    .min()
                    .unwrap_or(0);
                let right_min = selected
                    .iter()
                    .map(|chosen| topology.hops(right, chosen).unwrap_or(usize::MAX / 4))
                    .min()
                    .unwrap_or(0);
                left_min.cmp(&right_min).then_with(|| right.cmp(left))
            });
        let Some(next) = next else { break };
        selected.push(next.clone());
    }
    selected
}

fn response_separation_site_order(
    training: &[CaseFeature],
    candidates: &[String],
    site_ids: &[String],
) -> Vec<String> {
    let mut centroids = Vec::<Vec<f64>>::new();
    for target in candidates {
        let rows: Vec<&CaseFeature> =
            training.iter().filter(|row| row.target == *target).collect();
        if rows.is_empty() {
            return site_ids.to_vec();
        }
        let mut centroid = vec![0.0; site_ids.len()];
        for row in &rows {
            for (site, value) in row.values.iter().enumerate() {
                centroid[site] += value / rows.len() as f64;
            }
        }
        centroids.push(centroid);
    }

    // Cache the accumulated squared distance for each target pair. Each candidate
    // site adds one squared-difference term, avoiding recomputing every selected
    // dimension for every candidate at every greedy step.
    let target_count = candidates.len();
    let site_count = site_ids.len();
    let mut accumulated = vec![vec![0.0; target_count]; target_count];
    let mut selected = vec![false; site_count];
    let mut order = Vec::with_capacity(site_count);

    while order.len() < site_count {
        let mut best: Option<(usize, f64)> = None;
        for site in 0..site_count {
            if selected[site] {
                continue;
            }
            let mut minimum_squared = f64::INFINITY;
            for left in 0..target_count {
                for right in (left + 1)..target_count {
                    let delta = centroids[left][site] - centroids[right][site];
                    minimum_squared =
                        minimum_squared.min(accumulated[left][right] + delta * delta);
                }
            }
            let replace = best.as_ref().is_none_or(|(best_site, best_score)| {
                minimum_squared > *best_score
                    || (minimum_squared == *best_score
                        && site_ids[site] < site_ids[*best_site])
            });
            if replace {
                best = Some((site, minimum_squared));
            }
        }
        let Some((chosen, _)) = best else { break };
        selected[chosen] = true;
        order.push(site_ids[chosen].clone());
        for left in 0..target_count {
            for right in (left + 1)..target_count {
                let delta = centroids[left][chosen] - centroids[right][chosen];
                accumulated[left][right] += delta * delta;
            }
        }
    }
    order
}

fn deterministic_random_order(candidates: &[String], seed: u64) -> Vec<String> {
    let mut order = candidates.to_vec();
    let mut state = seed;
    for index in (1..order.len()).rev() {
        let target = (splitmix64(&mut state) as usize) % (index + 1);
        order.swap(index, target);
    }
    order
}

fn summarize_predictions(
    strategy_id: &str,
    budget: usize,
    selected_sensors: &[String],
    noise_fraction: f64,
    replicate: usize,
    held_out_magnitude: f64,
    training: &[CaseFeature],
    testing: &[CaseFeature],
    candidates: &[String],
    site_ids: &[String],
    topology: &Topology,
) -> SensorPlacementResult {
    let site_indices: Vec<usize> = selected_sensors
        .iter()
        .filter_map(|selected| site_ids.iter().position(|site| site == selected))
        .collect();
    let mut top1 = 0usize;
    let mut top3 = 0usize;
    let mut reciprocal_rank = 0.0;
    let mut errors = Vec::<usize>::new();
    let mut disconnected = 0usize;

    for test in testing {
        let ranked = ranked_targets(test, training, candidates, &site_indices);
        let actual_rank = ranked
            .iter()
            .position(|(target, _)| target == &test.target)
            .map(|index| index + 1)
            .unwrap_or(candidates.len() + 1);
        if actual_rank == 1 {
            top1 += 1;
        }
        if actual_rank <= 3 {
            top3 += 1;
        }
        reciprocal_rank += 1.0 / actual_rank as f64;
        let predicted = ranked.first().map(|row| row.0.as_str()).unwrap_or("");
        match topology.hops(&test.target, predicted) {
            Some(distance) => errors.push(distance),
            None => disconnected += 1,
        }
    }
    errors.sort_unstable();
    let count = testing.len();
    let mean_error = if errors.is_empty() {
        None
    } else {
        Some(errors.iter().sum::<usize>() as f64 / errors.len() as f64)
    };
    let p95 = if errors.is_empty() {
        None
    } else {
        let rank = (errors.len() * 95 + 99) / 100;
        Some(errors[rank.saturating_sub(1)])
    };
    SensorPlacementResult {
        strategy_id: strategy_id.to_owned(),
        sensor_budget: budget,
        selected_sensor_node_ids: selected_sensors.to_vec(),
        noise_sigma_fraction: noise_fraction,
        replicate,
        held_out_magnitude,
        test_count: count,
        top1_accuracy: top1 as f64 / count as f64,
        top3_accuracy: top3 as f64 / count as f64,
        mean_reciprocal_rank: reciprocal_rank / count as f64,
        mean_topological_error_hops: mean_error,
        p95_topological_error_hops: p95,
        worst_topological_error_hops: errors.last().copied(),
        disconnected_prediction_count: disconnected,
    }
}

/// Evaluate fixed, seeded-random, graph-coverage and response-separation layouts
/// on the same hash-bound, completed corpus. All node-report artifacts are verified
/// before parsing; all target labels are withheld until after candidate ranking.
pub fn evaluate_sensor_placement(
    corpus_root: &Path,
    manifest_bytes: &[u8],
    source_inp: &[u8],
    plan_bytes: &[u8],
) -> Result<SensorPlacementEnvelope, String> {
    let manifest: RunManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| format!("invalid scenario run manifest JSON: {e}"))?;
    if manifest.schema_version != RUN_SCHEMA
        || manifest.run_state != "Completed"
        || manifest.failed_count != 0
        || manifest.completed_count != manifest.scenario_count
        || manifest.scenarios.len() != manifest.scenario_count
    {
        return Err("scenario manifest must be fully Completed with no failed runs".to_owned());
    }
    let plan: FrozenPlan = serde_json::from_slice(plan_bytes)
        .map_err(|e| format!("invalid frozen scenario plan JSON: {e}"))?;
    if plan.schema_version != "water-scenario-corpus-plan/v0.1"
        || plan.plan_id != manifest.plan_id
        || manifest.plan_sha256 != sha256(plan_bytes)
    {
        return Err("run manifest plan identity or SHA-256 does not match frozen plan".to_owned());
    }
    if plan.unit_system != manifest.unit_system
        || plan.base_input.sha256 != manifest.base_input_sha256
        || plan.base_input.byte_length != source_inp.len() as u64
        || plan.observation_node_ids != manifest.observation_node_ids
        || plan.scenarios.len() != manifest.scenario_count
    {
        return Err("frozen plan does not match run manifest or source input".to_owned());
    }
    let mut planned = BTreeMap::<String, &PlannedScenario>::new();
    for scenario in &plan.scenarios {
        if planned.insert(scenario.scenario_id.clone(), scenario).is_some() {
            return Err(format!("frozen plan has duplicate scenario ID {:?}", scenario.scenario_id));
        }
    }
    if planned.len() != manifest.scenarios.len() {
        return Err("frozen plan and run manifest scenario counts differ".to_owned());
    }
    for scenario in &manifest.scenarios {
        let expected = planned.get(&scenario.scenario_id).ok_or_else(|| {
            format!(
                "run manifest has undeclared scenario {:?}",
                scenario.scenario_id
            )
        })?;
        if scenario.ground_truth_class != expected.ground_truth_class
            || scenario.target_junction_id != expected.target_junction_id
            || scenario.additional_demand != expected.additional_demand
        {
            return Err(format!(
                "scenario {} labels/perturbation differ from frozen plan",
                scenario.scenario_id
            ));
        }
    }
    if sha256(source_inp) != manifest.base_input_sha256 {
        return Err("source INP SHA-256 differs from run manifest base_input_sha256".to_owned());
    }
    let (flow_unit, pressure_unit) = expected_units(&manifest.unit_system)?;
    if manifest.flow_unit != flow_unit || manifest.pressure_unit != pressure_unit {
        return Err("run manifest's units are inconsistent with EPANET unit system".to_owned());
    }
    let topology = parse_topology(source_inp, &manifest.unit_system)?;
    if manifest.observation_node_ids.is_empty() {
        return Err("manifest observation_node_ids must not be empty".to_owned());
    }
    let unique_observation_ids: BTreeSet<&str> =
        manifest.observation_node_ids.iter().map(String::as_str).collect();
    if unique_observation_ids.len() != manifest.observation_node_ids.len() {
        return Err("manifest observation_node_ids contains duplicates".to_owned());
    }
    for node in &manifest.observation_node_ids {
        if !topology.junctions.contains(node) {
            return Err(format!("declared observation node {node:?} is not a junction"));
        }
    }

    let mut scenario_ids = HashSet::new();
    let mut raw_pressures = BTreeMap::<String, BTreeMap<String, f64>>::new();
    let mut baseline_ids = Vec::new();
    for scenario in &manifest.scenarios {
        if scenario.scenario_id.is_empty() || !scenario_ids.insert(scenario.scenario_id.clone()) {
            return Err(format!("empty or duplicate scenario ID {:?}", scenario.scenario_id));
        }
        if scenario.run_state != "Completed"
            || !scenario.additional_demand.is_finite()
            || scenario.additional_demand < 0.0
        {
            return Err(format!(
                "scenario {} is incomplete or has invalid perturbation",
                scenario.scenario_id
            ));
        }
        if scenario.ground_truth_class == "no_fault"
            && scenario.target_junction_id.is_none()
            && scenario.additional_demand == 0.0
        {
            baseline_ids.push(scenario.scenario_id.clone());
        }
        let report = scenario.report.as_ref()
            .ok_or_else(|| format!("scenario {} has no report artifact", scenario.scenario_id))?;
        let bytes = read_verified_report(corpus_root, report)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| format!("scenario {} report is not UTF-8", scenario.scenario_id))?;
        let pressures = parse_first_period_pressures(text, &topology.junctions)
            .map_err(|e| format!("scenario {}: {e}", scenario.scenario_id))?;
        validate_declared_pressures(scenario, &manifest.observation_node_ids, &pressures)?;
        raw_pressures.insert(scenario.scenario_id.clone(), pressures);
    }
    if baseline_ids.len() != 1 {
        return Err(format!("expected exactly one no-fault baseline, found {}", baseline_ids.len()));
    }
    let baseline_id = &baseline_ids[0];
    let baseline = raw_pressures.get(baseline_id)
        .ok_or_else(|| "no-fault pressure field unavailable".to_owned())?;

    let mut features = Vec::<CaseFeature>::new();
    let mut label_set = BTreeSet::<String>::new();
    for scenario in &manifest.scenarios {
        if scenario.scenario_id == *baseline_id {
            continue;
        }
        if scenario.ground_truth_class != "synthetic_withdrawal"
            || scenario.additional_demand <= 0.0
        {
            return Err(format!(
                "unsupported non-baseline scenario {}",
                scenario.scenario_id
            ));
        }
        let target = scenario.target_junction_id.as_ref()
            .ok_or_else(|| format!("scenario {} has no target junction", scenario.scenario_id))?;
        if !topology.junctions.contains(target) {
            return Err(format!(
                "scenario {} target {target:?} is not a junction",
                scenario.scenario_id
            ));
        }
        let pressures = raw_pressures.get(&scenario.scenario_id)
            .ok_or_else(|| format!("scenario {} pressures unavailable", scenario.scenario_id))?;
        let values = topology.junctions.iter()
            .map(|junction| pressures[junction] - baseline[junction])
            .collect::<Vec<_>>();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "scenario {} has non-finite pressure-change signature",
                scenario.scenario_id
            ));
        }
        label_set.insert(target.clone());
        features.push(CaseFeature {
            scenario_id: scenario.scenario_id.clone(),
            target: target.clone(),
            magnitude: scenario.additional_demand,
            values,
        });
    }
    if label_set.len() < 2 {
        return Err("sensor-placement evaluation needs at least two target classes".to_owned());
    }
    let candidates = label_set.into_iter().collect::<Vec<_>>();
    let site_ids = topology.junctions.clone();
    let mut magnitudes = features.iter().map(|row| row.magnitude).collect::<Vec<_>>();
    magnitudes.sort_by(f64::total_cmp);
    magnitudes.dedup_by(|a, b| *a == *b);
    if magnitudes.len() < 2 {
        return Err(
            "sensor-placement evaluation needs at least two perturbation magnitudes".to_owned(),
        );
    }
    for magnitude in &magnitudes {
        for target in &candidates {
            if !features.iter().any(|row| row.magnitude == *magnitude && row.target == *target) {
                return Err(format!("target {target} is absent at held-out magnitude {magnitude}"));
            }
        }
    }

    let max_budget = site_ids.len();
    let mut budgets = Vec::new();
    for proposed in [2usize, 4, 8] {
        let budget = proposed.min(max_budget);
        if budget > 0 && !budgets.contains(&budget) {
            budgets.push(budget);
        }
    }
    let fixed_order = fixed_site_order(&site_ids, &manifest.observation_node_ids);
    let coverage_order = coverage_site_order(&site_ids, &fixed_order, &topology);
    let strategy_ids = vec![
        "fixed-predeclared".to_owned(),
        "seeded-random".to_owned(),
        "graph-coverage-greedy".to_owned(),
        "response-separation-greedy".to_owned(),
    ];
    let mut results = Vec::new();
    for held_out in &magnitudes {
        let clean_training = features
            .iter()
            .filter(|row| row.magnitude != *held_out)
            .cloned()
            .collect::<Vec<_>>();
        let clean_testing = features
            .iter()
            .filter(|row| row.magnitude == *held_out)
            .cloned()
            .collect::<Vec<_>>();
        if clean_training.is_empty() || clean_testing.is_empty() {
            return Err(format!("empty train or test fold at magnitude {held_out}"));
        }
        let scales = unit_normalization_scales(&clean_training, site_ids.len());
        let clean_normalized = standardized_noisy_features(&features, &scales, 0.0, RANDOM_SEED);
        let clean_training = clean_normalized
            .iter()
            .filter(|row| row.magnitude != *held_out)
            .cloned()
            .collect::<Vec<_>>();
        let response_order =
            response_separation_site_order(&clean_training, &candidates, &site_ids);
        for (noise_index, noise_fraction) in NOISE_LEVELS.iter().copied().enumerate() {
            for replicate in 0..REPLICATES {
                let seed = RANDOM_SEED
                    ^ held_out.to_bits()
                    ^ ((noise_index as u64) << 32)
                    ^ (replicate as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                let noisy = standardized_noisy_features(&features, &scales, noise_fraction, seed);
                let training = noisy
                    .iter()
                    .filter(|row| row.magnitude != *held_out)
                    .cloned()
                    .collect::<Vec<_>>();
                let testing = noisy
                    .iter()
                    .filter(|row| row.magnitude == *held_out)
                    .cloned()
                    .collect::<Vec<_>>();
                let random_order = deterministic_random_order(
                    &site_ids,
                    RANDOM_SEED ^ held_out.to_bits() ^ (replicate as u64).wrapping_mul(0x9E37_79B9),
                );
                for budget in &budgets {
                    let layout_candidates = [
                        ("fixed-predeclared", &fixed_order),
                        ("seeded-random", &random_order),
                        ("graph-coverage-greedy", &coverage_order),
                        ("response-separation-greedy", &response_order),
                    ];
                    for (strategy, order) in layout_candidates {
                        let selected = order.iter().take(*budget).cloned().collect::<Vec<_>>();
                        let ids = selected.iter().map(String::as_str).collect::<BTreeSet<_>>();
                        if selected.len() != *budget || ids.len() != *budget {
                            return Err(format!(
                                "strategy {strategy} failed to produce {budget} unique sites"
                            ));
                        }
                        results.push(summarize_predictions(
                            strategy,
                            *budget,
                            &selected,
                            noise_fraction,
                            replicate,
                            *held_out,
                            &training,
                            &testing,
                            &candidates,
                            &site_ids,
                            &topology,
                        ));
                    }
                }
            }
        }
    }
    results.sort_by(|a, b| {
        a.held_out_magnitude.total_cmp(&b.held_out_magnitude)
            .then(a.noise_sigma_fraction.total_cmp(&b.noise_sigma_fraction))
            .then(a.sensor_budget.cmp(&b.sensor_budget))
            .then(a.replicate.cmp(&b.replicate))
            .then(a.strategy_id.cmp(&b.strategy_id))
    });
    Ok(SensorPlacementEnvelope {
        schema_version: "water-sensor-placement-evaluation-envelope/v0.1".to_owned(),
        source_manifest_sha256: sha256(manifest_bytes),
        source_plan_sha256: sha256(plan_bytes),
        source_topology_sha256: sha256(source_inp),
        evaluation: SensorPlacementEvaluation {
            schema_version: OUTPUT_SCHEMA.to_owned(),
            plan_id: manifest.plan_id,
            run_state: "Completed".to_owned(),
            unit_system: manifest.unit_system,
            flow_unit: manifest.flow_unit,
            pressure_unit: manifest.pressure_unit,
            topology_node_count: topology.adjacency.len(),
            topology_link_count: topology.link_count,
            junction_count: topology.junctions.len(),
            scenario_count: manifest.scenario_count,
            fault_scenario_count: features.len(),
            candidate_target_count: candidates.len(),
            sensor_budgets: budgets,
            noise_sigma_fractions: NOISE_LEVELS.to_vec(),
            replicate_count: REPLICATES,
            held_out_magnitudes: magnitudes,
            strategy_ids,
            results,
            claim_ceiling: "synthetic_sensor_layout_comparison_only".to_owned(),
            nonclaims: vec![
                "The corpus labels are synthetic junction-demand perturbations, not physical leaks."
                    .to_owned(),
                concat!(
                    "Noise levels are dimensionless synthetic stress fractions; they are not ",
                    "calibrated pressure-transducer specifications."
                )
                .to_owned(),
                concat!(
                    "All strategies reuse the same hydraulic reports; no extra hydraulic solver ",
                    "run is performed per layout."
                )
                .to_owned(),
                concat!(
                    "The held-out axis is perturbation magnitude within one topology; no cross-topology, ",
                    "operating-regime, or real-network generalization is established."
                )
                .to_owned(),
                concat!(
                    "The fixed layout extends the declared observation-node list in junction-ID order ",
                    "when a budget exceeds that list; it is a comparison baseline, not an asserted ",
                    "optimal design."
                )
                .to_owned(),
                concat!(
                    "Graph-hop error is an unweighted diagnostic, not pipe-length, hydraulic ",
                    "influence, travel time, or repair cost."
                )
                .to_owned(),
                concat!(
                    "No field sensor economics, false-alarm rate, detection delay, engineering approval, ",
                    "or operational readiness is established."
                )
                .to_owned(),
            ],
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_reads_all_junctions_from_first_reported_period() {
        let ids = vec!["J1".to_owned(), "J2".to_owned()];
        let report = concat!(
            "Node Results at 0:00:00\n",
            "Node Demand Head Pressure\n",
            "J1 0.00 100.00 40.00\n",
            "J2 2.00 90.00 30.00\n",
            "Link Results at 0:00:00\n",
            "J1 999.00 999.00 999.00\n",
        );
        let parsed = parse_first_period_pressures(report, &ids).unwrap();
        assert_eq!(parsed["J1"], 40.0);
        assert_eq!(parsed["J2"], 30.0);
    }

    #[test]
    fn parser_rejects_missing_nonfinite_or_duplicate_pressure_rows() {
        let ids = vec!["J1".to_owned(), "J2".to_owned()];
        let missing = "Node Results at 0:00:00\nJ1 0 10 2\nLink Results at 0:00:00\n";
        assert!(parse_first_period_pressures(missing, &ids).unwrap_err().contains("missing"));
        let nonfinite = concat!(
            "Node Results at 0:00:00\n",
            "J1 0 10 NaN\nJ2 0 10 2\n",
            "Link Results at 0:00:00\n"
        );
        assert!(parse_first_period_pressures(nonfinite, &ids).is_err());
        let duplicate = concat!(
            "Node Results at 0:00:00\n",
            "J1 0 10 2\nJ1 0 10 3\nJ2 0 10 2\n",
            "Link Results at 0:00:00\n"
        );
        assert!(parse_first_period_pressures(duplicate, &ids).unwrap_err().contains("duplicate"));
    }

    #[test]
    fn noise_is_seeded_and_standardization_is_finite() {
        let rows = vec![CaseFeature {
            scenario_id: "x".to_owned(),
            target: "J1".to_owned(),
            magnitude: 1.0,
            values: vec![2.0, -4.0],
        }];
        let scales = vec![2.0, 4.0];
        let a = standardized_noisy_features(&rows, &scales, 0.1, 42);
        let b = standardized_noisy_features(&rows, &scales, 0.1, 42);
        assert_eq!(a[0].values, b[0].values);
        assert!(a[0].values.iter().all(|v| v.is_finite()));
        assert_eq!(
            standardized_noisy_features(&rows, &scales, 0.0, 42)[0].values,
            vec![1.0, -1.0]
        );
    }

    #[test]
    fn layouts_are_unique_and_coverage_is_deterministic() {
        let nodes = vec!["J1".to_owned(), "J2".to_owned(), "J3".to_owned()];
        let inp = concat!(
            "[JUNCTIONS]\nJ1 0 1\nJ2 0 1\nJ3 0 1\n",
            "[RESERVOIRS]\nR1 100\n",
            "[PIPES]\nP1 R1 J1 1 1 1 0 OPEN\nP2 J1 J2 1 1 1 0 OPEN\nP3 J2 J3 1 1 1 0 OPEN\n",
            "[OPTIONS]\nUNITS LPS\n[END]\n"
        ).as_bytes();
        let topology = parse_topology(inp, "LPS").unwrap();
        let fixed = fixed_site_order(&nodes, &["J2".to_owned()]);
        let a = coverage_site_order(&nodes, &fixed, &topology);
        let b = coverage_site_order(&nodes, &fixed, &topology);
        assert_eq!(a, b);
        assert_eq!(a.len(), 3);
        assert_eq!(a.iter().collect::<BTreeSet<_>>().len(), 3);
        let random_a = deterministic_random_order(&nodes, 7);
        let random_b = deterministic_random_order(&nodes, 7);
        assert_eq!(random_a, random_b);
    }

    #[test]
    fn source_topology_rejects_unit_mismatch_and_missing_link_endpoints() {
        let good = concat!(
            "[JUNCTIONS]\nJ1 0 1\n[RESERVOIRS]\nR1 100\n",
            "[PIPES]\nP1 R1 J1 1 1 1 0 OPEN\n",
            "[OPTIONS]\nUNITS LPS\n[END]\n"
        ).as_bytes();
        assert!(parse_topology(good, "LPS").is_ok());
        assert!(parse_topology(good, "GPM").is_err());
        let broken = concat!(
            "[JUNCTIONS]\nJ1 0 1\n[RESERVOIRS]\nR1 100\n",
            "[PIPES]\nP1 R1 J2 1 1 1 0 OPEN\n",
            "[OPTIONS]\nUNITS LPS\n[END]\n"
        ).as_bytes();
        assert!(parse_topology(broken, "LPS").is_err());
    }

    
    #[test]
    fn end_to_end_reuses_verified_all_node_reports_for_every_layout() {
        use serde_json::{json, Value};
        use std::time::{SystemTime, UNIX_EPOCH};

        fn add_scenario(
            root: &Path,
            scenarios: &mut Vec<Value>,
            id: &str,
            class: &str,
            target: Option<&str>,
            magnitude: f64,
            pressures: [f64; 3],
        ) {
            let relative = format!("{id}/scenario.rpt");
            let report_path = root.join(&relative);
            fs::create_dir_all(report_path.parent().unwrap()).unwrap();
            let report = format!(
                concat!(
                    "Node Results at 0:00:00\n",
                    "Node Demand Head Pressure\n",
                    "J1 0 100 {}\n",
                    "J2 0 100 {}\n",
                    "J3 0 100 {}\n",
                    "Link Results at 0:00:00\n"
                ),
                pressures[0], pressures[1], pressures[2]
            );
            fs::write(&report_path, report.as_bytes()).unwrap();
            scenarios.push(json!({
                "scenario_id": id,
                "ground_truth_class": class,
                "target_junction_id": target,
                "additional_demand": magnitude,
                "run_state": "Completed",
                "pressure_readings": [
                    {
                        "node_id": "J1",
                        "state": "measured",
                        "pressure_value": pressures[0],
                        "reason": null
                    },
                    {
                        "node_id": "J2",
                        "state": "measured",
                        "pressure_value": pressures[1],
                        "reason": null
                    }
                ],
                "report": {
                    "path": relative,
                    "sha256": sha256(report.as_bytes()),
                    "byte_length": report.len()
                }
            }));
        }

        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sol-atlas-sensor-placement-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let source_inp = concat!(
            "[JUNCTIONS]\nJ1 0 1\nJ2 0 1\nJ3 0 1\n",
            "[RESERVOIRS]\nR1 100\n",
            "[PIPES]\nP1 R1 J1 100 100 100 0 OPEN\n",
            "P2 J1 J2 100 100 100 0 OPEN\n",
            "P3 J2 J3 100 100 100 0 OPEN\n",
            "[OPTIONS]\nUNITS LPS\n[END]\n"
        ).as_bytes();
        let mut scenarios = Vec::new();
        add_scenario(&root, &mut scenarios, "base", "no_fault", None, 0.0, [50.0, 50.0, 50.0]);
        add_scenario(
            &root, &mut scenarios, "j1-small", "synthetic_withdrawal", Some("J1"),
            1.0, [49.0, 50.2, 50.1],
        );
        add_scenario(
            &root, &mut scenarios, "j1-large", "synthetic_withdrawal", Some("J1"),
            2.0, [48.0, 50.3, 50.2],
        );
        add_scenario(
            &root, &mut scenarios, "j2-small", "synthetic_withdrawal", Some("J2"),
            1.0, [50.2, 49.0, 50.1],
        );
        add_scenario(
            &root, &mut scenarios, "j2-large", "synthetic_withdrawal", Some("J2"),
            2.0, [50.3, 48.0, 50.2],
        );
        add_scenario(
            &root, &mut scenarios, "j3-small", "synthetic_withdrawal", Some("J3"),
            1.0, [50.1, 50.2, 49.0],
        );
        add_scenario(
            &root, &mut scenarios, "j3-large", "synthetic_withdrawal", Some("J3"),
            2.0, [50.2, 50.3, 48.0],
        );
        let planned_scenarios = scenarios.clone();
        let plan = json!({
            "schema_version": "water-scenario-corpus-plan/v0.1",
            "classification": "synthetic_public",
            "plan_id": "test-plan",
            "unit_system": "LPS",
            "base_input": {
                "path": "synthetic.inp",
                "sha256": sha256(source_inp),
                "byte_length": source_inp.len()
            },
            "observation_node_ids": ["J1", "J2"],
            "scenarios": planned_scenarios.iter().map(|row| json!({
                "scenario_id": row["scenario_id"],
                "ground_truth_class": row["ground_truth_class"],
                "target_junction_id": row["target_junction_id"],
                "additional_demand": row["additional_demand"]
            })).collect::<Vec<_>>(),
            "assumptions": [],
            "nonclaims": []
        });
        let plan_bytes = serde_json::to_vec(&plan).unwrap();
        let manifest = json!({
            "schema_version": RUN_SCHEMA,
            "plan_id": "test-plan",
            "plan_sha256": sha256(&plan_bytes),
            "base_input_sha256": sha256(source_inp),
            "unit_system": "LPS",
            "flow_unit": "L/s",
            "pressure_unit": "m",
            "observation_node_ids": ["J1", "J2"],
            "scenario_count": scenarios.len(),
            "completed_count": scenarios.len(),
            "failed_count": 0,
            "run_state": "Completed",
            "scenarios": scenarios
        });
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let result =
            evaluate_sensor_placement(&root, &manifest_bytes, source_inp, &plan_bytes).unwrap();
        assert_eq!(result.evaluation.scenario_count, 7);
        assert_eq!(result.evaluation.fault_scenario_count, 6);
        assert_eq!(result.evaluation.candidate_target_count, 3);
        assert_eq!(result.evaluation.sensor_budgets, vec![2, 3]);
        assert_eq!(result.evaluation.results.len(), 240);
        assert!(result.evaluation.results.iter().all(|row| row.test_count == 3));
        assert!(result.evaluation.results.iter().all(|row| {
            row.top1_accuracy.is_finite()
                && (0.0..=1.0).contains(&row.top1_accuracy)
                && row.selected_sensor_node_ids.len() == row.sensor_budget
        }));

        let mut tampered: Value = serde_json::from_slice(&manifest_bytes).unwrap();
        tampered["scenarios"][1]["report"]["sha256"] = json!("0".repeat(64));
        let tampered_bytes = serde_json::to_vec(&tampered).unwrap();
        assert!(evaluate_sensor_placement(&root, &tampered_bytes, source_inp, &plan_bytes)
            .unwrap_err()
            .contains("SHA-256 mismatch"));

        let mut inconsistent: Value = serde_json::from_slice(&manifest_bytes).unwrap();
        inconsistent["scenarios"][1]["pressure_readings"][0]["pressure_value"] = json!(123.0);
        let inconsistent_bytes = serde_json::to_vec(&inconsistent).unwrap();
        assert!(evaluate_sensor_placement(&root, &inconsistent_bytes, source_inp, &plan_bytes)
            .unwrap_err()
            .contains("does not match the hash-verified report"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsafe_report_paths_are_rejected() {
        assert!(safe_relative_artifact("../report.rpt").is_err());
        assert!(safe_relative_artifact("/tmp/report.rpt").is_err());
        assert!(safe_relative_artifact("scenario\\report.rpt").is_err());
    }
}
