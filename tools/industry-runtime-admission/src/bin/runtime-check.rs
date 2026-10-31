use industry_runtime_admission::{
    independent_check, report_digest_is_valid, validate_runtime_report,
};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: industry-runtime-check <profile.json> <request.json> <sealed-report.json>");
        std::process::exit(2);
    }

    let profile = match read_json(Path::new(&args[0])) {
        Ok(value) => value,
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    };
    let request = match read_json(Path::new(&args[1])) {
        Ok(value) => value,
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    };
    let report = match read_json(Path::new(&args[2])) {
        Ok(value) => value,
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    };

    let schema_errors = validate_runtime_report(&report);
    let replay_errors = independent_check(&profile, &request, &report);
    let digest_valid = report_digest_is_valid(&report);

    if !schema_errors.is_empty() || !replay_errors.is_empty() || !digest_valid {
        eprintln!("schema errors: {schema_errors:#?}");
        eprintln!("replay errors: {replay_errors:#?}");
        eprintln!("report digest valid: {digest_valid}");
        std::process::exit(1);
    }

    println!("independent runtime admission report verification: PASS");
}
