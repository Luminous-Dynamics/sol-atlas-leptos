// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use sol_atlas_core::water_localization_benchmark::evaluate_localization_manifest;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: water-localization-benchmark-evaluate <scenario-corpus-run.json> <source-network.inp>");
        return ExitCode::from(64);
    };
    let Some(source_path) = args.next() else {
        eprintln!("usage: water-localization-benchmark-evaluate <scenario-corpus-run.json> <source-network.inp>");
        return ExitCode::from(64);
    };
    if args.next().is_some() {
        eprintln!("usage: water-localization-benchmark-evaluate <scenario-corpus-run.json> <source-network.inp>");
        return ExitCode::from(64);
    }

    let input = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("WATER_LOCALIZATION_INPUT_ERROR: {path}: {error}");
            return ExitCode::from(2);
        }
    };
    let source_inp = match fs::read(&source_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("WATER_LOCALIZATION_TOPOLOGY_INPUT_ERROR: {source_path}: {error}");
            return ExitCode::from(2);
        }
    };
    match evaluate_localization_manifest(&input, &source_inp) {
        Ok(result) => {
            match serde_json::to_string_pretty(&result) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("WATER_LOCALIZATION_SERIALIZATION_ERROR: {error}");
                    return ExitCode::from(2);
                }
            }
            eprintln!(
                "WATER_LOCALIZATION_BENCHMARK: state={} cases={} targets={} models={} claim_ceiling={}",
                result.evaluation.run_state,
                result.evaluation.fault_scenario_count,
                result.evaluation.candidate_target_count,
                result.evaluation.models.len(),
                result.evaluation.claim_ceiling
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("WATER_LOCALIZATION_BENCHMARK_INVALID_OR_BLOCKED: {error}");
            ExitCode::from(2)
        }
    }
}
