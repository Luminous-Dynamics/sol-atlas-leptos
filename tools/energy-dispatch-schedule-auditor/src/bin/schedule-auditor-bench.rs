use energy_dispatch_schedule_auditor::{
    audit_schedule_bytes_profiled, AuditTimings, MAX_REQUEST_BYTES, MAX_SCHEDULE_BYTES,
};
use serde::Serialize;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Serialize)]
struct Distribution {
    count: usize,
    p50_ns: u128,
    p95_ns: u128,
    p99_ns: u128,
    min_ns: u128,
    max_ns: u128,
}

#[derive(Debug, Serialize)]
struct BenchmarkReport {
    schema_version: String,
    warmup_count: usize,
    sample_count: usize,
    request_bytes: usize,
    schedule_bytes: usize,
    request_sha256: String,
    schedule_sha256: String,
    disposition: String,
    request_parse: Distribution,
    schedule_parse: Distribution,
    semantic_validation: Distribution,
    objective_recompute: Distribution,
    receipt_construction: Distribution,
    total_audit: Distribution,
    claim_ceiling: String,
}

fn read_bounded(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("inspect {label}: {error}"))?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err(format!("{label} must be a regular file no larger than {limit} bytes"));
    }
    let bytes = std::fs::read(path).map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded its limit while reading"));
    }
    Ok(bytes)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn distribution(samples: &[u128]) -> Distribution {
    Distribution {
        count: samples.len(),
        p50_ns: percentile(samples, 50),
        p95_ns: percentile(samples, 95),
        p99_ns: percentile(samples, 99),
        min_ns: *samples.iter().min().expect("samples are non-empty"),
        max_ns: *samples.iter().max().expect("samples are non-empty"),
    }
}

fn collect(samples: &[AuditTimings], select: impl Fn(&AuditTimings) -> u128) -> Vec<u128> {
    samples.iter().map(select).collect()
}

fn run() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    if !(3..=4).contains(&args.len()) {
        return Err(format!(
            "usage: {} <request.json> <schedule.json> [sample-count]",
            args.first().map(String::as_str).unwrap_or("schedule-auditor-bench")
        ));
    }
    let sample_count = if args.len() == 4 {
        args[3]
            .parse::<usize>()
            .map_err(|error| format!("invalid sample count: {error}"))?
    } else {
        500
    };
    if !(100..=10_000).contains(&sample_count) {
        return Err("sample count must be between 100 and 10000".to_owned());
    }
    let request = read_bounded(Path::new(&args[1]), MAX_REQUEST_BYTES, "request")?;
    let schedule = read_bounded(Path::new(&args[2]), MAX_SCHEDULE_BYTES, "schedule")?;

    const WARMUPS: usize = 50;
    for _ in 0..WARMUPS {
        let result = audit_schedule_bytes_profiled(&request, &schedule)
            .map_err(|error| format!("warmup audit failed: {error}"))?;
        if result.receipt.disposition
            != energy_dispatch_schedule_auditor::AuditDisposition::Accepted
        {
            return Err("benchmark input schedule is not accepted".to_owned());
        }
    }

    let mut timings = Vec::with_capacity(sample_count);
    let mut final_receipt = None;
    for _ in 0..sample_count {
        let result = audit_schedule_bytes_profiled(&request, &schedule)
            .map_err(|error| format!("sample audit failed: {error}"))?;
        if result.receipt.disposition
            != energy_dispatch_schedule_auditor::AuditDisposition::Accepted
        {
            return Err("benchmark input schedule is not accepted".to_owned());
        }
        timings.push(result.timings);
        final_receipt = Some(result.receipt);
    }
    let receipt = final_receipt.expect("at least 100 measured samples");

    let report = BenchmarkReport {
        schema_version: "energy-dispatch-schedule-audit-benchmark-v0.1".to_owned(),
        warmup_count: WARMUPS,
        sample_count,
        request_bytes: request.len(),
        schedule_bytes: schedule.len(),
        request_sha256: receipt.request_sha256,
        schedule_sha256: receipt.schedule_sha256,
        disposition: "accepted".to_owned(),
        request_parse: distribution(&collect(
            &timings,
            |timing| timing.request_parse_ns,
        )),
        schedule_parse: distribution(&collect(
            &timings,
            |timing| timing.schedule_parse_ns,
        )),
        semantic_validation: distribution(&collect(
            &timings,
            |timing| timing.semantic_validation_ns,
        )),
        objective_recompute: distribution(&collect(
            &timings,
            |timing| timing.objective_recompute_ns,
        )),
        receipt_construction: distribution(&collect(
            &timings,
            |timing| timing.receipt_construction_ns,
        )),
        total_audit: distribution(&collect(
            &timings,
            |timing| timing.total_ns,
        )),
        claim_ceiling: concat!(
            "in-process schedule audit phase timings for the supplied synthetic fixture; ",
            "not a real-site workload, solver comparison, or general performance claim"
        )
        .to_owned(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("serialize benchmark: {error}"))?
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("schedule auditor benchmark failed: {error}");
            ExitCode::from(2)
        }
    }
}
