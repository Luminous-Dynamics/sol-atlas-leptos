// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Exhaustive finite-instance network-constrained DC dispatch oracle.
//!
//! Exactness is limited to the declared discrete generator-power lattice and
//! fixed-injection linear DC model. Each feasible candidate is independently
//! checked by the separate DC receipt auditor. This is not continuous DC-OPF,
//! AC-OPF, security-constrained redispatch, or an operational controller.

use std::collections::BTreeMap;

use dc_power_flow_independent_auditor::{
    AuditOptions, AuditReport, audit_dc_power_flow,
};
use dc_power_flow_reference::{
    DcBranch, DcBus, DcNetwork, SolveOptions, solve_dc_power_flow,
};

const MAX_BUSES: usize = 64;
const MAX_GENERATORS: usize = 32;
const MAX_BRANCHES: usize = 256;
const MAX_COMBINATIONS: u64 = 1_000_000;
const AUDIT_SLACK_TOLERANCE_MW: f64 = 1.0e-7;
const ANGLE_LIMIT_TOLERANCE_DEGREES: f64 = 1.0e-9;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DcOpfBus {
    pub id: i64,
    /// Fixed demand in integer multiples of the case power quantum.
    pub demand_quanta: u64,
    pub is_reference: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DcOpfGenerator {
    pub id: u64,
    pub bus_id: i64,
    /// Inclusive output bounds on the declared integer power lattice.
    pub p_min_quanta: u64,
    pub p_max_quanta: u64,
    /// Integer objective units per output quantum. This is not a currency.
    pub cost_units_per_quantum: u64,
    /// Grams of CO2e per output quantum under the declared accounting boundary.
    pub carbon_grams_per_quantum: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DcOpfBranch {
    pub id: u64,
    pub from_bus: i64,
    pub to_bus: i64,
    pub reactance_pu: f64,
    /// MATPOWER convention: zero means unity tap.
    pub tap_ratio: f64,
    pub phase_shift_degrees: f64,
    /// None means no declared Rate A limit.
    pub rate_a_limit_mw: Option<f64>,
    /// Symmetric absolute bound on theta(from) - theta(to); None means unbounded.
    pub angle_difference_limit_degrees: Option<f64>,
    pub in_service: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DcOpfCase {
    /// Caller-supplied immutable source/case identity. A digest is recommended.
    pub source_identity: String,
    pub base_mva: f64,
    /// All fixed demand and generator outputs are integer multiples of this MW quantum.
    pub power_quantum_mw: f64,
    /// Exactly one reference bus and exactly one generator at that bus are required.
    pub buses: Vec<DcOpfBus>,
    pub generators: Vec<DcOpfGenerator>,
    /// Branch order is stable and retained in result receipts.
    pub branches: Vec<DcOpfBranch>,
    /// Hard all-or-nothing cap on non-reference generator assignments.
    pub max_combinations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratorAllocation {
    pub generator_id: u64,
    pub power_quanta: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BranchFlowReceipt {
    pub branch_id: u64,
    pub from_bus: i64,
    pub to_bus: i64,
    pub flow_from_to_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BusAngleReceipt {
    pub bus_id: i64,
    pub angle_degrees: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParetoDispatch {
    pub generation: Vec<GeneratorAllocation>,
    pub operating_cost_units: u128,
    pub carbon_grams: u128,
    pub branch_flows: Vec<BranchFlowReceipt>,
    pub bus_angles: Vec<BusAngleReceipt>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OracleReport {
    pub source_identity: String,
    pub expected_combinations: u64,
    pub evaluated_combinations: u64,
    pub feasible_assignments: u64,
    pub declared_infeasible_assignments: u64,
    pub solver_unknowns: u64,
    pub auditor_rejections: u64,
    /// True only when every assignment was enumerated and every candidate
    /// received an auditable feasible/infeasible disposition.
    pub frontier_complete: bool,
    pub frontier: Vec<ParetoDispatch>,
    pub solver_unknown_reasons: Vec<String>,
    pub auditor_rejection_reasons: Vec<String>,
    pub claim_ceiling: &'static str,
}

impl OracleReport {
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.frontier_complete
            && self.evaluated_combinations == self.expected_combinations
            && self.solver_unknowns == 0
            && self.auditor_rejections == 0
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let frontier = self
            .frontier
            .iter()
            .map(dispatch_to_json)
            .collect::<Vec<_>>()
            .join(",\n");
        let unknowns = self
            .solver_unknown_reasons
            .iter()
            .map(|reason| json_quote(reason))
            .collect::<Vec<_>>()
            .join(",");
        let rejections = self
            .auditor_rejection_reasons
            .iter()
            .map(|reason| json_quote(reason))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{\n",
                "  \"schema_version\":\"dc-opf-finite-oracle.v0.1.0\",\n",
                "  \"source_identity\":{},\n",
                "  \"expected_combinations\":{},\n",
                "  \"evaluated_combinations\":{},\n",
                "  \"feasible_assignments\":{},\n",
                "  \"declared_infeasible_assignments\":{},\n",
                "  \"solver_unknowns\":{},\n",
                "  \"auditor_rejections\":{},\n",
                "  \"frontier_complete\":{},\n",
                "  \"is_exact\":{},\n",
                "  \"frontier\":[{}],\n",
                "  \"solver_unknown_reasons\":[{}],\n",
                "  \"auditor_rejection_reasons\":[{}],\n",
                "  \"claim_ceiling\":{}\n",
                "}}"
            ),
            json_quote(&self.source_identity),
            self.expected_combinations,
            self.evaluated_combinations,
            self.feasible_assignments,
            self.declared_infeasible_assignments,
            self.solver_unknowns,
            self.auditor_rejections,
            self.frontier_complete,
            self.is_exact(),
            frontier,
            unknowns,
            rejections,
            json_quote(self.claim_ceiling)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleError {
    pub reason: String,
}

impl OracleError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for OracleError {}

#[derive(Clone, Debug)]
struct ValidatedCase {
    reference_bus_id: i64,
    reference_generator_index: usize,
    variable_generator_indices: Vec<usize>,
    total_demand_quanta: u64,
    expected_combinations: u64,
}

fn validate_case(case: &DcOpfCase) -> Result<ValidatedCase, OracleError> {
    if case.source_identity.trim().is_empty() {
        return Err(OracleError::new("source identity must not be empty"));
    }
    if !case.base_mva.is_finite() || case.base_mva <= 0.0 {
        return Err(OracleError::new("base MVA must be finite and positive"));
    }
    if !case.power_quantum_mw.is_finite() || case.power_quantum_mw <= 0.0 {
        return Err(OracleError::new(
            "power quantum must be finite and positive",
        ));
    }
    if case.max_combinations == 0 || case.max_combinations > MAX_COMBINATIONS {
        return Err(OracleError::new(format!(
            "combination budget must be within 1..={MAX_COMBINATIONS}"
        )));
    }
    if case.buses.is_empty() || case.buses.len() > MAX_BUSES {
        return Err(OracleError::new(format!(
            "bus count must be within 1..={MAX_BUSES}"
        )));
    }
    if case.generators.is_empty() || case.generators.len() > MAX_GENERATORS {
        return Err(OracleError::new(format!(
            "generator count must be within 1..={MAX_GENERATORS}"
        )));
    }
    if case.branches.len() > MAX_BRANCHES {
        return Err(OracleError::new(format!(
            "branch count must not exceed {MAX_BRANCHES}"
        )));
    }

    let mut bus_indices = BTreeMap::new();
    let mut reference_bus_id = None;
    let mut total_demand_quanta = 0_u64;
    let mut previous_bus_id = None;
    for (index, bus) in case.buses.iter().enumerate() {
        if bus.id <= 0 {
            return Err(OracleError::new(format!(
                "bus ID must be positive, got {}",
                bus.id
            )));
        }
        if previous_bus_id.is_some_and(|previous| bus.id <= previous) {
            return Err(OracleError::new(
                "buses must be ordered by strictly increasing ID",
            ));
        }
        previous_bus_id = Some(bus.id);
        bus_indices.insert(bus.id, index);
        total_demand_quanta = total_demand_quanta
            .checked_add(bus.demand_quanta)
            .ok_or_else(|| OracleError::new("total demand quanta overflow"))?;
        if bus.is_reference && reference_bus_id.replace(bus.id).is_some() {
            return Err(OracleError::new(
                "exactly one reference bus is supported",
            ));
        }
    }
    let reference_bus_id =
        reference_bus_id.ok_or_else(|| OracleError::new("one reference bus is required"))?;

    let mut previous_generator_id = None;
    let mut reference_generator_index = None;
    let mut variable_generator_indices = Vec::new();
    for (index, generator) in case.generators.iter().enumerate() {
        if generator.id == 0 {
            return Err(OracleError::new("generator ID must be positive"));
        }
        if previous_generator_id.is_some_and(|previous| generator.id <= previous) {
            return Err(OracleError::new(
                "generators must be ordered by strictly increasing ID",
            ));
        }
        previous_generator_id = Some(generator.id);
        if !bus_indices.contains_key(&generator.bus_id) {
            return Err(OracleError::new(format!(
                "generator {} references unknown bus {}",
                generator.id, generator.bus_id
            )));
        }
        if generator.p_min_quanta > generator.p_max_quanta {
            return Err(OracleError::new(format!(
                "generator {} has Pmin greater than Pmax",
                generator.id
            )));
        }
        if generator.bus_id == reference_bus_id {
            if reference_generator_index.replace(index).is_some() {
                return Err(OracleError::new(
                    "exactly one generator at the reference bus is supported",
                ));
            }
        } else {
            variable_generator_indices.push(index);
        }
    }
    let reference_generator_index = reference_generator_index
        .ok_or_else(|| OracleError::new("one generator at the reference bus is required"))?;

    let mut previous_branch_id = None;
    for branch in &case.branches {
        if branch.id == 0 {
            return Err(OracleError::new("branch ID must be positive"));
        }
        if previous_branch_id.is_some_and(|previous| branch.id <= previous) {
            return Err(OracleError::new(
                "branches must be ordered by strictly increasing ID",
            ));
        }
        previous_branch_id = Some(branch.id);
        if branch.from_bus == branch.to_bus {
            return Err(OracleError::new(format!(
                "branch {} is a self-loop",
                branch.id
            )));
        }
        if !bus_indices.contains_key(&branch.from_bus)
            || !bus_indices.contains_key(&branch.to_bus)
        {
            return Err(OracleError::new(format!(
                "branch {} references an unknown bus",
                branch.id
            )));
        }
        if !branch.reactance_pu.is_finite() || branch.reactance_pu <= 0.0 {
            return Err(OracleError::new(format!(
                "branch {} reactance must be finite and positive",
                branch.id
            )));
        }
        if !branch.tap_ratio.is_finite() || branch.tap_ratio < 0.0 {
            return Err(OracleError::new(format!(
                "branch {} tap ratio must be finite and non-negative",
                branch.id
            )));
        }
        if !branch.phase_shift_degrees.is_finite() {
            return Err(OracleError::new(format!(
                "branch {} phase shift must be finite",
                branch.id
            )));
        }
        if let Some(limit) = branch.rate_a_limit_mw
            && (!limit.is_finite() || limit <= 0.0)
        {
            return Err(OracleError::new(format!(
                "branch {} Rate A must be positive or absent",
                branch.id
            )));
        }
        if let Some(limit) = branch.angle_difference_limit_degrees
            && (!limit.is_finite() || limit <= 0.0)
        {
            return Err(OracleError::new(format!(
                "branch {} angle-difference limit must be positive or absent",
                branch.id
            )));
        }
    }

    let mut expected_combinations = 1_u64;
    for index in &variable_generator_indices {
        let generator = &case.generators[*index];
        let width = generator
            .p_max_quanta
            .checked_sub(generator.p_min_quanta)
            .and_then(|range| range.checked_add(1))
            .ok_or_else(|| OracleError::new("generator assignment width overflow"))?;
        expected_combinations = expected_combinations
            .checked_mul(width)
            .ok_or_else(|| OracleError::new("search-space combination count overflow"))?;
        if expected_combinations > case.max_combinations {
            return Err(OracleError::new(format!(
                "exact search requires {expected_combinations} combinations; configured budget is {}",
                case.max_combinations
            )));
        }
    }

    Ok(ValidatedCase {
        reference_bus_id,
        reference_generator_index,
        variable_generator_indices,
        total_demand_quanta,
        expected_combinations,
    })
}

fn json_quote(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control <= '\u{1f}' => {
                output.push_str(&format!("\\u{:04x}", control as u32));
            }
            ordinary => output.push(ordinary),
        }
    }
    output.push('"');
    output
}

fn json_number(value: f64) -> String {
    if !value.is_finite() {
        "null".to_owned()
    } else if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

fn dispatch_to_json(dispatch: &ParetoDispatch) -> String {
    let generation = dispatch
        .generation
        .iter()
        .map(|allocation| {
            format!(
                "{{\"generator_id\":{},\"power_quanta\":{}}}",
                allocation.generator_id, allocation.power_quanta
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let flows = dispatch
        .branch_flows
        .iter()
        .map(|flow| {
            format!(
                "{{\"branch_id\":{},\"from_bus\":{},\"to_bus\":{},\"flow_from_to_mw\":{}}}",
                flow.branch_id,
                flow.from_bus,
                flow.to_bus,
                json_number(flow.flow_from_to_mw)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let angles = dispatch
        .bus_angles
        .iter()
        .map(|angle| {
            format!(
                "{{\"bus_id\":{},\"angle_degrees\":{}}}",
                angle.bus_id,
                json_number(angle.angle_degrees)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"operating_cost_units\":{},",
            "\"carbon_grams\":{},",
            "\"generation\":[{}],",
            "\"branch_flows\":[{}],",
            "\"bus_angles\":[{}]",
            "}}"
        ),
        dispatch.operating_cost_units,
        dispatch.carbon_grams,
        generation,
        flows,
        angles
    )
}

fn dominates(left: &ParetoDispatch, right: &ParetoDispatch) -> bool {
    left.operating_cost_units <= right.operating_cost_units
        && left.carbon_grams <= right.carbon_grams
        && (left.operating_cost_units < right.operating_cost_units
            || left.carbon_grams < right.carbon_grams)
}

fn insert_frontier(frontier: &mut Vec<ParetoDispatch>, candidate: ParetoDispatch) {
    if frontier
        .iter()
        .any(|existing| dominates(existing, &candidate))
    {
        return;
    }
    frontier.retain(|existing| !dominates(&candidate, existing));
    frontier.push(candidate);
    frontier.sort_by(|left, right| {
        left.operating_cost_units
            .cmp(&right.operating_cost_units)
            .then_with(|| left.carbon_grams.cmp(&right.carbon_grams))
            .then_with(|| {
                left.generation
                    .iter()
                    .map(|allocation| allocation.power_quanta)
                    .cmp(
                        right
                            .generation
                            .iter()
                            .map(|allocation| allocation.power_quanta),
                    )
            })
    });
}

fn checked_objective(
    case: &DcOpfCase,
    generation_quanta: &[u64],
) -> Result<(u128, u128), OracleError> {
    let mut cost = 0_u128;
    let mut carbon = 0_u128;
    for (generator, quanta) in case.generators.iter().zip(generation_quanta) {
        let units = u128::from(*quanta);
        cost = cost
            .checked_add(
                u128::from(generator.cost_units_per_quantum)
                    .checked_mul(units)
                    .ok_or_else(|| OracleError::new("operating-cost multiplication overflow"))?,
            )
            .ok_or_else(|| OracleError::new("operating-cost accumulation overflow"))?;
        carbon = carbon
            .checked_add(
                u128::from(generator.carbon_grams_per_quantum)
                    .checked_mul(units)
                    .ok_or_else(|| OracleError::new("carbon multiplication overflow"))?,
            )
            .ok_or_else(|| OracleError::new("carbon accumulation overflow"))?;
    }
    Ok((cost, carbon))
}

fn evaluate_assignment(
    case: &DcOpfCase,
    validated: &ValidatedCase,
    generation_quanta: &mut [u64],
    report: &mut OracleReport,
) -> Result<(), OracleError> {
    let non_reference_generation = generation_quanta
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != validated.reference_generator_index)
        .try_fold(0_u64, |total, (_, quanta)| total.checked_add(*quanta))
        .ok_or_else(|| OracleError::new("total generation quanta overflow"))?;

    if non_reference_generation > validated.total_demand_quanta {
        report.declared_infeasible_assignments += 1;
        return Ok(());
    }
    let reference_generation = validated.total_demand_quanta - non_reference_generation;
    let reference_generator = &case.generators[validated.reference_generator_index];
    if reference_generation < reference_generator.p_min_quanta
        || reference_generation > reference_generator.p_max_quanta
    {
        report.declared_infeasible_assignments += 1;
        return Ok(());
    }
    generation_quanta[validated.reference_generator_index] = reference_generation;

    let buses = case
        .buses
        .iter()
        .map(|bus| {
            let generation_at_bus = case
                .generators
                .iter()
                .zip(generation_quanta.iter())
                .filter(|(generator, _)| generator.bus_id == bus.id)
                .try_fold(0_u64, |total, (_, quanta)| total.checked_add(*quanta))
                .ok_or_else(|| {
                    OracleError::new(format!(
                        "generation sum overflow at bus {}",
                        bus.id
                    ))
                })?;
            let specified_injection_mw = if bus.is_reference {
                None
            } else {
                let net_quanta =
                    i128::from(generation_at_bus) - i128::from(bus.demand_quanta);
                Some(net_quanta as f64 * case.power_quantum_mw)
            };
            Ok(DcBus {
                id: bus.id,
                specified_injection_mw,
            })
        })
        .collect::<Result<Vec<_>, OracleError>>()?;
    let branches = case
        .branches
        .iter()
        .map(|branch| DcBranch {
            from_bus: branch.from_bus,
            to_bus: branch.to_bus,
            reactance_pu: branch.reactance_pu,
            tap_ratio: branch.tap_ratio,
            phase_shift_degrees: branch.phase_shift_degrees,
            rate_a_limit_mw: branch.rate_a_limit_mw,
            in_service: branch.in_service,
        })
        .collect::<Vec<_>>();
    let network = DcNetwork {
        base_mva: case.base_mva,
        buses,
        branches,
    };

    let solution = match solve_dc_power_flow(&network, SolveOptions::default()) {
        Ok(solution) => solution,
        Err(error) => {
            report.solver_unknowns += 1;
            if report.solver_unknown_reasons.len() < 8 {
                report.solver_unknown_reasons.push(error.to_string());
            }
            return Ok(());
        }
    };
    let audit = match audit_dc_power_flow(&network, &solution, AuditOptions::default()) {
        Ok(audit) => audit,
        Err(error) => {
            report.auditor_rejections += 1;
            if report.auditor_rejection_reasons.len() < 8 {
                report.auditor_rejection_reasons.push(error.to_string());
            }
            return Ok(());
        }
    };

    let reference_bus = case
        .buses
        .iter()
        .find(|bus| bus.id == validated.reference_bus_id)
        .expect("validated reference bus");
    let expected_reference_injection_mw = (reference_generation as f64
        - reference_bus.demand_quanta as f64)
        * case.power_quantum_mw;
    let observed_reference_injection_mw = solution
        .slack_injections
        .iter()
        .find(|slack| slack.bus_id == validated.reference_bus_id)
        .map(|slack| slack.injection_mw);
    let Some(observed_reference_injection_mw) = observed_reference_injection_mw else {
        report.auditor_rejections += 1;
        if report.auditor_rejection_reasons.len() < 8 {
            report
                .auditor_rejection_reasons
                .push("independent receipt omitted reference-bus slack injection".to_owned());
        }
        return Ok(());
    };
    if (observed_reference_injection_mw - expected_reference_injection_mw).abs()
        > AUDIT_SLACK_TOLERANCE_MW
    {
        report.auditor_rejections += 1;
        if report.auditor_rejection_reasons.len() < 8 {
            report.auditor_rejection_reasons.push(format!(
                "reference generator balance mismatch: expected {expected_reference_injection_mw}, observed {observed_reference_injection_mw}"
            ));
        }
        return Ok(());
    }

    if !audit.within_nodal_tolerance || !audit.within_declared_branch_limits {
        report.declared_infeasible_assignments += 1;
        return Ok(());
    }

    let angle_by_bus = solution
        .bus_states
        .iter()
        .map(|state| (state.bus_id, state.voltage_angle_rad))
        .collect::<BTreeMap<_, _>>();
    for branch in case.branches.iter().filter(|branch| branch.in_service) {
        let Some(limit) = branch.angle_difference_limit_degrees else {
            continue;
        };
        let from = *angle_by_bus
            .get(&branch.from_bus)
            .ok_or_else(|| OracleError::new("solution omitted branch from-bus angle"))?;
        let to = *angle_by_bus
            .get(&branch.to_bus)
            .ok_or_else(|| OracleError::new("solution omitted branch to-bus angle"))?;
        let difference_degrees = (from - to).to_degrees().abs();
        if difference_degrees > limit + ANGLE_LIMIT_TOLERANCE_DEGREES {
            report.declared_infeasible_assignments += 1;
            return Ok(());
        }
    }

    let (operating_cost_units, carbon_grams) =
        checked_objective(case, generation_quanta)?;
    let generation = case
        .generators
        .iter()
        .zip(generation_quanta.iter())
        .map(|(generator, quanta)| GeneratorAllocation {
            generator_id: generator.id,
            power_quanta: *quanta,
        })
        .collect::<Vec<_>>();
    let branch_flows = solution
        .branch_flows
        .iter()
        .zip(case.branches.iter())
        .map(|(flow, branch)| BranchFlowReceipt {
            branch_id: branch.id,
            from_bus: flow.from_bus,
            to_bus: flow.to_bus,
            flow_from_to_mw: flow.flow_from_to_mw,
        })
        .collect::<Vec<_>>();
    let bus_angles = solution
        .bus_states
        .iter()
        .map(|state| BusAngleReceipt {
            bus_id: state.bus_id,
            angle_degrees: state.voltage_angle_rad.to_degrees(),
        })
        .collect::<Vec<_>>();

    report.feasible_assignments += 1;
    insert_frontier(
        &mut report.frontier,
        ParetoDispatch {
            generation,
            operating_cost_units,
            carbon_grams,
            branch_flows,
            bus_angles,
        },
    );
    Ok(())
}

fn enumerate_depth(
    case: &DcOpfCase,
    validated: &ValidatedCase,
    depth: usize,
    generation_quanta: &mut [u64],
    report: &mut OracleReport,
) -> Result<(), OracleError> {
    if depth == validated.variable_generator_indices.len() {
        report.evaluated_combinations += 1;
        return evaluate_assignment(case, validated, generation_quanta, report);
    }
    let generator_index = validated.variable_generator_indices[depth];
    let generator = &case.generators[generator_index];
    let mut quanta = generator.p_min_quanta;
    loop {
        generation_quanta[generator_index] = quanta;
        enumerate_depth(
            case,
            validated,
            depth + 1,
            generation_quanta,
            report,
        )?;
        if quanta == generator.p_max_quanta {
            break;
        }
        quanta += 1;
    }
    Ok(())
}

/// Exhaustively enumerate every declared non-reference generator assignment.
///
/// The reference generator is not a free optimization variable: its output is
/// derived from system power balance and then checked against its own Pmin/Pmax.
/// Every complete assignment receives an audited feasible, declared-infeasible,
/// solver-unknown, or auditor-rejected disposition. Unknowns/rejections prevent
/// an exact Pareto-completeness claim.
pub fn solve_finite_dc_opf(case: &DcOpfCase) -> Result<OracleReport, OracleError> {
    let validated = validate_case(case)?;
    let mut report = OracleReport {
        source_identity: case.source_identity.clone(),
        expected_combinations: validated.expected_combinations,
        evaluated_combinations: 0,
        feasible_assignments: 0,
        declared_infeasible_assignments: 0,
        solver_unknowns: 0,
        auditor_rejections: 0,
        frontier_complete: false,
        frontier: Vec::new(),
        solver_unknown_reasons: Vec::new(),
        auditor_rejection_reasons: Vec::new(),
        claim_ceiling: "exact_finite_quantized_fixed_injection_linear_dc_dispatch_only",
    };
    let mut generation_quanta = vec![0_u64; case.generators.len()];
    enumerate_depth(
        case,
        &validated,
        0,
        &mut generation_quanta,
        &mut report,
    )?;
    report.frontier_complete = report.evaluated_combinations
        == report.expected_combinations
        && report.solver_unknowns == 0
        && report.auditor_rejections == 0;
    Ok(report)
}

/// Three-bus analytic case used by tests and the standalone self-test binary.
///
/// Total demand is 4 MW. The cost/carbon Pareto frontier is expected to include
/// (22 cost units, 5 g), (20, 14 g), (18, 23 g), and (16, 32 g).
#[must_use]
pub fn three_bus_known_answer_case() -> DcOpfCase {
    DcOpfCase {
        source_identity: "synthetic-known-answer:three-bus-dc-opf-v0.1".to_owned(),
        base_mva: 100.0,
        power_quantum_mw: 1.0,
        buses: vec![
            DcOpfBus {
                id: 1,
                demand_quanta: 1,
                is_reference: true,
            },
            DcOpfBus {
                id: 2,
                demand_quanta: 2,
                is_reference: false,
            },
            DcOpfBus {
                id: 3,
                demand_quanta: 1,
                is_reference: false,
            },
        ],
        generators: vec![
            DcOpfGenerator {
                id: 10,
                bus_id: 1,
                p_min_quanta: 1,
                p_max_quanta: 3,
                cost_units_per_quantum: 10,
                carbon_grams_per_quantum: 2,
            },
            DcOpfGenerator {
                id: 20,
                bus_id: 2,
                p_min_quanta: 0,
                p_max_quanta: 3,
                cost_units_per_quantum: 2,
                carbon_grams_per_quantum: 10,
            },
            DcOpfGenerator {
                id: 30,
                bus_id: 3,
                p_min_quanta: 0,
                p_max_quanta: 3,
                cost_units_per_quantum: 4,
                carbon_grams_per_quantum: 1,
            },
        ],
        branches: vec![
            DcOpfBranch {
                id: 100,
                from_bus: 1,
                to_bus: 2,
                reactance_pu: 0.1,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_limit_mw: Some(10.0),
                angle_difference_limit_degrees: Some(30.0),
                in_service: true,
            },
            DcOpfBranch {
                id: 200,
                from_bus: 2,
                to_bus: 3,
                reactance_pu: 0.1,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_limit_mw: Some(10.0),
                angle_difference_limit_degrees: Some(30.0),
                in_service: true,
            },
            DcOpfBranch {
                id: 300,
                from_bus: 1,
                to_bus: 3,
                reactance_pu: 0.2,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_limit_mw: Some(10.0),
                angle_difference_limit_degrees: Some(30.0),
                in_service: true,
            },
        ],
        max_combinations: 100,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_bus_known_answer_has_complete_cost_carbon_frontier() {
        let report = solve_finite_dc_opf(&three_bus_known_answer_case())
            .expect("three-bus finite DC dispatch should complete");
        assert!(report.is_exact(), "{}", report.to_json());
        assert_eq!(report.expected_combinations, 16);
        assert_eq!(report.evaluated_combinations, 16);
        assert_eq!(report.feasible_assignments, 9);
        assert_eq!(report.declared_infeasible_assignments, 7);
        assert_eq!(report.solver_unknowns, 0);
        assert_eq!(report.auditor_rejections, 0);
        assert_eq!(report.frontier.len(), 4);

        let objectives = report
            .frontier
            .iter()
            .map(|dispatch| (dispatch.operating_cost_units, dispatch.carbon_grams))
            .collect::<Vec<_>>();
        assert_eq!(objectives, vec![(16, 32), (18, 23), (20, 14), (22, 5)]);
        assert_eq!(
            report.frontier[0]
                .generation
                .iter()
                .map(|entry| entry.power_quanta)
                .collect::<Vec<_>>(),
            vec![1, 3, 0]
        );
        assert_eq!(
            report.frontier[3]
                .generation
                .iter()
                .map(|entry| entry.power_quanta)
                .collect::<Vec<_>>(),
            vec![1, 0, 3]
        );
    }

    #[test]
    fn combination_budget_rejects_before_any_partial_search() {
        let mut case = three_bus_known_answer_case();
        case.max_combinations = 15;
        let error = solve_finite_dc_opf(&case)
            .expect_err("16 combinations must exceed a budget of 15");
        assert!(error.reason.contains("requires 16 combinations"));
    }

    #[test]
    fn tight_branch_limits_prove_infeasibility_not_unknown() {
        let mut case = three_bus_known_answer_case();
        case.generators[1].p_max_quanta = 1;
        case.generators[2].p_max_quanta = 0;
        for branch in &mut case.branches {
            branch.rate_a_limit_mw = Some(0.01);
        }
        let report = solve_finite_dc_opf(&case).expect("bounded enumeration completes");
        assert!(report.is_exact(), "{}", report.to_json());
        assert_eq!(report.feasible_assignments, 0);
        assert_eq!(report.frontier.len(), 0);
        assert_eq!(report.declared_infeasible_assignments, 4);
    }

    #[test]
    fn disconnected_island_is_unknown_and_blocks_exactness() {
        let mut case = three_bus_known_answer_case();
        case.branches[1].in_service = false;
        case.branches[2].in_service = false;
        let report = solve_finite_dc_opf(&case).expect("all assignments are attempted");
        assert_eq!(report.evaluated_combinations, 16);
        assert_eq!(report.solver_unknowns, 16);
        assert!(!report.is_exact());
        assert!(!report.frontier_complete);
    }

    #[test]
    fn missing_reference_generator_is_rejected() {
        let mut case = three_bus_known_answer_case();
        case.generators.remove(0);
        assert!(
            solve_finite_dc_opf(&case)
                .unwrap_err()
                .reason
                .contains("one generator at the reference bus is required")
        );
    }

    #[test]
    fn duplicate_and_unsorted_ids_fail_closed() {
        let mut case = three_bus_known_answer_case();
        case.buses.swap(0, 1);
        assert!(
            solve_finite_dc_opf(&case)
                .unwrap_err()
                .reason
                .contains("buses must be ordered")
        );
    }

    #[test]
    fn json_report_is_deterministic_and_claim_scoped() {
        let case = three_bus_known_answer_case();
        let first = solve_finite_dc_opf(&case).expect("first run");
        let second = solve_finite_dc_opf(&case).expect("second run");
        assert_eq!(first.to_json(), second.to_json());
        assert!(first.to_json().contains("\"is_exact\":true"));
        assert!(first.to_json().contains(
            "\"claim_ceiling\":\"exact_finite_quantized_fixed_injection_linear_dc_dispatch_only\""
        ));
    }
}
