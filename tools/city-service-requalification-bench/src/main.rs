use city_service_requalification_reference::{CORPUS_ID, evaluate_case, matches_expected};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

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

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        eprintln!("usage: city-service-requalification-reference CORPUS.json --source-commit <40-lowercase-hex-sha>");
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
    if flag != "--source-commit" || !valid_commit(&source_commit) || args.next().is_some() {
        eprintln!("source commit must be a lowercase 40-character hex SHA; no extra arguments are accepted");
        return ExitCode::from(2);
    }

    let corpus_bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read service requalification corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: Value = match serde_json::from_slice(&corpus_bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid service requalification corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        eprintln!("service requalification corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["schema_version"] != "luminous.city-service-requalification-corpus.v0.1"
        || corpus["corpus_id"] != CORPUS_ID
        || corpus["classification"] != "synthetic_fixture_only"
        || cases.len() != 26
    {
        eprintln!("unexpected service requalification corpus identity, classification or case count");
        return ExitCode::from(2);
    }

    let mut case_ids = BTreeSet::new();
    for case in cases {
        let Some(id) = case.get("case_id").and_then(Value::as_str) else {
            eprintln!("service requalification case is missing a string case_id");
            return ExitCode::from(2);
        };
        if id.is_empty() || !case_ids.insert(id.to_owned()) {
            eprintln!("service requalification case ID is empty or duplicated: {id}");
            return ExitCode::from(2);
        }
    }

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
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "route_state": actual["route_state"],
            "service_results": actual["service_results"],
            "reason_codes": actual["reason_codes"],
            "service_outcome_verified": false,
            "owner_report_content_verified": false,
            "owner_authentication": "not_performed",
            "action_authority": "none"
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-service-requalification.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": CORPUS_ID,
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "scenario_count": cases.len(),
        "failure_count": failures,
        "source_commit": source_commit,
        "claim_ceiling": "synthetic service-owner envelope and state-composition semantics only; no owner-report authentication, engineering threshold validation, physical safety determination, source completeness or action authority",
        "service_outcome_verified": false,
        "owner_report_content_verified": false,
        "owner_authentication": "not_performed",
        "action_authority": "none",
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
