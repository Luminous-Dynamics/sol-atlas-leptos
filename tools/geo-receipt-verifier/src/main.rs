// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This verifier checks evidence protocol and integrity. It does not prove that a
// geophysical model or interpretation is physically correct.

use jsonschema::draft202012;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

const REPORT_SCHEMA_VERSION: u32 = 1;
const QUALIFIED_STATUSES: [&str; 3] = [
    "qualified_synthetic",
    "source_adapter_qualified",
    "real_observation_evaluated",
];

#[derive(Debug)]
struct Args {
    repo_root: PathBuf,
    schema_path: String,
    receipt_path: String,
    artifact_root: Option<PathBuf>,
    target_status: Option<String>,
    verifier_report_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct Check {
    id: String,
    passed: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct ArtifactCheck {
    artifact_id: String,
    role: String,
    locator: String,
    expected_sha256: Option<String>,
    actual_sha256: Option<String>,
    expected_byte_length: Option<u64>,
    actual_byte_length: Option<u64>,
    passed: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct VerificationReport {
    report_schema_version: u32,
    verifier_name: String,
    verifier_version: String,
    verifier_commit_sha: Option<String>,
    qualification_core_sha256: Option<String>,
    input_receipt_status: Option<String>,
    target_qualification_status: Option<String>,
    schema_valid: bool,
    schema_errors: Vec<String>,
    benchmark_specification_integrity: bool,
    artifact_integrity: bool,
    artifact_checks: Vec<ArtifactCheck>,
    repository_head_sha: Option<String>,
    repository_worktree_clean: Option<bool>,
    repository_identity_matches: bool,
    checks: Vec<Check>,
    qualification_eligible: bool,
    verification_outcome: String,
    findings: Vec<String>,
    claim_ceiling: String,
}

fn main() -> ExitCode {
    let args = match parse_args(env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(64);
        }
    };

    match verify(args) {
        Ok(report) => {
            let outcome = report.verification_outcome.clone();
            let hard_failure = !report.schema_valid
                || !report.benchmark_specification_integrity
                || !report.artifact_integrity;
            let eligible = report.qualification_eligible;
            match serde_json::to_vec_pretty(&report) {
                Ok(mut bytes) => {
                    bytes.push(b'\n');
                    if std::io::stdout().write_all(&bytes).is_err() {
                        eprintln!("failed to write verification report to stdout");
                        return ExitCode::from(1);
                    }
                }
                Err(error) => {
                    eprintln!("failed to serialize verification report: {error}");
                    return ExitCode::from(1);
                }
            }
            if hard_failure {
                ExitCode::from(1)
            } else if outcome == "incomplete" || outcome == "rejected"
                || (!eligible
                    && report.target_qualification_status.as_deref() != Some("executed_unqualified"))
            {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    if args.len() < 3 {
        return Err(usage());
    }
    let repo_root = PathBuf::from(&args[0]);
    let schema_path = args[1].clone();
    let receipt_path = args[2].clone();
    let mut artifact_root = None;
    let mut target_status = None;
    let mut verifier_report_path = None;
    let mut index = 3;

    while index < args.len() {
        match args[index].as_str() {
            "--artifact-root" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                artifact_root = Some(PathBuf::from(value));
            }
            "--target-status" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                let valid = value == "executed_unqualified"
                    || QUALIFIED_STATUSES.contains(&value.as_str());
                if !valid {
                    return Err(format!("unknown target status: {value}\n{}", usage()));
                }
                target_status = Some(value.clone());
            }
            "--verifier-report" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                verifier_report_path = Some(PathBuf::from(value));
            }
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
        index += 1;
    }

    Ok(Args {
        repo_root,
        schema_path,
        receipt_path,
        artifact_root,
        target_status,
        verifier_report_path,
    })
}

fn usage() -> String {
    "usage: geo-receipt-verifier <repo-root> <schema-path> <receipt-path> \
     [--artifact-root PATH] [--target-status STATUS] [--verifier-report PATH]"
        .to_string()
}

fn verify(args: Args) -> Result<VerificationReport, String> {
    let repo_root = fs::canonicalize(&args.repo_root)
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    let artifact_root_candidate = match args.artifact_root.as_ref() {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.clone(),
    };
    let artifact_root = fs::canonicalize(&artifact_root_candidate)
        .map_err(|error| format!("cannot resolve artifact root: {error}"))?;

    let schema_path = resolve_cli_input(&repo_root, &args.schema_path)?;
    let receipt_path = resolve_cli_input(&repo_root, &args.receipt_path)?;
    let schema_bytes = fs::read(&schema_path)
        .map_err(|error| format!("cannot read schema {}: {error}", schema_path.display()))?;
    let receipt_bytes = fs::read(&receipt_path)
        .map_err(|error| format!("cannot read receipt {}: {error}", receipt_path.display()))?;
    let schema: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema is not valid JSON: {error}"))?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| format!("receipt is not valid JSON: {error}"))?;

    let mut schema_errors = Vec::new();
    if let Err(error) = draft202012::meta::validate(&schema) {
        schema_errors.push(format!("schema fails Draft 2020-12 meta-schema: {error}"));
    }
    let validator = draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| format!("cannot compile local receipt schema: {error}"))?;
    schema_errors.extend(
        validator
            .iter_errors(&receipt)
            .map(|error| format!("{}: {error}", error.instance_path())),
    );
    let schema_valid = schema_errors.is_empty();

    let input_status = string_field(&receipt, &["qualification_status"]).map(str::to_owned);
    let target_status = args
        .target_status
        .clone()
        .or_else(|| input_status.clone())
        .unwrap_or_else(|| "executed_unqualified".to_string());
    if target_status != "executed_unqualified"
        && !QUALIFIED_STATUSES.contains(&target_status.as_str())
    {
        return Err(format!("unsupported target status: {target_status}"));
    }

    let core_sha256 = qualification_core_sha256(&receipt).ok();
    let (actual_head, worktree_clean, remote_url) = git_snapshot(&repo_root);
    let verifier_commit_sha = actual_head.clone();

    let mut checks = Vec::new();
    let mut findings = schema_errors.clone();
    push_check(
        &mut checks,
        "receipt_schema",
        schema_valid,
        if schema_valid {
            "receipt conforms to the supplied local Draft 2020-12 schema".to_string()
        } else {
            "receipt or schema failed validation".to_string()
        },
    );

    let spec_integrity = verify_benchmark_specification(&repo_root, &receipt, &mut checks, &mut findings);
    let artifact_checks = verify_artifacts(&artifact_root, &receipt, &mut findings);
    let artifact_integrity = !artifact_checks.is_empty()
        && artifact_checks.iter().all(|artifact| artifact.passed);
    push_check(
        &mut checks,
        "declared_artifact_integrity",
        artifact_integrity,
        format!("{} artifact(s) checked", artifact_checks.len()),
    );

    let recorded_commit = string_field(&receipt, &["code", "commit_sha"]);
    let head_matches = recorded_commit.is_some() && recorded_commit == actual_head.as_deref();
    push_check(
        &mut checks,
        "exact_git_head",
        head_matches,
        format!(
            "receipt commit {}; checkout HEAD {}",
            recorded_commit.unwrap_or("<missing>"),
            actual_head.as_deref().unwrap_or("<unavailable>")
        ),
    );

    let remote_matches = match (
        string_field(&receipt, &["code", "repository_url"]),
        remote_url.as_deref(),
    ) {
        (Some(expected), Some(actual)) => normalize_git_url(expected) == normalize_git_url(actual),
        _ => false,
    };
    push_check(
        &mut checks,
        "repository_identity",
        remote_matches,
        format!(
            "recorded repository {}; origin {}",
            string_field(&receipt, &["code", "repository_url"]).unwrap_or("<missing>"),
            remote_url.as_deref().unwrap_or("<unavailable>")
        ),
    );

    push_check(
        &mut checks,
        "clean_worktree",
        worktree_clean == Some(true)
            && string_field(&receipt, &["code", "worktree_state"]) == Some("clean"),
        format!(
            "recorded worktree state {}; measured clean state {:?}",
            string_field(&receipt, &["code", "worktree_state"]).unwrap_or("<missing>"),
            worktree_clean
        ),
    );

    let mut qualification_reasons = Vec::new();
    let ci_passed = ci_checks_all_pass(&receipt);
    push_check(
        &mut checks,
        "exact_head_ci",
        ci_passed,
        if ci_passed {
            "all declared required checks completed successfully on the recorded code commit".to_string()
        } else {
            "one or more required checks are absent, queued, skipped, cancelled, stale-head, non-success, or tied to another commit".to_string()
        },
    );
    if target_status != "executed_unqualified" {
        if !schema_valid {
            qualification_reasons.push("receipt/schema validation did not pass".to_string());
        }
        if !spec_integrity {
            qualification_reasons.push("benchmark specification digest did not verify".to_string());
        }
        if !artifact_integrity {
            qualification_reasons.push("one or more declared artifact digests did not verify".to_string());
        }
        if !head_matches {
            qualification_reasons.push("checkout HEAD does not match the recorded code commit".to_string());
        }
        if !remote_matches {
            qualification_reasons.push("origin does not match the recorded repository identity".to_string());
        }
        if worktree_clean != Some(true)
            || string_field(&receipt, &["code", "worktree_state"]) != Some("clean")
        {
            qualification_reasons.push("checkout is not demonstrably clean".to_string());
        }

        if !completed_successfully(&receipt) {
            qualification_reasons.push(
                "run must be completed with exit code zero and a recorded finish time".to_string(),
            );
        }
        if !all_gates_and_controls_pass(&receipt) {
            qualification_reasons.push(
                "all predeclared gates and negative controls must be present and pass".to_string(),
            );
        }
        if !metrics_present(&receipt) {
            qualification_reasons.push("at least one metric result with sample count and uncertainty method is required".to_string());
        }
        if !ci_passed {
            qualification_reasons.push("required CI checks must be completed successes on the exact code commit".to_string());
        }
        if !configuration_digest_is_bound(&receipt, &artifact_checks) {
            qualification_reasons.push(
                "configuration SHA-256 must match a verified configuration artifact".to_string(),
            );
        }
        match target_status.as_str() {
            "qualified_synthetic" => {
                if !synthetic_artifacts_and_controls_valid(&receipt, &artifact_checks) {
                    qualification_reasons.push(
                        "synthetic qualification requires withheld truth, synthetic observations, and passing null-signal/truth-leakage controls".to_string(),
                    );
                }
            }
            "source_adapter_qualified" => {
                if !source_adapter_valid(&receipt, &artifact_checks) {
                    qualification_reasons.push(
                        "source adapter needs verified payload bytes and passed variable-shape, units, and time-axis audits".to_string(),
                    );
                }
            }
            "real_observation_evaluated" => {
                if !source_adapter_valid(&receipt, &artifact_checks)
                    || !real_observation_valid(&receipt, &artifact_checks)
                {
                    qualification_reasons.push(
                        "real-observation evaluation needs a qualified source adapter, verified real data and holdout manifest, and passed leakage/overlap audits".to_string(),
                    );
                }
            }
            _ => {}
        }
    }

    let eligible = target_status == "executed_unqualified" || qualification_reasons.is_empty();
    let mut verification_outcome = if schema_valid && spec_integrity && artifact_integrity {
        "accepted".to_string()
    } else {
        "rejected".to_string()
    };

    if target_status != "executed_unqualified" && !eligible {
        verification_outcome = "rejected".to_string();
        findings.extend(qualification_reasons.iter().cloned());
    }

    if target_status != "executed_unqualified" && eligible {
        let input_is_promoted = input_status.as_deref() == Some(target_status.as_str());
        if input_is_promoted {
            match validate_prior_verifier_report(
                args.verifier_report_path.as_deref(),
                &repo_root,
                &receipt,
                &target_status,
                core_sha256.as_deref(),
            ) {
                Ok(()) => {
                    verification_outcome = "accepted".to_string();
                    push_check(
                        &mut checks,
                        "prior_verifier_report",
                        true,
                        "recorded verifier-report digest and core binding verified".to_string(),
                    );
                }
                Err(error) => {
                    verification_outcome = "incomplete".to_string();
                    findings.push(error.clone());
                    push_check(&mut checks, "prior_verifier_report", false, error);
                }
            }
        } else {
            if args.verifier_report_path.is_some() {
                let error = "a verifier report can only attest a receipt whose qualification status has been promoted to its target".to_string();
                verification_outcome = "rejected".to_string();
                findings.push(error.clone());
                push_check(&mut checks, "prior_verifier_report", false, error);
            } else {
                push_check(
                    &mut checks,
                    "promotion_prerequisites",
                    true,
                    "all local promotion prerequisites pass; this provisional report can be independently reviewed and bound to the final receipt".to_string(),
                );
            }
        }
    } else if target_status == "executed_unqualified" {
        push_check(
            &mut checks,
            "qualification_claim",
            true,
            "receipt remains explicitly unqualified; no benchmark pass is inferred".to_string(),
        );
    }

    if target_status != "executed_unqualified" && eligible && input_status.as_deref() == Some(target_status.as_str()) {
        // Reaching this point means a final promoted receipt has an accepted,
        // digest-matched prior verifier report. The schema itself does not
        // authenticate the person or organization operating that verifier.
        if string_field(&receipt, &["qualification_evidence", "verification_outcome"]) != Some("accepted") {
            verification_outcome = "incomplete".to_string();
            findings.push("qualification_evidence.verification_outcome is not accepted".to_string());
        }
    }

    if !spec_integrity || !artifact_integrity || !schema_valid {
        verification_outcome = "rejected".to_string();
    }

    Ok(VerificationReport {
        report_schema_version: REPORT_SCHEMA_VERSION,
        verifier_name: "geo-receipt-verifier".to_string(),
        verifier_version: env!("CARGO_PKG_VERSION").to_string(),
        verifier_commit_sha,
        qualification_core_sha256: core_sha256,
        input_receipt_status: input_status,
        target_qualification_status: Some(target_status),
        schema_valid,
        schema_errors,
        benchmark_specification_integrity: spec_integrity,
        artifact_integrity,
        artifact_checks,
        repository_head_sha: actual_head,
        repository_worktree_clean: worktree_clean,
        repository_identity_matches: remote_matches,
        checks,
        qualification_eligible: eligible,
        verification_outcome,
        findings,
        claim_ceiling: "This report verifies receipt structure, declared local file digests, exact checkout identity, and the declared benchmark protocol. It does not prove physical-model validity, publisher authenticity, or truth about the real Earth.".to_string(),
    })
}

fn resolve_cli_input(root: &Path, raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return fs::canonicalize(&path)
            .map_err(|error| format!("cannot resolve input {}: {error}", path.display()));
    }
    safe_join(root, raw)
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("path must be relative and cannot traverse its root: {relative}"));
    }
    let root = fs::canonicalize(root)
        .map_err(|error| format!("cannot canonicalize root {}: {error}", root.display()))?;
    let joined = fs::canonicalize(root.join(path))
        .map_err(|error| format!("cannot resolve {relative}: {error}"))?;
    if !joined.starts_with(&root) {
        return Err(format!("resolved path escapes its root: {relative}"));
    }
    if !joined.is_file() {
        return Err(format!("path is not a regular file: {relative}"));
    }
    Ok(joined)
}

fn hash_file(path: &Path) -> Result<(String, u64), String> {
    let mut file = File::open(path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        length = length
            .checked_add(count as u64)
            .ok_or_else(|| format!("file length overflow while reading {}", path.display()))?;
    }
    Ok((format!("{:x}", hasher.finalize()), length))
}

fn qualification_core_sha256(receipt: &Value) -> Result<String, String> {
    let mut core = receipt.clone();
    let object = core
        .as_object_mut()
        .ok_or_else(|| "receipt root must be a JSON object".to_string())?;
    object.remove("qualification_evidence");
    object.insert(
        "qualification_status".to_string(),
        Value::String("executed_unqualified".to_string()),
    );
    let canonical = canonical_json_value(&core);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| format!("cannot canonicalize receipt core: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn canonical_json_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<String, Value> = object
                .iter()
                .map(|(key, value)| (key.clone(), canonical_json_value(value)))
                .collect();
            let mut result = serde_json::Map::new();
            for (key, value) in sorted {
                result.insert(key, value);
            }
            Value::Object(result)
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(canonical_json_value).collect())
        }
        scalar => scalar.clone(),
    }
}

fn verify_benchmark_specification(
    repo_root: &Path,
    receipt: &Value,
    checks: &mut Vec<Check>,
    findings: &mut Vec<String>,
) -> bool {
    let path_value = string_field(receipt, &["benchmark_specification", "path"]);
    let expected = string_field(receipt, &["benchmark_specification", "sha256"]);
    let result = match (path_value, expected) {
        (Some(path), Some(expected)) => match safe_join(repo_root, path) {
            Ok(path) => match hash_file(&path) {
                Ok((actual, _)) if actual.eq_ignore_ascii_case(expected) => Ok(()),
                Ok((actual, _)) => Err(format!(
                    "benchmark specification digest mismatch: expected {expected}, actual {actual}"
                )),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        },
        _ => Err("benchmark_specification.path or sha256 is missing".to_string()),
    };
    let passed = result.is_ok();
    if let Err(error) = result.as_ref() {
        findings.push(error.clone());
    }
    push_check(
        checks,
        "benchmark_specification_digest",
        passed,
        result.err().unwrap_or_else(|| "benchmark specification SHA-256 matches".to_string()),
    );
    passed
}

fn verify_artifacts(
    artifact_root: &Path,
    receipt: &Value,
    findings: &mut Vec<String>,
) -> Vec<ArtifactCheck> {
    let Some(artifacts) = receipt.get("artifacts").and_then(Value::as_array) else {
        findings.push("artifacts field is missing or not an array".to_string());
        return Vec::new();
    };

    artifacts
        .iter()
        .enumerate()
        .map(|(index, artifact)| {
            let id = string_field(artifact, &["artifact_id"])
                .map(str::to_owned)
                .unwrap_or_else(|| format!("artifact-{index}"));
            let role = string_field(artifact, &["role"])
                .map(str::to_owned)
                .unwrap_or_else(|| "<missing>".to_string());
            let locator = string_field(artifact, &["locator"])
                .map(str::to_owned)
                .unwrap_or_else(|| "<missing>".to_string());
            let expected_sha = string_field(artifact, &["sha256"]).map(str::to_owned);
            let expected_len = artifact.get("byte_length").and_then(Value::as_u64);

            let calculated = safe_join(artifact_root, &locator)
                .and_then(|path| hash_file(&path));
            let (actual_sha, actual_len, result) = match calculated {
                Ok((sha, length)) => {
                    let digest_ok = expected_sha
                        .as_deref()
                        .is_some_and(|expected| sha.eq_ignore_ascii_case(expected));
                    let length_ok = expected_len == Some(length);
                    let detail = if digest_ok && length_ok {
                        "SHA-256 and byte length match".to_string()
                    } else {
                        format!(
                            "mismatch: expected SHA-256 {:?}, actual {sha}; expected length {:?}, actual {length}",
                            expected_sha, expected_len
                        )
                    };
                    (Some(sha), Some(length), if digest_ok && length_ok { Ok(()) } else { Err(detail) })
                }
                Err(error) => (None, None, Err(error)),
            };
            let passed = result.is_ok();
            if let Err(error) = result.as_ref() {
                findings.push(format!("artifact {id}: {error}"));
            }
            ArtifactCheck {
                artifact_id: id,
                role,
                locator,
                expected_sha256: expected_sha,
                actual_sha256: actual_sha,
                expected_byte_length: expected_len,
                actual_byte_length: actual_len,
                passed,
                detail: result.err().unwrap_or_else(|| "verified".to_string()),
            }
        })
        .collect()
}

fn completed_successfully(receipt: &Value) -> bool {
    string_field(receipt, &["run", "outcome"]) == Some("completed")
        && receipt.get("run").and_then(|run| run.get("exit_code")).and_then(Value::as_i64) == Some(0)
        && receipt.get("run").and_then(|run| run.get("finished_at")).and_then(Value::as_str).is_some()
}

fn all_gates_and_controls_pass(receipt: &Value) -> bool {
    let gates = receipt.get("gates").and_then(Value::as_array);
    let controls = receipt.get("controls").and_then(Value::as_array);
    matches!(gates, Some(values) if !values.is_empty() && values.iter().all(|value| string_field(value, &["decision"]) == Some("passed")))
        && matches!(controls, Some(values) if !values.is_empty() && values.iter().all(|value| string_field(value, &["result"]) == Some("passed")))
}

fn metrics_present(receipt: &Value) -> bool {
    matches!(
        receipt.get("metrics").and_then(Value::as_array),
        Some(metrics)
            if !metrics.is_empty()
                && metrics.iter().all(|metric| {
                    metric.get("sample_count").and_then(Value::as_u64).is_some_and(|count| count > 0)
                        && string_field(metric, &["uncertainty_method"]).is_some_and(|method| method != "none")
                })
    )
}

fn ci_checks_all_pass(receipt: &Value) -> bool {
    let Some(checks) = receipt.get("ci_checks").and_then(Value::as_array) else {
        return false;
    };
    let Some(code_commit) = string_field(receipt, &["code", "commit_sha"]) else {
        return false;
    };
    !checks.is_empty()
        && checks.iter().all(|check| {
            string_field(check, &["commit_sha"]) == Some(code_commit)
                && string_field(check, &["status"]) == Some("completed")
                && string_field(check, &["conclusion"]) == Some("success")
        })
}

fn configuration_digest_is_bound(receipt: &Value, artifacts: &[ArtifactCheck]) -> bool {
    let expected = string_field(receipt, &["configuration", "sha256"]);
    expected.is_some_and(|expected| {
        artifacts.iter().any(|artifact| {
            artifact.passed
                && artifact.role == "configuration"
                && artifact
                    .actual_sha256
                    .as_deref()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
        })
    })
}

fn synthetic_artifacts_and_controls_valid(receipt: &Value, artifacts: &[ArtifactCheck]) -> bool {
    let truth_hidden = receipt
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|record| {
            let Some(id) = string_field(record, &["artifact_id"]) else {
                return false;
            };
            string_field(record, &["role"]) == Some("truth_state")
                && string_field(record, &["selector_visibility"])
                    == Some("withheld_until_scoring")
                && artifacts.iter().any(|checked| {
                    checked.artifact_id == id
                        && checked.role == "truth_state"
                        && checked.passed
                })
        });
    let synthetic_observations = artifacts
        .iter()
        .any(|artifact| artifact.passed && artifact.role == "synthetic_observations");
    let controls = receipt.get("controls").and_then(Value::as_array);
    let has_passing_control = |kind: &str| {
        matches!(controls, Some(values) if values.iter().any(|value| string_field(value, &["kind"]) == Some(kind) && string_field(value, &["result"]) == Some("passed")))
    };
    truth_hidden
        && synthetic_observations
        && has_passing_control("null_signal")
        && has_passing_control("truth_leakage")
}

fn source_adapter_valid(receipt: &Value, artifacts: &[ArtifactCheck]) -> bool {
    let audit_ok = ["units_audit", "time_axis_audit", "variable_shape_audit"]
        .iter()
        .all(|field| string_field(receipt, &["source_adapter", field]) == Some("passed"));
    let digest = string_field(receipt, &["source_adapter", "local_sha256"]);
    audit_ok
        && artifacts.iter().any(|artifact| {
            artifact.passed
                && artifact.role == "source_payload"
                && digest.is_some_and(|expected| {
                    artifact
                        .actual_sha256
                        .as_deref()
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
                })
        })
}

fn real_observation_valid(receipt: &Value, artifacts: &[ArtifactCheck]) -> bool {
    let leakage_ok = string_field(
        receipt,
        &["real_observation_evaluation", "leakage_audit"],
    ) == Some("passed");
    let overlap_ok = string_field(
        receipt,
        &["real_observation_evaluation", "reference_overlap_audit"],
    ) == Some("passed");
    let holdout_digest = string_field(
        receipt,
        &["real_observation_evaluation", "holdout_manifest_sha256"],
    );
    let has_holdout = artifacts.iter().any(|artifact| {
        artifact.passed
            && artifact.role == "holdout_manifest"
            && holdout_digest.is_some_and(|expected| {
                artifact
                    .actual_sha256
                    .as_deref()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
            })
    });
    let has_real_data = artifacts
        .iter()
        .any(|artifact| artifact.passed && artifact.role == "real_observations");
    leakage_ok && overlap_ok && has_holdout && has_real_data
}

fn validate_prior_verifier_report(
    report_path: Option<&Path>,
    repo_root: &Path,
    receipt: &Value,
    target_status: &str,
    core_sha256: Option<&str>,
) -> Result<(), String> {
    let path = report_path.ok_or_else(|| {
        "promoted receipt cannot be accepted without --verifier-report pointing to the exact verifier result".to_string()
    })?;
    let path = resolve_cli_input(repo_root, &path.to_string_lossy())?;
    let (actual_sha, _) = hash_file(&path)?;
    let expected_sha = string_field(
        receipt,
        &["qualification_evidence", "verifier_result_sha256"],
    )
    .ok_or_else(|| "qualification_evidence.verifier_result_sha256 is missing".to_string())?;
    if !actual_sha.eq_ignore_ascii_case(expected_sha) {
        return Err(format!(
            "verifier report digest mismatch: receipt expects {expected_sha}, file hashes to {actual_sha}"
        ));
    }
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read verifier report {}: {error}", path.display()))?;
    let report: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("verifier report is not valid JSON: {error}"))?;
    if string_field(&report, &["verification_outcome"]) != Some("accepted") {
        return Err("referenced verifier report is not accepted".to_string());
    }
    if report.get("qualification_eligible").and_then(Value::as_bool) != Some(true) {
        return Err("referenced verifier report does not establish qualification eligibility".to_string());
    }
    if report.get("schema_valid").and_then(Value::as_bool) != Some(true)
        || report.get("benchmark_specification_integrity").and_then(Value::as_bool) != Some(true)
        || report.get("artifact_integrity").and_then(Value::as_bool) != Some(true)
    {
        return Err("referenced verifier report has a failed schema or integrity check".to_string());
    }
    if string_field(&report, &["target_qualification_status"]) != Some(target_status) {
        return Err("referenced verifier report targets a different qualification status".to_string());
    }
    if string_field(&report, &["qualification_core_sha256"]) != core_sha256 {
        return Err("referenced verifier report is bound to a different qualification core".to_string());
    }
    let evidence_commit = string_field(
        receipt,
        &["qualification_evidence", "verifier_commit_sha"],
    )
    .ok_or_else(|| "qualification_evidence.verifier_commit_sha is missing".to_string())?;
    if string_field(&report, &["verifier_commit_sha"]) != Some(evidence_commit) {
        return Err("referenced verifier report commit does not match qualification evidence".to_string());
    }
    if string_field(&receipt, &["qualification_evidence", "verification_outcome"]) != Some("accepted") {
        return Err("qualification evidence outcome is not accepted".to_string());
    }
    Ok(())
}

fn git_snapshot(root: &Path) -> (Option<String>, Option<bool>, Option<String>) {
    let head = git_output(root, &["rev-parse", "HEAD"]);
    let status = git_output(root, &["status", "--porcelain", "--untracked-files=all"]);
    let remote = git_output(root, &["remote", "get-url", "origin"]);
    (
        head,
        status.map(|value| value.trim().is_empty()),
        remote,
    )
}

fn git_output(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
}

fn normalize_git_url(url: &str) -> String {
    let mut normalized = url.trim().trim_end_matches('/').to_string();
    if let Some(rest) = normalized.strip_prefix("git@github.com:") {
        normalized = format!("https://github.com/{rest}");
    } else if let Some(rest) = normalized.strip_prefix("ssh://git@github.com/") {
        normalized = format!("https://github.com/{rest}");
    }
    if let Some(rest) = normalized.strip_suffix(".git") {
        normalized = rest.to_string();
    }
    normalized.trim_end_matches('/').to_ascii_lowercase()
}

fn push_check(checks: &mut Vec<Check>, id: &str, passed: bool, detail: String) {
    checks.push(Check {
        id: id.to_string(),
        passed,
        detail,
    });
}

fn string_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const RECEIPT_SCHEMA: &str =
        include_str!("../../../docs/earth-interior/benchmark-run-receipt.schema.json");

    #[test]
    fn embedded_receipt_schema_is_valid_draft_2020_12() {
        let schema: Value = serde_json::from_str(RECEIPT_SCHEMA).expect("schema JSON parses");
        assert!(draft202012::meta::validate(&schema).is_ok());
        assert!(draft202012::options()
            .should_validate_formats(true)
            .build(&schema)
            .is_ok());
    }

    #[test]
    fn minimal_unqualified_receipt_matches_schema() {
        let schema: Value = serde_json::from_str(RECEIPT_SCHEMA).expect("schema JSON parses");
        let validator = draft202012::options()
            .should_validate_formats(true)
            .build(&schema)
            .expect("schema compiles");
        let receipt = json!({
            "schema_version": 1,
            "receipt_id": "test-unqualified-001",
            "benchmark_id": "geo-test-001",
            "benchmark_specification": {
                "path": "docs/earth-interior/benchmark-resolution-frontier.json",
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "git_commit_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            },
            "run": {
                "run_id": "test-run-001",
                "started_at": "2026-10-17T00:00:00Z",
                "finished_at": null,
                "outcome": "interrupted",
                "command": "fixture-only",
                "exit_code": null,
                "runner_id": null
            },
            "code": {
                "repository_url": "https://github.com/Luminous-Dynamics/sol-atlas-leptos",
                "commit_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "worktree_state": "unknown",
                "toolchain": "not-run"
            },
            "configuration": {
                "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "canonicalization": "fixture",
                "randomness": {
                    "mode": "deterministic_no_rng",
                    "algorithm": null,
                    "seeds": []
                }
            },
            "environment": {
                "operating_system": "test",
                "architecture": "test",
                "locale": "C",
                "timezone": "UTC"
            },
            "qualification_status": "executed_unqualified",
            "ci_checks": [],
            "artifacts": [{
                "artifact_id": "fixture-config",
                "role": "configuration",
                "locator": "config.json",
                "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "byte_length": 2,
                "media_type": "application/json",
                "selector_visibility": "not_applicable"
            }],
            "metrics": [],
            "controls": [],
            "gates": [],
            "claim_boundary": {
                "permitted_claim": "fixture only",
                "forbidden_claims": ["not a result"]
            }
        });
        assert!(validator.is_valid(&receipt), "{:?}", validator.iter_errors(&receipt).map(|e| e.to_string()).collect::<Vec<_>>());
    }

    #[test]
    fn qualification_core_digest_ignores_attestation_and_promotion_status() {
        let provisional = json!({
            "receipt_id": "run-1",
            "qualification_status": "executed_unqualified",
            "metrics": [{"name": "rmse", "value": 1.0}]
        });
        let promoted = json!({
            "receipt_id": "run-1",
            "qualification_status": "qualified_synthetic",
            "metrics": [{"name": "rmse", "value": 1.0}],
            "qualification_evidence": {
                "verifier_result_sha256": "b".repeat(64),
                "verification_outcome": "accepted"
            }
        });
        assert_eq!(
            qualification_core_sha256(&provisional).unwrap(),
            qualification_core_sha256(&promoted).unwrap()
        );
    }

    #[test]
    fn qualification_core_digest_changes_when_metric_changes() {
        let first = json!({
            "qualification_status": "executed_unqualified",
            "metrics": [{"name": "rmse", "value": 1.0}]
        });
        let changed = json!({
            "qualification_status": "qualified_synthetic",
            "metrics": [{"name": "rmse", "value": 1.1}],
            "qualification_evidence": {"verification_outcome": "accepted"}
        });
        assert_ne!(
            qualification_core_sha256(&first).unwrap(),
            qualification_core_sha256(&changed).unwrap()
        );
    }

    #[test]
    fn safe_join_rejects_parent_traversal() {
        let root = env::current_dir().unwrap();
        assert!(safe_join(&root, "../Cargo.toml").is_err());
        assert!(safe_join(&root, "/etc/passwd").is_err());
    }

    #[test]
    fn repository_url_normalization_supports_https_and_ssh_git_forms() {
        assert_eq!(
            normalize_git_url("https://github.com/Luminous-Dynamics/sol-atlas-leptos.git"),
            normalize_git_url("git@github.com:Luminous-Dynamics/sol-atlas-leptos")
        );
    }

    #[test]
    fn controls_and_gates_fail_closed_when_empty_or_not_passed() {
        assert!(!all_gates_and_controls_pass(&json!({"gates": [], "controls": []})));
        assert!(!all_gates_and_controls_pass(&json!({
            "gates": [{"decision":"passed"}],
            "controls": [{"result":"not_evaluated"}]
        })));
        assert!(all_gates_and_controls_pass(&json!({
            "gates": [{"decision":"passed"}],
            "controls": [{"result":"passed"}]
        })));
    }
}
