// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic, synthetic time-dependent water-service benchmark.
//! Run: WATER_TRAJECTORY_BENCH_SAMPLES=30 cargo bench --bench water_service_trajectory.

use serde::Serialize;
use sol_atlas_core::dependency_logic::ScenarioBasis;
use sol_atlas_core::service_function::FunctionMeasurement;
use sol_atlas_core::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
use sol_atlas_core::service_water_trajectory::{
    WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectory, WaterServiceTrajectoryStep,
    assess_water_service_trajectories,
};
use sol_atlas_core::spatial_economic::Quantity;
use std::env;
use std::time::{Duration, Instant};

#[derive(Serialize)]
struct Report {
    workload: &'static str,
    trajectories: usize,
    total_intervals: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    result_count: usize,
    observed_recovery_count: usize,
    unmet_demand_volume_lower_bound: f64,
    unmet_demand_volume_upper_bound: f64,
    serialized_result_bytes: usize,
    mass_balance_max_abs_residual: f64,
}

fn measurement(entity: String, value: f64, unit: &str) -> FunctionMeasurement {
    FunctionMeasurement {
        entity_id: entity,
        quantity: Quantity {
            value,
            unit: unit.to_owned(),
            uncertainty_abs: None,
        },
        basis: ScenarioBasis::Assumed,
        rationale: "Synthetic benchmark scenario, not observed infrastructure evidence.".into(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn timestamp(hour: usize) -> String {
    format!("2026-10-16T{hour:02}:00:00Z")
}

fn trajectory(index: usize) -> WaterServiceTrajectory {
    let prefix = format!("water-{index:05}");
    let service = format!("{prefix}-service");
    let tank = format!("{prefix}-tank");
    let distribution = format!("{prefix}-distribution");

    let steps = (0..10)
        .map(|hour| {
            let source_rate = if hour == 5 || hour == 6 { 0.0 } else { 100.0 };
            WaterServiceTrajectoryStep {
                id: format!("step-{hour:02}"),
                as_of: timestamp(hour),
                step_hours: 1.0,
                source_flow_rate: measurement(
                    format!("{prefix}-source"),
                    source_rate,
                    WATER_RATE_UNIT,
                ),
                pump_capacity: measurement(format!("{prefix}-pump"), 100.0, WATER_RATE_UNIT),
                treatment_capacity: measurement(
                    format!("{prefix}-treatment"),
                    100.0,
                    WATER_RATE_UNIT,
                ),
                distribution_capacity: measurement(distribution.clone(), 120.0, WATER_RATE_UNIT),
                demand_rate: measurement(service.clone(), 100.0, WATER_RATE_UNIT),
                distribution_loss_fraction: measurement(
                    distribution.clone(),
                    0.0,
                    DIMENSIONLESS_UNIT,
                ),
            }
        })
        .collect();

    WaterServiceTrajectory {
        schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
        id: prefix,
        service_entity_id: service,
        initial_storage: measurement(tank.clone(), 50.0, WATER_VOLUME_UNIT),
        storage_capacity: measurement(tank, 150.0, WATER_VOLUME_UNIT),
        steps,
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
    let warmup = env::var("WATER_TRAJECTORY_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3);
    let samples = env::var("WATER_TRAJECTORY_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(30)
        .max(1);

    // 10 intervals per trajectory gives 100, 1,000, and 10,000 total intervals.
    for trajectory_count in [10usize, 100, 1_000] {
        let inputs: Vec<WaterServiceTrajectory> = (0..trajectory_count).map(trajectory).collect();
        let reference = assess_water_service_trajectories(&inputs, &[])
            .expect("deterministic synthetic trajectories must assess");
        assert_eq!(reference.len(), trajectory_count);
        assert!(reference.iter().all(|item| item.step_count == 10));
        assert!(reference.iter().all(|item| {
            item.unmet_demand_volume_lower_bound == 150.0
                && item.unmet_demand_volume_upper_bound == 150.0
                && item.first_adequate_step_id.as_deref() == Some("step-07")
                && item.adequacy_maintained_through_last_sample
        }));
        assert!(reference.iter().all(|item| {
            item.steps.iter().all(|step| {
                step.conservative_mass_balance_residual.abs() <= 1e-9
                    && step.optimistic_mass_balance_residual.abs() <= 1e-9
                    && step.ending_storage_lower_bound <= 150.0
                    && step.ending_storage_upper_bound <= 150.0
            })
        }));
        let reference_bytes = serde_json::to_vec(&reference)
            .expect("reference trajectory serialization must succeed");

        for _ in 0..warmup {
            let warm = assess_water_service_trajectories(&inputs, &[])
                .expect("warmup trajectory assessment must succeed");
            assert_eq!(
                serde_json::to_vec(&warm).expect("warmup serialization must succeed"),
                reference_bytes
            );
        }

        let mut timings = Vec::with_capacity(samples);
        for _ in 0..samples {
            let start = Instant::now();
            let result = assess_water_service_trajectories(&inputs, &[])
                .expect("measured trajectory assessment must succeed");
            timings.push(start.elapsed());
            assert_eq!(
                serde_json::to_vec(&result).expect("result serialization must succeed"),
                reference_bytes
            );
        }

        let report = Report {
            workload: "synthetic_contiguous_water_service_trajectory",
            trajectories: trajectory_count,
            total_intervals: trajectory_count * 10,
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
            observed_recovery_count: reference
                .iter()
                .filter(|item| item.first_adequate_step_id.as_deref() == Some("step-07"))
                .count(),
            unmet_demand_volume_lower_bound: reference
                .iter()
                .map(|item| item.unmet_demand_volume_lower_bound)
                .sum(),
            unmet_demand_volume_upper_bound: reference
                .iter()
                .map(|item| item.unmet_demand_volume_upper_bound)
                .sum(),
            serialized_result_bytes: reference_bytes.len(),
            mass_balance_max_abs_residual: reference
                .iter()
                .flat_map(|item| item.steps.iter())
                .flat_map(|step| {
                    [
                        step.conservative_mass_balance_residual.abs(),
                        step.optimistic_mass_balance_residual.abs(),
                    ]
                })
                .fold(0.0, f64::max),
        };
        assert_eq!(report.observed_recovery_count, trajectory_count);
        println!(
            "{}",
            serde_json::to_string(&report).expect("benchmark report serializes")
        );
    }
}
