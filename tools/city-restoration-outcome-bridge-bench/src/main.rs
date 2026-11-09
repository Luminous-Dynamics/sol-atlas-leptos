use std::env;
use std::fs;
use std::process::ExitCode;

use city_restoration_outcome_bridge_reference::{
    CLAIM_CEILING, CORPUS_ID, evaluate_case, validate_corpus,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid_input(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(2)
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("failed to read {path}: {error}"))
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        return invalid_input(
            "usage: city-restoration-outcome-bridge-reference CORPUS.json SCHEMA.json --source-commit <40-lowercase-hex-sha>",
        );
    };
    let Some(schema_path) = args.next() else {
        return invalid_input("missing corpus schema path");
    };
    let Some(flag) = args.next() else {
        return invalid_input("missing --source-commit argument");
    };
    let Some(source_commit) = args.next() else {
        return invalid_input("missing source commit SHA");
    };
    if flag != "--source-commit" || !valid_commit(&source_commit) || args.next().is_some() {
        return invalid_input("invalid source commit or unexpected trailing arguments");
    }

    let corpus_bytes = match read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid_input(&error),
    };
    let schema_bytes = match read(&schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid_input(&error),
    };
    let corpus: Value = match serde_json::from_slice(&corpus_bytes) {
        Ok(value) => value,
        Err(error) => return invalid_input(&format!("invalid corpus JSON: {error}")),
    };
    let schema: Value = match serde_json::from_slice(&schema_bytes) {
        Ok(value) => value,
        Err(error) => return invalid_input(&format!("invalid schema JSON: {error}")),
    };
    if schema["$schema"] != "https://json-schema.org/draft/2020-12/schema"
        || schema["$id"]
            != "https://luminousdynamics.io/schemas/city-restoration-outcome-bridge-corpus-v0.1.schema.json"
        || schema["title"] != "City Restoration Outcome Bridge Synthetic Corpus v0.1"
    {
        return invalid_input("unsupported corpus schema identity");
    }
    let errors = validate_corpus(&corpus);
    if !errors.is_empty() {
        return invalid_input(&format!(
            "invalid corpus structure: {}",
            errors.join(",")
        ));
    }

    let cases = corpus["cases"].as_array().expect("validated cases array");
    let mut failures = 0usize;
    let mut scenarios = Vec::with_capacity(cases.len());
    for case in cases {
        let actual = evaluate_case(case);
        let passed = actual == case["expected"];
        if !passed {
            failures += 1;
        }
        scenarios.push(json!({
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "expected_match": passed,
            "actual": actual
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-restoration-outcome-bridge.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "schema_sha256": sha256_hex(&schema_bytes),
        "scenario_count": cases.len(),
        "failure_count": failures,
        "source_commit": source_commit,
        "owner_evaluators_reused": [
            "CITY-SERVICE-RESTORATION-001",
            "CITY-MORPH-ACCESS-001",
            "CITY-IN-USE-OUTCOMES-001",
            "CITY-SCOPE-BINDING-001",
            "CIV-SERVICE-001"
        ],
        "owner_evaluations_recomputed": false,
        "physical_measurements_performed": false,
        "professional_approval": "not_assessed",
        "action_authority": "none",
        "causal_effect_established": false,
        "claim_ceiling": CLAIM_CEILING,
        "scenarios": scenarios
    });
    println!("{}", serde_json::to_string_pretty(&receipt).expect("receipt serializes"));

    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
