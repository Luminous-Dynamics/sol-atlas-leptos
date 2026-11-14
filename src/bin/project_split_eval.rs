// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::project_split::{
    build_leave_one_project_out, validate_project_split_manifest, ProjectDatasetIndex,
    ProjectSplitManifest,
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
struct ProjectSplitSensitivityRow {
    calibration_projects_per_fold: usize,
    fit_projects_per_fold: usize,
    blind_projects_per_fold: usize,
    fold_count: usize,
    every_project_blind_once: bool,
    roles_disjoint_in_every_fold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectSplitSensitivityReceipt {
    benchmark_id: String,
    input_sha256: String,
    calibration_sensitivity: Vec<ProjectSplitSensitivityRow>,
    status: String,
    claim_boundary: String,
}

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

fn build_sensitivity_receipt(input: &[u8]) -> Result<ProjectSplitSensitivityReceipt, String> {
    let index: ProjectDatasetIndex =
        serde_json::from_slice(input).map_err(|error| format!("project index JSON: {error}"))?;
    let project_count = index.projects.len();
    if project_count < 4 {
        return Err("sensitivity sweep requires at least four project clusters".to_string());
    }

    let mut rows = Vec::new();
    for calibration_count in [1usize, 2, 3] {
        if calibration_count >= project_count.saturating_sub(1) {
            continue;
        }
        let manifest = build_leave_one_project_out(&index, calibration_count)
            .map_err(|errors| format!("project split invalid: {errors:?}"))?;
        validate_project_split_manifest(&index, &manifest)
            .map_err(|errors| format!("project split manifest invalid: {errors:?}"))?;

        let blind_ids: std::collections::BTreeSet<&str> = manifest
            .folds
            .iter()
            .map(|fold| fold.blind_project_id.as_str())
            .collect();
        let roles_disjoint = manifest.folds.iter().all(|fold| {
            let fit: std::collections::BTreeSet<&str> =
                fold.fit_project_ids.iter().map(String::as_str).collect();
            let calibration: std::collections::BTreeSet<&str> = fold
                .calibration_project_ids
                .iter()
                .map(String::as_str)
                .collect();
            !fit.contains(fold.blind_project_id.as_str())
                && !calibration.contains(fold.blind_project_id.as_str())
                && fit.is_disjoint(&calibration)
                && fit.len() + calibration.len() + 1 == project_count
        });

        rows.push(ProjectSplitSensitivityRow {
            calibration_projects_per_fold: calibration_count,
            fit_projects_per_fold: project_count - calibration_count - 1,
            blind_projects_per_fold: 1,
            fold_count: manifest.folds.len(),
            every_project_blind_once: blind_ids.len() == project_count,
            roles_disjoint_in_every_fold: roles_disjoint,
        });
    }

    Ok(ProjectSplitSensitivityReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-PROJECT-SPLIT-SENSITIVITY-001".to_string(),
        input_sha256: sha256_hex(input),
        calibration_sensitivity: rows,
        status: "synthetic_split_allocation_sensitivity_not_model_performance".to_string(),
        claim_boundary: "This sweep compares deterministic project-role allocation only. It does not estimate calibration quality, statistical power, field generalization or model performance.".to_string(),
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
    let first = args.next();
    if first.as_deref() == Some("--check-pinned") {
        if args.next().is_some() {
            eprintln!("--check-pinned accepts no extra arguments");
            std::process::exit(2);
        }
        match check_pinned() {
            Ok(()) => {
                println!("PINNED_PROJECT_SPLIT_PASS");
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    if first.as_deref() == Some("--sensitivity-sweep") {
        if args.next().is_some() {
            eprintln!("--sensitivity-sweep uses the committed synthetic index and accepts no extra arguments");
            std::process::exit(2);
        }
        match build_sensitivity_receipt(INDEX) {
            Ok(receipt) => {
                println!("{}", serde_json::to_string_pretty(&receipt).expect("sweep serialization"));
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    let input_path = match first {
        Some(value) => value,
        None => {
            eprintln!("usage: project_split_eval <project-index.json> | --check-pinned | --sensitivity-sweep");
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
            assert!(!fold.calibration_project_ids.contains(&fold.blind_project_id));
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
    fn calibration_cluster_sensitivity_sweep_reports_one_two_three() {
        let sweep = build_sensitivity_receipt(INDEX).unwrap();
        let values: Vec<usize> = sweep
            .calibration_sensitivity
            .iter()
            .map(|row| row.calibration_projects_per_fold)
            .collect();
        assert_eq!(values, vec![1, 2, 3]);
        assert_eq!(
            sweep.calibration_sensitivity.iter().map(|row| row.fit_projects_per_fold).collect::<Vec<_>>(),
            vec![11, 10, 9]
        );
        assert!(
            sweep
                .calibration_sensitivity
                .iter()
                .all(|row| row.every_project_blind_once && row.roles_disjoint_in_every_fold)
        );
    }

    #[test]
    fn committed_project_split_matches_pinned_manifest() {
        check_pinned().expect("pinned project split matches");
    }
}
