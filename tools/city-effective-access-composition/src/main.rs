use std::env;
use std::fs;
use std::process::ExitCode;

use city_effective_access_composition_reference::{
    CLAIM_CEILING, CORPUS_ID, evaluate_case, validate_corpus,
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

fn invalid(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        return invalid(
            "usage: city-effective-access-composition-reference CORPUS.json --source-commit <40-lowercase-hex-sha>",
        );
    };
    let Some(flag) = args.next() else {
        return invalid("missing --source-commit flag");
    };
    let Some(source_commit) = args.next() else {
        return invalid("missing source commit SHA");
    };
    if flag != "--source-commit" || !valid_commit(&source_commit) || args.next().is_some() {
        return invalid("invalid source commit argument");
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&format!("cannot read corpus: {error}")),
    };
    let corpus: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => return invalid(&format!("corpus JSON invalid: {error}")),
    };
    let errors = validate_corpus(&corpus);
    if !errors.is_empty() {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "result": "invalid",
                "corpus_id": CORPUS_ID,
                "validation_errors": errors
            }))
            .expect("validation receipt serializes")
        );
        return ExitCode::FAILURE;
    }

    let mut failures = 0usize;
    let scenarios = corpus["cases"]
        .as_array()
        .expect("validated corpus cases")
        .iter()
        .map(|case| {
            let actual = evaluate_case(case);
            let passed = actual == case["expected"];
            if !passed {
                failures += 1;
            }
            json!({
                "case_id": case["case_id"],
                "status": if passed { "pass" } else { "fail" },
                "actual": actual,
                "expected": case["expected"]
            })
        })
        .collect::<Vec<_>>();

    let receipt = json!({
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": CORPUS_ID,
        "corpus_classification": "synthetic_fixture_only",
        "corpus_sha256": sha256_hex(&bytes),
        "source_commit": source_commit,
        "scenario_count": scenarios.len(),
        "failure_count": failures,
        "claim_ceiling": CLAIM_CEILING,
        "action_authority": "none",
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
