// Copyright (C) 2026 Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use landscape_chain_conformance::{default_fixture_dir, run_conformance};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();
    let fixture_dir = match args.next() {
        Some(path) if args.next().is_none() => PathBuf::from(path),
        Some(_) => {
            eprintln!("usage: landscape-chain-conformance [fixture-directory]");
            return ExitCode::from(2);
        }
        None => default_fixture_dir(),
    };

    match run_conformance(&fixture_dir) {
        Ok(report) => match serde_json::to_string_pretty(&report) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("could not serialize conformance report: {error}");
                ExitCode::from(4)
            }
        },
        Err(errors) => {
            for error in errors {
                eprintln!("landscape chain conformance rejected: {error}");
            }
            ExitCode::from(3)
        }
    }
}
