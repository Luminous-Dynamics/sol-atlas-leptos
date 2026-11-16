// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::process::ExitCode;

use dc_opf_finite_oracle::{solve_finite_dc_opf, three_bus_known_answer_case};

fn run() -> Result<(), String> {
    match env::args().nth(1).as_deref() {
        Some("--self-test") => {
            let report = solve_finite_dc_opf(&three_bus_known_answer_case())
                .map_err(|error| error.to_string())?;
            println!("{}", report.to_json());
            if !report.is_exact()
                || report.expected_combinations != 16
                || report.feasible_assignments != 9
                || report.frontier.len() != 4
            {
                return Err("finite DC-OPF known-answer contract failed".to_owned());
            }
            Ok(())
        }
        _ => Err("usage: dc-opf-finite-oracle --self-test".to_owned()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dc-opf-finite-oracle: {error}");
            ExitCode::FAILURE
        }
    }
}
