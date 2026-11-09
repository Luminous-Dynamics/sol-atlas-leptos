// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared-resource allocation scaling benchmark with conservative synthetic oracles.
//! Run: SHARED_RESOURCE_ALLOCATION_BENCH_SAMPLES=20 cargo bench --bench shared_resource_allocation.

use serde::Serialize;
use sol_atlas_core::dependency_logic::ScenarioBasis;
use sol_atlas_core::service_function::FunctionMeasurement;
use sol_atlas_core::service_shared_resource_allocation::{
    MAX_SHARED_RESOURCE_WORK_UNITS, SHARED_RESOURCE_ALLOCATION_SCHEMA_VERSION,
    SharedResourceAllocationPolicy, SharedResourceAllocationRequest, SharedResourceCapacity,
    SharedResourceServiceDemand, SharedResourceUsage, assess_shared_resource_allocation,
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
    service_count: usize,
    shared_resource_count: usize,
    usage_constraints: usize,
    estimated_work_units: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    proposed_p95_target_micros: u128,
    proposed_p95_target_met: bool,
    expected_coverage: f64,
    observed_coverage_lower_min: f64,
    observed_coverage_lower_max: f64,
    protected_floor_compliance_lower_bound: f64,
    all_resource_capacity_constraints_hold: bool,
    deterministic_output_bytes: bool,
    output_bytes: usize,
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

fn build_request(
    service_count: usize,
    resource_count: usize,
    usages_per_service: usize,
) -> SharedResourceAllocationRequest {
    let services = (0..service_count)
        .map(|index| {
            let id = format!("service-{index:04}");
            let entity = format!("entity-{index:04}");
            SharedResourceServiceDemand {
                id: id.clone(),
                service_entity_id: entity.clone(),
                community_id: format!("community-{:02}", index % 20),
                beneficiary_weight: 100,
                delivery_rate_unit: "m3/h".to_owned(),
                demand_rate: measurement(&entity, 100.0, "m3/h"),
                essential_coverage_floor: 0.4,
            }
        })
        .collect();
    let resources = (0..resource_count)
        .map(|index| {
            let id = format!("resource-{index:03}");
            SharedResourceCapacity {
                id: id.clone(),
                capacity: measurement(&id, 10_000.0, "m3/h"),
            }
        })
        .collect();
    let usages = (0..service_count)
        .flat_map(|service_index| {
            (0..usages_per_service).map(move |slot| {
                let resource_index = (service_index * 3 + slot * 7) % resource_count;
                let id = format!("usage-{service_index:04}-{slot:02}");
                SharedResourceUsage {
                    id: id.clone(),
                    resource_id: format!("resource-{resource_index:03}"),
                    service_id: format!("service-{service_index:04}"),
                    resource_use_per_delivery_rate: measurement(&id, 1.0, "m3/h per (m3/h)"),
                }
            })
        })
        .collect();
    SharedResourceAllocationRequest {
        schema_version: SHARED_RESOURCE_ALLOCATION_SCHEMA_VERSION.to_owned(),
        id: format!("bench-shared-resource-{service_count}-{resource_count}"),
        as_of: "2026-10-16T00:00:00Z".to_owned(),
        policy: SharedResourceAllocationPolicy::EqualCoverageProgressiveFill,
        services,
        resources,
        usages,
        max_work_units: MAX_SHARED_RESOURCE_WORK_UNITS,
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
    if let Ok(value) = env::var("GITHUB_SHA").or_else(|_| env::var("GIT_COMMIT")) {
        return Some(value);
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
    line.split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()?
        .checked_mul(1024)
}

fn run(
    service_count: usize,
    resource_count: usize,
    usages_per_service: usize,
    samples: usize,
    warmup: usize,
    target_ms: u128,
    expected_coverage: f64,
) -> Report {
    let request = build_request(service_count, resource_count, usages_per_service);
    run_request(
        request,
        samples,
        warmup,
        target_ms,
        expected_coverage,
        "one-snapshot equal-coverage allocation across shared resource constraints",
    )
}

fn run_request(
    request: SharedResourceAllocationRequest,
    samples: usize,
    warmup: usize,
    target_ms: u128,
    expected_coverage: f64,
    workload: &'static str,
) -> Report {
    let service_count = request.services.len();
    let resource_count = request.resources.len();
    let usage_count = request.usages.len();
    let reference = assess_shared_resource_allocation(&request, &[])
        .expect("synthetic shared-resource benchmark input must be valid");
    let reference_bytes = serde_json::to_vec(&reference).expect("reference output must serialize");
    assert_eq!(reference.service_count, service_count);
    assert_eq!(reference.resource_count, resource_count);
    assert_eq!(reference.usage_constraint_count, usage_count);
    assert!((reference.floor_compliance_lower_bound - 1.0).abs() < 1e-9);
    assert!(reference.services.iter().all(|service| {
        (service.coverage_lower_bound - expected_coverage).abs() < 1e-9
            && (service.coverage_upper_bound - expected_coverage).abs() < 1e-9
    }));
    assert!(reference.resources.iter().all(|resource| {
        resource.conservative_allocated_capacity <= resource.capacity_lower_bound + 1e-9
            && resource.optimistic_allocated_capacity <= resource.capacity_upper_bound + 1e-9
    }));

    for _ in 0..warmup {
        let output = assess_shared_resource_allocation(&request, &[])
            .expect("benchmark warmup must be valid");
        assert_eq!(
            serde_json::to_vec(&output).expect("warmup serialization"),
            reference_bytes
        );
    }

    let mut times = Vec::with_capacity(samples);
    let mut last_bytes = Vec::new();
    let mut observed_lower_min = f64::INFINITY;
    let mut observed_lower_max = f64::NEG_INFINITY;
    for _ in 0..samples {
        let started = Instant::now();
        let output = assess_shared_resource_allocation(&request, &[])
            .expect("benchmark assessment must succeed");
        times.push(started.elapsed());
        assert_eq!(output.services.len(), service_count);
        assert_eq!(output.resources.len(), resource_count);
        for service in &output.services {
            assert!((service.coverage_lower_bound - expected_coverage).abs() < 1e-9);
            observed_lower_min = observed_lower_min.min(service.coverage_lower_bound);
            observed_lower_max = observed_lower_max.max(service.coverage_lower_bound);
        }
        assert!(output.resources.iter().all(|resource| {
            resource.conservative_allocated_capacity <= resource.capacity_lower_bound + 1e-9
                && resource.optimistic_allocated_capacity <= resource.capacity_upper_bound + 1e-9
        }));
        last_bytes = serde_json::to_vec(&output).expect("output must serialize");
        assert_eq!(
            last_bytes, reference_bytes,
            "identical inputs must serialize identically"
        );
    }

    let p50 = percentile(&times, 50);
    let p95 = percentile(&times, 95);
    let max = times.iter().copied().max().unwrap_or_default();
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
        rss_measurement_scope: "process VmHWM since process start; workload sizes share a process",
        workload,
        service_count,
        shared_resource_count: resource_count,
        usage_constraints: usage_count,
        estimated_work_units: reference.estimated_work_units,
        warmup_iterations: warmup,
        measured_iterations: samples,
        p50_micros: p50.as_micros(),
        p95_micros: p95.as_micros(),
        max_micros: max.as_micros(),
        proposed_p95_target_micros: target_ms * 1_000,
        proposed_p95_target_met: p95.as_micros() <= target_ms * 1_000,
        expected_coverage,
        observed_coverage_lower_min: observed_lower_min,
        observed_coverage_lower_max: observed_lower_max,
        protected_floor_compliance_lower_bound: reference.floor_compliance_lower_bound,
        all_resource_capacity_constraints_hold: reference.resources.iter().all(|resource| {
            resource.conservative_allocated_capacity <= resource.capacity_lower_bound + 1e-9
                && resource.optimistic_allocated_capacity <= resource.capacity_upper_bound + 1e-9
        }),
        deterministic_output_bytes: last_bytes == reference_bytes,
        output_bytes: reference_bytes.len(),
    }
}

fn run_sparse(samples: usize, warmup: usize, target_ms: u128) -> Report {
    let mut request = build_request(1_000, 50, 3);
    let mut resource_degrees = vec![0_usize; request.resources.len()];
    for usage in &request.usages {
        let resource_index = request
            .resources
            .iter()
            .position(|resource| resource.id == usage.resource_id)
            .expect("sparse usage resource must exist");
        resource_degrees[resource_index] += 1;
    }
    for (resource, degree) in request.resources.iter_mut().zip(resource_degrees) {
        // Every service demands 100 m3/h; this capacity profile makes 0.5 coverage
        // simultaneously feasible and binding for every resource in the sparse graph.
        resource.capacity.quantity.value = 50.0 * degree as f64;
    }
    run_request(
        request,
        samples,
        warmup,
        target_ms,
        0.5,
        "sparse graph: 1,000 services x 50 resources x 3 usage edges each",
    )
}
fn run_staggered(samples: usize, warmup: usize, target_ms: u128) -> Report {
    let mut request = build_request(100, 10, 1);
    for resource in &mut request.resources {
        let suffix = resource.id.rsplit('-').next().expect("resource ID suffix");
        let index = suffix.parse::<usize>().expect("numeric resource ID suffix");
        resource.capacity.quantity.value = (index + 1) as f64 * 100.0;
    }
    for service in &mut request.services {
        service.essential_coverage_floor = 0.3;
    }
    let reference = assess_shared_resource_allocation(&request, &[])
        .expect("staggered synthetic shared-resource request must be valid");
    let reference_bytes = serde_json::to_vec(&reference).expect("reference must serialize");
    assert_eq!(reference.conservative_allocation_iterations, 10);
    assert_eq!(reference.optimistic_allocation_iterations, 10);
    assert!((reference.floor_compliance_lower_bound - 0.8).abs() < 1e-9);
    assert!((reference.floor_compliance_upper_bound - 0.8).abs() < 1e-9);
    assert!(reference.resources.iter().all(|resource| {
        resource.conservative_allocated_capacity <= resource.capacity_lower_bound + 1e-9
    }));
    for usage in &request.usages {
        let capacity = request
            .resources
            .iter()
            .find(|resource| resource.id == usage.resource_id)
            .expect("usage resource exists")
            .capacity
            .quantity
            .value;
        let expected_coverage = capacity / 1_000.0;
        let service = reference
            .services
            .iter()
            .find(|service| service.service_id == usage.service_id)
            .expect("usage service exists");
        assert!((service.coverage_lower_bound - expected_coverage).abs() < 1e-9);
    }

    for _ in 0..warmup {
        let result = assess_shared_resource_allocation(&request, &[])
            .expect("staggered warmup must succeed");
        assert_eq!(
            serde_json::to_vec(&result).expect("warmup bytes"),
            reference_bytes
        );
    }
    let mut times = Vec::with_capacity(samples);
    let mut last_bytes = Vec::new();
    let mut coverage_min = f64::INFINITY;
    let mut coverage_max = f64::NEG_INFINITY;
    for _ in 0..samples {
        let started = Instant::now();
        let output = assess_shared_resource_allocation(&request, &[])
            .expect("staggered benchmark assessment must succeed");
        times.push(started.elapsed());
        assert_eq!(output.conservative_allocation_iterations, 10);
        assert_eq!(output.optimistic_allocation_iterations, 10);
        assert!((output.floor_compliance_lower_bound - 0.8).abs() < 1e-9);
        for service in &output.services {
            coverage_min = coverage_min.min(service.coverage_lower_bound);
            coverage_max = coverage_max.max(service.coverage_lower_bound);
        }
        last_bytes = serde_json::to_vec(&output).expect("staggered result must serialize");
        assert_eq!(
            last_bytes, reference_bytes,
            "staggered output must be deterministic"
        );
    }
    let p50 = percentile(&times, 50);
    let p95 = percentile(&times, 95);
    let max = times.iter().copied().max().unwrap_or_default();
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
        rss_measurement_scope: "process VmHWM since process start; workload sizes share a process",
        workload: "staggered shared-resource bottlenecks requiring ten successive freeze rounds",
        service_count: 100,
        shared_resource_count: 10,
        usage_constraints: 100,
        estimated_work_units: reference.estimated_work_units,
        warmup_iterations: warmup,
        measured_iterations: samples,
        p50_micros: p50.as_micros(),
        p95_micros: p95.as_micros(),
        max_micros: max.as_micros(),
        proposed_p95_target_micros: target_ms * 1_000,
        proposed_p95_target_met: p95.as_micros() <= target_ms * 1_000,
        expected_coverage: 0.1,
        observed_coverage_lower_min: coverage_min,
        observed_coverage_lower_max: coverage_max,
        protected_floor_compliance_lower_bound: reference.floor_compliance_lower_bound,
        all_resource_capacity_constraints_hold: reference.resources.iter().all(|resource| {
            resource.conservative_allocated_capacity <= resource.capacity_lower_bound + 1e-9
        }),
        deterministic_output_bytes: last_bytes == reference_bytes,
        output_bytes: reference_bytes.len(),
    }
}

fn main() {
    let samples = env::var("SHARED_RESOURCE_ALLOCATION_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(10)
        .max(1);
    let warmup = env::var("SHARED_RESOURCE_ALLOCATION_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2);
    let reports = [
        run(100, 10, 10, samples, warmup, 50, 1.0),
        run(500, 25, 10, samples, warmup, 500, 0.5),
        run(1_000, 50, 10, samples, warmup, 2_000, 0.5),
        run_staggered(samples, warmup, 50),
        run_sparse(samples, warmup, 1_000),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&reports).expect("reports must serialize")
    );
}
