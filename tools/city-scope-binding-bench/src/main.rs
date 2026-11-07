use std::env;
use std::fs;
use std::process::ExitCode;

use city_scope_binding_reference::{evaluate_case, matches_expected};
use serde_json::json;
use sha2::{Digest, Sha256};

fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        eprintln!("usage: city-scope-binding-reference CORPUS.json --source-commit <40-hex-sha>");
        return ExitCode::from(2);
    };
    let Some(flag) = args.next() else {
        eprintln!("missing --source-commit argument");
        return ExitCode::from(2);
    };
    let Some(source_commit) = args.next() else {
        eprintln!("missing source commit SHA");
        return ExitCode::from(2);
    };
    if flag != "--source-commit" || !valid_commit(&source_commit) {
        eprintln!("source commit must be supplied as --source-commit followed by 40 hex characters");
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read scope-binding corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid scope-binding corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(serde_json::Value::as_array) else {
        eprintln!("corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["corpus_id"] != "CITY-SCOPE-BINDING-CORPUS-V0.1"
        || corpus["schema_version"] != "0.1.0"
        || corpus["classification"] != "synthetic_fixture_only"
        || cases.len() != 25
    {
        eprintln!("unexpected scope-binding corpus identity, schema, classification or case count");
        return ExitCode::from(2);
    }

    let digest = Sha256::digest(&bytes);
    let corpus_sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let mut failures = 0usize;
    let mut scenario_receipts = Vec::with_capacity(cases.len());

    for case in cases {
        let actual = evaluate_case(case);
        let expected = case.get("expected").unwrap_or(&serde_json::Value::Null);
        let passed = matches_expected(&actual, expected);
        if !passed {
            failures += 1;
        }
        scenario_receipts.push(json!({
            "case_id": case.get("case_id").and_then(serde_json::Value::as_str).unwrap_or("missing-case-id"),
            "status": if passed { "pass" } else { "fail" },
            "actual": actual
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-scope-binding.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "scenario_count": cases.len(),
        "failure_count": failures,
        "corpus_sha256": corpus_sha256,
        "source_commit": source_commit,
        "scope_digest_algorithm": corpus["scope_digest_algorithm"],
        "claim_ceiling": "synthetic scope metadata and owner-envelope binding only; no owner payload bytes are verified and no real city, service, product or construction claim is qualified",
        "scenarios": scenario_receipts
    });
    println!("{}", serde_json::to_string_pretty(&receipt).expect("receipt serializes"));

    if failures == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
