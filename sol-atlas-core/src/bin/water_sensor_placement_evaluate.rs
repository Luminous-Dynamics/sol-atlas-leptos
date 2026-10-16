// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use sol_atlas_core::water_sensor_placement::evaluate_sensor_placement;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "usage: water-sensor-placement-evaluate <corpus-root> <run-manifest.json> <source-network.inp>"
        );
        return ExitCode::from(2);
    }
    let corpus_root = Path::new(&args[1]);
    let manifest = match fs::read(&args[2]) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("WATER_SENSOR_PLACEMENT_INPUT_ERROR: cannot read {}: {error}", args[2]);
            return ExitCode::from(2);
        }
    };
    let topology = match fs::read(&args[3]) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("WATER_SENSOR_PLACEMENT_TOPOLOGY_ERROR: cannot read {}: {error}", args[3]);
            return ExitCode::from(2);
        }
    };
    match evaluate_sensor_placement(corpus_root, &manifest, &topology) {
        Ok(envelope) => {
            match serde_json::to_string_pretty(&envelope) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("WATER_SENSOR_PLACEMENT_SERIALIZATION_ERROR: {error}");
                    return ExitCode::from(2);
                }
            }
            eprintln!(
                "WATER_SENSOR_PLACEMENT: state={} scenarios={} targets={} result_rows={} claim_ceiling={}",
                envelope.evaluation.run_state,
                envelope.evaluation.scenario_count,
                envelope.evaluation.candidate_target_count,
                envelope.evaluation.results.len(),
                envelope.evaluation.claim_ceiling
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("WATER_SENSOR_PLACEMENT_INVALID_OR_BLOCKED: {error}");
            ExitCode::from(2)
        }
    }
}
