// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Local AGS4 structural preflight command.
//!
//! This reports the local file envelope and basic BGS required-group presence.
//! It never uploads source data, calls the external BGS validator, or qualifies
//! geotechnical/physical measurements.

use serde::Serialize;
use sol_atlas_core::ags_preflight::{
    AGS4_PREFLIGHT_MAX_BYTES, AgsDocument, AgsPreflightError, AgsVersionError, ags_dictionary_id,
    parse_ags4_preflight,
};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const DEFAULT_FIXTURE: &str = "research/earth-observability/fixtures/ags4-synthetic-v0.1.ags";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AgsGroupSummary {
    name: String,
    heading_count: usize,
    row_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AgsPreflightReport {
    file_name: String,
    source_file_bytes: usize,
    source_sha256: String,
    structural_preflight: String,
    declared_ags_version: Option<String>,
    dictionary_id: Option<String>,
    version_status: String,
    groups: Vec<AgsGroupSummary>,
    bgs_required_group_issues: Vec<String>,
    official_ags_validation: String,
    bgs_submission_validation: String,
    rights_authorization: String,
    physical_qualification: String,
    claim_boundary: String,
}

fn build_report(file_name: &str, bytes: &[u8]) -> Result<AgsPreflightReport, AgsPreflightError> {
    let document: AgsDocument = parse_ags4_preflight(bytes)?;
    let declared_version = document.declared_ags_version().map(str::to_string);
    let (dictionary_id, version_status) = match declared_version.as_deref() {
        Some(version) => match ags_dictionary_id(version) {
            Ok(dictionary) => (Some(dictionary.to_string()), "SUPPORTED_EXPLICIT_VERSION"),
            Err(AgsVersionError::Missing) => (None, "VERSION_MISSING"),
            Err(AgsVersionError::Unsupported(_)) => (None, "VERSION_UNSUPPORTED"),
        },
        None => (None, "VERSION_MISSING_OR_AMBIGUOUS"),
    };

    let group_summaries = document
        .groups
        .iter()
        .map(|group| AgsGroupSummary {
            name: group.name.clone(),
            heading_count: group.headings.len(),
            row_count: group.rows.len(),
        })
        .collect();

    let bgs_issues = document
        .bgs_required_group_issues()
        .iter()
        .map(|issue| format!("{issue:?}"))
        .collect::<Vec<_>>();

    let ready = dictionary_id.is_some() && bgs_issues.is_empty();
    Ok(AgsPreflightReport {
        file_name: file_name.to_string(),
        source_file_bytes: bytes.len(),
        source_sha256: "NOT_COMPUTED_BY_THIS_COMMAND".to_string(),
        structural_preflight: if ready {
            "PASS_LOCAL_PREFLIGHT_EXTERNAL_VALIDATION_PENDING".to_string()
        } else {
            "PASS_STRUCTURE_BUT_ADMISSION_BLOCKED".to_string()
        },
        declared_ags_version: declared_version,
        dictionary_id,
        version_status: version_status.to_string(),
        groups: group_summaries,
        bgs_required_group_issues: bgs_issues,
        official_ags_validation: "NOT_RUN".to_string(),
        bgs_submission_validation: "NOT_RUN".to_string(),
        rights_authorization: "NOT_RUN".to_string(),
        physical_qualification: "NOT_RUN".to_string(),
        claim_boundary: "Local structural preflight only. Not full AGS conformance, BGS submission acceptance, geotechnical data validation, measurement accuracy, or engineering approval.".to_string(),
    })
}

fn read_bounded_file(path: &str) -> Result<Vec<u8>, String> {
    let file =
        File::open(Path::new(path)).map_err(|error| format!("cannot open {path}: {error}"))?;
    let maximum_read = AGS4_PREFLIGHT_MAX_BYTES as u64 + 1;
    let mut bytes = Vec::new();
    file.take(maximum_read)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {path}: {error}"))?;
    if bytes.len() > AGS4_PREFLIGHT_MAX_BYTES {
        return Err(format!(
            "{path} exceeds the {} byte AGS4 preflight limit",
            AGS4_PREFLIGHT_MAX_BYTES
        ));
    }
    Ok(bytes)
}

fn usage(program: &str) {
    eprintln!("Usage: {program} [FILE.ags]");
    eprintln!("Default synthetic fixture: {DEFAULT_FIXTURE}");
    eprintln!("Maximum input size: {AGS4_PREFLIGHT_MAX_BYTES} bytes");
}

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "ags4-preflight".to_string());
    let first = args.next();

    let path = match first.as_deref() {
        None => DEFAULT_FIXTURE.to_string(),
        Some("--help" | "-h") => {
            usage(&program);
            return;
        }
        Some(value) => value.to_string(),
    };

    if args.next().is_some() {
        usage(&program);
        std::process::exit(64);
    }

    let result = read_bounded_file(&path).and_then(|bytes| {
        build_report(&path, &bytes).map_err(|error| format!("AGS4 preflight failed: {error:?}"))
    });

    match result {
        Ok(report) => {
            let ready =
                report.structural_preflight == "PASS_LOCAL_PREFLIGHT_EXTERNAL_VALIDATION_PENDING";
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("preflight report must serialize")
            );
            if !ready {
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../research/earth-observability/fixtures/ags4-synthetic-v0.1.ags");

    #[test]
    fn synthetic_fixture_has_explicit_dictionary_and_required_groups() {
        let report = build_report(DEFAULT_FIXTURE, FIXTURE.as_bytes())
            .expect("synthetic AGS fixture passes structural preflight");
        assert_eq!(report.declared_ags_version.as_deref(), Some("4.1.1"));
        assert_eq!(report.dictionary_id.as_deref(), Some("v4_1_1"));
        assert!(report.bgs_required_group_issues.is_empty());
        assert_eq!(
            report.structural_preflight,
            "PASS_LOCAL_PREFLIGHT_EXTERNAL_VALIDATION_PENDING"
        );
    }

    #[test]
    fn report_never_claims_external_or_physical_validation() {
        let report = build_report(DEFAULT_FIXTURE, FIXTURE.as_bytes()).unwrap();
        assert_eq!(report.official_ags_validation, "NOT_RUN");
        assert_eq!(report.bgs_submission_validation, "NOT_RUN");
        assert_eq!(report.rights_authorization, "NOT_RUN");
        assert_eq!(report.physical_qualification, "NOT_RUN");
        assert_eq!(report.source_sha256, "NOT_COMPUTED_BY_THIS_COMMAND");
    }

    #[test]
    fn unsupported_version_blocks_admission_without_defaulting() {
        let unsupported = FIXTURE.replace("\"4.1.1\"", "\"4\"");
        let report = build_report(DEFAULT_FIXTURE, unsupported.as_bytes()).unwrap();
        assert_eq!(report.version_status, "VERSION_UNSUPPORTED");
        assert_eq!(report.dictionary_id, None);
        assert_eq!(
            report.structural_preflight,
            "PASS_STRUCTURE_BUT_ADMISSION_BLOCKED"
        );
    }

    #[test]
    fn fixture_group_counts_are_reported() {
        let report = build_report(DEFAULT_FIXTURE, FIXTURE.as_bytes()).unwrap();
        assert_eq!(report.groups.len(), 7);
        assert!(
            report
                .groups
                .iter()
                .any(|group| group.name == "GEOL" && group.row_count == 1)
        );
    }
}
