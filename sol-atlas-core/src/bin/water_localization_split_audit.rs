// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde::Deserialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::water_localization_split_audit::evaluate_split_plan;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn valid_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn run() -> Result<i32, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.len() > 2 {
        return Err(
            "usage: water-localization-split-audit <split-plan.json> [receipt.json]".to_owned(),
        );
    }
    let input_path = PathBuf::from(&args[0]);
    let bytes =
        fs::read(&input_path).map_err(|error| format!("read {}: {error}", input_path.display()))?;
    let plan = serde_json::from_slice::<SplitPlanOwned>(&bytes)
        .map_err(|error| format!("parse {}: {error}", input_path.display()))?;
    let plan_sha = sha256_hex(&bytes);
    let mut receipt = evaluate_split_plan(&plan.0, &plan_sha);
    receipt.implementation_revision = env::var("GITHUB_SHA")
        .ok()
        .filter(|revision| valid_revision(revision));
    let serialized = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize receipt: {error}"))?
        + "\n";
    if let Some(path) = args.get(1).map(PathBuf::from) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        fs::write(&path, serialized.as_bytes())
            .map_err(|error| format!("write {}: {error}", path.display()))?;
    }
    println!("{serialized}");
    Ok(if receipt.run_state == "Passed" { 0 } else { 2 })
}

#[derive(Deserialize)]
struct SplitPlanOwned(sol_atlas_core::water_localization_split_audit::SplitPlan);

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("WATER_LOCALIZATION_SPLIT_AUDIT_ERROR: {error}");
            ExitCode::from(64)
        }
    }
}
