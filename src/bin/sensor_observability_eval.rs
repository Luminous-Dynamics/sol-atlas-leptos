// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline scoring CLI for the analytic sensor-observability fixture.
//! Outputs descriptive point estimates only; it does not qualify physical
//! sensors or compute cluster-aware confidence intervals.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::sensor_observability::{
    ClusterBootstrapConfig, DetectionEvaluationMetrics, SensorObservabilityFixture,
    evaluate_blind_trials_with_cluster_bootstrap,
};
use std::env;
use std::fs;
use std::path::Path;

const DEFAULT_FIXTURE: &str =
    "research/earth-observability/fixtures/sensor-observability-analytic-v0.1.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeRequirements {
    envelope_id: String,
    sensor_id: String,
    configuration_id: String,
    calibration_id: String,
    target_class: String,
    spatial_support_id: String,
    minimum_pod_requirement: f64,
    maximum_false_alarms_per_km: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DetectionEvaluationReceipt {
    receipt_version: String,
    benchmark_id: String,
    fixture_id: String,
    fixture_version: String,
    fixture_sha256: String,
    seed: u64,
    status: String,
    geography: String,
    claim_boundary: String,
    fixture_contract_validation: String,
    json_schema_validation: String,
    requirements: EnvelopeRequirements,
    cluster_bootstrap_config: ClusterBootstrapConfig,
    metrics: DetectionEvaluationMetrics,
    physical_validation: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_receipt(bytes: &[u8]) -> Result<DetectionEvaluationReceipt, String> {
    let fixture: SensorObservabilityFixture = serde_json::from_slice(bytes)
        .map_err(|error| format!("fixture JSON parse failed: {error}"))?;
    let cluster_bootstrap_config = ClusterBootstrapConfig {
        confidence_level: 0.95,
        bootstrap_replicates: 2_000,
        // This is a preregistered pilot gate for the benchmark run, not a
        // universal statistical standard. The four-site fixture will not
        // produce inferential intervals under this threshold.
        minimum_independent_site_clusters: 10,
        seed: fixture.seed,
    };
    let metrics = evaluate_blind_trials_with_cluster_bootstrap(&fixture, cluster_bootstrap_config)
        .map_err(|errors| format!("fixture contract failed: {errors:?}"))?;

    Ok(DetectionEvaluationReceipt {
        receipt_version: "0.1.0".to_string(),
        benchmark_id: fixture.benchmark_id,
        fixture_id: fixture.fixture_id,
        fixture_version: fixture.fixture_version,
        fixture_sha256: sha256_hex(bytes),
        seed: fixture.seed,
        status: "synthetic_analytic_scoring_not_physical_evidence".to_string(),
        geography: fixture.geography,
        claim_boundary: fixture.claim_boundary,
        fixture_contract_validation: "PASS_RUST_TYPED_CONTRACT".to_string(),
        json_schema_validation: "NOT_RUN".to_string(),
        requirements: EnvelopeRequirements {
            envelope_id: fixture.envelope.envelope_id,
            sensor_id: fixture.envelope.sensor_id,
            configuration_id: fixture.envelope.configuration_id,
            calibration_id: fixture.envelope.calibration_id,
            target_class: fixture.envelope.target_class,
            spatial_support_id: fixture.envelope.spatial_support_id,
            minimum_pod_requirement: fixture.envelope.minimum_pod_requirement,
            maximum_false_alarms_per_km: fixture.envelope.maximum_false_alarms_per_km,
        },
        cluster_bootstrap_config,
        metrics,
        physical_validation: "NOT_RUN".to_string(),
    })
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} [FIXTURE.json]");
    eprintln!("       {program} --check REPORT.json [FIXTURE.json]");
    eprintln!("Default fixture: {DEFAULT_FIXTURE}");
    eprintln!("Scores blind analytic trials; no physical qualification is performed.");
}

fn main() {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "sensor_observability_eval".to_string());
    let first = args.next();

    let (check_report, fixture_path) = match first.as_deref() {
        None => (None, DEFAULT_FIXTURE.to_string()),
        Some("--help" | "-h") => {
            print_usage(&program);
            return;
        }
        Some("--check") => {
            let Some(report_path) = args.next() else {
                print_usage(&program);
                std::process::exit(64);
            };
            let fixture_path = args.next().unwrap_or_else(|| DEFAULT_FIXTURE.to_string());
            (Some(report_path), fixture_path)
        }
        Some(path) => (None, path.to_string()),
    };

    if args.next().is_some() {
        print_usage(&program);
        std::process::exit(64);
    }

    let bytes = match fs::read(Path::new(&fixture_path)) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {fixture_path}: {error}");
            std::process::exit(1);
        }
    };
    let receipt = match build_receipt(&bytes) {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    if let Some(report_path) = check_report {
        let expected_bytes = match fs::read(Path::new(&report_path)) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("cannot read {report_path}: {error}");
                std::process::exit(1);
            }
        };
        let expected: DetectionEvaluationReceipt = match serde_json::from_slice(&expected_bytes) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("receipt JSON parse failed: {error}");
                std::process::exit(1);
            }
        };
        if expected != receipt {
            eprintln!("FAIL: pinned detection receipt differs from recomputed fixture metrics");
            std::process::exit(3);
        }
        println!(
            "PASS: {report_path} matches fixture {} SHA-256 {}",
            receipt.fixture_id, receipt.fixture_sha256
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&receipt)
                .expect("detection evaluation receipt serializes")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!(
        "../../research/earth-observability/fixtures/sensor-observability-analytic-v0.1.json"
    );

    #[test]
    fn analytic_fixture_emits_pod_false_alarm_and_localization_metrics() {
        let receipt = build_receipt(FIXTURE).expect("analytic fixture passes contract");
        assert_eq!(receipt.metrics.positive_targets, 8);
        assert_eq!(receipt.metrics.true_positives, 7);
        assert_eq!(receipt.metrics.false_negatives, 1);
        assert_eq!(receipt.metrics.probability_of_detection, Some(0.875));
        assert_eq!(receipt.metrics.negative_controls, 6);
        assert_eq!(receipt.metrics.false_positives, 1);
        assert_eq!(receipt.metrics.false_alarms_per_km, Some(2.0 / 3.0));
        assert_eq!(receipt.metrics.localization_sample_count, 7);
        assert_eq!(receipt.metrics.localization_median_error_m, Some(0.25));
        assert_eq!(receipt.metrics.localization_p95_error_m, Some(0.28));
        assert_eq!(receipt.metrics.out_of_domain_blind_trials, 1);
        assert_eq!(receipt.metrics.per_site.len(), 4);
        let worst_site = receipt
            .metrics
            .per_site
            .iter()
            .find(|site| site.site_id == "blind-site-d")
            .expect("blind-site-d is represented");
        assert_eq!(worst_site.probability_of_detection, Some(0.5));
        assert_eq!(
            receipt.metrics.point_estimates_meet_declared_requirements,
            Some(false)
        );
    }

    #[test]
    fn synthetic_receipt_does_not_claim_cluster_or_physical_qualification() {
        let receipt = build_receipt(FIXTURE).expect("analytic fixture passes contract");
        assert_eq!(
            receipt.status,
            "synthetic_analytic_scoring_not_physical_evidence"
        );
        assert_eq!(receipt.json_schema_validation, "NOT_RUN");
        assert_eq!(
            receipt.metrics.clustered_confidence_interval_status,
            "INSUFFICIENT_SITE_CLUSTERS_POD_4_FAR_4_REQUIRED_10_NOT_RUN"
        );
        assert_eq!(
            receipt
                .cluster_bootstrap_config
                .minimum_independent_site_clusters,
            10
        );
        assert_eq!(receipt.metrics.pod_cluster_bootstrap_interval, None);
        assert_eq!(
            receipt.metrics.false_alarm_rate_cluster_bootstrap_interval,
            None
        );
        assert_eq!(
            receipt.metrics.qualification_status,
            "NOT_QUALIFIED_CLUSTERED_INTERVALS_NOT_RUN"
        );
        assert_eq!(receipt.physical_validation, "NOT_RUN");
    }

    #[test]
    fn fixture_digest_is_bound_to_exact_input_bytes() {
        let receipt = build_receipt(FIXTURE).expect("analytic fixture passes contract");
        assert_eq!(receipt.fixture_sha256, sha256_hex(FIXTURE));
        let mut changed = FIXTURE.to_vec();
        changed.push(b' ');
        assert_ne!(sha256_hex(&changed), receipt.fixture_sha256);
    }

    #[test]
    fn contract_rejects_sensor_configuration_mismatch() {
        let mut fixture: SensorObservabilityFixture =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        fixture.trials[1].configuration_id = "unqualified-antenna".to_string();
        let changed = serde_json::to_vec(&fixture).expect("fixture serializes");
        assert!(
            build_receipt(&changed)
                .unwrap_err()
                .contains("WrongSensorOrConfiguration")
        );
    }
}
