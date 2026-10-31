use industry_runtime_admission::{
    evaluate_admission, independent_check, report_digest_is_valid, seal_report, validate_runtime_report,
};
use serde_json::Value;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

fn write_new_report(path: &Path, report: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(report).map_err(|error| error.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("refusing to overwrite or cannot create {}: {error}", path.display()))?;
    file.write_all(&bytes).map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    file.write_all(b"\n").map_err(|error| format!("cannot finish {}: {error}", path.display()))?;
    file.sync_all().map_err(|error| format!("cannot sync {}: {error}", path.display()))?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: industry-runtime-admission <profile.json> <request.json> <new-report.json>");
        std::process::exit(2);
    }
    let profile_path = Path::new(&args[0]);
    let request_path = Path::new(&args[1]);
    let report_path = Path::new(&args[2]);

    if report_path == profile_path || report_path == request_path {
        eprintln!("refusing to use an input path as the report destination");
        std::process::exit(2);
    }

    let profile = match read_json(profile_path) {
        Ok(value) => value,
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    };
    let request = match read_json(request_path) {
        Ok(value) => value,
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    };

    let mut report = evaluate_admission(&profile, &request);
    let checker_errors = independent_check(&profile, &request, &report);
    let checker_status = if checker_errors.is_empty() { "passed" } else { "failed" };
    seal_report(&mut report, checker_status, &checker_errors);

    let report_issues = validate_runtime_report(&report);
    if !report_issues.is_empty() {
        seal_report(&mut report, "failed", &report_issues);
    }

    if !validate_runtime_report(&report).is_empty() || !report_digest_is_valid(&report) {
        eprintln!("internal report digest self-check failed");
        std::process::exit(2);
    }
    if let Err(error) = write_new_report(report_path, &report) {
        eprintln!("{error}");
        std::process::exit(2);
    }
    println!("{}", serde_json::to_string_pretty(&report).expect("report serializes"));

    if !checker_errors.is_empty() {
        eprintln!("independent report checker failed: {checker_errors:?}");
        std::process::exit(2);
    }
    if report["admission"].as_str().is_some_and(|value| value.starts_with("admit_")) {
        std::process::exit(0);
    }
    std::process::exit(1);
}
