// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Paired, target-cluster bootstrap comparisons for sensor-placement evaluation.
//!
//! Inference is conditional on the supplied network and synthetic scenario corpus;
//! it is not cross-network evidence and cannot establish field performance.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

const INPUT_SCHEMA: &str = "water-sensor-placement-evaluation/v0.1";
const OUTPUT_SCHEMA: &str = "water-sensor-placement-comparison/v0.1";
const REFERENCE_STRATEGY: &str = "fixed-predeclared";
const BOOTSTRAP_REPLICATES: usize = 2_000;

#[derive(Debug, Clone, Deserialize)]
struct EvaluationEnvelope {
    schema_version: String,
    source_manifest_sha256: String,
    source_plan_sha256: String,
    source_topology_sha256: String,
    evaluation: EvaluationInput,
}

#[derive(Debug, Clone, Deserialize)]
struct EvaluationInput {
    schema_version: String,
    plan_id: String,
    run_state: String,
    sensor_budgets: Vec<usize>,
    noise_sigma_fractions: Vec<f64>,
    replicate_count: usize,
    held_out_magnitudes: Vec<f64>,
    strategy_ids: Vec<String>,
    results: Vec<StrategyResultInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct StrategyResultInput {
    strategy_id: String,
    sensor_budget: usize,
    noise_sigma_fraction: f64,
    replicate: usize,
    held_out_magnitude: f64,
    test_count: usize,
    top1_accuracy: f64,
    top3_accuracy: f64,
    mean_reciprocal_rank: f64,
    selected_sensor_node_ids: Vec<String>,
    predictions: Vec<PredictionInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct PredictionInput {
    scenario_id: String,
    actual_target: String,
    actual_rank: usize,
    top1_hit: bool,
    top3_hit: bool,
    reciprocal_rank: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct StratumKey {
    sensor_budget: usize,
    noise_bits: u64,
    replicate: usize,
    magnitude_bits: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct Delta {
    top1: f64,
    top3: f64,
    mrr: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonEnvelope {
    pub schema_version: String,
    pub source_evaluation_sha256: String,
    pub source_manifest_sha256: String,
    pub source_plan_sha256: String,
    pub source_topology_sha256: String,
    pub comparison: PairedComparison,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairedComparison {
    pub schema_version: String,
    pub run_state: String,
    pub plan_id: String,
    pub reference_strategy: String,
    pub bootstrap_method: String,
    pub confidence_level: f64,
    pub bootstrap_replicates: usize,
    pub sensor_budgets: Vec<usize>,
    pub noise_sigma_fractions: Vec<f64>,
    pub held_out_magnitudes: Vec<f64>,
    pub strategy_ids: Vec<String>,
    pub results: Vec<PairedComparisonRow>,
    pub claim_ceiling: String,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairedComparisonRow {
    pub candidate_strategy: String,
    pub reference_strategy: String,
    pub sensor_budget: usize,
    pub noise_sigma_fraction: f64,
    pub replicate: usize,
    pub held_out_magnitude: f64,
    pub test_case_count: usize,
    pub target_cluster_count: usize,
    pub mean_delta_top1: f64,
    pub ci95_low_delta_top1: f64,
    pub ci95_high_delta_top1: f64,
    pub mean_delta_top3: f64,
    pub ci95_low_delta_top3: f64,
    pub ci95_high_delta_top3: f64,
    pub mean_delta_mrr: f64,
    pub ci95_low_delta_mrr: f64,
    pub ci95_high_delta_mrr: f64,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn near(left: f64, right: f64) -> bool {
    left.is_finite() && right.is_finite() && (left - right).abs() <= 1.0e-9
}

fn key_for(row: &StrategyResultInput) -> StratumKey {
    StratumKey {
        sensor_budget: row.sensor_budget,
        noise_bits: row.noise_sigma_fraction.to_bits(),
        replicate: row.replicate,
        magnitude_bits: row.held_out_magnitude.to_bits(),
    }
}

fn validate_input(input: &EvaluationEnvelope) -> Result<(), String> {
    if input.schema_version != "water-sensor-placement-evaluation-envelope/v0.1"
        || input.evaluation.schema_version != INPUT_SCHEMA
        || input.evaluation.run_state != "Completed"
    {
        return Err("input must be a completed water-sensor-placement evaluation envelope".to_owned());
    }
    if !is_sha256(&input.source_manifest_sha256)
        || !is_sha256(&input.source_plan_sha256)
        || !is_sha256(&input.source_topology_sha256)
    {
        return Err("input envelope contains a malformed provenance digest".to_owned());
    }
    let eval = &input.evaluation;
    if eval.plan_id.is_empty()
        || eval.sensor_budgets.is_empty()
        || eval.noise_sigma_fractions.is_empty()
        || eval.held_out_magnitudes.len() < 2
        || eval.replicate_count == 0
    {
        return Err("evaluation design fields are incomplete".to_owned());
    }
    let strategy_set: BTreeSet<&str> = eval.strategy_ids.iter().map(String::as_str).collect();
    if strategy_set.len() != eval.strategy_ids.len()
        || !strategy_set.contains(REFERENCE_STRATEGY)
        || strategy_set.len() < 2
    {
        return Err("strategy_ids must be unique and include fixed-predeclared plus a comparator".to_owned());
    }
    for (name, values) in [
        ("sensor budgets", eval.sensor_budgets.iter().map(|v| *v as f64).collect::<Vec<_>>()),
        ("noise fractions", eval.noise_sigma_fractions.clone()),
        ("held-out magnitudes", eval.held_out_magnitudes.clone()),
    ] {
        let mut seen = BTreeSet::new();
        for value in values {
            if !value.is_finite() || !seen.insert(value.to_bits()) {
                return Err(format!("{name} must contain unique finite values"));
            }
        }
    }
    if eval.sensor_budgets.contains(&0)
        || eval.noise_sigma_fractions.iter().any(|value| *value < 0.0)
        || eval.held_out_magnitudes.iter().any(|value| *value <= 0.0)
    {
        return Err("budgets/noise/magnitudes are outside the supported range".to_owned());
    }
    for values in [
        eval.noise_sigma_fractions.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        eval.held_out_magnitudes.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
    ] {
        if values.is_empty() {
            return Err("empty evaluation dimension".to_owned());
        }
    }
    let expected_count = eval.strategy_ids.len()
        .checked_mul(eval.sensor_budgets.len())
        .and_then(|n| n.checked_mul(eval.noise_sigma_fractions.len()))
        .and_then(|n| n.checked_mul(eval.replicate_count))
        .and_then(|n| n.checked_mul(eval.held_out_magnitudes.len()))
        .ok_or_else(|| "expected result grid overflows usize".to_owned())?;
    if eval.results.len() != expected_count {
        return Err(format!(
            "result grid has {} rows; frozen dimensions require {expected_count}",
            eval.results.len()
        ));
    }

    let budget_set: BTreeSet<usize> = eval.sensor_budgets.iter().copied().collect();
    let noise_set: BTreeSet<u64> = eval.noise_sigma_fractions.iter().map(|v| v.to_bits()).collect();
    let magnitude_set: BTreeSet<u64> = eval.held_out_magnitudes.iter().map(|v| v.to_bits()).collect();
    let mut strata = BTreeMap::<StratumKey, BTreeMap<&str, &StrategyResultInput>>::new();

    for row in &eval.results {
        if !strategy_set.contains(row.strategy_id.as_str())
            || !budget_set.contains(&row.sensor_budget)
            || !noise_set.contains(&row.noise_sigma_fraction.to_bits())
            || row.replicate >= eval.replicate_count
            || !magnitude_set.contains(&row.held_out_magnitude.to_bits())
        {
            return Err("result row is outside the declared evaluation design".to_owned());
        }
        if row.test_count == 0 || row.predictions.len() != row.test_count {
            return Err(format!("{} has inconsistent prediction count", row.strategy_id));
        }
        if row.selected_sensor_node_ids.len() != row.sensor_budget {
            return Err(format!("{} selected-site count differs from budget", row.strategy_id));
        }
        let unique_sensors: BTreeSet<&str> =
            row.selected_sensor_node_ids.iter().map(String::as_str).collect();
        if unique_sensors.len() != row.selected_sensor_node_ids.len() {
            return Err(format!("{} selected layout contains duplicate sites", row.strategy_id));
        }
        let mut scenario_ids = HashSet::new();
        let mut correct_top1 = 0usize;
        let mut correct_top3 = 0usize;
        let mut mrr = 0.0;
        for prediction in &row.predictions {
            if prediction.scenario_id.is_empty()
                || prediction.actual_target.is_empty()
                || prediction.actual_rank == 0
                || !scenario_ids.insert(prediction.scenario_id.as_str())
            {
                return Err(format!("{} contains an invalid or duplicate scenario prediction", row.strategy_id));
            }
            if prediction.top1_hit != (prediction.actual_rank == 1)
                || prediction.top3_hit != (prediction.actual_rank <= 3)
                || !near(prediction.reciprocal_rank, 1.0 / prediction.actual_rank as f64)
            {
                return Err(format!("{} has inconsistent per-scenario rank metrics", row.strategy_id));
            }
            if prediction.top1_hit { correct_top1 += 1; }
            if prediction.top3_hit { correct_top3 += 1; }
            mrr += prediction.reciprocal_rank;
        }
        if !near(row.top1_accuracy, correct_top1 as f64 / row.test_count as f64)
            || !near(row.top3_accuracy, correct_top3 as f64 / row.test_count as f64)
            || !near(row.mean_reciprocal_rank, mrr / row.test_count as f64)
        {
            return Err(format!("{} aggregate metrics contradict per-scenario predictions", row.strategy_id));
        }

        let key = key_for(row);
        if strata.entry(key).or_default().insert(&row.strategy_id, row).is_some() {
            return Err("duplicate strategy result for the same evaluation stratum".to_owned());
        }
    }
    for (key, rows) in &strata {
        if rows.len() != strategy_set.len()
            || strategy_set.iter().any(|strategy| !rows.contains_key(strategy))
        {
            return Err(format!("incomplete paired strategy group for {key:?}"));
        }
        let reference = rows[REFERENCE_STRATEGY].predictions.iter()
            .map(|prediction| (prediction.scenario_id.as_str(), prediction.actual_target.as_str()))
            .collect::<BTreeMap<_, _>>();
        for (strategy, row) in rows {
            let targets = row.predictions.iter()
                .map(|prediction| (prediction.scenario_id.as_str(), prediction.actual_target.as_str()))
                .collect::<BTreeMap<_, _>>();
            if targets != reference {
                return Err(format!(
                    "scenario identity/target mismatch between fixed-predeclared and {strategy}"
                ));
            }
        }
    }
    Ok(())
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn average_target_deltas(clusters: &BTreeMap<String, Vec<Delta>>) -> Vec<Delta> {
    clusters.values().map(|values| {
        let count = values.len() as f64;
        Delta {
            top1: values.iter().map(|row| row.top1).sum::<f64>() / count,
            top3: values.iter().map(|row| row.top3).sum::<f64>() / count,
            mrr: values.iter().map(|row| row.mrr).sum::<f64>() / count,
        }
    }).collect()
}

fn metric_value(row: Delta, metric: usize) -> f64 {
    match metric {
        0 => row.top1,
        1 => row.top3,
        _ => row.mrr,
    }
}

fn paired_cluster_bootstrap(
    cluster_means: &[Delta],
    seed: u64,
    metric: usize,
) -> (f64, f64, f64) {
    let n = cluster_means.len();
    let observed = cluster_means.iter().map(|row| metric_value(*row, metric)).sum::<f64>() / n as f64;
    let mut rng = seed;
    let mut estimates = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut total = 0.0;
        for _ in 0..n {
            let index = (splitmix64(&mut rng) % n as u64) as usize;
            total += metric_value(cluster_means[index], metric);
        }
        estimates.push(total / n as f64);
    }
    estimates.sort_by(f64::total_cmp);
    let low = ((BOOTSTRAP_REPLICATES - 1) as f64 * 0.025).floor() as usize;
    let high = ((BOOTSTRAP_REPLICATES - 1) as f64 * 0.975).ceil() as usize;
    (observed, estimates[low], estimates[high])
}

/// Compare each declared candidate against the predeclared fixed layout with paired
/// target-cluster bootstrap intervals, stratified by budget/noise/replicate/fold.
pub fn compare_sensor_placement(input_bytes: &[u8]) -> Result<ComparisonEnvelope, String> {
    let input: EvaluationEnvelope = serde_json::from_slice(input_bytes)
        .map_err(|error| format!("invalid sensor-placement evaluation JSON: {error}"))?;
    validate_input(&input)?;

    let mut strata = BTreeMap::<StratumKey, BTreeMap<&str, &StrategyResultInput>>::new();
    for row in &input.evaluation.results {
        strata.entry(key_for(row)).or_default().insert(&row.strategy_id, row);
    }
    let digest = sha256(input_bytes);
    let digest_seed = u64::from_str_radix(&digest[..16], 16)
        .map_err(|_| "could not derive deterministic bootstrap seed from evaluation hash".to_owned())?;
    let mut results = Vec::new();

    for (stratum, rows) in strata {
        let reference = rows.get(REFERENCE_STRATEGY)
            .ok_or_else(|| "paired stratum lacks fixed-predeclared reference".to_owned())?;
        for candidate_id in &input.evaluation.strategy_ids {
            if candidate_id == REFERENCE_STRATEGY { continue; }
            let candidate = rows.get(candidate_id.as_str())
                .ok_or_else(|| format!("paired stratum lacks strategy {candidate_id}"))?;
            let reference_by_id = reference.predictions.iter()
                .map(|row| (row.scenario_id.as_str(), row))
                .collect::<BTreeMap<_, _>>();
            let candidate_by_id = candidate.predictions.iter()
                .map(|row| (row.scenario_id.as_str(), row))
                .collect::<BTreeMap<_, _>>();
            if reference_by_id.len() != candidate_by_id.len()
                || reference_by_id.keys().ne(candidate_by_id.keys())
            {
                return Err(format!("paired scenario IDs differ for {candidate_id}"));
            }
            let mut clusters = BTreeMap::<String, Vec<Delta>>::new();
            for (scenario_id, reference_prediction) in reference_by_id {
                let candidate_prediction = candidate_by_id[scenario_id];
                if reference_prediction.actual_target != candidate_prediction.actual_target {
                    return Err(format!("paired ground truth differs for scenario {scenario_id}"));
                }
                clusters.entry(reference_prediction.actual_target.clone()).or_default().push(Delta {
                    top1: f64::from(candidate_prediction.top1_hit) - f64::from(reference_prediction.top1_hit),
                    top3: f64::from(candidate_prediction.top3_hit) - f64::from(reference_prediction.top3_hit),
                    mrr: candidate_prediction.reciprocal_rank - reference_prediction.reciprocal_rank,
                });
            }
            let target_cluster_count = clusters.len();
            if target_cluster_count < 2 {
                return Err("paired bootstrap requires at least two distinct target clusters".to_owned());
            }
            let cluster_means = average_target_deltas(&clusters);
            let seed_base = digest_seed
                ^ (stratum.sensor_budget as u64).rotate_left(7)
                ^ stratum.noise_bits.rotate_left(13)
                ^ (stratum.replicate as u64).rotate_left(23)
                ^ stratum.magnitude_bits.rotate_left(31)
                ^ candidate_id.bytes().fold(0u64, |acc, byte| acc.rotate_left(5) ^ byte as u64);
            let (mean_top1, low_top1, high_top1) = paired_cluster_bootstrap(&cluster_means, seed_base ^ 0x11, 0);
            let (mean_top3, low_top3, high_top3) = paired_cluster_bootstrap(&cluster_means, seed_base ^ 0x33, 1);
            let (mean_mrr, low_mrr, high_mrr) = paired_cluster_bootstrap(&cluster_means, seed_base ^ 0x55, 2);
            results.push(PairedComparisonRow {
                candidate_strategy: candidate_id.clone(),
                reference_strategy: REFERENCE_STRATEGY.to_owned(),
                sensor_budget: stratum.sensor_budget,
                noise_sigma_fraction: f64::from_bits(stratum.noise_bits),
                replicate: stratum.replicate,
                held_out_magnitude: f64::from_bits(stratum.magnitude_bits),
                test_case_count: candidate.test_count,
                target_cluster_count,
                mean_delta_top1: mean_top1,
                ci95_low_delta_top1: low_top1,
                ci95_high_delta_top1: high_top1,
                mean_delta_top3: mean_top3,
                ci95_low_delta_top3: low_top3,
                ci95_high_delta_top3: high_top3,
                mean_delta_mrr: mean_mrr,
                ci95_low_delta_mrr: low_mrr,
                ci95_high_delta_mrr: high_mrr,
            });
        }
    }
    results.sort_by(|a, b| {
        a.sensor_budget.cmp(&b.sensor_budget)
            .then(a.noise_sigma_fraction.total_cmp(&b.noise_sigma_fraction))
            .then(a.replicate.cmp(&b.replicate))
            .then(a.held_out_magnitude.total_cmp(&b.held_out_magnitude))
            .then(a.candidate_strategy.cmp(&b.candidate_strategy))
    });

    Ok(ComparisonEnvelope {
        schema_version: "water-sensor-placement-comparison-envelope/v0.1".to_owned(),
        source_evaluation_sha256: digest,
        source_manifest_sha256: input.source_manifest_sha256,
        source_plan_sha256: input.source_plan_sha256,
        source_topology_sha256: input.source_topology_sha256,
        comparison: PairedComparison {
            schema_version: OUTPUT_SCHEMA.to_owned(),
            run_state: "Completed".to_owned(),
            plan_id: input.evaluation.plan_id,
            reference_strategy: REFERENCE_STRATEGY.to_owned(),
            bootstrap_method: "paired_target_cluster_percentile_bootstrap".to_owned(),
            confidence_level: 0.95,
            bootstrap_replicates: BOOTSTRAP_REPLICATES,
            sensor_budgets: input.evaluation.sensor_budgets,
            noise_sigma_fractions: input.evaluation.noise_sigma_fractions,
            held_out_magnitudes: input.evaluation.held_out_magnitudes,
            strategy_ids: input.evaluation.strategy_ids,
            results,
            claim_ceiling: "conditional_synthetic_within_topology_comparison".to_owned(),
            nonclaims: vec![
                "Confidence intervals describe target-to-target variation conditional on one fixed topology and scenario corpus; they are not cross-network confidence intervals.".to_owned(),
                "Noise/seed replicates and clean repeated scores are kept in separate strata; they are not treated as independent target observations.".to_owned(),
                "The reference is predeclared fixed-predeclared, not selected after looking at held-out scores.".to_owned(),
                "The bootstrap is descriptive uncertainty for paired target clusters, not evidence of real-world leak-localization performance.".to_owned(),
                "These intervals do not correct for multiple comparisons across all strategies, budgets, noise levels, replicates, and magnitudes.".to_owned(),
            ],
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn fixture() -> Vec<u8> {
        let strategies = ["fixed-predeclared", "seeded-random"];
        let mut results = Vec::new();
        for strategy in strategies {
            for magnitude in [10.0, 25.0] {
                let predictions: Vec<Value> = (0..4).map(|i| {
                    let rank = if strategy == "seeded-random" { 1 } else if i == 0 { 1 } else { 4 };
                    json!({
                        "scenario_id": format!("target-{i}-magnitude-{magnitude}"),
                        "actual_target": format!("target-{i}"),
                        "predicted_top1": format!("predicted-{i}"),
                        "actual_rank": rank,
                        "top1_hit": rank == 1,
                        "top3_hit": rank <= 3,
                        "reciprocal_rank": 1.0 / rank as f64,
                        "topological_error_hops": 0
                    })
                }).collect();
                let top1 = predictions.iter().filter(|p| p["top1_hit"] == true).count() as f64 / 4.0;
                let top3 = predictions.iter().filter(|p| p["top3_hit"] == true).count() as f64 / 4.0;
                let mrr = predictions.iter().map(|p| p["reciprocal_rank"].as_f64().unwrap()).sum::<f64>() / 4.0;
                results.push(json!({
                    "strategy_id": strategy,
                    "sensor_budget": 2,
                    "selected_sensor_node_ids": ["J1","J2"],
                    "noise_sigma_fraction": 0.0,
                    "replicate": 0,
                    "held_out_magnitude": magnitude,
                    "test_count": 4,
                    "top1_accuracy": top1,
                    "top3_accuracy": top3,
                    "mean_reciprocal_rank": mrr,
                    "mean_topological_error_hops": 0.0,
                    "p95_topological_error_hops": 0,
                    "worst_topological_error_hops": 0,
                    "disconnected_prediction_count": 0,
                    "predictions": predictions
                }));
            }
        }
        json!({
            "schema_version": "water-sensor-placement-evaluation-envelope/v0.1",
            "source_manifest_sha256": "a".repeat(64),
            "source_plan_sha256": "b".repeat(64),
            "source_topology_sha256": "c".repeat(64),
            "evaluation": {
                "schema_version": INPUT_SCHEMA,
                "plan_id": "test-plan",
                "run_state": "Completed",
                "sensor_budgets": [2],
                "noise_sigma_fractions": [0.0],
                "replicate_count": 1,
                "held_out_magnitudes": [10.0,25.0],
                "strategy_ids": strategies,
                "results": results
            }
        }).to_string().into_bytes()
    }

    #[test]
    fn paired_bootstrap_is_deterministic_and_detects_injected_gain() {
        let bytes = fixture();
        let first = compare_sensor_placement(&bytes).unwrap();
        let second = compare_sensor_placement(&bytes).unwrap();
        let first_json = serde_json::to_string(&first).unwrap();
        assert_eq!(first_json, serde_json::to_string(&second).unwrap());
        assert_eq!(first.comparison.results.len(), 2);
        assert!(first.comparison.results.iter().all(|row| row.mean_delta_top3 > 0.0));
        assert!(first.comparison.results.iter().all(|row| {
            row.ci95_low_delta_top3 <= row.mean_delta_top3
                && row.mean_delta_top3 <= row.ci95_high_delta_top3
        }));
    }

    #[test]
    fn tampered_aggregate_metrics_are_rejected() {
        let mut input: Value = serde_json::from_slice(&fixture()).unwrap();
        input["evaluation"]["results"][0]["top3_accuracy"] = json!(1.0);
        assert!(compare_sensor_placement(input.to_string().as_bytes())
            .unwrap_err()
            .contains("aggregate metrics contradict"));
    }

    #[test]
    fn target_or_scenario_mismatch_is_rejected_before_bootstrap() {
        let mut input: Value = serde_json::from_slice(&fixture()).unwrap();
        input["evaluation"]["results"][1]["predictions"][0]["actual_target"] = json!("other");
        assert!(compare_sensor_placement(input.to_string().as_bytes())
            .unwrap_err()
            .contains("scenario identity/target mismatch"));
    }
}
