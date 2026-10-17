// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Validate a water-engineering bundle's cross-record identities and artifact bytes.
//! This command does not run EPANET/SWMM and cannot qualify engineering accuracy.

use sol_atlas_core::water_engineering::validate_bundle;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!(
            "usage: water-bundle-validate <bundle.json> <repository-root>\n\
             This validates semantic identity and artifact bytes only; it does not run a solver."
        );
        return ExitCode::from(64);
    }

    let bundle_path = PathBuf::from(&args[0]);
    let root = PathBuf::from(&args[1]);
    let bytes = match fs::read(&bundle_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", bundle_path.display());
            return ExitCode::from(1);
        }
    };
    let bundle: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(bundle) => bundle,
        Err(error) => {
            eprintln!("invalid JSON {}: {error}", bundle_path.display());
            return ExitCode::from(2);
        }
    };

    let report = validate_bundle(&bundle, &root);
    if report.is_valid() {
        println!(
            "SEMANTIC_VALIDATION_PASS: identity and artifact bytes are consistent; no solver or engineering validity is implied."
        );
        return ExitCode::SUCCESS;
    }

    for issue in &report.issues {
        eprintln!("{} {}: {}", issue.code, issue.path, issue.message);
    }
    eprintln!(
        "SEMANTIC_VALIDATION_FAIL: {} issue(s); no engineering approval implied.",
        report.issues.len()
    );
    ExitCode::from(2)
}
