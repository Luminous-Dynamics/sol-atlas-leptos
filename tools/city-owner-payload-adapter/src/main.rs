use city_owner_payload_adapter::{evaluate_case, matches_expected};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

fn valid_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        eprintln!(
            "usage: city-owner-payload-adapter CORPUS.json --source-commit <40-lowercase-hex>"
        );
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
        eprintln!("source commit must be a lowercase 40-character hex SHA");
        return ExitCode::from(2);
    }
    if args.next().is_some() {
        eprintln!("unexpected trailing command-line argument");
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        eprintln!("corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["schema"] != "city-owner-payload-adapter-corpus/0.1"
        || corpus["schema_version"] != "0.1.0"
        || corpus["corpus_id"] != "CITY-OWNER-PAYLOAD-ADAPTER-CORPUS-V0.1"
        || corpus["classification"] != "synthetic_fixture_only"
        || cases.len() != 24
    {
        eprintln!("unexpected corpus identity, schema, classification or case count");
        return ExitCode::from(2);
    }

    let mut ids = BTreeSet::new();
    for case in cases {
        let Some(id) = case.get("case_id").and_then(Value::as_str) else {
            eprintln!("case is missing a string case_id");
            return ExitCode::from(2);
        };
        if id.is_empty() || !ids.insert(id.to_owned()) {
            eprintln!("case ID is empty or duplicated: {id}");
            return ExitCode::from(2);
        }
    }

    let corpus_sha256 = sha256_hex(&bytes);
    let mut failures = 0usize;
    let mut scenario_receipts = Vec::with_capacity(cases.len());
    for case in cases {
        let actual = evaluate_case(case);
        let expected = case.get("expected").unwrap_or(&Value::Null);
        let passed = matches_expected(&actual, expected);
        if !passed {
            failures += 1;
        }
        scenario_receipts.push(json!({
            "case_id": case.get("case_id").and_then(Value::as_str).unwrap_or("missing"),
            "status": if passed { "pass" } else { "fail" },
            "report_integrity_state": actual.get("report_integrity_state").cloned().unwrap_or(Value::Null),
            "payload_verification_state": actual.get("payload_verification_state").cloned().unwrap_or(Value::Null),
            "actual": actual
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-owner-payload-adapter.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "scenario_count": cases.len(),
        "failure_count": failures,
        "corpus_sha256": corpus_sha256,
        "source_commit": source_commit,
        "claim_ceiling": "synthetic owner-envelope to byte-report adapter semantics only; no source authenticity, rights authentication, engineering correctness, scope approval, physical acceptance or action authority",
        "action_authority": "none",
        "owner_result_promotion": "none",
        "causal_effect_established": false,
        "scenarios": scenario_receipts
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
