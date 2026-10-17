// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This verifier checks evidence protocol and integrity. It does not prove that a
// geophysical model or interpretation is physically correct.

use jsonschema::draft202012;
use serde::Serialize;
use serde_json::{Value, json};
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
    verifier_root: Option<PathBuf>,
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
    verifier_repository_url: Option<String>,
    verifier_worktree_clean: Option<bool>,
    verifier_checkout_independent: bool,
    verifier_build_matches_checkout: bool,
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
            } else if outcome == "incomplete"
                || outcome == "rejected"
                || (!eligible
                    && report.target_qualification_status.as_deref()
                        != Some("executed_unqualified"))
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
    let mut verifier_root = None;
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
            "--verifier-root" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                verifier_root = Some(PathBuf::from(value));
            }
            "--target-status" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                let valid =
                    value == "executed_unqualified" || QUALIFIED_STATUSES.contains(&value.as_str());
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
        verifier_root,
        schema_path,
        receipt_path,
        artifact_root,
        target_status,
        verifier_report_path,
    })
}

fn usage() -> String {
    "usage: geo-receipt-verifier <repo-root> <schema-path> <receipt-path> \
     [--artifact-root PATH] [--verifier-root PATH] \
     [--target-status STATUS] [--verifier-report PATH]"
        .to_string()
}

fn verify(args: Args) -> Result<VerificationReport, String> {
    let repo_root = fs::canonicalize(&args.repo_root)
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    let verifier_root_candidate = match args.verifier_root.as_ref() {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.clone(),
    };
    let verifier_root = fs::canonicalize(&verifier_root_candidate)
        .map_err(|error| format!("cannot resolve verifier root: {error}"))?;
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
    let (verifier_head, verifier_worktree_clean, verifier_repository_url) =
        git_snapshot(&verifier_root);
    let verifier_build_matches_checkout =
        compiled_verifier_checkout_root().is_ok_and(|compiled_root| compiled_root == verifier_root);
    let verifier_checkout_independent = independent_verifier_checkout(
        &repo_root,
        actual_head.as_deref(),
        &verifier_root,
        verifier_head.as_deref(),
        verifier_worktree_clean,
        verifier_repository_url.as_deref(),
    );
    let verifier_commit_sha = verifier_head.clone();

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

    let spec_integrity =
        verify_benchmark_specification(&repo_root, &receipt, &mut checks, &mut findings);
    let artifact_checks = verify_artifacts(&artifact_root, &receipt, &mut findings);
    let artifact_integrity =
        !artifact_checks.is_empty() && artifact_checks.iter().all(|artifact| artifact.passed);
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
    push_check(
        &mut checks,
        "independent_verifier_checkout",
        verifier_checkout_independent,
        if verifier_checkout_independent {
            "verifier uses a separate clean checkout at a different exact commit".to_string()
        } else {
            "promotion requires a separate clean verifier checkout at a different commit with a known origin".to_string()
        },
    );
    push_check(
        &mut checks,
        "verifier_build_matches_checkout",
        verifier_build_matches_checkout,
        if verifier_build_matches_checkout {
            "compiled binary manifest directory resolves to the declared verifier checkout"
                .to_string()
        } else {
            "compiled binary was not built from the declared verifier checkout".to_string()
        },
    );
    let run_passed = completed_successfully(&receipt);
    let gates_controls_passed = all_gates_and_controls_pass(&receipt);
    let metrics_passed = metrics_present(&receipt);
    let config_digest_passed = configuration_digest_is_bound(&receipt, &artifact_checks);
    let ci_passed = ci_checks_all_pass(&receipt, &artifact_checks, &artifact_root, &repo_root);
    push_check(
        &mut checks,
        "completed_run",
        run_passed,
        "run outcome, exit code, and finished_at must prove successful completion".to_string(),
    );
    push_check(
        &mut checks,
        "gates_and_controls",
        gates_controls_passed,
        "all non-empty predeclared gates and negative controls must pass".to_string(),
    );
    push_check(&mut checks, "metrics_and_uncertainty", metrics_passed, "metric records must carry a positive sample count and an explicit uncertainty method other than none".to_string());
    push_check(
        &mut checks,
        "configuration_artifact_binding",
        config_digest_passed,
        "configuration SHA-256 must equal the verified configuration artifact digest".to_string(),
    );
    push_check(
        &mut checks,
        "exact_head_ci",
        ci_passed,
        if ci_passed {
            "all declared required checks completed successfully on the recorded code commit"
                .to_string()
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
            qualification_reasons
                .push("one or more declared artifact digests did not verify".to_string());
        }
        if !head_matches {
            qualification_reasons
                .push("checkout HEAD does not match the recorded code commit".to_string());
        }
        if !verifier_checkout_independent {
            qualification_reasons.push(
                "promotion requires a separate clean verifier checkout with a distinct exact commit and recorded origin".to_string(),
            );
        }
        if !verifier_build_matches_checkout {
            qualification_reasons.push(
                "the executing verifier binary must be built from the declared separate verifier checkout".to_string(),
            );
        }
        if !remote_matches {
            qualification_reasons
                .push("origin does not match the recorded repository identity".to_string());
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
            qualification_reasons.push(
                "at least one metric result with sample count and uncertainty method is required"
                    .to_string(),
            );
        }
        if !ci_passed {
            qualification_reasons.push(
                "required CI checks must be completed successes on the exact code commit"
                    .to_string(),
            );
        }
        if !config_digest_passed {
            qualification_reasons.push(
                "configuration SHA-256 must match a verified configuration artifact".to_string(),
            );
        }
        match target_status.as_str() {
            "qualified_synthetic" => {
                let passed = synthetic_artifacts_and_controls_valid(&receipt, &artifact_checks);
                push_check(
                    &mut checks,
                    "synthetic_evidence_integrity",
                    passed,
                    "verified truth and synthetic-observation artifacts plus passing null-signal and truth-leakage controls are required".to_string(),
                );
                if !passed {
                    qualification_reasons.push(
                        "synthetic qualification requires withheld truth, synthetic observations, and passing null-signal/truth-leakage controls".to_string(),
                    );
                }
            }
            "source_adapter_qualified" => {
                let passed = source_adapter_valid(&receipt, &artifact_checks);
                push_check(
                    &mut checks,
                    "source_adapter_integrity",
                    passed,
                    "source payload SHA-256, variable-shape, units, and time-axis audits must all pass".to_string(),
                );
                if !passed {
                    qualification_reasons.push(
                        "source adapter needs verified payload bytes and passed variable-shape, units, and time-axis audits".to_string(),
                    );
                }
            }
            "real_observation_evaluated" => {
                let adapter_passed = source_adapter_valid(&receipt, &artifact_checks);
                let real_passed = real_observation_valid(&receipt, &artifact_checks);
                push_check(
                    &mut checks,
                    "source_adapter_integrity",
                    adapter_passed,
                    "source payload SHA-256, variable-shape, units, and time-axis audits must all pass".to_string(),
                );
                push_check(
                    &mut checks,
                    "real_observation_holdout_integrity",
                    real_passed,
                    "real data and frozen holdout manifests must hash correctly, with passed leakage and reference-overlap audits".to_string(),
                );
                if !adapter_passed || !real_passed {
                    qualification_reasons.push(
                        "real-observation evaluation needs a qualified source adapter, verified real data and holdout manifest, and passed leakage/overlap audits".to_string(),
                    );
                }
            }
            _ => {}
        }
    }

    let eligible = target_status != "executed_unqualified" && qualification_reasons.is_empty();
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
                &verifier_root,
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

    if target_status != "executed_unqualified"
        && eligible
        && input_status.as_deref() == Some(target_status.as_str())
    {
        // Reaching this point means a final promoted receipt has an accepted,
        // digest-matched prior verifier report. The schema itself does not
        // authenticate the person or organization operating that verifier.
        if string_field(
            &receipt,
            &["qualification_evidence", "verification_outcome"],
        ) != Some("accepted")
        {
            verification_outcome = "incomplete".to_string();
            findings
                .push("qualification_evidence.verification_outcome is not accepted".to_string());
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
        verifier_repository_url,
        verifier_worktree_clean,
        verifier_checkout_independent,
        verifier_build_matches_checkout,
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
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "path must be relative and cannot traverse its root: {relative}"
        ));
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
    let mut file =
        File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
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
        Value::Array(values) => Value::Array(values.iter().map(canonical_json_value).collect()),
        scalar => scalar.clone(),
    }
}

fn benchmark_specification_commit_matches(receipt: &Value) -> bool {
    string_field(receipt, &["benchmark_specification", "git_commit_sha"])
        .zip(string_field(receipt, &["code", "commit_sha"]))
        .is_some_and(|(spec_commit, code_commit)| spec_commit == code_commit)
}

fn verify_benchmark_specification(
    repo_root: &Path,
    receipt: &Value,
    checks: &mut Vec<Check>,
    findings: &mut Vec<String>,
) -> bool {
    let spec_commit_matches_code = benchmark_specification_commit_matches(receipt);
    push_check(
        checks,
        "specification_commit_identity",
        spec_commit_matches_code,
        if spec_commit_matches_code {
            "benchmark specification commit matches the code commit".to_string()
        } else {
            "benchmark specification git_commit_sha does not match code.commit_sha".to_string()
        },
    );
    if !spec_commit_matches_code {
        findings.push(
            "benchmark specification must be bound to the same exact Git commit as the code under evaluation".to_string(),
        );
    }

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
    let digest_matches = result.is_ok();
    if let Err(error) = result.as_ref() {
        findings.push(error.clone());
    }
    push_check(
        checks,
        "benchmark_specification_digest",
        digest_matches,
        result
            .err()
            .unwrap_or_else(|| "benchmark specification SHA-256 matches".to_string()),
    );
    spec_commit_matches_code && digest_matches
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
        && receipt
            .get("run")
            .and_then(|run| run.get("exit_code"))
            .and_then(Value::as_i64)
            == Some(0)
        && receipt
            .get("run")
            .and_then(|run| run.get("finished_at"))
            .and_then(Value::as_str)
            .is_some()
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

fn ci_checks_all_pass(
    receipt: &Value,
    artifacts: &[ArtifactCheck],
    artifact_root: &Path,
    repo_root: &Path,
) -> bool {
    let Some(checks) = receipt.get("ci_checks").and_then(Value::as_array) else {
        return false;
    };
    let Some(code_commit) = string_field(receipt, &["code", "commit_sha"]) else {
        return false;
    };
    let Some(spec_path) = string_field(receipt, &["benchmark_specification", "path"]) else {
        return false;
    };
    let Ok(spec_path) = safe_join(repo_root, spec_path) else {
        return false;
    };
    let Ok(spec_bytes) = fs::read(spec_path) else {
        return false;
    };
    let Ok(specification) = serde_json::from_slice::<Value>(&spec_bytes) else {
        return false;
    };
    let Some(required_ci_checks) = specification
        .get("required_ci_checks")
        .and_then(Value::as_array)
    else {
        return false;
    };
    let required_names = required_ci_checks
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>();
    let Some(required_names) = required_names else {
        return false;
    };
    let required_names: Vec<String> = required_names.into_iter().map(str::to_owned).collect();
    if !ci_check_names_match_policy(checks, &required_names) {
        return false;
    }

    let mut seen_check_ids = std::collections::HashSet::new();
    checks.iter().all(|check| {
        let Some(check_id) = string_field(check, &["check_id"]) else {
            return false;
        };
        if !seen_check_ids.insert(check_id.to_string())
            || string_field(check, &["commit_sha"]) != Some(code_commit)
            || string_field(check, &["status"]) != Some("completed")
            || string_field(check, &["conclusion"]) != Some("success")
        {
            return false;
        }
        let Some(evidence_id) = string_field(check, &["evidence_artifact_id"]) else {
            return false;
        };
        let evidence_is_hashed = artifacts.iter().any(|artifact| {
            artifact.artifact_id == evidence_id
                && artifact.role == "ci_check_evidence"
                && artifact.passed
        });
        if !evidence_is_hashed {
            return false;
        }
        let Some(record) = receipt
            .get("artifacts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|record| string_field(record, &["artifact_id"]) == Some(evidence_id))
        else {
            return false;
        };
        let Some(locator) = string_field(record, &["locator"]) else {
            return false;
        };
        let Ok(path) = safe_join(artifact_root, locator) else {
            return false;
        };
        let Ok(bytes) = fs::read(path) else {
            return false;
        };
        let Ok(payload) = serde_json::from_slice::<Value>(&bytes) else {
            return false;
        };
        ci_check_evidence_matches(check, &payload, code_commit)
    })
}

fn ci_check_names_match_policy(checks: &[Value], required_names: &[String]) -> bool {
    if checks.is_empty() || required_names.is_empty() || checks.len() != required_names.len() {
        return false;
    }
    let required_set: std::collections::HashSet<&str> =
        required_names.iter().map(String::as_str).collect();
    if required_set.len() != required_names.len() {
        return false;
    }
    let mut seen_names = std::collections::HashSet::new();
    checks.iter().all(|check| {
        let Some(name) = string_field(check, &["name"]) else {
            return false;
        };
        required_set.contains(name) && seen_names.insert(name)
    }) && seen_names.len() == required_set.len()
}

fn ci_check_evidence_matches(check: &Value, evidence: &Value, expected_commit: &str) -> bool {
    let check_id = string_field(check, &["check_id"]);
    let evidence_id = evidence.get("id").and_then(|value| {
        value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_u64().map(|id| id.to_string()))
    });
    check_id.is_some_and(|id| evidence_id.as_deref() == Some(id))
        && string_field(check, &["name"]) == string_field(evidence, &["name"])
        && string_field(check, &["commit_sha"]) == Some(expected_commit)
        && string_field(evidence, &["head_sha"]) == Some(expected_commit)
        && string_field(check, &["status"]) == Some("completed")
        && string_field(check, &["conclusion"]) == Some("success")
        && string_field(evidence, &["status"]) == Some("completed")
        && string_field(evidence, &["conclusion"]) == Some("success")
        && string_field(check, &["url"]) == string_field(evidence, &["html_url"])
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
                && string_field(record, &["selector_visibility"]) == Some("withheld_until_scoring")
                && artifacts.iter().any(|checked| {
                    checked.artifact_id == id && checked.role == "truth_state" && checked.passed
                })
        });
    let synthetic_observations = artifacts
        .iter()
        .any(|artifact| artifact.passed && artifact.role == "synthetic_observations");
    let controls = receipt.get("controls").and_then(Value::as_array);
    let has_passing_control = |kind: &str| matches!(controls, Some(values) if values.iter().any(|value| string_field(value, &["kind"]) == Some(kind) && string_field(value, &["result"]) == Some("passed")));
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
    let leakage_ok =
        string_field(receipt, &["real_observation_evaluation", "leakage_audit"]) == Some("passed");
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
    verifier_root: &Path,
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
    if report
        .get("qualification_eligible")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err(
            "referenced verifier report does not establish qualification eligibility".to_string(),
        );
    }
    if report.get("schema_valid").and_then(Value::as_bool) != Some(true)
        || report
            .get("benchmark_specification_integrity")
            .and_then(Value::as_bool)
            != Some(true)
        || report.get("artifact_integrity").and_then(Value::as_bool) != Some(true)
    {
        return Err(
            "referenced verifier report has a failed schema or integrity check".to_string(),
        );
    }
    if string_field(&report, &["target_qualification_status"]) != Some(target_status) {
        return Err(
            "referenced verifier report targets a different qualification status".to_string(),
        );
    }
    let mut required_checks = vec![
        "receipt_schema",
        "specification_commit_identity",
        "benchmark_specification_digest",
        "declared_artifact_integrity",
        "exact_git_head",
        "repository_identity",
        "clean_worktree",
        "exact_head_ci",
        "independent_verifier_checkout",
        "verifier_build_matches_checkout",
        "completed_run",
        "gates_and_controls",
        "metrics_and_uncertainty",
        "configuration_artifact_binding",
        "promotion_prerequisites",
    ];
    match target_status {
        "qualified_synthetic" => required_checks.push("synthetic_evidence_integrity"),
        "source_adapter_qualified" => required_checks.push("source_adapter_integrity"),
        "real_observation_evaluated" => {
            required_checks.push("source_adapter_integrity");
            required_checks.push("real_observation_holdout_integrity");
        }
        _ => {}
    }
    let report_checks_pass = report
        .get("checks")
        .and_then(Value::as_array)
        .is_some_and(|checks| {
            required_checks.iter().all(|required_id| {
                checks.iter().any(|check| {
                    string_field(check, &["id"]) == Some(required_id)
                        && check.get("passed").and_then(Value::as_bool) == Some(true)
                })
            })
        });
    if !report_checks_pass {
        return Err(
            "referenced verifier report does not contain every required passing gate".to_string(),
        );
    }
    if string_field(&report, &["qualification_core_sha256"]) != core_sha256 {
        return Err(
            "referenced verifier report is bound to a different qualification core".to_string(),
        );
    }
    let evidence_repo = string_field(
        receipt,
        &["qualification_evidence", "verifier_repository_url"],
    )
    .ok_or_else(|| "qualification_evidence.verifier_repository_url is missing".to_string())?;
    let measured_verifier_origin = git_output(verifier_root, &["remote", "get-url", "origin"])
        .ok_or_else(|| "cannot independently read verifier checkout origin".to_string())?;
    if normalize_git_url(evidence_repo) != normalize_git_url(&measured_verifier_origin) {
        return Err(
            "verifier repository URL does not match the separate verifier checkout origin"
                .to_string(),
        );
    }
    if verifier_root == repo_root {
        return Err(
            "verifier checkout is the same directory as the evaluated repository".to_string(),
        );
    }
    let evidence_commit = string_field(receipt, &["qualification_evidence", "verifier_commit_sha"])
        .ok_or_else(|| "qualification_evidence.verifier_commit_sha is missing".to_string())?;
    let report_verifier_repo = string_field(&report, &["verifier_repository_url"])
        .ok_or_else(|| "verifier report is missing its repository URL".to_string())?;
    if normalize_git_url(report_verifier_repo) != normalize_git_url(evidence_repo) {
        return Err(
            "verifier report repository URL does not match qualification evidence".to_string(),
        );
    }
    if report.get("verifier_worktree_clean").and_then(Value::as_bool) != Some(true)
        || report.get("verifier_checkout_independent").and_then(Value::as_bool) != Some(true)
        || report.get("verifier_build_matches_checkout").and_then(Value::as_bool) != Some(true)
    {
        return Err("verifier report does not establish a separate clean verifier checkout".to_string());
    }
    if string_field(&report, &["verifier_commit_sha"]) != Some(evidence_commit) {
        return Err(
            "referenced verifier report commit does not match qualification evidence".to_string(),
        );
    }
    if git_output(verifier_root, &["rev-parse", "HEAD"]).as_deref() != Some(evidence_commit) {
        return Err(
            "verifier report was not produced from the exact currently checked-out verifier commit"
                .to_string(),
        );
    }
    let evaluated_commit = string_field(receipt, &["code", "commit_sha"])
        .ok_or_else(|| "code.commit_sha is missing".to_string())?;
    if evaluated_commit == evidence_commit {
        return Err("verifier and evaluated code must be pinned to distinct commits".to_string());
    }
    if string_field(
        &receipt,
        &["qualification_evidence", "verification_outcome"],
    ) != Some("accepted")
    {
        return Err("qualification evidence outcome is not accepted".to_string());
    }
    Ok(())
}

fn compiled_verifier_checkout_root() -> Result<PathBuf, String> {
    let manifest_dir = fs::canonicalize(env!("CARGO_MANIFEST_DIR"))
        .map_err(|error| format!("cannot canonicalize compiled verifier manifest dir: {error}"))?;
    let repository_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            "compiled verifier manifest is not in the expected tools/<crate> layout".to_string()
        })?;
    fs::canonicalize(repository_root)
        .map_err(|error| format!("cannot canonicalize compiled verifier repository root: {error}"))
}

fn independent_verifier_checkout(
    target_root: &Path,
    target_head: Option<&str>,
    verifier_root: &Path,
    verifier_head: Option<&str>,
    verifier_worktree_clean: Option<bool>,
    verifier_origin: Option<&str>,
) -> bool {
    target_root != verifier_root
        && target_head
            .zip(verifier_head)
            .is_some_and(|(target, verifier)| target != verifier)
        && verifier_worktree_clean == Some(true)
        && verifier_origin.is_some_and(|origin| !origin.trim().is_empty())
}

fn git_snapshot(root: &Path) -> (Option<String>, Option<bool>, Option<String>) {
    let head = git_output(root, &["rev-parse", "HEAD"]);
    let status = git_output(root, &["status", "--porcelain", "--untracked-files=all"]);
    let remote = git_output(root, &["remote", "get-url", "origin"]);
    (head, status.map(|value| value.trim().is_empty()), remote)
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
        assert!(
            draft202012::options()
                .should_validate_formats(true)
                .build(&schema)
                .is_ok()
        );
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
        assert!(
            validator.is_valid(&receipt),
            "{:?}",
            validator
                .iter_errors(&receipt)
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        );
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
        assert!(!all_gates_and_controls_pass(
            &json!({"gates": [], "controls": []})
        ));
        assert!(!all_gates_and_controls_pass(&json!({
            "gates": [{"decision":"passed"}],
            "controls": [{"result":"not_evaluated"}]
        })));
        assert!(all_gates_and_controls_pass(&json!({
            "gates": [{"decision":"passed"}],
            "controls": [{"result":"passed"}]
        })));
    }
    #[test]
    fn queued_skipped_cancelled_missing_and_stale_head_ci_never_pass() {
        let commit = "a".repeat(40);
        let run_url = "https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/123";

        let valid_check = json!({
            "check_id": "123",
            "name": "Check",
            "commit_sha": commit,
            "status": "completed",
            "conclusion": "success",
            "url": run_url
        });
        let valid_evidence = json!({
            "id": 123,
            "name": "Check",
            "head_sha": commit,
            "status": "completed",
            "conclusion": "success",
            "html_url": run_url
        });
        assert!(ci_check_evidence_matches(
            &valid_check,
            &valid_evidence,
            &commit
        ));

        for status in [
            "pending",
            "queued",
            "in_progress",
            "skipped",
            "cancelled",
            "missing",
        ] {
            let check = json!({
                "check_id": "123",
                "name": "Check",
                "commit_sha": commit,
                "status": status,
                "conclusion": null,
                "url": run_url
            });
            let evidence = json!({
                "id": 123,
                "name": "Check",
                "head_sha": commit,
                "status": status,
                "conclusion": null,
                "html_url": run_url
            });
            assert!(
                !ci_check_evidence_matches(&check, &evidence, &commit),
                "{status} was incorrectly accepted"
            );
        }

        let stale_commit = "b".repeat(40);
        let stale_check = json!({
            "check_id": "123",
            "name": "Check",
            "commit_sha": stale_commit,
            "status": "completed",
            "conclusion": "success",
            "url": run_url
        });
        let stale_evidence = json!({
            "id": 123,
            "name": "Check",
            "head_sha": stale_commit,
            "status": "completed",
            "conclusion": "success",
            "html_url": run_url
        });
        assert!(!ci_check_evidence_matches(
            &stale_check,
            &stale_evidence,
            &commit
        ));

        let no_checks = json!({
            "code": {"commit_sha": commit},
            "ci_checks": []
        });
        assert!(!ci_check_names_match_policy(&[], &["Check".to_string()]));
        let one_check = vec![json!({"name":"Check"})];
        let required = vec![
            "Check".to_string(),
            "Earth Interior receipt verifier".to_string(),
        ];
        assert!(!ci_check_names_match_policy(&one_check, &required));
        let duplicated = vec![json!({"name":"Check"}), json!({"name":"Check"})];
        assert!(!ci_check_names_match_policy(&duplicated, &required));
        let complete = vec![
            json!({"name":"Check"}),
            json!({"name":"Earth Interior receipt verifier"}),
        ];
        assert!(ci_check_names_match_policy(&complete, &required));
    }

    #[test]
    fn metrics_without_uncertainty_method_cannot_qualify() {
        assert!(!metrics_present(&json!({
            "metrics": [{
                "metric_id": "rmse",
                "name": "RMSE",
                "value": 1.0,
                "unit": "nT",
                "sample_count": 100,
                "uncertainty_method": "none"
            }]
        })));
        assert!(metrics_present(&json!({
            "metrics": [{
                "metric_id": "rmse",
                "name": "RMSE",
                "value": 1.0,
                "unit": "nT",
                "sample_count": 100,
                "uncertainty_method": "bootstrap",
                "lower_bound": 0.9,
                "upper_bound": 1.1,
                "confidence_level": 0.95
            }]
        })));
    }

    #[test]
    fn canonical_json_digest_ignores_object_insertion_order() {
        let mut first = serde_json::Map::new();
        first.insert("z".to_string(), json!(2));
        first.insert("a".to_string(), json!({"q": 1, "b": 0}));
        let mut second = serde_json::Map::new();
        second.insert("a".to_string(), json!({"b": 0, "q": 1}));
        second.insert("z".to_string(), json!(2));
        assert_eq!(
            serde_json::to_vec(&canonical_json_value(&Value::Object(first))).unwrap(),
            serde_json::to_vec(&canonical_json_value(&Value::Object(second))).unwrap()
        );
    }

    #[test]
    fn truth_visibility_and_required_negative_controls_gate_synthetic_claims() {
        let artifacts = vec![
            ArtifactCheck {
                artifact_id: "truth".to_string(),
                role: "truth_state".to_string(),
                locator: "truth.json".to_string(),
                expected_sha256: None,
                actual_sha256: Some("a".repeat(64)),
                expected_byte_length: None,
                actual_byte_length: None,
                passed: true,
                detail: "verified".to_string(),
            },
            ArtifactCheck {
                artifact_id: "observations".to_string(),
                role: "synthetic_observations".to_string(),
                locator: "observations.json".to_string(),
                expected_sha256: None,
                actual_sha256: Some("b".repeat(64)),
                expected_byte_length: None,
                actual_byte_length: None,
                passed: true,
                detail: "verified".to_string(),
            },
        ];
        let receipt = json!({
            "artifacts": [
                {"artifact_id":"truth","role":"truth_state","locator":"truth.json","selector_visibility":"withheld_until_scoring"},
                {"artifact_id":"observations","role":"synthetic_observations","locator":"observations.json","selector_visibility":"visible_to_selector"}
            ],
            "controls": [
                {"kind":"null_signal","result":"passed"},
                {"kind":"truth_leakage","result":"passed"}
            ]
        });
        assert!(synthetic_artifacts_and_controls_valid(&receipt, &artifacts));

        let leaked = json!({
            "artifacts": [
                {"artifact_id":"truth","role":"truth_state","locator":"truth.json","selector_visibility":"visible_to_selector"},
                {"artifact_id":"observations","role":"synthetic_observations","locator":"observations.json","selector_visibility":"visible_to_selector"}
            ],
            "controls": [
                {"kind":"null_signal","result":"passed"},
                {"kind":"truth_leakage","result":"passed"}
            ]
        });
        assert!(!synthetic_artifacts_and_controls_valid(&leaked, &artifacts));
    }

    #[test]
    fn compiled_binary_manifest_resolves_to_repository_root() {
        let root =
            compiled_verifier_checkout_root().expect("compiled verifier root should resolve");
        assert!(root.join("tools/geo-receipt-verifier/Cargo.toml").is_file());
        assert!(
            root.join("docs/earth-interior/benchmark-run-receipt.schema.json")
                .is_file()
        );
    }

    #[test]
    fn promotion_requires_a_separate_clean_verifier_checkout_and_commit() {
        let target_root = Path::new("/tmp/evaluated");
        let verifier_root = Path::new("/tmp/verifier");
        let target_sha = "a".repeat(40);
        let verifier_sha = "b".repeat(40);
        let origin = "https://github.com/Luminous-Dynamics/sol-atlas-leptos";

        assert!(independent_verifier_checkout(
            target_root,
            Some(&target_sha),
            verifier_root,
            Some(&verifier_sha),
            Some(true),
            Some(origin),
        ));
        assert!(!independent_verifier_checkout(
            target_root,
            Some(&target_sha),
            target_root,
            Some(&verifier_sha),
            Some(true),
            Some(origin),
        ));
        assert!(!independent_verifier_checkout(
            target_root,
            Some(&target_sha),
            verifier_root,
            Some(&target_sha),
            Some(true),
            Some(origin),
        ));
        assert!(!independent_verifier_checkout(
            target_root,
            Some(&target_sha),
            verifier_root,
            Some(&verifier_sha),
            Some(false),
            Some(origin),
        ));
        assert!(!independent_verifier_checkout(
            target_root,
            Some(&target_sha),
            verifier_root,
            Some(&verifier_sha),
            Some(true),
            None,
        ));
    }

    #[test]
    fn benchmark_specification_commit_must_match_evaluated_code_commit() {
        let matching = json!({
            "benchmark_specification": {"git_commit_sha": "a".repeat(40)},
            "code": {"commit_sha": "a".repeat(40)}
        });
        let mismatched = json!({
            "benchmark_specification": {"git_commit_sha": "b".repeat(40)},
            "code": {"commit_sha": "a".repeat(40)}
        });
        let missing = json!({
            "benchmark_specification": {},
            "code": {"commit_sha": "a".repeat(40)}
        });
        assert!(benchmark_specification_commit_matches(&matching));
        assert!(!benchmark_specification_commit_matches(&mismatched));
        assert!(!benchmark_specification_commit_matches(&missing));
    }

    #[test]
    fn source_adapter_requires_verified_payload_and_all_audits() {
        let digest = "a".repeat(64);
        let receipt = json!({
            "source_adapter": {
                "local_sha256": digest,
                "units_audit": "passed",
                "time_axis_audit": "passed",
                "variable_shape_audit": "passed"
            }
        });
        let payload = vec![ArtifactCheck {
            artifact_id: "payload".to_string(),
            role: "source_payload".to_string(),
            locator: "payload.mat".to_string(),
            expected_sha256: Some(digest.clone()),
            actual_sha256: Some(digest.clone()),
            expected_byte_length: Some(10),
            actual_byte_length: Some(10),
            passed: true,
            detail: "verified".to_string(),
        }];
        assert!(source_adapter_valid(&receipt, &payload));

        let invalid_audit = json!({
            "source_adapter": {
                "local_sha256": digest,
                "units_audit": "passed",
                "time_axis_audit": "not_run",
                "variable_shape_audit": "passed"
            }
        });
        assert!(!source_adapter_valid(&invalid_audit, &payload));
        assert!(!source_adapter_valid(&receipt, &[]));
    }

    #[test]
    fn real_observation_claim_requires_real_data_and_frozen_holdout() {
        let real_digest = "a".repeat(64);
        let holdout_digest = "b".repeat(64);
        let payload_digest = "c".repeat(64);
        let receipt = json!({
            "source_adapter": {
                "local_sha256": payload_digest,
                "units_audit": "passed",
                "time_axis_audit": "passed",
                "variable_shape_audit": "passed"
            },
            "real_observation_evaluation": {
                "leakage_audit": "passed",
                "reference_overlap_audit": "passed",
                "holdout_manifest_sha256": holdout_digest
            }
        });
        let artifacts = vec![
            ArtifactCheck {
                artifact_id: "payload".to_string(),
                role: "source_payload".to_string(),
                locator: "payload.bin".to_string(),
                expected_sha256: Some(payload_digest.clone()),
                actual_sha256: Some(payload_digest.clone()),
                expected_byte_length: Some(1),
                actual_byte_length: Some(1),
                passed: true,
                detail: "verified".to_string(),
            },
            ArtifactCheck {
                artifact_id: "real-data".to_string(),
                role: "real_observations".to_string(),
                locator: "obs.bin".to_string(),
                expected_sha256: Some(real_digest.clone()),
                actual_sha256: Some(real_digest.clone()),
                expected_byte_length: Some(2),
                actual_byte_length: Some(2),
                passed: true,
                detail: "verified".to_string(),
            },
            ArtifactCheck {
                artifact_id: "holdout".to_string(),
                role: "holdout_manifest".to_string(),
                locator: "holdout.json".to_string(),
                expected_sha256: Some(holdout_digest.clone()),
                actual_sha256: Some(holdout_digest.clone()),
                expected_byte_length: Some(3),
                actual_byte_length: Some(3),
                passed: true,
                detail: "verified".to_string(),
            },
        ];
        assert!(source_adapter_valid(&receipt, &artifacts));
        assert!(real_observation_valid(&receipt, &artifacts));

        let missing_holdout = json!({
            "source_adapter": {
                "local_sha256": payload_digest,
                "units_audit": "passed",
                "time_axis_audit": "passed",
                "variable_shape_audit": "passed"
            },
            "real_observation_evaluation": {
                "leakage_audit": "passed",
                "reference_overlap_audit": "passed",
                "holdout_manifest_sha256": holdout_digest
            }
        });
        assert!(!real_observation_valid(&missing_holdout, &artifacts[..2]));
    }
    #[test]
    fn every_published_benchmark_spec_freezes_the_required_ci_set() {
        const SPECS: &[(&str, &str)] = &[
            (
                "geo-active-observation-design-001",
                include_str!(
                    "../../../docs/earth-interior/benchmark-adaptive-observation-design.json"
                ),
            ),
            (
                "geo-resolution-frontier-001",
                include_str!("../../../docs/earth-interior/benchmark-resolution-frontier.json"),
            ),
            (
                "geo-model-falsification-001",
                include_str!("../../../docs/earth-interior/benchmark-model-falsification.json"),
            ),
            (
                "geo-chron-discrete-age-marginalization-001",
                include_str!("../../../docs/earth-interior/benchmark-paleo-chronology.json"),
            ),
            (
                "geo-robust-design-under-discrepancy-001",
                include_str!(
                    "../../../docs/earth-interior/benchmark-robust-design-under-discrepancy.json"
                ),
            ),
        ];
        let expected = vec![
            "Check".to_string(),
            "Earth Interior receipt verifier".to_string(),
        ];
        for (expected_id, raw) in SPECS {
            let spec: Value = serde_json::from_str(raw).expect("benchmark spec JSON parses");
            assert_eq!(string_field(&spec, &["benchmark_id"]), Some(*expected_id));
            let names = spec
                .get("required_ci_checks")
                .and_then(Value::as_array)
                .expect("required_ci_checks must exist");
            let names: Vec<String> = names
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .expect("CI check name is a string")
                        .to_string()
                })
                .collect();
            assert_eq!(
                names, expected,
                "{expected_id} must freeze the full required CI set"
            );
            assert!(ci_check_names_match_policy(
                &names
                    .iter()
                    .map(|name| json!({"name":name}))
                    .collect::<Vec<_>>(),
                &expected
            ));
        }
    }
}
