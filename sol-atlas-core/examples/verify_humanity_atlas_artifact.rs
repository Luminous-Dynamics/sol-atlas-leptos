// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline verification for a Humanity Atlas run envelope and its source artifact.
//!
//! Usage:
//! cargo run -p sol-atlas-core --example verify_humanity_atlas_artifact --
//!   humanity-atlas-envelope.v0.1.json humanity-atlas-run.json source-simulator-output

use sol_atlas_core::humanity_atlas::{
    ScenarioRunEnvelope,
    technology_tree::import_technology_tree,
};
use std::{env, fs, io, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Humanity Atlas artifact rejected: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: verify_humanity_atlas_artifact <run-envelope.json> <source-artifact.json> [artifact-id]",
        ).into());
    }

    let envelope_path = &args[0];
    let artifact_path = &args[1];
    let artifact_id = args.get(2)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "source-simulator-output".to_string());

    let envelope_bytes = fs::read_to_string(envelope_path)?;
    let source_bytes = fs::read(artifact_path)?;
    let run = ScenarioRunEnvelope::from_json(&envelope_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let tree = import_technology_tree(&run, &artifact_id, &source_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    let summary = serde_json::json!({
        "verification": "structural_and_cross_binding_checks_passed",
        "scientific_validation": "not_established",
        "classification": tree.classification,
        "schema_version": tree.schema_version,
        "source_model": tree.source_model,
        "definition_source": tree.definition_source,
        "definition_sha256": tree.definition_sha256,
        "source_artifact_sha256": tree.source_artifact_sha256,
        "source_revision": tree.source_revision,
        "seed": tree.seed,
        "horizon_ticks": tree.horizon_ticks,
        "horizon_relative_years": tree.horizon_relative_years,
        "survived": tree.survived,
        "milestone_count": tree.milestones.len(),
        "dependency_count": tree.dependencies.len(),
    });
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
