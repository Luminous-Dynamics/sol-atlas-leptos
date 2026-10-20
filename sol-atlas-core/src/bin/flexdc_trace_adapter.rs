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
const PINNED_POWER_CAP_W: f64 = 24_216.0;
const PINNED_REGULATION_RESERVE_W: f64 = 9_626.0;
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
    synthetic: bool,
    source_revision: String,
    simulator_command: String,
    random_seed: u64,
    power_cap_w: f64,
    regulation_reserve_w: f64,
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
    if !manifest.power_cap_w.is_finite()
        || manifest.power_cap_w != PINNED_POWER_CAP_W
    {
        return Err(format!(
            "manifest.power_cap_w must match the pinned AQA policy value {PINNED_POWER_CAP_W}"
        ));
    }
    if !manifest.regulation_reserve_w.is_finite()
        || manifest.regulation_reserve_w != PINNED_REGULATION_RESERVE_W
    {
        return Err(format!(
            "manifest.regulation_reserve_w must match the pinned AQA policy value {PINNED_REGULATION_RESERVE_W}"
        ));
    }
    if manifest.simulation_duration_seconds == 0 {
        return Err("manifest.simulation_duration_seconds must be positive".to_owned());
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
            return Err(format!(
                "duplicate config artifact path {:?}",
                artifact.path
            ));
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
        .any(|role| !seen_roles.contains(*role))
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

fn parse_nonnegative_power_w(raw: &str, row_number: usize, field: &str) -> Result<f64, String> {
    if raw.trim().is_empty() {
        return Err(format!("power trace row {row_number}: {field} is missing"));
    }
    let watts = raw.parse::<f64>().map_err(|_| {
        format!("power trace row {row_number}: {field} must be numeric watts")
    })?;
    if !watts.is_finite() || watts < 0.0 {
        return Err(format!(
            "power trace row {row_number}: {field} must be finite non-negative watts"
        ));
    }
    Ok(watts)
}

fn parse_power_trace(
    csv_text: &str,
    manifest: &RunManifest,
) -> Result<(Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
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
    let find_indices = |field: &str| {
        header
            .iter()
            .enumerate()
            .filter_map(|(i, name)| (*name == field).then_some(i))
            .collect::<Vec<_>>()
    };
    let time_indices = find_indices("time");
    let target_indices = find_indices("target");
    let real_sum_indices = find_indices("realSum");
    let error_indices = find_indices("trackingError");
    if time_indices.len() != 1
        || target_indices.len() != 1
        || real_sum_indices.len() != 1
        || error_indices.len() != 1
    {
        return Err(
            "power trace must contain exactly one each of time, target, realSum, trackingError"
                .to_owned(),
        );
    }
    let time_idx = time_indices[0];
    let target_idx = target_indices[0];
    let real_sum_idx = real_sum_indices[0];
    let error_idx = error_indices[0];
    let expected_rows_u64 =
        manifest.simulation_duration_seconds / manifest.simulation_step_seconds + 1;
    let expected_rows = usize::try_from(expected_rows_u64)
        .map_err(|_| "declared simulation row count exceeds platform limits".to_owned())?;

    let mut interval_ids = Vec::with_capacity(expected_rows);
    let mut target_power_kw = Vec::with_capacity(expected_rows);
    let mut measured_power_kw = Vec::with_capacity(expected_rows);
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

        let target_w = parse_nonnegative_power_w(fields[target_idx], row_number, "target")?;
        let real_sum_w = parse_nonnegative_power_w(fields[real_sum_idx], row_number, "realSum")?;
        let target_kw = target_w / 1000.0;
        let real_sum_kw = real_sum_w / 1000.0;
        if !target_kw.is_finite() || !real_sum_kw.is_finite() {
            return Err(format!(
                "power trace row {row_number}: converted power must remain finite in kW"
            ));
        }

        let raw_error = fields[error_idx];
        if raw_error.trim().is_empty() {
            return Err(format!(
                "power trace row {row_number}: trackingError is missing"
            ));
        }
        let error = raw_error
            .parse::<f64>()
            .map_err(|_| format!("power trace row {row_number}: trackingError is not numeric"))?;
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
        target_power_kw.push(target_kw);
        measured_power_kw.push(real_sum_kw);
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
    Ok((
        interval_ids,
        target_power_kw,
        measured_power_kw,
        tracking_errors,
    ))
}

fn build_trace_input(power_csv: &[u8], manifest_bytes: &[u8]) -> Result<Value, String> {
    let manifest: RunManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("could not parse strict run manifest JSON: {error}"))?;
    validate_manifest(&manifest)?;

    let csv_text = std::str::from_utf8(power_csv)
        .map_err(|error| format!("power trace must be UTF-8 CSV: {error}"))?;
    let (interval_ids, target_power_kw, measured_power_kw, signed_errors) =
        parse_power_trace(csv_text, &manifest)?;
    let artifact_sha256 = sha256_hex(power_csv);
    let manifest_sha256 = sha256_hex(manifest_bytes);
    let reserve_capacity_kw = manifest.regulation_reserve_w / 1000.0;
    let normalization_note = format!(
        "FlexDC-Sim source commit {}; run {}; command {}; random seed {}; configured power cap P={} W and reserve R={} W. The exported trackingError is the source-native signed normalized error from prior-cycle real_power and previous signal cap, divided by R + 0.0001. Same-row target/realSum values are separately converted from W to kW and paired with exact configured R for the rederived RSR view. Startup rows before signal step {} seconds are zero. Time is preserved in seconds as interval IDs with no resampling. power_trace.csv SHA-256={artifact_sha256}; exact run-manifest JSON SHA-256={manifest_sha256}. {} This is simulator output, not wall-metered facility power.",
        manifest.source_revision,
        manifest.run_id,
        manifest.simulator_command,
        manifest.random_seed,
        manifest.power_cap_w,
        manifest.regulation_reserve_w,
        manifest.signal_step_seconds,
        manifest.notes
    );

    Ok(json!({
        "schema_version": "1.0",
        "fixture_id": format!("flexdc-sim-{}", manifest.run_id),
        "synthetic": manifest.synthetic,
        "claim_ceiling": "metric_accounting_only",
        "rsr": {
            "interval_ids": interval_ids.clone(),
            "target_power_kw": target_power_kw,
            "measured_power_kw": measured_power_kw,
            "reserve_capacity_kw": reserve_capacity_kw
        },
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
    use super::{RunManifest, build_trace_input, parse_power_trace, sha256_hex, validate_manifest};

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
            output["rsr"]["interval_ids"].as_array().unwrap().len(),
            10
        );
        assert_eq!(
            output["rsr_reference"]["interval_ids"].as_array().unwrap().len(),
            10
        );
        assert_eq!(output["rsr"]["interval_ids"][0], "0");
        assert_eq!(output["rsr"]["interval_ids"][9], "9");
        assert_eq!(output["rsr_reference"]["interval_ids"][0], "0");
        assert_eq!(output["rsr_reference"]["interval_ids"][9], "9");
        assert_eq!(output["rsr"]["target_power_kw"][8], 10.0);
        assert_eq!(output["rsr"]["measured_power_kw"][8], 13.8504);
        assert_eq!(output["rsr"]["reserve_capacity_kw"], 9.626);
        assert_eq!(
            output["rsr_reference"]["signed_normalized_tracking_error"][9],
            0.8
        );
        assert_eq!(
            output["rsr"]["interval_ids"],
            output["rsr_reference"]["interval_ids"]
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
    fn adapter_preserves_manifest_synthetic_flag() {
        let output = build_trace_input(TRACE.as_bytes(), MANIFEST.as_bytes())
            .expect("synthetic fixture should adapt");
        assert_eq!(output["synthetic"], true);

        let real_run_manifest = MANIFEST.replace(
            "\"synthetic\": true",
            "\"synthetic\": false",
        );
        let output = build_trace_input(TRACE.as_bytes(), real_run_manifest.as_bytes())
            .expect("manifest with a non-synthetic run flag should adapt");
        assert_eq!(output["synthetic"], false);
    }

    #[test]
    fn source_native_and_same_row_metrics_are_deliberately_distinct() {
        let output = build_trace_input(TRACE.as_bytes(), MANIFEST.as_bytes())
            .expect("paired known-answer trace should adapt");
        let aligned = output["rsr"]["target_power_kw"]
            .as_array()
            .expect("row-aligned target series");
        assert_eq!(aligned[8], 10.0);
        let measured = output["rsr"]["measured_power_kw"]
            .as_array()
            .expect("row-aligned measured series");
        assert_eq!(measured[8], 13.8504);
        let source_errors = output["rsr_reference"]["signed_normalized_tracking_error"]
            .as_array()
            .expect("source-native tracking error series");
        assert_eq!(source_errors[8], 0.0);
        assert_eq!(source_errors[9], 0.8);
    }

    #[test]
    fn malformed_source_metadata_and_csv_rows_fail_closed() {
        let manifest = parsed_manifest();
        let bad_header = "time,realSum\n0,0\n";
        assert!(
            parse_power_trace(bad_header, &manifest)
                .expect_err("missing trackingError column must reject")
                .contains("exactly one")
        );
        let bad_time = TRACE.replace("5,10000.0,10000.0,0.0", "6,10000.0,10000.0,0.0");
        assert!(
            parse_power_trace(&bad_time, &manifest)
                .expect_err("gap in timestamps must reject")
                .contains("expected time 5, got 6")
        );
        let missing_error = TRACE.replace("9,10000.0,13850.4,0.8", "9,10000.0,13850.4,");
        assert!(
            parse_power_trace(&missing_error, &manifest)
                .expect_err("missing trackingError must reject")
                .contains("trackingError is missing")
        );
    }

    #[test]
    fn manifest_must_pin_all_configuration_roles() {
        let mut manifest = parsed_manifest();
        manifest.config_artifacts.pop();
        assert!(
            validate_manifest(&manifest)
                .expect_err("incomplete config list must reject")
                .contains("exactly 6 required roles")
        );
    }

    #[test]
    fn source_revision_must_be_canonical_lowercase_commit_sha() {
        let mut manifest = parsed_manifest();
        manifest.source_revision = "A".repeat(40);
        assert!(
            validate_manifest(&manifest)
                .expect_err("uppercase revision must reject")
                .contains("lowercase Git commit SHA")
        );
    }

    #[test]
    fn source_revision_must_match_the_pinned_reference_not_just_sha_shape() {
        let mut manifest = parsed_manifest();
        manifest.source_revision = "0000000000000000000000000000000000000000".to_owned();
        assert!(
            validate_manifest(&manifest)
                .expect_err("different source revision must be rejected")
                .contains("supports only pinned FlexDC-Sim revision")
        );
    }

    #[test]
    fn manifest_rejects_unpinned_power_cap_or_reserve() {
        let mut manifest = parsed_manifest();
        manifest.power_cap_w = 24_217.0;
        assert!(
            validate_manifest(&manifest)
                .expect_err("substituted power cap must reject")
                .contains("power_cap_w must match")
        );

        let mut manifest = parsed_manifest();
        manifest.regulation_reserve_w = 9_627.0;
        assert!(
            validate_manifest(&manifest)
                .expect_err("substituted reserve must reject")
                .contains("regulation_reserve_w must match")
        );
    }

    #[test]
    fn same_row_power_values_reject_missing_negative_and_non_finite_values() {
        let manifest = parsed_manifest();

        let missing_target = TRACE.replace(
            "8,10000.0,13850.4,0.0",
            "8,,13850.4,0.0",
        );
        assert!(
            parse_power_trace(&missing_target, &manifest)
                .expect_err("missing target watts must reject")
                .contains("target is missing")
        );

        let negative_power = TRACE.replace(
            "8,10000.0,13850.4,0.0",
            "8,-1.0,13850.4,0.0",
        );
        assert!(
            parse_power_trace(&negative_power, &manifest)
                .expect_err("negative target watts must reject")
                .contains("finite non-negative watts")
        );

        let non_finite_power = TRACE.replace(
            "8,10000.0,13850.4,0.0",
            "8,NaN,13850.4,0.0",
        );
        assert!(
            parse_power_trace(&non_finite_power, &manifest)
                .expect_err("non-finite target watts must reject")
                .contains("finite non-negative watts")
        );
    }

    #[test]
    fn manifest_rejects_zero_duration() {
        let mut manifest = parsed_manifest();
        manifest.simulation_duration_seconds = 0;
        assert!(
            validate_manifest(&manifest)
                .expect_err("zero-duration run must reject")
                .contains("simulation_duration_seconds must be positive")
        );
    }

    #[test]
    fn manifest_rejects_incompatible_simulation_and_signal_steps() {
        let mut manifest = parsed_manifest();
        manifest.simulation_duration_seconds = 10;
        manifest.simulation_step_seconds = 2;
        manifest.signal_step_seconds = 3;
        assert!(
            validate_manifest(&manifest)
                .expect_err("signal step not divisible by simulation step should reject")
                .contains("positive multiple")
        );
        let mut manifest = parsed_manifest();
        manifest.simulation_duration_seconds = 10;
        manifest.simulation_step_seconds = 4;
        assert!(
            validate_manifest(&manifest)
                .expect_err("non-divisible duration should reject")
                .contains("must be divisible")
        );
    }

    #[test]
    fn pinned_source_rejects_a_substituted_config_blob() {
        let mut manifest = parsed_manifest();
        manifest.config_artifacts[0].git_blob_sha1 =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned();
        assert!(
            validate_manifest(&manifest)
                .expect_err("config identity substitution must be rejected")
                .contains("pinned path/blob identity")
        );
    }

    #[test]
    fn quoted_rows_and_non_finite_errors_fail_closed() {
        let manifest = parsed_manifest();
        let quoted = TRACE.replace("time,target", "time,\"target\"");
        assert!(
            parse_power_trace(&quoted, &manifest)
                .expect_err("quoted CSV rows are unsupported by this narrow adapter")
                .contains("quoted fields")
        );

        let non_finite = TRACE.replace("9,10000.0,13850.4,0.8", "9,10000.0,13850.4,NaN");
        assert!(
            parse_power_trace(&non_finite, &manifest)
                .expect_err("non-finite source error must reject")
                .contains("must be finite")
        );
    }

    #[test]
    fn startup_errors_must_match_the_pinned_source_branch() {
        let manifest = parsed_manifest();
        let invalid_startup = TRACE.replace("0,10000.0,10000.0,0.0", "0,10000.0,10000.0,0.1");
        assert!(
            parse_power_trace(&invalid_startup, &manifest)
                .expect_err("nonzero source error before signal step must reject")
                .contains("startup trackingError")
        );
    }

    #[test]
    fn sha256_known_answer_is_correct() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
