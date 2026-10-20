//! Evaluate a compute-energy scenario JSON file using the bounded reference evaluator.
//!
//! Usage:
//!   cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- path/to/scenario.json
//!
//! Output is a JSON assessment on stdout. Exit codes:
//!   0 = feasible or capacity-limited assessment completed
//!   2 = protected floors are infeasible
//!   3 = required resource inputs are unknown (indeterminate)
//!   4 = request is outside the evaluator's supported scope
//!   5 = request is invalid
//!   64 = command-line usage error
//!   65 = scenario JSON could not be parsed
//!   66 = scenario file could not be read
//!   70 = assessment could not be serialized

use sol_atlas_core::ai_compute_energy::{assess, AssessmentStatus, Request};
use std::env;
use std::fs;
use std::process;

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: ai_compute_energy_assess <scenario.json>");
        process::exit(64);
    };
    if args.next().is_some() {
        eprintln!("usage: ai_compute_energy_assess <scenario.json>");
        process::exit(64);
    }

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("could not read scenario file: {error}");
            process::exit(66);
        }
    };

    let request: Request = match serde_json::from_str(&content) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("could not parse compute-energy scenario JSON: {error}");
            process::exit(65);
        }
    };

    let result = assess(&request);
    let exit_code = match result.status {
        AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited => 0,
        AssessmentStatus::Infeasible => 2,
        AssessmentStatus::Indeterminate => 3,
        AssessmentStatus::Unsupported => 4,
        AssessmentStatus::Invalid => 5,
    };

    match serde_json::to_string_pretty(&result) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("could not serialize assessment: {error}");
            process::exit(70);
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}
