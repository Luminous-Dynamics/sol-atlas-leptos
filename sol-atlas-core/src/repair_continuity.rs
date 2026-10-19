// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic regional repair-workcell continuity comparison.
//!
//! This module compares modeled feasibility against matched demand. It consumes
//! repair demand, inventory snapshots, qualification records and dependency states
//! owned by other systems. It does not own work orders, inventory truth, part-fit
//! decisions, routing, calibration authority or verified restoration records.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const REPAIR_CONTINUITY_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_WINDOWS: usize = 64;
pub const MAX_DEMAND: usize = 10_000;
pub const MAX_CELLS_PER_CANDIDATE: usize = 128;
pub const MAX_STOCK_LOTS_PER_CANDIDATE: usize = 10_000;
pub const MAX_FIT_EVALUATIONS: usize = 2_000_000;
pub const MAX_STOCK_INSPECTIONS: usize = 20_000_000;
const EPSILON: f64 = 1.0e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    Observed,
    Curated,
    Assumed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceMetadata {
    pub class: EvidenceClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Completeness {
    Complete,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Unavailable,
    Unknown,
    Derated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyObservation {
    pub dependency_id: String,
    pub window_index: usize,
    pub availability: Availability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_fraction: Option<f64>,
    pub evidence: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceWindow {
    pub id: String,
    pub duration_hours: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartRequirement {
    /// Request-side key from the canonical parts-compatibility owner.
    pub part_key: String,
    pub required_revision: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairDemand {
    /// Stable demand ID from the canonical work-order/repair owner.
    pub request_id: String,
    pub capability_id: String,
    #[serde(default)]
    pub required_skill_ids: Vec<String>,
    #[serde(default)]
    pub required_tool_capabilities: Vec<String>,
    #[serde(default)]
    pub required_instrument_ids: Vec<String>,
    #[serde(default)]
    pub required_parts: Vec<PartRequirement>,
    pub service_hours: f64,
    pub arrival_window: usize,
    pub due_window: usize,
    pub max_turnaround_hours: f64,
    pub priority: u16,
    /// V1 deliberately limits this evaluator to modest low-voltage repairs.
    pub scope_profile: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartFit {
    ExactRevision,
    ApprovedSubstitute,
    Unknown,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockLot {
    pub stock_id: String,
    pub part_key: String,
    pub revision: String,
    pub quantity: u32,
    pub fit: PartFit,
    #[serde(default)]
    pub approved_for_part_keys: Vec<String>,
    pub available_from_window: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after_window: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supply_dependency_id: Option<String>,
    pub evidence: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryPool {
    pub pool_id: String,
    pub location_id: String,
    pub lots: Vec<StockLot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryAccess {
    pub pool_id: String,
    pub route_dependency_id: String,
    pub transfer_hours: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationState {
    Qualified,
    Expired,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instrument {
    pub instrument_id: String,
    pub calibration_state: CalibrationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibrated_through_window: Option<usize>,
    pub evidence: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkCell {
    pub cell_id: String,
    pub node_id: String,
    pub capability_ids: Vec<String>,
    pub qualified_skill_ids: Vec<String>,
    pub tool_capabilities: Vec<String>,
    pub readiness_completeness: Completeness,
    pub readiness_evidence: EvidenceMetadata,
    #[serde(default)]
    pub instruments: Vec<Instrument>,
    /// Capacity is per window. A request must fit entirely within one window in v1.
    pub service_hours_by_window: Vec<f64>,
    pub travel_hours_to_service: f64,
    pub staffing_dependency_id: String,
    pub power_dependency_id: String,
    pub service_route_dependency_id: String,
    #[serde(default)]
    pub inventory_access: Vec<InventoryAccess>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    CentralizedDepot,
    LocalServiceCell,
    DistributedSharedReserve,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureCandidate {
    pub candidate_id: String,
    pub architecture: Architecture,
    pub workcells: Vec<WorkCell>,
    pub inventory_pools: Vec<InventoryPool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairContinuityRequest {
    pub schema_version: String,
    pub comparison_id: String,
    pub windows: Vec<ServiceWindow>,
    /// Every architecture is assessed against this same set of requests.
    pub demand: Vec<RepairDemand>,
    /// One shared disruption schedule is applied to all architecture candidates.
    pub dependency_observations: Vec<DependencyObservation>,
    pub candidates: Vec<ArchitectureCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestOutcome {
    ScheduledWithinWindow,
    Indeterminate,
    UnservedWithinWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Blocker {
    NoQualifiedCell,
    CapabilityEvidenceIncomplete,
    MissingSkill,
    MissingTool,
    MissingInstrument,
    CalibrationExpired,
    CalibrationUnknown,
    StaffingUnavailable,
    StaffingUnknown,
    PowerUnavailable,
    PowerUnknown,
    ServiceRouteUnavailable,
    ServiceRouteUnknown,
    NoCompatiblePart,
    PartFitUnknown,
    PartSupplyUnavailable,
    PartSupplyUnknown,
    InventoryRouteUnavailable,
    InventoryRouteUnknown,
    CapacityUnavailable,
    ResponseWindowExceeded,
    StockEvidenceUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockConsumption {
    pub stock_id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestAssessment {
    pub request_id: String,
    pub capability_id: String,
    pub outcome: RequestOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_cell_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_window: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modelled_turnaround_hours: Option<f64>,
    pub blockers: Vec<Blocker>,
    pub stock_consumption: Vec<StockConsumption>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub capability_id: String,
    pub demand_count: usize,
    pub scheduled_count: usize,
    pub indeterminate_count: usize,
    pub unserved_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapacityLedgerRow {
    pub cell_id: String,
    pub window_index: usize,
    pub nominal_service_hours: f64,
    /// None means at least one necessary resource state is unknown, not zero.
    pub effective_capacity_hours: Option<f64>,
    pub allocated_service_hours: Option<f64>,
    pub remaining_service_hours: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateAssessment {
    pub candidate_id: String,
    pub architecture: Architecture,
    pub demand_count: usize,
    pub scheduled_count: usize,
    pub indeterminate_count: usize,
    pub unserved_count: usize,
    /// Confirmed modeled assignments divided by all requests. Indeterminate
    /// requests remain in the denominator and are never counted as successes.
    pub scheduled_share_lower_bound: f64,
    pub turnaround_p50_hours: Option<f64>,
    pub turnaround_p90_hours: Option<f64>,
    pub by_capability: Vec<CapabilitySummary>,
    pub requests: Vec<RequestAssessment>,
    pub capacity_ledger: Vec<CapacityLedgerRow>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairContinuityAssessment {
    pub schema_version: String,
    pub comparison_id: String,
    pub window_count: usize,
    pub demand_count: usize,
    pub assessments: Vec<CandidateAssessment>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairContinuityError {
    InvalidInput(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResourceFactor {
    Available(f64),
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone)]
struct Assignment {
    cell_id: String,
    window_index: usize,
    turnaround_hours: f64,
    stock_consumption: Vec<StockConsumption>,
    blockers: BTreeSet<Blocker>,
}

#[derive(Debug, Clone)]
enum Fit {
    Feasible(Assignment),
    Indeterminate(BTreeSet<Blocker>),
    Infeasible(BTreeSet<Blocker>),
}

fn validate_evidence(value: &EvidenceMetadata, label: &str, errors: &mut Vec<String>) {
    let has_text = |value: &Option<String>| value.as_ref().is_some_and(|value| !value.trim().is_empty());
    match value.class {
        EvidenceClass::Observed => {
            if !has_text(&value.source_id) || !has_text(&value.source_uri) {
                errors.push(format!("{label}: observed evidence requires source_id and source_uri"));
            }
            if !value.snapshot_date.as_deref().is_some_and(is_date_shape) {
                errors.push(format!("{label}: observed evidence requires YYYY-MM-DD snapshot_date"));
            }
            if !value.artifact_sha256.as_deref().is_some_and(is_sha256) {
                errors.push(format!("{label}: observed evidence requires a 64-character SHA-256"));
            }
        }
        EvidenceClass::Curated => {
            if !has_text(&value.source_id) || !has_text(&value.source_uri) {
                errors.push(format!("{label}: curated evidence requires source_id and source_uri"));
            }
        }
        EvidenceClass::Assumed | EvidenceClass::Unknown => {}
    }
}

fn is_date_shape(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(index, byte)| {
            index == 4 || index == 7 || byte.is_ascii_digit()
        })
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_ids(values: &[String], label: &str, errors: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            errors.push(format!("{label} contains an empty identifier"));
        } else if !seen.insert(value.as_str()) {
            errors.push(format!("{label} contains duplicate identifier {value:?}"));
        }
    }
}

fn validate(request: &RepairContinuityRequest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if request.schema_version != REPAIR_CONTINUITY_SCHEMA_VERSION {
        errors.push("unsupported schema_version".to_owned());
    }
    if request.comparison_id.trim().is_empty() {
        errors.push("comparison_id cannot be empty".to_owned());
    }
    if request.windows.is_empty() || request.windows.len() > MAX_WINDOWS {
        errors.push(format!("window count must be in [1, {MAX_WINDOWS}]"));
    }
    if request.demand.is_empty() || request.demand.len() > MAX_DEMAND {
        errors.push(format!("demand count must be in [1, {MAX_DEMAND}]"));
    }
    if request.candidates.len() != 3 {
        errors.push("exactly three matched architecture candidates are required".to_owned());
    }

    let mut window_ids = BTreeSet::new();
    for (index, window) in request.windows.iter().enumerate() {
        if window.id.trim().is_empty() || !window_ids.insert(window.id.as_str()) {
            errors.push(format!("windows[{index}] has empty or duplicate ID"));
        }
        if !window.duration_hours.is_finite() || window.duration_hours <= 0.0 {
            errors.push(format!("windows[{index}] duration must be finite and positive"));
        }
    }

    let mut request_ids = BTreeSet::new();
    for (index, demand) in request.demand.iter().enumerate() {
        if demand.request_id.trim().is_empty() || !request_ids.insert(demand.request_id.as_str()) {
            errors.push(format!("demand[{index}] has empty or duplicate request_id"));
        }
        if demand.capability_id.trim().is_empty() {
            errors.push(format!("demand[{index}] capability_id cannot be empty"));
        }
        if !demand.service_hours.is_finite() || demand.service_hours <= 0.0 {
            errors.push(format!("demand[{index}] service_hours must be finite and positive"));
        }
        if !demand.max_turnaround_hours.is_finite() || demand.max_turnaround_hours <= 0.0 {
            errors.push(format!("demand[{index}] max_turnaround_hours must be finite and positive"));
        }
        if demand.arrival_window > demand.due_window || demand.due_window >= request.windows.len() {
            errors.push(format!("demand[{index}] has invalid arrival/due windows"));
        }
        if !matches!(
            demand.scope_profile.as_str(),
            "low_voltage_fan" | "sensor" | "small_pump" | "simple_controller_board"
        ) {
            errors.push(format!("demand[{index}] is outside the guarded low-voltage v1 profile"));
        }
        validate_ids(&demand.required_skill_ids, "required_skill_ids", &mut errors);
        validate_ids(&demand.required_tool_capabilities, "required_tool_capabilities", &mut errors);
        validate_ids(&demand.required_instrument_ids, "required_instrument_ids", &mut errors);
        let mut part_keys = BTreeSet::new();
        for part in &demand.required_parts {
            if part.part_key.trim().is_empty() || part.required_revision.trim().is_empty() || part.quantity == 0 {
                errors.push(format!("demand[{index}] has an invalid part requirement"));
            }
            if !part_keys.insert(part.part_key.as_str()) {
                errors.push(format!("demand[{index}] repeats part key {:?}", part.part_key));
            }
        }
    }

    let mut dependency_keys = BTreeSet::new();
    for (index, observation) in request.dependency_observations.iter().enumerate() {
        if observation.dependency_id.trim().is_empty() || observation.window_index >= request.windows.len() {
            errors.push(format!("dependency_observations[{index}] has invalid ID/window"));
        }
        if !dependency_keys.insert((observation.dependency_id.as_str(), observation.window_index)) {
            errors.push(format!("dependency_observations[{index}] duplicates ID/window"));
        }
        match observation.availability {
            Availability::Derated => {
                if !observation.available_fraction.is_some_and(|value| value.is_finite() && value > 0.0 && value <= 1.0) {
                    errors.push(format!("dependency_observations[{index}] derated fraction must be in (0, 1]"));
                }
            }
            _ if observation.available_fraction.is_some() => errors.push(format!(
                "dependency_observations[{index}] non-derated state must omit available_fraction"
            )),
            _ => {}
        }
        validate_evidence(&observation.evidence, "dependency evidence", &mut errors);
    }

    let expected_architectures: BTreeSet<Architecture> = [
        Architecture::CentralizedDepot,
        Architecture::LocalServiceCell,
        Architecture::DistributedSharedReserve,
    ].into_iter().collect();
    let actual_architectures: BTreeSet<Architecture> = request.candidates.iter().map(|item| item.architecture).collect();
    if actual_architectures != expected_architectures {
        errors.push("architectures must include centralized_depot, local_service_cell and distributed_shared_reserve exactly once".to_owned());
    }
    let mut candidate_ids = BTreeSet::new();
    let mut fit_evaluations = 0usize;
    let mut stock_inspection_estimate = 0usize;
    for (candidate_index, candidate) in request.candidates.iter().enumerate() {
        let candidate_evaluations = request
            .demand
            .len()
            .checked_mul(candidate.workcells.len())
            .and_then(|value| value.checked_mul(request.windows.len()));
        match candidate_evaluations.and_then(|value| fit_evaluations.checked_add(value)) {
            Some(total) => fit_evaluations = total,
            None => errors.push("repair evaluation budget arithmetic overflowed".to_owned()),
        }
        if candidate.candidate_id.trim().is_empty() || !candidate_ids.insert(candidate.candidate_id.as_str()) {
            errors.push(format!("candidates[{candidate_index}] has empty or duplicate candidate_id"));
        }
        if candidate.workcells.is_empty() || candidate.workcells.len() > MAX_CELLS_PER_CANDIDATE {
            errors.push(format!("candidates[{candidate_index}] workcell count is outside the allowed range"));
        }
        let mut pool_ids = BTreeSet::new();
        let mut stock_ids = BTreeSet::new();
        let mut stock_lot_count = 0usize;
        for pool in &candidate.inventory_pools {
            if pool.pool_id.trim().is_empty() || !pool_ids.insert(pool.pool_id.as_str()) {
                errors.push(format!("candidate {:?} has empty or duplicate pool ID", candidate.candidate_id));
            }
            if pool.location_id.trim().is_empty() {
                errors.push(format!("inventory pool {:?} needs location_id", pool.pool_id));
            }
            for lot in &pool.lots {
                stock_lot_count = stock_lot_count.saturating_add(1);
                if lot.stock_id.trim().is_empty() || !stock_ids.insert(lot.stock_id.as_str()) {
                    errors.push(format!("candidate {:?} has empty or duplicate stock ID", candidate.candidate_id));
                }
                if lot.part_key.trim().is_empty() || lot.revision.trim().is_empty() {
                    errors.push(format!("stock lot {:?} needs part_key and revision", lot.stock_id));
                }
                if lot.quantity == 0 || lot.available_from_window >= request.windows.len() {
                    errors.push(format!("stock lot {:?} quantity or availability window is invalid", lot.stock_id));
                }
                if lot.expires_after_window.is_some_and(|end| end < lot.available_from_window || end >= request.windows.len()) {
                    errors.push(format!("stock lot {:?} expiry window is invalid", lot.stock_id));
                }
                if lot.available_from_window > 0 && lot.supply_dependency_id.as_deref().unwrap_or("").trim().is_empty() {
                    errors.push(format!("future stock lot {:?} requires supply_dependency_id", lot.stock_id));
                }
                validate_ids(&lot.approved_for_part_keys, "approved_for_part_keys", &mut errors);
                validate_evidence(&lot.evidence, "stock evidence", &mut errors);
            }
        }
        if stock_lot_count > MAX_STOCK_LOTS_PER_CANDIDATE {
            errors.push(format!(
                "candidate {:?} exceeds stock-lot budget of {MAX_STOCK_LOTS_PER_CANDIDATE}",
                candidate.candidate_id
            ));
        }
        match candidate_evaluations
            .and_then(|evaluations| evaluations.checked_mul(stock_lot_count.max(1)))
            .and_then(|inspections| stock_inspection_estimate.checked_add(inspections))
        {
            Some(total) => stock_inspection_estimate = total,
            None => errors.push("repair stock-inspection budget arithmetic overflowed".to_owned()),
        }
        let mut cell_ids = BTreeSet::new();
        for cell in &candidate.workcells {
            if cell.cell_id.trim().is_empty() || !cell_ids.insert(cell.cell_id.as_str()) {
                errors.push(format!(
                    "candidate {:?} has empty or duplicate workcell ID {:?}",
                    candidate.candidate_id, cell.cell_id
                ));
            }
            if cell.cell_id.trim().is_empty() || cell.node_id.trim().is_empty() {
                errors.push(format!("candidate {:?} has a workcell with empty ID", candidate.candidate_id));
            }
            validate_ids(&cell.capability_ids, "capability_ids", &mut errors);
            validate_ids(&cell.qualified_skill_ids, "qualified_skill_ids", &mut errors);
            validate_ids(&cell.tool_capabilities, "tool_capabilities", &mut errors);
            if cell.service_hours_by_window.len() != request.windows.len()
                || cell.service_hours_by_window.iter().any(|hours| !hours.is_finite() || *hours < 0.0)
            {
                errors.push(format!("workcell {:?} capacity must be finite, non-negative and specified for every window", cell.cell_id));
            }
            if !cell.travel_hours_to_service.is_finite() || cell.travel_hours_to_service < 0.0 {
                errors.push(format!("workcell {:?} travel_hours must be finite and non-negative", cell.cell_id));
            }
            for dependency in [&cell.staffing_dependency_id, &cell.power_dependency_id, &cell.service_route_dependency_id] {
                if dependency.trim().is_empty() {
                    errors.push(format!("workcell {:?} has an empty dependency ID", cell.cell_id));
                }
            }
            validate_evidence(&cell.readiness_evidence, "workcell readiness evidence", &mut errors);
            let mut instrument_ids = BTreeSet::new();
            for instrument in &cell.instruments {
                if instrument.instrument_id.trim().is_empty() || !instrument_ids.insert(instrument.instrument_id.as_str()) {
                    errors.push(format!("workcell {:?} has empty or duplicate instrument ID", cell.cell_id));
                }
                if instrument.calibrated_through_window.is_some_and(|end| end >= request.windows.len()) {
                    errors.push(format!("instrument {:?} has an out-of-range calibration window", instrument.instrument_id));
                }
                validate_evidence(&instrument.evidence, "instrument evidence", &mut errors);
            }
            let mut access_pools = BTreeSet::new();
            for access in &cell.inventory_access {
                if !pool_ids.contains(access.pool_id.as_str()) {
                    errors.push(format!("workcell {:?} references unknown inventory pool {:?}", cell.cell_id, access.pool_id));
                }
                if !access_pools.insert(access.pool_id.as_str()) {
                    errors.push(format!("workcell {:?} repeats inventory access {:?}", cell.cell_id, access.pool_id));
                }
                if access.route_dependency_id.trim().is_empty() || !access.transfer_hours.is_finite() || access.transfer_hours < 0.0 {
                    errors.push(format!("workcell {:?} has invalid inventory access", cell.cell_id));
                }
            }
        }
    }

    if fit_evaluations > MAX_FIT_EVALUATIONS {
        errors.push(format!(
            "repair fit evaluation budget exceeded: {fit_evaluations} > {MAX_FIT_EVALUATIONS}"
        ));
    }
    if stock_inspection_estimate > MAX_STOCK_INSPECTIONS {
        errors.push(format!(
            "repair stock-inspection budget exceeded: {stock_inspection_estimate} > {MAX_STOCK_INSPECTIONS}"
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

fn dependency_factor(
    observations: &[DependencyObservation],
    dependency_id: &str,
    window: usize,
) -> ResourceFactor {
    let Some(observation) = observations.iter().find(|value| value.dependency_id == dependency_id && value.window_index == window) else {
        return ResourceFactor::Unknown;
    };
    // An explicit Unknown evidence grade cannot establish resource availability.
    // Assumed and curated states remain explicit inputs to the synthetic model.
    if observation.evidence.class == EvidenceClass::Unknown {
        return ResourceFactor::Unknown;
    }
    match observation.availability {
        Availability::Available => ResourceFactor::Available(1.0),
        Availability::Unavailable => ResourceFactor::Unavailable,
        Availability::Unknown => ResourceFactor::Unknown,
        Availability::Derated => observation
            .available_fraction
            .filter(|fraction| fraction.is_finite() && *fraction > 0.0 && *fraction <= 1.0)
            .map(ResourceFactor::Available)
            .unwrap_or(ResourceFactor::Unknown),
    }
}

fn initial_stock(candidate: &ArchitectureCandidate, observations: &[DependencyObservation]) -> BTreeMap<String, u32> {
    let mut remaining = BTreeMap::new();
    for pool in &candidate.inventory_pools {
        for lot in &pool.lots {
            let quantity = if lot.evidence.class == EvidenceClass::Unknown {
                0
            } else if let Some(dependency) = &lot.supply_dependency_id {
                match dependency_factor(observations, dependency, lot.available_from_window) {
                    ResourceFactor::Available(fraction) => ((f64::from(lot.quantity) * fraction).floor()) as u32,
                    ResourceFactor::Unavailable | ResourceFactor::Unknown => 0,
                }
            } else {
                lot.quantity
            };
            remaining.insert(lot.stock_id.clone(), quantity);
        }
    }
    remaining
}

fn check_parts(
    candidate: &ArchitectureCandidate,
    cell: &WorkCell,
    demand: &RepairDemand,
    window: usize,
    remaining: &BTreeMap<String, u32>,
    observations: &[DependencyObservation],
) -> Result<(Vec<StockConsumption>, f64), BTreeSet<Blocker>> {
    let mut tentative = remaining.clone();
    let mut used_pool_transfer = BTreeMap::<String, f64>::new();
    let mut consumption = BTreeMap::<String, u32>::new();
    let access_by_pool: BTreeMap<&str, &InventoryAccess> = cell.inventory_access.iter()
        .map(|access| (access.pool_id.as_str(), access))
        .collect();

    for requirement in &demand.required_parts {
        let mut known = Vec::<(u8, String, String, u32, f64)>::new();
        let mut possible_unknown = 0_u64;
        let mut route_unknown = false;

        for pool in &candidate.inventory_pools {
            let Some(access) = access_by_pool.get(pool.pool_id.as_str()) else { continue };
            let transfer = match dependency_factor(observations, &access.route_dependency_id, window) {
                ResourceFactor::Available(factor) => access.transfer_hours / factor,
                ResourceFactor::Unavailable => continue,
                ResourceFactor::Unknown => {
                    route_unknown = true;
                    continue;
                }
            };
            for lot in &pool.lots {
                if window < lot.available_from_window || lot.expires_after_window.is_some_and(|end| window > end) {
                    continue;
                }
                let may_match = lot.part_key == requirement.part_key
                    || lot.approved_for_part_keys.iter().any(|value| value == &requirement.part_key);
                if !may_match { continue; }
                let qty = tentative.get(&lot.stock_id).copied().unwrap_or(0);
                if qty == 0 {
                    if lot.quantity > 0 && (lot.evidence.class == EvidenceClass::Unknown || lot.fit == PartFit::Unknown
                        || lot.supply_dependency_id.as_ref().is_some_and(|id| matches!(dependency_factor(observations, id, lot.available_from_window), ResourceFactor::Unknown))) {
                        possible_unknown += u64::from(lot.quantity);
                    }
                    continue;
                }
                match lot.fit {
                    PartFit::ExactRevision if lot.part_key == requirement.part_key && lot.revision == requirement.required_revision => {
                        if lot.evidence.class == EvidenceClass::Unknown { possible_unknown += u64::from(qty); }
                        else { known.push((0, pool.pool_id.clone(), lot.stock_id.clone(), qty, transfer)); }
                    }
                    PartFit::ApprovedSubstitute if lot.approved_for_part_keys.iter().any(|value| value == &requirement.part_key) => {
                        if lot.evidence.class == EvidenceClass::Unknown { possible_unknown += u64::from(qty); }
                        else { known.push((1, pool.pool_id.clone(), lot.stock_id.clone(), qty, transfer)); }
                    }
                    PartFit::Unknown => possible_unknown += u64::from(qty),
                    PartFit::ExactRevision | PartFit::ApprovedSubstitute | PartFit::Incompatible => {}
                }
            }
        }

        known.sort_by(|left, right| (left.0, &left.1, &left.2).cmp(&(right.0, &right.1, &right.2)));
        let mut needed = requirement.quantity;
        for (_fit_rank, pool_id, stock_id, available, transfer) in known {
            if needed == 0 { break; }
            let take = needed.min(available);
            needed -= take;
            tentative.insert(stock_id.clone(), tentative.get(&stock_id).copied().unwrap_or(0) - take);
            *consumption.entry(stock_id).or_default() += take;
            used_pool_transfer.entry(pool_id).and_modify(|old| *old = old.max(transfer)).or_insert(transfer);
        }
        if needed > 0 {
            if possible_unknown >= u64::from(needed) || route_unknown {
                return Err(BTreeSet::from([Blocker::PartFitUnknown]));
            }
            return Err(BTreeSet::from([Blocker::NoCompatiblePart]));
        }
    }
    let transfer_hours = used_pool_transfer.values().copied().fold(0.0_f64, f64::max);
    let stock_consumption = consumption.into_iter().map(|(stock_id, quantity)| StockConsumption { stock_id, quantity }).collect();
    Ok((stock_consumption, transfer_hours))
}

fn check_cell(
    request: &RepairContinuityRequest,
    candidate: &ArchitectureCandidate,
    cell: &WorkCell,
    demand: &RepairDemand,
    window: usize,
    remaining_stock: &BTreeMap<String, u32>,
) -> Fit {
    let mut blockers = BTreeSet::new();
    let mut uncertain = cell.readiness_evidence.class == EvidenceClass::Unknown;
    if uncertain {
        blockers.insert(Blocker::CapabilityEvidenceIncomplete);
    }
    let mut impossible = false;

    if !cell.capability_ids.contains(&demand.capability_id) {
        if cell.readiness_completeness == Completeness::Complete {
            blockers.insert(Blocker::NoQualifiedCell); impossible = true;
        } else {
            blockers.insert(Blocker::CapabilityEvidenceIncomplete); uncertain = true;
        }
    }
    for required in &demand.required_skill_ids {
        if !cell.qualified_skill_ids.contains(required) {
            if cell.readiness_completeness == Completeness::Complete {
                blockers.insert(Blocker::MissingSkill); impossible = true;
            } else {
                blockers.insert(Blocker::CapabilityEvidenceIncomplete); uncertain = true;
            }
        }
    }
    for required in &demand.required_tool_capabilities {
        if !cell.tool_capabilities.contains(required) {
            if cell.readiness_completeness == Completeness::Complete {
                blockers.insert(Blocker::MissingTool); impossible = true;
            } else {
                blockers.insert(Blocker::CapabilityEvidenceIncomplete); uncertain = true;
            }
        }
    }
    for required in &demand.required_instrument_ids {
        match cell.instruments.iter().find(|item| item.instrument_id == *required) {
            None if cell.readiness_completeness == Completeness::Complete => {
                blockers.insert(Blocker::MissingInstrument); impossible = true;
            }
            None => { blockers.insert(Blocker::CalibrationUnknown); uncertain = true; }
            Some(item) if item.evidence.class == EvidenceClass::Unknown || item.calibration_state == CalibrationState::Unknown => {
                blockers.insert(Blocker::CalibrationUnknown); uncertain = true;
            }
            Some(item) if item.calibration_state == CalibrationState::Qualified
                && item.calibrated_through_window.is_some_and(|end| window <= end) => {}
            Some(_) => { blockers.insert(Blocker::CalibrationExpired); impossible = true; }
        }
    }
    if impossible {
        return Fit::Infeasible(blockers);
    }

    let staff = dependency_factor(&request.dependency_observations, &cell.staffing_dependency_id, window);
    let power = dependency_factor(&request.dependency_observations, &cell.power_dependency_id, window);
    let route = dependency_factor(&request.dependency_observations, &cell.service_route_dependency_id, window);
    let staff_factor = match staff {
        ResourceFactor::Available(value) => Some(value),
        ResourceFactor::Unavailable => { blockers.insert(Blocker::StaffingUnavailable); return Fit::Infeasible(blockers); }
        ResourceFactor::Unknown => { blockers.insert(Blocker::StaffingUnknown); uncertain = true; None }
    };
    let power_factor = match power {
        ResourceFactor::Available(value) => Some(value),
        ResourceFactor::Unavailable => { blockers.insert(Blocker::PowerUnavailable); return Fit::Infeasible(blockers); }
        ResourceFactor::Unknown => { blockers.insert(Blocker::PowerUnknown); uncertain = true; None }
    };
    let route_factor = match route {
        ResourceFactor::Available(value) => Some(value),
        ResourceFactor::Unavailable => { blockers.insert(Blocker::ServiceRouteUnavailable); return Fit::Infeasible(blockers); }
        ResourceFactor::Unknown => { blockers.insert(Blocker::ServiceRouteUnknown); uncertain = true; None }
    };

    let nominal_hours = cell.service_hours_by_window[window];
    if nominal_hours + EPSILON < demand.service_hours {
        blockers.insert(Blocker::CapacityUnavailable);
        return Fit::Infeasible(blockers);
    }
    if staff_factor.is_none() || power_factor.is_none() || route_factor.is_none() {
        return Fit::Indeterminate(blockers);
    }
    let effective_hours = nominal_hours * staff_factor.unwrap().min(power_factor.unwrap());
    if effective_hours + EPSILON < demand.service_hours {
        blockers.insert(Blocker::CapacityUnavailable);
        return Fit::Infeasible(blockers);
    }

    let (stock_consumption, transfer_hours) = match check_parts(
        candidate, cell, demand, window, remaining_stock, &request.dependency_observations,
    ) {
        Ok(result) => result,
        Err(reasons) => {
            blockers.extend(reasons.iter().copied());
            if reasons.contains(&Blocker::PartFitUnknown) {
                return Fit::Indeterminate(blockers);
            }
            return Fit::Infeasible(blockers);
        }
    };

    let waiting_hours = request.windows[demand.arrival_window..window].iter()
        .map(|item| item.duration_hours).sum::<f64>();
    let travel_hours = cell.travel_hours_to_service / route_factor.unwrap();
    let turnaround = waiting_hours + travel_hours + transfer_hours + demand.service_hours;
    if !turnaround.is_finite() || turnaround > demand.max_turnaround_hours + EPSILON {
        blockers.insert(Blocker::ResponseWindowExceeded);
        return Fit::Infeasible(blockers);
    }
    if uncertain {
        Fit::Indeterminate(blockers)
    } else {
        Fit::Feasible(Assignment {
            cell_id: cell.cell_id.clone(),
            window_index: window,
            turnaround_hours: turnaround,
            stock_consumption,
            blockers,
        })
    }
}

fn percentile(values: &[f64], p: usize) -> Option<f64> {
    if values.is_empty() { return None; }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = (sorted.len() * p).div_ceil(100).saturating_sub(1).min(sorted.len() - 1);
    sorted.get(index).copied()
}

fn assess_candidate(request: &RepairContinuityRequest, candidate: &ArchitectureCandidate) -> CandidateAssessment {
    let mut remaining_stock = BTreeMap::new();
    for pool in &candidate.inventory_pools {
        for lot in &pool.lots {
            let quantity = if lot.evidence.class == EvidenceClass::Unknown {
                0
            } else if let Some(supplier) = &lot.supply_dependency_id {
                match dependency_factor(&request.dependency_observations, supplier, lot.available_from_window) {
                    ResourceFactor::Available(factor) => (f64::from(lot.quantity) * factor).floor() as u32,
                    ResourceFactor::Unavailable | ResourceFactor::Unknown => 0,
                }
            } else {
                lot.quantity
            };
            remaining_stock.insert(lot.stock_id.clone(), quantity);
        }
    }
    let mut effective_capacity = BTreeMap::<(String, usize), Option<f64>>::new();
    let mut remaining_capacity = BTreeMap::<(String, usize), Option<f64>>::new();
    for cell in &candidate.workcells {
        for (window, nominal) in cell.service_hours_by_window.iter().enumerate() {
            let staff = dependency_factor(&request.dependency_observations, &cell.staffing_dependency_id, window);
            let power = dependency_factor(&request.dependency_observations, &cell.power_dependency_id, window);
            let capacity = match (staff, power) {
                (ResourceFactor::Available(a), ResourceFactor::Available(b)) => Some(nominal * a.min(b)),
                (ResourceFactor::Unavailable, _) | (_, ResourceFactor::Unavailable) => Some(0.0),
                _ => None,
            };
            let key = (cell.cell_id.clone(), window);
            effective_capacity.insert(key.clone(), capacity);
            remaining_capacity.insert(key, capacity);
        }
    }

    let mut demands = request.demand.iter().collect::<Vec<_>>();
    demands.sort_by(|left, right| {
        let scarcity = |demand: &RepairDemand| candidate.workcells.iter().filter(|cell| {
            cell.capability_ids.contains(&demand.capability_id)
                && demand.required_skill_ids.iter().all(|value| cell.qualified_skill_ids.contains(value))
                && demand.required_tool_capabilities.iter().all(|value| cell.tool_capabilities.contains(value))
        }).count();
        (scarcity(left), left.due_window, std::cmp::Reverse(left.priority), &left.request_id)
            .cmp(&(scarcity(right), right.due_window, std::cmp::Reverse(right.priority), &right.request_id))
    });

    let mut results = Vec::new();
    let mut turnaround = Vec::new();
    for demand in demands {
        let mut best: Option<Assignment> = None;
        let mut possible_unknown = false;
        let mut blockers = BTreeSet::new();
        for window in demand.arrival_window..=demand.due_window {
            let mut cells = candidate.workcells.iter().collect::<Vec<_>>();
            cells.sort_by(|left, right| left.cell_id.cmp(&right.cell_id));
            for cell in cells {
                let capacity_blocked = remaining_capacity
                    .get(&(cell.cell_id.clone(), window))
                    .copied()
                    .flatten()
                    .is_some_and(|available| available + EPSILON < demand.service_hours);
                match check_cell(request, candidate, cell, demand, window, &remaining_stock) {
                    Fit::Feasible(_assignment) if capacity_blocked => {
                        blockers.insert(Blocker::CapacityUnavailable);
                    }
                    Fit::Feasible(assignment) => {
                        if best.as_ref().is_none_or(|old| {
                            (assignment.turnaround_hours, assignment.window_index, &assignment.cell_id)
                                < (old.turnaround_hours, old.window_index, &old.cell_id)
                        }) { best = Some(assignment); }
                    }
                    Fit::Indeterminate(reasons) => { possible_unknown = true; blockers.extend(reasons); }
                    Fit::Infeasible(reasons) => blockers.extend(reasons),
                }
            }
        }

        let result = if let Some(assignment) = best {
            let key = (assignment.cell_id.clone(), assignment.window_index);
            let old = remaining_capacity
                .get(&key)
                .copied()
                .flatten()
                .expect("a scheduled assignment must have known resource capacity");
            remaining_capacity.insert(key, Some((old - demand.service_hours).max(0.0)));
            for consumed in &assignment.stock_consumption {
                let old = remaining_stock.get(&consumed.stock_id).copied().unwrap_or(0);
                remaining_stock.insert(consumed.stock_id.clone(), old.saturating_sub(consumed.quantity));
            }
            turnaround.push(assignment.turnaround_hours);
            RequestAssessment {
                request_id: demand.request_id.clone(),
                capability_id: demand.capability_id.clone(),
                outcome: RequestOutcome::ScheduledWithinWindow,
                assigned_cell_id: Some(assignment.cell_id),
                assigned_window: Some(assignment.window_index),
                modelled_turnaround_hours: Some(assignment.turnaround_hours),
                blockers: assignment.blockers.into_iter().collect(),
                stock_consumption: assignment.stock_consumption,
            }
        } else {
            if possible_unknown {
                blockers.insert(Blocker::StockEvidenceUnknown);
            } else if blockers.is_empty() {
                blockers.insert(Blocker::NoQualifiedCell);
            }
            RequestAssessment {
                request_id: demand.request_id.clone(),
                capability_id: demand.capability_id.clone(),
                outcome: if possible_unknown { RequestOutcome::Indeterminate } else { RequestOutcome::UnservedWithinWindow },
                assigned_cell_id: None,
                assigned_window: None,
                modelled_turnaround_hours: None,
                blockers: blockers.into_iter().collect(),
                stock_consumption: Vec::new(),
            }
        };
        results.push(result);
    }
    results.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    turnaround.sort_by(f64::total_cmp);

    let scheduled = results.iter().filter(|item| item.outcome == RequestOutcome::ScheduledWithinWindow).count();
    let indeterminate = results.iter().filter(|item| item.outcome == RequestOutcome::Indeterminate).count();
    let unserved = results.iter().filter(|item| item.outcome == RequestOutcome::UnservedWithinWindow).count();

    let capabilities: BTreeSet<&str> = results.iter().map(|item| item.capability_id.as_str()).collect();
    let by_capability = capabilities.into_iter().map(|capability_id| {
        let subset = results.iter().filter(|item| item.capability_id == capability_id).collect::<Vec<_>>();
        CapabilitySummary {
            capability_id: capability_id.to_owned(),
            demand_count: subset.len(),
            scheduled_count: subset.iter().filter(|item| item.outcome == RequestOutcome::ScheduledWithinWindow).count(),
            indeterminate_count: subset.iter().filter(|item| item.outcome == RequestOutcome::Indeterminate).count(),
            unserved_count: subset.iter().filter(|item| item.outcome == RequestOutcome::UnservedWithinWindow).count(),
        }
    }).collect();

    let mut capacity_ledger = Vec::new();
    for cell in &candidate.workcells {
        for (window_index, nominal) in cell.service_hours_by_window.iter().enumerate() {
            let key = (cell.cell_id.clone(), window_index);
            let effective = effective_capacity.get(&key).copied().flatten();
            let remaining = remaining_capacity.get(&key).copied().flatten();
            let allocated = effective.zip(remaining).map(|(capacity, unallocated)| {
                (capacity - unallocated).max(0.0)
            });
            capacity_ledger.push(CapacityLedgerRow {
                cell_id: cell.cell_id.clone(),
                window_index,
                nominal_service_hours: *nominal,
                effective_capacity_hours: effective,
                allocated_service_hours: allocated,
                remaining_service_hours: remaining,
            });
        }
    }
    capacity_ledger.sort_by(|left, right| (&left.cell_id, left.window_index).cmp(&(&right.cell_id, right.window_index)));

    CandidateAssessment {
        candidate_id: candidate.candidate_id.clone(),
        architecture: candidate.architecture,
        demand_count: results.len(),
        scheduled_count: scheduled,
        indeterminate_count: indeterminate,
        unserved_count: unserved,
        scheduled_share_lower_bound: if results.is_empty() { 0.0 } else { scheduled as f64 / results.len() as f64 },
        turnaround_p50_hours: percentile(&turnaround, 50),
        turnaround_p90_hours: percentile(&turnaround, 90),
        by_capability,
        requests: results,
        capacity_ledger,
        claim_ceiling: "synthetic_modelled_feasibility_only_not_verified_restoration".to_owned(),
    }
}

/// Evaluate the same demand against centralized, local and distributed resource
/// topologies. Scenario counts are not event probabilities and scheduled work is
/// not evidence of an actual or safe restoration.
pub fn assess_repair_continuity(
    request: &RepairContinuityRequest,
) -> Result<RepairContinuityAssessment, RepairContinuityError> {
    validate(request).map_err(RepairContinuityError::InvalidInput)?;
    let mut candidates = request.candidates.iter().collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.architecture);
    let assessments = candidates.into_iter().map(|candidate| assess_candidate(request, candidate)).collect();
    Ok(RepairContinuityAssessment {
        schema_version: REPAIR_CONTINUITY_SCHEMA_VERSION.to_owned(),
        comparison_id: request.comparison_id.clone(),
        window_count: request.windows.len(),
        demand_count: request.demand.len(),
        assessments,
        claim_ceiling: "synthetic_modelled_feasibility_only_not_verified_restoration".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(class: EvidenceClass) -> EvidenceMetadata {
        EvidenceMetadata { class, source_id: None, source_uri: None, snapshot_date: None, artifact_sha256: None }
    }

    fn dep(id: &str, window: usize, availability: Availability, fraction: Option<f64>) -> DependencyObservation {
        DependencyObservation {
            dependency_id: id.into(), window_index: window, availability, available_fraction: fraction,
            evidence: evidence(EvidenceClass::Assumed),
        }
    }

    fn lot(id: &str, part: &str, revision: &str, quantity: u32, from: usize) -> StockLot {
        StockLot {
            stock_id: id.into(), part_key: part.into(), revision: revision.into(), quantity,
            fit: PartFit::ExactRevision, approved_for_part_keys: Vec::new(), available_from_window: from,
            expires_after_window: None, supply_dependency_id: if from == 0 { None } else { Some("supplier".into()) },
            evidence: evidence(EvidenceClass::Assumed),
        }
    }

    fn cell(id: &str, skills: &[&str], tools: &[&str], hours: Vec<f64>, travel: f64, pool: &str) -> WorkCell {
        WorkCell {
            cell_id: id.into(), node_id: format!("node-{id}"),
            capability_ids: vec!["fan".into(), "sensor".into(), "controller".into()],
            qualified_skill_ids: skills.iter().map(|value| (*value).into()).collect(),
            tool_capabilities: tools.iter().map(|value| (*value).into()).collect(),
            readiness_completeness: Completeness::Complete, readiness_evidence: evidence(EvidenceClass::Assumed),
            instruments: vec![
                Instrument { instrument_id: "multimeter".into(), calibration_state: CalibrationState::Qualified, calibrated_through_window: Some(3), evidence: evidence(EvidenceClass::Assumed) },
                Instrument { instrument_id: "oscilloscope".into(), calibration_state: CalibrationState::Qualified, calibrated_through_window: Some(1), evidence: evidence(EvidenceClass::Assumed) },
            ],
            service_hours_by_window: hours, travel_hours_to_service: travel,
            staffing_dependency_id: "staff".into(), power_dependency_id: "power".into(), service_route_dependency_id: "road".into(),
            inventory_access: vec![InventoryAccess { pool_id: pool.into(), route_dependency_id: "parts-road".into(), transfer_hours: 0.0 }],
        }
    }

    fn base_request() -> RepairContinuityRequest {
        let windows = (0..3).map(|index| ServiceWindow { id: format!("window-{index}"), duration_hours: 8.0 }).collect::<Vec<_>>();
        let demand = vec![
            RepairDemand {
                request_id: "fan-1".into(), capability_id: "fan".into(), required_skill_ids: vec!["electrical".into()],
                required_tool_capabilities: vec!["hand_tools".into(), "multimeter_tool".into()],
                required_instrument_ids: vec!["multimeter".into()],
                required_parts: vec![PartRequirement { part_key: "fan-motor".into(), required_revision: "r1".into(), quantity: 1 }],
                service_hours: 2.0, arrival_window: 0, due_window: 2, max_turnaround_hours: 16.0, priority: 2, scope_profile: "low_voltage_fan".into(),
            },
            RepairDemand {
                request_id: "sensor-1".into(), capability_id: "sensor".into(), required_skill_ids: vec!["electrical".into(), "calibration".into()],
                required_tool_capabilities: vec!["multimeter_tool".into()], required_instrument_ids: vec!["multimeter".into()],
                required_parts: vec![PartRequirement { part_key: "sensor-x".into(), required_revision: "r2".into(), quantity: 1 }],
                service_hours: 3.0, arrival_window: 0, due_window: 2, max_turnaround_hours: 18.0, priority: 1, scope_profile: "sensor".into(),
            },
        ];
        let mut dependencies = Vec::new();
        for window in 0..3 {
            dependencies.push(dep("staff", window, Availability::Available, None));
            dependencies.push(if window == 1 { dep("power", window, Availability::Derated, Some(0.5)) } else { dep("power", window, Availability::Available, None) });
            dependencies.push(if window == 1 { dep("road", window, Availability::Unavailable, None) } else { dep("road", window, Availability::Available, None) });
            dependencies.push(dep("parts-road", window, Availability::Available, None));
            dependencies.push(dep("supplier", window, Availability::Available, None));
        }
        let central = ArchitectureCandidate {
            candidate_id: "B0-central".into(), architecture: Architecture::CentralizedDepot,
            workcells: vec![cell("depot", &["electrical", "calibration"], &["hand_tools", "multimeter_tool"], vec![3.0; 3], 2.5, "central-pool")],
            inventory_pools: vec![InventoryPool { pool_id: "central-pool".into(), location_id: "depot".into(), lots: vec![lot("central-fan", "fan-motor", "r1", 1, 0), lot("central-sensor", "sensor-x", "r2", 1, 1)] }],
        };
        let local = ArchitectureCandidate {
            candidate_id: "B1-local".into(), architecture: Architecture::LocalServiceCell,
            workcells: vec![cell("local", &["electrical"], &["hand_tools", "multimeter_tool"], vec![3.0; 3], 0.25, "local-pool")],
            inventory_pools: vec![InventoryPool { pool_id: "local-pool".into(), location_id: "local".into(), lots: vec![lot("local-fan", "fan-motor", "r1", 1, 0)] }],
        };
        let distributed = ArchitectureCandidate {
            candidate_id: "B2-distributed".into(), architecture: Architecture::DistributedSharedReserve,
            workcells: vec![
                cell("local-node", &["electrical"], &["hand_tools", "multimeter_tool"], vec![2.0; 3], 0.25, "reserve"),
                cell("specialist-node", &["electrical", "calibration"], &["hand_tools", "multimeter_tool"], vec![3.0; 3], 1.0, "reserve"),
            ],
            inventory_pools: vec![InventoryPool { pool_id: "reserve".into(), location_id: "regional-reserve".into(), lots: vec![lot("reserve-fan", "fan-motor", "r1", 1, 0), lot("reserve-sensor", "sensor-x", "r2", 1, 1)] }],
        };
        RepairContinuityRequest {
            schema_version: REPAIR_CONTINUITY_SCHEMA_VERSION.into(), comparison_id: "synthetic-repair-continuity".into(),
            windows, demand, dependency_observations: dependencies, candidates: vec![central, local, distributed],
        }
    }


    #[test]
    fn excessive_inventory_scan_work_fails_before_evaluation() {
        let mut request = base_request();
        for index in 0..2_000 {
            let mut demand = request.demand[0].clone();
            demand.request_id = format!("fan-load-{index:04}");
            request.demand.push(demand);
        }
        let central = request.candidates.iter_mut()
            .find(|candidate| candidate.architecture == Architecture::CentralizedDepot)
            .unwrap();
        let mut lots = Vec::with_capacity(MAX_STOCK_LOTS_PER_CANDIDATE);
        for index in 0..MAX_STOCK_LOTS_PER_CANDIDATE {
            lots.push(StockLot {
                stock_id: format!("load-lot-{index:05}"),
                part_key: "fan-motor".into(),
                revision: "r1".into(),
                quantity: 1,
                fit: PartFit::ExactRevision,
                approved_for_part_keys: Vec::new(),
                available_from_window: 0,
                expires_after_window: None,
                supply_dependency_id: None,
                evidence: evidence(EvidenceClass::Assumed),
            });
        }
        central.inventory_pools[0].lots = lots;
        let error = assess_repair_continuity(&request).unwrap_err();
        assert!(matches!(
            error,
            RepairContinuityError::InvalidInput(errors)
                if errors.iter().any(|message| message.contains("stock-inspection budget exceeded"))
        ));
    }

    #[test]
    fn candidates_share_demand_and_report_modelled_not_verified_service() {
        let result = assess_repair_continuity(&base_request()).unwrap();
        assert_eq!(result.demand_count, 2);
        assert_eq!(result.assessments.len(), 3);
        assert!(result.assessments.iter().all(|assessment| assessment.demand_count == 2));
        assert!(result.assessments.iter().all(|assessment| assessment.claim_ceiling == "synthetic_modelled_feasibility_only_not_verified_restoration"));
    }

    #[test]
    fn shared_reserve_stock_cannot_be_double_booked() {
        let mut request = base_request();
        request.demand.push(RepairDemand {
            request_id: "fan-2".into(), capability_id: "fan".into(), required_skill_ids: vec!["electrical".into()],
            required_tool_capabilities: vec!["hand_tools".into(), "multimeter_tool".into()], required_instrument_ids: vec!["multimeter".into()],
            required_parts: vec![PartRequirement { part_key: "fan-motor".into(), required_revision: "r1".into(), quantity: 1 }],
            service_hours: 1.0, arrival_window: 0, due_window: 2, max_turnaround_hours: 24.0, priority: 0, scope_profile: "low_voltage_fan".into(),
        });
        let assessment = assess_repair_continuity(&request).unwrap();
        let distributed = assessment.assessments.iter().find(|value| value.architecture == Architecture::DistributedSharedReserve).unwrap();
        let scheduled = distributed.requests.iter().filter(|value| value.capability_id == "fan" && value.outcome == RequestOutcome::ScheduledWithinWindow).count();
        assert_eq!(scheduled, 1, "one finite shared motor stock cannot satisfy two requests");
    }

    #[test]
    fn missing_dependency_state_is_indeterminate_not_zero_or_available() {
        let mut request = base_request();
        request.dependency_observations.retain(|value| !(value.dependency_id == "power-local" && value.window_index == 0));
        let assessment = assess_repair_continuity(&request).unwrap();
        assert!(assessment.assessments.iter().any(|candidate| candidate.indeterminate_count > 0));
        let local = assessment.assessments.iter()
            .find(|candidate| candidate.architecture == Architecture::LocalServiceCell)
            .unwrap();
        assert_eq!(
            local.capacity_ledger.iter()
                .find(|row| row.cell_id == "local" && row.window_index == 0)
                .unwrap()
                .effective_capacity_hours,
            None,
            "unknown power must remain unknown, not a zero-capacity observation"
        );
    }


    #[test]
    fn availability_with_unknown_evidence_resolves_to_unknown() {
        let mut request = base_request();
        for observation in &mut request.dependency_observations {
            if observation.dependency_id == "power" && observation.window_index == 0 {
                observation.evidence = evidence(EvidenceClass::Unknown);
            }
        }
        let assessment = assess_repair_continuity(&request).unwrap();
        assert!(assessment.assessments.iter().any(|candidate| candidate.indeterminate_count > 0));
        let local = assessment.assessments.iter().find(|item| item.architecture == Architecture::LocalServiceCell).unwrap();
        assert_eq!(
            local.capacity_ledger.iter().find(|row| row.cell_id == "local" && row.window_index == 0).unwrap().effective_capacity_hours,
            None
        );
    }

    #[test]
    fn duplicate_stock_identity_and_malformed_observed_digest_reject() {
        let mut duplicate = base_request();
        let lot_copy = duplicate.candidates[0].inventory_pools[0].lots[0].clone();
        duplicate.candidates[0].inventory_pools[0].lots.push(lot_copy);
        assert!(matches!(assess_repair_continuity(&duplicate), Err(RepairContinuityError::InvalidInput(errors)) if errors.iter().any(|error| error.contains("duplicate stock ID"))));

        let mut malformed = base_request();
        malformed.dependency_observations[0].evidence = EvidenceMetadata {
            class: EvidenceClass::Observed,
            source_id: Some("source-1".into()), source_uri: Some("https://example.invalid/snapshot".into()),
            snapshot_date: Some("2026-10-19".into()), artifact_sha256: Some("not-a-digest".into()),
        };
        assert!(assess_repair_continuity(&malformed).is_err());
    }

    #[test]
    fn input_permutation_does_not_change_results() {
        let mut first = base_request();
        let expected = assess_repair_continuity(&first).unwrap();
        first.demand.reverse();
        first.dependency_observations.reverse();
        first.candidates.reverse();
        for candidate in &mut first.candidates {
            candidate.workcells.reverse();
            candidate.inventory_pools.reverse();
            for pool in &mut candidate.inventory_pools { pool.lots.reverse(); }
        }
        assert_eq!(expected, assess_repair_continuity(&first).unwrap());
    }

    #[test]
    fn json_fixture_compares_central_local_and_distributed_architectures() {
        let request: RepairContinuityRequest = serde_json::from_str(include_str!(
            "../../schemas/examples/repair-workcell-continuity-v0.1.json"
        ))
        .expect("repair-continuity fixture must deserialize");
        let result = assess_repair_continuity(&request).expect("fixture must be valid");
        let central = result.assessments.iter().find(|item| item.architecture == Architecture::CentralizedDepot).unwrap();
        let local = result.assessments.iter().find(|item| item.architecture == Architecture::LocalServiceCell).unwrap();
        let distributed = result.assessments.iter().find(|item| item.architecture == Architecture::DistributedSharedReserve).unwrap();

        assert_eq!(central.unserved_count, 1, "central stock arrives after the central route is disrupted");
        assert_eq!(local.scheduled_count, 1, "local stock supports repair before the disruption window");
        assert_eq!(distributed.scheduled_count, 1, "the shared reserve is modeled as arriving in the later window");
        assert_eq!(
            distributed.requests[0].assigned_window,
            Some(1),
            "distributed reserve assignment should be explicitly time-bounded"
        );
        assert!(result.assessments.iter().all(|item| {
            item.scheduled_count + item.indeterminate_count + item.unserved_count == item.demand_count
        }));
    }

    #[test]
    fn expired_instrument_and_unknown_part_fit_cannot_confirm_schedule() {
        let mut expired = base_request();
        let candidate = expired.candidates.iter_mut().find(|value| value.architecture == Architecture::DistributedSharedReserve).unwrap();
        let cell = candidate.workcells.iter_mut().find(|value| value.cell_id == "specialist-node").unwrap();
        cell.instruments.iter_mut().find(|value| value.instrument_id == "multimeter").unwrap().calibration_state = CalibrationState::Expired;
        let outcome = assess_repair_continuity(&expired).unwrap();
        let distributed = outcome.assessments.iter().find(|value| value.architecture == Architecture::DistributedSharedReserve).unwrap();
        let sensor = distributed.requests.iter().find(|value| value.request_id == "sensor-1").unwrap();
        assert_ne!(sensor.outcome, RequestOutcome::ScheduledWithinWindow);
        assert!(sensor.blockers.contains(&Blocker::CalibrationExpired) || sensor.blockers.contains(&Blocker::MissingSkill));

        let mut unknown_fit = base_request();
        let local_candidate = unknown_fit.candidates.iter_mut()
            .find(|candidate| candidate.architecture == Architecture::LocalServiceCell)
            .unwrap();
        local_candidate.inventory_pools[0].lots[0].fit = PartFit::Unknown;
        let outcome = assess_repair_continuity(&unknown_fit).unwrap();
        let local = outcome.assessments.iter()
            .find(|value| value.architecture == Architecture::LocalServiceCell)
            .unwrap();
        let fan = local.requests.iter().find(|value| value.request_id == "fan-1").unwrap();
        assert_eq!(fan.outcome, RequestOutcome::Indeterminate);
        assert!(fan.blockers.contains(&Blocker::PartFitUnknown));
    }
}
