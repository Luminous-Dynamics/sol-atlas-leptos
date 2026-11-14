// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::utility_reconstruction_metrics::{
    score_asset_association, AssetAssociationEvaluation, AssetAssociationMetrics,
};
use std::env;
use std::fs;

const FIXTURE: &[u8] = include_bytes!(
    "../../research/earth-observability/fixtures/asset-association-evaluation-synthetic-v0.1.json"
);
const EXPECTED_METRICS: &str = include_str!(
    "../../research/earth-observability/baselines/asset-association-baseline-v0.1.json"
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetAssociationReceipt {
    benchmark_id: String,
    evaluator_version: String,
    input_sha256: String,
    evaluation_id: String,
    scope_id: String,
    status: String,
    metrics: AssetAssociationMetrics,
    claim_boundary: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_receipt(input: &[u8]) -> Result<AssetAssociationReceipt, String> {
    let evaluation: AssetAssociationEvaluation =
        serde_json::from_slice(input).map_err(|error| format!("input JSON: {error}"))?;
    let metrics = score_asset_association(&evaluation)
        .map_err(|errors| format!("invalid evaluation: {errors:?}"))?;

    Ok(AssetAssociationReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-ASSET-ASSOCIATION-001".to_string(),
        evaluator_version: "0.1.0".to_string(),
        input_sha256: sha256_hex(input),
        evaluation_id: evaluation.evaluation_id,
        scope_id: evaluation.scope_id,
        status: "synthetic_asset_association_scoring_not_physical_qualification".to_string(),
        metrics,
        claim_boundary: "Scored only eligible reference-event associations. Structural and clustering metrics do not establish physical geometry, connected topology, calibrated probabilities, utility ownership, survey certification, or excavation clearance.".to_string(),
    })
}

fn check_pinned_fixture() -> Result<(), String> {
    let receipt = build_receipt(FIXTURE)?;
    let expected: AssetAssociationMetrics = serde_json::from_str(EXPECTED_METRICS)
        .map_err(|error| format!("pinned metrics JSON: {error}"))?;
    if receipt.metrics != expected {
        return Err(format!(
            "pinned asset association metrics differ\nexpected: {expected:#?}\nobserved: {:#?}",
            receipt.metrics
        ));
    }
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    let input_path = match args.next() {
        Some(value) if value != "--check-pinned" => value,
        Some(_) => {
            match check_pinned_fixture() {
                Ok(()) => {
                    println!("PINNED_ASSET_ASSOCIATION_BASELINE_PASS");
                    return;
                }
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("usage: utility_reconstruction_eval <evaluation.json> | --check-pinned");
            std::process::exit(2);
        }
    };

    if args.next().is_some() {
        eprintln!("unexpected extra arguments");
        std::process::exit(2);
    }

    let input = match fs::read(&input_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {input_path}: {error}");
            std::process::exit(1);
        }
    };
    let receipt = match build_receipt(&input) {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).expect("receipt serialization")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytic_asset_fixture_emits_source_bound_receipt() {
        let receipt = build_receipt(FIXTURE).expect("fixture scores");
        assert_eq!(
            receipt.benchmark_id,
            "EARTH-OBSERVABILITY-ASSET-ASSOCIATION-001"
        );
        assert_eq!(receipt.input_sha256, sha256_hex(FIXTURE));
        assert_eq!(receipt.metrics.evaluated_events, 4);
        assert_eq!(receipt.metrics.excluded_owner_record_only_events, 1);
        assert_eq!(receipt.metrics.b_cubed_f1, 1.0);
    }

    #[test]
    fn pinned_asset_association_baseline_is_reproducible() {
        check_pinned_fixture().expect("pinned asset baseline matches");
    }

    #[test]
    fn input_byte_change_changes_receipt_digest() {
        let original = build_receipt(FIXTURE).expect("fixture scores");
        let mut changed = FIXTURE.to_vec();
        changed.push(b' ');
        let changed_receipt = build_receipt(&changed).expect("trailing whitespace parses");
        assert_ne!(original.input_sha256, changed_receipt.input_sha256);
    }
}
