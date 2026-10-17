// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evaluate a small ensemble of alternative declared measurement models.
//! This does not invoke EPANET/SWMM or validate any real water system.

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::water_measurement::{
    evaluate_robust_measurement_ensemble, RobustMeasurementEnsembleInput,
    RobustMeasurementEnsembleEvaluation,
};
use std::env;
use std::fs;
use std::process::ExitCode;

#[derive(Serialize)]
struct RobustMeasurementEvaluationEnvelope {
    schema_version: String,
    source_input_sha256: String,
    evaluation: RobustMeasurementEnsembleEvaluation,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!(
            "usage: water-measurement-robust-evaluate <synthetic-ensemble.json>\n\
             Outputs minima/maxima across declared probability models; not a hydraulic validation."
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
    let ensemble: RobustMeasurementEnsembleInput = match serde_json::from_slice(&bytes) {
        Ok(ensemble) => ensemble,
        Err(error) => {
            eprintln!("invalid measurement ensemble JSON {path}: {error}");
            return ExitCode::from(2);
        }
    };

    match evaluate_robust_measurement_ensemble(&ensemble) {
        Ok(evaluation) => {
            let envelope = RobustMeasurementEvaluationEnvelope {
                schema_version: "water-measurement-robust-run-envelope/v0.1".to_owned(),
                source_input_sha256,
                evaluation,
            };
            match serde_json::to_string_pretty(&envelope) {
                Ok(output) => {
                    println!("{output}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("cannot encode robust evaluation output: {error}");
                    ExitCode::from(3)
                }
            }
        }
        Err(error) => {
            eprintln!("ROBUST_MEASUREMENT_ENSEMBLE_INVALID: {error}");
            ExitCode::from(2)
        }
    }
}
