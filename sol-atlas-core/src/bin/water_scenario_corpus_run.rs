// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Materialize and execute a deterministic, hash-bound synthetic EPANET corpus.

use sol_atlas_core::water_scenario_corpus::run_scenario_corpus;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 5 {
        eprintln!(
            "usage: water-scenario-corpus-run <base.inp> <plan.json> <runepanet> <output-dir> <solver-source-commit>\n\
             Generates declared synthetic demand-withdrawal cases, executes EPANET, and writes a hash-bound manifest."
        );
        return ExitCode::from(64);
    }

    match run_scenario_corpus(&args[0], &args[1], &args[2], &args[3], &args[4]) {
        Ok(manifest) => {
            eprintln!(
                "WATER_SCENARIO_CORPUS: state={} scenarios={} completed={} failed={} claim_ceiling={}",
                manifest.run_state,
                manifest.scenario_count,
                manifest.completed_count,
                manifest.failed_count,
                manifest.claim_ceiling
            );
            if manifest.failed_count == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("WATER_SCENARIO_CORPUS_INVALID_OR_BLOCKED: {error}");
            ExitCode::from(2)
        }
    }
}
