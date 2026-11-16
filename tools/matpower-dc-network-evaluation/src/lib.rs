// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Source-bound MATPOWER v2 projection, DC reference solve and independent audit.
//!
//! This crate bridges the narrow MATPOWER v2 intake profile to the small-network
//! DC reference solver. It is not DC-OPF, AC power flow or an operational controller.

use dc_fixed_injection_contingency_sweep::{
    ContingencySweepReport, SweepOptions, sweep_fixed_injection_single_branch_outages,
};
use dc_lodf_outage_screen::{
    LODFScreenReport, ScreenOptions, compare_screen_to_exact, screen_single_branch_outages,
};
use dc_power_flow_independent_auditor::{AuditOptions, AuditReport, audit_dc_power_flow};
use dc_power_flow_reference::{
    DcBranch, DcBus, DcNetwork, DcPowerFlowResult, SolveOptions, solve_dc_power_flow,
};
use pglib_case_auditor::{MatpowerDcProjection, project_matpower_dc_case};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluationError {
    pub stage: &'static str,
    pub reason: String,
}

impl EvaluationError {
    fn new(stage: &'static str, reason: impl Into<String>) -> Self {
        Self {
            stage,
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.stage, self.reason)
    }
}

impl std::error::Error for EvaluationError {}

#[derive(Clone, Debug)]
pub struct MatpowerDcEvaluation {
    pub source_git_blob_sha1: String,
    pub source_function_name: String,
    pub matpower_version: String,
    pub network: DcNetwork,
    pub solution: DcPowerFlowResult,
    pub audit: AuditReport,
    pub limitations: &'static [&'static str],
    pub claim_ceiling: &'static str,
}

fn map_projection(projection: &MatpowerDcProjection) -> Result<DcNetwork, EvaluationError> {
    if projection.source_git_blob_sha1.len() != 40
        || !projection
            .source_git_blob_sha1
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(EvaluationError::new(
            "projection",
            "source Git blob SHA-1 is malformed",
        ));
    }
    if !projection.base_mva.is_finite() || projection.base_mva <= 0.0 {
        return Err(EvaluationError::new(
            "projection",
            "base MVA must be finite and positive",
        ));
    }
    if projection.buses.is_empty() {
        return Err(EvaluationError::new(
            "projection",
            "projection contains no buses",
        ));
    }

    let buses = projection
        .buses
        .iter()
        .map(|bus| {
            if bus.bus_type == 4 {
                return Err(EvaluationError::new(
                    "projection",
                    format!("isolated type-4 bus {} is unsupported", bus.id),
                ));
            }
            let specified_injection_mw = match (bus.bus_type, bus.specified_injection_mw) {
                (3, None) => None,
                (3, Some(_)) => {
                    return Err(EvaluationError::new(
                        "projection",
                        format!(
                            "type-3 reference bus {} must have unspecified injection",
                            bus.id
                        ),
                    ));
                }
                (1 | 2, Some(value)) if value.is_finite() => Some(value),
                (1 | 2, _) => {
                    return Err(EvaluationError::new(
                        "projection",
                        format!(
                            "non-reference bus {} requires a finite specified injection",
                            bus.id
                        ),
                    ));
                }
                (unsupported, _) => {
                    return Err(EvaluationError::new(
                        "projection",
                        format!(
                            "unsupported MATPOWER bus type {unsupported} at bus {}",
                            bus.id
                        ),
                    ));
                }
            };
            Ok(DcBus {
                id: bus.id,
                specified_injection_mw,
            })
        })
        .collect::<Result<Vec<_>, EvaluationError>>()?;

    let branches = projection
        .branches
        .iter()
        .map(|branch| {
            if !branch.rate_a_mva.is_finite() || branch.rate_a_mva < 0.0 {
                return Err(EvaluationError::new(
                    "projection",
                    format!("invalid Rate A at source branch row {}", branch.source_row),
                ));
            }
            let rate_a_mva = (branch.rate_a_mva > 0.0).then_some(branch.rate_a_mva);
            Ok(DcBranch {
                from_bus: branch.from_bus,
                to_bus: branch.to_bus,
                reactance_pu: branch.reactance_pu,
                tap_ratio: branch.tap_ratio,
                phase_shift_degrees: branch.phase_shift_degrees,
                rate_a_mva,
                in_service: branch.in_service,
            })
        })
        .collect::<Result<Vec<_>, EvaluationError>>()?;

    Ok(DcNetwork {
        base_mva: projection.base_mva,
        buses,
        branches,
    })
}

fn audit_options(solve: SolveOptions) -> AuditOptions {
    AuditOptions {
        value_tolerance_mw: solve.residual_tolerance_mw,
        angle_tolerance_rad: 1.0e-9,
        residual_tolerance_mw: solve.residual_tolerance_mw,
        branch_limit_tolerance_mw: solve.branch_limit_tolerance_mw,
    }
}

/// Parse the original source, create the typed projection, solve the fixed-injection
/// DC model and independently audit the result. The digest is computed from the
/// exact UTF-8 source bytes supplied here, not copied from a caller-provided struct.
///
/// A successful return means the independent audit accepted receipt consistency.
/// The audit's feasibility flag separately reports nodal and Rate A outcomes.
pub fn evaluate_matpower_dc_case(
    source: &str,
    solve: SolveOptions,
) -> Result<MatpowerDcEvaluation, EvaluationError> {
    let projection = project_matpower_dc_case(source)
        .map_err(|error| EvaluationError::new("matpower_projection", error))?;
    evaluate_projection(&projection, solve)
}

fn evaluate_projection(
    projection: &MatpowerDcProjection,
    solve: SolveOptions,
) -> Result<MatpowerDcEvaluation, EvaluationError> {
    let network = map_projection(projection)?;
    let solution = solve_dc_power_flow(&network, solve)
        .map_err(|error| EvaluationError::new("dc_reference_solve", error.to_string()))?;
    let audit = audit_dc_power_flow(&network, &solution, audit_options(solve))
        .map_err(|error| EvaluationError::new("independent_audit", error.to_string()))?;

    Ok(MatpowerDcEvaluation {
        source_git_blob_sha1: projection.source_git_blob_sha1.clone(),
        source_function_name: projection.function_name.clone(),
        matpower_version: projection.matpower_version.clone(),
        network,
        solution,
        audit,
        limitations: projection.limitations,
        claim_ceiling: "audited_fixed_injection_dc_power_flow_only",
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

fn number(value: f64) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

impl MatpowerDcEvaluation {
    /// Deterministic dependency-free JSON evidence receipt.
    pub fn to_json(&self) -> String {
        let bus_states = self
            .solution
            .bus_states
            .iter()
            .map(|bus| {
                format!(
                    concat!(
                        "{{\"bus_id\":{},\"angle_rad\":{},",
                        "\"net_injection_mw\":{},\"net_outflow_mw\":{},",
                        "\"nodal_residual_mw\":{},\"reference_bus\":{}}}"
                    ),
                    bus.bus_id,
                    number(bus.voltage_angle_rad),
                    number(bus.net_injection_mw),
                    number(bus.calculated_net_outflow_mw),
                    number(bus.nodal_residual_mw),
                    bus.is_reference_bus
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let branch_flows = self
            .solution
            .branch_flows
            .iter()
            .map(|branch| {
                let rate = branch
                    .rate_a_limit_mw
                    .map(number)
                    .unwrap_or_else(|| "null".to_owned());
                format!(
                    concat!(
                        "{{\"branch_index\":{},\"from_bus\":{},\"to_bus\":{},",
                        "\"in_service\":{},\"flow_from_to_mw\":{},",
                        "\"rate_a_limit_mw\":{},\"overload_mw\":{}}}"
                    ),
                    branch.branch_index,
                    branch.from_bus,
                    branch.to_bus,
                    branch.in_service,
                    number(branch.flow_from_to_mw),
                    rate,
                    number(branch.overload_mw)
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let limitations = self
            .limitations
            .iter()
            .map(|limitation| json_quote(limitation))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            concat!(
                "{{",
                "\"schema_version\":\"matpower-dc-evaluation.v0.1.0\",",
                "\"source_git_blob_sha1\":{},",
                "\"source_function_name\":{},",
                "\"matpower_version\":{},",
                "\"base_mva\":{},",
                "\"bus_count\":{},",
                "\"branch_count\":{},",
                "\"energized_island_count\":{},",
                "\"max_abs_nodal_residual_mw\":{},",
                "\"max_branch_overload_mw\":{},",
                "\"within_nodal_tolerance\":{},",
                "\"within_declared_branch_limits\":{},",
                "\"independently_feasible_under_declared_checks\":{},",
                "\"bus_states\":[{}],",
                "\"branch_flows\":[{}],",
                "\"limitations\":[{}],",
                "\"claim_ceiling\":{}",
                "}}"
            ),
            json_quote(&self.source_git_blob_sha1),
            json_quote(&self.source_function_name),
            json_quote(&self.matpower_version),
            number(self.network.base_mva),
            self.network.buses.len(),
            self.network.branches.len(),
            self.audit.energized_island_count,
            number(self.audit.max_abs_nodal_residual_mw),
            number(self.audit.max_branch_overload_mw),
            self.audit.within_nodal_tolerance,
            self.audit.within_declared_branch_limits,
            self.audit.independently_feasible_under_declared_checks,
            bus_states,
            branch_flows,
            limitations,
            json_quote(self.claim_ceiling)
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatpowerDcContingencyEvaluation {
    pub source_git_blob_sha1: String,
    pub source_function_name: String,
    pub matpower_version: String,
    pub sweep: ContingencySweepReport,
    pub claim_ceiling: &'static str,
}

/// Run the intact case and every in-service single-branch outage for an exact
/// MATPOWER source. Each candidate is solved by the DC reference crate and
/// independently audited by the contingency evaluator's separate auditor.
///
/// max_contingencies is a hard all-or-nothing bound. A budget failure returns
/// no partial report. Solver failures and islanding remain explicit unknowns.
pub fn sweep_matpower_dc_case(
    source: &str,
    solve: SolveOptions,
    max_contingencies: usize,
) -> Result<MatpowerDcContingencyEvaluation, EvaluationError> {
    let projection = project_matpower_dc_case(source)
        .map_err(|error| EvaluationError::new("matpower_projection", error))?;
    let network = map_projection(&projection)?;
    let identity = format!(
        "{}:{}",
        projection.function_name, projection.source_git_blob_sha1
    );
    let sweep = sweep_fixed_injection_single_branch_outages(
        &identity,
        &network,
        SweepOptions {
            solve_options: solve,
            max_contingencies,
        },
    )
    .map_err(|error| EvaluationError::new("contingency_sweep", error.to_string()))?;

    Ok(MatpowerDcContingencyEvaluation {
        source_git_blob_sha1: projection.source_git_blob_sha1,
        source_function_name: projection.function_name,
        matpower_version: projection.matpower_version,
        sweep,
        claim_ceiling: "audited_fixed_injection_dc_single_branch_outage_screening_only",
    })
}

impl MatpowerDcContingencyEvaluation {
    /// Deterministic JSON receipt with exact source identity and complete sweep.
    pub fn to_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"schema_version\":\"matpower-dc-contingency-evaluation.v0.1.0\",",
                "\"source_git_blob_sha1\":{},",
                "\"source_function_name\":{},",
                "\"matpower_version\":{},",
                "\"sweep\":{},",
                "\"claim_ceiling\":{}",
                "}}"
            ),
            json_quote(&self.source_git_blob_sha1),
            json_quote(&self.source_function_name),
            json_quote(&self.matpower_version),
            self.sweep.to_json(),
            json_quote(self.claim_ceiling)
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatpowerDcLodfScreenEvaluation {
    pub source_git_blob_sha1: String,
    pub source_function_name: String,
    pub matpower_version: String,
    pub screen: LODFScreenReport,
}

impl MatpowerDcLodfScreenEvaluation {
    pub fn to_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"schema_version\":\"matpower-dc-lodf-screen-evaluation.v0.1.0\",",
                "\"source_git_blob_sha1\":{},",
                "\"source_function_name\":{},",
                "\"matpower_version\":{},",
                "\"screen\":{},",
                "\"claim_ceiling\":\"{}\"",
                "}}"
            ),
            json_quote(&self.source_git_blob_sha1),
            json_quote(&self.source_function_name),
            json_quote(&self.matpower_version),
            self.screen.to_json(),
            self.screen.claim_ceiling
        )
    }
}

/// Run the source-bound fixed-injection DC base case and produce an LODF
/// contingency screen. Bridge/near-bridge outages remain exact-solve abstentions.
pub fn screen_matpower_dc_case(
    source: &str,
    solve_options: SolveOptions,
    screen_options: ScreenOptions,
) -> Result<MatpowerDcLodfScreenEvaluation, EvaluationError> {
    let projection = project_matpower_dc_case(source)
        .map_err(|error| EvaluationError::new("matpower_projection", error))?;
    let network = map_projection(&projection)?;
    let screen = screen_single_branch_outages(&network, solve_options, screen_options)
        .map_err(|error| EvaluationError::new("lodf_screen", error.to_string()))?;

    Ok(MatpowerDcLodfScreenEvaluation {
        source_git_blob_sha1: projection.source_git_blob_sha1,
        source_function_name: projection.function_name,
        matpower_version: projection.matpower_version,
        screen,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASE14: &str = include_str!("../../pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m");
    const CASE24: &str =
        include_str!("../../pglib-case-auditor/fixtures/pglib_opf_case24_ieee_rts.m");
    const CASE57: &str = include_str!("../../pglib-case-auditor/fixtures/pglib_opf_case57_ieee.m");

    fn evaluate(source: &str) -> MatpowerDcEvaluation {
        evaluate_matpower_dc_case(source, SolveOptions::default())
            .expect("PGLib fixed-injection DC evaluation should solve and audit")
    }

    #[test]
    fn ieee_14_bus_projection_solves_and_receipt_is_independently_audited() {
        let result = evaluate(CASE14);
        assert_eq!(
            result.source_git_blob_sha1,
            "2c8260fff0311c902c3d36dcabcbe861576fc97c"
        );
        assert_eq!(result.source_function_name, "pglib_opf_case14_ieee");
        assert_eq!(result.network.buses.len(), 14);
        assert_eq!(result.network.branches.len(), 20);
        assert_eq!(result.audit.checked_bus_count, 14);
        assert_eq!(result.audit.checked_branch_count, 20);
        assert!(result.audit.within_nodal_tolerance);
        assert!(result.audit.max_abs_nodal_residual_mw <= 1.0e-7);
    }

    #[test]
    fn ieee_rts_24_bus_projection_solves_and_receipt_is_independently_audited() {
        let result = evaluate(CASE24);
        assert_eq!(
            result.source_git_blob_sha1,
            "00b024c2e583b82452392514d2c22881c708ba7c"
        );
        assert_eq!(result.source_function_name, "pglib_opf_case24_ieee_rts");
        assert_eq!(result.network.buses.len(), 24);
        assert_eq!(result.network.branches.len(), 38);
        assert_eq!(result.audit.checked_bus_count, 24);
        assert_eq!(result.audit.checked_branch_count, 38);
        assert!(result.audit.within_nodal_tolerance);
        assert!(result.audit.max_abs_nodal_residual_mw <= 1.0e-7);
    }

    #[test]
    fn ieee_57_bus_projection_solves_and_receipt_is_independently_audited() {
        let result = evaluate(CASE57);
        assert_eq!(
            result.source_git_blob_sha1,
            "d8b0c9e8b45265e57f52b7ab96880ad3a874cde5"
        );
        assert_eq!(result.source_function_name, "pglib_opf_case57_ieee");
        assert_eq!(result.network.buses.len(), 57);
        assert_eq!(result.network.branches.len(), 80);
        assert_eq!(result.audit.checked_bus_count, 57);
        assert_eq!(result.audit.checked_branch_count, 80);
        assert!(result.audit.within_nodal_tolerance);
        assert!(result.audit.max_abs_nodal_residual_mw <= 1.0e-7);
    }

    #[test]
    fn exact_source_byte_change_changes_receipt_identity() {
        let original = evaluate(CASE14);
        let changed_source = format!("{CASE14}\n% provenance-only byte change\n");
        let changed = evaluate(&changed_source);
        assert_ne!(original.source_git_blob_sha1, changed.source_git_blob_sha1);
        assert_eq!(original.network.buses.len(), changed.network.buses.len());
    }

    #[test]
    fn unsupported_statement_fails_before_solver_execution() {
        let source = format!("{CASE14}\nmpc.unsupported = 1;\n");
        let error = evaluate_matpower_dc_case(&source, SolveOptions::default())
            .expect_err("unknown MATPOWER extension must fail closed");
        assert_eq!(error.stage, "matpower_projection");
        assert!(error.reason.contains("unsupported statement"));
    }

    fn sweep(source: &str) -> MatpowerDcContingencyEvaluation {
        sweep_matpower_dc_case(source, SolveOptions::default(), 256)
            .expect("PGLib fixed-injection DC contingency sweep should complete")
    }

    fn assert_complete_audited_panel(result: &MatpowerDcContingencyEvaluation, branches: usize) {
        assert!(result.sweep.base_case.disposition.is_audited());
        assert_eq!(result.sweep.branch_outages.len(), branches);
        assert_eq!(result.sweep.summary.expected_branch_outages, branches);
        assert_eq!(result.sweep.summary.evaluated_branch_outages, branches);
        assert!(result.sweep.summary.coverage_complete);
        assert_eq!(result.sweep.summary.auditor_rejections, 0);
        assert_eq!(
            result.sweep.summary.audited_within_declared_checks
                + result.sweep.summary.audited_with_declared_violations
                + result.sweep.summary.solver_unknowns
                + result.sweep.summary.auditor_rejections,
            branches
        );
    }

    #[test]
    fn ieee_14_bus_runs_complete_independently_audited_branch_outage_panel() {
        let result = sweep(CASE14);
        assert_eq!(
            result.source_git_blob_sha1,
            "2c8260fff0311c902c3d36dcabcbe861576fc97c"
        );
        assert_complete_audited_panel(&result, 20);
    }

    #[test]
    fn ieee_rts_24_bus_runs_complete_independently_audited_branch_outage_panel() {
        let result = sweep(CASE24);
        assert_eq!(
            result.source_git_blob_sha1,
            "00b024c2e583b82452392514d2c22881c708ba7c"
        );
        assert_complete_audited_panel(&result, 38);
    }

    #[test]
    fn ieee_57_bus_runs_complete_independently_audited_branch_outage_panel() {
        let result = sweep(CASE57);
        assert_eq!(
            result.source_git_blob_sha1,
            "d8b0c9e8b45265e57f52b7ab96880ad3a874cde5"
        );
        assert_complete_audited_panel(&result, 80);
    }

    #[test]
    fn contingency_budget_failure_returns_no_partial_evaluation() {
        let error = sweep_matpower_dc_case(CASE14, SolveOptions::default(), 19)
            .expect_err("20 in-service branches exceed a 19-case budget");
        assert_eq!(error.stage, "contingency_sweep");
        assert!(error.reason.contains("configured budget is 19"));
    }

    fn assert_lodf_screen_matches_exact_outage_resolves(source: &str, branches: usize) {
        let evaluated = evaluate(source);
        let screen = screen_single_branch_outages(
            &evaluated.network,
            SolveOptions::default(),
            ScreenOptions::default(),
        )
        .expect("PGLib DC network can be LODF-screened");
        assert_eq!(screen.summary.expected_branch_outages, branches);
        assert_eq!(screen.summary.evaluated_branch_outages, branches);
        assert!(screen.summary.coverage_complete);

        let validation =
            compare_screen_to_exact(&evaluated.network, &screen, SolveOptions::default(), 1.0e-6)
                .expect("LODF screen must reconcile with exact fixed-injection DC re-solves");
        assert_eq!(validation.expected_branch_outages, branches);
        assert_eq!(validation.evaluated_branch_outages, branches);
        assert!(validation.coverage_complete);
        assert_eq!(
            validation.compared_predictions + validation.exact_resolve_abstentions,
            branches
        );
        assert_eq!(
            validation.flow_mismatch_count,
            0,
            "{}",
            validation.to_json()
        );
        assert_eq!(
            validation.false_safe_branch_count,
            0,
            "{}",
            validation.to_json()
        );
        assert_eq!(
            validation.false_safe_outage_count,
            0,
            "{}",
            validation.to_json()
        );
        assert_eq!(
            validation.unexpected_exact_unknown_count,
            0,
            "{}",
            validation.to_json()
        );
        assert!(validation.max_abs_flow_error_mw <= 1.0e-6);
        assert!(
            validation.passes_declared_checks(),
            "{}",
            validation.to_json()
        );
    }

    #[test]
    fn ieee_14_lodf_screen_matches_all_resolvable_exact_outage_solves() {
        assert_lodf_screen_matches_exact_outage_resolves(CASE14, 20);
    }

    #[test]
    fn ieee_rts_24_lodf_screen_matches_all_resolvable_exact_outage_solves() {
        assert_lodf_screen_matches_exact_outage_resolves(CASE24, 38);
    }

    #[test]
    fn ieee_57_lodf_screen_matches_all_resolvable_exact_outage_solves() {
        assert_lodf_screen_matches_exact_outage_resolves(CASE57, 80);
    }

    #[test]
    fn matpower_lodf_screen_receipt_binds_source_identity_and_coverage() {
        let evaluation =
            screen_matpower_dc_case(CASE14, SolveOptions::default(), ScreenOptions::default())
                .expect("MATPOWER case can be screened");
        assert_eq!(
            evaluation.source_git_blob_sha1,
            "2c8260fff0311c902c3d36dcabcbe861576fc97c"
        );
        assert_eq!(evaluation.screen.summary.expected_branch_outages, 20);
        assert!(evaluation.screen.summary.coverage_complete);
        let json = evaluation.to_json();
        assert!(json.contains("\"matpower-dc-lodf-screen-evaluation.v0.1.0\""));
        assert!(json.contains("\"dc-lodf-outage-screen.v0.1.0\""));
    }

    #[test]
    fn evaluation_json_is_parseable_and_contains_all_branch_flows() {
        let result = evaluate(CASE14);
        let json = result.to_json();
        assert!(json.starts_with("{"));
        assert!(json.ends_with("}"));
        assert!(json.contains("\"branch_flows\":["));
        assert!(json.contains("\"source_git_blob_sha1\":"));
        assert_eq!(result.solution.branch_flows.len(), 20);
        assert_eq!(result.solution.bus_states.len(), 14);
    }
}
