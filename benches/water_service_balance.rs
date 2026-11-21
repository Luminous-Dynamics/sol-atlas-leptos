// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic serial water-service balance benchmark.
//! Run with WATER_BENCH_SAMPLES=100 cargo bench --bench water_service_balance.

use serde::Serialize;
use sol_atlas_core::dependency_logic::ScenarioBasis;
use sol_atlas_core::service_function::FunctionMeasurement;
use sol_atlas_core::service_water::{
    WATER_SERVICE_SCHEMA_VERSION, WaterServiceScenario, WaterServiceState, assess_water_services,
};
use sol_atlas_core::spatial_economic::Quantity;
use std::env;
use std::time::{Duration, Instant};

#[derive(Serialize)]
struct Report {
    workload: &'static str,
    services: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    result_count: usize,
    degraded_count: usize,
    serialized_result_bytes: usize,
    mass_balance_max_abs_residual: f64,
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
        rationale: "Synthetic benchmark assumption; not infrastructure data".into(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn scenario(index: usize) -> WaterServiceScenario {
    let id = format!("water-{index:05}");
    let source = format!("{id}-source");
    let pump = format!("{id}-pump");
    let treatment = format!("{id}-treatment");
    let distribution = format!("{id}-distribution");
    let tank = format!("{id}-tank");
    let service = format!("{id}-service");
    WaterServiceScenario {
        schema_version: WATER_SERVICE_SCHEMA_VERSION.into(),
        id,
        as_of: "2026-10-16T09:00:00Z".into(),
        service_entity_id: service.clone(),
        step_hours: 1.0,
        source_flow_rate: measurement(source, 120.0, "m3/h"),
        pump_capacity: measurement(pump, 110.0, "m3/h"),
        treatment_capacity: measurement(treatment, 100.0, "m3/h"),
        distribution_capacity: measurement(distribution.clone(), 90.0, "m3/h"),
        initial_storage: measurement(tank.clone(), 100.0, "m3"),
        storage_capacity: measurement(tank, 500.0, "m3"),
        demand_rate: measurement(service, 100.0, "m3/h"),
        distribution_loss_fraction: measurement(distribution, 0.05, "1"),
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
    let warmup = env::var("WATER_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);
    let samples = env::var("WATER_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(30)
        .max(1);

    for count in [100usize, 1_000, 10_000] {
        let scenarios: Vec<WaterServiceScenario> = (0..count).map(scenario).collect();
        let reference = assess_water_services(&scenarios, &[])
            .expect("deterministic synthetic scenarios must assess");
        assert_eq!(reference.len(), count);
        assert!(
            reference
                .iter()
                .all(|item| item.state == WaterServiceState::Degraded)
        );
        assert!(reference.iter().all(|item| {
            item.customer_delivery_capacity_lower_bound
                <= item.customer_delivery_capacity_upper_bound
                && item.conservative_path.mass_balance_residual.abs() <= 1e-9
                && item.optimistic_path.mass_balance_residual.abs() <= 1e-9
                && item.conservative_path.ending_storage_volume <= 500.0
                && item.optimistic_path.ending_storage_volume <= 500.0
                && (item.conservative_path.available_water_volume
                    - item.conservative_path.customer_delivered_volume
                    - item.conservative_path.distribution_loss_volume
                    - item.conservative_path.ending_storage_volume
                    - item.conservative_path.spilled_volume)
                    .abs()
                    <= 1e-9
                && (item.optimistic_path.available_water_volume
                    - item.optimistic_path.customer_delivered_volume
                    - item.optimistic_path.distribution_loss_volume
                    - item.optimistic_path.ending_storage_volume
                    - item.optimistic_path.spilled_volume)
                    .abs()
                    <= 1e-9
        }));
        let reference_bytes = serde_json::to_vec(&reference)
            .expect("reference assessment serialization must succeed");

        for _ in 0..warmup {
            let warm =
                assess_water_services(&scenarios, &[]).expect("warmup assessment must succeed");
            assert_eq!(
                serde_json::to_vec(&warm).expect("warmup serialization must succeed"),
                reference_bytes
            );
        }

        let mut timings = Vec::with_capacity(samples);
        for _ in 0..samples {
            let start = Instant::now();
            let result =
                assess_water_services(&scenarios, &[]).expect("measured assessment must succeed");
            timings.push(start.elapsed());
            let bytes = serde_json::to_vec(&result).expect("result serialization must succeed");
            assert_eq!(bytes, reference_bytes, "output must remain deterministic");
        }

        let report = Report {
            workload: "synthetic_serial_water_service_balance",
            services: count,
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
            degraded_count: reference
                .iter()
                .filter(|item| item.state == WaterServiceState::Degraded)
                .count(),
            serialized_result_bytes: reference_bytes.len(),
            mass_balance_max_abs_residual: reference
                .iter()
                .flat_map(|item| {
                    [
                        item.conservative_path.mass_balance_residual.abs(),
                        item.optimistic_path.mass_balance_residual.abs(),
                    ]
                })
                .fold(0.0, f64::max),
        };
        println!(
            "{}",
            serde_json::to_string(&report).expect("benchmark report serializes")
        );
    }
}
