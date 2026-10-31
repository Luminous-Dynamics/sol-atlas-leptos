use industry_twin_interface_validator::{validate_manifest, VALIDATOR_VERSION};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run() -> Result<(Value, bool, Option<String>), String> {
    let mut args = env::args().skip(1);
    let input_path = args
        .next()
        .ok_or("usage: industry-twin-interface-validator <manifest.json> [new-report.json]")?;
    let output_path = args.next();
    if args.next().is_some() {
        return Err("too many arguments".to_owned());
    }
    let bytes = fs::read(&input_path)
        .map_err(|error| format!("cannot read {input_path}: {error}"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid JSON in {input_path}: {error}"))?;
    let issues = validate_manifest(&value);
    let valid = issues.is_empty();
    let schema_bytes = include_bytes!(
        "../../../schemas/industry-twin-interface-manifest-v0.1.schema.json"
    );
    let report = json!({
        "schema_version": "luminous.industry-twin-interface-validation-report.v0.1",
        "manifest_id": value["manifest_id"],
        "manifest_sha256": digest(&bytes),
        "schema_sha256": digest(schema_bytes),
        "validator_version": VALIDATOR_VERSION,
        "evaluation_mode": "synthetic_fixture",
        "claim_ceiling": "synthetic_fixture",
        "standard_conformity_claimed": false,
        "contract_valid": valid,
        "status": if valid {
            "INTERFACE_CONTRACT_VALID_NOT_STANDARD_CONFORMANT"
        } else {
            "INTERFACE_CONTRACT_INVALID"
        },
        "issue_count": issues.len(),
        "issues": issues
            .iter()
            .map(|issue| json!({"code": issue.code, "path": issue.path}))
            .collect::<Vec<_>>()
    });
    Ok((report, valid, output_path))
}

fn main() {
    match run() {
        Ok((report, valid, output_path)) => {
            let bytes = serde_json::to_vec_pretty(&report).expect("validation report serializes");
            if let Some(path) = output_path {
                match OpenOptions::new().write(true).create_new(true).open(&path) {
                    Ok(mut file) => {
                        if let Err(error) = file
                            .write_all(&bytes)
                            .and_then(|_| file.write_all(b"\n"))
                        {
                            eprintln!("cannot write {path}: {error}");
                            std::process::exit(2);
                        }
                    }
                    Err(error) => {
                        eprintln!("refusing to overwrite or cannot create {path}: {error}");
                        std::process::exit(2);
                    }
                }
            }
            println!("{}", String::from_utf8_lossy(&bytes));
            if !valid {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
