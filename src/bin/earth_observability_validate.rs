// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline validator for the Earth Observability synthetic case manifest.
//!
//! This validates the authored case-contract only. It does not execute survey
//! simulations, validate full JSON Schema conformance, or qualify physical data.

use serde::Serialize;
use sol_atlas_core::earth_observability::{ChallengeManifest, validate_challenge_manifest};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

const DEFAULT_MANIFEST: &str =
    "research/earth-observability/earth-observability-case-manifest-v0.1.json";

#[derive(Debug, Serialize)]
struct ValidationReceipt {
    benchmark_id: String,
    manifest_version: String,
    case_count: usize,
    case_counts_by_track: BTreeMap<String, usize>,
    case_counts_by_kind: BTreeMap<String, usize>,
    contract_validation: &'static str,
    json_schema_validation: &'static str,
    synthetic_measurement_execution: &'static str,
    physical_validation: &'static str,
    errors: Vec<String>,
}

fn validate_manifest_bytes(bytes: &[u8]) -> Result<ValidationReceipt, String> {
    let manifest: ChallengeManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("manifest parse failed: {error}"))?;

    let mut tracks = BTreeMap::new();
    let mut kinds = BTreeMap::new();
    for case in &manifest.cases {
        *tracks.entry(case.track.clone()).or_insert(0) += 1;
        *kinds.entry(case.kind.clone()).or_insert(0) += 1;
    }

    let errors = validate_challenge_manifest(&manifest).err().unwrap_or_default();
    Ok(ValidationReceipt {
        benchmark_id: manifest.benchmark_id,
        manifest_version: manifest.manifest_version,
        case_count: manifest.cases.len(),
        case_counts_by_track: tracks,
        case_counts_by_kind: kinds,
        contract_validation: if errors.is_empty() { "PASS" } else { "FAIL" },
        json_schema_validation: "NOT_RUN",
        synthetic_measurement_execution: "NOT_RUN",
        physical_validation: "NOT_RUN",
        errors,
    })
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} [MANIFEST.json]");
    eprintln!("Default: {DEFAULT_MANIFEST}");
    eprintln!("Validates the Earth Observability case contract only.");
}

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "earth-observability-validate".to_string());
    let path = match (args.next(), args.next()) {
        (None, None) => DEFAULT_MANIFEST.to_string(),
        (Some(flag), None) if flag == "--help" || flag == "-h" => {
            print_usage(&program);
            return;
        }
        (Some(path), None) => path,
        _ => {
            print_usage(&program);
            std::process::exit(64);
        }
    };

    let result = fs::read(Path::new(&path))
        .map_err(|error| format!("cannot read {path}: {error}"))
        .and_then(|bytes| validate_manifest_bytes(&bytes));

    match result {
        Ok(receipt) => {
            let passed = receipt.contract_validation == "PASS";
            println!(
                "{}",
                serde_json::to_string_pretty(&receipt)
                    .expect("validation receipt must serialize")
            );
            if !passed {
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str =
        include_str!("../../research/earth-observability/earth-observability-case-manifest-v0.1.json");

    #[test]
    fn canonical_manifest_emits_contract_pass_and_18_cases() {
        let receipt = validate_manifest_bytes(MANIFEST.as_bytes()).expect("manifest parses");
        assert_eq!(receipt.contract_validation, "PASS");
        assert_eq!(receipt.case_count, 18);
        assert_eq!(receipt.json_schema_validation, "NOT_RUN");
        assert_eq!(receipt.synthetic_measurement_execution, "NOT_RUN");
        assert_eq!(receipt.physical_validation, "NOT_RUN");
    }

    #[test]
    fn contract_failure_is_reported_without_promoting_data_status() {
        let mut manifest: ChallengeManifest =
            serde_json::from_str(MANIFEST).expect("manifest parses");
        manifest.status = "synthetic_measurements_generated".to_string();
        let encoded = serde_json::to_vec(&manifest).expect("manifest serializes");
        let receipt = validate_manifest_bytes(&encoded).expect("typed manifest parses");
        assert_eq!(receipt.contract_validation, "FAIL");
        assert!(!receipt.errors.is_empty());
        assert_eq!(receipt.synthetic_measurement_execution, "NOT_RUN");
    }

    #[test]
    fn track_and_case_kind_counts_are_deterministic() {
        let receipt = validate_manifest_bytes(MANIFEST.as_bytes()).expect("manifest parses");
        assert_eq!(receipt.case_counts_by_kind.get("negative_control"), Some(&9));
        assert_eq!(receipt.case_counts_by_kind.get("adversarial"), Some(&6));
        assert_eq!(receipt.case_counts_by_kind.get("positive_control"), Some(&3));
        assert_eq!(receipt.case_counts_by_track.get("observability"), Some(&4));
    }
}
