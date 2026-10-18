use std::{env, process};

use city_belonging_admission::verify_files;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 || args[1] != "verify" {
        eprintln!("usage: city-belonging-admission verify <fixture.jsonl> <manifest.json>");
        process::exit(2);
    }

    match verify_files(&args[2], &args[3]) {
        Ok(report) => match serde_json::to_string_pretty(&report) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("failed to serialize verification report: {error}");
                process::exit(2);
            }
        },
        Err(error) => {
            eprintln!("admission fixture rejected: {error}");
            process::exit(2);
        }
    }
}
