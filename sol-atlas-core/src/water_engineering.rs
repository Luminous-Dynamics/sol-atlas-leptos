// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fail-closed semantic validation for water engineering run bundles.
//!
//! JSON Schema validates shape. This module validates cross-record identity and
//! checks the actual bytes referenced by artifact manifests. Passing this
//! validator is not hydraulic validation, calibration, or engineering approval.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    fn push(&mut self, code: &'static str, path: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
}

/// Validate bundle identity and referenced artifact bytes against a repository root.
///
/// The caller should run JSON Schema validation as a separate gate. This function
/// intentionally does not infer missing values, convert units, or run a solver.
pub fn validate_bundle(bundle: &Value, repository_root: &Path) -> ValidationReport {
    let mut report = ValidationReport::default();
    let root = match repository_root.canonicalize() {
        Ok(root) if root.is_dir() => root,
        Ok(_) => {
            report.push(
                "ROOT_NOT_DIRECTORY",
                "$",
                "repository root is not a directory",
            );
            return report;
        }
        Err(error) => {
            report.push(
                "ROOT_UNAVAILABLE",
                "$",
                format!("cannot canonicalize repository root: {error}"),
            );
            return report;
        }
    };

    let Some(object) = bundle.as_object() else {
        report.push("BUNDLE_NOT_OBJECT", "$", "bundle must be a JSON object");
        return report;
    };

    if object.get("schema_version").and_then(Value::as_str) != Some("water-engineering-bundle/v0.1")
    {
        report.push(
            "SCHEMA_VERSION_UNSUPPORTED",
            "$.schema_version",
            "expected water-engineering-bundle/v0.1",
        );
    }

    let Some(model) = bundle.get("model_package") else {
        report.push(
            "MODEL_MISSING",
            "$.model_package",
            "model_package is required",
        );
        return report;
    };
    let Some(scenario) = bundle.get("scenario") else {
        report.push("SCENARIO_MISSING", "$.scenario", "scenario is required");
        return report;
    };
    let Some(run) = bundle.get("run_result") else {
        report.push("RUN_MISSING", "$.run_result", "run_result is required");
        return report;
    };

    let model_id = model.get("model_id").and_then(Value::as_str);
    let scenario_model_id = scenario.get("base_model_id").and_then(Value::as_str);
    if model_id.is_none() || scenario_model_id != model_id {
        report.push(
            "SCENARIO_MODEL_MISMATCH",
            "$.scenario.base_model_id",
            "scenario base_model_id must exactly match model_package.model_id",
        );
    }

    let input = model.get("input_snapshot");
    let input_digest = input.and_then(|v| v.get("sha256")).and_then(Value::as_str);
    let scenario_digest = scenario
        .get("base_input_snapshot_sha256")
        .and_then(Value::as_str);
    if input_digest.is_none() || scenario_digest != input_digest {
        report.push(
            "SCENARIO_INPUT_DIGEST_MISMATCH",
            "$.scenario.base_input_snapshot_sha256",
            "scenario input digest must exactly match model_package.input_snapshot.sha256",
        );
    }

    // The input snapshot is also expected once in source_artifacts as an inventory
    // entry. It is a second reference to the same artifact, not a duplicate.
    let source_artifacts = model.get("source_artifacts").and_then(Value::as_array);
    let mut source_ids = HashSet::new();
    if let Some(artifacts) = source_artifacts {
        for (index, artifact) in artifacts.iter().enumerate() {
            let path = format!("$.model_package.source_artifacts[{index}]");
            if let Some(id) = artifact.get("artifact_id").and_then(Value::as_str) {
                if !source_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_SOURCE_ARTIFACT_ID",
                        format!("{path}.artifact_id"),
                        format!("source artifact id {id:?} occurs more than once"),
                    );
                }
            }
        }
    } else {
        report.push(
            "SOURCE_ARTIFACTS_MISSING",
            "$.model_package.source_artifacts",
            "source_artifacts must be an array",
        );
    }

    if let (Some(input), Some(artifacts)) = (input, source_artifacts) {
        let input_id = input.get("artifact_id").and_then(Value::as_str);
        let matching: Vec<&Value> = artifacts
            .iter()
            .filter(|artifact| artifact.get("artifact_id").and_then(Value::as_str) == input_id)
            .collect();
        if input_id.is_none() || matching.len() != 1 {
            report.push(
                "INPUT_INVENTORY_IDENTITY_MISMATCH",
                "$.model_package.input_snapshot",
                "input snapshot must have exactly one source_artifacts entry with the same artifact_id",
            );
        } else if !artifact_identity_matches(input, matching[0]) {
            report.push(
                "INPUT_INVENTORY_CONTENT_MISMATCH",
                "$.model_package.input_snapshot",
                "input snapshot and inventory entry must agree on path, digest, byte length, and synthetic flag",
            );
        }
    }

    // Asset identifiers must be unique and every scenario change must target a
    // declared asset. Unknown target assets block the bundle rather than guessing.
    let asset_index = model.get("asset_index").and_then(Value::as_array);
    let mut asset_ids = HashSet::new();
    if let Some(assets) = asset_index {
        for (index, asset) in assets.iter().enumerate() {
            if let Some(id) = asset.get("asset_id").and_then(Value::as_str) {
                if !asset_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_ASSET_ID",
                        format!("$.model_package.asset_index[{index}].asset_id"),
                        format!("asset id {id:?} occurs more than once"),
                    );
                }
            } else {
                report.push(
                    "ASSET_ID_MISSING",
                    format!("$.model_package.asset_index[{index}].asset_id"),
                    "asset_id is required",
                );
            }
        }
    } else {
        report.push(
            "ASSET_INDEX_MISSING",
            "$.model_package.asset_index",
            "asset_index must be an array",
        );
    }

    if let Some(changes) = scenario.get("changes").and_then(Value::as_array) {
        for (index, change) in changes.iter().enumerate() {
            let change_path = format!("$.scenario.changes[{index}].asset_id");
            match change.get("asset_id").and_then(Value::as_str) {
                Some(id) if asset_ids.contains(id) => {}
                Some(id) => report.push(
                    "SCENARIO_ASSET_UNKNOWN",
                    change_path,
                    format!(
                        "scenario target asset {id:?} is absent from model_package.asset_index"
                    ),
                ),
                None => report.push(
                    "SCENARIO_ASSET_ID_MISSING",
                    change_path,
                    "scenario change requires an asset_id",
                ),
            }
        }
    } else {
        report.push(
            "SCENARIO_CHANGES_MISSING",
            "$.scenario.changes",
            "scenario changes must be an array",
        );
    }

    let model_engine = model
        .get("solver_profile")
        .and_then(|v| v.get("engine"))
        .and_then(Value::as_str);
    let run_engine = run
        .get("solver")
        .and_then(|v| v.get("engine"))
        .and_then(Value::as_str);
    if model_engine.is_none() || run_engine != model_engine {
        report.push(
            "SOLVER_ENGINE_MISMATCH",
            "$.run_result.solver.engine",
            "run solver engine must match the model package solver profile",
        );
    }

    // Validate bytes, not merely hex syntax. Reject absolute paths, parent
    // traversal, and symlinks that resolve outside the repository root.
    let mut artifacts_to_check: Vec<(String, &Value)> = Vec::new();
    if let Some(input) = input {
        artifacts_to_check.push(("$.model_package.input_snapshot".to_owned(), input));
    }
    if let Some(artifacts) = source_artifacts {
        for (index, artifact) in artifacts.iter().enumerate() {
            artifacts_to_check.push((
                format!("$.model_package.source_artifacts[{index}]"),
                artifact,
            ));
        }
    }
    if let Some(artifacts) = run.get("result_artifacts").and_then(Value::as_array) {
        let mut result_ids = HashSet::new();
        for (index, artifact) in artifacts.iter().enumerate() {
            let path = format!("$.run_result.result_artifacts[{index}]");
            if let Some(id) = artifact.get("artifact_id").and_then(Value::as_str) {
                if !result_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_RESULT_ARTIFACT_ID",
                        format!("{path}.artifact_id"),
                        format!("result artifact id {id:?} occurs more than once"),
                    );
                }
            }
            artifacts_to_check.push((path, artifact));
        }
    }

    for (manifest_path, artifact) in artifacts_to_check {
        validate_artifact_bytes(&root, artifact, &manifest_path, &mut report);
    }

    validate_run_claims(run, bundle.get("validation"), &mut report);
    report
}

fn artifact_identity_matches(left: &Value, right: &Value) -> bool {
    ["artifact_id", "path", "sha256", "byte_length", "synthetic"]
        .iter()
        .all(|key| left.get(*key) == right.get(*key))
}

fn validate_artifact_bytes(
    root: &Path,
    artifact: &Value,
    manifest_path: &str,
    report: &mut ValidationReport,
) {
    let relative = match artifact.get("path").and_then(Value::as_str) {
        Some(path) => Path::new(path),
        None => {
            report.push(
                "ARTIFACT_PATH_MISSING",
                format!("{manifest_path}.path"),
                "artifact path is required",
            );
            return;
        }
    };

    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        report.push(
            "ARTIFACT_PATH_UNSAFE",
            format!("{manifest_path}.path"),
            "artifact path must be a non-empty repository-relative path with no dot, parent, or root components",
        );
        return;
    }

    let joined = root.join(relative);
    let resolved = match joined.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            report.push(
                "ARTIFACT_UNAVAILABLE",
                format!("{manifest_path}.path"),
                format!("cannot resolve artifact: {error}"),
            );
            return;
        }
    };
    if !resolved.starts_with(root) {
        report.push(
            "ARTIFACT_PATH_ESCAPES_ROOT",
            format!("{manifest_path}.path"),
            "resolved artifact path escapes repository root",
        );
        return;
    }

    let bytes = match fs::read(&resolved) {
        Ok(bytes) => bytes,
        Err(error) => {
            report.push(
                "ARTIFACT_UNREADABLE",
                format!("{manifest_path}.path"),
                format!("cannot read artifact: {error}"),
            );
            return;
        }
    };

    let expected_length = artifact.get("byte_length").and_then(Value::as_u64);
    if expected_length != Some(bytes.len() as u64) {
        report.push(
            "ARTIFACT_BYTE_LENGTH_MISMATCH",
            format!("{manifest_path}.byte_length"),
            format!(
                "manifest length {:?} does not match actual byte length {}",
                expected_length,
                bytes.len()
            ),
        );
    }

    let expected_digest = artifact.get("sha256").and_then(Value::as_str);
    let actual_digest = format!("{:x}", Sha256::digest(&bytes));
    if !expected_digest.is_some_and(is_canonical_sha256) {
        report.push(
            "ARTIFACT_DIGEST_NONCANONICAL",
            format!("{manifest_path}.sha256"),
            "SHA-256 must be exactly 64 lowercase hexadecimal characters",
        );
    } else if expected_digest != Some(actual_digest.as_str()) {
        report.push(
            "ARTIFACT_DIGEST_MISMATCH",
            format!("{manifest_path}.sha256"),
            format!(
                "declared SHA-256 does not match bytes at {}",
                relative.display()
            ),
        );
    }
}

fn is_canonical_sha256(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_run_claims(run: &Value, validation: Option<&Value>, report: &mut ValidationReport) {
    let state = run.get("state").and_then(Value::as_str);
    let metrics = run.get("metrics").and_then(Value::as_array);
    let result_artifacts = run.get("result_artifacts").and_then(Value::as_array);
    let execution = run.get("execution");
    let diagnostics = run.get("diagnostics");

    if state == Some("NotRun") {
        if metrics.is_none_or(|items| !items.is_empty()) {
            report.push(
                "NOTRUN_HAS_METRICS",
                "$.run_result.metrics",
                "NotRun cannot report metrics; unavailable values must not be replaced with zero",
            );
        }
        if result_artifacts.is_none_or(|items| !items.is_empty()) {
            report.push(
                "NOTRUN_HAS_RESULT_ARTIFACTS",
                "$.run_result.result_artifacts",
                "NotRun cannot claim result artifacts",
            );
        }
        for key in ["started_at", "finished_at"] {
            if execution
                .and_then(|v| v.get(key))
                .is_none_or(|v| !v.is_null())
            {
                report.push(
                    "NOTRUN_HAS_EXECUTION_TIME",
                    format!("$.run_result.execution.{key}"),
                    "NotRun execution timestamps must be null",
                );
            }
        }
        if diagnostics
            .and_then(|v| v.get("solver_exit_code"))
            .is_none_or(|v| !v.is_null())
        {
            report.push(
                "NOTRUN_HAS_EXIT_CODE",
                "$.run_result.diagnostics.solver_exit_code",
                "NotRun solver exit code must be null",
            );
        }
        for key in ["convergence_state", "continuity_check_state"] {
            if diagnostics.and_then(|v| v.get(key)).and_then(Value::as_str) != Some("NotRun") {
                report.push(
                    "NOTRUN_HAS_DIAGNOSTIC_CLAIM",
                    format!("$.run_result.diagnostics.{key}"),
                    "NotRun diagnostics must remain NotRun",
                );
            }
        }
        if validation
            .and_then(|v| v.get("accepted_for_engineering_use"))
            .and_then(Value::as_bool)
            != Some(false)
        {
            report.push(
                "NOTRUN_ENGINEERING_ACCEPTANCE",
                "$.validation.accepted_for_engineering_use",
                "NotRun bundles must explicitly deny engineering-use acceptance",
            );
        }
        if validation
            .and_then(|v| v.get("empirical_state"))
            .and_then(Value::as_str)
            == Some("ValidatedForDeclaredScope")
        {
            report.push(
                "NOTRUN_CLAIMS_EMPIRICAL_VALIDATION",
                "$.validation.empirical_state",
                "a NotRun bundle cannot claim empirical validation",
            );
        }
        if validation
            .and_then(|v| v.get("independent_review_state"))
            .and_then(Value::as_str)
            == Some("AcceptedForDeclaredScope")
        {
            report.push(
                "NOTRUN_CLAIMS_INDEPENDENT_ACCEPTANCE",
                "$.validation.independent_review_state",
                "a NotRun bundle cannot claim independent acceptance for engineering use",
            );
        }
    }

    if state == Some("Completed") {
        let exit_code = diagnostics
            .and_then(|v| v.get("solver_exit_code"))
            .and_then(Value::as_i64);
        if exit_code != Some(0) {
            report.push(
                "COMPLETED_WITHOUT_ZERO_EXIT",
                "$.run_result.diagnostics.solver_exit_code",
                "Completed requires an explicitly recorded zero solver exit code",
            );
        }
        if diagnostics
            .and_then(|v| v.get("convergence_state"))
            .and_then(Value::as_str)
            != Some("Converged")
        {
            report.push(
                "COMPLETED_WITHOUT_CONVERGENCE",
                "$.run_result.diagnostics.convergence_state",
                "Completed requires explicit convergence; use NonConverged or Failed otherwise",
            );
        }
        for key in ["version", "binary_sha256", "settings_sha256"] {
            let value = run.get("solver").and_then(|v| v.get(key));
            let present = match key {
                "version" => value
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.trim().is_empty()),
                _ => value
                    .and_then(Value::as_str)
                    .is_some_and(is_canonical_sha256),
            };
            if !present {
                report.push(
                    "COMPLETED_SOLVER_IDENTITY_INCOMPLETE",
                    format!("$.run_result.solver.{key}"),
                    "Completed requires a non-empty version and canonical solver/settings digests",
                );
            }
        }
        for key in ["started_at", "finished_at"] {
            if execution
                .and_then(|v| v.get(key))
                .and_then(Value::as_str)
                .is_none_or(|s| s.trim().is_empty())
            {
                report.push(
                    "COMPLETED_TIMESTAMP_MISSING",
                    format!("$.run_result.execution.{key}"),
                    "Completed requires both execution timestamps",
                );
            }
        }
        if result_artifacts.is_none_or(Vec::is_empty) {
            report.push(
                "COMPLETED_RESULT_ARTIFACTS_MISSING",
                "$.run_result.result_artifacts",
                "Completed requires at least one reproducible result artifact",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("sol-atlas-core must live under the repository root")
            .to_path_buf()
    }

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/water-engineering/w0-p2-diameter-restriction.synthetic.json"
        ))
        .expect("synthetic water bundle fixture must parse")
    }

    #[test]
    fn existing_not_run_fixture_has_consistent_semantics_and_artifact_bytes() {
        let report = validate_bundle(&fixture(), &repo_root());
        assert!(
            report.is_valid(),
            "unexpected validation issues: {:#?}",
            report.issues
        );
    }

    #[test]
    fn rejects_scenario_bound_to_another_model() {
        let mut bundle = fixture();
        bundle["scenario"]["base_model_id"] = json!("different-model");
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "SCENARIO_MODEL_MISMATCH")
        );
    }

    #[test]
    fn rejects_scenario_bound_to_another_input_digest() {
        let mut bundle = fixture();
        bundle["scenario"]["base_input_snapshot_sha256"] = json!("0".repeat(64));
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "SCENARIO_INPUT_DIGEST_MISMATCH")
        );
    }

    #[test]
    fn rejects_not_run_bundle_with_reported_metrics() {
        let mut bundle = fixture();
        bundle["run_result"]["metrics"] = json!([{"metric_id":"pressure","value":0,"unit":"m"}]);
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "NOTRUN_HAS_METRICS")
        );
    }

    #[test]
    fn rejects_unknown_scenario_asset() {
        let mut bundle = fixture();
        bundle["scenario"]["changes"][0]["asset_id"] = json!("P999");
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "SCENARIO_ASSET_UNKNOWN")
        );
    }

    #[test]
    fn rejects_a_wrong_artifact_digest_even_when_it_is_well_formed() {
        let mut bundle = fixture();
        bundle["model_package"]["input_snapshot"]["sha256"] = json!("0".repeat(64));
        bundle["model_package"]["source_artifacts"][0]["sha256"] = json!("0".repeat(64));
        bundle["scenario"]["base_input_snapshot_sha256"] = json!("0".repeat(64));
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "ARTIFACT_DIGEST_MISMATCH")
        );
    }

    #[test]
    fn rejects_artifact_path_traversal() {
        let mut bundle = fixture();
        bundle["model_package"]["input_snapshot"]["path"] = json!("../../etc/passwd");
        bundle["model_package"]["source_artifacts"][0]["path"] = json!("../../etc/passwd");
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "ARTIFACT_PATH_UNSAFE")
        );
    }

    #[test]
    fn rejects_duplicate_asset_ids() {
        let mut bundle = fixture();
        let duplicate = bundle["model_package"]["asset_index"][0].clone();
        bundle["model_package"]["asset_index"]
            .as_array_mut()
            .expect("asset_index is an array")
            .push(duplicate);
        let report = validate_bundle(&bundle, &repo_root());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "DUPLICATE_ASSET_ID")
        );
    }
}
