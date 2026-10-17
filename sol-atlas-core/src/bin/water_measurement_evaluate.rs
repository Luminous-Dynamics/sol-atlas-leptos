// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evaluate a small declared measurement-probability model.
//! This does not invoke EPANET/SWMM or validate a real water system.

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::water_measurement::{
    evaluate_measurement_model, MeasurementModelInput, MeasurementModelEvaluation,
};
use std::env;
use std::fs;
use std::process::ExitCode;

#[derive(Serialize)]
struct MeasurementEvaluationEnvelope {
    schema_version: String,
    source_input_sha256: String,
    evaluation: MeasurementModelEvaluation,
}

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
    let source_input_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let model: MeasurementModelInput = match serde_json::from_slice(&bytes) {
        Ok(model) => model,
        Err(error) => {
            eprintln!("invalid measurement model JSON {path}: {error}");
            return ExitCode::from(2);
        }
    };

    match evaluate_measurement_model(&model) {
        Ok(evaluation) => {
            let envelope = MeasurementEvaluationEnvelope {
                schema_version: "water-measurement-run-envelope/v0.1".to_owned(),
                source_input_sha256,
                evaluation,
            };
            match serde_json::to_string_pretty(&envelope) {
                Ok(output) => {
                    println!("{output}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("cannot encode evaluation output: {error}");
                    ExitCode::from(3)
                }
            }
        }
        Err(error) => {
            eprintln!("MEASUREMENT_MODEL_INVALID: {error}");
            ExitCode::from(2)
        }
    }
}
