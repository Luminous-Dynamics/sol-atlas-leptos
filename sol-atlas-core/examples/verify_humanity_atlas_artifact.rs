// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline verification for a Humanity Atlas run envelope and its source artifact.
//!
//! Usage, from the standalone Sol-Atlas repository root:
//! cargo run --manifest-path sol-atlas-core/Cargo.toml --example verify_humanity_atlas_artifact --
//!   humanity-atlas-envelope.v0.1.json humanity-atlas-run.json source-simulator-output

use sol_atlas_core::humanity_atlas::{
    ScenarioRunEnvelope,
    technology_tree::{MAX_SOURCE_ARTIFACT_BYTES, import_technology_tree},
};
use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read},
    process::ExitCode,
};

const MAX_RUN_ENVELOPE_BYTES: usize = 4 * 1024 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Humanity Atlas artifact rejected: {error}");
            ExitCode::from(2)
        }
    }
}

fn read_bounded_bytes(path: &OsStr, limit: usize, label: &str) -> io::Result<Vec<u8>> {
    let path = std::path::Path::new(path);
    let metadata = fs::metadata(path)?;
    if metadata.len() > limit as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{label} exceeds the {limit}-byte limit"),
        ));
    }

    // Read at most limit+1 bytes to close the metadata/read growth race.
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{label} exceeds the {limit}-byte limit"),
        ));
    }
    Ok(bytes)
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
    let artifact_id = args
        .get(2)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "source-simulator-output".to_string());

    let envelope_bytes = read_bounded_bytes(
        envelope_path.as_os_str(),
        MAX_RUN_ENVELOPE_BYTES,
        "run envelope",
    )?;
    let envelope_text = String::from_utf8(envelope_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let source_bytes = read_bounded_bytes(
        artifact_path.as_os_str(),
        MAX_SOURCE_ARTIFACT_BYTES,
        "source artifact",
    )?;
    let run = ScenarioRunEnvelope::from_json(&envelope_text)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let tree = import_technology_tree(&run, &artifact_id, &source_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    let summary = serde_json::json!({
        "qualification": "structural_contract_pass",
        "qualification_scope": [
            "run_envelope_structure",
            "exact_source_artifact_sha256",
            "tree_definition_digest",
            "manifest_and_resolved_input_bindings",
            "scenario_identity_and_summary_metrics",
            "milestone_graph_integrity"
        ],
        "verification": "structural_and_cross_binding_checks_passed",
        "scientific_validation": "not_established",
        "run_id": run.run_id,
        "scenario_id": run.scenario_id,
        "engine_version": run.engine.engine_version,
        "model_identity_kind": run.engine.model_identity_kind,
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
