// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PTDF/LODF single-branch outage screening for the bounded linear DC model.
//!
//! One reduced susceptance matrix is factorized per energized island. Each
//! in-service branch defines one unit transaction. Bridge and near-bridge
//! outages abstain and must be escalated to an exact re-solve.
//!
//! This is a fast screening approximation, not DC-OPF, AC security analysis,
//! physical reliability evidence, or an operational protection/control path.

use std::collections::{BTreeMap, VecDeque};

use dc_power_flow_reference::{DcBus, DcNetwork, DcPowerFlowResult, SolveOptions, solve_dc_power_flow};

#[derive(Clone, Copy, Debug)]
pub struct ScreenOptions {
    /// Outages with 1 - PTDF[k,k] at or below this value require exact re-solving.
    pub lodf_denominator_tolerance: f64,
}

impl Default for ScreenOptions {
    fn default() -> Self {
        Self {
            lodf_denominator_tolerance: 1.0e-10,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenError {
    InvalidDenominatorTolerance,
    InvalidFlowTolerance,
    BaseCaseSolveFailed(String),
    InvalidTopology(String),
    SingularIslandMatrix { island_index: usize, reason: String },
    NonFiniteSensitivity { branch_index: usize },
}

impl std::fmt::Display for ScreenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDenominatorTolerance => formatter.write_str(
                "LODF denominator tolerance must be finite and strictly between zero and one",
            ),
            Self::InvalidFlowTolerance => {
                formatter.write_str("flow comparison tolerance must be finite and non-negative")
            }
            Self::BaseCaseSolveFailed(reason) => {
                write!(formatter, "base DC power flow failed: {reason}")
            }
            Self::InvalidTopology(reason) => formatter.write_str(reason),
            Self::SingularIslandMatrix {
                island_index,
                reason,
            } => write!(
                formatter,
                "island {island_index} sensitivity matrix: {reason}"
            ),
            Self::NonFiniteSensitivity { branch_index } => {
                write!(
                    formatter,
                    "non-finite PTDF/LODF sensitivity at branch {branch_index}"
                )
            }
        }
    }
}

impl std::error::Error for ScreenError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExactResolveReason {
    BridgeOrNearBridge,
    NonPositiveLodfDenominator,
}

impl ExactResolveReason {
    fn as_str(&self) -> &'static str {
        match self {
            Self::BridgeOrNearBridge => "bridge_or_near_bridge",
            Self::NonPositiveLodfDenominator => "non_positive_lodf_denominator",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutageDisposition {
    PredictedWithinDeclaredLimits,
    PredictedOverload {
        max_overload_mw: f64,
    },
    RequiresExactResolve {
        reason: ExactResolveReason,
        ptdf_self: f64,
        lodf_denominator: f64,
    },
}

impl OutageDisposition {
    pub fn requires_exact_resolve(&self) -> bool {
        matches!(self, Self::RequiresExactResolve { .. })
    }

    pub fn predicts_overload(&self) -> bool {
        matches!(self, Self::PredictedOverload { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BranchFlowPrediction {
    pub branch_index: usize,
    pub from_bus: i64,
    pub to_bus: i64,
    pub in_service: bool,
    /// Positive means flow from the input branch's from-bus to to-bus.
    pub predicted_flow_mw: f64,
    pub rate_a_limit_mw: Option<f64>,
    pub predicted_overload_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutagePrediction {
    pub outaged_branch_index: usize,
    pub outaged_from_bus: i64,
    pub outaged_to_bus: i64,
    pub ptdf_self: f64,
    pub lodf_denominator: f64,
    pub disposition: OutageDisposition,
    /// Empty only for abstained cases. Otherwise contains one row per input branch.
    pub branch_flows: Vec<BranchFlowPrediction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenSummary {
    pub expected_branch_outages: usize,
    pub evaluated_branch_outages: usize,
    pub resolved_predictions: usize,
    pub predicted_overload_outages: usize,
    pub exact_resolve_abstentions: usize,
    pub coverage_complete: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LODFScreenReport {
    pub base_case: DcPowerFlowResult,
    /// One ordered record for every in-service branch in the input network.
    pub outages: Vec<OutagePrediction>,
    pub summary: ScreenSummary,
    pub claim_ceiling: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenMismatch {
    pub outage_branch_index: usize,
    pub branch_index: usize,
    pub predicted_flow_mw: f64,
    pub exact_flow_mw: f64,
    pub absolute_error_mw: f64,
    pub tolerance_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenValidationReport {
    pub expected_branch_outages: usize,
    pub evaluated_branch_outages: usize,
    pub coverage_complete: bool,
    pub compared_predictions: usize,
    pub exact_resolve_abstentions: usize,
    pub abstentions_with_exact_solution: usize,
    pub exact_solver_unknowns: usize,
    pub false_safe_branch_count: usize,
    pub false_safe_outage_count: usize,
    pub false_alarm_branch_count: usize,
    pub flow_comparison_count: usize,
    pub max_abs_flow_error_mw: f64,
    pub max_abs_overload_error_mw: f64,
    pub flow_mismatch_count: usize,
    pub unexpected_exact_unknown_count: usize,
    pub mismatches: Vec<ScreenMismatch>,
    pub claim_ceiling: &'static str,
}

impl ScreenValidationReport {
    /// True only if every usable prediction matches an exact re-solve within
    /// tolerance and there are no false-safe branch or outage predictions.
    pub fn passes_declared_checks(&self) -> bool {
        self.coverage_complete
            && self.flow_mismatch_count == 0
            && self.false_safe_branch_count == 0
            && self.false_safe_outage_count == 0
            && self.unexpected_exact_unknown_count == 0
    }

    pub fn to_json(&self) -> String {
        let mismatches = self
            .mismatches
            .iter()
            .map(|mismatch| {
                format!(
                    concat!(
                        "{{\"outage_branch_index\":{},\"branch_index\":{},",
                        "\"predicted_flow_mw\":{},\"exact_flow_mw\":{},",
                        "\"absolute_error_mw\":{},\"tolerance_mw\":{}}}"
                    ),
                    mismatch.outage_branch_index,
                    mismatch.branch_index,
                    json_number(mismatch.predicted_flow_mw),
                    json_number(mismatch.exact_flow_mw),
                    json_number(mismatch.absolute_error_mw),
                    json_number(mismatch.tolerance_mw)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{",
                "\"schema_version\":\"dc-lodf-screen-validation.v0.1.0\",",
                "\"expected_branch_outages\":{},",
                "\"evaluated_branch_outages\":{},",
                "\"coverage_complete\":{},",
                "\"compared_predictions\":{},",
                "\"exact_resolve_abstentions\":{},",
                "\"abstentions_with_exact_solution\":{},",
                "\"exact_solver_unknowns\":{},",
                "\"false_safe_branch_count\":{},",
                "\"false_safe_outage_count\":{},",
                "\"false_alarm_branch_count\":{},",
                "\"flow_comparison_count\":{},",
                "\"max_abs_flow_error_mw\":{},",
                "\"max_abs_overload_error_mw\":{},",
                "\"flow_mismatch_count\":{},",
                "\"unexpected_exact_unknown_count\":{},",
                "\"passes_declared_checks\":{},",
                "\"mismatches\":[{}],",
                "\"claim_ceiling\":{}",
                "}}"
            ),
            self.expected_branch_outages,
            self.evaluated_branch_outages,
            self.coverage_complete,
            self.compared_predictions,
            self.exact_resolve_abstentions,
            self.abstentions_with_exact_solution,
            self.exact_solver_unknowns,
            self.false_safe_branch_count,
            self.false_safe_outage_count,
            self.false_alarm_branch_count,
            self.flow_comparison_count,
            json_number(self.max_abs_flow_error_mw),
            json_number(self.max_abs_overload_error_mw),
            self.flow_mismatch_count,
            self.unexpected_exact_unknown_count,
            self.passes_declared_checks(),
            mismatches,
            json_quote(self.claim_ceiling)
        )
    }
}

impl LODFScreenReport {
    pub fn to_json(&self) -> String {
        let outages = self
            .outages
            .iter()
            .map(outage_to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{",
                "\"schema_version\":\"dc-lodf-outage-screen.v0.1.0\",",
                "\"base_case\":{},",
                "\"summary\":{{",
                "\"expected_branch_outages\":{},",
                "\"evaluated_branch_outages\":{},",
                "\"resolved_predictions\":{},",
                "\"predicted_overload_outages\":{},",
                "\"exact_resolve_abstentions\":{},",
                "\"coverage_complete\":{}",
                "}},",
                "\"outages\":[{}],",
                "\"claim_ceiling\":{}",
                "}}"
            ),
            base_case_summary_to_json(&self.base_case),
            self.summary.expected_branch_outages,
            self.summary.evaluated_branch_outages,
            self.summary.resolved_predictions,
            self.summary.predicted_overload_outages,
            self.summary.exact_resolve_abstentions,
            self.summary.coverage_complete,
            outages,
            json_quote(self.claim_ceiling)
        )
    }
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

fn base_case_summary_to_json(base_case: &DcPowerFlowResult) -> String {
    format!(
        concat!(
            "{{",
            "\"bus_count\":{},\"branch_count\":{},",
            "\"energized_island_count\":{},",
            "\"max_abs_nodal_residual_mw\":{},",
            "\"max_branch_overload_mw\":{},",
            "\"within_nodal_tolerance\":{},",
            "\"within_declared_branch_limits\":{},",
            "\"feasible_under_declared_checks\":{}",
            "}}"
        ),
        base_case.bus_states.len(),
        base_case.branch_flows.len(),
        base_case.energized_island_count,
        json_number(base_case.max_abs_nodal_residual_mw),
        json_number(base_case.max_branch_overload_mw),
        base_case.within_nodal_tolerance,
        base_case.within_declared_branch_limits,
        base_case.feasible_under_declared_checks
    )
}

fn outage_to_json(outage: &OutagePrediction) -> String {
    let disposition = match &outage.disposition {
        OutageDisposition::PredictedWithinDeclaredLimits => {
            "{\"status\":\"predicted_within_declared_limits\"}".to_owned()
        }
        OutageDisposition::PredictedOverload { max_overload_mw } => format!(
            "{{\"status\":\"predicted_overload\",\"max_overload_mw\":{}}}",
            json_number(*max_overload_mw)
        ),
        OutageDisposition::RequiresExactResolve {
            reason,
            ptdf_self,
            lodf_denominator,
        } => format!(
            concat!(
                "{{\"status\":\"requires_exact_resolve\",",
                "\"reason\":{},\"ptdf_self\":{},\"lodf_denominator\":{}}}"
            ),
            json_quote(reason.as_str()),
            json_number(*ptdf_self),
            json_number(*lodf_denominator)
        ),
    };
    let flows = outage
        .branch_flows
        .iter()
        .map(|flow| {
            format!(
                concat!(
                    "{{\"branch_index\":{},\"from_bus\":{},\"to_bus\":{},",
                    "\"in_service\":{},\"predicted_flow_mw\":{},",
                    "\"rate_a_limit_mw\":{},\"predicted_overload_mw\":{}}}"
                ),
                flow.branch_index,
                flow.from_bus,
                flow.to_bus,
                flow.in_service,
                json_number(flow.predicted_flow_mw),
                flow.rate_a_limit_mw
                    .map(json_number)
                    .unwrap_or_else(|| "null".to_owned()),
                json_number(flow.predicted_overload_mw)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\"outaged_branch_index\":{},\"outaged_from_bus\":{},",
            "\"outaged_to_bus\":{},\"ptdf_self\":{},",
            "\"lodf_denominator\":{},\"disposition\":{},",
            "\"branch_flows\":[{}]}}"
        ),
        outage.outaged_branch_index,
        outage.outaged_from_bus,
        outage.outaged_to_bus,
        json_number(outage.ptdf_self),
        json_number(outage.lodf_denominator),
        disposition,
        flows
    )
}

#[derive(Clone, Debug)]
struct LuFactor {
    matrix: Vec<Vec<f64>>,
    permutation: Vec<usize>,
}

impl LuFactor {
    fn factorize(mut matrix: Vec<Vec<f64>>, island_index: usize) -> Result<Self, ScreenError> {
        let size = matrix.len();
        if matrix.iter().any(|row| row.len() != size) {
            return Err(ScreenError::SingularIslandMatrix {
                island_index,
                reason: "matrix is not square".to_owned(),
            });
        }
        if size == 0 {
            return Ok(Self {
                matrix,
                permutation: Vec::new(),
            });
        }
        let scale = matrix
            .iter()
            .flat_map(|row| row.iter())
            .map(|value| value.abs())
            .fold(0.0_f64, f64::max);
        if !scale.is_finite() || scale <= 0.0 {
            return Err(ScreenError::SingularIslandMatrix {
                island_index,
                reason: "matrix has no finite positive coefficient scale".to_owned(),
            });
        }
        let pivot_tolerance = scale * 1.0e-12;
        let mut permutation = (0..size).collect::<Vec<_>>();

        for pivot_column in 0..size {
            let pivot_row = (pivot_column..size)
                .max_by(|&left, &right| {
                    matrix[left][pivot_column]
                        .abs()
                        .total_cmp(&matrix[right][pivot_column].abs())
                })
                .expect("pivot range is non-empty");
            let pivot_value = matrix[pivot_row][pivot_column].abs();
            if !pivot_value.is_finite() || pivot_value <= pivot_tolerance {
                return Err(ScreenError::SingularIslandMatrix {
                    island_index,
                    reason: format!(
                        "pivot {pivot_column} is below the relative factorization tolerance"
                    ),
                });
            }
            if pivot_row != pivot_column {
                matrix.swap(pivot_row, pivot_column);
                permutation.swap(pivot_row, pivot_column);
            }

            for row in (pivot_column + 1)..size {
                matrix[row][pivot_column] /= matrix[pivot_column][pivot_column];
                let multiplier = matrix[row][pivot_column];
                for column in (pivot_column + 1)..size {
                    matrix[row][column] -= multiplier * matrix[pivot_column][column];
                }
            }
        }

        Ok(Self {
            matrix,
            permutation,
        })
    }

    fn solve(&self, rhs: &[f64]) -> Result<Vec<f64>, ScreenError> {
        let size = self.matrix.len();
        if rhs.len() != size {
            return Err(ScreenError::InvalidTopology(
                "transaction right-hand side has an unexpected dimension".to_owned(),
            ));
        }
        let mut solution = self
            .permutation
            .iter()
            .map(|&original_row| rhs[original_row])
            .collect::<Vec<_>>();

        for row in 0..size {
            for column in 0..row {
                solution[row] -= self.matrix[row][column] * solution[column];
            }
        }
        for row in (0..size).rev() {
            for column in (row + 1)..size {
                solution[row] -= self.matrix[row][column] * solution[column];
            }
            solution[row] /= self.matrix[row][row];
            if !solution[row].is_finite() {
                return Err(ScreenError::InvalidTopology(
                    "LU solve produced a non-finite angle sensitivity".to_owned(),
                ));
            }
        }
        Ok(solution)
    }
}

#[derive(Clone, Debug)]
struct IslandSensitivity {
    factor: LuFactor,
    reduced_index_by_bus: Vec<Option<usize>>,
}

#[derive(Clone, Debug)]
struct SensitivityModel {
    bus_index: BTreeMap<i64, usize>,
    component_by_bus: Vec<usize>,
    islands: Vec<IslandSensitivity>,
    susceptance_mw_per_rad: Vec<Option<f64>>,
}

fn build_sensitivity_model(network: &DcNetwork) -> Result<SensitivityModel, ScreenError> {
    let bus_count = network.buses.len();
    let mut bus_index = BTreeMap::new();
    for (index, bus) in network.buses.iter().enumerate() {
        if bus_index.insert(bus.id, index).is_some() {
            return Err(ScreenError::InvalidTopology(format!(
                "duplicate bus ID {}",
                bus.id
            )));
        }
    }

    let mut adjacency = vec![Vec::<usize>::new(); bus_count];
    let mut susceptance_mw_per_rad = vec![None; network.branches.len()];
    for (branch_index, branch) in network.branches.iter().enumerate() {
        let from = *bus_index.get(&branch.from_bus).ok_or_else(|| {
            ScreenError::InvalidTopology(format!(
                "branch {branch_index} references unknown from-bus {}",
                branch.from_bus
            ))
        })?;
        let to = *bus_index.get(&branch.to_bus).ok_or_else(|| {
            ScreenError::InvalidTopology(format!(
                "branch {branch_index} references unknown to-bus {}",
                branch.to_bus
            ))
        })?;
        if branch.in_service {
            let tap = if branch.tap_ratio == 0.0 {
                1.0
            } else {
                branch.tap_ratio
            };
            let value = network.base_mva / (branch.reactance_pu * tap);
            if !value.is_finite() || value <= 0.0 {
                return Err(ScreenError::InvalidTopology(format!(
                    "invalid susceptance at branch {branch_index}"
                )));
            }
            susceptance_mw_per_rad[branch_index] = Some(value);
            adjacency[from].push(to);
            adjacency[to].push(from);
        }
    }

    let components = connected_components(&adjacency);
    let mut component_by_bus = vec![usize::MAX; bus_count];
    for (component_index, component) in components.iter().enumerate() {
        for &bus in component {
            component_by_bus[bus] = component_index;
        }
    }

    let mut islands = Vec::with_capacity(components.len());
    for (island_index, component) in components.iter().enumerate() {
        let reference_buses = component
            .iter()
            .copied()
            .filter(|&bus| network.buses[bus].specified_injection_mw.is_none())
            .collect::<Vec<_>>();
        if reference_buses.len() != 1 {
            return Err(ScreenError::InvalidTopology(format!(
                "island {island_index} must have exactly one reference bus; found {}",
                reference_buses.len()
            )));
        }
        let reference_bus = reference_buses[0];
        let mut reduced_index_by_bus = vec![None; bus_count];
        let mut reduced_bus_indices = Vec::with_capacity(component.len().saturating_sub(1));
        for &bus in component {
            if bus != reference_bus {
                reduced_index_by_bus[bus] = Some(reduced_bus_indices.len());
                reduced_bus_indices.push(bus);
            }
        }

        let reduced_size = reduced_bus_indices.len();
        let mut matrix = vec![vec![0.0_f64; reduced_size]; reduced_size];
        for (branch_index, branch) in network.branches.iter().enumerate() {
            let Some(susceptance) = susceptance_mw_per_rad[branch_index] else {
                continue;
            };
            let from = bus_index[&branch.from_bus];
            if component_by_bus[from] != island_index {
                continue;
            }
            let to = bus_index[&branch.to_bus];
            if let Some(row) = reduced_index_by_bus[from] {
                matrix[row][row] += susceptance;
            }
            if let Some(row) = reduced_index_by_bus[to] {
                matrix[row][row] += susceptance;
            }
            if let (Some(from_row), Some(to_row)) =
                (reduced_index_by_bus[from], reduced_index_by_bus[to])
            {
                matrix[from_row][to_row] -= susceptance;
                matrix[to_row][from_row] -= susceptance;
            }
        }

        let factor = LuFactor::factorize(matrix, island_index)?;
        islands.push(IslandSensitivity {
            factor,
            reduced_index_by_bus,
        });
    }

    Ok(SensitivityModel {
        bus_index,
        component_by_bus,
        islands,
        susceptance_mw_per_rad,
    })
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

/// Build the intact fixed-injection solution and screen each in-service branch
/// outage with PTDF/LODF sensitivities. Bridge/near-bridge outages abstain.
pub fn screen_single_branch_outages(
    network: &DcNetwork,
    solve_options: SolveOptions,
    screen_options: ScreenOptions,
) -> Result<LODFScreenReport, ScreenError> {
    if !screen_options.lodf_denominator_tolerance.is_finite()
        || screen_options.lodf_denominator_tolerance <= 0.0
        || screen_options.lodf_denominator_tolerance >= 1.0
    {
        return Err(ScreenError::InvalidDenominatorTolerance);
    }

    let base_case = solve_dc_power_flow(network, solve_options)
        .map_err(|error| ScreenError::BaseCaseSolveFailed(error.to_string()))?;
    let sensitivity = build_sensitivity_model(network)?;
    let expected_branch_outages = network
        .branches
        .iter()
        .filter(|branch| branch.in_service)
        .count();

    let mut outages = Vec::with_capacity(expected_branch_outages);
    let mut resolved_predictions = 0;
    let mut predicted_overload_outages = 0;
    let mut exact_resolve_abstentions = 0;

    for (outaged_branch_index, outaged_branch) in network.branches.iter().enumerate() {
        if !outaged_branch.in_service {
            continue;
        }
        let from_index = sensitivity.bus_index[&outaged_branch.from_bus];
        let to_index = sensitivity.bus_index[&outaged_branch.to_bus];
        let island_index = sensitivity.component_by_bus[from_index];
        if island_index != sensitivity.component_by_bus[to_index] {
            return Err(ScreenError::InvalidTopology(format!(
                "active branch {outaged_branch_index} connects distinct islands"
            )));
        }

        let island = &sensitivity.islands[island_index];
        let mut transaction = vec![0.0_f64; island.factor.matrix.len()];
        if let Some(row) = island.reduced_index_by_bus[from_index] {
            transaction[row] += 1.0;
        }
        if let Some(row) = island.reduced_index_by_bus[to_index] {
            transaction[row] -= 1.0;
        }
        let angle_sensitivity = island.factor.solve(&transaction)?;
        let mut angle_delta = vec![0.0_f64; network.buses.len()];
        for bus_index in 0..network.buses.len() {
            if sensitivity.component_by_bus[bus_index] == island_index
                && let Some(row) = island.reduced_index_by_bus[bus_index]
            {
                angle_delta[bus_index] = angle_sensitivity[row];
            }
        }

        let outaged_susceptance = sensitivity.susceptance_mw_per_rad[outaged_branch_index]
            .expect("in-service branch has a susceptance");
        let ptdf_self = outaged_susceptance * (angle_delta[from_index] - angle_delta[to_index]);
        let lodf_denominator = 1.0 - ptdf_self;
        if !ptdf_self.is_finite() || !lodf_denominator.is_finite() {
            return Err(ScreenError::NonFiniteSensitivity {
                branch_index: outaged_branch_index,
            });
        }

        let mut branch_flows = Vec::new();
        let disposition = if lodf_denominator <= screen_options.lodf_denominator_tolerance {
            exact_resolve_abstentions += 1;
            let reason = if lodf_denominator.abs() <= screen_options.lodf_denominator_tolerance {
                ExactResolveReason::BridgeOrNearBridge
            } else {
                ExactResolveReason::NonPositiveLodfDenominator
            };
            OutageDisposition::RequiresExactResolve {
                reason,
                ptdf_self,
                lodf_denominator,
            }
        } else {
            let base_outaged_flow = base_case.branch_flows[outaged_branch_index].flow_from_to_mw;
            let mut max_overload_mw = 0.0_f64;
            branch_flows.reserve(network.branches.len());

            for (branch_index, branch) in network.branches.iter().enumerate() {
                let base_flow = base_case.branch_flows[branch_index].flow_from_to_mw;
                let in_service = branch.in_service && branch_index != outaged_branch_index;
                let predicted_flow = if !in_service {
                    0.0
                } else {
                    let from = sensitivity.bus_index[&branch.from_bus];
                    if sensitivity.component_by_bus[from] == island_index
                        && branch_index != outaged_branch_index
                    {
                        let to = sensitivity.bus_index[&branch.to_bus];
                        let susceptance = sensitivity.susceptance_mw_per_rad[branch_index]
                            .expect("in-service branch has a susceptance");
                        let ptdf = susceptance * (angle_delta[from] - angle_delta[to]);
                        base_flow + ptdf / lodf_denominator * base_outaged_flow
                    } else {
                        base_flow
                    }
                };
                if !predicted_flow.is_finite() {
                    return Err(ScreenError::NonFiniteSensitivity {
                        branch_index: outaged_branch_index,
                    });
                }
                let rate_a_limit_mw = branch.rate_a_mva.filter(|limit| *limit > 0.0);
                let predicted_overload_mw = rate_a_limit_mw
                    .map(|limit| (predicted_flow.abs() - limit).max(0.0))
                    .unwrap_or(0.0);
                max_overload_mw = max_overload_mw.max(predicted_overload_mw);
                branch_flows.push(BranchFlowPrediction {
                    branch_index,
                    from_bus: branch.from_bus,
                    to_bus: branch.to_bus,
                    in_service,
                    predicted_flow_mw: predicted_flow,
                    rate_a_limit_mw,
                    predicted_overload_mw,
                });
            }

            resolved_predictions += 1;
            if max_overload_mw > solve_options.branch_limit_tolerance_mw {
                predicted_overload_outages += 1;
                OutageDisposition::PredictedOverload { max_overload_mw }
            } else {
                OutageDisposition::PredictedWithinDeclaredLimits
            }
        };

        outages.push(OutagePrediction {
            outaged_branch_index,
            outaged_from_bus: outaged_branch.from_bus,
            outaged_to_bus: outaged_branch.to_bus,
            ptdf_self,
            lodf_denominator,
            disposition,
            branch_flows,
        });
    }

    let evaluated_branch_outages = outages.len();
    Ok(LODFScreenReport {
        base_case,
        outages,
        summary: ScreenSummary {
            expected_branch_outages,
            evaluated_branch_outages,
            resolved_predictions,
            predicted_overload_outages,
            exact_resolve_abstentions,
            coverage_complete: evaluated_branch_outages == expected_branch_outages,
        },
        claim_ceiling: "linear_dc_ptdf_lodf_single_branch_screening_with_exact_resolve_abstention",
    })
}

/// Compare every resolved LODF prediction against a direct DC re-solve for
/// the same fixed injections and outage. This intentionally performs one
/// additional exact solve per outage and is a validation/bake-off routine.
pub fn compare_screen_to_exact(
    network: &DcNetwork,
    screen: &LODFScreenReport,
    solve_options: SolveOptions,
    flow_tolerance_mw: f64,
) -> Result<ScreenValidationReport, ScreenError> {
    if !flow_tolerance_mw.is_finite() || flow_tolerance_mw < 0.0 {
        return Err(ScreenError::InvalidFlowTolerance);
    }
    let mut report = ScreenValidationReport {
        expected_branch_outages: screen.summary.expected_branch_outages,
        evaluated_branch_outages: screen.outages.len(),
        coverage_complete: screen.summary.coverage_complete
            && screen.outages.len() == screen.summary.expected_branch_outages,
        compared_predictions: 0,
        exact_resolve_abstentions: 0,
        abstentions_with_exact_solution: 0,
        exact_solver_unknowns: 0,
        false_safe_branch_count: 0,
        false_safe_outage_count: 0,
        false_alarm_branch_count: 0,
        flow_comparison_count: 0,
        max_abs_flow_error_mw: 0.0,
        max_abs_overload_error_mw: 0.0,
        flow_mismatch_count: 0,
        unexpected_exact_unknown_count: 0,
        mismatches: Vec::new(),
        claim_ceiling: "screen_vs_exact_fixed_injection_dc_resolve_only",
    };

    let exact_base = solve_dc_power_flow(network, solve_options)
        .map_err(|error| ScreenError::BaseCaseSolveFailed(error.to_string()))?;
    if exact_base.branch_flows.len() != screen.base_case.branch_flows.len() {
        return Err(ScreenError::InvalidTopology(
            "screen base receipt does not match the validation network branch count".to_owned(),
        ));
    }
    for (index, (exact, screened)) in exact_base
        .branch_flows
        .iter()
        .zip(screen.base_case.branch_flows.iter())
        .enumerate()
    {
        if exact.from_bus != screened.from_bus
            || exact.to_bus != screened.to_bus
            || exact.in_service != screened.in_service
            || (exact.flow_from_to_mw - screened.flow_from_to_mw).abs() > flow_tolerance_mw
        {
            return Err(ScreenError::InvalidTopology(format!(
                "base receipt mismatch at branch {index}"
            )));
        }
    }

    for outage in &screen.outages {
        let mut contingency = network.clone();
        let branch = contingency
            .branches
            .get_mut(outage.outaged_branch_index)
            .ok_or_else(|| {
                ScreenError::InvalidTopology(format!(
                    "screen refers to unknown outage branch {}",
                    outage.outaged_branch_index
                ))
            })?;
        if branch.from_bus != outage.outaged_from_bus
            || branch.to_bus != outage.outaged_to_bus
            || !branch.in_service
        {
            return Err(ScreenError::InvalidTopology(format!(
                "screen outage identity mismatch at branch {}",
                outage.outaged_branch_index
            )));
        }
        branch.in_service = false;
        let exact = solve_dc_power_flow(&contingency, solve_options);

        match &outage.disposition {
            OutageDisposition::RequiresExactResolve { .. } => {
                report.exact_resolve_abstentions += 1;
                match exact {
                    Ok(_) => report.abstentions_with_exact_solution += 1,
                    Err(_) => report.exact_solver_unknowns += 1,
                }
            }
            OutageDisposition::PredictedWithinDeclaredLimits
            | OutageDisposition::PredictedOverload { .. } => {
                let exact = match exact {
                    Ok(result) => result,
                    Err(_) => {
                        report.unexpected_exact_unknown_count += 1;
                        continue;
                    }
                };
                report.compared_predictions += 1;
                let predicted_outage_safe = matches!(
                    &outage.disposition,
                    OutageDisposition::PredictedWithinDeclaredLimits
                );
                if predicted_outage_safe && !exact.within_declared_branch_limits {
                    report.false_safe_outage_count += 1;
                }

                for predicted in &outage.branch_flows {
                    let exact_flow =
                        exact
                            .branch_flows
                            .get(predicted.branch_index)
                            .ok_or_else(|| {
                                ScreenError::InvalidTopology(format!(
                                    "exact solve omitted branch {}",
                                    predicted.branch_index
                                ))
                            })?;
                    if exact_flow.from_bus != predicted.from_bus
                        || exact_flow.to_bus != predicted.to_bus
                        || exact_flow.in_service != predicted.in_service
                    {
                        return Err(ScreenError::InvalidTopology(format!(
                            "branch identity/status mismatch for branch {}",
                            predicted.branch_index
                        )));
                    }
                    report.flow_comparison_count += 1;
                    let error = (predicted.predicted_flow_mw - exact_flow.flow_from_to_mw).abs();
                    report.max_abs_flow_error_mw = report.max_abs_flow_error_mw.max(error);
                    let overload_error =
                        (predicted.predicted_overload_mw - exact_flow.overload_mw).abs();
                    report.max_abs_overload_error_mw =
                        report.max_abs_overload_error_mw.max(overload_error);
                    if error > flow_tolerance_mw {
                        report.flow_mismatch_count += 1;
                        report.mismatches.push(ScreenMismatch {
                            outage_branch_index: outage.outaged_branch_index,
                            branch_index: predicted.branch_index,
                            predicted_flow_mw: predicted.predicted_flow_mw,
                            exact_flow_mw: exact_flow.flow_from_to_mw,
                            absolute_error_mw: error,
                            tolerance_mw: flow_tolerance_mw,
                        });
                    }
                    let predicted_safe =
                        predicted.predicted_overload_mw <= solve_options.branch_limit_tolerance_mw;
                    let exact_unsafe =
                        exact_flow.overload_mw > solve_options.branch_limit_tolerance_mw;
                    if predicted_safe && exact_unsafe {
                        report.false_safe_branch_count += 1;
                    }
                    if !predicted_safe && !exact_unsafe {
                        report.false_alarm_branch_count += 1;
                    }
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> DcNetwork {
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
                DcBus {
                    id: 3,
                    specified_injection_mw: Some(0.0),
                },
            ],
            branches: vec![
                dc_branch(1, 2, 0.1, 0.0, 0.0, Some(45.0), true),
                dc_branch(2, 3, 0.2, 0.0, 0.0, Some(45.0), true),
                dc_branch(1, 3, 0.2, 0.0, 0.0, Some(45.0), true),
            ],
        }
    }

    fn dc_branch(
        from_bus: i64,
        to_bus: i64,
        reactance_pu: f64,
        tap_ratio: f64,
        phase_shift_degrees: f64,
        rate_a_mva: Option<f64>,
        in_service: bool,
    ) -> dc_power_flow_reference::DcBranch {
        dc_power_flow_reference::DcBranch {
            from_bus,
            to_bus,
            reactance_pu,
            tap_ratio,
            phase_shift_degrees,
            rate_a_mva,
            in_service,
        }
    }

    fn validate(network: &DcNetwork) -> ScreenValidationReport {
        let screen = screen_single_branch_outages(
            network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .expect("network can be screened");
        compare_screen_to_exact(network, &screen, SolveOptions::default(), 1.0e-7)
            .expect("screen can be compared to exact re-solves")
    }

    #[test]
    fn triangle_lodf_predictions_match_every_exact_branch_outage_resolve() {
        let network = triangle();
        let screen = screen_single_branch_outages(
            &network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .expect("triangle can be screened");
        let validation =
            compare_screen_to_exact(&network, &screen, SolveOptions::default(), 1.0e-7)
                .expect("exact comparison succeeds");

        assert!(screen.summary.coverage_complete);
        assert_eq!(screen.summary.expected_branch_outages, 3);
        assert_eq!(screen.summary.resolved_predictions, 3);
        assert_eq!(screen.summary.exact_resolve_abstentions, 0);
        assert!(
            validation.passes_declared_checks(),
            "{}",
            validation.to_json()
        );
        assert_eq!(validation.flow_comparison_count, 9);
        assert!(validation.max_abs_flow_error_mw <= 1.0e-7);
    }

    #[test]
    fn bridge_outage_abstains_and_exact_solver_reports_unknown() {
        let network = DcNetwork {
            base_mva: 100.0,
            buses: vec![
                DcBus {
                    id: 1,
                    specified_injection_mw: None,
                },
                DcBus {
                    id: 2,
                    specified_injection_mw: Some(-10.0),
                },
            ],
            branches: vec![dc_branch(1, 2, 0.1, 0.0, 0.0, None, true)],
        };
        let screen = screen_single_branch_outages(
            &network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .expect("base case is connected");
        assert!(matches!(
            &screen.outages[0].disposition,
            OutageDisposition::RequiresExactResolve {
                reason: ExactResolveReason::BridgeOrNearBridge,
                ..
            }
        ));
        let validation =
            compare_screen_to_exact(&network, &screen, SolveOptions::default(), 1.0e-7)
                .expect("unknown can be reconciled");
        assert_eq!(validation.exact_resolve_abstentions, 1);
        assert_eq!(validation.exact_solver_unknowns, 1);
        assert_eq!(validation.abstentions_with_exact_solution, 0);
    }

    #[test]
    fn transformer_tap_and_phase_shift_predictions_match_exact_resolves() {
        let mut network = triangle();
        network.branches[0].tap_ratio = 1.1;
        network.branches[0].phase_shift_degrees = 4.0;
        network.branches[2].tap_ratio = 0.9;
        network.branches[1].phase_shift_degrees = -2.5;
        let validation = validate(&network);
        assert!(
            validation.passes_declared_checks(),
            "{}",
            validation.to_json()
        );
        assert!(validation.max_abs_flow_error_mw <= 1.0e-7);
    }

    #[test]
    fn multiple_islands_are_factorized_independently() {
        let mut network = triangle();
        network.buses.extend([
            DcBus {
                id: 4,
                specified_injection_mw: None,
            },
            DcBus {
                id: 5,
                specified_injection_mw: Some(-20.0),
            },
            DcBus {
                id: 6,
                specified_injection_mw: Some(0.0),
            },
        ]);
        network.branches.extend([
            dc_branch(4, 5, 0.1, 0.0, 0.0, None, true),
            dc_branch(5, 6, 0.2, 0.0, 0.0, None, true),
            dc_branch(4, 6, 0.2, 0.0, 0.0, None, true),
        ]);
        let validation = validate(&network);
        assert!(
            validation.passes_declared_checks(),
            "{}",
            validation.to_json()
        );
        assert_eq!(validation.expected_branch_outages, 6);
    }

    #[test]
    fn outages_on_deenergized_branches_are_not_in_the_contingency_panel() {
        let mut network = triangle();
        network
            .branches
            .push(dc_branch(1, 3, 0.2, 0.0, 0.0, None, false));
        let screen = screen_single_branch_outages(
            &network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .expect("out-of-service rows are not contingencies");
        assert_eq!(screen.summary.expected_branch_outages, 3);
        assert_eq!(screen.summary.evaluated_branch_outages, 3);
    }

    #[test]
    fn invalid_lodf_denominator_tolerance_is_rejected() {
        let error = screen_single_branch_outages(
            &triangle(),
            SolveOptions::default(),
            ScreenOptions {
                lodf_denominator_tolerance: 0.0,
            },
        )
        .unwrap_err();
        assert_eq!(error, ScreenError::InvalidDenominatorTolerance);
    }

    #[test]
    fn screening_is_deterministic_and_json_is_parseable() {
        let network = triangle();
        let first = screen_single_branch_outages(
            &network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .unwrap();
        let second = screen_single_branch_outages(
            &network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(first.to_json().starts_with("{"));
        assert!(
            first
                .to_json()
                .contains("\"schema_version\":\"dc-lodf-outage-screen.v0.1.0\"")
        );
    }

    #[test]
    fn no_active_branches_returns_complete_empty_contingency_panel() {
        let network = DcNetwork {
            base_mva: 100.0,
            buses: vec![DcBus {
                id: 1,
                specified_injection_mw: None,
            }],
            branches: Vec::new(),
        };
        let screen = screen_single_branch_outages(
            &network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .expect("single-reference singleton island is valid");
        assert_eq!(screen.summary.expected_branch_outages, 0);
        assert_eq!(screen.summary.evaluated_branch_outages, 0);
        assert!(screen.summary.coverage_complete);
    }
}
