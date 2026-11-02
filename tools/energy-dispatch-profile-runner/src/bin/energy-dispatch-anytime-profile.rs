#![forbid(unsafe_code)]
//! Source-bound v0.3 anytime profile runner.
//!
//! This binary executes the shared exact/anytime traversal once, then invokes
//! the schedule auditor and service-floor auditor as separate processes.
//! It does not authorize actuator operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sol_atlas_core::energy_dispatch::{
    assess_energy_dispatch_execution, EnergyDispatchAnytimeAssessment, EnergyDispatchRequest,
    EnergyDispatchAnytimeDisposition,
};
use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod trace_support {
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub enum MeasurementClass {
        Synthetic,
        Measured,
    }

    impl MeasurementClass {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Synthetic => "synthetic",
                Self::Measured => "measured",
            }
        }
    }

    pub const MAX_TRACE_CSV_BYTES: usize = 64 * 1024 * 1024;
    pub const P50_MIN_SAMPLES: usize = 5;
    pub const P95_MIN_SAMPLES: usize = 20;

    pub fn safe_identifier(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 128
            && value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
    }

    pub fn is_sha256_hex(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    pub mod trace_v0_3 {
        include!("../../../energy-solver-compute-calibration/src/trace_v0_3.rs");
    }
}

use trace_support::trace_v0_3::{
    AuditDispositionV03, SearchDispositionV03, ServiceFloorDispositionV03, SolverTraceV03,
    TRACE_V03_HEADER, TRACE_V03_SCHEMA,
};
use trace_support::MeasurementClass;

const REPORT_SCHEMA: &str = "sol-atlas-anytime-profile-run-v0.1";
const MAX_REQUEST_BYTES: usize = 1_048_576;

const DISPATCH_SOURCE: &[u8] = include_bytes!("../../../../sol-atlas-core/src/energy_dispatch.rs");
const PATHWAY_SOURCE: &[u8] = include_bytes!("../../../../sol-atlas-core/src/energy_pathway.rs");
const CORE_LIB_SOURCE: &[u8] = include_bytes!("../../../../sol-atlas-core/src/lib.rs");
const CORE_MANIFEST: &[u8] = include_bytes!("../../../../sol-atlas-core/Cargo.toml");
const RUNNER_MANIFEST: &[u8] = include_bytes!("../../Cargo.toml");
const ROOT_MANIFEST: &[u8] = include_bytes!("../../../../Cargo.toml");
const ROOT_LOCKFILE: &[u8] = include_bytes!("../../../../Cargo.lock");

#[derive(Debug, Clone, Serialize)]
struct AuditIntervalDispatch {
    interval_id: String,
    grid_import_quanta: u16,
    charge_input_quanta: u16,
    discharge_withdrawal_quanta: u16,
    soc_after_quanta: u16,
    essential_served_quanta: u16,
    essential_unserved_quanta: u16,
    flexible_served_quanta: u16,
    flexible_unserved_quanta: u16,
    pv_used_quanta: u16,
    pv_curtailed_quanta: u16,
}

#[derive(Debug, Clone, Serialize)]
struct DispatchScheduleForAudit {
    schema_version: &'static str,
    case_id: String,
    request_sha256: String,
    interval_dispatches: Vec<AuditIntervalDispatch>,
    terminal_soc_quanta: u16,
}

#[derive(Debug, Deserialize)]
struct ScheduleAuditReceiptLite {
    disposition: String,
    request_sha256: String,
    schedule_sha256: String,
    auditor_binary_sha256: String,
    receipt_sha256: String,
}

#[derive(Debug, Deserialize)]
struct ServiceFloorReceiptLite {
    disposition: String,
    request_sha256: String,
    schedule_sha256: String,
    policy_sha256: String,
    schedule_audit_receipt_sha256: String,
    schedule_auditor_binary_sha256: String,
    service_floor_auditor_binary_sha256: String,
    receipt_sha256: String,
}

#[derive(Debug, Serialize)]
struct AnytimeProfileReport {
    report_schema: &'static str,
    trace_schema: &'static str,
    status: &'static str,
    failure: Option<String>,
    request_sha256: String,
    scenario_panel_sha256: String,
    model_sha256: String,
    solver_binary_sha256: String,
    solver_config_sha256: String,
    hardware_profile_sha256: String,
    elapsed_ns: u64,
    trace_csv_header: &'static str,
    trace_csv_row: String,
    anytime_assessment: EnergyDispatchAnytimeAssessment,
    selected_schedule: Option<DispatchScheduleForAudit>,
    schedule_audit_receipt: Option<Value>,
    service_floor_receipt: Option<Value>,
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

fn read_bounded(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| format!("open {label}: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("inspect opened {label}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{label} path must identify a regular file"));
    }
    if metadata.len() > limit as u64 {
        return Err(format!("{label} exceeds {limit} byte limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded {limit} byte limit while reading"));
    }
    Ok(bytes)
}

fn valid_sha256(value: &str) -> bool {
    trace_support::is_sha256_hex(value)
}

fn current_binary_sha256() -> Result<String, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("resolve this runner binary: {error}"))?;
    let bytes = fs::read(executable)
        .map_err(|error| format!("read this runner binary: {error}"))?;
    Ok(sha256(&bytes))
}

fn file_sha256(path: &Path, label: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {label} for digest: {error}"))?;
    Ok(sha256(&bytes))
}

fn host_profile_sha256() -> Result<String, String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")
        .map_err(|error| format!("read CPU profile: {error}"))?;
    let meminfo = fs::read_to_string("/proc/meminfo")
        .map_err(|error| format!("read memory profile: {error}"))?;
    let cpu = cpuinfo
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            matches!(key.trim(), "model name" | "Hardware" | "Processor")
                .then(|| value.trim().to_owned())
        })
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "CPU model is unavailable".to_owned())?;
    let logical = cpuinfo.lines().filter(|line| line.starts_with("processor")).count();
    if logical == 0 {
        return Err("logical CPU count is zero".to_owned());
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
        .ok_or_else(|| "MemTotal unavailable".to_owned())?;
    let material = format!(
        "os={}\narch={}\ncpu={}\nlogical={}\nphysical_memory_bytes={}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        cpu,
        logical,
        memory_kib.checked_mul(1024).ok_or("memory overflow")?
    );
    Ok(sha256(material.as_bytes()))
}

fn horizon_minutes(request: &EnergyDispatchRequest) -> Result<u16, String> {
    let seconds = request.intervals.iter().try_fold(0u64, |total, interval| {
        total
            .checked_add(interval.duration_seconds)
            .ok_or_else(|| "dispatch horizon overflow".to_owned())
    })?;
    let minutes = u16::try_from(seconds.div_ceil(60))
        .map_err(|_| "dispatch horizon does not fit u16".to_owned())?;
    if minutes == 0 || minutes > 1_440 {
        return Err("dispatch horizon must be within 1..=1440 minutes".to_owned());
    }
    Ok(minutes)
}

fn schedule_from_candidate(
    request: &EnergyDispatchRequest,
    request_sha256: &str,
    candidate: &sol_atlas_core::energy_dispatch::DispatchParetoAlternative,
) -> DispatchScheduleForAudit {
    DispatchScheduleForAudit {
        schema_version: "energy-dispatch-schedule-v0.1",
        case_id: request.case_id.clone(),
        request_sha256: request_sha256.to_owned(),
        interval_dispatches: candidate
            .schedule
            .iter()
            .map(|step| AuditIntervalDispatch {
                interval_id: step.interval_id.clone(),
                grid_import_quanta: step.grid_import_quanta,
                charge_input_quanta: step.battery_charge_input_quanta,
                discharge_withdrawal_quanta: step.battery_soc_withdrawal_quanta,
                soc_after_quanta: step.battery_soc_after_quanta,
                essential_served_quanta: step.essential_served_quanta,
                essential_unserved_quanta: step.essential_unserved_quanta,
                flexible_served_quanta: step.flexible_served_quanta,
                flexible_unserved_quanta: step.flexible_unserved_quanta,
                pv_used_quanta: step.pv_used_quanta,
                pv_curtailed_quanta: step.pv_curtailed_quanta,
            })
            .collect(),
        terminal_soc_quanta: candidate.objective.terminal_soc_quanta,
    }
}

fn atomic_write(path: &Path, bytes: &[u8], label: &str) -> Result<(), String> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|error| format!("write temporary {label}: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("publish {label}: {error}"))
}

fn write_json(path: &Path, value: &impl Serialize, label: &str) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize {label}: {error}"))?;
    atomic_write(path, &bytes, label)
}

fn append_trace(path: &Path, trace: &SolverTraceV03) -> Result<String, String> {
    let row = trace
        .render_csv_row()
        .map_err(|error| format!("v0.3 trace validation/render: {error}"))?;
    let existing = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("read existing v0.3 trace: {error}")),
    };
    if existing.len() > trace_support::MAX_TRACE_CSV_BYTES {
        return Err("existing v0.3 trace exceeds declared byte limit".to_owned());
    }
    let mut output = String::new();
    if existing.is_empty() {
        output.push_str(TRACE_V03_HEADER);
        output.push('\n');
    } else {
        if existing.lines().next() != Some(TRACE_V03_HEADER) {
            return Err("existing trace has wrong v0.3 header".to_owned());
        }
        if !existing.ends_with('\n') {
            return Err("existing trace must end with newline before append".to_owned());
        }
    }
    output.push_str(&row);
    output.push('\n');
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("open v0.3 trace for append: {error}"))?;
    file.write_all(output.as_bytes())
        .map_err(|error| format!("append v0.3 trace row: {error}"))?;
    Ok(row)
}

fn run_profile(
    request_bytes: &[u8],
    scenario_panel_sha256: &str,
    trace_path: &Path,
    report_path: &Path,
    schedule_path: &Path,
    audit_receipt_path: &Path,
    policy_path: &Path,
    service_floor_receipt_path: &Path,
    schedule_auditor_path: &Path,
    service_floor_auditor_path: &Path,
) -> Result<AnytimeProfileReport, String> {
    if !valid_sha256(scenario_panel_sha256) {
        return Err("scenario panel SHA-256 must be 64 lowercase hex characters".to_owned());
    }
    let request: EnergyDispatchRequest = serde_json::from_slice(request_bytes)
        .map_err(|error| format!("parse EnergyDispatchRequest: {error}"))?;
    let request_sha256 = sha256(request_bytes);
    let panel_hash = scenario_panel_sha256.to_owned();
    let host_hash = host_profile_sha256()?;
    let runner_binary_hash = current_binary_sha256()?;
    let model_hash = sha256_many(&[
        b"sol-atlas-anytime-dispatch-model-v0.1",
        DISPATCH_SOURCE,
        PATHWAY_SOURCE,
        CORE_LIB_SOURCE,
        CORE_MANIFEST,
        RUNNER_MANIFEST,
        ROOT_MANIFEST,
        ROOT_LOCKFILE,
    ]);
    let solver_config_hash = sha256(
        format!(
            "algorithm=bounded-exhaustive-anytime-v0.1\ncandidate_policy=lexicographic-dispatch-objective-v1\nmax_candidate_expansions={}\nmax_frontier={}\n",
            request.max_candidate_expansions,
            sol_atlas_core::energy_dispatch::MAX_PARETO_FRONTIER
        )
        .as_bytes(),
    );
    let cadence: Vec<u64> = request.intervals.iter().map(|x| x.duration_seconds).collect();
    let mesh_policy_hash = sha256(
        &serde_json::to_vec(&cadence).map_err(|e| format!("serialize mesh policy: {e}"))?,
    );
    let mesh_instance: Vec<(&str, i64, u64)> = request
        .intervals
        .iter()
        .map(|x| (x.interval_id.as_str(), x.start_unix_seconds, x.duration_seconds))
        .collect();
    let mesh_instance_hash = sha256(
        &serde_json::to_vec(&mesh_instance).map_err(|e| format!("serialize mesh instance: {e}"))?,
    );
    let horizon = horizon_minutes(&request)?;
    let interval_count = u32::try_from(request.intervals.len())
        .map_err(|_| "interval count does not fit u32".to_owned())?;
    let auditor_binary_hash = file_sha256(schedule_auditor_path, "schedule auditor executable").ok();
    let service_floor_binary_hash =
        file_sha256(service_floor_auditor_path, "service-floor auditor executable").ok();
    let policy_bytes = read_bounded(policy_path, MAX_REQUEST_BYTES, "service-floor policy")?;
    let policy_hash = sha256(&policy_bytes);
    let cohort_material = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        panel_hash,
        model_hash,
        mesh_policy_hash,
        horizon,
        "sol-atlas-anytime-enumerator",
        "0.1.0",
        runner_binary_hash,
        solver_config_hash,
        host_hash,
        request.schema_version
    );
    let cohort_id = format!("cohort-{}", &sha256(cohort_material.as_bytes())[..24]);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock before epoch: {error}"))?
        .as_nanos();
    let sample_id = format!(
        "sample-{}-{}-{timestamp}",
        &request_sha256[..16],
        std::process::id()
    );
    let scenario_id = format!("scenario-{}", &sha256(request.case_id.as_bytes())[..24]);

    let invocation_started = Instant::now();
    let execution = assess_energy_dispatch_execution(&request)
        .map_err(|error| format!("anytime dispatch execution error: {error:?}"))?;
    let anytime = execution.anytime;

    let candidate = anytime.candidate.as_ref();
    let schedule = candidate.map(|value| schedule_from_candidate(&request, &request_sha256, value));
    let schedule_bytes = schedule
        .as_ref()
        .map(|value| serde_json::to_vec_pretty(value))
        .transpose()
        .map_err(|error| format!("serialize candidate schedule: {error}"))?;
    let candidate_schedule_hash = schedule_bytes.as_ref().map(|bytes| sha256(bytes));
    if let Some(bytes) = schedule_bytes.as_ref() {
        atomic_write(schedule_path, bytes, "candidate schedule")?;
    } else {
        let _ = fs::remove_file(schedule_path);
        let _ = fs::remove_file(audit_receipt_path);
        let _ = fs::remove_file(service_floor_receipt_path);
    }

    let mut audit_disposition = AuditDispositionV03::NotRun;
    let mut audit_receipt_sha = None;
    let mut audit_request_sha = None;
    let mut audit_schedule_sha = None;
    let mut audit_binary_sha = None;
    let mut audit_elapsed_ns = None;
    let mut time_to_first_audit_accept_ns = None;
    let mut audit_receipt_value = None;
    let mut service_floor_disposition = ServiceFloorDispositionV03::NotEvaluated;
    let mut service_floor_audit_receipt_sha = None;
    let mut service_floor_receipt_sha = None;
    let mut service_floor_binary_sha = None;
    let mut service_floor_receipt_value = None;
    let mut service_floor_elapsed_ns = None;
    let mut failure = None;

    // The exact source bytes are supplied through an immutable file path. Write
    // them beside the report so both independent processes consume those bytes.
    let request_path = report_path.with_extension("request.json");
    atomic_write(&request_path, request_bytes, "exact request snapshot")?;

    if schedule_bytes.is_some() {
        let _ = fs::remove_file(audit_receipt_path);
        let audit_started = Instant::now();
        let audit_output = Command::new(schedule_auditor_path)
            .arg(&request_path)
            .arg(schedule_path)
            .arg(audit_receipt_path)
            .output();
        audit_elapsed_ns = Some(
            u64::try_from(audit_started.elapsed().as_nanos())
                .map_err(|_| "audit elapsed nanoseconds overflow".to_owned())?,
        );
        match audit_output {
            Ok(output) => {
                let receipt_bytes = fs::read(audit_receipt_path).ok();
                if let Some(receipt_bytes) = receipt_bytes {
                    let decoded = (|| -> Result<(Value, ScheduleAuditReceiptLite), String> {
                        let value: Value = serde_json::from_slice(&receipt_bytes)
                            .map_err(|error| format!("parse schedule audit receipt: {error}"))?;
                        let lite: ScheduleAuditReceiptLite =
                            serde_json::from_value(value.clone())
                                .map_err(|error| format!("decode schedule audit receipt: {error}"))?;
                        Ok((value, lite))
                    })();
                    match decoded {
                        Ok((receipt_value, lite)) => {
                            if lite.request_sha256 != request_sha256
                                || lite.schedule_sha256
                                    != candidate_schedule_hash.as_deref().unwrap_or("")
                                || auditor_binary_hash.as_deref()
                                    != Some(lite.auditor_binary_sha256.as_str())
                                || !valid_sha256(&lite.receipt_sha256)
                            {
                                audit_disposition = AuditDispositionV03::Unavailable;
                                failure = Some(
                                    "schedule audit receipt bindings did not match exact input bytes or executable"
                                        .to_owned(),
                                );
                            } else {
                                audit_disposition = match lite.disposition.as_str() {
                                    "accepted" if output.status.success() => {
                                        AuditDispositionV03::Accepted
                                    }
                                    "rejected" if output.status.code() == Some(1) => {
                                        AuditDispositionV03::Rejected
                                    }
                                    _ => AuditDispositionV03::Unavailable,
                                };
                                if audit_disposition != AuditDispositionV03::Unavailable {
                                    audit_receipt_sha = Some(lite.receipt_sha256.clone());
                                    audit_request_sha = Some(lite.request_sha256.clone());
                                    audit_schedule_sha = Some(lite.schedule_sha256.clone());
                                    audit_binary_sha = Some(lite.auditor_binary_sha256.clone());
                                    audit_receipt_value = Some(receipt_value);
                                }
                                if audit_disposition == AuditDispositionV03::Accepted {
                                    time_to_first_audit_accept_ns = Some(
                                        u64::try_from(invocation_started.elapsed().as_nanos())
                                            .map_err(|_| "audit acceptance elapsed overflow".to_owned())?,
                                    );
                                }
                            }
                        }
                        Err(error) => {
                            audit_disposition = AuditDispositionV03::Unavailable;
                            failure = Some(error);
                        }
                    }
                } else {
                    audit_disposition = AuditDispositionV03::Unavailable;
                    failure = Some(format!("schedule auditor produced no receipt; exit={:?}", output.status.code()));
                }
            }
            Err(error) => {
                audit_disposition = AuditDispositionV03::Unavailable;
                failure = Some(format!("could not execute schedule auditor: {error}"));
            }
        }

        if audit_disposition == AuditDispositionV03::Unavailable {
            if audit_receipt_sha.is_some() {
                audit_receipt_sha = None;
                audit_request_sha = None;
                audit_schedule_sha = None;
                audit_binary_sha = None;
                audit_receipt_value = None;
            }
        }
    }

    if audit_disposition == AuditDispositionV03::Accepted {
        let _ = fs::remove_file(service_floor_receipt_path);
        let floor_started = Instant::now();
        let floor_output = Command::new(service_floor_auditor_path)
            .arg(&request_path)
            .arg(schedule_path)
            .arg(policy_path)
            .arg(audit_receipt_path)
            .arg(service_floor_receipt_path)
            .output();
        let floor_elapsed = u64::try_from(floor_started.elapsed().as_nanos())
            .map_err(|_| "service-floor elapsed nanoseconds overflow".to_owned())?;
        service_floor_elapsed_ns = Some(floor_elapsed);
        match floor_output {
            Ok(output) => {
                if let Ok(receipt_bytes) = fs::read(service_floor_receipt_path) {
                    let decoded = (|| -> Result<(Value, ServiceFloorReceiptLite), String> {
                        let value: Value = serde_json::from_slice(&receipt_bytes)
                            .map_err(|error| format!("parse service-floor receipt: {error}"))?;
                        let lite: ServiceFloorReceiptLite =
                            serde_json::from_value(value.clone())
                                .map_err(|error| format!("decode service-floor receipt: {error}"))?;
                        Ok((value, lite))
                    })();
                    match decoded {
                        Ok((receipt_value, lite)) => {
                            if lite.request_sha256 != request_sha256
                                || lite.schedule_sha256
                                    != candidate_schedule_hash.as_deref().unwrap_or("")
                                || lite.schedule_audit_receipt_sha256
                                    != audit_receipt_sha.as_deref().unwrap_or("")
                                || lite.schedule_auditor_binary_sha256
                                    != audit_binary_sha.as_deref().unwrap_or("")
                                || Some(lite.service_floor_auditor_binary_sha256.as_str())
                                    != service_floor_binary_hash.as_deref()
                                || lite.policy_sha256 != policy_hash
                                || !valid_sha256(&lite.receipt_sha256)
                            {
                                service_floor_disposition =
                                    ServiceFloorDispositionV03::Unavailable;
                                failure = Some(
                                    "service-floor receipt bindings did not match exact policy, request, schedule, audit receipt or evaluator executable".to_owned(),
                                );
                            } else {
                                service_floor_disposition = match lite.disposition.as_str() {
                                    "passed" if output.status.success() => {
                                        ServiceFloorDispositionV03::Passed
                                    }
                                    "failed" if output.status.code() == Some(1) => {
                                        ServiceFloorDispositionV03::Failed
                                    }
                                    _ => ServiceFloorDispositionV03::Unavailable,
                                };
                                if matches!(
                                    service_floor_disposition,
                                    ServiceFloorDispositionV03::Passed
                                        | ServiceFloorDispositionV03::Failed
                                ) {
                                    service_floor_audit_receipt_sha =
                                        Some(lite.schedule_audit_receipt_sha256.clone());
                                    service_floor_receipt_sha =
                                        Some(lite.receipt_sha256.clone());
                                    service_floor_binary_sha =
                                        Some(lite.service_floor_auditor_binary_sha256.clone());
                                    service_floor_receipt_value = Some(receipt_value);
                                }
                            }
                        }
                        Err(error) => {
                            service_floor_disposition = ServiceFloorDispositionV03::Unavailable;
                            failure = Some(error);
                        }
                    }
                } else {
                    service_floor_disposition = ServiceFloorDispositionV03::Unavailable;
                    failure = Some(format!("service-floor auditor produced no receipt; exit={:?}", output.status.code()));
                }
            }
            Err(error) => {
                service_floor_disposition = ServiceFloorDispositionV03::Unavailable;
                failure = Some(format!("could not execute service-floor auditor: {error}"));
            }
        }
    }

    let elapsed_ns = u64::try_from(invocation_started.elapsed().as_nanos())
        .map_err(|_| "profile elapsed nanoseconds overflow".to_owned())?;
    let stop_reason_and_status = match anytime.disposition {
        EnergyDispatchAnytimeDisposition::ExactComplete => (
            SearchDispositionV03::ExactComplete,
            true,
            execution.exact.as_ref().map(|report| report.pareto_frontier_count as u32),
        ),
        EnergyDispatchAnytimeDisposition::FeasibleUnproven => {
            (SearchDispositionV03::FeasibleUnproven, false, None)
        }
        EnergyDispatchAnytimeDisposition::ProvenInfeasible => {
            (SearchDispositionV03::ProvenInfeasible, true, Some(0))
        }
        EnergyDispatchAnytimeDisposition::LimitWithoutCandidate => {
            (SearchDispositionV03::LimitWithoutCandidate, false, None)
        }
    };
    let search_disposition = stop_reason_and_status.0;
    let pareto_frontier_complete = stop_reason_and_status.1;
    let pareto_frontier_count = stop_reason_and_status.2;
    let policy_hash_for_trace = match service_floor_disposition {
        ServiceFloorDispositionV03::NotEvaluated => None,
        ServiceFloorDispositionV03::Passed
        | ServiceFloorDispositionV03::Failed
        | ServiceFloorDispositionV03::Unavailable => Some(policy_hash.clone()),
    };
    let measurement_class = if request.synthetic {
        MeasurementClass::Synthetic
    } else {
        MeasurementClass::Measured
    };
    let trace = SolverTraceV03 {
        schema_version: TRACE_V03_SCHEMA.to_owned(),
        measurement_class,
        cohort_id,
        sample_id,
        scenario_id,
        scenario_panel_sha256: panel_hash,
        request_sha256: request_sha256.clone(),
        model_sha256: model_hash.clone(),
        mesh_policy_sha256: mesh_policy_hash,
        mesh_instance_sha256: mesh_instance_hash,
        solver_name: "sol-atlas-anytime-enumerator".to_owned(),
        solver_version: "0.1.0".to_owned(),
        solver_binary_sha256: runner_binary_hash.clone(),
        solver_config_sha256: solver_config_hash.clone(),
        hardware_profile_sha256: host_hash.clone(),
        horizon_minutes: horizon,
        interval_count,
        search_disposition,
        elapsed_ns,
        time_to_first_candidate_ns: anytime.time_to_first_complete_candidate_ns,
        time_to_first_audit_accept_ns,
        peak_memory_bytes: None,
        timeout_limit_ns: None,
        enumeration_prefix_visits: Some(anytime.enumeration_prefix_visits),
        feasible_complete_schedules: Some(anytime.feasible_complete_schedules),
        pareto_frontier_complete,
        pareto_frontier_count,
        candidate_complete_horizon: schedule.is_some(),
        candidate_schedule_sha256: candidate_schedule_hash,
        audit_disposition,
        audit_receipt_sha256: audit_receipt_sha,
        audit_request_sha256: audit_request_sha,
        audit_schedule_sha256: audit_schedule_sha,
        auditor_binary_sha256: audit_binary_sha,
        audit_elapsed_ns,
        service_floor_disposition,
        service_floor_policy_sha256: policy_hash_for_trace,
        service_floor_schedule_audit_receipt_sha256: service_floor_audit_receipt_sha,
        service_floor_receipt_sha256: service_floor_receipt_sha,
        service_floor_auditor_binary_sha256: service_floor_binary_sha,
        service_floor_elapsed_ns,
    };
    let trace_row = append_trace(trace_path, &trace)?;
    let status = match trace.search_disposition {
        SearchDispositionV03::ExactComplete => "exact_complete",
        SearchDispositionV03::FeasibleUnproven => "feasible_unproven",
        SearchDispositionV03::ProvenInfeasible => "proven_infeasible",
        SearchDispositionV03::LimitWithoutCandidate => "limit_without_candidate",
        SearchDispositionV03::TimedOut => "timed_out",
        SearchDispositionV03::Cancelled => "cancelled",
        SearchDispositionV03::Error => "error",
    };
    let report = AnytimeProfileReport {
        report_schema: REPORT_SCHEMA,
        trace_schema: TRACE_V03_SCHEMA,
        status,
        failure,
        request_sha256,
        scenario_panel_sha256: scenario_panel_sha256.to_owned(),
        model_sha256: model_hash,
        solver_binary_sha256: runner_binary_hash,
        solver_config_sha256,
        hardware_profile_sha256: host_hash,
        elapsed_ns,
        trace_csv_header: TRACE_V03_HEADER,
        trace_csv_row: trace_row,
        anytime_assessment: anytime,
        selected_schedule: schedule,
        schedule_audit_receipt: audit_receipt_value,
        service_floor_receipt: service_floor_receipt_value,
        claim_ceiling: concat!(
            "search outcomes, candidate and external receipts refer only to exact declared bytes ",
            "and the bounded finite quantized model; no continuous-physics, operational safety, ",
            "source-truth, field reliability, grid-code or actuation-authority claim is made"
        ),
    };
    write_json(report_path, &report, "anytime run report")?;
    Ok(report)
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 11 {
        return Err(format!(
            "usage: {} <request.json> <scenario-panel-sha256> <trace-v0.3.csv> <report.json> <schedule.json> <schedule-audit-receipt.json> <service-floor-policy.json> <service-floor-receipt.json> <schedule-auditor-bin> <service-floor-auditor-bin>",
            args.first()
                .map(|value| value.to_string_lossy())
                .unwrap_or_else(|| "energy-dispatch-anytime-profile".into())
        ));
    }
    let request = read_bounded(Path::new(&args[1]), MAX_REQUEST_BYTES, "source request")?;
    let report = run_profile(
        &request,
        &args[2].to_string_lossy(),
        Path::new(&args[3]),
        Path::new(&args[4]),
        Path::new(&args[5]),
        Path::new(&args[6]),
        Path::new(&args[7]),
        Path::new(&args[8]),
        Path::new(&args[9]),
        Path::new(&args[10]),
    )?;
    println!("status={}", report.status);
    println!("request_sha256={}", report.request_sha256);
    println!("elapsed_ns={}", report.elapsed_ns);
    println!("trace_csv={}", args[3].to_string_lossy());
    println!("report_json={}", args[4].to_string_lossy());
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("anytime dispatch profile failed closed: {error}");
            ExitCode::from(2)
        }
    }
}
