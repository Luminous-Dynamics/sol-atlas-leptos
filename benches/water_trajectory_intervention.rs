// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic benchmark for paired water-service trajectory comparison.
//! Run: WATER_INTERVENTION_BENCH_SAMPLES=30 cargo bench --bench water_trajectory_intervention.

use serde::Serialize;
use sol_atlas_core::dependency_logic::ScenarioBasis;
use sol_atlas_core::service_function::FunctionMeasurement;
use sol_atlas_core::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
use sol_atlas_core::service_water_trajectory::{
    WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectory, WaterServiceTrajectoryStep,
};
use sol_atlas_core::service_water_trajectory_intervention::{
    WATER_SERVICE_TRAJECTORY_INTERVENTION_SCHEMA_VERSION,
    WaterServiceTrajectoryInterventionRequest, WaterTrajectoryInterventionEffect,
    WaterTrajectoryRecoveryComparison, compare_water_service_trajectory_interventions,
};
use sol_atlas_core::spatial_economic::Quantity;
use std::env;
use std::time::{Duration, Instant};

#[derive(Serialize)]
struct Report {
    workload: &'static str,
    comparison_pairs: usize,
    intervals_per_pair: usize,
    total_paired_intervals: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    result_count: usize,
    robust_improvement_count: usize,
    unmet_demand_reduction_lower_bound_total: f64,
    unmet_demand_reduction_upper_bound_total: f64,
    serialized_result_bytes: usize,
}

fn measurement(entity_id: String, value: f64, unit: &str) -> FunctionMeasurement {
    FunctionMeasurement {
        entity_id,
        quantity: Quantity {
            value,
            unit: unit.into(),
            uncertainty_abs: None,
        },
        basis: ScenarioBasis::Assumed,
        rationale: "Synthetic benchmark assumptions, not observed infrastructure facts.".into(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn trajectory(prefix: String, service_entity: String, outage_flow: f64) -> WaterServiceTrajectory {
    let tank = format!("{prefix}-tank");
    let distribution = format!("{prefix}-distribution");
    let source = format!("{prefix}-source");
    let pump = format!("{prefix}-pump");
    let treatment = format!("{prefix}-treatment");
    let steps = (0..10)
        .map(|hour| {
            let source_flow = if hour == 5 || hour == 6 {
                outage_flow
            } else {
                100.0
            };
            WaterServiceTrajectoryStep {
                id: format!("step-{hour:02}"),
                as_of: format!("2026-10-16T{hour:02}:00:00Z"),
                step_hours: 1.0,
                source_flow_rate: measurement(source.clone(), source_flow, WATER_RATE_UNIT),
                pump_capacity: measurement(pump.clone(), 100.0, WATER_RATE_UNIT),
                treatment_capacity: measurement(treatment.clone(), 100.0, WATER_RATE_UNIT),
                distribution_capacity: measurement(distribution.clone(), 120.0, WATER_RATE_UNIT),
                demand_rate: measurement(service_entity.clone(), 100.0, WATER_RATE_UNIT),
                distribution_loss_fraction: measurement(
                    distribution.clone(),
                    0.0,
                    DIMENSIONLESS_UNIT,
                ),
            }
        })
        .collect();
    WaterServiceTrajectory {
        schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.into(),
        id: prefix,
        service_entity_id: service_entity,
        initial_storage: measurement(tank.clone(), 50.0, WATER_VOLUME_UNIT),
        storage_capacity: measurement(tank, 150.0, WATER_VOLUME_UNIT),
        steps,
    }
}

fn request(index: usize) -> WaterServiceTrajectoryInterventionRequest {
    let suffix = format!("{index:05}");
    let service = format!("service-{suffix}");
    let baseline = trajectory(format!("baseline-{suffix}"), service.clone(), 0.0);
    let candidate = trajectory(format!("candidate-{suffix}"), service, 50.0);
    WaterServiceTrajectoryInterventionRequest {
        schema_version: WATER_SERVICE_TRAJECTORY_INTERVENTION_SCHEMA_VERSION.into(),
        id: format!("comparison-{suffix}"),
        rationale: "Synthetic backup-source assumption; no causal field effect is claimed.".into(),
        baseline,
        candidate,
    }
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = (sorted.len() * percentile)
        .div_ceil(100)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[index]
}

fn main() {
    let warmup = env::var("WATER_INTERVENTION_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3);
    let samples = env::var("WATER_INTERVENTION_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(30)
        .max(1);

    for pair_count in [100usize, 1_000, 10_000] {
        let requests: Vec<WaterServiceTrajectoryInterventionRequest> =
            (0..pair_count).map(request).collect();
        let reference = compare_water_service_trajectory_interventions(&requests, &[])
            .expect("paired synthetic interventions must assess");
        assert_eq!(reference.len(), pair_count);
        assert!(reference.iter().all(|item| {
            item.effect == WaterTrajectoryInterventionEffect::RobustImprovement
                && item.unmet_demand_reduction_lower_bound == 100.0
                && item.unmet_demand_reduction_upper_bound == 100.0
                && item.recovery_comparison == WaterTrajectoryRecoveryComparison::BothSameSample
                && item.paired_interval_count == 10
                && item.steps.iter().all(|step| {
                    step.baseline_supporting_evidence_ids.is_empty()
                        && step.candidate_supporting_evidence_ids.is_empty()
                })
        }));
        let reference_bytes = serde_json::to_vec(&reference).expect("reference serialization");
        for _ in 0..warmup {
            let warm = compare_water_service_trajectory_interventions(&requests, &[])
                .expect("warmup comparison must succeed");
            assert_eq!(
                serde_json::to_vec(&warm).expect("warmup serialization"),
                reference_bytes
            );
        }
        let mut timings = Vec::with_capacity(samples);
        for _ in 0..samples {
            let start = Instant::now();
            let measured = compare_water_service_trajectory_interventions(&requests, &[])
                .expect("measured comparison must succeed");
            timings.push(start.elapsed());
            assert_eq!(
                serde_json::to_vec(&measured).expect("measured serialization"),
                reference_bytes
            );
        }
        let report = Report {
            workload: "synthetic_water_trajectory_intervention",
            comparison_pairs: pair_count,
            intervals_per_pair: 10,
            total_paired_intervals: pair_count * 10,
            warmup_iterations: warmup,
            measured_iterations: samples,
            p50_micros: percentile(&timings, 50).as_micros(),
            p95_micros: percentile(&timings, 95).as_micros(),
            max_micros: timings
                .iter()
                .copied()
                .max()
                .unwrap_or_default()
                .as_micros(),
            result_count: reference.len(),
            robust_improvement_count: reference
                .iter()
                .filter(|item| item.effect == WaterTrajectoryInterventionEffect::RobustImprovement)
                .count(),
            unmet_demand_reduction_lower_bound_total: reference
                .iter()
                .map(|item| item.unmet_demand_reduction_lower_bound)
                .sum(),
            unmet_demand_reduction_upper_bound_total: reference
                .iter()
                .map(|item| item.unmet_demand_reduction_upper_bound)
                .sum(),
            serialized_result_bytes: reference_bytes.len(),
        };
        assert_eq!(report.robust_improvement_count, pair_count);
        assert_eq!(
            report.unmet_demand_reduction_lower_bound_total,
            pair_count as f64 * 100.0
        );
        println!(
            "{}",
            serde_json::to_string(&report).expect("benchmark report serializes")
        );
    }
}
