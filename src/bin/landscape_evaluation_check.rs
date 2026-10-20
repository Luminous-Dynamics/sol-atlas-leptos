// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Native, deterministic checker for landscape evaluation-plan artifacts.
//!
//! This checker validates document structure and policy-contract invariants. It
//! does not resolve record references, verify legal consent, assess scientific
//! adequacy, or establish that an intervention had an effect.

#[path = "../landscape_evaluation.rs"]
mod landscape_evaluation;

use landscape_evaluation::{INTERPRETATION_LIMIT, PLAN_SCHEMA_URI, validate_plan};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::process::ExitCode;

const REPORT_SCHEMA: &str = "luminous.landscape-evaluation-validation-report.v0";
const REPORT_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-evaluation-validation-report-v0.schema.json";
const MAX_PLAN_BYTES: usize = 8 * 1024 * 1024;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return ExitCode::SUCCESS;
    }
    if args.len() != 1 {
        eprintln!("landscape-evaluation-check: expected exactly one plan file");
        print_usage();
        return ExitCode::from(64);
    }

    let path = &args[0];
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("landscape-evaluation-check: could not read input: {error}");
            return ExitCode::from(66);
        }
    };

    let report = validate_bytes(&bytes);
    let is_valid = report["valid"].as_bool() == Some(true);
    match serde_json::to_string_pretty(&report) {
        Ok(serialized) => println!("{serialized}"),
        Err(error) => {
            eprintln!("landscape-evaluation-check: could not serialize report: {error}");
            return ExitCode::from(70);
        }
    }

    if is_valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn print_usage() {
    println!(
        "Landscape evaluation-plan contract checker\n\
         Usage: cargo run --features landscape-eval-check --bin landscape-evaluation-check -- PLAN.json\n\
         The checker emits a deterministic JSON report to stdout.\n\
         Exit code 0 means the plan passed structural validation; exit code 2 means it failed.\n\
         Validation is not proof of scientific adequacy, legal consent, impact, or causality."
    );
}

/// Validate the exact input bytes and include their SHA-256 in every report.
fn validate_bytes(bytes: &[u8]) -> Value {
    let input_sha256 = format!("{:x}", Sha256::digest(bytes));
    let mut errors = Vec::new();
    let mut evaluation_id = Value::Null;

    if bytes.len() > MAX_PLAN_BYTES {
        errors.push(format!(
            "$: input size {} exceeds checker limit of {} bytes",
            bytes.len(),
            MAX_PLAN_BYTES
        ));
    } else {
        match serde_json::from_slice::<Value>(bytes) {
            Ok(plan) => {
                evaluation_id = plan
                    .get("evaluation_id")
                    .and_then(Value::as_str)
                    .map(|id| Value::String(id.to_string()))
                    .unwrap_or(Value::Null);
                if let Err(violations) = validate_plan(&plan) {
                    errors = violations;
                }
            }
            Err(error) => errors.push(format!("$: invalid JSON: {error}")),
        }
    }

    json!({
        "report_schema": REPORT_SCHEMA,
        "schema_uri": REPORT_SCHEMA_URI,
        "plan_schema_uri": PLAN_SCHEMA_URI,
        "input_sha256": input_sha256,
        "input_bytes": bytes.len(),
        "evaluation_id": evaluation_id,
        "valid": errors.is_empty(),
        "errors": errors,
        "interpretation_limit": INTERPRETATION_LIMIT
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_fixture() -> Vec<u8> {
        include_bytes!("../../fixtures/landscape/evaluation-plan-v0.json").to_vec()
    }

    #[test]
    fn report_schema_contract_matches_cli_output() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../schemas/landscape-evaluation-validation-report-v0.schema.json"
        ))
        .unwrap();
        let report = validate_bytes(&valid_fixture());
        assert_eq!(schema["$id"], REPORT_SCHEMA_URI);
        assert_eq!(schema["properties"]["report_schema"]["const"], REPORT_SCHEMA);
        assert_eq!(report["report_schema"], schema["properties"]["report_schema"]["const"]);
        assert_eq!(report["schema_uri"], schema["properties"]["schema_uri"]["const"]);
    }

    #[test]
    fn valid_fixture_produces_repeatable_success_report() {
        let bytes = valid_fixture();
        let first = validate_bytes(&bytes);
        let second = validate_bytes(&bytes);

        assert_eq!(first, second);
        assert_eq!(first["valid"], true);
        assert_eq!(first["evaluation_id"], "eval-watershed-demo-v0");
        assert_eq!(first["input_bytes"], json!(bytes.len()));
        assert_eq!(first["input_sha256"].as_str().unwrap().len(), 64);
        assert_eq!(first["errors"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn invalid_json_produces_a_hashed_failure_report() {
        let bytes = b"{invalid json";
        let report = validate_bytes(bytes);

        assert_eq!(report["valid"], false);
        assert_eq!(report["input_sha256"].as_str().unwrap().len(), 64);
        assert!(report["errors"][0].as_str().unwrap().contains("invalid JSON"));
    }

    #[test]
    fn unsupported_plan_is_rejected_without_losing_input_digest() {
        let mut plan: Value = serde_json::from_slice(&valid_fixture()).unwrap();
        plan["project_scope"] = json!("planetary");
        plan["unexpected"] = json!(true);
        let bytes = serde_json::to_vec(&plan).unwrap();
        let report = validate_bytes(&bytes);

        assert_eq!(report["valid"], false);
        assert_eq!(report["input_sha256"], format!("{:x}", Sha256::digest(&bytes)));
        let errors = report["errors"].as_array().unwrap();
        assert!(errors.iter().any(|error| {
            error.as_str().unwrap().contains("unknown field")
        }));
        assert!(errors.iter().any(|error| {
            error.as_str().unwrap().contains("unsupported planning scope")
        }));
    }

    #[test]
    fn overlarge_input_is_rejected_before_json_parsing() {
        let bytes = vec![b' '; MAX_PLAN_BYTES + 1];
        let report = validate_bytes(&bytes);

        assert_eq!(report["valid"], false);
        assert!(
            report["errors"][0]
                .as_str()
                .unwrap()
                .contains("exceeds checker limit")
        );
    }
}
