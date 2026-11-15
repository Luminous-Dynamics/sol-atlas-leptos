// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Small-network DC power-flow reference kernel.
//!
//! This is a linear active-power feasibility model, not AC power flow or OPF.
//! It intentionally has no generator-cost objective, voltage-magnitude or
//! reactive-power equations. Use it as a reference checker for tiny cases and
//! differential tests, not as a grid-operations controller.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

const MAX_BUSES: usize = 256;
const MAX_BRANCHES: usize = 4096;
const PIVOT_RELATIVE_TOLERANCE: f64 = 1.0e-12;

#[derive(Clone, Debug)]
pub struct DcBus {
    /// Positive MATPOWER-style bus identifier.
    pub id: i64,
    /// Specified net injection in MW (generation positive, load negative).
    /// Exactly one bus per energized island must be None; its injection is
    /// the island slack balancing value computed by the solver.
    pub specified_injection_mw: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct DcBranch {
    pub from_bus: i64,
    pub to_bus: i64,
    /// Series reactance in per unit on the network base MVA.
    pub reactance_pu: f64,
    /// MATPOWER convention: zero means a unity tap; positive values are used.
    pub tap_ratio: f64,
    /// Transformer phase shift in degrees, positive in the from-to orientation.
    pub phase_shift_degrees: f64,
    /// MATPOWER Rate A in MVA. None or zero means no declared limit.
    /// The DC approximation compares absolute MW flow to this value.
    pub rate_a_mva: Option<f64>,
    pub in_service: bool,
}

#[derive(Clone, Debug)]
pub struct DcNetwork {
    pub base_mva: f64,
    pub buses: Vec<DcBus>,
    pub branches: Vec<DcBranch>,
}

#[derive(Clone, Copy, Debug)]
pub struct SolveOptions {
    /// Maximum accepted absolute nodal balance residual, in MW.
    pub residual_tolerance_mw: f64,
    /// Numerical tolerance for declaring a branch thermal-limit violation, MW.
    pub branch_limit_tolerance_mw: f64,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            residual_tolerance_mw: 1.0e-7,
            branch_limit_tolerance_mw: 1.0e-7,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BusState {
    pub bus_id: i64,
    pub voltage_angle_rad: f64,
    pub net_injection_mw: f64,
    pub calculated_net_outflow_mw: f64,
    pub nodal_residual_mw: f64,
    pub is_reference_bus: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BranchFlow {
    pub branch_index: usize,
    pub from_bus: i64,
    pub to_bus: i64,
    pub in_service: bool,
    /// Positive flow is from from_bus to to_bus.
    pub flow_from_to_mw: f64,
    /// None means there is no declared Rate A limit; zero is also unlimited.
    pub rate_a_limit_mw: Option<f64>,
    /// Positive values represent overload; zero means no violation.
    pub overload_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlackInjection {
    pub bus_id: i64,
    pub island_index: usize,
    pub injection_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DcPowerFlowResult {
    pub bus_states: Vec<BusState>,
    pub branch_flows: Vec<BranchFlow>,
    pub slack_injections: Vec<SlackInjection>,
    pub energized_island_count: usize,
    pub max_abs_nodal_residual_mw: f64,
    pub max_branch_overload_mw: f64,
    /// False when a declared branch Rate A limit is exceeded. A solved linear
    /// power flow can exist while the dispatch is thermally infeasible.
    pub within_declared_branch_limits: bool,
    pub within_nodal_tolerance: bool,
    pub feasible_under_declared_checks: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DcPowerFlowError {
    EmptyNetwork,
    BusBudgetExceeded { count: usize, maximum: usize },
    BranchBudgetExceeded { count: usize, maximum: usize },
    InvalidBaseMva,
    InvalidSolveTolerance,
    InvalidBusId { bus_id: i64 },
    DuplicateBusId { bus_id: i64 },
    NonFiniteBusInjection { bus_id: i64 },
    IslandMustHaveExactlyOneReference { island_index: usize, count: usize },
    InvalidBranchEndpoint { branch_index: usize, bus_id: i64 },
    SelfLoopBranch { branch_index: usize },
    InvalidReactance { branch_index: usize },
    InvalidTapRatio { branch_index: usize },
    InvalidPhaseShift { branch_index: usize },
    InvalidRateA { branch_index: usize },
    NonFiniteNetworkCoefficient { branch_index: usize },
    SingularNetworkMatrix,
    NonFiniteSolution,
}

impl std::fmt::Display for DcPowerFlowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyNetwork => write!(formatter, "network has no buses"),
            Self::BusBudgetExceeded { count, maximum } => {
                write!(
                    formatter,
                    "bus count {count} exceeds reference-kernel cap {maximum}"
                )
            }
            Self::BranchBudgetExceeded { count, maximum } => {
                write!(
                    formatter,
                    "branch count {count} exceeds reference-kernel cap {maximum}"
                )
            }
            Self::InvalidBaseMva => write!(formatter, "base MVA must be finite and positive"),
            Self::InvalidSolveTolerance => write!(
                formatter,
                "solve tolerances must be finite and non-negative"
            ),
            Self::InvalidBusId { bus_id } => {
                write!(formatter, "bus ID must be positive, got {bus_id}")
            }
            Self::DuplicateBusId { bus_id } => write!(formatter, "duplicate bus ID {bus_id}"),
            Self::NonFiniteBusInjection { bus_id } => {
                write!(formatter, "non-finite specified injection at bus {bus_id}")
            }
            Self::IslandMustHaveExactlyOneReference {
                island_index,
                count,
            } => write!(
                formatter,
                "energized island {island_index} has {count} reference buses; exactly one is required"
            ),
            Self::InvalidBranchEndpoint {
                branch_index,
                bus_id,
            } => write!(
                formatter,
                "branch {branch_index} references unknown bus {bus_id}"
            ),
            Self::SelfLoopBranch { branch_index } => {
                write!(formatter, "branch {branch_index} connects a bus to itself")
            }
            Self::InvalidReactance { branch_index } => {
                write!(
                    formatter,
                    "branch {branch_index} requires finite positive reactance"
                )
            }
            Self::InvalidTapRatio { branch_index } => {
                write!(
                    formatter,
                    "branch {branch_index} tap ratio must be finite and non-negative"
                )
            }
            Self::InvalidPhaseShift { branch_index } => {
                write!(
                    formatter,
                    "branch {branch_index} phase shift must be finite"
                )
            }
            Self::InvalidRateA { branch_index } => {
                write!(
                    formatter,
                    "branch {branch_index} Rate A must be finite and non-negative"
                )
            }
            Self::NonFiniteNetworkCoefficient { branch_index } => {
                write!(
                    formatter,
                    "branch {branch_index} produces a non-finite DC coefficient"
                )
            }
            Self::SingularNetworkMatrix => write!(
                formatter,
                "reduced DC network matrix is singular or ill-conditioned"
            ),
            Self::NonFiniteSolution => write!(
                formatter,
                "DC power-flow solution contains non-finite values"
            ),
        }
    }
}

impl std::error::Error for DcPowerFlowError {}

/// Solve the linear DC power-flow equations for a fixed network and injection set.
///
/// Sign convention: positive specified injection is generation; negative is load.
/// One unspecified-injection reference bus per energized island balances that
/// island. The solver fixes each reference angle at zero and reports its balancing
/// injection from the recomputed branch flows.
///
/// For an in-service branch from f to t:
/// P_ft = baseMVA / (x * tap) * (theta_f - theta_t - phase_shift_rad).
///
/// Rate A is applied to the magnitude of active MW flow, the usual DC-OPF
/// approximation to an MVA branch limit. No AC voltage, reactive-power, loss,
/// contingency, generator-capability or OPF objective is modeled.
pub fn solve_dc_power_flow(
    network: &DcNetwork,
    options: SolveOptions,
) -> Result<DcPowerFlowResult, DcPowerFlowError> {
    validate_input(network, options)?;

    let bus_count = network.buses.len();
    let mut bus_index = BTreeMap::new();
    for (index, bus) in network.buses.iter().enumerate() {
        bus_index.insert(bus.id, index);
    }

    let mut adjacency = vec![Vec::<usize>::new(); bus_count];
    for (branch_index, branch) in network.branches.iter().enumerate() {
        let from =
            *bus_index
                .get(&branch.from_bus)
                .ok_or(DcPowerFlowError::InvalidBranchEndpoint {
                    branch_index,
                    bus_id: branch.from_bus,
                })?;
        let to = *bus_index
            .get(&branch.to_bus)
            .ok_or(DcPowerFlowError::InvalidBranchEndpoint {
                branch_index,
                bus_id: branch.to_bus,
            })?;
        if from == to {
            return Err(DcPowerFlowError::SelfLoopBranch { branch_index });
        }
        if branch.in_service {
            adjacency[from].push(to);
            adjacency[to].push(from);
        }
    }

    let components = connected_components(&adjacency);
    let mut reference_by_bus = vec![false; bus_count];
    let mut slack_injection_by_bus = vec![None; bus_count];
    for (island_index, component) in components.iter().enumerate() {
        let references = component
            .iter()
            .copied()
            .filter(|&index| network.buses[index].specified_injection_mw.is_none())
            .collect::<Vec<_>>();
        if references.len() != 1 {
            return Err(DcPowerFlowError::IslandMustHaveExactlyOneReference {
                island_index,
                count: references.len(),
            });
        }
        let reference = references[0];
        reference_by_bus[reference] = true;

        // Compute the slack injection from specified injections alone. The
        // later branch-flow calculation independently audits this balance.
        let specified_sum = component
            .iter()
            .filter_map(|&index| network.buses[index].specified_injection_mw)
            .sum::<f64>();
        let slack_injection = -specified_sum;
        if !slack_injection.is_finite() {
            return Err(DcPowerFlowError::NonFiniteSolution);
        }
        slack_injection_by_bus[reference] = Some(slack_injection);
    }

    let mut reduced_index = vec![None; bus_count];
    let mut reduced_bus_indices = Vec::with_capacity(bus_count - components.len());
    for bus_index in 0..bus_count {
        if !reference_by_bus[bus_index] {
            reduced_index[bus_index] = Some(reduced_bus_indices.len());
            reduced_bus_indices.push(bus_index);
        }
    }

    let reduced_size = reduced_bus_indices.len();
    let mut matrix = vec![vec![0.0_f64; reduced_size]; reduced_size];
    let mut rhs = vec![0.0_f64; reduced_size];

    for (reduced, &bus_idx) in reduced_bus_indices.iter().enumerate() {
        let specified = network.buses[bus_idx]
            .specified_injection_mw
            .expect("non-reference buses have specified injections");
        rhs[reduced] = specified / network.base_mva;
    }

    for (branch_index, branch) in network.branches.iter().enumerate() {
        if !branch.in_service {
            continue;
        }
        let from = bus_index[&branch.from_bus];
        let to = bus_index[&branch.to_bus];
        let tap = if branch.tap_ratio == 0.0 {
            1.0
        } else {
            branch.tap_ratio
        };
        let susceptance = 1.0 / (branch.reactance_pu * tap);
        let phase_shift_rad = branch.phase_shift_degrees.to_radians();
        let phase_injection = susceptance * phase_shift_rad;
        if !susceptance.is_finite() || !phase_injection.is_finite() {
            return Err(DcPowerFlowError::NonFiniteNetworkCoefficient { branch_index });
        }

        if let Some(row) = reduced_index[from] {
            matrix[row][row] += susceptance;
            rhs[row] += phase_injection;
        }
        if let Some(row) = reduced_index[to] {
            matrix[row][row] += susceptance;
            rhs[row] -= phase_injection;
        }
        if let (Some(from_row), Some(to_row)) = (reduced_index[from], reduced_index[to]) {
            matrix[from_row][to_row] -= susceptance;
            matrix[to_row][from_row] -= susceptance;
        }
    }

    let reduced_angles = solve_dense_system(matrix, rhs)?;
    let mut angles = vec![0.0_f64; bus_count];
    for (reduced, &bus_idx) in reduced_bus_indices.iter().enumerate() {
        angles[bus_idx] = reduced_angles[reduced];
    }

    let mut net_outflow = vec![0.0_f64; bus_count];
    let mut branch_flows = Vec::with_capacity(network.branches.len());
    let mut max_branch_overload_mw = 0.0_f64;

    for (branch_index, branch) in network.branches.iter().enumerate() {
        let (flow, rate_limit) = if branch.in_service {
            let from = bus_index[&branch.from_bus];
            let to = bus_index[&branch.to_bus];
            let tap = if branch.tap_ratio == 0.0 {
                1.0
            } else {
                branch.tap_ratio
            };
            let susceptance = 1.0 / (branch.reactance_pu * tap);
            let phase_shift_rad = branch.phase_shift_degrees.to_radians();
            let value =
                susceptance * network.base_mva * (angles[from] - angles[to] - phase_shift_rad);
            let limit = branch.rate_a_mva.filter(|rating| *rating > 0.0);
            if !value.is_finite() {
                return Err(DcPowerFlowError::NonFiniteSolution);
            }
            net_outflow[from] += value;
            net_outflow[to] -= value;
            if !net_outflow[from].is_finite() || !net_outflow[to].is_finite() {
                return Err(DcPowerFlowError::NonFiniteSolution);
            }
            (value, limit)
        } else {
            (0.0, None)
        };

        let overload = rate_limit
            .map(|limit| (flow.abs() - limit).max(0.0))
            .unwrap_or(0.0);
        max_branch_overload_mw = max_branch_overload_mw.max(overload);
        branch_flows.push(BranchFlow {
            branch_index,
            from_bus: branch.from_bus,
            to_bus: branch.to_bus,
            in_service: branch.in_service,
            flow_from_to_mw: flow,
            rate_a_limit_mw: rate_limit,
            overload_mw: overload,
        });
    }

    let island_for_bus = component_index_by_bus(bus_count, &components);
    let mut bus_states = Vec::with_capacity(bus_count);
    let mut slack_injections = Vec::with_capacity(components.len());
    let mut max_abs_nodal_residual_mw = 0.0_f64;

    for (index, bus) in network.buses.iter().enumerate() {
        let is_reference = reference_by_bus[index];
        let injection = if is_reference {
            slack_injection_by_bus[index].expect("each island has one computed slack injection")
        } else {
            bus.specified_injection_mw
                .expect("validated non-reference injection")
        };
        let residual = net_outflow[index] - injection;
        if !injection.is_finite()
            || !net_outflow[index].is_finite()
            || !residual.is_finite()
            || !angles[index].is_finite()
        {
            return Err(DcPowerFlowError::NonFiniteSolution);
        }
        max_abs_nodal_residual_mw = max_abs_nodal_residual_mw.max(residual.abs());

        if is_reference {
            slack_injections.push(SlackInjection {
                bus_id: bus.id,
                island_index: island_for_bus[index],
                injection_mw: injection,
            });
        }

        bus_states.push(BusState {
            bus_id: bus.id,
            voltage_angle_rad: angles[index],
            net_injection_mw: injection,
            calculated_net_outflow_mw: net_outflow[index],
            nodal_residual_mw: residual,
            is_reference_bus: is_reference,
        });
    }

    let within_nodal_tolerance = max_abs_nodal_residual_mw <= options.residual_tolerance_mw;
    let within_declared_branch_limits = max_branch_overload_mw <= options.branch_limit_tolerance_mw;
    Ok(DcPowerFlowResult {
        bus_states,
        branch_flows,
        slack_injections,
        energized_island_count: components.len(),
        max_abs_nodal_residual_mw,
        max_branch_overload_mw,
        within_declared_branch_limits,
        within_nodal_tolerance,
        feasible_under_declared_checks: within_nodal_tolerance && within_declared_branch_limits,
    })
}

fn validate_input(network: &DcNetwork, options: SolveOptions) -> Result<(), DcPowerFlowError> {
    if network.buses.is_empty() {
        return Err(DcPowerFlowError::EmptyNetwork);
    }
    if network.buses.len() > MAX_BUSES {
        return Err(DcPowerFlowError::BusBudgetExceeded {
            count: network.buses.len(),
            maximum: MAX_BUSES,
        });
    }
    if network.branches.len() > MAX_BRANCHES {
        return Err(DcPowerFlowError::BranchBudgetExceeded {
            count: network.branches.len(),
            maximum: MAX_BRANCHES,
        });
    }
    if !network.base_mva.is_finite() || network.base_mva <= 0.0 {
        return Err(DcPowerFlowError::InvalidBaseMva);
    }
    if !options.residual_tolerance_mw.is_finite()
        || options.residual_tolerance_mw < 0.0
        || !options.branch_limit_tolerance_mw.is_finite()
        || options.branch_limit_tolerance_mw < 0.0
    {
        return Err(DcPowerFlowError::InvalidSolveTolerance);
    }

    let mut ids = BTreeSet::new();
    for bus in &network.buses {
        if bus.id <= 0 {
            return Err(DcPowerFlowError::InvalidBusId { bus_id: bus.id });
        }
        if !ids.insert(bus.id) {
            return Err(DcPowerFlowError::DuplicateBusId { bus_id: bus.id });
        }
        if let Some(injection) = bus.specified_injection_mw
            && !injection.is_finite()
        {
            return Err(DcPowerFlowError::NonFiniteBusInjection { bus_id: bus.id });
        }
    }

    let bus_ids = ids;
    for (branch_index, branch) in network.branches.iter().enumerate() {
        for bus_id in [branch.from_bus, branch.to_bus] {
            if !bus_ids.contains(&bus_id) {
                return Err(DcPowerFlowError::InvalidBranchEndpoint {
                    branch_index,
                    bus_id,
                });
            }
        }
        if branch.from_bus == branch.to_bus {
            return Err(DcPowerFlowError::SelfLoopBranch { branch_index });
        }
        if let Some(rate) = branch.rate_a_mva
            && (!rate.is_finite() || rate < 0.0)
        {
            return Err(DcPowerFlowError::InvalidRateA { branch_index });
        }
        if branch.in_service {
            if !branch.reactance_pu.is_finite() || branch.reactance_pu <= 0.0 {
                return Err(DcPowerFlowError::InvalidReactance { branch_index });
            }
            if !branch.tap_ratio.is_finite() || branch.tap_ratio < 0.0 {
                return Err(DcPowerFlowError::InvalidTapRatio { branch_index });
            }
            if !branch.phase_shift_degrees.is_finite() {
                return Err(DcPowerFlowError::InvalidPhaseShift { branch_index });
            }
            let tap = if branch.tap_ratio == 0.0 {
                1.0
            } else {
                branch.tap_ratio
            };
            if !(branch.reactance_pu * tap).is_finite() || branch.reactance_pu * tap <= 0.0 {
                return Err(DcPowerFlowError::NonFiniteNetworkCoefficient { branch_index });
            }
        }
    }
    Ok(())
}

fn connected_components(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut visited = vec![false; adjacency.len()];
    let mut components = Vec::new();

    for start in 0..adjacency.len() {
        if visited[start] {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::new();
        visited[start] = true;

        while let Some(bus) = queue.pop_front() {
            component.push(bus);
            for &neighbor in &adjacency[bus] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    components
}

fn component_index_by_bus(bus_count: usize, components: &[Vec<usize>]) -> Vec<usize> {
    let mut result = vec![0; bus_count];
    for (component_index, component) in components.iter().enumerate() {
        for &bus in component {
            result[bus] = component_index;
        }
    }
    result
}

fn solve_dense_system(
    mut matrix: Vec<Vec<f64>>,
    mut rhs: Vec<f64>,
) -> Result<Vec<f64>, DcPowerFlowError> {
    let size = rhs.len();
    if size == 0 {
        return Ok(Vec::new());
    }

    for pivot_col in 0..size {
        let mut pivot_row = pivot_col;
        for (row_index, row) in matrix.iter().enumerate().skip(pivot_col + 1) {
            if row[pivot_col].abs() > matrix[pivot_row][pivot_col].abs() {
                pivot_row = row_index;
            }
        }
        let pivot_abs = matrix[pivot_row][pivot_col].abs();
        let row_scale = matrix[pivot_row]
            .iter()
            .skip(pivot_col)
            .fold(0.0_f64, |scale, value| scale.max(value.abs()));
        if !pivot_abs.is_finite()
            || row_scale == 0.0
            || pivot_abs <= PIVOT_RELATIVE_TOLERANCE * row_scale
        {
            return Err(DcPowerFlowError::SingularNetworkMatrix);
        }

        if pivot_row != pivot_col {
            matrix.swap(pivot_row, pivot_col);
            rhs.swap(pivot_row, pivot_col);
        }

        let pivot = matrix[pivot_col][pivot_col];
        let pivot_values = matrix[pivot_col].clone();
        let pivot_rhs = rhs[pivot_col];

        for (row_index, row) in matrix.iter_mut().enumerate().skip(pivot_col + 1) {
            let factor = row[pivot_col] / pivot;
            row[pivot_col] = 0.0;
            for (column_index, value) in row.iter_mut().enumerate().skip(pivot_col + 1) {
                *value -= factor * pivot_values[column_index];
            }
            rhs[row_index] -= factor * pivot_rhs;
        }
    }

    let mut solution = vec![0.0_f64; size];
    for row_index in (0..size).rev() {
        let tail = matrix[row_index]
            .iter()
            .enumerate()
            .skip(row_index + 1)
            .map(|(column_index, coefficient)| coefficient * solution[column_index])
            .sum::<f64>();
        let value = (rhs[row_index] - tail) / matrix[row_index][row_index];
        if !value.is_finite() {
            return Err(DcPowerFlowError::NonFiniteSolution);
        }
        solution[row_index] = value;
    }
    Ok(solution)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_bus(rate_a_mva: Option<f64>) -> DcNetwork {
        DcNetwork {
            base_mva: 100.0,
            buses: vec![
                DcBus {
                    id: 1,
                    specified_injection_mw: None,
                },
                DcBus {
                    id: 2,
                    specified_injection_mw: Some(-50.0),
                },
            ],
            branches: vec![DcBranch {
                from_bus: 1,
                to_bus: 2,
                reactance_pu: 0.1,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_mva,
                in_service: true,
            }],
        }
    }

    #[test]
    fn two_bus_power_flow_balances_and_reports_slack() {
        let result = solve_dc_power_flow(&two_bus(None), SolveOptions::default()).unwrap();
        assert!(result.feasible_under_declared_checks);
        assert_eq!(result.energized_island_count, 1);
        assert!((result.branch_flows[0].flow_from_to_mw - 50.0).abs() < 1.0e-9);
        assert!((result.bus_states[1].voltage_angle_rad + 0.05).abs() < 1.0e-9);
        assert!((result.slack_injections[0].injection_mw - 50.0).abs() < 1.0e-9);
        assert!(result.max_abs_nodal_residual_mw < 1.0e-9);
    }

    #[test]
    fn rate_a_overload_is_reported_without_hiding_the_solved_flow() {
        let result = solve_dc_power_flow(&two_bus(Some(40.0)), SolveOptions::default()).unwrap();
        assert!(!result.within_declared_branch_limits);
        assert!(!result.feasible_under_declared_checks);
        assert!((result.branch_flows[0].overload_mw - 10.0).abs() < 1.0e-9);
        assert!((result.branch_flows[0].flow_from_to_mw - 50.0).abs() < 1.0e-9);
    }

    #[test]
    fn three_bus_triangle_matches_hand_calculated_counter_flows() {
        let network = DcNetwork {
            base_mva: 100.0,
            buses: vec![
                DcBus {
                    id: 1,
                    specified_injection_mw: None,
                },
                DcBus {
                    id: 2,
                    specified_injection_mw: Some(-50.0),
                },
                DcBus {
                    id: 3,
                    specified_injection_mw: Some(0.0),
                },
            ],
            branches: vec![
                DcBranch {
                    from_bus: 1,
                    to_bus: 2,
                    reactance_pu: 0.1,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: None,
                    in_service: true,
                },
                DcBranch {
                    from_bus: 2,
                    to_bus: 3,
                    reactance_pu: 0.2,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: None,
                    in_service: true,
                },
                DcBranch {
                    from_bus: 1,
                    to_bus: 3,
                    reactance_pu: 0.2,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: None,
                    in_service: true,
                },
            ],
        };
        let result = solve_dc_power_flow(&network, SolveOptions::default()).unwrap();

        assert!(result.feasible_under_declared_checks);
        assert_eq!(result.energized_island_count, 1);
        assert!((result.bus_states[1].voltage_angle_rad + 0.04).abs() < 1.0e-9);
        assert!((result.bus_states[2].voltage_angle_rad + 0.02).abs() < 1.0e-9);
        assert!((result.branch_flows[0].flow_from_to_mw - 40.0).abs() < 1.0e-9);
        assert!((result.branch_flows[1].flow_from_to_mw + 10.0).abs() < 1.0e-9);
        assert!((result.branch_flows[2].flow_from_to_mw - 10.0).abs() < 1.0e-9);
        assert!((result.slack_injections[0].injection_mw - 50.0).abs() < 1.0e-9);
        assert!(result.max_abs_nodal_residual_mw < 1.0e-9);
    }

    #[test]
    fn zero_tap_ratio_uses_unity_transformer_ratio() {
        let zero_tap = solve_dc_power_flow(&two_bus(None), SolveOptions::default()).unwrap();
        let mut explicit_unity = two_bus(None);
        explicit_unity.branches[0].tap_ratio = 1.0;
        let unity = solve_dc_power_flow(&explicit_unity, SolveOptions::default()).unwrap();
        assert!(
            (zero_tap.branch_flows[0].flow_from_to_mw - unity.branch_flows[0].flow_from_to_mw)
                .abs()
                < 1.0e-12
        );
    }

    #[test]
    fn non_unity_tap_changes_angle_but_not_required_power_transfer() {
        let mut network = two_bus(None);
        network.branches[0].tap_ratio = 2.0;
        let result = solve_dc_power_flow(&network, SolveOptions::default()).unwrap();
        assert!((result.branch_flows[0].flow_from_to_mw - 50.0).abs() < 1.0e-9);
        assert!((result.bus_states[1].voltage_angle_rad + 0.1).abs() < 1.0e-9);
    }

    #[test]
    fn phase_shift_is_applied_with_from_to_sign_convention() {
        let mut network = DcNetwork {
            base_mva: 100.0,
            buses: vec![
                DcBus {
                    id: 1,
                    specified_injection_mw: None,
                },
                DcBus {
                    id: 2,
                    specified_injection_mw: Some(0.0),
                },
            ],
            branches: vec![DcBranch {
                from_bus: 1,
                to_bus: 2,
                reactance_pu: 0.1,
                tap_ratio: 0.0,
                phase_shift_degrees: 5.729577951308232,
                rate_a_mva: None,
                in_service: true,
            }],
        };
        let result = solve_dc_power_flow(&network, SolveOptions::default()).unwrap();
        assert!(result.feasible_under_declared_checks);
        assert!(result.branch_flows[0].flow_from_to_mw.abs() < 1.0e-9);
        assert!((result.bus_states[1].voltage_angle_rad + 0.1).abs() < 1.0e-9);
        network.branches[0].phase_shift_degrees = -5.729577951308232;
        let reversed = solve_dc_power_flow(&network, SolveOptions::default()).unwrap();
        assert!(reversed.branch_flows[0].flow_from_to_mw.abs() < 1.0e-9);
        assert!((reversed.bus_states[1].voltage_angle_rad - 0.1).abs() < 1.0e-9);
    }

    #[test]
    fn disconnected_islands_receive_independent_slack_injections() {
        let network = DcNetwork {
            base_mva: 100.0,
            buses: vec![
                DcBus {
                    id: 1,
                    specified_injection_mw: None,
                },
                DcBus {
                    id: 2,
                    specified_injection_mw: Some(-25.0),
                },
                DcBus {
                    id: 3,
                    specified_injection_mw: None,
                },
                DcBus {
                    id: 4,
                    specified_injection_mw: Some(-10.0),
                },
            ],
            branches: vec![
                DcBranch {
                    from_bus: 1,
                    to_bus: 2,
                    reactance_pu: 0.1,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: None,
                    in_service: true,
                },
                DcBranch {
                    from_bus: 3,
                    to_bus: 4,
                    reactance_pu: 0.2,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: None,
                    in_service: true,
                },
            ],
        };
        let result = solve_dc_power_flow(&network, SolveOptions::default()).unwrap();
        assert_eq!(result.energized_island_count, 2);
        assert_eq!(result.slack_injections.len(), 2);
        assert!((result.slack_injections[0].injection_mw - 25.0).abs() < 1.0e-9);
        assert!((result.slack_injections[1].injection_mw - 10.0).abs() < 1.0e-9);
    }

    #[test]
    fn multiple_reference_buses_in_one_island_are_rejected() {
        let mut network = two_bus(None);
        network.buses[1].specified_injection_mw = None;
        assert!(matches!(
            solve_dc_power_flow(&network, SolveOptions::default()),
            Err(DcPowerFlowError::IslandMustHaveExactlyOneReference { count: 2, .. })
        ));
    }

    #[test]
    fn out_of_service_branch_does_not_connect_islands() {
        let mut network = two_bus(None);
        network.branches[0].in_service = false;
        assert!(matches!(
            solve_dc_power_flow(&network, SolveOptions::default()),
            Err(DcPowerFlowError::IslandMustHaveExactlyOneReference { count: 0, .. })
        ));
    }

    #[test]
    fn unknown_branch_bus_is_rejected() {
        let mut network = two_bus(None);
        network.branches[0].to_bus = 99;
        assert!(matches!(
            solve_dc_power_flow(&network, SolveOptions::default()),
            Err(DcPowerFlowError::InvalidBranchEndpoint { bus_id: 99, .. })
        ));
    }

    #[test]
    fn zero_reactance_is_rejected() {
        let mut network = two_bus(None);
        network.branches[0].reactance_pu = 0.0;
        assert!(matches!(
            solve_dc_power_flow(&network, SolveOptions::default()),
            Err(DcPowerFlowError::InvalidReactance { .. })
        ));
    }

    #[test]
    fn duplicate_bus_id_is_rejected() {
        let mut network = two_bus(None);
        network.buses[1].id = 1;
        assert!(matches!(
            solve_dc_power_flow(&network, SolveOptions::default()),
            Err(DcPowerFlowError::DuplicateBusId { bus_id: 1 })
        ));
    }

    #[test]
    fn non_finite_injection_is_rejected() {
        let mut network = two_bus(None);
        network.buses[1].specified_injection_mw = Some(f64::NAN);
        assert!(matches!(
            solve_dc_power_flow(&network, SolveOptions::default()),
            Err(DcPowerFlowError::NonFiniteBusInjection { bus_id: 2 })
        ));
    }
}
