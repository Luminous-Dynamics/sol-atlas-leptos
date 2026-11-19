use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use city_morph_access_reference::{evaluate_case, matches_expected, validate_source_register};
use serde_json::json;
use sha2::{Digest, Sha256};

fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        eprintln!("usage: city-morph-access-reference CORPUS.json --source-commit <40-hex-sha>");
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
        eprintln!(
            "source commit must be supplied as --source-commit followed by 40 hex characters"
        );
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid city-morph corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(serde_json::Value::as_array) else {
        eprintln!("corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["corpus_id"] != "CITY-MORPH-ACCESS-CORPUS-V0.1"
        || corpus["schema_version"] != "0.1.0"
        || corpus["classification"] != "synthetic_fixture_only"
        || cases.len() != 24
    {
        eprintln!("unexpected corpus identity, schema version, classification or case count");
        return ExitCode::from(2);
    }

    let register_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/research/city-morph-access/JOHANNESBURG_SOURCE_REGISTER_V0_1.json");
    let register_bytes = match fs::read(&register_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read Johannesburg source register: {error}");
            return ExitCode::from(2);
        }
    };
    let source_register: serde_json::Value = match serde_json::from_slice(&register_bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid Johannesburg source register JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let source_count = match validate_source_register(&source_register) {
        Ok(count) => count,
        Err(errors) => {
            eprintln!(
                "invalid Johannesburg source register: {}",
                errors.join(", ")
            );
            return ExitCode::from(2);
        }
    };
    let source_register_digest = Sha256::digest(&register_bytes);
    let source_register_sha256 = source_register_digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    let digest = Sha256::digest(&bytes);
    let corpus_sha256 = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut scenario_receipts = Vec::with_capacity(cases.len());
    let mut failures = 0usize;

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
            "actual": actual,
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-morph-access.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "scenario_count": cases.len(),
        "failure_count": failures,
        "corpus_sha256": corpus_sha256,
        "source_register_id": source_register["register_id"],
        "source_register_as_of": source_register["as_of"],
        "source_register_classification": source_register["classification"],
        "source_register_count": source_count,
        "source_register_sha256": source_register_sha256,
        "source_commit": source_commit,
        "claim_ceiling": "synthetic effective-access reference agreement only; not real city access, safety, affordability, service quality, planning, or community-consent qualification",
        "scenarios": scenario_receipts,
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
