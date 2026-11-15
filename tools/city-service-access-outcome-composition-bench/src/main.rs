use std::env;
use std::fs;
use std::process::ExitCode;

use city_service_access_outcome_composition_reference::{
    CLAIM_CEILING, CORPUS_ID, evaluate_case, validate_corpus,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("failed to read {path}: {error}"))
}

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

fn parse_json(bytes: &[u8], name: &str) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("invalid {name} JSON: {error}"))
}

fn invalid(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        return invalid(
            "usage: city-service-access-outcome-composition-reference CORPUS.json CORPUS_SCHEMA.json --source-commit <40-lowercase-hex-sha>",
        );
    };
    let Some(schema_path) = args.next() else {
        return invalid("missing corpus schema path");
    };
    let Some(flag) = args.next() else {
        return invalid("missing --source-commit flag");
    };
    let Some(source_commit) = args.next() else {
        return invalid("missing source commit SHA");
    };
    if flag != "--source-commit" || !valid_commit(&source_commit) || args.next().is_some() {
        return invalid("invalid source commit SHA or unexpected trailing arguments");
    }

    let corpus_bytes = match read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&error),
    };
    let schema_bytes = match read(&schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&error),
    };
    let corpus = match parse_json(&corpus_bytes, "corpus") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };
    let schema = match parse_json(&schema_bytes, "corpus schema") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };

    if corpus["corpus_id"] != CORPUS_ID
        || corpus["claim_ceiling"] != CLAIM_CEILING
        || schema["$schema"] != "https://json-schema.org/draft/2020-12/schema"
        || schema["$id"]
            != "https://luminousdynamics.io/schemas/city-service-access-outcome-composition-corpus-v0.1.schema.json"
        || schema["title"] != "City service access and realized outcome composition corpus v0.1"
    {
        return invalid("unexpected corpus or schema identity");
    }
    let errors = validate_corpus(&corpus);
    if !errors.is_empty() {
        return invalid(&format!("invalid corpus: {}", errors.join(",")));
    }

    let mut failures = 0usize;
    let mut scenarios = Vec::new();
    for case in corpus["cases"]
        .as_array()
        .expect("corpus validation checks cases")
    {
        let evaluation = evaluate_case(case);
        let passed = evaluation.output == case["expected"];
        if !passed {
            failures += 1;
        }
        scenarios.push(json!({
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "expected_match": passed,
            "output": evaluation.output
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-service-access-outcome-composition.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "corpus_schema_sha256": sha256_hex(&schema_bytes),
        "scenario_count": scenarios.len(),
        "failure_count": failures,
        "source_commit": source_commit,
        "owner_contracts_composed": [
            "CITY-SCOPE-BINDING-001",
            "CITY-RESTORATION-OUTCOME-BRIDGE-001",
            "CITY-MORPH-ACCESS-001",
            "CITY-IN-USE-OUTCOMES-001"
        ],
        "potential_access_is_actual_use": false,
        "resident_experience_requires_consent": true,
        "missing_demand_is_zero": false,
        "zero_demand_is_divided": false,
        "issuer_authentication": "not_performed",
        "source_byte_authentication": "not_performed",
        "population_representativeness": "not_established_by_synthetic_oracle",
        "clinical_or_service_quality": "not_assessed",
        "causal_effect": "not_assessed",
        "professional_approval": "not_assessed",
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
