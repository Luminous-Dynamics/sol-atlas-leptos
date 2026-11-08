use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::process::ExitCode;

use city_service_restoration_reference::{evaluate_case, matches_expected};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        eprintln!(
            "usage: city-service-restoration-reference CORPUS.json --source-commit <40-hex-sha>"
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
        eprintln!(
            "source commit must be supplied as --source-commit followed by 40 hex characters"
        );
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read restoration corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid restoration corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        eprintln!("restoration corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if corpus["schema"] != "city-service-restoration-corpus/0.1"
        || corpus["schema_version"] != "0.1.0"
        || corpus["input_contract"] != "city-service-restoration-input/0.1"
        || corpus["evidence_class"] != "synthetic"
        || cases.len() != 32
    {
        eprintln!("unexpected restoration corpus identity, schema, evidence class or case count");
        return ExitCode::from(2);
    }

    let corpus_sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut failures = 0usize;
    let mut plan_counts = BTreeMap::<String, usize>::new();
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
        let expected = case.get("expected").unwrap_or(&Value::Null);
        let passed = matches_expected(&actual, expected);
        if !passed {
            failures += 1;
        }
        let plan = actual
            .get("plan_readiness")
            .and_then(Value::as_str)
            .unwrap_or("invalid")
            .to_owned();
        *plan_counts.entry(plan).or_default() += 1;

        scenario_receipts.push(json!({
            "case_id": case_id,
            "status": if passed { "pass" } else { "fail" },
            "actual": actual
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-service-restoration.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_schema": corpus["schema"],
        "corpus_schema_version": corpus["schema_version"],
        "input_contract": corpus["input_contract"],
        "evidence_class": corpus["evidence_class"],
        "scenario_count": cases.len(),
        "failure_count": failures,
        "plan_readiness_counts": plan_counts,
        "corpus_sha256": corpus_sha256,
        "source_commit": source_commit,
        "claim_ceiling": "synthetic restoration-feasibility and lifecycle-separation semantics only; no real resource, safety, engineering or service claim is qualified",
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
