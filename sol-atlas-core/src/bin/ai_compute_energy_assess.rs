//! Evaluate a compute-energy scenario JSON file with a scheduling baseline.
//!
//! Usage:
//!   cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- <scenario.json> [earliest-ready|lowest-cost|lowest-carbon|pareto]
//!
//! Output is a JSON schedule assessment on stdout. Exit codes:
//!   0 = feasible or capacity-limited assessment completed
//!   2 = protected floors are infeasible
//!   3 = required resource/objective inputs are unknown (indeterminate)
//!   4 = request is outside the evaluator's supported scope
//!   5 = request is invalid
//!   64 = command-line usage error
//!   65 = scenario JSON could not be parsed
//!   66 = scenario file could not be read
//!   70 = assessment could not be serialized

use sol_atlas_core::ai_compute_energy::{
    AssessmentStatus, Request, SchedulingPolicy, compare_scheduling_policies, schedule_deferrable,
};
use std::env;
use std::fs;
use std::process;

fn parse_policy(value: &str) -> Option<SchedulingPolicy> {
    match value {
        "earliest-ready" => Some(SchedulingPolicy::EarliestReady),
        "lowest-cost" => Some(SchedulingPolicy::LowestCost),
        "lowest-carbon" => Some(SchedulingPolicy::LowestCarbon),
        _ => None,
    }
}

fn exit_code(status: AssessmentStatus) -> i32 {
    match status {
        AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited => 0,
        AssessmentStatus::Infeasible => 2,
        AssessmentStatus::Indeterminate => 3,
        AssessmentStatus::Unsupported => 4,
        AssessmentStatus::Invalid => 5,
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!(
            "usage: ai_compute_energy_assess <scenario.json> [earliest-ready|lowest-cost|lowest-carbon|pareto]"
        );
        process::exit(64);
    };

    let (pareto_mode, policy) = match args.next() {
        None => (false, SchedulingPolicy::EarliestReady),
        Some(raw) => match raw.to_str() {
            Some("pareto") => (true, SchedulingPolicy::EarliestReady),
            Some(value) => match parse_policy(value) {
                Some(policy) => (false, policy),
                None => {
                    eprintln!(
                        "policy must be earliest-ready, lowest-cost, lowest-carbon, or pareto"
                    );
                    process::exit(64);
                }
            },
            None => {
                eprintln!("policy argument must be valid UTF-8");
                process::exit(64);
            }
        },
    };
    if args.next().is_some() {
        eprintln!(
            "usage: ai_compute_energy_assess <scenario.json> [earliest-ready|lowest-cost|lowest-carbon|pareto]"
        );
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

    let (json_result, exit_code) = if pareto_mode {
        let result = compare_scheduling_policies(&request);
        (
            serde_json::to_string_pretty(&result),
            exit_code(result.status),
        )
    } else {
        let result = schedule_deferrable(&request, policy);
        (
            serde_json::to_string_pretty(&result),
            exit_code(result.status),
        )
    };

    match json_result {
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
