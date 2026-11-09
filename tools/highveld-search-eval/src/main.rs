use highveld_search_eval::evaluate_bundle;
use serde_json::Value;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("result=invalid");
        eprintln!("violation=BENCHMARK_BUNDLE_PATH_REQUIRED");
        return ExitCode::FAILURE;
    };

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("result=invalid");
            eprintln!("violation=BENCHMARK_BUNDLE_READ_FAILED");
            eprintln!("detail={error}");
            return ExitCode::FAILURE;
        }
    };

    let bundle: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("result=invalid");
            eprintln!("violation=BENCHMARK_BUNDLE_JSON_INVALID");
            eprintln!("detail={error}");
            return ExitCode::FAILURE;
        }
    };

    let report = evaluate_bundle(&bundle);
    match serde_json::to_string_pretty(&report.to_value()) {
        Ok(serialized) => println!("{serialized}"),
        Err(error) => {
            eprintln!("result=invalid");
            eprintln!("violation=REPORT_SERIALIZATION_FAILED");
            eprintln!("detail={error}");
            return ExitCode::FAILURE;
        }
    }

    if report.is_valid() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
