use industry_assurance_validator::{build_report, validate_profile};
use serde_json::Value;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.len() > 2 {
        eprintln!("usage: industry-assurance-validate <profile.json> [report.json]");
        return ExitCode::from(2);
    }

    let profile_path = &args[0];
    let raw = match fs::read_to_string(profile_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("cannot read {profile_path}: {error}");
            return ExitCode::from(2);
        }
    };
    let profile: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid profile JSON: {error}");
            return ExitCode::from(2);
        }
    };

    let issues = validate_profile(&profile);
    let report = build_report(&profile, &issues);
    let rendered = match serde_json::to_string_pretty(&report) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("cannot serialize validation report: {error}");
            return ExitCode::from(2);
        }
    };

    if let Some(path) = args.get(1) {
        if let Err(error) = fs::write(path, format!("{rendered}\n")) {
            eprintln!("cannot write {path}: {error}");
            return ExitCode::from(2);
        }
    }

    println!("{rendered}");
    if issues.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
