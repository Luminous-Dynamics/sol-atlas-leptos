use energy_dispatch_schedule_auditor::{
    audit_schedule_bytes, AuditDisposition, MAX_REQUEST_BYTES, MAX_SCHEDULE_BYTES,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn read_bounded(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("inspect {label} path: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!("{label} path must identify a regular file"));
    }
    if metadata.len() > limit as u64 {
        return Err(format!("{label} exceeds {limit} byte limit"));
    }
    let bytes = std::fs::read(path).map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded {limit} byte limit while reading"));
    }
    Ok(bytes)
}

fn run() -> Result<ExitCode, String> {
    let mut args = std::env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(request_path) = args.next() else {
        return Err(format!(
            "usage: {} <source-request.json> <dispatch-schedule.json> <audit-receipt.json>",
            program.to_string_lossy()
        ));
    };
    let Some(schedule_path) = args.next() else {
        return Err("dispatch schedule path is required".to_owned());
    };
    let Some(receipt_path) = args.next() else {
        return Err("audit receipt output path is required".to_owned());
    };
    if args.next().is_some() {
        return Err("expected exactly three file paths".to_owned());
    }

    let request = read_bounded(
        Path::new(&request_path),
        MAX_REQUEST_BYTES,
        "source request",
    )?;
    let schedule = read_bounded(
        Path::new(&schedule_path),
        MAX_SCHEDULE_BYTES,
        "dispatch schedule",
    )?;
    let receipt = audit_schedule_bytes(&request, &schedule)
        .map_err(|error| format!("schedule audit could not be completed: {error}"))?;

    let output_path = PathBuf::from(OsString::from(receipt_path));
    if let Some(parent) = output_path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create receipt directory: {error}"))?;
    }
    let output = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize audit receipt: {error}"))?;
    std::fs::write(&output_path, &output)
        .map_err(|error| format!("write audit receipt: {error}"))?;

    println!("case_id={}", receipt.case_id);
    println!("disposition={:?}", receipt.disposition);
    println!("request_sha256={}", receipt.request_sha256);
    println!("schedule_sha256={}", receipt.schedule_sha256);
    println!("receipt_sha256={}", receipt.receipt_sha256);
    println!(
        "failed_predicates={}",
        receipt.findings.iter().filter(|finding| !finding.passed).count()
    );

    Ok(match receipt.disposition {
        AuditDisposition::Accepted => ExitCode::SUCCESS,
        AuditDisposition::Rejected => ExitCode::from(1),
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("energy dispatch schedule audit failed closed: {error}");
            ExitCode::from(2)
        }
    }
}
