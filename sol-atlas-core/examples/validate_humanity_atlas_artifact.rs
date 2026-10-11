// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline qualification tool for one captured Humanity Atlas simulator artifact.
//!
//! Usage:
//! cargo run --manifest-path sol-atlas-core/Cargo.toml --example validate_humanity_atlas_artifact -- //!   run-envelope.v0.1.json humanity-atlas-run.json source-simulator-output
//!
//! The source artifact is read once from disk and those exact bytes are checked
//! against the envelope SHA-256 before the model-specific tree import.

use sol_atlas_core::humanity_atlas::{
    ScenarioRunEnvelope,
    technology_tree::import_technology_tree,
};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("humanity-atlas artifact validation failed: {message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: validate_humanity_atlas_artifact <run-envelope.json> <source-artifact.json> <artifact-id>"
                .into(),
        );
    }

    let envelope_text = fs::read_to_string(&args[1])
        .map_err(|error| format!("cannot read run envelope {}: {error}", args[1]))?;
    let source_bytes = fs::read(&args[2])
        .map_err(|error| format!("cannot read source artifact {}: {error}", args[2]))?;

    let run = ScenarioRunEnvelope::from_json(&envelope_text)?;
    let tree = import_technology_tree(&run, &args[3], &source_bytes)?;

    let achieved = tree.milestones.iter().filter(|item| item.achieved).count();
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
        "scientific_classification": "unvalidated_simulation",
        "source_artifact_sha256": tree.source_artifact_sha256,
        "definition_sha256": tree.definition_sha256,
        "schema_version": tree.schema_version,
        "source_model": tree.source_model,
        "definition_source": tree.definition_source,
        "source_revision": tree.source_revision,
        "seed": tree.seed,
        "horizon_ticks": tree.horizon_ticks,
        "horizon_relative_years": tree.horizon_relative_years,
        "survived": tree.survived,
        "milestone_count": tree.milestones.len(),
        "achieved_milestone_count": achieved,
        "dependency_count": tree.dependencies.len()
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&summary)
            .map_err(|error| format!("cannot render validation summary: {error}"))?
    );
    Ok(())
}
