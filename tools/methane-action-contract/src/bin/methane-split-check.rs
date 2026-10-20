use methane_action_contract::sha256_hex;
use methane_action_contract::split::{SplitManifest, SplitValidationReport, validate_split_manifest};
use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Serialize)]
struct SplitReceipt {
    receipt_schema_version: u32,
    evaluator: &'static str,
    evaluator_version: &'static str,
    corpus_path: String,
    manifest_path: String,
    corpus_sha256: Option<String>,
    manifest_sha256: String,
    validation_status: &'static str,
    validation: SplitValidationReport,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: methane-split-check <corpus.json> <split-manifest.json>");
        return ExitCode::FAILURE;
    }
    let corpus_path = PathBuf::from(&args[0]);
    let manifest_path = PathBuf::from(&args[1]);

    let corpus_bytes = match fs::read(&corpus_path) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        Ok(_) => {
            eprintln!("methane-split-check: corpus file is empty");
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("methane-split-check: failed to read corpus: {error}");
            return ExitCode::FAILURE;
        }
    };
    let manifest_bytes = match fs::read(&manifest_path) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        Ok(_) => {
            eprintln!("methane-split-check: split manifest file is empty");
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("methane-split-check: failed to read manifest: {error}");
            return ExitCode::FAILURE;
        }
    };
    let manifest: SplitManifest = match serde_json::from_slice(&manifest_bytes) {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("methane-split-check: invalid manifest JSON: {error}");
            return ExitCode::FAILURE;
        }
    };

    let report = validate_split_manifest(&corpus_bytes, &manifest);
    let valid = report.is_valid();
    let receipt = SplitReceipt {
        receipt_schema_version: 1,
        evaluator: "methane-action-contract",
        evaluator_version: env!("CARGO_PKG_VERSION"),
        corpus_path: corpus_path.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        corpus_sha256: report.computed_corpus_sha256.clone(),
        manifest_sha256: sha256_hex(&manifest_bytes),
        validation_status: if valid { "valid" } else { "invalid" },
        validation: report,
    };
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    if let Err(error) = serde_json::to_writer_pretty(&mut writer, &receipt) {
        eprintln!("methane-split-check: failed to write receipt: {error}");
        return ExitCode::FAILURE;
    }
    if let Err(error) = writeln!(writer) {
        eprintln!("methane-split-check: failed to finish receipt: {error}");
        return ExitCode::FAILURE;
    }
    if valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}
