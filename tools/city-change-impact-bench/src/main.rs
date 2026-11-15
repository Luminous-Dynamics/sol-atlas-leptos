use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::process::ExitCode;

use city_change_impact_reference::{evaluate_case, matches_expected};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        eprintln!("usage: city-change-impact-reference CORPUS.json --source-commit <40-hex-sha>");
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
            eprintln!("failed to read change-impact corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid change-impact corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        eprintln!("change-impact corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["schema"] != "city-change-impact-corpus/0.1"
        || corpus["schema_version"] != "0.1.0"
        || corpus["classification"] != "synthetic_fixture_only"
        || corpus["corpus_profile"] != "owner_report_projection_v0.1"
        || cases.len() != 28
    {
        eprintln!("unexpected change-impact corpus identity, schema, profile or case count");
        return ExitCode::from(2);
    }

    let corpus_sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut failures = 0usize;
    let mut state_counts = BTreeMap::<String, usize>::new();
    let mut scenario_receipts = Vec::with_capacity(cases.len());
    let mut seen_ids = std::collections::BTreeSet::new();

    for case in cases {
        let case_id = case
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("missing-case-id");
        if case_id.is_empty() || !seen_ids.insert(case_id.to_owned()) {
            failures += 1;
            scenario_receipts.push(json!({
                "case_id": case_id,
                "status": "fail",
                "reason": "duplicate_or_missing_case_id"
            }));
            continue;
        }

        let actual = evaluate_case(case);
        let expected = case
            .get("expected")
            .and_then(|value| value.get("claim_applicability"))
            .unwrap_or(&Value::Null);
        let observed = actual.get("claim_applicability").unwrap_or(&Value::Null);
        let passed = matches_expected(observed, expected);
        if !passed {
            failures += 1;
        }

        if let Some(results) = observed.as_array() {
            for result in results {
                let state = result
                    .get("state")
                    .and_then(Value::as_str)
                    .unwrap_or("invalid")
                    .to_owned();
                *state_counts.entry(state).or_default() += 1;
            }
        }

        scenario_receipts.push(json!({
            "case_id": case_id,
            "status": if passed { "pass" } else { "fail" },
            "actual": actual
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-change-impact.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_schema": corpus["schema"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_profile": corpus["corpus_profile"],
        "evidence_class": corpus["classification"],
        "scenario_count": cases.len(),
        "failure_count": failures,
        "claim_state_counts": state_counts,
        "corpus_sha256": corpus_sha256,
        "source_commit": source_commit,
        "action_authority": "none",
        "claim_ceiling": "synthetic owner-report projection semantics only; no real dependency coverage, change authority, engineering applicability or action authority is qualified",
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
