use shared_amenity_manifest_audit::audit_manifest_bytes;
use std::{env, fs, process};

const DEFAULT_MANIFEST: &str =
    "docs/research/highveld-neighborhood-commons/SHARED_AMENITY_COMMONS_MANIFEST_V0.1.json";

fn main() {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        None => DEFAULT_MANIFEST.to_owned(),
        Some(arg) if arg == "--help" || arg == "-h" => {
            println!("Usage: shared-amenity-manifest-audit [MANIFEST.json]");
            println!("Defaults to {DEFAULT_MANIFEST}");
            return;
        }
        Some(arg) => arg,
    };

    if args.next().is_some() {
        eprintln!("expected at most one manifest path");
        process::exit(2);
    }

    let bytes = fs::read(&path).unwrap_or_else(|error| {
        eprintln!("cannot read {path}: {error}");
        process::exit(2);
    });

    match audit_manifest_bytes(&bytes) {
        Ok(summary) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).expect("summary serialization")
            );
        }
        Err(error) => {
            eprintln!("audit blocked at {}: {}", error.path, error.message);
            process::exit(1);
        }
    }
}
