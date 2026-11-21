// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Benchmark for explicit joint water-energy failure sweeps.
//! Run: WATER_ENERGY_STRESS_BENCH_SAMPLES=3 cargo bench --bench water_energy_stress_sweep.

use serde::Serialize;
use sol_atlas_core::dependency_logic::ScenarioBasis;
use sol_atlas_core::service_function::FunctionMeasurement;
use sol_atlas_core::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
use sol_atlas_core::service_water_energy_coupling::{
    ELECTRIC_POWER_UNIT, PUMP_SPECIFIC_ENERGY_UNIT, PowerMeasurementLocation,
    WATER_ENERGY_COUPLING_SCHEMA_VERSION, WaterEnergyCouplingRequest, WaterEnergyCouplingStepInput,
};
use sol_atlas_core::service_water_energy_stress::{
    ProtectedWaterServiceFloor, WATER_ENERGY_STRESS_SCHEMA_VERSION,
    WaterEnergyDeclaredFailureScenario, WaterEnergyFailureScenarioKind, WaterEnergyFailureTarget,
    WaterEnergyMeasurementOverride, WaterEnergyStressSweepRequest,
    assess_water_energy_stress_sweep,
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
    declared_failure_scenarios: usize,
    declared_failure_overrides: usize,
    evaluated_failure_overrides: usize,
    service_cells: usize,
    intervals_per_service: usize,
    total_service_step_assessments: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    proposed_p95_target_micros: u128,
    proposed_p95_target_met: bool,
    assessed_scenario_count: usize,
    worst_case_floor_compliance_lower_bound: f64,
    worst_case_unmet_demand_upper_bound_m3: f64,
    serialized_result_bytes: usize,
}

fn measurement(entity: &str, value: f64, unit: &str) -> FunctionMeasurement {
    FunctionMeasurement {
        entity_id: entity.to_owned(),
        quantity: Quantity {
            value,
            unit: unit.to_owned(),
            uncertainty_abs: None,
        },
        basis: ScenarioBasis::Assumed,
        rationale: "Synthetic stress benchmark assumption; not physical evidence.".to_owned(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn service_request(index: usize, intervals: usize) -> WaterEnergyCouplingRequest {
    let id = format!("cell-{index:03}");
    let service_entity = format!("community-{index:03}");
    let source = format!("{id}-source");
    let pump = format!("{id}-pump");
    let treatment = format!("{id}-treatment");
    let distribution = format!("{id}-distribution");
    let tank = format!("{id}-tank");

    let water_steps = (0..intervals)
        .map(|hour| WaterServiceTrajectoryStep {
            id: format!("step-{hour:02}"),
            as_of: format!("2026-10-16T{hour:02}:00:00Z"),
            step_hours: 1.0,
            source_flow_rate: measurement(&source, 100.0, WATER_RATE_UNIT),
            pump_capacity: measurement(&pump, 100.0, WATER_RATE_UNIT),
            treatment_capacity: measurement(&treatment, 100.0, WATER_RATE_UNIT),
            distribution_capacity: measurement(&distribution, 120.0, WATER_RATE_UNIT),
            demand_rate: measurement(&service_entity, 100.0, WATER_RATE_UNIT),
            distribution_loss_fraction: measurement(&distribution, 0.0, DIMENSIONLESS_UNIT),
        })
        .collect::<Vec<_>>();
    let energy_steps = water_steps
        .iter()
        .map(|step| WaterEnergyCouplingStepInput {
            step_id: step.id.clone(),
            as_of: step.as_of.clone(),
            operating_regime_id: format!("{pump}-regime"),
            power_measurement_location: PowerMeasurementLocation::PumpTerminals,
            available_power: measurement(&pump, 50.0, ELECTRIC_POWER_UNIT),
            specific_energy: measurement(&format!("{pump}-regime"), 0.5, PUMP_SPECIFIC_ENERGY_UNIT),
        })
        .collect();

    WaterEnergyCouplingRequest {
        schema_version: WATER_ENERGY_COUPLING_SCHEMA_VERSION.to_owned(),
        id: id.clone(),
        trajectory: WaterServiceTrajectory {
            schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
            id: format!("trajectory-{id}"),
            service_entity_id: service_entity,
            initial_storage: measurement(&tank, 0.0, WATER_VOLUME_UNIT),
            storage_capacity: measurement(&tank, 100.0, WATER_VOLUME_UNIT),
            steps: water_steps,
        },
        steps: energy_steps,
    }
}

fn build_request(
    scenarios: usize,
    service_cells: usize,
    intervals: usize,
) -> WaterEnergyStressSweepRequest {
    let baseline_services = (0..service_cells)
        .map(|index| service_request(index, intervals))
        .collect::<Vec<_>>();
    let protected_service_floors = (0..service_cells)
        .map(|index| ProtectedWaterServiceFloor {
            service_request_id: format!("cell-{index:03}"),
            community_id: format!("community-{index:03}"),
            beneficiary_weight: 1_000,
            essential_coverage_floor: 0.8,
        })
        .collect::<Vec<_>>();

    let failure_scenarios = (0..scenarios)
        .map(|index| {
            let step_index = index % intervals;
            let power_level = ((index / intervals) % 10) as f64 * 5.0;
            let step_id = format!("step-{step_index:02}");
            let overrides = (0..service_cells)
                .map(|cell| WaterEnergyMeasurementOverride {
                    service_request_id: format!("cell-{cell:03}"),
                    step_id: step_id.clone(),
                    target: WaterEnergyFailureTarget::PumpAvailablePower,
                    forced_value: power_level,
                    forced_uncertainty_abs: None,
                })
                .collect();
            WaterEnergyDeclaredFailureScenario {
                id: format!("case-{index:05}"),
                kind: WaterEnergyFailureScenarioKind::CommonCause,
                rationale: format!(
                    "Pump-terminal power derating to {power_level} kW at one shared sample."
                ),
                overrides,
            }
        })
        .collect();

    WaterEnergyStressSweepRequest {
        schema_version: WATER_ENERGY_STRESS_SCHEMA_VERSION.to_owned(),
        id: format!("benchmark-{scenarios}-{service_cells}-{intervals}"),
        baseline_services,
        failure_scenarios,
        protected_service_floors,
        max_failure_scenarios: scenarios.max(1),
        max_service_step_assessments: 1_000_000,
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

fn run(
    scenario_count: usize,
    samples: usize,
    warmup: usize,
    target_ms: u128,
    service_cells: usize,
    intervals: usize,
) -> Report {
    let input = build_request(scenario_count, service_cells, intervals);
    let reference_output = assess_water_energy_stress_sweep(&input, &[])
        .expect("synthetic declared-scenario reference must be valid");
    let reference_bytes =
        serde_json::to_vec(&reference_output).expect("reference result must serialize");
    let mut last = Some(reference_output);
    for _ in 0..warmup {
        last = Some(
            assess_water_energy_stress_sweep(&input, &[])
                .expect("synthetic declared-scenario benchmark must be valid"),
        );
    }

    let mut timings = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        let output = assess_water_energy_stress_sweep(&input, &[])
            .expect("synthetic declared-scenario benchmark must be valid");
        timings.push(started.elapsed());
        let serialized = serde_json::to_vec(&output).expect("measured result must serialize");
        if serialized != reference_bytes {
            panic!("joint-stress output differs from its untimed deterministic reference");
        }
        last = Some(output);
    }
    let output = last.expect("reference output guarantees a result");
    let expected_step_assessments = service_cells * intervals * (scenario_count + 1);
    if output.evaluated_failure_scenario_count != scenario_count
        || output.candidate_failure_scenario_count != scenario_count
        || !output.exhaustive_over_declared_scenarios
        || output.total_service_step_assessments != expected_step_assessments
        || output.cases.len() != scenario_count
        || output.declared_failure_override_count != scenario_count * service_cells
        || output.evaluated_failure_override_count != scenario_count * service_cells
        || (output.worst_case_floor_compliance_lower_bound - 0.0).abs() > 1e-9
    {
        panic!("joint-stress benchmark failed its exhaustive-count or worst-floor oracle");
    }

    let p50 = percentile(&timings, 50);
    let p95 = percentile(&timings, 95);
    let max = timings.iter().copied().max().unwrap_or_default();
    let serialized_result_bytes = reference_bytes.len();
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
        workload: "joint water-energy stress sweep with protected service floors",
        declared_failure_scenarios: scenario_count,
        declared_failure_overrides: output.declared_failure_override_count,
        evaluated_failure_overrides: output.evaluated_failure_override_count,
        service_cells,
        intervals_per_service: intervals,
        total_service_step_assessments: output.total_service_step_assessments,
        warmup_iterations: warmup,
        measured_iterations: samples,
        p50_micros: p50.as_micros(),
        p95_micros: p95.as_micros(),
        max_micros: max.as_micros(),
        proposed_p95_target_micros: target_micros,
        proposed_p95_target_met: p95.as_micros() <= target_micros,
        assessed_scenario_count: output.evaluated_failure_scenario_count,
        worst_case_floor_compliance_lower_bound: output.worst_case_floor_compliance_lower_bound,
        worst_case_unmet_demand_upper_bound_m3: output.worst_case_unmet_demand_upper_bound_m3,
        serialized_result_bytes,
    }
}

fn main() {
    let samples = env::var("WATER_ENERGY_STRESS_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let warmup = env::var("WATER_ENERGY_STRESS_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let report = [
        run(10, samples, warmup, 500, 100, 10),
        run(100, samples, warmup, 5_000, 100, 10),
        run(900, samples, warmup, 45_000, 100, 10),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("benchmark report must serialize")
    );
}
