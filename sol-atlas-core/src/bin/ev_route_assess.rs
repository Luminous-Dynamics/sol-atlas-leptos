// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut input_path = None;
    let mut trust_store_path = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input_path = args.next(),
            "--trust-store" => trust_store_path = args.next(),
            "--help" | "-h" => {
                println!("usage: ev_route_assess --input PATH --trust-store PATH");
                return ExitCode::SUCCESS;
            }
            _ => {
                eprintln!("unknown argument: {arg}");
                eprintln!("usage: ev_route_assess --input PATH --trust-store PATH");
                return ExitCode::from(2);
            }
        }
    }

    let (Some(input_path), Some(trust_store_path)) = (input_path, trust_store_path) else {
        eprintln!("both --input and --trust-store are required");
        eprintln!("usage: ev_route_assess --input PATH --trust-store PATH");
        return ExitCode::from(2);
    };

    let input = match fs::read(&input_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read route input {input_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let trust_store = match fs::read(&trust_store_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read evidence trust store {trust_store_path}: {error}");
            return ExitCode::from(2);
        }
    };

    match sol_atlas_core::ev_route_assurance::assess_json(&input, &trust_store) {
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
