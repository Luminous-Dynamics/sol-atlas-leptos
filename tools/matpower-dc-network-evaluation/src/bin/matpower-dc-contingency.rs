// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::process::ExitCode;

use dc_power_flow_reference::SolveOptions;
use matpower_dc_network_evaluation::sweep_matpower_dc_case;

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| "usage: matpower-dc-contingency <case.m>".to_owned())?;
    let bytes = fs::read(&path).map_err(|error| format!("read {path}: {error}"))?;
    let source = std::str::from_utf8(&bytes).map_err(|_| format!("source {path} is not UTF-8"))?;
    let evaluation = sweep_matpower_dc_case(source, SolveOptions::default(), 256)
        .map_err(|error| error.to_string())?;
    println!("{}", evaluation.to_json());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
