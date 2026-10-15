// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Semantic validation for a public water observation bundle.
//! This does not independently verify or endorse source-reported measurements.

use sol_atlas_core::water_observations::validate_public_observation_bundle;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: water-observation-validate <bundle.json>");
        return ExitCode::from(64);
    };
    if args.next().is_some() {
        eprintln!("usage: water-observation-validate <bundle.json>");
        return ExitCode::from(64);
    }

    let path = PathBuf::from(path);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", path.display());
            return ExitCode::from(1);
        }
    };
    let bundle: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(bundle) => bundle,
        Err(error) => {
            eprintln!("invalid JSON {}: {error}", path.display());
            return ExitCode::from(2);
        }
    };

    let report = validate_public_observation_bundle(&bundle);
    if report.is_valid() {
        println!(
            "OBSERVATION_SEMANTIC_PASS: all IDs and source references are consistent; source values are not independently verified."
        );
        return ExitCode::SUCCESS;
    }
    for issue in &report.issues {
        eprintln!("{} {}: {}", issue.code, issue.path, issue.message);
    }
    eprintln!("OBSERVATION_SEMANTIC_FAIL: {} issue(s).", report.issues.len());
    ExitCode::from(2)
}
