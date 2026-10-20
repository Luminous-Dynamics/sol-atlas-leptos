//! Adapt a pinned FlexDC-Sim power_trace.csv into strict Sol-Atlas RSR input.
//!
//! Usage:
//!   cargo run -p sol-atlas-core --bin flexdc_trace_adapter -- power_trace.csv run-manifest.json
//!
//! The adapter supports the numeric CSV shape produced by FlexDC-Sim. It rejects
//! quoted fields, absent/non-finite trackingError values, missing rows and grid
//! misalignment rather than repairing the trace silently. The output has
//! synthetic=true and a metric-accounting-only claim ceiling: this is simulator
//! output, not metered facility power or causal grid-benefit evidence.

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

const MANIFEST_SCHEMA_VERSION: &str = "1.0";
const PINNED_FLEXDC_REVISION: &str = "ea50bd582b5d1b55fcb09ea46899fa9d325cce3b";
const PINNED_CONFIG_ARTIFACTS: [(&str, &str, &str); 6] = [
    (
        "experiment_config",
        "configs/experiment/exp_low_util.ini",
        "554bb80355e85fb0c3fbf73e4d8a4f1521fa2184",
    ),
    (
        "cluster_config",
        "configs/cluster/cluster.ini",
        "9acf360f05e88e82e812c8d412979a91dcbe3399",
    ),
    (
        "workload_config",
        "configs/workload/W2-short-qos3445.ini",
        "c7f5e7d9e91de0ef17a4995b4d07cd896dc436f1",
    ),
    (
        "runtime_policy_config",
        "configs/policy/example_flexDC_simulation.ini",
        "8a12f500ed8116b99e7e6b79010d5ab76be85051",
    ),
    (
        "power_signal",
        "data/ISO24h.csv",
        "6f7a8e4275d00ad43702f8a94d0a085b79f21f65",
    ),
    (
        "optimization_weights",
        "src/peacsim/output/optimization/W2-simulated_annealing_low_util_availability=False_20250308130817/opt_weights.csv",
        "f84164c63ba4215f92bb41c174955ec6e041db85",
    ),
];
const REQUIRED_CONFIG_ROLES: [&str; 6] = [
    "experiment_config",
    "cluster_config",
    "workload_config",
    "runtime_policy_config",
    "power_signal",
    "optimization_weights",
];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunManifest {
    schema_version: String,
    run_id: String,
    source_revision: String,
    simulator_command: String,
    random_seed: u64,
    simulation_duration_seconds: u64,
    simulation_step_seconds: u64,
    signal_step_seconds: u64,
    config_artifacts: Vec<ConfigArtifact>,
    notes: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigArtifact {
    role: String,
    path: String,
    git_blob_sha1: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_manifest(manifest: &RunManifest) -> Result<(), String> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(format!(
            "schema_version must be {MANIFEST_SCHEMA_VERSION}, got {:?}",
            manifest.schema_version
        ));
    }
    for (field, value) in [
        ("run_id", manifest.run_id.as_str()),
        ("simulator_command", manifest.simulator_command.as_str()),
        ("notes", manifest.notes.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("manifest.{field} must not be blank"));
        }
    }
    if !is_lower_hex(&manifest.source_revision, 40) {
        return Err(
            "manifest.source_revision must be a 40-character lowercase Git commit SHA".to_owned(),
        );
    }
    if manifest.source_revision != PINNED_FLEXDC_REVISION {
        return Err(format!(
            "this adapter supports only pinned FlexDC-Sim revision {PINNED_FLEXDC_REVISION}"
        ));
    }
    if manifest.simulation_step_seconds == 0 {
        return Err("manifest.simulation_step_seconds must be positive".to_owned());
    }
    if manifest.signal_step_seconds < manifest.simulation_step_seconds
        || manifest.signal_step_seconds % manifest.simulation_step_seconds != 0
    {
        return Err(
            "manifest.signal_step_seconds must be a positive multiple of simulation_step_seconds"
                .to_owned(),
        );
    }
    if manifest.simulation_duration_seconds % manifest.simulation_step_seconds != 0 {
        return Err(
            "manifest.simulation_duration_seconds must be divisible by simulation_step_seconds"
                .to_owned(),
        );
    }
    if manifest.config_artifacts.len() != REQUIRED_CONFIG_ROLES.len() {
        return Err(format!(
            "manifest.config_artifacts must contain exactly {} required roles",
            REQUIRED_CONFIG_ROLES.len()
        ));
    }

    let required: HashSet<&str> = REQUIRED_CONFIG_ROLES.into_iter().collect();
    let mut seen_roles = HashSet::with_capacity(manifest.config_artifacts.len());
    let mut seen_paths = HashSet::with_capacity(manifest.config_artifacts.len());
    for artifact in &manifest.config_artifacts {
        if !required.contains(artifact.role.as_str()) {
            return Err(format!(
                "unsupported config artifact role {:?}",
                artifact.role
            ));
        }
        if !seen_roles.insert(artifact.role.as_str()) {
            return Err(format!(
                "duplicate config artifact role {:?}",
                artifact.role
            ));
        }
        if artifact.path.trim().is_empty() || artifact.path.contains('\n') {
            return Err("config artifact paths must be nonblank single-line values".to_owned());
        }
        if !seen_paths.insert(artifact.path.as_str()) {
            return Err(format!("duplicate config artifact path {:?}", artifact.path));
        }
        if !is_lower_hex(&artifact.git_blob_sha1, 40) {
            return Err(format!(
                "config artifact {:?} must carry a 40-character lowercase Git blob SHA-1",
                artifact.role
            ));
        }
        let (expected_role, expected_path, expected_sha1) = PINNED_CONFIG_ARTIFACTS
            .iter()
            .find(|(role, _, _)| *role == artifact.role)
            .expect("required role and pinned artifact table must stay aligned");
        if artifact.path != *expected_path || artifact.git_blob_sha1 != *expected_sha1 {
            return Err(format!(
                "config artifact {:?} does not match the pinned path/blob identity",
                expected_role
            ));
        }
    }
    if REQUIRED_CONFIG_ROLES
        .iter()
        .any(|role| !seen_roles.contains(role))
    {
        return Err("manifest is missing one or more required config artifact roles".to_owned());
    }
    Ok(())
}

fn split_numeric_csv_line(line: &str, row_number: usize) -> Result<Vec<&str>, String> {
    if line.contains('"') {
        return Err(format!(
            "CSV row {row_number} contains quoted fields; only the numeric FlexDC-Sim export format is supported"
        ));
    }
    Ok(line.split(',').collect())
}

fn parse_power_trace(
    csv_text: &str,
    manifest: &RunManifest,
) -> Result<(Vec<String>, Vec<f64>), String> {
    let mut lines = csv_text.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| "power trace is empty".to_owned())?
        .trim_start_matches('\u{feff}')
        .trim_end_matches('\r');
    if header_line.trim().is_empty() {
        return Err("power trace header is blank".to_owned());
    }
    let header = split_numeric_csv_line(header_line, 1)?;
    let time_indices = header
        .iter()
        .enumerate()
        .filter_map(|(i, name)| (*name == "time").then_some(i))
        .collect::<Vec<_>>();
    let error_indices = header
        .iter()
        .enumerate()
        .filter_map(|(i, name)| (*name == "trackingError").then_some(i))
        .collect::<Vec<_>>();
    if time_indices.len() != 1 || error_indices.len() != 1 {
        return Err(
            "power trace must contain exactly one time and one trackingError column".to_owned(),
        );
    }
    let time_idx = time_indices[0];
    let error_idx = error_indices[0];
    let expected_rows_u64 =
        manifest.simulation_duration_seconds / manifest.simulation_step_seconds + 1;
    let expected_rows = usize::try_from(expected_rows_u64)
        .map_err(|_| "declared simulation row count exceeds platform limits".to_owned())?;

    let mut interval_ids = Vec::with_capacity(expected_rows);
    let mut tracking_errors = Vec::with_capacity(expected_rows);
    for (line_index, line) in lines.enumerate() {
        let row_number = line_index + 2;
        if line.trim().is_empty() {
            return Err(format!("power trace row {row_number} is blank"));
        }
        let fields = split_numeric_csv_line(line.trim_end_matches('\r'), row_number)?;
        if fields.len() != header.len() {
            return Err(format!(
                "power trace row {row_number} has {} fields; header has {}",
                fields.len(),
                header.len()
            ));
        }
        let time = fields[time_idx].parse::<u64>().map_err(|_| {
            format!("power trace row {row_number}: time must be an unsigned integer in seconds")
        })?;
        let expected_time = (line_index as u64)
            .checked_mul(manifest.simulation_step_seconds)
            .ok_or_else(|| "expected time grid overflows u64".to_owned())?;
        if time != expected_time {
            return Err(format!(
                "power trace row {row_number}: expected time {expected_time}, got {time}; missing, duplicate, or out-of-order rows are not silently repaired"
            ));
        }
        let raw_error = fields[error_idx];
        if raw_error.trim().is_empty() {
            return Err(format!(
                "power trace row {row_number}: trackingError is missing"
            ));
        }
        let error = raw_error.parse::<f64>().map_err(|_| {
            format!("power trace row {row_number}: trackingError is not numeric")
        })?;
        if !error.is_finite() {
            return Err(format!(
                "power trace row {row_number}: trackingError must be finite"
            ));
        }
        if time < manifest.signal_step_seconds && error != 0.0 {
            return Err(format!(
                "power trace row {row_number}: startup trackingError before the first signal interval must be zero for this source revision"
            ));
        }
        interval_ids.push(time.to_string());
        tracking_errors.push(error);
    }
    if tracking_errors.len() != expected_rows {
        return Err(format!(
            "power trace row count mismatch: manifest expects {expected_rows}, file contains {}",
            tracking_errors.len()
        ));
    }
    if tracking_errors.is_empty() {
        return Err("power trace contains no samples".to_owned());
    }
    Ok((interval_ids, tracking_errors))
}

fn build_trace_input(power_csv: &[u8], manifest_bytes: &[u8]) -> Result<Value, String> {
    let manifest: RunManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("could not parse strict run manifest JSON: {error}"))?;
    validate_manifest(&manifest)?;

    let csv_text = std::str::from_utf8(power_csv)
        .map_err(|error| format!("power trace must be UTF-8 CSV: {error}"))?;
    let (interval_ids, signed_errors) = parse_power_trace(csv_text, &manifest)?;
    let artifact_sha256 = sha256_hex(power_csv);
    let manifest_sha256 = sha256_hex(manifest_bytes);
    let normalization_note = format!(
        "FlexDC-Sim source commit {}; run {}; command {}; random seed {}; exported trackingError is the source-native signed normalized error from prior-cycle real_power and previous signal cap, divided by R + 0.0001. Startup rows before signal step {} seconds are zero. Time is preserved in seconds as interval IDs with no resampling. power_trace.csv SHA-256={artifact_sha256}; exact run-manifest JSON SHA-256={manifest_sha256}. {} This is simulator output, not wall-metered facility power.",
        manifest.source_revision,
        manifest.run_id,
        manifest.simulator_command,
        manifest.random_seed,
        manifest.signal_step_seconds,
        manifest.notes
    );

    Ok(json!({
        "schema_version": "1.0",
        "fixture_id": format!("flexdc-sim-{}", manifest.run_id),
        "synthetic": true,
        "claim_ceiling": "metric_accounting_only",
        "rsr": null,
        "rsr_reference": {
            "metadata": {
                "source_name": "FlexDC-Sim",
                "source_revision": manifest.source_revision,
                "source_field": "trackingError",
                "sample_time_semantics": "prior_cycle_state",
                "denominator_convention": "reserve_plus_epsilon",
                "source_artifact_sha256": artifact_sha256,
                "source_config_manifest_sha256": manifest_sha256,
                "normalization_note": normalization_note
            },
            "interval_ids": interval_ids,
            "signed_normalized_tracking_error": signed_errors
        },
        "edr": null
    }))
}

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(csv_path) = args.next() else {
        eprintln!("usage: flexdc_trace_adapter <power_trace.csv> <run-manifest.json>");
        process::exit(64);
    };
    let Some(manifest_path) = args.next() else {
        eprintln!("usage: flexdc_trace_adapter <power_trace.csv> <run-manifest.json>");
        process::exit(64);
    };
    if args.next().is_some() {
        eprintln!("usage: flexdc_trace_adapter <power_trace.csv> <run-manifest.json>");
        process::exit(64);
    }

    let power_csv = match fs::read(&csv_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read power trace: {error}");
            process::exit(66);
        }
    };
    let manifest_bytes = match fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read run manifest: {error}");
            process::exit(66);
        }
    };
    let input = match build_trace_input(&power_csv, &manifest_bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("could not adapt FlexDC-Sim trace: {error}");
            process::exit(65);
        }
    };
    match serde_json::to_string_pretty(&input) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("could not serialize adapted trace: {error}");
            process::exit(70);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_trace_input, parse_power_trace, sha256_hex, validate_manifest, RunManifest};

    const TRACE: &str =
        include_str!("../../../schemas/examples/flexdc-sim-power-trace.synthetic.csv");
    const MANIFEST: &str =
        include_str!("../../../schemas/examples/flexdc-sim-run-manifest-v1.synthetic.json");

    fn parsed_manifest() -> RunManifest {
        serde_json::from_str(MANIFEST).expect("synthetic manifest fixture should parse strictly")
    }

    #[test]
    fn known_answer_trace_preserves_source_errors_and_receipt_digests() {
        let output = build_trace_input(TRACE.as_bytes(), MANIFEST.as_bytes())
            .expect("known-answer FlexDC-Sim trace should adapt");
        assert_eq!(output["synthetic"], true);
        assert_eq!(output["claim_ceiling"], "metric_accounting_only");
        assert_eq!(
            output["rsr_reference"]["interval_ids"].as_array().unwrap().len(),
            10
        );
        assert_eq!(output["rsr_reference"]["interval_ids"][0], "0");
        assert_eq!(output["rsr_reference"]["interval_ids"][9], "9");
        assert_eq!(
            output["rsr_reference"]["signed_normalized_tracking_error"][9],
            0.8
        );
        assert_eq!(
            output["rsr_reference"]["metadata"]["sample_time_semantics"],
            "prior_cycle_state"
        );
        assert_eq!(
            output["rsr_reference"]["metadata"]["denominator_convention"],
            "reserve_plus_epsilon"
        );
        assert_eq!(
            output["rsr_reference"]["metadata"]["source_artifact_sha256"],
            sha256_hex(TRACE.as_bytes())
        );
        assert_eq!(
            output["rsr_reference"]["metadata"]["source_config_manifest_sha256"],
            sha256_hex(MANIFEST.as_bytes())
        );
    }

    #[test]
    fn malformed_source_metadata_and_csv_rows_fail_closed() {
        let manifest = parsed_manifest();
        let bad_header = "time,realSum\n0,0\n";
        assert!(parse_power_trace(bad_header, &manifest)
            .expect_err("missing trackingError column must reject")
            .contains("exactly one"));
        let bad_time = TRACE.replace("5,0.0", "6,0.0");
        assert!(parse_power_trace(&bad_time, &manifest)
            .expect_err("gap in timestamps must reject")
            .contains("expected time 5, got 6"));
        let missing_error = TRACE.replace("9,0.8", "9,");
        assert!(parse_power_trace(&missing_error, &manifest)
            .expect_err("missing trackingError must reject")
            .contains("trackingError is missing"));
    }

    #[test]
    fn manifest_must_pin_all_configuration_roles() {
        let mut manifest = parsed_manifest();
        manifest.config_artifacts.pop();
        assert!(validate_manifest(&manifest)
            .expect_err("incomplete config list must reject")
            .contains("exactly 6 required roles"));
    }

    #[test]
    fn source_revision_must_be_canonical_lowercase_commit_sha() {
        let mut manifest = parsed_manifest();
        manifest.source_revision = "A".repeat(40);
        assert!(validate_manifest(&manifest)
            .expect_err("uppercase revision must reject")
            .contains("lowercase Git commit SHA"));
    }

    #[test]
    fn sha256_known_answer_is_correct() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
