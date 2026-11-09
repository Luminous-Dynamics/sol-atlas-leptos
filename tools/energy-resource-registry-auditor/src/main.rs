use energy_resource_registry_auditor::{audit_bytes, MAX_INPUT_BYTES};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

fn read_bounded(path: &PathBuf) -> Result<Vec<u8>, String> {
    let file = File::open(path)
        .map_err(|cause| format!("open {}: {cause}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|cause| format!("inspect {}: {cause}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a regular file", path.display()));
    }
    if metadata.len() > MAX_INPUT_BYTES as u64 {
        return Err(format!(
            "{} exceeds the {MAX_INPUT_BYTES}-byte input limit",
            path.display()
        ));
    }

    let mut bytes = Vec::new();
    file.take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|cause| format!("read {}: {cause}", path.display()))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(format!(
            "{} exceeded the {MAX_INPUT_BYTES}-byte input limit while reading",
            path.display()
        ));
    }
    Ok(bytes)
}

fn run() -> Result<u8, String> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.len() != 3 {
        return Err(
            "usage: energy-source-registry-auditor <schema.json> <registry.json> <receipt.json>"
                .to_owned(),
        );
    }

    let schema_path = PathBuf::from(&arguments[0]);
    let registry_path = PathBuf::from(&arguments[1]);
    let receipt_path = PathBuf::from(&arguments[2]);

    let schema_bytes = read_bounded(&schema_path)?;
    let registry_bytes = read_bounded(&registry_path)?;
    let receipt = audit_bytes(&schema_bytes, &registry_bytes)
        .map_err(|cause| format!("audit could not be completed: {cause}"))?;

    let receipt_bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|cause| format!("serialize audit receipt: {cause}"))?;
    let mut output = File::create(&receipt_path)
        .map_err(|cause| format!("create {}: {cause}", receipt_path.display()))?;
    output
        .write_all(&receipt_bytes)
        .map_err(|cause| format!("write {}: {cause}", receipt_path.display()))?;
    output
        .write_all(b"\n")
        .map_err(|cause| format!("finish {}: {cause}", receipt_path.display()))?;

    println!(
        "{}: {} source records; receipt {}",
        receipt.disposition, receipt.entry_count, receipt.receipt_sha256
    );

    Ok(if receipt.disposition == "accepted" {
        0
    } else {
        1
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}
