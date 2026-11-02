// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Source-bound profiling entrypoint for Sol Atlas's bounded exact dispatcher.
//!
//! The binary emits the generic energy-solver-compute-trace-v0.2 CSV schema.
//! Exact-search counters are recorded separately from MILP variable counts;
//! no synthetic scalar objective or unobserved first-feasible timestamp is
//! fabricated for the lexicographic Pareto enumerator.

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::energy_dispatch::{
    EnergyDispatchAssessment, EnergyDispatchError, EnergyDispatchRequest, MAX_CANDIDATE_EXPANSIONS,
    MAX_PARETO_FRONTIER, assess_energy_dispatch,
};
use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const REPORT_SCHEMA: &str = "sol-atlas-profiled-dispatch-run-v0.1";
const TRACE_SCHEMA: &str = "energy-solver-compute-trace-v0.2";
const TRACE_HEADER: &str = "schema_version,measurement_class,cohort_id,sample_id,scenario_id,scenario_panel_sha256,request_sha256,model_sha256,mesh_policy_sha256,mesh_instance_sha256,solver_name,solver_version,solver_binary_sha256,solver_config_sha256,hardware_profile_sha256,horizon_minutes,interval_count,continuous_variables,integer_variables,binary_variables,constraints,random_seed,status,elapsed_ns,time_to_first_feasible_ns,time_to_optimal_ns,objective_value_units,best_bound_units,optimality_gap_ppm,peak_memory_bytes,timeout_limit_ns,search_action_expansions,feasible_complete_schedules,pareto_frontier_count";
const MAX_REQUEST_BYTES: usize = 1_048_576;

// These bytes are compiled into the runner and included in its model fingerprint.
const DISPATCH_SOURCE: &[u8] = include_bytes!("../../../sol-atlas-core/src/energy_dispatch.rs");
const PATHWAY_SOURCE: &[u8] = include_bytes!("../../../sol-atlas-core/src/energy_pathway.rs");
const CORE_LIB_SOURCE: &[u8] = include_bytes!("../../../sol-atlas-core/src/lib.rs");
const CORE_MANIFEST: &[u8] = include_bytes!("../../../sol-atlas-core/Cargo.toml");
const RUNNER_MANIFEST: &[u8] = include_bytes!("../Cargo.toml");
const ROOT_MANIFEST: &[u8] = include_bytes!("../../../Cargo.toml");
const ROOT_LOCKFILE: &[u8] = include_bytes!("../../../Cargo.lock");

#[derive(Clone, Debug, Serialize)]
struct HostProfile {
    operating_system: &'static str,
    architecture: &'static str,
    cpu_model: String,
    logical_cpu_count: u32,
    physical_memory_bytes: u64,
    profile_sha256: String,
}

#[derive(Debug, Serialize)]
struct ProfiledRun {
    report_schema: &'static str,
    trace_schema: &'static str,
    status: &'static str,
    request_sha256: String,
    scenario_panel_sha256: String,
    model_sha256: String,
    mesh_policy_sha256: String,
    mesh_instance_sha256: String,
    solver_binary_sha256: String,
    solver_config_sha256: String,
    hardware_profile: HostProfile,
    elapsed_ns: u64,
    peak_memory_bytes: Option<u64>,
    trace_csv_header: &'static str,
    trace_csv_row: String,
    failure: Option<String>,
    assessment: Option<EnergyDispatchAssessment>,
    claim_ceiling: &'static str,
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn sha256_many(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    let digest = hasher.finalize();
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| format!("open request file: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("inspect opened request file: {error}"))?;
    if !metadata.is_file() {
        return Err("request path must identify a regular file".to_owned());
    }
    if metadata.len() > limit as u64 {
        return Err(format!("request exceeded {limit} byte limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read request file: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("request exceeded {limit} byte limit while reading"));
    }
    Ok(bytes)
}

fn host_profile() -> Result<HostProfile, String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")
        .map_err(|error| format!("read /proc/cpuinfo for hardware profile: {error}"))?;
    let meminfo = fs::read_to_string("/proc/meminfo")
        .map_err(|error| format!("read /proc/meminfo for hardware profile: {error}"))?;

    let cpu_model = cpuinfo
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            matches!(key.trim(), "model name" | "Hardware" | "Processor")
                .then(|| value.trim().to_owned())
        })
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "CPU model identifier is unavailable in /proc/cpuinfo".to_owned())?;

    let logical_cpu_count = u32::try_from(
        cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count(),
    )
    .map_err(|_| "logical CPU count does not fit u32".to_owned())?;
    if logical_cpu_count == 0 {
        return Err("logical CPU count is zero in /proc/cpuinfo".to_owned());
    }

    let memory_kib = meminfo
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.trim() != "MemTotal" {
                return None;
            }
            value.split_whitespace().next()?.parse::<u64>().ok()
        })
        .ok_or_else(|| "MemTotal is unavailable in /proc/meminfo".to_owned())?;
    let physical_memory_bytes = memory_kib
        .checked_mul(1024)
        .ok_or_else(|| "physical memory byte count overflowed".to_owned())?;

    let profile_material = format!(
        "os={}\narch={}\ncpu_model={}\nlogical_cpu_count={}\nphysical_memory_bytes={}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        cpu_model,
        logical_cpu_count,
        physical_memory_bytes
    );
    Ok(HostProfile {
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        cpu_model,
        logical_cpu_count,
        physical_memory_bytes,
        profile_sha256: sha256(profile_material.as_bytes()),
    })
}

fn peak_memory_bytes() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let memory_kib = status.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim() != "VmHWM" {
            return None;
        }
        value.split_whitespace().next()?.parse::<u64>().ok()
    })?;
    memory_kib.checked_mul(1024)
}

fn mesh_fingerprints(request: &EnergyDispatchRequest) -> Result<(String, String), String> {
    let cadence_seconds: Vec<u64> = request
        .intervals
        .iter()
        .map(|interval| interval.duration_seconds)
        .collect();
    let mesh_instance: Vec<(&str, i64, u64)> = request
        .intervals
        .iter()
        .map(|interval| {
            (
                interval.interval_id.as_str(),
                interval.start_unix_seconds,
                interval.duration_seconds,
            )
        })
        .collect();
    let policy_bytes = serde_json::to_vec(&cadence_seconds)
        .map_err(|error| format!("serialize temporal mesh policy: {error}"))?;
    let instance_bytes = serde_json::to_vec(&mesh_instance)
        .map_err(|error| format!("serialize exact temporal mesh instance: {error}"))?;
    Ok((sha256(&policy_bytes), sha256(&instance_bytes)))
}

fn model_fingerprint() -> String {
    sha256_many(&[
        b"sol-atlas-exact-dispatch-model-v0.1",
        DISPATCH_SOURCE,
        PATHWAY_SOURCE,
        CORE_LIB_SOURCE,
        CORE_MANIFEST,
        RUNNER_MANIFEST,
        ROOT_MANIFEST,
        ROOT_LOCKFILE,
    ])
}

fn solver_configuration_fingerprint(request: &EnergyDispatchRequest) -> String {
    let material = format!(
        "algorithm=bounded-exhaustive-pareto-enumerator-v0.1\nmax_candidate_expansions={}\nhard_candidate_expansions={}\nmax_pareto_frontier={}\nenumeration_order=ascending-integer-actions\n",
        request.max_candidate_expansions, MAX_CANDIDATE_EXPANSIONS, MAX_PARETO_FRONTIER
    );
    sha256(material.as_bytes())
}

fn horizon_minutes(request: &EnergyDispatchRequest) -> Result<u16, String> {
    let seconds = request.intervals.iter().try_fold(0u64, |total, interval| {
        total
            .checked_add(interval.duration_seconds)
            .ok_or_else(|| "dispatch horizon duration overflowed".to_owned())
    })?;
    let minutes = seconds.div_ceil(60);
    let minutes =
        u16::try_from(minutes).map_err(|_| "dispatch horizon does not fit u16".to_owned())?;
    if minutes == 0 || minutes > 1_440 {
        return Err("dispatch horizon must be within 1..=1440 minutes".to_owned());
    }
    Ok(minutes)
}

fn status_for_error(error: &EnergyDispatchError) -> &'static str {
    match error {
        EnergyDispatchError::NoFeasibleSchedules => "infeasible",
        EnergyDispatchError::CandidateExpansionCapExceeded { .. }
        | EnergyDispatchError::ParetoFrontierCapExceeded { .. } => "limit_reached",
        _ => "error",
    }
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "NA".to_owned())
}

fn run_profile(request_bytes: &[u8], scenario_panel_sha256: &str) -> Result<ProfiledRun, String> {
    if !valid_sha256(scenario_panel_sha256) {
        return Err("scenario panel digest must be 64 lowercase hexadecimal characters".to_owned());
    }
    let parse_started = Instant::now();
    let request: EnergyDispatchRequest = serde_json::from_slice(request_bytes)
        .map_err(|error| format!("parse EnergyDispatchRequest JSON: {error}"))?;
    let _request_parse_ns = u64::try_from(parse_started.elapsed().as_nanos())
        .map_err(|_| "request parse duration does not fit u64".to_owned())?;

    let request_sha256 = sha256(request_bytes);
    let model_sha256 = model_fingerprint();
    let (mesh_policy_sha256, mesh_instance_sha256) = mesh_fingerprints(&request)?;
    let solver_config_sha256 = solver_configuration_fingerprint(&request);
    let executable = std::env::current_exe()
        .map_err(|error| format!("resolve solver runner binary: {error}"))?;
    let solver_binary_sha256 = sha256(
        &fs::read(&executable)
            .map_err(|error| format!("read exact solver runner binary: {error}"))?,
    );
    let hardware = host_profile()?;
    let horizon_minutes = horizon_minutes(&request)?;
    let interval_count = u32::try_from(request.intervals.len())
        .map_err(|_| "interval count does not fit u32".to_owned())?;

    let cohort_material = format!(
        "{}:{}:{}:{}:{}:{}:{}",
        scenario_panel_sha256,
        model_sha256,
        mesh_policy_sha256,
        horizon_minutes,
        solver_binary_sha256,
        solver_config_sha256,
        hardware.profile_sha256
    );
    let cohort_digest = sha256(cohort_material.as_bytes());
    let cohort_id = format!("cohort-{}", &cohort_digest[..24]);
    let invocation_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock predates UNIX epoch: {error}"))?
        .as_nanos();
    let sample_id = format!(
        "sample-{}-{}-{invocation_ns}",
        &request_sha256[..16],
        std::process::id()
    );
    let scenario_digest = sha256(request.case_id.as_bytes());
    let scenario_id = format!("scenario-{}", &scenario_digest[..24]);

    let solve_started = Instant::now();
    let assessed = assess_energy_dispatch(&request);
    let elapsed_ns = u64::try_from(solve_started.elapsed().as_nanos())
        .map_err(|_| "solver elapsed time does not fit u64 nanoseconds".to_owned())?;
    if elapsed_ns == 0 {
        return Err("solver elapsed time rounded to zero nanoseconds".to_owned());
    }
    let peak_memory_bytes = peak_memory_bytes();

    let (
        status,
        time_to_optimal_ns,
        search_action_expansions,
        feasible_complete_schedules,
        pareto_frontier_count,
        failure,
        assessment,
    ) = match assessed {
        Ok(assessment) => (
            "optimal",
            Some(elapsed_ns),
            Some(assessment.evaluated_action_expansions),
            Some(assessment.feasible_complete_schedules),
            Some(
                u32::try_from(assessment.pareto_frontier_count)
                    .map_err(|_| "Pareto frontier count does not fit u32".to_owned())?,
            ),
            None,
            Some(assessment),
        ),
        Err(error) => (
            status_for_error(&error),
            None,
            None,
            None,
            None,
            Some(format!("{error:?}")),
            None,
        ),
    };

    let measurement_class = if request.synthetic {
        "synthetic"
    } else {
        "measured"
    };
    let trace_csv_row = format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        TRACE_SCHEMA,
        measurement_class,
        cohort_id,
        sample_id,
        scenario_id,
        scenario_panel_sha256,
        request_sha256,
        model_sha256,
        mesh_policy_sha256,
        mesh_instance_sha256,
        "sol-atlas-exact-enumerator",
        "0.1.0",
        solver_binary_sha256,
        solver_config_sha256,
        hardware.profile_sha256,
        horizon_minutes,
        interval_count,
        "NA", // continuous variables: no explicit mathematical-program formulation
        "NA", // integer variables: no explicit mathematical-program formulation
        "NA", // binary variables: no explicit mathematical-program formulation
        "NA", // constraints: enforced by the enumerator's transition rules
        0,    // the deterministic enumerator consumes no random seed
        status,
        elapsed_ns,
        "NA", // first-feasible wall time is not currently instrumented by the kernel
        optional_u64(time_to_optimal_ns),
        "NA", // exact objective is a lexicographic Pareto vector, not a scalar
        "NA", // exhaustive search does not expose a scalar dual bound
        "NA", // no scalar optimality gap is defined for the Pareto objective
        optional_u64(peak_memory_bytes),
        "NA", // this runner records expansion limits, not a wall-clock timeout
        optional_u64(search_action_expansions),
        optional_u64(feasible_complete_schedules),
        pareto_frontier_count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NA".to_owned()),
    );

    Ok(ProfiledRun {
        report_schema: REPORT_SCHEMA,
        trace_schema: TRACE_SCHEMA,
        status,
        request_sha256,
        scenario_panel_sha256: scenario_panel_sha256.to_owned(),
        model_sha256,
        mesh_policy_sha256,
        mesh_instance_sha256,
        solver_binary_sha256,
        solver_config_sha256,
        hardware_profile: hardware,
        elapsed_ns,
        peak_memory_bytes,
        trace_csv_header: TRACE_HEADER,
        trace_csv_row,
        failure,
        assessment,
        claim_ceiling: concat!(
            "measured execution of the Sol Atlas bounded exhaustive enumerator for the exact ",
            "request and compiled source fingerprint; optimality applies only to the declared ",
            "finite quantized model; hardware fingerprint is descriptive, not remotely attested; ",
            "no real-world safety or continuous-physics claim is made"
        ),
    })
}

fn append_trace(path: &Path, profile: &ProfiledRun) -> Result<(), String> {
    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("read existing trace CSV: {error}")),
    };
    let mut output = String::new();
    if existing.is_empty() {
        output.push_str(TRACE_HEADER);
        output.push('\n');
    } else {
        if existing.lines().next() != Some(TRACE_HEADER) {
            return Err("existing trace CSV does not have the exact v0.2 header".to_owned());
        }
        if !existing.ends_with('\n') {
            return Err("existing trace CSV must end with a newline before append".to_owned());
        }
    }
    output.push_str(&profile.trace_csv_row);
    output.push('\n');
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("open trace CSV for append: {error}"))?;
    file.write_all(output.as_bytes())
        .map_err(|error| format!("append trace CSV row: {error}"))
}

fn write_report(path: &Path, profile: &ProfiledRun) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(profile)
        .map_err(|error| format!("serialize profiled run report: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("write temporary profiled report: {error}"))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("atomically publish profiled report: {error}"))
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err(format!(
            "usage: {} <request.json> <scenario-panel-sha256> <trace.csv> <report.json>",
            args.first()
                .map(String::as_str)
                .unwrap_or("energy-dispatch-profile")
        ));
    }
    let request_bytes = read_bounded(Path::new(&args[1]), MAX_REQUEST_BYTES)?;
    let profile = run_profile(&request_bytes, &args[2])?;
    append_trace(Path::new(&args[3]), &profile)?;
    write_report(Path::new(&args[4]), &profile)?;
    println!("status={}", profile.status);
    println!("request_sha256={}", profile.request_sha256);
    println!("solver_binary_sha256={}", profile.solver_binary_sha256);
    println!("model_sha256={}", profile.model_sha256);
    println!("elapsed_ns={}", profile.elapsed_ns);
    println!("trace_csv={}", args[3]);
    println!("report_json={}", args[4]);
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("energy dispatch profiling failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] =
        include_bytes!("../../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");
    const PANEL_SHA256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn frozen_synthetic_dispatch_runs_through_the_real_exact_engine_and_emits_34_column_trace() {
        let profile = run_profile(FIXTURE, PANEL_SHA256).expect("profile frozen request");
        assert_eq!(profile.status, "optimal");
        assert!(profile.assessment.as_ref().unwrap().search_is_exact);
        assert!(
            profile
                .assessment
                .as_ref()
                .unwrap()
                .pareto_frontier_complete
        );

        let columns: Vec<&str> = profile.trace_csv_row.split(',').collect();
        assert_eq!(columns.len(), 34);
        assert_eq!(columns[0], TRACE_SCHEMA);
        assert_eq!(columns[1], "synthetic");
        assert_eq!(columns[22], "optimal");
        assert_eq!(columns[24], "NA");
        assert_ne!(columns[25], "NA");
        assert_eq!(&columns[17..21], &["NA", "NA", "NA", "NA"]);
        assert_eq!(&columns[26..29], &["NA", "NA", "NA"]);
        assert!(columns[31].parse::<u64>().unwrap() > 0);
        assert!(columns[32].parse::<u64>().unwrap() > 0);
        assert!(columns[33].parse::<u32>().unwrap() > 0);
    }

    #[test]
    fn expansion_cap_exhaustion_is_recorded_as_limit_reached_not_optimal() {
        let mut request: EnergyDispatchRequest =
            serde_json::from_slice(FIXTURE).expect("valid frozen request");
        request.max_candidate_expansions = 1;
        let bytes = serde_json::to_vec(&request).unwrap();
        let profile = run_profile(&bytes, PANEL_SHA256).expect("profile cap-limited request");
        assert_eq!(profile.status, "limit_reached");
        assert!(profile.assessment.is_none());
        assert!(
            profile
                .failure
                .as_deref()
                .unwrap()
                .contains("CandidateExpansionCapExceeded")
        );

        let columns: Vec<&str> = profile.trace_csv_row.split(',').collect();
        assert_eq!(columns[22], "limit_reached");
        assert_eq!(&columns[25..29], &["NA", "NA", "NA", "NA"]);
        assert_eq!(&columns[31..34], &["NA", "NA", "NA"]);
    }

    #[test]
    fn unsupported_request_schema_is_preserved_as_an_error_trace_not_a_fake_infeasibility() {
        let mut request: EnergyDispatchRequest =
            serde_json::from_slice(FIXTURE).expect("valid frozen request");
        request.schema_version = "energy-dispatch-unknown".to_owned();
        let bytes = serde_json::to_vec(&request).unwrap();
        let profile = run_profile(&bytes, PANEL_SHA256).expect("profile rejected model request");
        assert_eq!(profile.status, "error");
        assert!(profile.assessment.is_none());
        assert!(
            profile
                .failure
                .as_deref()
                .unwrap()
                .contains("UnsupportedSchemaVersion")
        );
        let columns: Vec<&str> = profile.trace_csv_row.split(',').collect();
        assert_eq!(columns[22], "error");
    }
}
