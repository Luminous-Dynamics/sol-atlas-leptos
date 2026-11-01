#![forbid(unsafe_code)]
//! Strict solver-trace validation and per-cohort latency/quality aggregation.
//!
//! This crate deliberately does not synthesize solver measurements. Its checked-in
//! CSV is a parser fixture. Real traces must identify the exact solver binary,
//! model, request, mesh, scenario panel and hardware profile before aggregation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const TRACE_SCHEMA: &str = "energy-solver-compute-trace-v0.1";
pub const SUMMARY_SCHEMA: &str = "energy-solver-compute-summary-v0.1";
pub const MAX_TRACE_CSV_BYTES: usize = 64 * 1024 * 1024;
pub const P50_MIN_SAMPLES: usize = 5;
pub const P95_MIN_SAMPLES: usize = 20;
pub const P99_MIN_SAMPLES: usize = 1_000;

pub const TRACE_HEADER: &str = "schema_version,measurement_class,cohort_id,sample_id,scenario_id,scenario_panel_sha256,request_sha256,model_sha256,mesh_policy_sha256,mesh_instance_sha256,solver_name,solver_version,solver_binary_sha256,solver_config_sha256,hardware_profile_sha256,horizon_minutes,interval_count,continuous_variables,integer_variables,binary_variables,constraints,random_seed,status,elapsed_ns,time_to_first_feasible_ns,time_to_optimal_ns,objective_value_units,best_bound_units,optimality_gap_ppm,peak_memory_bytes,timeout_limit_ns";

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

    fn parse(value: &str) -> Result<Self, TraceError> {
        match value {
            "synthetic" => Ok(Self::Synthetic),
            "measured" => Ok(Self::Measured),
            _ => Err(TraceError::InvalidMeasurementClass),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SolveStatus {
    Optimal,
    Feasible,
    Infeasible,
    TimedOut,
    Cancelled,
    Error,
}

impl SolveStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Optimal => "optimal",
            Self::Feasible => "feasible",
            Self::Infeasible => "infeasible",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::Error => "error",
        }
    }

    fn parse(value: &str) -> Result<Self, TraceError> {
        match value {
            "optimal" => Ok(Self::Optimal),
            "feasible" => Ok(Self::Feasible),
            "infeasible" => Ok(Self::Infeasible),
            "timed_out" => Ok(Self::TimedOut),
            "cancelled" => Ok(Self::Cancelled),
            "error" => Ok(Self::Error),
            _ => Err(TraceError::InvalidSolveStatus),
        }
    }

    const fn has_feasible_solution(self) -> bool {
        matches!(self, Self::Optimal | Self::Feasible)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolverTrace {
    pub schema_version: String,
    pub measurement_class: MeasurementClass,
    pub cohort_id: String,
    pub sample_id: String,
    pub scenario_id: String,
    pub scenario_panel_sha256: String,
    pub request_sha256: String,
    pub model_sha256: String,
    /// Stable temporal-mesh policy/design digest for a comparable cohort.
    pub mesh_policy_sha256: String,
    /// Exact selected mesh instance digest for this request and scenario.
    pub mesh_instance_sha256: String,
    pub solver_name: String,
    pub solver_version: String,
    pub solver_binary_sha256: String,
    /// Solver tolerances, presolve, thread count and other material settings.
    pub solver_config_sha256: String,
    pub hardware_profile_sha256: String,
    pub horizon_minutes: u16,
    pub interval_count: u32,
    pub continuous_variables: u64,
    pub integer_variables: u64,
    pub binary_variables: u64,
    pub constraints: u64,
    pub random_seed: u64,
    pub status: SolveStatus,
    /// Wall-clock process elapsed time. For timeouts this is right-censored at
    /// the recorded timeout limit, not an estimate of eventual completion time.
    pub elapsed_ns: u64,
    pub time_to_first_feasible_ns: Option<u64>,
    pub time_to_optimal_ns: Option<u64>,
    pub objective_value_units: Option<i64>,
    pub best_bound_units: Option<i64>,
    pub optimality_gap_ppm: Option<u32>,
    pub peak_memory_bytes: Option<u64>,
    pub timeout_limit_ns: u64,
}

impl SolverTrace {
    pub fn validate(&self) -> Result<(), TraceError> {
        if self.schema_version != TRACE_SCHEMA {
            return Err(TraceError::UnsupportedSchema);
        }
        for value in [
            &self.cohort_id,
            &self.sample_id,
            &self.scenario_id,
            &self.solver_name,
            &self.solver_version,
        ] {
            if !safe_identifier(value) {
                return Err(TraceError::InvalidIdentifier);
            }
        }
        for digest in [
            &self.scenario_panel_sha256,
            &self.request_sha256,
            &self.model_sha256,
            &self.mesh_policy_sha256,
            &self.mesh_instance_sha256,
            &self.solver_binary_sha256,
            &self.solver_config_sha256,
            &self.hardware_profile_sha256,
        ] {
            if !is_sha256_hex(digest) {
                return Err(TraceError::InvalidDigest);
            }
        }
        if self.horizon_minutes == 0
            || self.horizon_minutes > 1_440
            || self.interval_count == 0
            || self.binary_variables > self.integer_variables
            || self.elapsed_ns == 0
            || self.timeout_limit_ns == 0
        {
            return Err(TraceError::InvalidModelOrTiming);
        }
        if self
            .time_to_first_feasible_ns
            .is_some_and(|time| time > self.elapsed_ns)
            || self
                .time_to_optimal_ns
                .is_some_and(|time| time > self.elapsed_ns)
        {
            return Err(TraceError::InvalidModelOrTiming);
        }
        if self.objective_value_units.is_some() && self.time_to_first_feasible_ns.is_none() {
            return Err(TraceError::InvalidStatusSemantics);
        }
        if self.time_to_first_feasible_ns.is_some() && self.objective_value_units.is_none() {
            return Err(TraceError::InvalidStatusSemantics);
        }
        if self.time_to_optimal_ns.is_some() != (self.status == SolveStatus::Optimal) {
            return Err(TraceError::InvalidStatusSemantics);
        }
        if self.status == SolveStatus::Optimal
            && (self.time_to_first_feasible_ns.is_none()
                || self.objective_value_units.is_none()
                || self.best_bound_units.is_none()
                || self.optimality_gap_ppm.is_none())
        {
            return Err(TraceError::InvalidStatusSemantics);
        }
        if self.status == SolveStatus::Infeasible
            && (self.time_to_first_feasible_ns.is_some()
                || self.objective_value_units.is_some()
                || self.best_bound_units.is_some()
                || self.optimality_gap_ppm.is_some())
        {
            return Err(TraceError::InvalidStatusSemantics);
        }
        if self.optimality_gap_ppm.is_some()
            && (self.objective_value_units.is_none() || self.best_bound_units.is_none())
        {
            return Err(TraceError::InvalidStatusSemantics);
        }
        if self.status.has_feasible_solution()
            && (self.time_to_first_feasible_ns.is_none() || self.objective_value_units.is_none())
        {
            return Err(TraceError::InvalidStatusSemantics);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CohortIdentity {
    measurement_class: MeasurementClass,
    cohort_id: String,
    scenario_panel_sha256: String,
    model_sha256: String,
    mesh_policy_sha256: String,
    solver_name: String,
    solver_version: String,
    solver_binary_sha256: String,
    solver_config_sha256: String,
    hardware_profile_sha256: String,
    horizon_minutes: u16,
}

impl From<&SolverTrace> for CohortIdentity {
    fn from(trace: &SolverTrace) -> Self {
        Self {
            measurement_class: trace.measurement_class,
            cohort_id: trace.cohort_id.clone(),
            scenario_panel_sha256: trace.scenario_panel_sha256.clone(),
            model_sha256: trace.model_sha256.clone(),
            mesh_policy_sha256: trace.mesh_policy_sha256.clone(),
            solver_name: trace.solver_name.clone(),
            solver_version: trace.solver_version.clone(),
            solver_binary_sha256: trace.solver_binary_sha256.clone(),
            solver_config_sha256: trace.solver_config_sha256.clone(),
            hardware_profile_sha256: trace.hardware_profile_sha256.clone(),
            horizon_minutes: trace.horizon_minutes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceError {
    EmptyTraceSet,
    UnsupportedSchema,
    InvalidMeasurementClass,
    InvalidSolveStatus,
    InvalidIdentifier,
    InvalidDigest,
    InvalidModelOrTiming,
    InvalidStatusSemantics,
    InvalidCsvHeader,
    InvalidCsvFieldCount,
    CsvSizeLimitExceeded,
    InvalidCsvValue,
    DuplicateSampleId,
    MixedCohortIdentity,
}

impl fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TraceError {}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn parse_u64(value: &str) -> Result<u64, TraceError> {
    value.parse().map_err(|_| TraceError::InvalidCsvValue)
}

fn parse_u32(value: &str) -> Result<u32, TraceError> {
    value.parse().map_err(|_| TraceError::InvalidCsvValue)
}

fn parse_u16(value: &str) -> Result<u16, TraceError> {
    value.parse().map_err(|_| TraceError::InvalidCsvValue)
}

fn parse_i64(value: &str) -> Result<i64, TraceError> {
    value.parse().map_err(|_| TraceError::InvalidCsvValue)
}

fn parse_optional<T>(
    value: &str,
    parser: impl FnOnce(&str) -> Result<T, TraceError>,
) -> Result<Option<T>, TraceError> {
    if value == "NA" {
        Ok(None)
    } else {
        parser(value).map(Some)
    }
}

fn parse_csv_row(line: &str) -> Result<SolverTrace, TraceError> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 31 {
        return Err(TraceError::InvalidCsvFieldCount);
    }
    Ok(SolverTrace {
        schema_version: fields[0].to_owned(),
        measurement_class: MeasurementClass::parse(fields[1])?,
        cohort_id: fields[2].to_owned(),
        sample_id: fields[3].to_owned(),
        scenario_id: fields[4].to_owned(),
        scenario_panel_sha256: fields[5].to_owned(),
        request_sha256: fields[6].to_owned(),
        model_sha256: fields[7].to_owned(),
        mesh_policy_sha256: fields[8].to_owned(),
        mesh_instance_sha256: fields[9].to_owned(),
        solver_name: fields[10].to_owned(),
        solver_version: fields[11].to_owned(),
        solver_binary_sha256: fields[12].to_owned(),
        solver_config_sha256: fields[13].to_owned(),
        hardware_profile_sha256: fields[14].to_owned(),
        horizon_minutes: parse_u16(fields[15])?,
        interval_count: parse_u32(fields[16])?,
        continuous_variables: parse_u64(fields[17])?,
        integer_variables: parse_u64(fields[18])?,
        binary_variables: parse_u64(fields[19])?,
        constraints: parse_u64(fields[20])?,
        random_seed: parse_u64(fields[21])?,
        status: SolveStatus::parse(fields[22])?,
        elapsed_ns: parse_u64(fields[23])?,
        time_to_first_feasible_ns: parse_optional(fields[24], parse_u64)?,
        time_to_optimal_ns: parse_optional(fields[25], parse_u64)?,
        objective_value_units: parse_optional(fields[26], parse_i64)?,
        best_bound_units: parse_optional(fields[27], parse_i64)?,
        optimality_gap_ppm: parse_optional(fields[28], parse_u32)?,
        peak_memory_bytes: parse_optional(fields[29], parse_u64)?,
        timeout_limit_ns: parse_u64(fields[30])?,
    })
}

/// Strictly parses the fixed-column CSV. Identifiers are restricted to a
/// comma-free ASCII alphabet; free-form text and quoted CSV fields are not
/// accepted. This deliberately small grammar avoids ambiguous parsing.
pub fn parse_trace_csv(input: &str) -> Result<Vec<SolverTrace>, TraceError> {
    if input.len() > MAX_TRACE_CSV_BYTES {
        return Err(TraceError::CsvSizeLimitExceeded);
    }
    let mut lines = input.lines();
    let header = lines.next().ok_or(TraceError::InvalidCsvHeader)?;
    if header.trim_end_matches('\r') != TRACE_HEADER {
        return Err(TraceError::InvalidCsvHeader);
    }
    let mut traces = Vec::new();
    for line in lines {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            return Err(TraceError::InvalidCsvFieldCount);
        }
        let trace = parse_csv_row(line)?;
        trace.validate()?;
        traces.push(trace);
    }
    if traces.is_empty() {
        return Err(TraceError::EmptyTraceSet);
    }
    Ok(traces)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohortSummary {
    pub cohort_id: String,
    pub measurement_class: MeasurementClass,
    pub sample_count: usize,
    pub completed_sample_count: usize,
    pub first_feasible_sample_count: usize,
    pub peak_memory_sample_count: usize,
    pub optimal_count: usize,
    pub feasible_count: usize,
    pub infeasible_count: usize,
    pub timeout_count: usize,
    pub cancelled_count: usize,
    pub error_count: usize,
    pub completed_latency_p50_ns: Option<u64>,
    pub completed_latency_p95_ns: Option<u64>,
    pub completed_latency_p99_ns: Option<u64>,
    pub first_feasible_p50_ns: Option<u64>,
    pub first_feasible_p95_ns: Option<u64>,
    /// Includes timed-out attempts at their observed termination time. This
    /// is not an uncensored completion-time percentile.
    pub observed_attempt_elapsed_p95_ns: Option<u64>,
    pub peak_memory_p95_bytes: Option<u64>,
    pub interval_count_min: u32,
    pub interval_count_max: u32,
    pub binary_variables_min: u64,
    pub binary_variables_max: u64,
    pub constraints_min: u64,
    pub constraints_max: u64,
    pub p99_ready: bool,
}

fn nearest_rank(values: &mut [u64], percentile: usize) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let rank = (values.len() * percentile).div_ceil(100).max(1);
    values.get(rank - 1).copied()
}

fn gated_quantile(mut values: Vec<u64>, percentile: usize, minimum_samples: usize) -> Option<u64> {
    (values.len() >= minimum_samples)
        .then(|| nearest_rank(&mut values, percentile))
        .flatten()
}

fn summarize_group(traces: &[&SolverTrace]) -> CohortSummary {
    let first = traces[0];
    let mut completed_elapsed = Vec::new();
    let mut first_feasible = Vec::new();
    let mut observed_elapsed: Vec<u64> = traces.iter().map(|trace| trace.elapsed_ns).collect();
    let mut peak_memory = Vec::new();
    let mut optimal = 0;
    let mut feasible = 0;
    let mut infeasible = 0;
    let mut timeouts = 0;
    let mut cancelled = 0;
    let mut errors = 0;

    for trace in traces {
        match trace.status {
            SolveStatus::Optimal => {
                optimal += 1;
                completed_elapsed.push(trace.elapsed_ns);
            }
            SolveStatus::Feasible => {
                feasible += 1;
                completed_elapsed.push(trace.elapsed_ns);
            }
            SolveStatus::Infeasible => infeasible += 1,
            SolveStatus::TimedOut => timeouts += 1,
            SolveStatus::Cancelled => cancelled += 1,
            SolveStatus::Error => errors += 1,
        }
        if let Some(time) = trace.time_to_first_feasible_ns {
            first_feasible.push(time);
        }
        if let Some(bytes) = trace.peak_memory_bytes {
            peak_memory.push(bytes);
        }
    }
    observed_elapsed.shrink_to_fit();

    CohortSummary {
        cohort_id: first.cohort_id.clone(),
        measurement_class: first.measurement_class,
        sample_count: traces.len(),
        completed_sample_count: completed_elapsed.len(),
        first_feasible_sample_count: first_feasible.len(),
        peak_memory_sample_count: peak_memory.len(),
        optimal_count: optimal,
        feasible_count: feasible,
        infeasible_count: infeasible,
        timeout_count: timeouts,
        cancelled_count: cancelled,
        error_count: errors,
        completed_latency_p50_ns: gated_quantile(completed_elapsed.clone(), 50, P50_MIN_SAMPLES),
        completed_latency_p95_ns: gated_quantile(completed_elapsed.clone(), 95, P95_MIN_SAMPLES),
        completed_latency_p99_ns: gated_quantile(completed_elapsed, 99, P99_MIN_SAMPLES),
        first_feasible_p50_ns: gated_quantile(first_feasible.clone(), 50, P50_MIN_SAMPLES),
        first_feasible_p95_ns: gated_quantile(first_feasible, 95, P95_MIN_SAMPLES),
        observed_attempt_elapsed_p95_ns: gated_quantile(observed_elapsed, 95, P95_MIN_SAMPLES),
        peak_memory_p95_bytes: gated_quantile(peak_memory, 95, P95_MIN_SAMPLES),
        interval_count_min: traces
            .iter()
            .map(|trace| trace.interval_count)
            .min()
            .unwrap(),
        interval_count_max: traces
            .iter()
            .map(|trace| trace.interval_count)
            .max()
            .unwrap(),
        binary_variables_min: traces
            .iter()
            .map(|trace| trace.binary_variables)
            .min()
            .unwrap(),
        binary_variables_max: traces
            .iter()
            .map(|trace| trace.binary_variables)
            .max()
            .unwrap(),
        constraints_min: traces.iter().map(|trace| trace.constraints).min().unwrap(),
        constraints_max: traces.iter().map(|trace| trace.constraints).max().unwrap(),
        p99_ready: traces
            .iter()
            .filter(|trace| trace.status.has_feasible_solution())
            .count()
            >= P99_MIN_SAMPLES,
    }
}

/// Validate sample identities and prohibit aggregation across different solver,
/// model, mesh, hardware or scenario-panel identities.
pub fn summarize_traces(traces: &[SolverTrace]) -> Result<Vec<CohortSummary>, TraceError> {
    if traces.is_empty() {
        return Err(TraceError::EmptyTraceSet);
    }
    let mut groups = BTreeMap::<String, Vec<&SolverTrace>>::new();
    let mut cohort_identities = BTreeMap::<String, CohortIdentity>::new();
    let mut sample_ids = BTreeMap::<String, BTreeSet<String>>::new();

    for trace in traces {
        trace.validate()?;
        let identity = CohortIdentity::from(trace);
        if let Some(existing) = cohort_identities.get(&trace.cohort_id) {
            if existing != &identity {
                return Err(TraceError::MixedCohortIdentity);
            }
        } else {
            cohort_identities.insert(trace.cohort_id.clone(), identity);
        }
        if !sample_ids
            .entry(trace.cohort_id.clone())
            .or_default()
            .insert(trace.sample_id.clone())
        {
            return Err(TraceError::DuplicateSampleId);
        }
        groups
            .entry(trace.cohort_id.clone())
            .or_default()
            .push(trace);
    }

    Ok(groups
        .values()
        .map(|group| summarize_group(group))
        .collect())
}

pub fn render_summary_csv(summaries: &[CohortSummary]) -> String {
    let mut output = String::from(
        "cohort_id,measurement_class,samples,completed_samples,first_feasible_samples,peak_memory_samples,optimal,feasible,infeasible,timeouts,cancelled,errors,completed_p50_ns,completed_p95_ns,completed_p99_ns,first_feasible_p50_ns,first_feasible_p95_ns,observed_attempt_elapsed_p95_ns,peak_memory_p95_bytes,interval_count_min,interval_count_max,binary_variables_min,binary_variables_max,constraints_min,constraints_max,p99_ready\n",
    );
    for summary in summaries {
        let optional = |value: Option<u64>| {
            value
                .map(|number| number.to_string())
                .unwrap_or_else(|| "NA".to_owned())
        };
        output.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            summary.cohort_id,
            summary.measurement_class.as_str(),
            summary.sample_count,
            summary.completed_sample_count,
            summary.first_feasible_sample_count,
            summary.peak_memory_sample_count,
            summary.optimal_count,
            summary.feasible_count,
            summary.infeasible_count,
            summary.timeout_count,
            summary.cancelled_count,
            summary.error_count,
            optional(summary.completed_latency_p50_ns),
            optional(summary.completed_latency_p95_ns),
            optional(summary.completed_latency_p99_ns),
            optional(summary.first_feasible_p50_ns),
            optional(summary.first_feasible_p95_ns),
            optional(summary.observed_attempt_elapsed_p95_ns),
            optional(summary.peak_memory_p95_bytes),
            summary.interval_count_min,
            summary.interval_count_max,
            summary.binary_variables_min,
            summary.binary_variables_max,
            summary.constraints_min,
            summary.constraints_max,
            summary.p99_ready,
        ));
    }
    output
}

#[cfg(test)]
fn trace_to_csv(trace: &SolverTrace) -> String {
    let optional = |value: Option<u64>| {
        value
            .map(|number| number.to_string())
            .unwrap_or_else(|| "NA".to_owned())
    };
    let optional_i64 = |value: Option<i64>| {
        value
            .map(|number| number.to_string())
            .unwrap_or_else(|| "NA".to_owned())
    };
    let optional_u32 = |value: Option<u32>| {
        value
            .map(|number| number.to_string())
            .unwrap_or_else(|| "NA".to_owned())
    };
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        trace.schema_version,
        trace.measurement_class.as_str(),
        trace.cohort_id,
        trace.sample_id,
        trace.scenario_id,
        trace.scenario_panel_sha256,
        trace.request_sha256,
        trace.model_sha256,
        trace.mesh_policy_sha256,
        trace.mesh_instance_sha256,
        trace.solver_name,
        trace.solver_version,
        trace.solver_binary_sha256,
        trace.solver_config_sha256,
        trace.hardware_profile_sha256,
        trace.horizon_minutes,
        trace.interval_count,
        trace.continuous_variables,
        trace.integer_variables,
        trace.binary_variables,
        trace.constraints,
        trace.random_seed,
        trace.status.as_str(),
        trace.elapsed_ns,
        optional(trace.time_to_first_feasible_ns),
        optional(trace.time_to_optimal_ns),
        optional_i64(trace.objective_value_units),
        optional_i64(trace.best_bound_units),
        optional_u32(trace.optimality_gap_ppm),
        optional(trace.peak_memory_bytes),
        trace.timeout_limit_ns,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn trace(sample: usize, status: SolveStatus) -> SolverTrace {
        let timeout_limit_ns = 20_000_000;
        let elapsed_ns = if status == SolveStatus::TimedOut {
            timeout_limit_ns
        } else {
            1_000_000 + sample as u64 * 10_000
        };
        let feasible = status.has_feasible_solution();
        SolverTrace {
            schema_version: TRACE_SCHEMA.to_owned(),
            measurement_class: MeasurementClass::Synthetic,
            cohort_id: "cohort-a".to_owned(),
            sample_id: format!("sample-{sample:04}"),
            scenario_id: format!("scenario-{:02}", sample % 5),
            scenario_panel_sha256: digest('a'),
            request_sha256: digest(char::from(b'b' + (sample % 5) as u8)),
            model_sha256: digest('c'),
            mesh_policy_sha256: digest('d'),
            mesh_instance_sha256: digest(char::from(b'0' + (sample % 10) as u8)),
            solver_name: "highs".to_owned(),
            solver_version: "1.12.0".to_owned(),
            solver_binary_sha256: digest('e'),
            solver_config_sha256: digest('1'),
            hardware_profile_sha256: digest('f'),
            horizon_minutes: 1_440,
            interval_count: 96,
            continuous_variables: 400,
            integer_variables: 96,
            binary_variables: 48,
            constraints: 800,
            random_seed: sample as u64,
            status,
            elapsed_ns,
            time_to_first_feasible_ns: feasible.then_some(elapsed_ns / 2),
            time_to_optimal_ns: (status == SolveStatus::Optimal).then_some(elapsed_ns),
            objective_value_units: feasible.then_some(10_000 - sample as i64),
            best_bound_units: feasible.then_some(10_000 - sample as i64),
            optimality_gap_ppm: feasible.then_some(0),
            peak_memory_bytes: Some(2_000_000 + sample as u64 * 1_024),
            timeout_limit_ns,
        }
    }

    #[test]
    fn validates_exact_trace_identity_and_timing() {
        assert!(trace(0, SolveStatus::Optimal).validate().is_ok());
    }

    #[test]
    fn rejects_malformed_sha256_and_identifiers() {
        let mut bad_digest = trace(0, SolveStatus::Optimal);
        bad_digest.mesh_instance_sha256 = "deadbeef".to_owned();
        assert_eq!(bad_digest.validate(), Err(TraceError::InvalidDigest));

        let mut bad_id = trace(0, SolveStatus::Optimal);
        bad_id.sample_id = "sample,spoof".to_owned();
        assert_eq!(bad_id.validate(), Err(TraceError::InvalidIdentifier));
    }

    #[test]
    fn rejects_binary_variable_count_above_integer_variable_count() {
        let mut row = trace(0, SolveStatus::Optimal);
        row.binary_variables = row.integer_variables + 1;
        assert_eq!(row.validate(), Err(TraceError::InvalidModelOrTiming));
    }

    #[test]
    fn rejects_first_feasible_time_after_termination() {
        let mut row = trace(0, SolveStatus::Feasible);
        row.time_to_first_feasible_ns = Some(row.elapsed_ns + 1);
        assert_eq!(row.validate(), Err(TraceError::InvalidModelOrTiming));
    }

    #[test]
    fn rejects_optimal_status_without_proof_completion_and_bound() {
        let mut row = trace(0, SolveStatus::Optimal);
        row.time_to_optimal_ns = None;
        assert_eq!(row.validate(), Err(TraceError::InvalidStatusSemantics));
    }

    #[test]
    fn infeasible_status_cannot_carry_an_incumbent_objective() {
        let mut row = trace(0, SolveStatus::Infeasible);
        row.objective_value_units = Some(10);
        assert_eq!(row.validate(), Err(TraceError::InvalidStatusSemantics));
    }

    #[test]
    fn rejects_gap_without_both_incumbent_and_best_bound() {
        let mut row = trace(0, SolveStatus::TimedOut);
        row.optimality_gap_ppm = Some(10_000);
        assert_eq!(row.validate(), Err(TraceError::InvalidStatusSemantics));
    }

    #[test]
    fn refuses_to_pool_different_meshes_under_one_cohort_id() {
        let rows = vec![trace(0, SolveStatus::Optimal), {
            let mut row = trace(1, SolveStatus::Optimal);
            row.mesh_policy_sha256 = digest('9');
            row
        }];
        assert_eq!(
            summarize_traces(&rows),
            Err(TraceError::MixedCohortIdentity)
        );
    }

    #[test]
    fn refuses_to_pool_different_hardware_or_solver_binaries() {
        let rows = vec![trace(0, SolveStatus::Optimal), {
            let mut row = trace(1, SolveStatus::Optimal);
            row.hardware_profile_sha256 = digest('9');
            row
        }];
        assert_eq!(
            summarize_traces(&rows),
            Err(TraceError::MixedCohortIdentity)
        );

        let rows = vec![trace(0, SolveStatus::Optimal), {
            let mut row = trace(1, SolveStatus::Optimal);
            row.solver_binary_sha256 = digest('9');
            row
        }];
        assert_eq!(
            summarize_traces(&rows),
            Err(TraceError::MixedCohortIdentity)
        );

        let rows = vec![trace(0, SolveStatus::Optimal), {
            let mut row = trace(1, SolveStatus::Optimal);
            row.solver_config_sha256 = digest('9');
            row
        }];
        assert_eq!(
            summarize_traces(&rows),
            Err(TraceError::MixedCohortIdentity)
        );
    }

    #[test]
    fn exact_mesh_instances_vary_within_one_stable_mesh_policy_cohort() {
        let first = trace(0, SolveStatus::Optimal);
        let mut second = trace(1, SolveStatus::Optimal);
        assert_eq!(first.mesh_policy_sha256, second.mesh_policy_sha256);
        second.mesh_instance_sha256 = digest('9');

        let summary = summarize_traces(&[first, second]).unwrap();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].sample_count, 2);
    }

    #[test]
    fn duplicate_sample_ids_do_not_create_fake_repeats() {
        let first = trace(0, SolveStatus::Optimal);
        let mut duplicate = trace(1, SolveStatus::Optimal);
        duplicate.sample_id = first.sample_id.clone();
        assert_eq!(
            summarize_traces(&[first, duplicate]),
            Err(TraceError::DuplicateSampleId)
        );
    }

    #[test]
    fn timeout_with_an_incumbent_is_a_timeout_and_a_first_feasible_observation() {
        let mut row = trace(22, SolveStatus::TimedOut);
        row.time_to_first_feasible_ns = Some(10_000_000);
        row.objective_value_units = Some(9_000);
        row.best_bound_units = Some(8_900);
        row.optimality_gap_ppm = Some(11_111);
        assert!(row.validate().is_ok());

        let summary = summarize_traces(&[row]).unwrap().remove(0);
        assert_eq!(summary.timeout_count, 1);
        assert_eq!(summary.completed_sample_count, 0);
        assert_eq!(summary.first_feasible_sample_count, 1);
        assert_eq!(summary.completed_latency_p50_ns, None);
        assert_eq!(summary.first_feasible_p50_ns, None);
    }

    #[test]
    fn completed_and_memory_percentiles_expose_their_own_denominators() {
        let rows: Vec<SolverTrace> = (0..30)
            .map(|index| match index {
                0..=17 => trace(index, SolveStatus::Optimal),
                18..=21 => trace(index, SolveStatus::Feasible),
                22..=25 => trace(index, SolveStatus::TimedOut),
                _ => {
                    let mut row = trace(index, SolveStatus::Infeasible);
                    row.peak_memory_bytes = None;
                    row
                }
            })
            .collect();
        let summary = summarize_traces(&rows).unwrap().remove(0);
        assert_eq!(summary.sample_count, 30);
        assert_eq!(summary.completed_sample_count, 22);
        assert_eq!(summary.first_feasible_sample_count, 22);
        assert_eq!(summary.peak_memory_sample_count, 26);
    }

    #[test]
    fn p99_is_withheld_until_one_thousand_completed_samples() {
        let rows: Vec<SolverTrace> = (0..999)
            .map(|index| trace(index, SolveStatus::Optimal))
            .collect();
        let summary = summarize_traces(&rows).unwrap().remove(0);
        assert_eq!(summary.sample_count, 999);
        assert!(!summary.p99_ready);
        assert_eq!(summary.completed_latency_p99_ns, None);

        let rows: Vec<SolverTrace> = (0..1_000)
            .map(|index| trace(index, SolveStatus::Optimal))
            .collect();
        let summary = summarize_traces(&rows).unwrap().remove(0);
        assert!(summary.p99_ready);
        assert!(summary.completed_latency_p99_ns.is_some());
    }

    #[test]
    fn percentile_uses_nearest_rank_not_interpolated_quantiles() {
        let mut values = [1, 2, 3, 4, 100];
        assert_eq!(nearest_rank(&mut values, 95), Some(100));
    }

    #[test]
    fn sample_order_does_not_change_aggregate_results() {
        let mut rows: Vec<SolverTrace> = (0..30)
            .map(|index| {
                if index < 18 {
                    trace(index, SolveStatus::Optimal)
                } else if index < 22 {
                    trace(index, SolveStatus::Feasible)
                } else if index < 26 {
                    trace(index, SolveStatus::TimedOut)
                } else {
                    trace(index, SolveStatus::Infeasible)
                }
            })
            .collect();
        let first = summarize_traces(&rows).unwrap();
        rows.reverse();
        assert_eq!(first, summarize_traces(&rows).unwrap());
    }

    #[test]
    fn timeout_and_infeasible_denominators_are_separate_from_completion_latency() {
        let rows: Vec<SolverTrace> = (0..30)
            .map(|index| {
                if index < 20 {
                    trace(index, SolveStatus::Optimal)
                } else if index < 24 {
                    trace(index, SolveStatus::TimedOut)
                } else if index < 30 {
                    trace(index, SolveStatus::Infeasible)
                } else {
                    trace(index, SolveStatus::Error)
                }
            })
            .collect();
        let summary = summarize_traces(&rows).unwrap().remove(0);
        assert_eq!(summary.sample_count, 30);
        assert_eq!(summary.optimal_count, 20);
        assert_eq!(summary.timeout_count, 4);
        assert_eq!(summary.infeasible_count, 6);
        assert_eq!(summary.completed_sample_count, 20);
        assert_eq!(summary.completed_latency_p95_ns, Some(1_180_000));
        assert_eq!(summary.observed_attempt_elapsed_p95_ns, Some(20_000_000));
    }

    #[test]
    fn measurement_class_cannot_be_mixed_inside_one_cohort() {
        let first = trace(0, SolveStatus::Optimal);
        let mut second = trace(1, SolveStatus::Optimal);
        second.measurement_class = MeasurementClass::Measured;
        assert_eq!(
            summarize_traces(&[first, second]),
            Err(TraceError::MixedCohortIdentity)
        );
    }

    #[test]
    fn checked_in_csv_fixture_is_parseable_and_keeps_its_synthetic_label() {
        let rows = parse_trace_csv(include_str!("../fixtures/synthetic-traces.csv")).unwrap();
        assert_eq!(rows.len(), 30);
        assert!(
            rows.iter()
                .all(|row| row.measurement_class == MeasurementClass::Synthetic)
        );
        let summary = summarize_traces(&rows).unwrap().remove(0);
        assert_eq!(summary.optimal_count, 18);
        assert_eq!(summary.feasible_count, 4);
        assert_eq!(summary.timeout_count, 4);
        assert_eq!(summary.infeasible_count, 4);
        assert_eq!(summary.completed_sample_count, 22);
        assert_eq!(summary.first_feasible_sample_count, 22);
        assert_eq!(summary.peak_memory_sample_count, 30);
        assert_eq!(summary.completed_latency_p95_ns, Some(1_200_000));
        assert_eq!(summary.first_feasible_p95_ns, Some(600_000));
        assert_eq!(summary.peak_memory_p95_bytes, Some(2_028_672));
        assert_eq!(summary.observed_attempt_elapsed_p95_ns, Some(20_000_000));
        assert_eq!(summary.completed_latency_p99_ns, None);
        assert!(!summary.p99_ready);
    }

    #[test]
    fn parser_rejects_wrong_header_and_missing_columns() {
        assert_eq!(
            parse_trace_csv("schema_version,wrong\n"),
            Err(TraceError::InvalidCsvHeader)
        );
        assert_eq!(
            parse_csv_row("only,three,fields"),
            Err(TraceError::InvalidCsvFieldCount)
        );
    }

    #[test]
    fn parser_roundtrips_checked_in_synthetic_rows() {
        let input = format!(
            "{TRACE_HEADER}\n{}",
            (0..30)
                .map(|index| trace_to_csv(&trace(
                    index,
                    if index < 18 {
                        SolveStatus::Optimal
                    } else if index < 22 {
                        SolveStatus::Feasible
                    } else if index < 26 {
                        SolveStatus::TimedOut
                    } else {
                        SolveStatus::Infeasible
                    }
                )))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let rows = parse_trace_csv(&input).unwrap();
        assert_eq!(rows.len(), 30);
        assert_eq!(summarize_traces(&rows).unwrap()[0].timeout_count, 4);
    }
}
