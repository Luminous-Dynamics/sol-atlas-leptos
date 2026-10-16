// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use std::env;
use std::fs;
use std::process::ExitCode;

use sol_atlas_core::water_sensor_placement_statistics::compare_sensor_placement;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: water-sensor-placement-compare <evaluation.json>");
        return ExitCode::from(2);
    }
    let input = match fs::read(&args[1]) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("WATER_SENSOR_PLACEMENT_COMPARE_INPUT_ERROR: cannot read {}: {error}", args[1]);
            return ExitCode::from(2);
        }
    };
    match compare_sensor_placement(&input) {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(json) => {
                println!("{json}");
                eprintln!(
                    "WATER_SENSOR_PLACEMENT_COMPARE: state={} paired_comparisons={} bootstrap_replicates={} claim_ceiling={}",
                    result.comparison.run_state,
                    result.comparison.results.len(),
                    result.comparison.bootstrap_replicates,
                    result.comparison.claim_ceiling
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("WATER_SENSOR_PLACEMENT_COMPARE_SERIALIZATION_ERROR: {error}");
                ExitCode::from(2)
            }
        },
        Err(error) => {
            eprintln!("WATER_SENSOR_PLACEMENT_COMPARE_INVALID_OR_BLOCKED: {error}");
            ExitCode::from(2)
        }
    }
}
