// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Validate internal consistency of a water intervention decision packet.
//! JSON Schema and referenced run-bundle verification are separate gates.

use sol_atlas_core::water_decision::validate_decision_packet;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!(
            "usage: water-decision-validate <decision-packet.json>\n\
             This checks packet-local consistency only; it does not verify referenced run bundles or grant engineering approval."
        );
        return ExitCode::from(64);
    }

    let packet_path = &args[0];
    let bytes = match fs::read(packet_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {packet_path}: {error}");
            return ExitCode::from(1);
        }
    };
    let packet: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(packet) => packet,
        Err(error) => {
            eprintln!("invalid JSON {packet_path}: {error}");
            return ExitCode::from(2);
        }
    };

    let report = validate_decision_packet(&packet);
    if report.is_valid() {
        println!(
            "DECISION_SEMANTIC_VALIDATION_PASS: packet-local consistency only; no referenced-bundle, solver, or engineering validity implied."
        );
        return ExitCode::SUCCESS;
    }

    for issue in &report.issues {
        eprintln!("{} {}: {}", issue.code, issue.path, issue.message);
    }
    eprintln!(
        "DECISION_SEMANTIC_VALIDATION_FAIL: {} issue(s); no engineering approval implied.",
        report.issues.len()
    );
    ExitCode::from(2)
}
