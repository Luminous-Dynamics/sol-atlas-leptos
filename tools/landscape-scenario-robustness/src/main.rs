// Copyright (C) 2026 Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use landscape_scenario_robustness::{
    default_fixture_path, default_utility_method_path, evaluate_fixture_bytes,
};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 2 {
        eprintln!("usage: landscape-scenario-robustness [fixture.json] [utility-method.json]");
        return ExitCode::from(2);
    }
    let fixture_path = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(default_fixture_path);
    let method_path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_utility_method_path);
    let fixture_bytes = match fs::read(&fixture_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", fixture_path.display());
            return ExitCode::FAILURE;
        }
    };
    let method_bytes = match fs::read(&method_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", method_path.display());
            return ExitCode::FAILURE;
        }
    };
    match evaluate_fixture_bytes(&fixture_bytes, &method_bytes).and_then(|report| {
        serde_json::to_vec_pretty(&report).map_err(|error| vec![error.to_string()])
    }) {
        Ok(bytes) => {
            println!("{}", String::from_utf8_lossy(&bytes));
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}
