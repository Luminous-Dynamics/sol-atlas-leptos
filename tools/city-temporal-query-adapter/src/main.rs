use city_temporal_query_adapter::{evaluate_case, matches_expected};
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
        eprintln!(
            "usage: city-temporal-query-adapter CORPUS.json --source-commit <40-lowercase-hex>"
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
    if flag != "--source-commit" || !valid_commit(&source_commit) || args.next().is_some() {
        eprintln!("source commit must be a lowercase 40-character hex SHA; no extra arguments are accepted");
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read temporal corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid temporal corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        eprintln!("temporal corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["schema"] != "luminous.citymaker-temporal-query-corpus.v0.1"
        || corpus["schema_version"] != "0.1.0"
        || corpus["corpus_id"] != "CITY-TEMPORAL-QUERY-CORPUS-V0.1"
        || corpus["classification"] != "synthetic_fixture_only"
        || corpus["query_schema_version"] != "luminous.citymaker-temporal-query.v0.1"
        || cases.len() != 28
    {
        eprintln!("unexpected temporal corpus identity, schema, classification or case count");
        return ExitCode::from(2);
    }

    let mut case_ids = BTreeSet::new();
    for case in cases {
        let Some(id) = case.get("case_id").and_then(Value::as_str) else {
            eprintln!("temporal case is missing a string case_id");
            return ExitCode::from(2);
        };
        if id.is_empty() || !case_ids.insert(id.to_owned()) {
            eprintln!("temporal case ID is empty or duplicated: {id}");
            return ExitCode::from(2);
        }
    }

    let mut failures = 0usize;
    let mut scenarios = Vec::with_capacity(cases.len());
    for case in cases {
        let actual = evaluate_case(case);
        let expected = case.get("expected").unwrap_or(&Value::Null);
        let passed = matches_expected(&actual, expected);
        if !passed {
            failures += 1;
        }
        scenarios.push(json!({
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "disposition": actual["disposition"],
            "world_time_state": actual["world_time_state"],
            "knowledge_time_state": actual["knowledge_time_state"],
            "trust_time_state": actual["trust_time_state"],
            "report_binding_state": actual["report_binding_state"],
            "reason_codes": actual["reason_codes"]
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-temporal-query.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "scenario_count": cases.len(),
        "failure_count": failures,
        "corpus_sha256": sha256_hex(&bytes),
        "source_commit": source_commit,
        "claim_ceiling": "synthetic bitemporal query and owner-report binding semantics only; no timestamp authentication, signer authority, legal retroactivity, source completeness, engineering correctness or action authority",
        "owner_report_content_verified": false,
        "timestamp_authentication": "not_performed",
        "action_authority": "none",
        "scenarios": scenarios
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).expect("temporal receipt serializes")
    );

    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
