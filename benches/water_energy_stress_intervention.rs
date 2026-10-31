// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Repeated paired intervention evaluation across one fixed joint-failure suite.
//! Run: WATER_ENERGY_STRESS_INTERVENTION_BENCH_SAMPLES=3 cargo bench --bench water_energy_stress_intervention.

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
};
use sol_atlas_core::service_water_energy_stress_intervention::{
    WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION,
    WaterEnergyStressInterventionComparisonRequest, WaterEnergyStressInterventionEffect,
    compare_water_energy_stress_interventions,
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
    repeated_candidate_evaluations: usize,
    service_cells: usize,
    intervals_per_service: usize,
    declared_failure_scenarios: usize,
    service_step_assessments_per_comparison: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    proposed_p95_target_micros: u128,
    proposed_p95_target_met: bool,
    expected_effect: WaterEnergyStressInterventionEffect,
    protected_floor_regression_count: usize,
    worst_case_unmet_demand_reduction_lower_bound_m3: f64,
    result_bytes_per_comparison: usize,
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
        rationale: "Synthetic benchmark assumption; not physical evidence.".to_owned(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn timestamp_for_hour(hour: usize) -> String {
    format!("2026-10-16T{hour:02}:00:00Z")
}

fn service_request(index: usize, intervals: usize, storage: f64) -> WaterEnergyCouplingRequest {
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
            as_of: timestamp_for_hour(hour),
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
            initial_storage: measurement(&tank, storage, WATER_VOLUME_UNIT),
            storage_capacity: measurement(&tank, storage.max(100.0), WATER_VOLUME_UNIT),
            steps: water_steps,
        },
        steps: energy_steps,
    }
}

fn build_sweep(
    id: &str,
    intervals: usize,
    service_cells: usize,
    storage: f64,
) -> WaterEnergyStressSweepRequest {
    let baseline_services = (0..service_cells)
        .map(|index| service_request(index, intervals, storage))
        .collect::<Vec<_>>();
    let protected_service_floors = (0..service_cells)
        .map(|index| ProtectedWaterServiceFloor {
            service_request_id: format!("cell-{index:03}"),
            community_id: format!("community-{index:03}"),
            beneficiary_weight: 1_000,
            essential_coverage_floor: 0.8,
        })
        .collect::<Vec<_>>();
    let failure_scenarios = (0..intervals)
        .map(|hour| {
            let step_id = format!("step-{hour:02}");
            let overrides = (0..service_cells)
                .map(|cell| WaterEnergyMeasurementOverride {
                    service_request_id: format!("cell-{cell:03}"),
                    step_id: step_id.clone(),
                    target: WaterEnergyFailureTarget::PumpAvailablePower,
                    forced_value: 0.0,
                    forced_uncertainty_abs: None,
                })
                .collect();
            WaterEnergyDeclaredFailureScenario {
                id: format!("terminal-power-loss-{hour:02}"),
                kind: WaterEnergyFailureScenarioKind::CommonCause,
                rationale: format!(
                    "Synthetic common event removes pump-terminal power at sample {hour}."
                ),
                overrides,
            }
        })
        .collect();
    WaterEnergyStressSweepRequest {
        schema_version: WATER_ENERGY_STRESS_SCHEMA_VERSION.to_owned(),
        id: id.to_owned(),
        baseline_services,
        failure_scenarios,
        protected_service_floors,
        max_failure_scenarios: intervals,
        max_service_step_assessments: 1_000_000,
    }
}

fn build_request(
    intervals: usize,
    service_cells: usize,
) -> WaterEnergyStressInterventionComparisonRequest {
    WaterEnergyStressInterventionComparisonRequest {
        schema_version: WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION.to_owned(),
        id: format!("bench-paired-stress-{service_cells}-{intervals}"),
        baseline: build_sweep("baseline", intervals, service_cells, 0.0),
        candidate: build_sweep("candidate", intervals, service_cells, 200.0),
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
    repetitions: usize,
    samples: usize,
    warmup: usize,
    target_ms: u128,
    service_cells: usize,
    intervals: usize,
) -> Report {
    let input = build_request(intervals, service_cells);
    let reference = compare_water_energy_stress_interventions(&input, &[])
        .expect("synthetic paired stress scenario must be valid");
    let reference_bytes =
        serde_json::to_vec(&reference).expect("reference comparison must serialize");

    for _ in 0..warmup {
        let output = compare_water_energy_stress_interventions(&input, &[])
            .expect("synthetic paired stress scenario must be valid");
        assert_eq!(
            output.effect,
            WaterEnergyStressInterventionEffect::RobustImprovement
        );
    }

    let mut timings = Vec::with_capacity(samples);
    let mut last = None;
    for _ in 0..samples {
        let started = Instant::now();
        for _ in 0..repetitions {
            let output = compare_water_energy_stress_interventions(&input, &[])
                .expect("synthetic paired stress scenario must be valid");
            if output.effect != WaterEnergyStressInterventionEffect::RobustImprovement
                || output.possible_protected_floor_regression
                || output.worst_case_unmet_demand_reduction_lower_bound_m3 < 0.0
                || output.compared_case_count != intervals + 1
            {
                panic!("paired intervention benchmark failed its protected-floor oracle");
            }
            last = Some(output);
        }
        timings.push(started.elapsed());
    }
    let output = last.expect("at least one measured comparison");
    let serialized = serde_json::to_vec(&output).expect("result must serialize");
    if serialized != reference_bytes {
        panic!("paired stress benchmark differs from its untimed reference");
    }
    let p50 = percentile(&timings, 50);
    let p95 = percentile(&timings, 95);
    let max = timings.iter().copied().max().unwrap_or_default();

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
        workload: "paired interventions over one declared joint water-energy stress suite",
        repeated_candidate_evaluations: repetitions,
        service_cells,
        intervals_per_service: intervals,
        declared_failure_scenarios: intervals,
        service_step_assessments_per_comparison: service_cells * intervals * (intervals + 1) * 2,
        warmup_iterations: warmup,
        measured_iterations: samples,
        p50_micros: p50.as_micros(),
        p95_micros: p95.as_micros(),
        max_micros: max.as_micros(),
        proposed_p95_target_micros: target_ms * 1_000,
        proposed_p95_target_met: p95.as_micros() <= target_ms * 1_000,
        expected_effect: output.effect,
        protected_floor_regression_count: usize::from(output.possible_protected_floor_regression),
        worst_case_unmet_demand_reduction_lower_bound_m3: output
            .worst_case_unmet_demand_reduction_lower_bound_m3,
        result_bytes_per_comparison: serialized.len(),
    }
}

fn main() {
    let samples = env::var("WATER_ENERGY_STRESS_INTERVENTION_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let warmup = env::var("WATER_ENERGY_STRESS_INTERVENTION_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let reports = [
        run(10, samples, warmup, 1_000, 100, 10),
        run(100, samples, warmup, 10_000, 100, 10),
        run(500, samples, warmup, 50_000, 100, 10),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&reports).expect("benchmark report must serialize")
    );
}
