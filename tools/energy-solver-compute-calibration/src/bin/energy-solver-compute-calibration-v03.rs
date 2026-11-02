#![forbid(unsafe_code)]
//! Strict v0.3 anytime/audit trace summary CLI.

use std::path::Path;
use std::process::ExitCode;

pub mod trace_support {
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub enum MeasurementClass {
        Synthetic,
        Measured,
    }

    impl MeasurementClass {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Synthetic => "synthetic",
                Self::Measured => "measured",
            }
        }
    }

    pub const MAX_TRACE_CSV_BYTES: usize = 64 * 1024 * 1024;
    pub const P50_MIN_SAMPLES: usize = 5;
    pub const P95_MIN_SAMPLES: usize = 20;

    pub fn safe_identifier(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 128
            && value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
    }

    pub fn is_sha256_hex(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    pub mod trace_v0_3 {
        include!("../trace_v0_3.rs");
    }
}

use trace_support::trace_v0_3::{parse_trace_csv_v03, summarize_traces_v03};

const SUMMARY_HEADER: &str = "cohort_id,samples,exact_complete,feasible_unproven,proven_infeasible,limit_without_candidate,timed_out,cancelled,errors,candidates,audit_attempts,audit_accepted,audit_rejected,audit_unavailable,audit_not_run,service_floor_passed,service_floor_failed,service_floor_unavailable,service_floor_not_evaluated,decision_ready,first_candidate_samples,first_candidate_p50_ns,first_candidate_p95_ns,first_audit_accept_samples,first_audit_accept_p50_ns,first_audit_accept_p95_ns,total_prefix_visits,audit_accepted_per_million_prefix_visits,decision_ready_per_million_prefix_visits,service_floor_samples,service_floor_elapsed_p50_ns,service_floor_elapsed_p95_ns,decision_ready_latency_samples,decision_ready_latency_p50_ns,decision_ready_latency_p95_ns";

fn optional(value: Option<u64>) -> String {
    value.map(|n| n.to_string()).unwrap_or_else(|| "NA".to_owned())
}

fn summarize(path: &Path) -> Result<String, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("inspect v0.3 trace: {error}"))?;
    if !metadata.is_file() || metadata.len() > trace_support::MAX_TRACE_CSV_BYTES as u64 {
        return Err("v0.3 trace must be a regular file within the declared byte limit".to_owned());
    }
    let input = std::fs::read_to_string(path)
        .map_err(|error| format!("read v0.3 trace: {error}"))?;
    let traces = parse_trace_csv_v03(&input)
        .map_err(|error| format!("parse v0.3 trace: {error}"))?;
    let summaries = summarize_traces_v03(&traces)
        .map_err(|error| format!("aggregate v0.3 trace: {error}"))?;

    let mut output = String::new();
    output.push_str(SUMMARY_HEADER);
    output.push('\n');
    for row in summaries {
        let fields = [
            row.cohort_id,
            row.sample_count.to_string(),
            row.exact_complete_count.to_string(),
            row.feasible_unproven_count.to_string(),
            row.proven_infeasible_count.to_string(),
            row.limit_without_candidate_count.to_string(),
            row.timed_out_count.to_string(),
            row.cancelled_count.to_string(),
            row.error_count.to_string(),
            row.candidate_count.to_string(),
            row.audit_attempt_count.to_string(),
            row.audit_accepted_count.to_string(),
            row.audit_rejected_count.to_string(),
            row.audit_unavailable_count.to_string(),
            row.audit_not_run_count.to_string(),
            row.service_floor_passed_count.to_string(),
            row.service_floor_failed_count.to_string(),
            row.service_floor_unavailable_count.to_string(),
            row.service_floor_not_evaluated_count.to_string(),
            row.decision_ready_count.to_string(),
            row.first_candidate_sample_count.to_string(),
            optional(row.first_candidate_p50_ns),
            optional(row.first_candidate_p95_ns),
            row.first_audit_accept_sample_count.to_string(),
            optional(row.first_audit_accept_p50_ns),
            optional(row.first_audit_accept_p95_ns),
            optional(row.total_prefix_visits),
            optional(row.audit_accepted_per_million_prefix_visits),
            optional(row.decision_ready_per_million_prefix_visits),
            row.service_floor_sample_count.to_string(),
            optional(row.service_floor_elapsed_p50_ns),
            optional(row.service_floor_elapsed_p95_ns),
            row.decision_ready_latency_sample_count.to_string(),
            optional(row.decision_ready_latency_p50_ns),
            optional(row.decision_ready_latency_p95_ns),
        ];
        if fields.iter().any(|value| value.contains(',') || value.contains('\n') || value.contains('\r')) {
            return Err("v0.3 summary contains an invalid CSV identifier".to_owned());
        }
        output.push_str(&fields.join(","));
        output.push('\n');
    }
    Ok(output)
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(format!(
            "usage: {} <trace-v0.3.csv>",
            args.first()
                .map(|value| value.to_string_lossy())
                .unwrap_or_else(|| "energy-solver-compute-calibration-v03".into())
        ));
    }
    let output = summarize(Path::new(&args[1]))?;
    print!("{output}");
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("v0.3 trace aggregation failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_header_is_unique_and_has_35_columns() {
        assert_eq!(SUMMARY_HEADER.split(',').count(), 35);
    }

    #[test]
    fn optional_metric_is_not_fabricated_as_zero() {
        assert_eq!(optional(None), "NA");
        assert_eq!(optional(Some(0)), "0");
    }
}
