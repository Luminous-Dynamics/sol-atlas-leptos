// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Compare one-baseline compact candidate batches with repeated full-trace comparisons.
//! Run: WATER_ENERGY_STRESS_BATCH_BENCH_SAMPLES=3 cargo bench --bench water_energy_stress_intervention_batch.

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
    MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_OVERRIDES,
    MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_SERVICE_STEP_ASSESSMENTS,
    WATER_ENERGY_STRESS_INTERVENTION_BATCH_SCHEMA_VERSION,
    WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION, WaterEnergyStressInterventionBatchCandidate,
    WaterEnergyStressInterventionBatchRequest, WaterEnergyStressInterventionCandidateSummary,
    WaterEnergyStressInterventionComparisonRequest, compare_water_energy_stress_intervention_batch,
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
    service_cells: usize,
    intervals_per_service: usize,
    declared_failure_scenarios: usize,
    candidates_per_batch: usize,
    assessments_per_sweep: usize,
    expected_assessments_batch: usize,
    expected_assessments_repeated_full_trace: usize,
    estimated_assessments_avoided_by_baseline_reuse: usize,
    baseline_sweep_evaluations_batch: usize,
    baseline_sweep_evaluations_repeated: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    compact_batch_p50_micros: u128,
    compact_batch_p95_micros: u128,
    compact_batch_max_micros: u128,
    repeated_full_trace_p50_micros: u128,
    repeated_full_trace_p95_micros: u128,
    repeated_full_trace_max_micros: u128,
    compact_result_bytes: usize,
    sum_of_full_result_bytes: usize,
    compact_result_bytes_per_candidate: usize,
    proposed_batch_p95_target_micros: u128,
    proposed_batch_p95_target_met: bool,
    proposed_repeated_p95_target_micros: u128,
    proposed_repeated_p95_target_met: bool,
    compact_summaries_match_full_comparisons: bool,
    all_candidates_evaluated: bool,
    protected_floor_regression_count: usize,
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
            storage_capacity: measurement(&tank, storage.max(200.0), WATER_VOLUME_UNIT),
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

fn build_batch(
    candidates: usize,
    service_cells: usize,
    intervals: usize,
) -> WaterEnergyStressInterventionBatchRequest {
    let baseline = build_sweep("batch-baseline", intervals, service_cells, 0.0);
    let candidates = (0..candidates)
        .map(|index| WaterEnergyStressInterventionBatchCandidate {
            candidate_id: format!("storage-candidate-{index:03}"),
            experiment: build_sweep(
                &format!("storage-experiment-{index:03}"),
                intervals,
                service_cells,
                25.0 + index as f64,
            ),
        })
        .collect();
    WaterEnergyStressInterventionBatchRequest {
        schema_version: WATER_ENERGY_STRESS_INTERVENTION_BATCH_SCHEMA_VERSION.to_owned(),
        id: format!("batch-{service_cells}-{intervals}"),
        baseline,
        candidates,
        max_total_service_step_assessments:
            MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_SERVICE_STEP_ASSESSMENTS,
        max_total_declared_failure_overrides: MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_OVERRIDES,
    }
}

fn compare_summary(
    summary: &WaterEnergyStressInterventionCandidateSummary,
    full: &sol_atlas_core::service_water_energy_stress_intervention::WaterEnergyStressInterventionComparisonAssessment,
) {
    assert_eq!(summary.effect, full.effect);
    assert_eq!(
        summary.possible_protected_floor_regression,
        full.possible_protected_floor_regression
    );
    assert_eq!(summary.compared_case_count, full.compared_case_count);
    assert_eq!(
        summary.worst_case_unmet_demand_reduction_lower_bound_m3,
        full.worst_case_unmet_demand_reduction_lower_bound_m3
    );
    assert_eq!(
        summary.worst_case_unmet_demand_reduction_upper_bound_m3,
        full.worst_case_unmet_demand_reduction_upper_bound_m3
    );
    assert_eq!(
        summary.worst_case_customer_delivery_delta_lower_bound_m3,
        full.worst_case_customer_delivery_delta_lower_bound_m3
    );
    assert_eq!(
        summary.worst_case_customer_delivery_delta_upper_bound_m3,
        full.worst_case_customer_delivery_delta_upper_bound_m3
    );
    assert_eq!(
        summary.worst_case_floor_compliance_delta_lower_bound,
        full.worst_case_floor_compliance_delta_lower_bound
    );
    assert_eq!(
        summary.worst_case_floor_compliance_delta_upper_bound,
        full.worst_case_floor_compliance_delta_upper_bound
    );
    assert_eq!(
        summary.baseline_supporting_evidence_ids,
        full.baseline_supporting_evidence_ids
    );
    assert_eq!(
        summary.candidate_supporting_evidence_ids,
        full.candidate_supporting_evidence_ids
    );
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
    if let Ok(sha) = env::var("GITHUB_SHA").or_else(|_| env::var("GIT_COMMIT")) {
        return Some(sha);
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
        .map(|value| value.trim().to_owned())
}

fn compiler_version() -> Option<String> {
    let output = Command::new("rustc").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn peak_rss_bytes() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    let kibibytes = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    kibibytes.checked_mul(1024)
}

fn run(
    candidate_count: usize,
    cells: usize,
    intervals: usize,
    samples: usize,
    warmup: usize,
    batch_target_ms: u128,
    legacy_target_ms: u128,
) -> Report {
    let input = build_batch(candidate_count, cells, intervals);
    let reference = compare_water_energy_stress_intervention_batch(&input, &[])
        .expect("synthetic batch must satisfy all declared budgets and pairing rules");
    let reference_bytes = serde_json::to_vec(&reference).expect("batch reference must serialize");
    let steps_per_sweep = cells * intervals * (intervals + 1);
    assert_eq!(reference.baseline_sweep_evaluation_count, 1);
    assert_eq!(reference.evaluated_candidate_count, candidate_count);
    assert_eq!(
        reference.total_service_step_assessments,
        steps_per_sweep * (candidate_count + 1)
    );

    let mut matching = true;
    let mut legacy_bytes_per_run = 0usize;
    for (index, candidate) in input.candidates.iter().enumerate() {
        let pair = WaterEnergyStressInterventionComparisonRequest {
            schema_version: WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION.to_owned(),
            id: format!("legacy-compare-{index}"),
            baseline: input.baseline.clone(),
            candidate: candidate.experiment.clone(),
        };
        let full = compare_water_energy_stress_interventions(&pair, &[])
            .expect("legacy full-trace comparison must be valid");
        compare_summary(&reference.candidates[index], &full);
        legacy_bytes_per_run += serde_json::to_vec(&full)
            .expect("full result must serialize")
            .len();
    }

    for _ in 0..warmup {
        let output = compare_water_energy_stress_intervention_batch(&input, &[])
            .expect("batch warmup must succeed");
        assert_eq!(
            serde_json::to_vec(&output).expect("warmup serialize"),
            reference_bytes
        );
    }

    let mut batch_times = Vec::with_capacity(samples);
    let mut legacy_times = Vec::with_capacity(samples);
    let mut compact_bytes = 0usize;
    let mut full_bytes = 0usize;
    let mut floor_regression_count = 0usize;
    for _ in 0..samples {
        let started = Instant::now();
        let output = compare_water_energy_stress_intervention_batch(&input, &[])
            .expect("batch run must succeed");
        batch_times.push(started.elapsed());
        let bytes = serde_json::to_vec(&output).expect("batch result must serialize");
        assert_eq!(bytes, reference_bytes, "batch output must be deterministic");
        assert_eq!(output.baseline_sweep_evaluation_count, 1);
        assert_eq!(output.candidates.len(), candidate_count);
        compact_bytes = bytes.len();
        floor_regression_count = output
            .candidates
            .iter()
            .filter(|c| c.possible_protected_floor_regression)
            .count();

        let legacy_started = Instant::now();
        let mut sample_legacy_bytes = 0usize;
        for (index, candidate) in input.candidates.iter().enumerate() {
            let pair = WaterEnergyStressInterventionComparisonRequest {
                schema_version: WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION.to_owned(),
                id: format!("legacy-compare-{index}"),
                baseline: input.baseline.clone(),
                candidate: candidate.experiment.clone(),
            };
            let full = compare_water_energy_stress_interventions(&pair, &[])
                .expect("legacy full-trace comparison must be valid");
            compare_summary(&output.candidates[index], &full);
            sample_legacy_bytes += serde_json::to_vec(&full)
                .expect("legacy result must serialize")
                .len();
        }
        legacy_times.push(legacy_started.elapsed());
        matching &= sample_legacy_bytes == legacy_bytes_per_run;
        full_bytes = sample_legacy_bytes;
    }

    let batch_p50 = percentile(&batch_times, 50);
    let batch_p95 = percentile(&batch_times, 95);
    let legacy_p50 = percentile(&legacy_times, 50);
    let legacy_p95 = percentile(&legacy_times, 95);
    let batch_max = batch_times.iter().copied().max().unwrap_or_default();
    let legacy_max = legacy_times.iter().copied().max().unwrap_or_default();
    let expected_batch = steps_per_sweep * (candidate_count + 1);
    let expected_legacy = steps_per_sweep * candidate_count * 2;
    let saved = expected_legacy.saturating_sub(expected_batch);

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
        rss_measurement_scope: "process VmHWM since process start; compact and full-trace workloads share one benchmark process",
        workload: "one-baseline compact batch versus repeated full-trace per-candidate comparisons",
        service_cells: cells,
        intervals_per_service: intervals,
        declared_failure_scenarios: intervals,
        candidates_per_batch: candidate_count,
        assessments_per_sweep: steps_per_sweep,
        expected_assessments_batch: expected_batch,
        expected_assessments_repeated_full_trace: expected_legacy,
        estimated_assessments_avoided_by_baseline_reuse: saved,
        baseline_sweep_evaluations_batch: 1,
        baseline_sweep_evaluations_repeated: candidate_count,
        warmup_iterations: warmup,
        measured_iterations: samples,
        compact_batch_p50_micros: batch_p50.as_micros(),
        compact_batch_p95_micros: batch_p95.as_micros(),
        compact_batch_max_micros: batch_max.as_micros(),
        repeated_full_trace_p50_micros: legacy_p50.as_micros(),
        repeated_full_trace_p95_micros: legacy_p95.as_micros(),
        repeated_full_trace_max_micros: legacy_max.as_micros(),
        compact_result_bytes: compact_bytes,
        sum_of_full_result_bytes: full_bytes,
        compact_result_bytes_per_candidate: compact_bytes / candidate_count,
        proposed_batch_p95_target_micros: batch_target_ms * 1_000,
        proposed_batch_p95_target_met: batch_p95.as_micros() <= batch_target_ms * 1_000,
        proposed_repeated_p95_target_micros: legacy_target_ms * 1_000,
        proposed_repeated_p95_target_met: legacy_p95.as_micros() <= legacy_target_ms * 1_000,
        compact_summaries_match_full_comparisons: matching,
        all_candidates_evaluated: reference.evaluated_candidate_count == candidate_count,
        protected_floor_regression_count: floor_regression_count,
    }
}

fn main() {
    let samples = env::var("WATER_ENERGY_STRESS_BATCH_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let warmup = env::var("WATER_ENERGY_STRESS_BATCH_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let reports = [
        run(10, 100, 10, samples, warmup, 1_500, 3_000),
        run(30, 100, 10, samples, warmup, 4_000, 8_000),
        run(80, 100, 10, samples, warmup, 12_000, 24_000),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&reports).expect("reports must serialize")
    );
}
