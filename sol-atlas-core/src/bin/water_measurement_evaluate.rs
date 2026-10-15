// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evaluate a small declared measurement-probability model.
//! This does not invoke EPANET/SWMM or validate a real water system.

use sol_atlas_core::water_measurement::{
    evaluate_measurement_model, MeasurementModelInput,
};
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!(
            "usage: water-measurement-evaluate <synthetic-model.json>\n\
             Outputs information gain and expected loss reduction conditional on declared assumptions; not hydraulic validation."
        );
        return ExitCode::from(64);
    }

    let path = &args[0];
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(1);
        }
    };
    let model: MeasurementModelInput = match serde_json::from_slice(&bytes) {
        Ok(model) => model,
        Err(error) => {
            eprintln!("invalid measurement model JSON {path}: {error}");
            return ExitCode::from(2);
        }
    };

    match evaluate_measurement_model(&model) {
        Ok(evaluation) => match serde_json::to_string_pretty(&evaluation) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cannot encode evaluation output: {error}");
                ExitCode::from(3)
            }
        },
        Err(error) => {
            eprintln!("MEASUREMENT_MODEL_INVALID: {error}");
            ExitCode::from(2)
        }
    }
}
