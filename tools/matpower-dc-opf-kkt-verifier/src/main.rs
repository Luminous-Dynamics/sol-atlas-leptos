// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::process::ExitCode;

use matpower_dc_opf_kkt_verifier::{KktOptions, audit_matpower_dc_opf_kkt};
use matpower_dc_opf_primal_auditor::AuditOptions as PrimalAuditOptions;

fn run() -> Result<bool, String> {
    let mut args = env::args().skip(1);
    let case_path = args.next().ok_or_else(|| {
        "usage: matpower-dc-opf-kkt-verify <case.m> <matpower-dcopf.tsv>".to_owned()
    })?;
    let receipt_path = args
        .next()
        .ok_or_else(|| "missing MATPOWER DC-OPF candidate receipt path".to_owned())?;
    if args.next().is_some() {
        return Err("too many command-line arguments".to_owned());
    }

    let source_bytes = fs::read(&case_path)
        .map_err(|error| format!("read MATPOWER source {case_path}: {error}"))?;
    let source = std::str::from_utf8(&source_bytes)
        .map_err(|_| format!("MATPOWER source {case_path} is not UTF-8"))?;
    let receipt = fs::read_to_string(&receipt_path)
        .map_err(|error| format!("read MATPOWER candidate receipt {receipt_path}: {error}"))?;

    let report = audit_matpower_dc_opf_kkt(
        source,
        &receipt,
        PrimalAuditOptions::default(),
        KktOptions::default(),
    )?;
    println!("{}", report.to_json());
    Ok(report.numerical_kkt_satisfied)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("matpower-dc-opf-kkt-verify: {error}");
            ExitCode::FAILURE
        }
    }
}
