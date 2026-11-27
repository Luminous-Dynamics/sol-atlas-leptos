// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut input_path = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input_path = args.next(),
            "--help" | "-h" => {
                println!("usage: ev_n3_published_trip_audit --input PATH");
                return ExitCode::SUCCESS;
            }
            _ => {
                eprintln!("unknown argument: {arg}");
                eprintln!("usage: ev_n3_published_trip_audit --input PATH");
                return ExitCode::from(2);
            }
        }
    }
    let Some(input_path) = input_path else {
        eprintln!("--input is required");
        eprintln!("usage: ev_n3_published_trip_audit --input PATH");
        return ExitCode::from(2);
    };

    let input = match fs::read(&input_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read N3 published trip corpus {input_path}: {error}");
            return ExitCode::from(2);
        }
    };

    match sol_atlas_core::n3_published_trip_audit::audit_json(&input) {
        Ok(envelope) => match serde_json::to_string_pretty(&envelope) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("could not serialize N3 trip audit report: {error}");
                ExitCode::from(2)
            }
        },
        Err(error) => {
            eprintln!("N3 published trip audit failed: {error}");
            ExitCode::from(2)
        }
    }
}
