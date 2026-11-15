// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use pglib_case_auditor::project_matpower_dc_case;

const MAX_SOURCE_BYTES: u64 = 32 * 1024 * 1024;

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| "usage: matpower-projection <case.m>".to_owned())?;
    let source_path = Path::new(&path);
    let metadata = fs::symlink_metadata(source_path)
        .map_err(|error| format!("cannot inspect source path: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("source must be a regular, non-symlink file".to_owned());
    }
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(format!("source exceeds {MAX_SOURCE_BYTES} byte limit"));
    }
    let bytes = fs::read(source_path).map_err(|error| format!("cannot read source: {error}"))?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(format!("source exceeds {MAX_SOURCE_BYTES} byte limit"));
    }
    let source = std::str::from_utf8(&bytes)
        .map_err(|_| "source is not UTF-8".to_owned())?;
    let projection = project_matpower_dc_case(source)?;
    println!("{}", projection.to_json());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("matpower-projection: {error}");
            ExitCode::FAILURE
        }
    }
}
