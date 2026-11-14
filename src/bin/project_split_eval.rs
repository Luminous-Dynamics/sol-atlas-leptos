// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::project_split::{
    ProjectDatasetIndex, ProjectSplitManifest, build_leave_one_project_out,
    validate_project_split_manifest,
};
use std::env;
use std::fs;

const INDEX: &[u8] = include_bytes!(
    "../../research/earth-observability/fixtures/project-split-input-synthetic-v0.1.json"
);
const EXPECTED: &str = include_str!(
    "../../research/earth-observability/baselines/project-split-manifest-synthetic-v0.1.json"
);
const CALIBRATION_PROJECTS_PER_FOLD: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectSplitReceipt {
    benchmark_id: String,
    evaluator_version: String,
    input_sha256: String,
    manifest: ProjectSplitManifest,
    status: String,
    claim_boundary: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_receipt(input: &[u8]) -> Result<ProjectSplitReceipt, String> {
    let index: ProjectDatasetIndex =
        serde_json::from_slice(input).map_err(|error| format!("project index JSON: {error}"))?;
    let manifest = build_leave_one_project_out(&index, CALIBRATION_PROJECTS_PER_FOLD)
        .map_err(|errors| format!("project split invalid: {errors:?}"))?;
    validate_project_split_manifest(&index, &manifest)
        .map_err(|errors| format!("project split manifest invalid: {errors:?}"))?;

    Ok(ProjectSplitReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-PROJECT-SPLIT-001".to_string(),
        evaluator_version: "0.1.0".to_string(),
        input_sha256: sha256_hex(input),
        manifest,
        status: "synthetic_project_grouped_split_receipt_not_physical_data".to_string(),
        claim_boundary: "This receipt verifies a deterministic project-grouped partition of the supplied index. It does not verify source dataset truth, spatial overlap, near-duplicate leakage, activity-to-project lineage, statistical power, or physical model performance.".to_string(),
    })
}

fn check_pinned() -> Result<(), String> {
    let receipt = build_receipt(INDEX)?;
    let expected: ProjectSplitManifest =
        serde_json::from_str(EXPECTED).map_err(|error| format!("pinned split JSON: {error}"))?;
    if receipt.manifest != expected {
        return Err(format!(
            "pinned project split differs\nexpected: {expected:#?}\nobserved: {:#?}",
            receipt.manifest
        ));
    }
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    let input_path = match args.next() {
        Some(value) if value != "--check-pinned" => value,
        Some(_) => match check_pinned() {
            Ok(()) => {
                println!("PINNED_PROJECT_SPLIT_PASS");
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("usage: project_split_eval <project-index.json> | --check-pinned");
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
    match build_receipt(&input) {
        Ok(receipt) => println!(
            "{}",
            serde_json::to_string_pretty(&receipt).expect("receipt serialization")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ut_gold_d_project_index_builds_thirteen_disjoint_folds() {
        let receipt = build_receipt(INDEX).expect("project index builds");
        assert_eq!(receipt.manifest.project_count, 13);
        assert_eq!(receipt.manifest.folds.len(), 13);
        for fold in &receipt.manifest.folds {
            assert_eq!(fold.fit_project_ids.len(), 9);
            assert_eq!(fold.calibration_project_ids.len(), 3);
            assert!(!fold.fit_project_ids.contains(&fold.blind_project_id));
            assert!(
                !fold
                    .calibration_project_ids
                    .contains(&fold.blind_project_id)
            );
        }
    }

    #[test]
    fn project_split_is_bound_to_exact_input_bytes() {
        let receipt = build_receipt(INDEX).expect("project index builds");
        assert_eq!(receipt.input_sha256, sha256_hex(INDEX));
        let mut changed = INDEX.to_vec();
        changed.push(b' ');
        let changed_receipt = build_receipt(&changed).expect("trailing whitespace parses");
        assert_ne!(receipt.input_sha256, changed_receipt.input_sha256);
    }

    #[test]
    fn committed_project_split_matches_pinned_manifest() {
        check_pinned().expect("pinned project split matches");
    }
}
