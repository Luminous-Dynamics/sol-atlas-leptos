use city_impact_lane_router::{evaluate_case, matches_expected};
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
    let Some(registry_path) = args.next() else {
        eprintln!(
            "usage: city-impact-lane-router REGISTRY.json CORPUS.json --source-commit <40-lowercase-hex-sha>"
        );
        return ExitCode::from(2);
    };
    let Some(corpus_path) = args.next() else {
        eprintln!("missing corpus path");
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

    let registry_bytes = match fs::read(&registry_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read owner lane registry {registry_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus_bytes = match fs::read(&corpus_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read routing corpus {corpus_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let registry: Value = match serde_json::from_slice(&registry_bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid owner lane registry JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let corpus: Value = match serde_json::from_slice(&corpus_bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid routing corpus JSON: {error}");
            return ExitCode::from(2);
        }
    };
    let Some(cases) = corpus.get("cases").and_then(Value::as_array) else {
        eprintln!("routing corpus is missing a cases array");
        return ExitCode::from(2);
    };
    if registry["schema_version"] != "luminous.city-impact-owner-lane-registry.v0.1"
        || registry["registry_id"] != "citymaker-owner-impact-lanes-2026-11-08"
        || registry["action_authority"] != "none"
        || corpus["schema_version"] != "luminous.city-impact-lane-router-corpus.v0.1"
        || corpus["corpus_id"] != "CITY-IMPACT-LANE-ROUTER-CORPUS-V0.1"
        || corpus["classification"] != "synthetic_fixture_only"
        || corpus["registry_id"] != registry["registry_id"]
        || cases.len() != 32
    {
        eprintln!("unexpected registry/corpus identity, classification or case count");
        return ExitCode::from(2);
    }

    let mut case_ids = BTreeSet::new();
    for case in cases {
        let Some(id) = case.get("case_id").and_then(Value::as_str) else {
            eprintln!("routing case is missing a string case_id");
            return ExitCode::from(2);
        };
        if id.is_empty() || !case_ids.insert(id.to_owned()) {
            eprintln!("routing case ID is empty or duplicated: {id}");
            return ExitCode::from(2);
        }
    }

    let mut failures = 0usize;
    let mut scenario_receipts = Vec::with_capacity(cases.len());
    for case in cases {
        let actual = evaluate_case(case, &registry);
        let expected = case.get("expected").unwrap_or(&Value::Null);
        let passed = matches_expected(
            &json!({
                "route_state": actual["route_state"],
                "required_families": actual["required_families"],
                "reason_codes": actual["reason_codes"]
            }),
            &json!({
                "route_state": expected["route_state"],
                "required_families": expected["required_families"],
                "reason_codes": expected["reason_codes"]
            }),
        ) && lane_states_match(&actual, expected);

        if !passed {
            failures += 1;
        }
        scenario_receipts.push(json!({
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "route_state": actual["route_state"],
            "required_families": actual["required_families"],
            "reason_codes": actual["reason_codes"],
            "lane_results": actual["lane_results"],
            "owner_report_content_verified": false,
            "owner_authentication": "not_performed",
            "action_authority": "none"
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.city-impact-lane-router.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "registry_id": registry["registry_id"],
        "registry_schema_version": registry["schema_version"],
        "registry_sha256": sha256_hex(&registry_bytes),
        "corpus_id": corpus["corpus_id"],
        "corpus_schema_version": corpus["schema_version"],
        "corpus_classification": corpus["classification"],
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "scenario_count": cases.len(),
        "failure_count": failures,
        "source_commit": source_commit,
        "claim_ceiling": "synthetic owner-lane routing and declared contract-binding semantics only; no owner authentication, report-content verification, impact correctness, source completeness, engineering applicability or action authority",
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

fn lane_states_match(actual: &Value, expected: &Value) -> bool {
    let actual_rows = actual
        .get("lane_results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let expected_rows = expected
        .get("lane_states")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let actual_pairs = actual_rows
        .iter()
        .map(|row| {
            (
                row.get("family").and_then(Value::as_str).unwrap_or(""),
                row.get("state").and_then(Value::as_str).unwrap_or(""),
            )
        })
        .collect::<Vec<_>>();
    let expected_pairs = expected_rows
        .iter()
        .map(|row| {
            (
                row.get("family").and_then(Value::as_str).unwrap_or(""),
                row.get("state").and_then(Value::as_str).unwrap_or(""),
            )
        })
        .collect::<Vec<_>>();
    actual_pairs == expected_pairs
}
