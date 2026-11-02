#![forbid(unsafe_code)]
//! Versioned, fail-closed trace records for anytime search and independent audit.
//!
//! v0.2 remains unchanged. This module represents candidate feasibility,
//! search completeness, audit disposition and service-floor qualification as
//! separate facts. It deliberately does not infer actuation authority.

use super::{MeasurementClass, P50_MIN_SAMPLES, P95_MIN_SAMPLES};
use std::collections::BTreeMap;

pub const TRACE_V03_SCHEMA: &str = "energy-solver-compute-trace-v0.3";
pub const TRACE_V03_HEADER: &str = "schema_version,measurement_class,cohort_id,sample_id,scenario_id,scenario_panel_sha256,request_sha256,model_sha256,mesh_policy_sha256,mesh_instance_sha256,solver_name,solver_version,solver_binary_sha256,solver_config_sha256,hardware_profile_sha256,horizon_minutes,interval_count,search_disposition,elapsed_ns,time_to_first_candidate_ns,time_to_first_audit_accept_ns,peak_memory_bytes,timeout_limit_ns,enumeration_prefix_visits,feasible_complete_schedules,pareto_frontier_complete,pareto_frontier_count,candidate_complete_horizon,candidate_schedule_sha256,audit_disposition,audit_receipt_sha256,audit_request_sha256,audit_schedule_sha256,auditor_binary_sha256,audit_elapsed_ns,service_floor_disposition,service_floor_policy_sha256,service_floor_schedule_audit_receipt_sha256,service_floor_receipt_sha256,service_floor_auditor_binary_sha256";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SearchDispositionV03 {
    ExactComplete,
    FeasibleUnproven,
    ProvenInfeasible,
    LimitWithoutCandidate,
    TimedOut,
    Cancelled,
    Error,
}

impl SearchDispositionV03 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactComplete => "exact_complete",
            Self::FeasibleUnproven => "feasible_unproven",
            Self::ProvenInfeasible => "proven_infeasible",
            Self::LimitWithoutCandidate => "limit_without_candidate",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::Error => "error",
        }
    }

    fn parse(value: &str) -> Result<Self, TraceV03Error> {
        match value {
            "exact_complete" => Ok(Self::ExactComplete),
            "feasible_unproven" => Ok(Self::FeasibleUnproven),
            "proven_infeasible" => Ok(Self::ProvenInfeasible),
            "limit_without_candidate" => Ok(Self::LimitWithoutCandidate),
            "timed_out" => Ok(Self::TimedOut),
            "cancelled" => Ok(Self::Cancelled),
            "error" => Ok(Self::Error),
            _ => Err(TraceV03Error::InvalidSearchDisposition),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditDispositionV03 {
    NotRun,
    Accepted,
    Rejected,
    Unavailable,
}

impl AuditDispositionV03 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRun => "not_run",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Unavailable => "unavailable",
        }
    }

    fn parse(value: &str) -> Result<Self, TraceV03Error> {
        match value {
            "not_run" => Ok(Self::NotRun),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err(TraceV03Error::InvalidAuditDisposition),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ServiceFloorDispositionV03 {
    NotEvaluated,
    Passed,
    Failed,
    Unavailable,
}

impl ServiceFloorDispositionV03 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotEvaluated => "not_evaluated",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unavailable => "unavailable",
        }
    }

    fn parse(value: &str) -> Result<Self, TraceV03Error> {
        match value {
            "not_evaluated" => Ok(Self::NotEvaluated),
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err(TraceV03Error::InvalidServiceFloorDisposition),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceV03Error {
    InvalidHeader,
    InvalidFieldCount,
    UnsupportedSchema,
    InvalidIdentifier,
    InvalidDigest,
    InvalidNumber,
    InvalidBoolean,
    InvalidSearchDisposition,
    InvalidAuditDisposition,
    InvalidServiceFloorDisposition,
    InvalidCandidateBindings,
    InvalidSearchSemantics,
    InvalidAuditBindings,
    InvalidServiceFloorBindings,
    InvalidTiming,
    MixedCohortIdentity,
    EmptyCohort,
    InputTooLarge,
    DuplicateSampleConflict,
}

impl std::fmt::Display for TraceV03Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TraceV03Error {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolverTraceV03 {
    pub schema_version: String,
    pub measurement_class: MeasurementClass,
    pub cohort_id: String,
    pub sample_id: String,
    pub scenario_id: String,
    pub scenario_panel_sha256: String,
    pub request_sha256: String,
    pub model_sha256: String,
    pub mesh_policy_sha256: String,
    pub mesh_instance_sha256: String,
    pub solver_name: String,
    pub solver_version: String,
    pub solver_binary_sha256: String,
    pub solver_config_sha256: String,
    pub hardware_profile_sha256: String,
    pub horizon_minutes: u16,
    pub interval_count: u32,
    pub search_disposition: SearchDispositionV03,
    /// Entire solver invocation elapsed time. Audit time is included only when
    /// the runner actually executes the auditor within this invocation.
    pub elapsed_ns: u64,
    /// Optional: only present when the kernel measured the first complete candidate.
    pub time_to_first_candidate_ns: Option<u64>,
    /// Optional: invocation-relative time when an exact receipt was accepted.
    pub time_to_first_audit_accept_ns: Option<u64>,
    pub peak_memory_bytes: Option<u64>,
    pub timeout_limit_ns: Option<u64>,
    /// Five-level enumeration-prefix visits for the current exact dispatcher.
    pub enumeration_prefix_visits: Option<u64>,
    pub feasible_complete_schedules: Option<u64>,
    /// True only if the entire finite frontier/search result is complete.
    pub pareto_frontier_complete: bool,
    /// Only populated when the frontier is complete.
    pub pareto_frontier_count: Option<u32>,
    pub candidate_complete_horizon: bool,
    pub candidate_schedule_sha256: Option<String>,
    pub audit_disposition: AuditDispositionV03,
    pub audit_receipt_sha256: Option<String>,
    pub audit_request_sha256: Option<String>,
    pub audit_schedule_sha256: Option<String>,
    pub auditor_binary_sha256: Option<String>,
    pub audit_elapsed_ns: Option<u64>,
    pub service_floor_disposition: ServiceFloorDispositionV03,
    pub service_floor_policy_sha256: Option<String>,
    pub service_floor_schedule_audit_receipt_sha256: Option<String>,
    pub service_floor_receipt_sha256: Option<String>,
    pub service_floor_auditor_binary_sha256: Option<String>,
}

impl SolverTraceV03 {
    pub fn candidate_available(&self) -> bool {
        self.candidate_schedule_sha256.is_some() && self.candidate_complete_horizon
    }

    /// A decision-ready benchmark outcome needs a complete candidate, an
    /// accepted exact-byte audit receipt and a passed, digest-bound service floor.
    /// This is not actuation authorization.
    pub fn decision_ready(&self) -> bool {
        self.candidate_available()
            && self.audit_disposition == AuditDispositionV03::Accepted
            && self.service_floor_disposition == ServiceFloorDispositionV03::Passed
    }

    pub fn validate(&self) -> Result<(), TraceV03Error> {
        if self.schema_version != TRACE_V03_SCHEMA {
            return Err(TraceV03Error::UnsupportedSchema);
        }
        for identifier in [
            &self.cohort_id,
            &self.sample_id,
            &self.scenario_id,
            &self.solver_name,
            &self.solver_version,
        ] {
            if !super::safe_identifier(identifier) {
                return Err(TraceV03Error::InvalidIdentifier);
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
            if !super::is_sha256_hex(digest) {
                return Err(TraceV03Error::InvalidDigest);
            }
        }
        if self.horizon_minutes == 0
            || self.horizon_minutes > 1_440
            || self.interval_count == 0
            || self.elapsed_ns == 0
            || self.timeout_limit_ns.is_some_and(|value| value == 0)
            || self.audit_elapsed_ns.is_some_and(|value| value == 0)
        {
            return Err(TraceV03Error::InvalidNumber);
        }
        if self
            .time_to_first_candidate_ns
            .is_some_and(|time| time > self.elapsed_ns)
            || self
                .time_to_first_audit_accept_ns
                .is_some_and(|time| time > self.elapsed_ns)
            || self
                .time_to_first_audit_accept_ns
                .is_some_and(|_| self.audit_disposition != AuditDispositionV03::Accepted)
        {
            return Err(TraceV03Error::InvalidTiming);
        }

        let candidate_present = self.candidate_schedule_sha256.is_some();
        if candidate_present != self.candidate_complete_horizon {
            return Err(TraceV03Error::InvalidCandidateBindings);
        }
        if let Some(schedule_sha256) = &self.candidate_schedule_sha256 {
            if !super::is_sha256_hex(schedule_sha256) {
                return Err(TraceV03Error::InvalidDigest);
            }
            if self.interval_count == 0 {
                return Err(TraceV03Error::InvalidCandidateBindings);
            }
        }
        if candidate_present && self.feasible_complete_schedules.is_none_or(|count| count == 0) {
            return Err(TraceV03Error::InvalidCandidateBindings);
        }
        if self
            .time_to_first_candidate_ns
            .is_some_and(|_| !candidate_present)
        {
            return Err(TraceV03Error::InvalidTiming);
        }

        match self.search_disposition {
            SearchDispositionV03::ExactComplete => {
                if !candidate_present
                    || !self.pareto_frontier_complete
                    || self.pareto_frontier_count.is_none_or(|count| count == 0)
                    || self.feasible_complete_schedules.is_none_or(|count| count == 0)
                    || self.enumeration_prefix_visits.is_none_or(|count| count == 0)
                {
                    return Err(TraceV03Error::InvalidSearchSemantics);
                }
            }
            SearchDispositionV03::FeasibleUnproven => {
                if !candidate_present
                    || self.pareto_frontier_complete
                    || self.pareto_frontier_count.is_some()
                    || self.feasible_complete_schedules.is_none_or(|count| count == 0)
                    || self.enumeration_prefix_visits.is_none_or(|count| count == 0)
                {
                    return Err(TraceV03Error::InvalidSearchSemantics);
                }
            }
            SearchDispositionV03::ProvenInfeasible => {
                if candidate_present
                    || !self.pareto_frontier_complete
                    || self.pareto_frontier_count != Some(0)
                    || self.feasible_complete_schedules != Some(0)
                    || self.enumeration_prefix_visits.is_none_or(|count| count == 0)
                {
                    return Err(TraceV03Error::InvalidSearchSemantics);
                }
            }
            SearchDispositionV03::LimitWithoutCandidate => {
                if candidate_present
                    || self.pareto_frontier_complete
                    || self.pareto_frontier_count.is_some()
                    || self.feasible_complete_schedules != Some(0)
                    || self.enumeration_prefix_visits.is_none_or(|count| count == 0)
                {
                    return Err(TraceV03Error::InvalidSearchSemantics);
                }
            }
            SearchDispositionV03::TimedOut
            | SearchDispositionV03::Cancelled
            | SearchDispositionV03::Error => {
                if self.pareto_frontier_complete || self.pareto_frontier_count.is_some() {
                    return Err(TraceV03Error::InvalidSearchSemantics);
                }
                if candidate_present && self.feasible_complete_schedules.is_none_or(|count| count == 0) {
                    return Err(TraceV03Error::InvalidSearchSemantics);
                }
            }
        }

        let audit_receipt_present = self.audit_receipt_sha256.is_some();
        let audit_binding_present = self.audit_request_sha256.is_some()
            && self.audit_schedule_sha256.is_some()
            && self.auditor_binary_sha256.is_some();
        match self.audit_disposition {
            AuditDispositionV03::NotRun => {
                if audit_receipt_present
                    || audit_binding_present
                    || self.audit_request_sha256.is_some()
                    || self.audit_schedule_sha256.is_some()
                    || self.auditor_binary_sha256.is_some()
                    || self.audit_elapsed_ns.is_some()
                    || self.time_to_first_audit_accept_ns.is_some()
                {
                    return Err(TraceV03Error::InvalidAuditBindings);
                }
            }
            AuditDispositionV03::Accepted | AuditDispositionV03::Rejected => {
                if !candidate_present
                    || !audit_receipt_present
                    || !audit_binding_present
                    || self.audit_elapsed_ns.is_none()
                    || self.audit_request_sha256.as_deref() != Some(self.request_sha256.as_str())
                    || self.audit_schedule_sha256.as_deref()
                        != self.candidate_schedule_sha256.as_deref()
                {
                    return Err(TraceV03Error::InvalidAuditBindings);
                }
                for digest in [
                    self.audit_receipt_sha256.as_ref(),
                    self.audit_request_sha256.as_ref(),
                    self.audit_schedule_sha256.as_ref(),
                    self.auditor_binary_sha256.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    if !super::is_sha256_hex(digest) {
                        return Err(TraceV03Error::InvalidDigest);
                    }
                }
            }
            AuditDispositionV03::Unavailable => {
                if !candidate_present
                    || audit_receipt_present
                    || self.audit_request_sha256.is_some()
                    || self.audit_schedule_sha256.is_some()
                    || self.audit_elapsed_ns.is_none()
                    || self.time_to_first_audit_accept_ns.is_some()
                {
                    return Err(TraceV03Error::InvalidAuditBindings);
                }
                if let Some(digest) = &self.auditor_binary_sha256 {
                    if !super::is_sha256_hex(digest) {
                        return Err(TraceV03Error::InvalidDigest);
                    }
                }
            }
        }

        match self.service_floor_disposition {
            ServiceFloorDispositionV03::NotEvaluated => {
                if self.service_floor_policy_sha256.is_some()
                    || self.service_floor_schedule_audit_receipt_sha256.is_some()
                    || self.service_floor_receipt_sha256.is_some()
                    || self.service_floor_auditor_binary_sha256.is_some()
                {
                    return Err(TraceV03Error::InvalidServiceFloorBindings);
                }
            }
            ServiceFloorDispositionV03::Passed | ServiceFloorDispositionV03::Failed => {
                if !candidate_present
                    || self.audit_disposition != AuditDispositionV03::Accepted
                    || self
                        .service_floor_policy_sha256
                        .as_ref()
                        .is_none_or(|digest| !super::is_sha256_hex(digest))
                    || self.service_floor_schedule_audit_receipt_sha256
                        != self.audit_receipt_sha256
                    || self
                        .service_floor_receipt_sha256
                        .as_ref()
                        .is_none_or(|digest| !super::is_sha256_hex(digest))
                    || self
                        .service_floor_auditor_binary_sha256
                        .as_ref()
                        .is_none_or(|digest| !super::is_sha256_hex(digest))
                {
                    return Err(TraceV03Error::InvalidServiceFloorBindings);
                }
            }
            ServiceFloorDispositionV03::Unavailable => {
                if !candidate_present
                    || self
                        .service_floor_policy_sha256
                        .as_ref()
                        .is_none_or(|digest| !super::is_sha256_hex(digest))
                    || self.service_floor_schedule_audit_receipt_sha256.is_some()
                    || self.service_floor_receipt_sha256.is_some()
                {
                    return Err(TraceV03Error::InvalidServiceFloorBindings);
                }
                if let Some(digest) = &self.service_floor_auditor_binary_sha256 {
                    if !super::is_sha256_hex(digest) {
                        return Err(TraceV03Error::InvalidDigest);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn render_csv_row(&self) -> Result<String, TraceV03Error> {
        self.validate()?;
        let option = |value: Option<u64>| value.map(|n| n.to_string()).unwrap_or_else(|| "NA".to_owned());
        let option_u32 = |value: Option<u32>| value.map(|n| n.to_string()).unwrap_or_else(|| "NA".to_owned());
        let option_digest = |value: &Option<String>| value.as_deref().unwrap_or("NA");
        let fields = vec![
            self.schema_version.clone(),
            self.measurement_class.as_str().to_owned(),
            self.cohort_id.clone(),
            self.sample_id.clone(),
            self.scenario_id.clone(),
            self.scenario_panel_sha256.clone(),
            self.request_sha256.clone(),
            self.model_sha256.clone(),
            self.mesh_policy_sha256.clone(),
            self.mesh_instance_sha256.clone(),
            self.solver_name.clone(),
            self.solver_version.clone(),
            self.solver_binary_sha256.clone(),
            self.solver_config_sha256.clone(),
            self.hardware_profile_sha256.clone(),
            self.horizon_minutes.to_string(),
            self.interval_count.to_string(),
            self.search_disposition.as_str().to_owned(),
            self.elapsed_ns.to_string(),
            option(self.time_to_first_candidate_ns),
            option(self.time_to_first_audit_accept_ns),
            option(self.peak_memory_bytes),
            option(self.timeout_limit_ns),
            option(self.enumeration_prefix_visits),
            option(self.feasible_complete_schedules),
            self.pareto_frontier_complete.to_string(),
            option_u32(self.pareto_frontier_count),
            self.candidate_complete_horizon.to_string(),
            option_digest(&self.candidate_schedule_sha256).to_owned(),
            self.audit_disposition.as_str().to_owned(),
            option_digest(&self.audit_receipt_sha256).to_owned(),
            option_digest(&self.audit_request_sha256).to_owned(),
            option_digest(&self.audit_schedule_sha256).to_owned(),
            option_digest(&self.auditor_binary_sha256).to_owned(),
            option(self.audit_elapsed_ns),
            self.service_floor_disposition.as_str().to_owned(),
            option_digest(&self.service_floor_policy_sha256).to_owned(),
            option_digest(&self.service_floor_schedule_audit_receipt_sha256).to_owned(),
            option_digest(&self.service_floor_receipt_sha256).to_owned(),
            option_digest(&self.service_floor_auditor_binary_sha256).to_owned(),
        ];
        if fields.iter().any(|field| field.contains(',') || field.contains('\n') || field.contains('\r')) {
            return Err(TraceV03Error::InvalidIdentifier);
        }
        Ok(fields.join(","))
    }
}

pub fn parse_trace_csv_v03(input: &str) -> Result<Vec<SolverTraceV03>, TraceV03Error> {
    if input.len() > super::MAX_TRACE_CSV_BYTES {
        return Err(TraceV03Error::InputTooLarge);
    }
    let mut lines = input.lines();
    if lines.next() != Some(TRACE_V03_HEADER) {
        return Err(TraceV03Error::InvalidHeader);
    }
    let mut traces = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        traces.push(parse_trace_row_v03(line)?);
    }
    Ok(traces)
}

pub fn parse_trace_row_v03(line: &str) -> Result<SolverTraceV03, TraceV03Error> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 40 {
        return Err(TraceV03Error::InvalidFieldCount);
    }
    let parse_u64 = |value: &str| value.parse::<u64>().map_err(|_| TraceV03Error::InvalidNumber);
    let parse_u32 = |value: &str| value.parse::<u32>().map_err(|_| TraceV03Error::InvalidNumber);
    let parse_u16 = |value: &str| value.parse::<u16>().map_err(|_| TraceV03Error::InvalidNumber);
    let parse_option_u64 = |value: &str| if value == "NA" { Ok(None) } else { parse_u64(value).map(Some) };
    let parse_option_u32 = |value: &str| if value == "NA" { Ok(None) } else { parse_u32(value).map(Some) };
    let parse_option_string = |value: &str| if value == "NA" { Ok(None) } else { Ok(Some(value.to_owned())) };
    let parse_bool = |value: &str| match value { "true" => Ok(true), "false" => Ok(false), _ => Err(TraceV03Error::InvalidBoolean) };

    let measurement_class = match fields[1] {
        "synthetic" => MeasurementClass::Synthetic,
        "measured" => MeasurementClass::Measured,
        _ => return Err(TraceV03Error::InvalidIdentifier),
    };
    let trace = SolverTraceV03 {
        schema_version: fields[0].to_owned(),
        measurement_class,
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
        search_disposition: SearchDispositionV03::parse(fields[17])?,
        elapsed_ns: parse_u64(fields[18])?,
        time_to_first_candidate_ns: parse_option_u64(fields[19])?,
        time_to_first_audit_accept_ns: parse_option_u64(fields[20])?,
        peak_memory_bytes: parse_option_u64(fields[21])?,
        timeout_limit_ns: parse_option_u64(fields[22])?,
        enumeration_prefix_visits: parse_option_u64(fields[23])?,
        feasible_complete_schedules: parse_option_u64(fields[24])?,
        pareto_frontier_complete: parse_bool(fields[25])?,
        pareto_frontier_count: parse_option_u32(fields[26])?,
        candidate_complete_horizon: parse_bool(fields[27])?,
        candidate_schedule_sha256: parse_option_string(fields[28])?,
        audit_disposition: AuditDispositionV03::parse(fields[29])?,
        audit_receipt_sha256: parse_option_string(fields[30])?,
        audit_request_sha256: parse_option_string(fields[31])?,
        audit_schedule_sha256: parse_option_string(fields[32])?,
        auditor_binary_sha256: parse_option_string(fields[33])?,
        audit_elapsed_ns: parse_option_u64(fields[34])?,
        service_floor_disposition: ServiceFloorDispositionV03::parse(fields[35])?,
        service_floor_policy_sha256: parse_option_string(fields[36])?,
        service_floor_schedule_audit_receipt_sha256: parse_option_string(fields[37])?,
        service_floor_receipt_sha256: parse_option_string(fields[38])?,
        service_floor_auditor_binary_sha256: parse_option_string(fields[39])?,
    };
    trace.validate()?;
    Ok(trace)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceSummaryV03 {
    pub cohort_id: String,
    pub sample_count: usize,
    pub exact_complete_count: usize,
    pub feasible_unproven_count: usize,
    pub proven_infeasible_count: usize,
    pub limit_without_candidate_count: usize,
    pub timed_out_count: usize,
    pub cancelled_count: usize,
    pub error_count: usize,
    pub candidate_count: usize,
    pub audit_accepted_count: usize,
    pub audit_rejected_count: usize,
    pub audit_unavailable_count: usize,
    pub audit_not_run_count: usize,
    pub audit_attempt_count: usize,
    pub service_floor_passed_count: usize,
    pub service_floor_failed_count: usize,
    pub service_floor_not_evaluated_count: usize,
    pub service_floor_unavailable_count: usize,
    pub decision_ready_count: usize,
    pub first_candidate_sample_count: usize,
    pub first_candidate_p50_ns: Option<u64>,
    pub first_candidate_p95_ns: Option<u64>,
    pub first_audit_accept_sample_count: usize,
    pub first_audit_accept_p50_ns: Option<u64>,
    pub first_audit_accept_p95_ns: Option<u64>,
    pub total_prefix_visits: Option<u64>,
    pub audit_accepted_per_million_prefix_visits: Option<u64>,
    pub decision_ready_per_million_prefix_visits: Option<u64>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CohortV03Key {
    measurement_class: MeasurementClass,
    cohort_id: String,
    scenario_panel_sha256: String,
    model_sha256: String,
    mesh_policy_sha256: String,
    solver_name: String,
    solver_version: String,
    horizon_minutes: u16,
    solver_binary_sha256: String,
    solver_config_sha256: String,
    hardware_profile_sha256: String,
}

pub fn summarize_traces_v03(records: &[SolverTraceV03]) -> Result<Vec<TraceSummaryV03>, TraceV03Error> {
    let mut groups = BTreeMap::<CohortV03Key, Vec<&SolverTraceV03>>::new();
    let mut seen =
        BTreeMap::<CohortV03Key, BTreeMap<String, SolverTraceV03>>::new();
    let mut cohort_identity_by_id = BTreeMap::<String, CohortV03Key>::new();
    for record in records {
        record.validate()?;
        let key = CohortV03Key {
            measurement_class: record.measurement_class,
            cohort_id: record.cohort_id.clone(),
            scenario_panel_sha256: record.scenario_panel_sha256.clone(),
            model_sha256: record.model_sha256.clone(),
            mesh_policy_sha256: record.mesh_policy_sha256.clone(),
            solver_name: record.solver_name.clone(),
            solver_version: record.solver_version.clone(),
            horizon_minutes: record.horizon_minutes,
            solver_binary_sha256: record.solver_binary_sha256.clone(),
            solver_config_sha256: record.solver_config_sha256.clone(),
            hardware_profile_sha256: record.hardware_profile_sha256.clone(),
        };
        if let Some(existing) = cohort_identity_by_id.get(&record.cohort_id) {
            if existing != &key {
                return Err(TraceV03Error::MixedCohortIdentity);
            }
        } else {
            cohort_identity_by_id.insert(record.cohort_id.clone(), key.clone());
        }
        let samples = seen.entry(key.clone()).or_default();
        if let Some(existing) = samples.get(&record.sample_id) {
            if existing != record {
                return Err(TraceV03Error::DuplicateSampleConflict);
            }
            continue;
        }
        samples.insert(record.sample_id.clone(), record.clone());
        groups.entry(key).or_default().push(record);
    }
    let mut summaries = Vec::new();
    for (key, rows) in groups {
        let mut summary = TraceSummaryV03 {
            cohort_id: key.cohort_id,
            sample_count: rows.len(),
            exact_complete_count: 0,
            feasible_unproven_count: 0,
            proven_infeasible_count: 0,
            limit_without_candidate_count: 0,
            timed_out_count: 0,
            cancelled_count: 0,
            error_count: 0,
            candidate_count: 0,
            audit_accepted_count: 0,
            audit_rejected_count: 0,
            audit_unavailable_count: 0,
            audit_not_run_count: 0,
            audit_attempt_count: 0,
            service_floor_passed_count: 0,
            service_floor_failed_count: 0,
            service_floor_not_evaluated_count: 0,
            service_floor_unavailable_count: 0,
            decision_ready_count: 0,
            first_candidate_sample_count: 0,
            first_candidate_p50_ns: None,
            first_candidate_p95_ns: None,
            first_audit_accept_sample_count: 0,
            first_audit_accept_p50_ns: None,
            first_audit_accept_p95_ns: None,
            total_prefix_visits: None,
            audit_accepted_per_million_prefix_visits: None,
            decision_ready_per_million_prefix_visits: None,
        };
        let mut first_candidates = Vec::new();
        let mut first_audits = Vec::new();
        let mut prefix_visits = 0u64;
        let mut all_have_prefix_visits = true;
        for row in &rows {
            match row.search_disposition {
                SearchDispositionV03::ExactComplete => summary.exact_complete_count += 1,
                SearchDispositionV03::FeasibleUnproven => summary.feasible_unproven_count += 1,
                SearchDispositionV03::ProvenInfeasible => summary.proven_infeasible_count += 1,
                SearchDispositionV03::LimitWithoutCandidate => summary.limit_without_candidate_count += 1,
                SearchDispositionV03::TimedOut => summary.timed_out_count += 1,
                SearchDispositionV03::Cancelled => summary.cancelled_count += 1,
                SearchDispositionV03::Error => summary.error_count += 1,
            }
            if row.candidate_available() {
                summary.candidate_count += 1;
            }
            match row.audit_disposition {
                AuditDispositionV03::NotRun => summary.audit_not_run_count += 1,
                AuditDispositionV03::Accepted => {
                    summary.audit_accepted_count += 1;
                    if let Some(value) = row.time_to_first_audit_accept_ns {
                        first_audits.push(value);
                    }
                }
                AuditDispositionV03::Rejected => summary.audit_rejected_count += 1,
                AuditDispositionV03::Unavailable => summary.audit_unavailable_count += 1,
            }
            if row.audit_disposition != AuditDispositionV03::NotRun {
                summary.audit_attempt_count += 1;
            }
            match row.service_floor_disposition {
                ServiceFloorDispositionV03::NotEvaluated => {
                    summary.service_floor_not_evaluated_count += 1;
                }
                ServiceFloorDispositionV03::Passed => {
                    summary.service_floor_passed_count += 1;
                }
                ServiceFloorDispositionV03::Failed => {
                    summary.service_floor_failed_count += 1;
                }
                ServiceFloorDispositionV03::Unavailable => {
                    summary.service_floor_unavailable_count += 1;
                }
            }
            if row.decision_ready() {
                summary.decision_ready_count += 1;
            }
            if let Some(value) = row.time_to_first_candidate_ns {
                first_candidates.push(value);
            }
            if let Some(value) = row.enumeration_prefix_visits {
                prefix_visits = prefix_visits
                    .checked_add(value)
                    .ok_or(TraceV03Error::InvalidNumber)?;
            } else {
                all_have_prefix_visits = false;
            }
        }
        first_candidates.sort_unstable();
        first_audits.sort_unstable();
        summary.first_candidate_sample_count = first_candidates.len();
        summary.first_audit_accept_sample_count = first_audits.len();
        summary.first_candidate_p50_ns = nearest_rank_v03(&first_candidates, 50);
        summary.first_candidate_p95_ns = if first_candidates.len() >= P95_MIN_SAMPLES {
            nearest_rank_v03(&first_candidates, 95)
        } else {
            None
        };
        summary.first_audit_accept_p50_ns = nearest_rank_v03(&first_audits, 50);
        summary.first_audit_accept_p95_ns = if first_audits.len() >= P95_MIN_SAMPLES {
            nearest_rank_v03(&first_audits, 95)
        } else {
            None
        };
        if first_candidates.len() < P50_MIN_SAMPLES {
            summary.first_candidate_p50_ns = None;
        }
        if first_audits.len() < P50_MIN_SAMPLES {
            summary.first_audit_accept_p50_ns = None;
        }
        if all_have_prefix_visits && prefix_visits > 0 {
            summary.total_prefix_visits = Some(prefix_visits);
            summary.audit_accepted_per_million_prefix_visits = Some(
                ((summary.audit_accepted_count as u128 * 1_000_000) / u128::from(prefix_visits))
                    .min(u128::from(u64::MAX)) as u64
            );
            summary.decision_ready_per_million_prefix_visits = Some(
                ((summary.decision_ready_count as u128 * 1_000_000) / u128::from(prefix_visits))
                    .min(u128::from(u64::MAX)) as u64
            );
        }
        summaries.push(summary);
    }
    Ok(summaries)
}

fn nearest_rank_v03(values: &[u64], percentile: usize) -> Option<u64> {
    if values.is_empty() || !(1..=100).contains(&percentile) {
        return None;
    }
    let rank = values.len().saturating_mul(percentile).div_ceil(100).max(1);
    values.get(rank - 1).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: u8) -> String {
        format!("{byte:02x}{}", "00".repeat(31))
    }

    fn base(disposition: SearchDispositionV03) -> SolverTraceV03 {
        let candidate = matches!(
            disposition,
            SearchDispositionV03::ExactComplete | SearchDispositionV03::FeasibleUnproven
        );
        let complete_frontier = matches!(
            disposition,
            SearchDispositionV03::ExactComplete | SearchDispositionV03::ProvenInfeasible
        );
        SolverTraceV03 {
            schema_version: TRACE_V03_SCHEMA.to_owned(),
            measurement_class: MeasurementClass::Synthetic,
            cohort_id: "cohort-1".to_owned(),
            sample_id: "sample-1".to_owned(),
            scenario_id: "scenario-1".to_owned(),
            scenario_panel_sha256: digest(1),
            request_sha256: digest(2),
            model_sha256: digest(3),
            mesh_policy_sha256: digest(4),
            mesh_instance_sha256: digest(5),
            solver_name: "sol-atlas-anytime".to_owned(),
            solver_version: "0.1.0".to_owned(),
            solver_binary_sha256: digest(6),
            solver_config_sha256: digest(7),
            hardware_profile_sha256: digest(8),
            horizon_minutes: 60,
            interval_count: 2,
            search_disposition: disposition,
            elapsed_ns: 100,
            time_to_first_candidate_ns: candidate.then_some(10),
            time_to_first_audit_accept_ns: None,
            peak_memory_bytes: Some(1024),
            timeout_limit_ns: None,
            enumeration_prefix_visits: Some(20),
            feasible_complete_schedules: Some(if candidate { 1 } else { 0 }),
            pareto_frontier_complete: complete_frontier,
            pareto_frontier_count: complete_frontier.then_some(if candidate { 1 } else { 0 }),
            candidate_complete_horizon: candidate,
            candidate_schedule_sha256: candidate.then(|| digest(9)),
            audit_disposition: AuditDispositionV03::NotRun,
            audit_receipt_sha256: None,
            audit_request_sha256: None,
            audit_schedule_sha256: None,
            auditor_binary_sha256: None,
            audit_elapsed_ns: None,
            service_floor_disposition: ServiceFloorDispositionV03::NotEvaluated,
            service_floor_policy_sha256: None,
            service_floor_schedule_audit_receipt_sha256: None,
            service_floor_receipt_sha256: None,
            service_floor_auditor_binary_sha256: None,
        }
    }

    #[test]
    fn exact_complete_and_proven_infeasible_have_distinct_consistent_contracts() {
        let exact = base(SearchDispositionV03::ExactComplete);
        exact.validate().unwrap();
        let infeasible = base(SearchDispositionV03::ProvenInfeasible);
        infeasible.validate().unwrap();
    }

    #[test]
    fn feasible_unproven_cannot_claim_frontier_completeness_or_optimality() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.pareto_frontier_count = Some(1);
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidSearchSemantics));
    }

    #[test]
    fn capped_without_candidate_cannot_claim_feasible_schedules() {
        let mut trace = base(SearchDispositionV03::LimitWithoutCandidate);
        trace.feasible_complete_schedules = Some(1);
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidSearchSemantics));
    }

    #[test]
    fn accepted_audit_must_bind_exact_request_schedule_receipt_and_binary() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.audit_disposition = AuditDispositionV03::Accepted;
        trace.audit_receipt_sha256 = Some(digest(10));
        trace.audit_request_sha256 = Some(digest(2));
        trace.audit_schedule_sha256 = Some(digest(99));
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidAuditBindings));
    }

    #[test]
    fn a_matching_accepted_receipt_and_passed_service_floor_are_decision_ready() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        let schedule = digest(9);
        trace.audit_disposition = AuditDispositionV03::Accepted;
        trace.audit_receipt_sha256 = Some(digest(10));
        trace.audit_request_sha256 = Some(digest(2));
        trace.audit_schedule_sha256 = Some(schedule);
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        trace.time_to_first_audit_accept_ns = Some(95);
        trace.service_floor_disposition = ServiceFloorDispositionV03::Passed;
        trace.service_floor_policy_sha256 = Some(digest(12));
        trace.service_floor_schedule_audit_receipt_sha256 = Some(digest(10));
        trace.service_floor_receipt_sha256 = Some(digest(13));
        trace.service_floor_auditor_binary_sha256 = Some(digest(14));
        trace.validate().unwrap();
        assert!(trace.decision_ready());
    }

    #[test]
    fn passed_service_floor_requires_its_own_receipt_and_evaluator_binary() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.audit_disposition = AuditDispositionV03::Accepted;
        trace.audit_receipt_sha256 = Some(digest(10));
        trace.audit_request_sha256 = Some(digest(2));
        trace.audit_schedule_sha256 = Some(digest(9));
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        trace.service_floor_disposition = ServiceFloorDispositionV03::Passed;
        trace.service_floor_policy_sha256 = Some(digest(12));
        trace.service_floor_schedule_audit_receipt_sha256 = Some(digest(10));
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidServiceFloorBindings));
    }

    #[test]
    fn service_floor_receipt_must_reference_the_exact_schedule_audit_receipt() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.audit_disposition = AuditDispositionV03::Accepted;
        trace.audit_receipt_sha256 = Some(digest(10));
        trace.audit_request_sha256 = Some(digest(2));
        trace.audit_schedule_sha256 = Some(digest(9));
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        trace.service_floor_disposition = ServiceFloorDispositionV03::Passed;
        trace.service_floor_policy_sha256 = Some(digest(12));
        trace.service_floor_schedule_audit_receipt_sha256 = Some(digest(99));
        trace.service_floor_receipt_sha256 = Some(digest(13));
        trace.service_floor_auditor_binary_sha256 = Some(digest(14));
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidServiceFloorBindings));
    }

    #[test]
    fn accepted_audit_without_service_floor_is_not_decision_ready() {
        let mut trace = base(SearchDispositionV03::ExactComplete);
        trace.audit_disposition = AuditDispositionV03::Accepted;
        trace.audit_receipt_sha256 = Some(digest(10));
        trace.audit_request_sha256 = Some(digest(2));
        trace.audit_schedule_sha256 = Some(digest(9));
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        trace.validate().unwrap();
        assert!(!trace.decision_ready());
    }

    #[test]
    fn not_run_audit_cannot_carry_partial_receipt_bindings() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.auditor_binary_sha256 = Some(digest(11));
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidAuditBindings));
    }

    #[test]
    fn unavailable_audit_requires_an_actual_attempt_duration() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.audit_disposition = AuditDispositionV03::Unavailable;
        trace.auditor_binary_sha256 = Some(digest(11));
        assert_eq!(trace.validate(), Err(TraceV03Error::InvalidAuditBindings));
    }

    #[test]
    fn unavailable_audit_does_not_masquerade_as_rejection_or_acceptance() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        trace.audit_disposition = AuditDispositionV03::Unavailable;
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        trace.validate().unwrap();
        assert!(!trace.decision_ready());
    }

    #[test]
    fn csv_render_parse_roundtrip_keeps_unproven_candidate_and_audit_separate() {
        let mut trace = base(SearchDispositionV03::FeasibleUnproven);
        let schedule = digest(9);
        trace.audit_disposition = AuditDispositionV03::Accepted;
        trace.audit_receipt_sha256 = Some(digest(10));
        trace.audit_request_sha256 = Some(digest(2));
        trace.audit_schedule_sha256 = Some(schedule);
        trace.auditor_binary_sha256 = Some(digest(11));
        trace.audit_elapsed_ns = Some(5);
        let row = trace.render_csv_row().unwrap();
        let parsed = parse_trace_row_v03(&row).unwrap();
        assert_eq!(trace, parsed);
    }

    #[test]
    fn summary_preserves_all_search_and_audit_denominators() {
        let mut exact = base(SearchDispositionV03::ExactComplete);
        exact.sample_id = "sample-exact".into();
        let mut unproven = base(SearchDispositionV03::FeasibleUnproven);
        unproven.sample_id = "sample-unproven".into();
        let mut limit = base(SearchDispositionV03::LimitWithoutCandidate);
        limit.sample_id = "sample-limit".into();
        let summary = summarize_traces_v03(&[exact, unproven, limit]).unwrap().remove(0);
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.exact_complete_count, 1);
        assert_eq!(summary.feasible_unproven_count, 1);
        assert_eq!(summary.limit_without_candidate_count, 1);
        assert_eq!(summary.candidate_count, 2);
        assert_eq!(summary.decision_ready_count, 0);
        assert_eq!(summary.total_prefix_visits, Some(60));
        assert_eq!(summary.decision_ready_per_million_prefix_visits, Some(0));
    }

    #[test]
    fn summary_keeps_all_service_floor_and_audit_attempt_denominators() {
        let mut rejected = base(SearchDispositionV03::FeasibleUnproven);
        rejected.sample_id = "sample-rejected".into();
        rejected.audit_disposition = AuditDispositionV03::Rejected;
        rejected.audit_receipt_sha256 = Some(digest(20));
        rejected.audit_request_sha256 = Some(digest(2));
        rejected.audit_schedule_sha256 = Some(digest(9));
        rejected.auditor_binary_sha256 = Some(digest(11));
        rejected.audit_elapsed_ns = Some(5);
        rejected.service_floor_disposition = ServiceFloorDispositionV03::Failed;
        rejected.service_floor_policy_sha256 = Some(digest(12));

        let mut unavailable = base(SearchDispositionV03::FeasibleUnproven);
        unavailable.sample_id = "sample-unavailable".into();
        unavailable.audit_disposition = AuditDispositionV03::Unavailable;
        unavailable.auditor_binary_sha256 = Some(digest(11));
        unavailable.audit_elapsed_ns = Some(5);
        unavailable.service_floor_disposition = ServiceFloorDispositionV03::Unavailable;
        unavailable.service_floor_policy_sha256 = Some(digest(13));

        let summary = summarize_traces_v03(&[rejected, unavailable]).unwrap().remove(0);
        assert_eq!(summary.audit_attempt_count, 2);
        assert_eq!(summary.audit_rejected_count, 1);
        assert_eq!(summary.audit_unavailable_count, 1);
        assert_eq!(summary.service_floor_failed_count, 1);
        assert_eq!(summary.service_floor_unavailable_count, 1);
        assert_eq!(summary.service_floor_not_evaluated_count, 0);
        assert_eq!(summary.decision_ready_count, 0);
    }

    #[test]
    fn csv_parser_rejects_input_above_declared_byte_limit() {
        let oversized = "x".repeat(super::super::MAX_TRACE_CSV_BYTES + 1);
        assert_eq!(
            parse_trace_csv_v03(&oversized),
            Err(TraceV03Error::InputTooLarge)
        );
    }

    #[test]
    fn reused_cohort_id_with_changed_solver_identity_is_rejected() {
        let first = base(SearchDispositionV03::ExactComplete);
        let mut second = base(SearchDispositionV03::ExactComplete);
        second.sample_id = "sample-other".into();
        second.solver_binary_sha256 = digest(99);
        assert_eq!(
            summarize_traces_v03(&[first, second]),
            Err(TraceV03Error::MixedCohortIdentity)
        );
    }

    #[test]
    fn duplicate_sample_id_with_conflicting_outcome_is_rejected() {
        let first = base(SearchDispositionV03::ExactComplete);
        let mut conflict = first.clone();
        conflict.search_disposition = SearchDispositionV03::LimitWithoutCandidate;
        conflict.time_to_first_candidate_ns = None;
        conflict.candidate_complete_horizon = false;
        conflict.candidate_schedule_sha256 = None;
        conflict.feasible_complete_schedules = Some(0);
        conflict.pareto_frontier_complete = false;
        conflict.pareto_frontier_count = None;
        assert_eq!(
            summarize_traces_v03(&[first, conflict]),
            Err(TraceV03Error::DuplicateSampleConflict)
        );
    }

    #[test]
    fn duplicate_sample_ids_are_suppressed_within_a_cohort() {
        let first = base(SearchDispositionV03::ExactComplete);
        let duplicate = base(SearchDispositionV03::LimitWithoutCandidate);
        let summary = summarize_traces_v03(&[first, duplicate]).unwrap().remove(0);
        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.exact_complete_count, 1);
    }
}
