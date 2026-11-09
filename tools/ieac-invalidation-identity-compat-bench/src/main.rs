use std::env;
use std::fs;
use std::process::ExitCode;

use ieac_invalidation_identity_compat_reference::{
    CLAIM_CEILING, CORPUS_ID, evaluate_corpus_case, validate_corpus,
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

fn invalid(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(2)
}

fn parse_json(bytes: &[u8], name: &str) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("invalid {name} JSON: {error}"))
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(corpus_path) = args.next() else {
        return invalid("usage: ieac-invalidation-identity-compat-reference CORPUS.json CORPUS_SCHEMA.json GRAPH_SCHEMA.json EVENT_SCHEMA.json REPORT_SCHEMA.json --source-commit <40-lowercase-hex-sha>");
    };
    let Some(corpus_schema_path) = args.next() else {
        return invalid("missing corpus schema path");
    };
    let Some(graph_schema_path) = args.next() else {
        return invalid("missing graph schema path");
    };
    let Some(event_schema_path) = args.next() else {
        return invalid("missing event schema path");
    };
    let Some(report_schema_path) = args.next() else {
        return invalid("missing report schema path");
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
    let corpus_schema_bytes = match read(&corpus_schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&error),
    };
    let graph_schema_bytes = match read(&graph_schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&error),
    };
    let event_schema_bytes = match read(&event_schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&error),
    };
    let report_schema_bytes = match read(&report_schema_path) {
        Ok(bytes) => bytes,
        Err(error) => return invalid(&error),
    };

    let corpus = match parse_json(&corpus_bytes, "corpus") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };
    let corpus_schema = match parse_json(&corpus_schema_bytes, "corpus schema") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };
    let graph_schema = match parse_json(&graph_schema_bytes, "graph schema") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };
    let event_schema = match parse_json(&event_schema_bytes, "event schema") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };
    let report_schema = match parse_json(&report_schema_bytes, "report schema") {
        Ok(value) => value,
        Err(error) => return invalid(&error),
    };

    let schema_checks = [
        (
            &corpus_schema,
            "https://luminousdynamics.io/schemas/ieac-invalidation-identity-compat-corpus-v0.1.schema.json",
            "IEAC Invalidation Identity Compatibility Synthetic Corpus v0.1",
        ),
        (
            &graph_schema,
            "https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/schemas/industry-assurance-dependency-graph-v0.2.schema.json",
            "IEAC Assurance Dependency Graph v0.2",
        ),
        (
            &event_schema,
            "https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/schemas/industry-assurance-invalidation-event-v0.2.schema.json",
            "IEAC Assurance Invalidation Event v0.2",
        ),
        (
            &report_schema,
            "https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/schemas/industry-assurance-impact-report-v0.2.schema.json",
            "IEAC Assurance Dependency Impact Report v0.2",
        ),
    ];
    for (schema, expected_id, expected_title) in schema_checks {
        if schema["$schema"] != "https://json-schema.org/draft/2020-12/schema"
            || schema["$id"] != expected_id
            || schema["title"] != expected_title
        {
            return invalid("unsupported schema identity, edition or title");
        }
    }
    if corpus["corpus_id"] != CORPUS_ID {
        return invalid("unexpected corpus identity");
    }
    let structural_errors = validate_corpus(&corpus);
    if !structural_errors.is_empty() {
        return invalid(&format!(
            "invalid corpus structure: {}",
            structural_errors.join(",")
        ));
    }

    let mut failures = 0usize;
    let mut scenarios = Vec::with_capacity(corpus["cases"].as_array().unwrap().len());
    for case in corpus["cases"].as_array().unwrap() {
        let actual = evaluate_corpus_case(case);
        let passed = actual == case["expected"];
        if !passed {
            failures += 1;
        }
        scenarios.push(json!({
            "case_id": case["case_id"],
            "status": if passed { "pass" } else { "fail" },
            "expected_match": passed,
            "actual": actual
        }));
    }

    let receipt = json!({
        "schema": "org.luminousdynamics.ieac-invalidation-identity-compat.reference-receipt",
        "schema_version": "0.1.0",
        "result": if failures == 0 { "pass" } else { "fail" },
        "corpus_id": corpus["corpus_id"],
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "corpus_schema_sha256": sha256_hex(&corpus_schema_bytes),
        "graph_schema_sha256": sha256_hex(&graph_schema_bytes),
        "event_schema_sha256": sha256_hex(&event_schema_bytes),
        "report_schema_sha256": sha256_hex(&report_schema_bytes),
        "scenario_count": scenarios.len(),
        "failure_count": failures,
        "source_commit": source_commit,
        "compatibility_matrix": [
            {"graph": "v0.1", "event": "v0.1", "report": "v0.1", "policy": "preserve_without_coercion"},
            {"graph": "v0.2", "event": "v0.2", "report": "v0.2", "policy": "identity_and_content_digest_binding"},
            {"graph": "v0.2", "event": "v0.1", "report": "v0.1", "policy": "reject"},
            {"graph": "v0.1", "event": "v0.2", "report": "v0.2", "policy": "reject"}
        ],
        "native_identity_recomputed": true,
        "artifact_content_digest_recomputed_as_node_identity": false,
        "issuer_authentication": "not_performed",
        "scitt_authentication": "not_performed",
        "professional_engineering_approval": "not_assessed",
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING,
        "scenarios": scenarios
    });
    println!("{}", serde_json::to_string_pretty(&receipt).expect("receipt serializes"));
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
