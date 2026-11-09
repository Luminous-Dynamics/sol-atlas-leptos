use std::env;
use std::fs;
use std::process::ExitCode;

use highveld_federated_composition_reference::{
    CLAIM_CEILING, CORPUS_CLASSIFICATION, CORPUS_ID, PROFILE_ID, evaluate_case,
    parse_unique_json_bytes, validate_corpus, validate_profiles,
};
use serde_json::json;
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

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("failed to read {path}: {error}"))
}

fn invalid_input(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        return invalid_input(
            "usage: highveld-federated-composition-reference CORPUS.json SCHEMA.json OWNER_PROFILES.json --source-commit <40-lowercase-hex-sha>",
        );
    };
    let Some(schema_path) = args.next() else {
        return invalid_input("missing corpus schema path");
    };
    let Some(profile_path) = args.next() else {
        return invalid_input("missing owner profile path");
    };
    let Some(flag) = args.next() else {
        return invalid_input("missing --source-commit argument");
    };
    let Some(source_commit) = args.next() else {
        return invalid_input("missing source commit SHA");
    };
    if flag != "--source-commit"
        || !valid_commit(&source_commit)
        || args.next().is_some()
    {
        return invalid_input(
            "source commit must be a lowercase 40-character hex SHA; no extra arguments are accepted",
        );
    }

    let corpus_bytes = match read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid_input(&error),
    };
    let schema_bytes = match read(&schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid_input(&error),
    };
    let profile_bytes = match read(&profile_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid_input(&error),
    };

    let corpus = match parse_unique_json_bytes(&corpus_bytes) {
        Ok(value) => value,
        Err(error) => return invalid_input(&format!("invalid corpus JSON: {error}")),
    };
    let schema = match parse_unique_json_bytes(&schema_bytes) {
        Ok(value) => value,
        Err(error) => return invalid_input(&format!("invalid corpus schema JSON: {error}")),
    };
    let profiles = match parse_unique_json_bytes(&profile_bytes) {
        Ok(value) => value,
        Err(error) => return invalid_input(&format!("invalid owner profile JSON: {error}")),
    };

    if schema["$schema"] != "https://json-schema.org/draft/2020-12/schema"
        || schema["$id"]
            != "https://luminousdynamics.io/schemas/highveld-federated-twin-composition-corpus-v0.1.schema.json"
        || schema["title"] != "Highveld Federated Twin Composition Synthetic Corpus v0.1"
        || schema["type"] != "object"
        || schema["additionalProperties"] != false
    {
        return invalid_input("unsupported corpus schema identity or root strictness");
    }
    if corpus["corpus_id"] != CORPUS_ID
        || corpus["classification"] != CORPUS_CLASSIFICATION
        || profiles["profile_id"] != PROFILE_ID
    {
        return invalid_input("unexpected corpus or owner profile identity");
    }

    let profile_errors = validate_profiles(&profiles);
    if !profile_errors.is_empty() {
        return invalid_input(&format!("invalid owner profiles: {}", profile_errors.join(",")));
    }
    let corpus_errors = validate_corpus(&corpus, &profiles);
    if !corpus_errors.is_empty() {
        return invalid_input(&format!("invalid corpus structure: {}", corpus_errors.join(",")));
    }

    let cases = corpus["cases"].as_array().expect("validated cases array");
    let mut failures = 0usize;
    let mut scenarios = Vec::with_capacity(cases.len());
    for case in cases {
        let actual = evaluate_case(&case["input"], &profiles);
        let expected = &case["expected"];
        let passed = &actual == expected;
        if !passed {
            failures += 1;
        }
        scenarios.push(json!({
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "actual": actual,
            "expected_match": passed
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.highveld-federated-composition.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "corpus_schema_sha256": sha256_hex(&schema_bytes),
        "owner_profiles_id": profiles["profile_id"],
        "owner_profiles_sha256": sha256_hex(&profile_bytes),
        "scenario_count": cases.len(),
        "failure_count": failures,
        "source_commit": source_commit,
        "temporal_replay": "valid_recorded_registered_ingested_time_cuts_v0.1",
        "automatic_identity_merge_count": scenarios.iter()
            .map(|scenario| scenario["actual"]["automatic_identity_merges"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        "owner_payload_bytes_cryptographically_verified": false,
        "issuer_authentication": "not_performed",
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING,
        "scenarios": scenarios
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).expect("receipt serializes")
    );
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
