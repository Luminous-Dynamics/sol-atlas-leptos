use energy_dispatch_schedule_auditor::{
    evaluate_service_floor_bytes, ServiceFloorDisposition, MAX_REQUEST_BYTES, MAX_SCHEDULE_BYTES,
};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn read_bounded_bytes<R: std::io::Read>(
    reader: R,
    limit: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded {limit} byte limit while reading"));
    }
    Ok(bytes)
}

fn read_bounded(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|error| format!("open {label}: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("inspect opened {label}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{label} path must identify a regular file"));
    }
    if metadata.len() > limit as u64 {
        return Err(format!("{label} exceeds {limit} byte limit"));
    }
    read_bounded_bytes(file, limit, label)
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 6 {
        return Err(format!(
            "usage: {} <source-request.json> <dispatch-schedule.json> <service-floor-policy.json> <schedule-audit-receipt.json> <service-floor-receipt.json>",
            args.first()
                .map(|value| value.to_string_lossy())
                .unwrap_or_else(|| "energy-dispatch-service-floor-auditor".into())
        ));
    }

    let request = read_bounded(
        Path::new(&args[1]),
        MAX_REQUEST_BYTES,
        "source request",
    )?;
    let schedule = read_bounded(
        Path::new(&args[2]),
        MAX_SCHEDULE_BYTES,
        "dispatch schedule",
    )?;
    let policy = read_bounded(
        Path::new(&args[3]),
        MAX_SCHEDULE_BYTES,
        "service-floor policy",
    )?;
    let audit_receipt = read_bounded(
        Path::new(&args[4]),
        MAX_SCHEDULE_BYTES,
        "schedule audit receipt",
    )?;

    let receipt = evaluate_service_floor_bytes(&request, &schedule, &policy, &audit_receipt)
        .map_err(|error| format!("service-floor evaluation failed closed: {error}"))?;
    let output_path = PathBuf::from(&args[5]);
    if let Some(parent) = output_path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create service-floor receipt directory: {error}"))?;
    }
    let output = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize service-floor receipt: {error}"))?;
    let temporary = output_path.with_extension("json.tmp");
    std::fs::write(&temporary, output)
        .map_err(|error| format!("write temporary service-floor receipt: {error}"))?;
    std::fs::rename(&temporary, &output_path)
        .map_err(|error| format!("publish service-floor receipt: {error}"))?;

    println!("policy_id={}", receipt.policy_id);
    println!("disposition={:?}", receipt.disposition);
    println!("policy_sha256={}", receipt.policy_sha256);
    println!("request_sha256={}", receipt.request_sha256);
    println!("schedule_sha256={}", receipt.schedule_sha256);
    println!("schedule_audit_receipt_sha256={}", receipt.schedule_audit_receipt_sha256);
    println!("service_floor_auditor_binary_sha256={}", receipt.service_floor_auditor_binary_sha256);
    println!("receipt_sha256={}", receipt.receipt_sha256);
    println!("failed_findings={}", receipt.findings.iter().filter(|finding| !finding.passed).count());

    Ok(match receipt.disposition {
        ServiceFloorDisposition::Passed => ExitCode::SUCCESS,
        ServiceFloorDisposition::Failed => ExitCode::from(1),
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("service-floor auditor failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_bounded_bytes;
    use std::io::Cursor;

    #[test]
    fn bounded_stream_reader_accepts_limit_and_rejects_one_byte_over() {
        assert_eq!(
            read_bounded_bytes(Cursor::new(b"1234"), 4, "fixture").unwrap(),
            b"1234"
        );
        assert!(read_bounded_bytes(Cursor::new(b"12345"), 4, "fixture").is_err());
    }
}
