use energy_dispatch_lp_validator::{validate_lp, MAX_LP_BYTES, MAX_REQUEST_BYTES};
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn read_regular_bounded(path: &Path, maximum: usize, label: &str) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("inspect {label}: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!("{label} must be a regular file"));
    }
    if metadata.len() > maximum as u64 {
        return Err(format!("{label} exceeds the {maximum}-byte limit"));
    }
    let bytes = fs::read(path).map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > maximum {
        return Err(format!("{label} exceeded the {maximum}-byte limit while reading"));
    }
    Ok(bytes)
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(request_path) = args.next() else {
        eprintln!(
            "usage: {} <request.json> <objective_direction.lp> <receipt.json>",
            program.to_string_lossy()
        );
        return Ok(ExitCode::from(64));
    };
    let Some(lp_path) = args.next() else {
        eprintln!(
            "usage: {} <request.json> <objective_direction.lp> <receipt.json>",
            program.to_string_lossy()
        );
        return Ok(ExitCode::from(64));
    };
    let Some(receipt_path) = args.next() else {
        eprintln!(
            "usage: {} <request.json> <objective_direction.lp> <receipt.json>",
            program.to_string_lossy()
        );
        return Ok(ExitCode::from(64));
    };
    if args.next().is_some() {
        return Err("expected exactly three arguments".to_owned());
    }

    let request = read_regular_bounded(
        Path::new(&request_path),
        MAX_REQUEST_BYTES,
        "dispatch request",
    )?;
    let lp = read_regular_bounded(Path::new(&lp_path), MAX_LP_BYTES, "LP model")?;
    let file_name = Path::new(&lp_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "LP filename must be valid UTF-8".to_owned())?;

    let receipt = validate_lp(&request, &lp, file_name).map_err(|error| error.to_string())?;
    let receipt_bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize translation receipt: {error}"))?;
    fs::write(&receipt_path, &receipt_bytes)
        .map_err(|error| format!("write translation receipt: {error}"))?;

    println!("equivalent={}", receipt.equivalent);
    println!("request_sha256={}", receipt.request_sha256);
    println!("lp_sha256={}", receipt.lp_sha256);
    println!("semantic_model_sha256={}", receipt.reference_model_sha256);
    println!("receipt={}", Path::new(&receipt_path).display());
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("translation validation failed: {error}");
            ExitCode::from(2)
        }
    }
}
