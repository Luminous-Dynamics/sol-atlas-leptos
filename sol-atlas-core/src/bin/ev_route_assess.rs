// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: ev_route_assess <input.json>");
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("usage: ev_route_assess <input.json>");
        return ExitCode::from(2);
    }

    let input = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read input file: {error}");
            return ExitCode::from(2);
        }
    };
    match sol_atlas_core::ev_route_assurance::assess_json(&input) {
        Ok(envelope) => match serde_json::to_string_pretty(&envelope) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("could not serialize assessment envelope: {error}");
                ExitCode::from(2)
            }
        },
        Err(error) => {
            eprintln!("assessment rejected: {error}");
            ExitCode::from(2)
        }
    }
}
