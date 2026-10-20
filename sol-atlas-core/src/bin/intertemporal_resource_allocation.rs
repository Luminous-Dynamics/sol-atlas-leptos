// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::from_slice;
use sol_atlas_core::service_intertemporal_allocation::{
    IntertemporalAllocationRequest, assess_intertemporal_resource_allocation,
};
use std::{env, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("intertemporal-resource-allocation: {error}");
        process::exit(2);
    }
}

fn option_value(args: &[String], name: &str) -> Result<String, String> {
    let Some(index) = args.iter().position(|arg| arg == name) else {
        return Err(format!(
            "usage: intertemporal-resource-allocation --input REQUEST.json"
        ));
    };
    args.get(index + 1)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| format!("argument {name} requires a value"))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let input_path = option_value(&args, "--input")?;
    let input = fs::read(&input_path)
        .map_err(|error| format!("cannot read request {input_path}: {error}"))?;
    let request: IntertemporalAllocationRequest = from_slice(&input)
        .map_err(|error| format!("request JSON does not match the typed contract: {error}"))?;
    let receipt = assess_intertemporal_resource_allocation(&request).map_err(|issues| {
        format!("request rejected by structural/domain validation: {issues:?}")
    })?;
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt)
            .map_err(|error| format!("cannot serialize assessment receipt: {error}"))?
    );
    Ok(())
}
