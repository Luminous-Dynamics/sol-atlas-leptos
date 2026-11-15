use highveld_panel_thread_qualifier::bridge::evaluate_bridge_bytes;
use std::env;
use std::fs;
use std::process::ExitCode;

fn usage() -> &'static str {
    "Usage: highveld-bridge-qualifier --input <bridge.json> [--output <report.json>]"
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut input_path: Option<String> = None;
    let mut output_path: Option<String> = None;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(());
            }
            "--input" => {
                input_path = Some(args.next().ok_or("--input requires a path")?);
            }
            "--output" => {
                output_path = Some(args.next().ok_or("--output requires a path")?);
            }
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }

    let input_path = input_path.ok_or_else(|| usage().to_owned())?;
    let input = fs::read(&input_path)
        .map_err(|error| format!("cannot read {input_path}: {error}"))?;
    let report = evaluate_bridge_bytes(&input)
        .map_err(|error| format!("bridge input rejected: {error}"))?;
    let mut encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode bridge report: {error}"))?;
    encoded.push(b'\n');

    if let Some(path) = output_path {
        fs::write(&path, &encoded)
            .map_err(|error| format!("cannot write {path}: {error}"))?;
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
