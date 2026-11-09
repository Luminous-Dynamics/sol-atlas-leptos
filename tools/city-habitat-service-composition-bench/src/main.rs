use std::env;
use std::fs;
use std::process::ExitCode;

use city_habitat_service_composition_reference::{
    CLAIM_CEILING, CORPUS_ID, EXPECTED_CASES, evaluate_case, validate_corpus,
};
use serde_json::json;
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        return invalid("usage: city-habitat-service-composition-reference CORPUS.json --source-commit <40-lowercase-hex-sha>");
    };
    let Some(flag) = args.next() else {
        return invalid("missing --source-commit");
    };
    let Some(source_commit) = args.next() else {
        return invalid("missing source commit");
    };
    if flag != "--source-commit" || !valid_commit(&source_commit) || args.next().is_some() {
        return invalid("invalid source commit arguments");
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&format!("cannot read corpus: {error}")),
    };
    let corpus: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => return invalid(&format!("corpus JSON invalid: {error}")),
    };
    let validation_errors = validate_corpus(&corpus);
    if !validation_errors.is_empty() {
        println!("{}", json!({
            "result": "invalid",
            "corpus_id": CORPUS_ID,
            "validation_errors": validation_errors,
            "claim_ceiling": CLAIM_CEILING
        }));
        return ExitCode::FAILURE;
    }

    let cases = corpus["cases"].as_array().expect("validated cases array");
    let mut failures = 0usize;
    let mut outputs = Vec::with_capacity(cases.len());
    for case in cases {
        let actual = evaluate_case(case);
        if actual != case["expected"] {
            failures += 1;
        }
        outputs.push(actual);
    }

    let passed = cases.len().saturating_sub(failures);
    let receipt = json!({
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": CORPUS_ID,
        "corpus_classification": corpus["classification"],
        "scenario_count": cases.len(),
        "expected_scenario_count": EXPECTED_CASES,
        "passed_count": passed,
        "failure_count": failures,
        "corpus_sha256": sha256_hex(&bytes),
        "source_commit": source_commit,
        "outputs": outputs,
        "realized_use_claim": "none",
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING
    });
    println!("{}", serde_json::to_string_pretty(&receipt).expect("receipt serializes"));
    if failures == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
