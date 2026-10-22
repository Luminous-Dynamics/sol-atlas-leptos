// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic microbenchmark for coupled water-energy trajectory evaluation.
//! Run: WATER_ENERGY_BENCH_SAMPLES=20 cargo bench --bench water_energy_coupling.

use serde::Serialize;
use sol_atlas_core::dependency_logic::ScenarioBasis;
use sol_atlas_core::service_function::FunctionMeasurement;
use sol_atlas_core::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
use sol_atlas_core::service_water_energy_coupling::{
    ELECTRIC_POWER_UNIT, PUMP_SPECIFIC_ENERGY_UNIT, PowerMeasurementLocation,
    WATER_ENERGY_COUPLING_SCHEMA_VERSION, WaterEnergyCouplingRequest, WaterEnergyCouplingStepInput,
    assess_water_energy_coupled_trajectory,
};
use sol_atlas_core::service_water_trajectory::{
    WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectory, WaterServiceTrajectoryStep,
};
use sol_atlas_core::spatial_economic::Quantity;
use std::env;
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Serialize)]
struct Report {
    source_revision: Option<String>,
    rustc_version: Option<String>,
    ci_runner: Option<String>,
    runner_os: Option<String>,
    runner_arch: Option<String>,
    host_os: String,
    host_arch: String,
    process_peak_rss_bytes_at_report: Option<u64>,
    rss_measurement_scope: &'static str,
    workload: &'static str,
    coupled_intervals: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    proposed_p95_target_micros: u128,
    proposed_p95_target_met: bool,
    assessed_step_count: usize,
    effective_pump_flow_lower_bound_m3_per_h: f64,
    serialized_result_bytes: usize,
}

fn measurement(entity_id: &str, value: f64, unit: &str) -> FunctionMeasurement {
    FunctionMeasurement {
        entity_id: entity_id.to_owned(),
        quantity: Quantity {
            value,
            unit: unit.to_owned(),
            uncertainty_abs: None,
        },
        basis: ScenarioBasis::Assumed,
        rationale: "Synthetic benchmark input; not physical-infrastructure evidence.".to_owned(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => unreachable!("month is kept in 1..=12"),
    }
}

fn timestamp_for_hour(offset_hours: usize) -> String {
    let mut year = 2026;
    let mut month = 10;
    let mut day = 16;
    for _ in 0..(offset_hours / 24) {
        day += 1;
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:00:00Z",
        offset_hours % 24
    )
}

fn request(intervals: usize) -> WaterEnergyCouplingRequest {
    let steps = (0..intervals)
        .map(|hour| {
            let as_of = timestamp_for_hour(hour);
            WaterServiceTrajectoryStep {
                id: format!("step-{hour:05}"),
                as_of,
                step_hours: 1.0,
                source_flow_rate: measurement("source", 100.0, WATER_RATE_UNIT),
                pump_capacity: measurement("pump", 100.0, WATER_RATE_UNIT),
                treatment_capacity: measurement("treatment", 100.0, WATER_RATE_UNIT),
                distribution_capacity: measurement("distribution", 120.0, WATER_RATE_UNIT),
                demand_rate: measurement("community", 100.0, WATER_RATE_UNIT),
                distribution_loss_fraction: measurement("distribution", 0.0, DIMENSIONLESS_UNIT),
            }
        })
        .collect::<Vec<_>>();
    let coupling_steps = steps
        .iter()
        .map(|step| WaterEnergyCouplingStepInput {
            step_id: step.id.clone(),
            as_of: step.as_of.clone(),
            operating_regime_id: "pump-regime".to_owned(),
            power_measurement_location: PowerMeasurementLocation::PumpTerminals,
            available_power: measurement("pump", 40.0, ELECTRIC_POWER_UNIT),
            specific_energy: measurement("pump-regime", 0.5, PUMP_SPECIFIC_ENERGY_UNIT),
        })
        .collect();
    WaterEnergyCouplingRequest {
        schema_version: WATER_ENERGY_COUPLING_SCHEMA_VERSION.to_owned(),
        id: format!("bench-{intervals}"),
        trajectory: WaterServiceTrajectory {
            schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
            id: format!("water-bench-{intervals}"),
            service_entity_id: "community".to_owned(),
            initial_storage: measurement("tank", 0.0, WATER_VOLUME_UNIT),
            storage_capacity: measurement("tank", 100.0, WATER_VOLUME_UNIT),
            steps,
        },
        steps: coupling_steps,
    }
}

fn source_revision() -> Option<String> {
    let configured = env::var("GITHUB_SHA")
        .or_else(|_| env::var("GIT_COMMIT"))
        .ok();
    if configured.is_some() {
        return configured;
    }
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|revision| revision.trim().to_owned())
}

fn compiler_version() -> Option<String> {
    let output = Command::new("rustc").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|version| version.trim().to_owned())
}

fn peak_rss_bytes() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    let kibibytes = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    kibibytes.checked_mul(1024)
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

fn run(intervals: usize, samples: usize, warmup: usize, target_ms: u128) -> Report {
    let input = request(intervals);
    let mut last = None;
    for _ in 0..warmup {
        last = Some(
            assess_water_energy_coupled_trajectory(&input, &[])
                .expect("synthetic benchmark fixture must be valid"),
        );
    }
    let mut timings = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        last = Some(
            assess_water_energy_coupled_trajectory(&input, &[])
                .expect("synthetic benchmark fixture must be valid"),
        );
        timings.push(started.elapsed());
    }
    let output = last.expect("at least one warmup or measured run");
    let p50 = percentile(&timings, 50);
    let p95 = percentile(&timings, 95);
    let max = timings.iter().copied().max().unwrap_or_default();
    let serialized_result_bytes = serde_json::to_vec(&output)
        .expect("assessment must serialize")
        .len();
    let target_micros = target_ms * 1_000;
    Report {
        source_revision: source_revision(),
        rustc_version: compiler_version(),
        ci_runner: env::var("RUNNER_NAME")
            .or_else(|_| env::var("CI_RUNNER_NAME"))
            .ok(),
        runner_os: env::var("RUNNER_OS").ok(),
        runner_arch: env::var("RUNNER_ARCH").ok(),
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        process_peak_rss_bytes_at_report: peak_rss_bytes(),
        rss_measurement_scope: concat!(
            "process VmHWM since process start; ",
            "workloads share one benchmark process",
        ),
        workload: "bounded water-energy trajectory assessment",
        coupled_intervals: intervals,
        warmup_iterations: warmup,
        measured_iterations: samples,
        p50_micros: p50.as_micros(),
        p95_micros: p95.as_micros(),
        max_micros: max.as_micros(),
        proposed_p95_target_micros: target_micros,
        proposed_p95_target_met: p95.as_micros() <= target_micros,
        assessed_step_count: output.water_assessment.step_count,
        effective_pump_flow_lower_bound_m3_per_h: output
            .steps
            .first()
            .map(|step| step.effective_pump_capacity_lower_bound_m3_per_h)
            .unwrap_or_default(),
        serialized_result_bytes,
    }
}

fn main() {
    let samples = env::var("WATER_ENERGY_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20)
        .max(1);
    let warmup = env::var("WATER_ENERGY_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3);
    let reports = [
        run(100, samples, warmup, 20),
        run(1_000, samples, warmup, 150),
        run(10_000, samples, warmup, 1_000),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&reports).expect("benchmark report must serialize")
    );
}
