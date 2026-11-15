use highveld_panel_thread_qualifier::evaluate_bytes;
use std::env;
use std::fs;
use std::process::ExitCode;

fn usage() -> &'static str {
    "Usage: highveld-panel-thread-qualifier --cases <corpus.json> --submission <submission.json> [--output <report.json>]"
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut cases_path: Option<String> = None;
    let mut submission_path: Option<String> = None;
    let mut output_path: Option<String> = None;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(());
            }
            "--cases" => {
                cases_path = Some(args.next().ok_or("--cases requires a path")?);
            }
            "--submission" => {
                submission_path = Some(args.next().ok_or("--submission requires a path")?);
            }
            "--output" => {
                output_path = Some(args.next().ok_or("--output requires a path")?);
            }
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }

    let cases_path = cases_path.ok_or_else(|| usage().to_owned())?;
    let submission_path = submission_path.ok_or_else(|| usage().to_owned())?;
    let cases =
        fs::read(&cases_path).map_err(|error| format!("cannot read {cases_path}: {error}"))?;
    let submission = fs::read(&submission_path)
        .map_err(|error| format!("cannot read {submission_path}: {error}"))?;

    let report = evaluate_bytes(&cases, &submission)
        .map_err(|error| format!("evaluation rejected: {error}"))?;
    let mut encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode report: {error}"))?;
    encoded.push(b'\n');

    if let Some(path) = output_path {
        fs::write(&path, &encoded).map_err(|error| format!("cannot write {path}: {error}"))?;
    } else {
        print!(
            "{}",
            String::from_utf8(encoded).map_err(|error| error.to_string())?
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
