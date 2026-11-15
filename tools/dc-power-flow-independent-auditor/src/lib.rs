// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Independent verifier for a small-network DC power-flow receipt.
//!
//! This crate consumes the network model and a candidate result. It does not
//! call the solver, trust the solver's feasibility flags, or reuse its internal
//! matrices. It reconstructs energized islands, slack injections, branch flows,
//! nodal balances and declared Rate A violations from the candidate bus angles.
//!
//! This checks internal consistency under the declared linear DC model. It is
//! not an AC power-flow validation, operational safety case, or grid approval.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dc_power_flow_reference::{DcNetwork, DcPowerFlowResult};

const MAX_BUSES: usize = 256;
const MAX_BRANCHES: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub struct AuditOptions {
    /// Allowed discrepancy between a candidate receipt field and its recomputed value.
    pub value_tolerance_mw: f64,
    /// Allowed discrepancy for voltage angles, in radians.
    pub angle_tolerance_rad: f64,
    /// Threshold used to recompute the candidate's nodal-tolerance flag.
    pub residual_tolerance_mw: f64,
    /// Threshold used to recompute the candidate's branch-limit flag.
    pub branch_limit_tolerance_mw: f64,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            value_tolerance_mw: 1.0e-7,
            angle_tolerance_rad: 1.0e-9,
            residual_tolerance_mw: 1.0e-7,
            branch_limit_tolerance_mw: 1.0e-7,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditReport {
    pub checked_bus_count: usize,
    pub checked_branch_count: usize,
    pub energized_island_count: usize,
    pub reference_bus_ids: Vec<i64>,
    pub max_branch_flow_error_mw: f64,
    pub max_nodal_outflow_error_mw: f64,
    pub max_abs_nodal_residual_mw: f64,
    pub max_branch_overload_mw: f64,
    pub within_nodal_tolerance: bool,
    pub within_declared_branch_limits: bool,
    pub independently_feasible_under_declared_checks: bool,
    pub claim_ceiling: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditError {
    pub reason: String,
}

impl AuditError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for AuditError {}

fn require_close(
    label: &str,
    actual: f64,
    expected: f64,
    tolerance: f64,
) -> Result<f64, AuditError> {
    if !actual.is_finite() || !expected.is_finite() {
        return Err(AuditError::new(format!(
            "{label} is non-finite: actual={actual}, expected={expected}"
        )));
    }
    let error = (actual - expected).abs();
    if error > tolerance {
        return Err(AuditError::new(format!(
            "{label} mismatch: actual={actual:.12}, expected={expected:.12}, error={error:.12}, tolerance={tolerance:.12}"
        )));
    }
    Ok(error)
}

fn connected_components(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut visited = vec![false; adjacency.len()];
    let mut components = Vec::new();

    for start in 0..adjacency.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::new();

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

fn validate_options(options: AuditOptions) -> Result<(), AuditError> {
    for (name, value) in [
        ("value_tolerance_mw", options.value_tolerance_mw),
        ("angle_tolerance_rad", options.angle_tolerance_rad),
        ("residual_tolerance_mw", options.residual_tolerance_mw),
        ("branch_limit_tolerance_mw", options.branch_limit_tolerance_mw),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(AuditError::new(format!(
                "audit option {name} must be finite and non-negative"
            )));
        }
    }
    Ok(())
}

/// Independently audit a DC power-flow result against the source network.
///
/// The reference solver's tolerance options should be passed unchanged here so
/// that the independently recomputed threshold flags have identical semantics.
pub fn audit_dc_power_flow(
    network: &DcNetwork,
    candidate: &DcPowerFlowResult,
    options: AuditOptions,
) -> Result<AuditReport, AuditError> {
    validate_options(options)?;

    if network.buses.is_empty() {
        return Err(AuditError::new("network has no buses"));
    }
    if network.buses.len() > MAX_BUSES {
        return Err(AuditError::new(format!(
            "network bus count {} exceeds audit cap {MAX_BUSES}",
            network.buses.len()
        )));
    }
    if network.branches.len() > MAX_BRANCHES {
        return Err(AuditError::new(format!(
            "network branch count {} exceeds audit cap {MAX_BRANCHES}",
            network.branches.len()
        )));
    }
    if !network.base_mva.is_finite() || network.base_mva <= 0.0 {
        return Err(AuditError::new("network base MVA must be finite and positive"));
    }
    if candidate.bus_states.len() != network.buses.len() {
        return Err(AuditError::new(format!(
            "candidate bus-state count {} differs from network bus count {}",
            candidate.bus_states.len(),
            network.buses.len()
        )));
    }
    if candidate.branch_flows.len() != network.branches.len() {
        return Err(AuditError::new(format!(
            "candidate branch-flow count {} differs from network branch count {}",
            candidate.branch_flows.len(),
            network.branches.len()
        )));
    }

    let mut bus_index = BTreeMap::new();
    for (index, bus) in network.buses.iter().enumerate() {
        if bus.id <= 0 {
            return Err(AuditError::new(format!(
                "network bus ID must be positive, got {}",
                bus.id
            )));
        }
        if bus_index.insert(bus.id, index).is_some() {
            return Err(AuditError::new(format!("duplicate network bus ID {}", bus.id)));
        }
        if let Some(injection) = bus.specified_injection_mw
            && !injection.is_finite()
        {
            return Err(AuditError::new(format!(
                "specified injection at bus {} is non-finite",
                bus.id
            )));
        }
    }

    let mut adjacency = vec![Vec::<usize>::new(); network.buses.len()];
    for (branch_index, branch) in network.branches.iter().enumerate() {
        let Some(&from) = bus_index.get(&branch.from_bus) else {
            return Err(AuditError::new(format!(
                "network branch {branch_index} has unknown from-bus {}",
                branch.from_bus
            )));
        };
        let Some(&to) = bus_index.get(&branch.to_bus) else {
            return Err(AuditError::new(format!(
                "network branch {branch_index} has unknown to-bus {}",
                branch.to_bus
            )));
        };
        if from == to {
            return Err(AuditError::new(format!(
                "network branch {branch_index} is a self-loop"
            )));
        }
        if let Some(rate) = branch.rate_a_mva
            && (!rate.is_finite() || rate < 0.0)
        {
            return Err(AuditError::new(format!(
                "network branch {branch_index} has invalid Rate A"
            )));
        }
        if branch.in_service {
            if !branch.reactance_pu.is_finite() || branch.reactance_pu <= 0.0 {
                return Err(AuditError::new(format!(
                    "network branch {branch_index} has invalid reactance"
                )));
            }
            if !branch.tap_ratio.is_finite() || branch.tap_ratio < 0.0 {
                return Err(AuditError::new(format!(
                    "network branch {branch_index} has invalid tap ratio"
                )));
            }
            if !branch.phase_shift_degrees.is_finite() {
                return Err(AuditError::new(format!(
                    "network branch {branch_index} has invalid phase shift"
                )));
            }
            let tap = if branch.tap_ratio == 0.0 {
                1.0
            } else {
                branch.tap_ratio
            };
            let denominator = branch.reactance_pu * tap;
            if !denominator.is_finite() || denominator <= 0.0 {
                return Err(AuditError::new(format!(
                    "network branch {branch_index} produces an invalid DC coefficient"
                )));
            }
            adjacency[from].push(to);
            adjacency[to].push(from);
        }
    }

    let components = connected_components(&adjacency);
    let mut reference_by_bus = vec![false; network.buses.len()];
    let mut expected_slack_by_bus = vec![None; network.buses.len()];
    let mut reference_bus_ids = Vec::with_capacity(components.len());

    for (island_index, component) in components.iter().enumerate() {
        let references = component
            .iter()
            .copied()
            .filter(|&bus_index| network.buses[bus_index].specified_injection_mw.is_none())
            .collect::<Vec<_>>();
        if references.len() != 1 {
            return Err(AuditError::new(format!(
                "island {island_index} has {} unspecified-injection reference buses; exactly one is required",
                references.len()
            )));
        }

        let reference = references[0];
        reference_by_bus[reference] = true;
        reference_bus_ids.push(network.buses[reference].id);

        let specified_sum = component
            .iter()
            .filter_map(|&bus_index| network.buses[bus_index].specified_injection_mw)
            .sum::<f64>();
        let slack_injection = -specified_sum;
        if !slack_injection.is_finite() {
            return Err(AuditError::new(format!(
                "island {island_index} slack injection is non-finite"
            )));
        }
        expected_slack_by_bus[reference] = Some(slack_injection);
    }

    if candidate.energized_island_count != components.len() {
        return Err(AuditError::new(format!(
            "candidate reports {} energized islands; topology has {}",
            candidate.energized_island_count,
            components.len()
        )));
    }

    let mut angle_by_bus = vec![0.0; network.buses.len()];
    for (index, (source_bus, state)) in network
        .buses
        .iter()
        .zip(candidate.bus_states.iter())
        .enumerate()
    {
        if state.bus_id != source_bus.id {
            return Err(AuditError::new(format!(
                "candidate bus-state row {index} identifies bus {}, expected {}",
                state.bus_id, source_bus.id
            )));
        }
        if !state.voltage_angle_rad.is_finite() {
            return Err(AuditError::new(format!(
                "candidate angle at bus {} is non-finite",
                source_bus.id
            )));
        }
        if state.is_reference_bus != reference_by_bus[index] {
            return Err(AuditError::new(format!(
                "candidate reference-bus flag is inconsistent at bus {}",
                source_bus.id
            )));
        }
        if reference_by_bus[index] {
            require_close(
                &format!("reference angle at bus {}", source_bus.id),
                state.voltage_angle_rad,
                0.0,
                options.angle_tolerance_rad,
            )?;
        }
        angle_by_bus[index] = state.voltage_angle_rad;
    }

    // Recompute every branch transfer from the candidate angle vector.
    let mut recomputed_outflow = vec![0.0_f64; network.buses.len()];
    let mut max_branch_flow_error_mw = 0.0_f64;
    let mut max_branch_overload_mw = 0.0_f64;

    for (branch_index, (branch, supplied_flow)) in network
        .branches
        .iter()
        .zip(candidate.branch_flows.iter())
        .enumerate()
    {
        if supplied_flow.branch_index != branch_index
            || supplied_flow.from_bus != branch.from_bus
            || supplied_flow.to_bus != branch.to_bus
            || supplied_flow.in_service != branch.in_service
        {
            return Err(AuditError::new(format!(
                "candidate branch-flow identity mismatch at branch {branch_index}"
            )));
        }

        let (expected_flow, expected_limit) = if branch.in_service {
            let from = bus_index[&branch.from_bus];
            let to = bus_index[&branch.to_bus];
            let tap = if branch.tap_ratio == 0.0 {
                1.0
            } else {
                branch.tap_ratio
            };
            let susceptance = 1.0 / (branch.reactance_pu * tap);
            let phase_shift_rad = branch.phase_shift_degrees.to_radians();
            let flow = susceptance
                * network.base_mva
                * (angle_by_bus[from] - angle_by_bus[to] - phase_shift_rad);
            if !flow.is_finite() || !susceptance.is_finite() {
                return Err(AuditError::new(format!(
                    "recomputed flow at branch {branch_index} is non-finite"
                )));
            }
            recomputed_outflow[from] += flow;
            recomputed_outflow[to] -= flow;
            if !recomputed_outflow[from].is_finite()
                || !recomputed_outflow[to].is_finite()
            {
                return Err(AuditError::new(format!(
                    "recomputed nodal outflow became non-finite at branch {branch_index}"
                )));
            }
            (flow, branch.rate_a_mva.filter(|rating| *rating > 0.0))
        } else {
            (0.0, None)
        };

        let flow_error = require_close(
            &format!("branch {branch_index} flow"),
            supplied_flow.flow_from_to_mw,
            expected_flow,
            options.value_tolerance_mw,
        )?;
        max_branch_flow_error_mw = max_branch_flow_error_mw.max(flow_error);

        match (supplied_flow.rate_a_limit_mw, expected_limit) {
            (Some(actual), Some(expected)) => {
                require_close(
                    &format!("branch {branch_index} Rate A"),
                    actual,
                    expected,
                    options.value_tolerance_mw,
                )?;
            }
            (None, None) => {}
            _ => {
                return Err(AuditError::new(format!(
                    "branch {branch_index} Rate A declaration mismatch"
                )));
            }
        }

        let expected_overload = expected_limit
            .map(|limit| (expected_flow.abs() - limit).max(0.0))
            .unwrap_or(0.0);
        require_close(
            &format!("branch {branch_index} overload"),
            supplied_flow.overload_mw,
            expected_overload,
            options.value_tolerance_mw,
        )?;
        max_branch_overload_mw = max_branch_overload_mw.max(expected_overload);
    }

    let mut max_nodal_outflow_error_mw = 0.0_f64;
    let mut max_abs_nodal_residual_mw = 0.0_f64;
    let mut expected_slacks = Vec::with_capacity(components.len());

    for (index, (source_bus, state)) in network
        .buses
        .iter()
        .zip(candidate.bus_states.iter())
        .enumerate()
    {
        let expected_injection = if reference_by_bus[index] {
            expected_slack_by_bus[index].expect("topology pass computed one slack per island")
        } else {
            source_bus
                .specified_injection_mw
                .expect("topology pass established non-reference injections")
        };
        let expected_outflow = recomputed_outflow[index];
        let expected_residual = expected_outflow - expected_injection;

        require_close(
            &format!("bus {} net injection", source_bus.id),
            state.net_injection_mw,
            expected_injection,
            options.value_tolerance_mw,
        )?;
        let outflow_error = require_close(
            &format!("bus {} calculated net outflow", source_bus.id),
            state.calculated_net_outflow_mw,
            expected_outflow,
            options.value_tolerance_mw,
        )?;
        max_nodal_outflow_error_mw = max_nodal_outflow_error_mw.max(outflow_error);
        require_close(
            &format!("bus {} nodal residual", source_bus.id),
            state.nodal_residual_mw,
            expected_residual,
            options.value_tolerance_mw,
        )?;
        max_abs_nodal_residual_mw =
            max_abs_nodal_residual_mw.max(expected_residual.abs());

        if reference_by_bus[index] {
            expected_slacks.push((source_bus.id, expected_injection));
        }
    }

    if candidate.slack_injections.len() != expected_slacks.len() {
        return Err(AuditError::new(format!(
            "candidate slack-injection count {} differs from topology reference count {}",
            candidate.slack_injections.len(),
            expected_slacks.len()
        )));
    }

    for (index, (supplied, (expected_bus, expected_injection))) in candidate
        .slack_injections
        .iter()
        .zip(expected_slacks.iter())
        .enumerate()
    {
        if supplied.bus_id != *expected_bus || supplied.island_index != index {
            return Err(AuditError::new(format!(
                "candidate slack record {index} has inconsistent bus/island identity"
            )));
        }
        require_close(
            &format!("slack injection at bus {}", expected_bus),
            supplied.injection_mw,
            *expected_injection,
            options.value_tolerance_mw,
        )?;
    }

    let within_nodal_tolerance =
        max_abs_nodal_residual_mw <= options.residual_tolerance_mw;
    let within_declared_branch_limits =
        max_branch_overload_mw <= options.branch_limit_tolerance_mw;
    let independently_feasible_under_declared_checks =
        within_nodal_tolerance && within_declared_branch_limits;

    if candidate.within_nodal_tolerance != within_nodal_tolerance {
        return Err(AuditError::new(
            "candidate within_nodal_tolerance flag does not match independent recomputation",
        ));
    }
    if candidate.within_declared_branch_limits != within_declared_branch_limits {
        return Err(AuditError::new(
            "candidate within_declared_branch_limits flag does not match independent recomputation",
        ));
    }
    if candidate.feasible_under_declared_checks != independently_feasible_under_declared_checks {
        return Err(AuditError::new(
            "candidate feasibility flag does not match independent recomputation",
        ));
    }

    Ok(AuditReport {
        checked_bus_count: network.buses.len(),
        checked_branch_count: network.branches.len(),
        energized_island_count: components.len(),
        reference_bus_ids,
        max_branch_flow_error_mw,
        max_nodal_outflow_error_mw,
        max_abs_nodal_residual_mw,
        max_branch_overload_mw,
        within_nodal_tolerance,
        within_declared_branch_limits,
        independently_feasible_under_declared_checks,
        claim_ceiling: "independent_dc_model_receipt_consistency_only",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_power_flow_reference::{
        DcBranch, DcBus, DcPowerFlowResult, SolveOptions, solve_dc_power_flow,
    };

    fn two_bus(rate: Option<f64>) -> DcNetwork {
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
                rate_a_mva: rate,
                in_service: true,
            }],
        }
    }

    fn solve(network: &DcNetwork) -> DcPowerFlowResult {
        solve_dc_power_flow(network, SolveOptions::default()).expect("reference solve")
    }

    #[test]
    fn accepts_hand_calculated_two_bus_reference_result() {
        let network = two_bus(None);
        let result = solve(&network);
        let report = audit_dc_power_flow(&network, &result, AuditOptions::default()).unwrap();
        assert_eq!(report.checked_bus_count, 2);
        assert_eq!(report.checked_branch_count, 1);
        assert_eq!(report.reference_bus_ids, vec![1]);
        assert!(report.independently_feasible_under_declared_checks);
    }

    #[test]
    fn accepts_three_bus_mesh_with_counter_flow() {
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
        let result = solve(&network);
        assert!((result.branch_flows[1].flow_from_to_mw + 10.0).abs() < 1.0e-9);
        let report = audit_dc_power_flow(&network, &result, AuditOptions::default()).unwrap();
        assert_eq!(report.checked_bus_count, 3);
        assert!(report.independently_feasible_under_declared_checks);
    }

    #[test]
    fn rejects_mutated_branch_flow_receipt() {
        let network = two_bus(None);
        let mut result = solve(&network);
        result.branch_flows[0].flow_from_to_mw += 1.0;
        assert!(audit_dc_power_flow(&network, &result, AuditOptions::default())
            .unwrap_err()
            .reason
            .contains("branch 0 flow mismatch"));
    }

    #[test]
    fn rejects_forged_slack_injection() {
        let network = two_bus(None);
        let mut result = solve(&network);
        result.slack_injections[0].injection_mw += 5.0;
        assert!(audit_dc_power_flow(&network, &result, AuditOptions::default())
            .unwrap_err()
            .reason
            .contains("slack injection"));
    }

    #[test]
    fn rejects_forged_feasibility_flag_on_overloaded_branch() {
        let network = two_bus(Some(40.0));
        let mut result = solve(&network);
        assert!(!result.feasible_under_declared_checks);
        result.within_declared_branch_limits = true;
        result.feasible_under_declared_checks = true;
        assert!(audit_dc_power_flow(&network, &result, AuditOptions::default())
            .unwrap_err()
            .reason
            .contains("within_declared_branch_limits flag"));
    }

    #[test]
    fn rejects_changed_network_branch_identity() {
        let network = two_bus(None);
        let mut result = solve(&network);
        result.branch_flows[0].to_bus = 3;
        assert!(audit_dc_power_flow(&network, &result, AuditOptions::default())
            .unwrap_err()
            .reason
            .contains("branch-flow identity mismatch"));
    }

    #[test]
    fn rejects_forged_bus_balance_fields() {
        let network = two_bus(None);
        let mut result = solve(&network);
        result.bus_states[1].calculated_net_outflow_mw += 2.0;
        assert!(audit_dc_power_flow(&network, &result, AuditOptions::default())
            .unwrap_err()
            .reason
            .contains("calculated net outflow mismatch"));
    }

    #[test]
    fn rejects_inconsistent_island_and_reference_metadata() {
        let network = two_bus(None);
        let mut result = solve(&network);
        result.bus_states[0].is_reference_bus = false;
        assert!(audit_dc_power_flow(&network, &result, AuditOptions::default())
            .unwrap_err()
            .reason
            .contains("reference-bus flag"));
    }
}
