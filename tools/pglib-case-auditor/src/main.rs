use std::env;
use std::fs;
use std::path::Path;

use pglib_case_auditor::{AuditSummary, audit_matpower_bytes};

const MAX_SOURCE_BYTES: u64 = 32 * 1024 * 1024;

fn usage() -> &'static str {
    "Usage: pglib-case-auditor <case.m> [--expect-git-blob-sha1 <40-hex-digits>]"
}

fn run() -> Result<AuditSummary, String> {
    let mut args = env::args().skip(1);
    let path = args.next().ok_or_else(|| usage().to_owned())?;
    let mut expected_sha = None;

    while let Some(argument) = args.next() {
        if argument == "--expect-git-blob-sha1" {
            if expected_sha.is_some() {
                return Err("duplicate --expect-git-blob-sha1 argument".to_owned());
            }
            expected_sha = Some(
                args.next()
                    .ok_or_else(|| "missing digest after --expect-git-blob-sha1".to_owned())?,
            );
        } else {
            return Err(format!("unknown argument '{argument}'\n{}", usage()));
        }
    }

    if let Some(expected) = &expected_sha
        && (expected.len() != 40 || !expected.chars().all(|ch| ch.is_ascii_hexdigit()))
    {
        return Err(
            "expected Git blob SHA-1 must contain exactly 40 hexadecimal digits".to_owned(),
        );
    }

    let path = Path::new(&path);
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect source path: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("source must be a regular, non-symlink file".to_owned());
    }
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(format!("source exceeds {} byte limit", MAX_SOURCE_BYTES));
    }

    let bytes = fs::read(path).map_err(|error| format!("cannot read source: {error}"))?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(format!("source exceeds {} byte limit", MAX_SOURCE_BYTES));
    }
    let summary = audit_matpower_bytes(&bytes)?;

    if let Some(expected) = expected_sha {
        if !summary.source_git_blob_sha1.eq_ignore_ascii_case(&expected) {
            return Err(format!(
                "source Git blob SHA-1 mismatch: expected {}, got {}",
                expected, summary.source_git_blob_sha1
            ));
        }
    }

    Ok(summary)
}

fn main() {
    match run() {
        Ok(summary) => println!("{}", summary.to_json()),
        Err(error) => {
            eprintln!("pglib-case-auditor: {error}");
            std::process::exit(2);
        }
    }
}
