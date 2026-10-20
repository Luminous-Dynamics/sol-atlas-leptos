use methane_action_contract::census::{
    BenchmarkCase, EvaluationReport, SystemSubmission, evaluate_census,
};
use methane_action_contract::{executable_sha256, sha256_hex};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::process::ExitCode;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchmarkInput {
    schema_version: u32,
    cases: Vec<BenchmarkCase>,
    submission: SystemSubmission,
}

#[derive(Debug, Serialize)]
struct RunReceipt {
    receipt_schema_version: u32,
    evaluator: &'static str,
    evaluator_version: &'static str,
    evaluator_binary_sha256: String,
    input_sha256: String,
    input_bytes: usize,
    evaluation_status: &'static str,
    system_id: String,
    system_version: String,
    build_sha256: String,
    configuration_sha256: String,
    evaluation: EvaluationReport,
}

fn main() -> ExitCode {
    let mut input_bytes = Vec::new();
    if let Err(error) = io::stdin().lock().read_to_end(&mut input_bytes) {
        eprintln!("methane-census: failed to read stdin: {error}");
        return ExitCode::FAILURE;
    }
    if input_bytes.is_empty() {
        eprintln!("methane-census: stdin is empty; expected a JSON benchmark input");
        return ExitCode::FAILURE;
    }

    let input: BenchmarkInput = match serde_json::from_slice(&input_bytes) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("methane-census: invalid benchmark JSON: {error}");
            return ExitCode::FAILURE;
        }
    };
    if input.schema_version != 1 {
        eprintln!(
            "methane-census: unsupported schema_version {}; supported version is 1",
            input.schema_version
        );
        return ExitCode::FAILURE;
    }

    let input_sha256 = sha256_hex(&input_bytes);
    let evaluator_binary_sha256 = match executable_sha256() {
        Ok(digest) => digest,
        Err(error) => {
            eprintln!("methane-census: failed to hash evaluator executable: {error}");
            return ExitCode::FAILURE;
        }
    };
    let report = evaluate_census(&input.cases, &input.submission);
    let valid = report.is_valid();
    let receipt = RunReceipt {
        receipt_schema_version: 1,
        evaluator: "methane-action-contract",
        evaluator_version: env!("CARGO_PKG_VERSION"),
        evaluator_binary_sha256,
        input_sha256,
        input_bytes: input_bytes.len(),
        evaluation_status: if valid { "valid" } else { "invalid" },
        system_id: input.submission.system_id,
        system_version: input.submission.version,
        build_sha256: input.submission.build_sha256,
        configuration_sha256: input.submission.configuration_sha256,
        evaluation: report,
    };

    let stdout = io::stdout();
    let mut writer = stdout.lock();
    if let Err(error) = serde_json::to_writer_pretty(&mut writer, &receipt) {
        eprintln!("methane-census: failed to write JSON receipt: {error}");
        return ExitCode::FAILURE;
    }
    if let Err(error) = writeln!(writer) {
        eprintln!("methane-census: failed to finish JSON receipt: {error}");
        return ExitCode::FAILURE;
    }

    if valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}
