use methane_action_contract::census::{BenchmarkCase, SystemSubmission};
use methane_action_contract::comparison::{PairedComparisonReport, compare_submissions};
use methane_action_contract::sha256_hex;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::process::ExitCode;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComparisonInput {
    schema_version: u32,
    cases: Vec<BenchmarkCase>,
    baseline: SystemSubmission,
    candidate: SystemSubmission,
    bootstrap_replicates: usize,
    seed: u64,
}

#[derive(Debug, Serialize)]
struct ComparisonReceipt {
    receipt_schema_version: u32,
    evaluator: &'static str,
    evaluator_version: &'static str,
    input_sha256: String,
    input_bytes: usize,
    comparison_status: &'static str,
    baseline_system_id: String,
    candidate_system_id: String,
    comparison: PairedComparisonReport,
}

fn main() -> ExitCode {
    let mut input_bytes = Vec::new();
    if let Err(error) = io::stdin().lock().read_to_end(&mut input_bytes) {
        eprintln!("methane-compare: failed to read stdin: {error}");
        return ExitCode::FAILURE;
    }
    if input_bytes.is_empty() {
        eprintln!("methane-compare: stdin is empty; expected a JSON comparison request");
        return ExitCode::FAILURE;
    }

    let input: ComparisonInput = match serde_json::from_slice(&input_bytes) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("methane-compare: invalid comparison JSON: {error}");
            return ExitCode::FAILURE;
        }
    };
    if input.schema_version != 1 {
        eprintln!(
            "methane-compare: unsupported schema_version {}; supported version is 1",
            input.schema_version
        );
        return ExitCode::FAILURE;
    }

    let input_sha256 = sha256_hex(&input_bytes);
    let report = compare_submissions(
        &input.cases,
        &input.baseline,
        &input.candidate,
        input.bootstrap_replicates,
        input.seed,
    );
    let valid = report.is_valid();
    let receipt = ComparisonReceipt {
        receipt_schema_version: 1,
        evaluator: "methane-action-contract",
        evaluator_version: env!("CARGO_PKG_VERSION"),
        input_sha256,
        input_bytes: input_bytes.len(),
        comparison_status: if valid { "valid" } else { "invalid" },
        baseline_system_id: input.baseline.system_id,
        candidate_system_id: input.candidate.system_id,
        comparison: report,
    };

    let stdout = io::stdout();
    let mut writer = stdout.lock();
    if let Err(error) = serde_json::to_writer_pretty(&mut writer, &receipt) {
        eprintln!("methane-compare: failed to write JSON receipt: {error}");
        return ExitCode::FAILURE;
    }
    if let Err(error) = writeln!(writer) {
        eprintln!("methane-compare: failed to finish JSON receipt: {error}");
        return ExitCode::FAILURE;
    }

    if valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}
